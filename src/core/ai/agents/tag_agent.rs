use super::*;
use crate::core::ai::provider::{AIProvider, ProviderConfig};
use async_trait::async_trait;
use semver::{Version, Prerelease};
use std::sync::Arc;
use std::time::Instant;

/// 标签 Agent - 负责生成版本标签和发布说明
pub struct TagAgent {
    name: String,
    description: String,
    provider: Option<Arc<dyn AIProvider>>,
    status: AgentStatus,
    config: AgentConfig,
}

impl TagAgent {
    /// 创建新的 TagAgent
    pub fn new() -> Self {
        Self {
            name: "TagAgent".to_string(),
            description: "智能生成版本标签和发布说明".to_string(),
            provider: None,
            status: AgentStatus::Uninitialized,
            config: AgentConfig::default(),
        }
    }
    
    /// 分析提交历史，推断版本类型
    fn analyze_commits(&self, commits: &str) -> VersionBump {
        let mut has_breaking = false;
        let mut has_feature = false;
        let mut has_fix = false;
        
        for line in commits.lines() {
            let lower = line.to_lowercase();
            
            // 检查破坏性变更
            if lower.contains("breaking") || lower.contains("!:") {
                has_breaking = true;
            }
            // 检查新功能
            else if lower.starts_with("feat") {
                has_feature = true;
            }
            // 检查修复
            else if lower.starts_with("fix") {
                has_fix = true;
            }
        }
        
        if has_breaking {
            VersionBump::Major
        } else if has_feature {
            VersionBump::Minor
        } else if has_fix {
            VersionBump::Patch
        } else {
            VersionBump::Patch // 默认补丁版本
        }
    }
    
    /// 生成下一个版本号
    fn generate_next_version(&self, current: &str, bump: VersionBump) -> Result<String> {
        let mut version = Version::parse(current)
            .map_err(|e| anyhow::anyhow!("Invalid version format: {}", e))?;
        
        match bump {
            VersionBump::Major => {
                version.major += 1;
                version.minor = 0;
                version.patch = 0;
                version.pre = Prerelease::EMPTY;
            }
            VersionBump::Minor => {
                version.minor += 1;
                version.patch = 0;
                version.pre = Prerelease::EMPTY;
            }
            VersionBump::Patch => {
                version.patch += 1;
                version.pre = Prerelease::EMPTY;
            }
            VersionBump::Prerelease(ref label) => {
                // 处理预发布版本
                let pre_str = if version.pre.is_empty() {
                    format!("{}.1", label)
                } else {
                    // 递增预发布版本号
                    let parts: Vec<&str> = version.pre.as_str().split('.').collect();
                    if parts.len() >= 2 {
                        if let Ok(num) = parts[1].parse::<u32>() {
                            format!("{}.{}", label, num + 1)
                        } else {
                            format!("{}.1", label)
                        }
                    } else {
                        format!("{}.1", label)
                    }
                };
                version.pre = Prerelease::new(&pre_str)
                    .map_err(|e| anyhow::anyhow!("Invalid prerelease: {}", e))?;
            }
        }
        
        Ok(version.to_string())
    }
    
    /// 生成发布说明
    async fn generate_release_notes(
        &self,
        version: &str,
        commits: &str,
        context: &AgentContext,
    ) -> Result<String> {
        let provider = self.provider.as_ref()
            .ok_or_else(|| anyhow::anyhow!("AI provider not initialized"))?;
        
        // 构建发布说明提示词
        let prompt = self.build_release_prompt(version, commits)?;
        
        // 调用 AI 生成
        let provider_config = ProviderConfig {
            model: context.config.model.clone(),
            api_key: context.env_vars.get("API_KEY").cloned(),
            api_url: context.env_vars.get("API_URL")
                .unwrap_or(&"http://localhost:11434".to_string())
                .clone(),
            timeout_secs: context.config.timeout_secs,
            max_retries: context.config.max_retries,
            stream: context.config.stream,
        };
        
        let response = provider.generate(&prompt, &provider_config).await?;
        
        // 格式化发布说明
        Ok(self.format_release_notes(version, &response))
    }
    
    /// 构建发布说明提示词
    fn build_release_prompt(&self, version: &str, commits: &str) -> Result<String> {
        let mut prompt = String::new();
        
        prompt.push_str("你是一个专业的软件发布说明生成助手。\n");
        prompt.push_str(&format!("请为版本 {} 生成发布说明。\n\n", version));
        
        prompt.push_str("要求：\n");
        prompt.push_str("1. 使用中文\n");
        prompt.push_str("2. 按类别组织（新功能、改进、修复、其他）\n");
        prompt.push_str("3. 简洁明了，突出重点\n");
        prompt.push_str("4. 每项使用 - 开头\n");
        prompt.push_str("5. 重要变更加粗或使用 emoji\n\n");
        
        prompt.push_str("提交历史：\n");
        prompt.push_str(commits);
        prompt.push_str("\n\n请直接输出发布说明内容：");
        
        Ok(prompt)
    }
    
    /// 格式化发布说明
    fn format_release_notes(&self, version: &str, notes: &str) -> String {
        let mut formatted = String::new();
        
        // 添加版本标题
        formatted.push_str(&format!("# Release v{}\n\n", version));
        formatted.push_str(&format!("发布日期：{}\n\n", chrono::Local::now().format("%Y-%m-%d")));
        
        // 添加内容
        formatted.push_str(notes);
        
        // 添加尾部信息
        formatted.push_str("\n\n---\n");
        formatted.push_str("*此发布说明由 AI-Commit TagAgent 自动生成*");
        
        formatted
    }
    
    /// 生成变更日志
    async fn generate_changelog(
        &self,
        from_version: Option<&str>,
        to_version: &str,
        commits: &str,
        context: &AgentContext,
    ) -> Result<String> {
        let provider = self.provider.as_ref()
            .ok_or_else(|| anyhow::anyhow!("AI provider not initialized"))?;
        
        let prompt = self.build_changelog_prompt(from_version, to_version, commits)?;
        
        let provider_config = ProviderConfig {
            model: context.config.model.clone(),
            api_key: context.env_vars.get("API_KEY").cloned(),
            api_url: context.env_vars.get("API_URL")
                .unwrap_or(&"http://localhost:11434".to_string())
                .clone(),
            timeout_secs: context.config.timeout_secs,
            max_retries: context.config.max_retries,
            stream: context.config.stream,
        };
        
        provider.generate(&prompt, &provider_config).await
    }
    
    /// 构建变更日志提示词
    fn build_changelog_prompt(
        &self,
        from_version: Option<&str>,
        to_version: &str,
        commits: &str,
    ) -> Result<String> {
        let mut prompt = String::new();
        
        prompt.push_str("生成 CHANGELOG 条目，遵循 Keep a Changelog 格式。\n\n");
        
        if let Some(from) = from_version {
            prompt.push_str(&format!("版本范围：{} -> {}\n\n", from, to_version));
        } else {
            prompt.push_str(&format!("版本：{}\n\n", to_version));
        }
        
        prompt.push_str("格式要求：\n");
        prompt.push_str("## [版本号] - 日期\n");
        prompt.push_str("### Added\n");
        prompt.push_str("### Changed\n");
        prompt.push_str("### Deprecated\n");
        prompt.push_str("### Removed\n");
        prompt.push_str("### Fixed\n");
        prompt.push_str("### Security\n\n");
        
        prompt.push_str("提交记录：\n");
        prompt.push_str(commits);
        
        Ok(prompt)
    }
}

#[async_trait]
impl Agent for TagAgent {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn description(&self) -> &str {
        &self.description
    }
    
    fn capabilities(&self) -> Vec<AgentCapability> {
        vec![
            AgentCapability::GenerateTag,
            AgentCapability::GenerateDoc,
            AgentCapability::AnalyzeCode,
        ]
    }
    
    async fn initialize(&mut self, context: &AgentContext) -> Result<()> {
        use crate::core::ai::provider::ProviderFactory;
        
        let provider = ProviderFactory::create(&context.config.provider)?;
        self.provider = Some(Arc::from(provider));
        self.config = context.config.clone();
        self.status = AgentStatus::Ready;
        
        Ok(())
    }
    
    async fn execute(&self, task: AgentTask, context: &AgentContext) -> Result<AgentResult> {
        self.validate_task(&task)?;
        
        let start_time = Instant::now();
        
        let result = match task.task_type {
            TaskType::GenerateTag => {
                // 获取参数
                let current_version = task.params.get("current_version")
                    .ok_or_else(|| anyhow::anyhow!("Missing current_version parameter"))?;
                
                let commits = &task.input;
                
                // 分析提交，推断版本号
                let bump = self.analyze_commits(commits);
                let next_version = self.generate_next_version(current_version, bump)?;
                
                // 生成发布说明
                let release_notes = self.generate_release_notes(&next_version, commits, context).await?;
                
                let mut data = HashMap::new();
                data.insert("version".to_string(), serde_json::Value::String(next_version.clone()));
                data.insert("release_notes".to_string(), serde_json::Value::String(release_notes.clone()));
                
                AgentResult {
                    success: true,
                    content: format!("v{}\n\n{}", next_version, release_notes),
                    duration_ms: start_time.elapsed().as_millis() as u64,
                    tokens_used: None,
                    data,
                }
            }
            TaskType::GenerateDocumentation => {
                // 生成变更日志
                let from_version = task.params.get("from_version").map(|s| s.as_str());
                let to_version = task.params.get("to_version")
                    .ok_or_else(|| anyhow::anyhow!("Missing to_version parameter"))?;
                
                let changelog = self.generate_changelog(
                    from_version,
                    to_version,
                    &task.input,
                    context
                ).await?;
                
                AgentResult {
                    success: true,
                    content: changelog,
                    duration_ms: start_time.elapsed().as_millis() as u64,
                    tokens_used: None,
                    data: HashMap::new(),
                }
            }
            _ => {
                anyhow::bail!("Unsupported task type: {:?}", task.task_type);
            }
        };
        
        Ok(result)
    }
    
    fn status(&self) -> AgentStatus {
        self.status.clone()
    }
}

/// 版本升级类型
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
enum VersionBump {
    Major,
    Minor,
    Patch,
    Prerelease(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_tag_agent_creation() {
        let agent = TagAgent::new();
        assert_eq!(agent.name(), "TagAgent");
        assert!(agent.description().contains("版本标签"));
    }
    
    #[test]
    fn test_tag_agent_capabilities() {
        let agent = TagAgent::new();
        let caps = agent.capabilities();
        assert!(caps.contains(&AgentCapability::GenerateTag));
        assert!(caps.contains(&AgentCapability::GenerateDoc));
    }
    
    #[test]
    fn test_analyze_commits() {
        let agent = TagAgent::new();
        
        // 测试破坏性变更
        assert_eq!(
            agent.analyze_commits("feat!: breaking change\nfix: minor fix"),
            VersionBump::Major
        );
        
        // 测试新功能
        assert_eq!(
            agent.analyze_commits("feat: new feature\nfix: bug fix"),
            VersionBump::Minor
        );
        
        // 测试修复
        assert_eq!(
            agent.analyze_commits("fix: bug fix\nchore: update deps"),
            VersionBump::Patch
        );
    }
    
    #[test]
    fn test_generate_next_version() {
        let agent = TagAgent::new();
        
        // 测试主版本升级
        assert_eq!(
            agent.generate_next_version("1.2.3", VersionBump::Major).unwrap(),
            "2.0.0"
        );
        
        // 测试次版本升级
        assert_eq!(
            agent.generate_next_version("1.2.3", VersionBump::Minor).unwrap(),
            "1.3.0"
        );
        
        // 测试补丁版本升级
        assert_eq!(
            agent.generate_next_version("1.2.3", VersionBump::Patch).unwrap(),
            "1.2.4"
        );
        
        // 测试预发布版本
        assert_eq!(
            agent.generate_next_version("1.2.3", VersionBump::Prerelease("beta".to_string())).unwrap(),
            "1.2.3-beta.1"
        );
    }
    
    #[test]
    fn test_format_release_notes() {
        let agent = TagAgent::new();
        
        let notes = "### 新功能\n- 添加了 AI 支持";
        let formatted = agent.format_release_notes("1.2.0", notes);
        
        assert!(formatted.contains("# Release v1.2.0"));
        assert!(formatted.contains("发布日期"));
        assert!(formatted.contains("新功能"));
        assert!(formatted.contains("AI-Commit TagAgent"));
    }
}