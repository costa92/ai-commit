use super::*;
use crate::core::ai::provider::{AIProvider, ProviderConfig};
use async_trait::async_trait;
use once_cell::sync::Lazy;
use regex::Regex;
use std::sync::Arc;
use std::time::Instant;

/// 提交消息验证正则表达式
static COMMIT_FORMAT_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(feat|fix|docs|style|refactor|test|chore|perf|ci|build|revert)(\([^)]+\))?:\s+.+")
        .unwrap()
});

/// 提交消息 Agent
pub struct CommitAgent {
    name: String,
    description: String,
    provider: Option<Arc<dyn AIProvider>>,
    status: AgentStatus,
    config: AgentConfig,
}

impl CommitAgent {
    /// 创建新的 CommitAgent
    pub fn new() -> Self {
        Self {
            name: "CommitAgent".to_string(),
            description: "智能生成符合 Conventional Commits 规范的提交消息".to_string(),
            provider: None,
            status: AgentStatus::Uninitialized,
            config: AgentConfig::default(),
        }
    }
    
    /// 分析 diff 内容
    fn analyze_diff(&self, diff: &str) -> DiffAnalysis {
        let mut analysis = DiffAnalysis::default();
        
        // 统计文件变更
        for line in diff.lines() {
            if line.starts_with("+++") || line.starts_with("---") {
                analysis.files_changed += 1;
            } else if line.starts_with('+') && !line.starts_with("+++") {
                analysis.lines_added += 1;
            } else if line.starts_with('-') && !line.starts_with("---") {
                analysis.lines_deleted += 1;
            }
        }
        
        // 推断变更类型
        analysis.change_type = self.infer_change_type(diff);
        analysis.scope = self.extract_scope(diff);
        
        analysis
    }
    
    /// 推断变更类型
    fn infer_change_type(&self, diff: &str) -> String {
        let lower_diff = diff.to_lowercase();
        
        if lower_diff.contains("test") || lower_diff.contains("spec") {
            "test".to_string()
        } else if lower_diff.contains("readme") || lower_diff.contains(".md") {
            "docs".to_string()
        } else if lower_diff.contains("fix") || lower_diff.contains("bug") {
            "fix".to_string()
        } else if lower_diff.contains("feat") || lower_diff.contains("add") {
            "feat".to_string()
        } else if lower_diff.contains("refactor") || lower_diff.contains("optimize") {
            "refactor".to_string()
        } else if lower_diff.contains("style") || lower_diff.contains("format") {
            "style".to_string()
        } else {
            "chore".to_string()
        }
    }
    
    /// 提取作用域
    fn extract_scope(&self, diff: &str) -> Option<String> {
        // 从文件路径提取作用域
        for line in diff.lines() {
            if line.starts_with("+++") || line.starts_with("---") {
                if let Some(path) = line.split('/').nth(1) {
                    if let Some(name) = path.split('.').next() {
                        return Some(name.to_string());
                    }
                }
            }
        }
        None
    }
    
    /// 生成提交消息
    async fn generate_commit_message(
        &self,
        diff: &str,
        context: &AgentContext,
    ) -> Result<String> {
        let provider = self.provider.as_ref()
            .ok_or_else(|| anyhow::anyhow!("AI provider not initialized"))?;
        
        // 分析 diff
        let analysis = self.analyze_diff(diff);
        
        // 构建增强的提示词
        let enhanced_prompt = self.build_enhanced_prompt(diff, &analysis)?;
        
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
        
        let response = provider.generate(&enhanced_prompt, &provider_config).await?;
        
        // 先清理响应，再验证格式
        let cleaned_response = self.clean_commit_message(&response);
        self.validate_commit_message(&cleaned_response)?;
        Ok(cleaned_response)
    }
    
    /// 构建增强的提示词
    fn build_enhanced_prompt(&self, diff: &str, analysis: &DiffAnalysis) -> Result<String> {
        let mut prompt = String::new();
        
        prompt.push_str("你必须严格按照 Conventional Commits 规范输出 Git 提交消息。\n\n");
        
        prompt.push_str("输出格式：<type>(<scope>): <subject>\n\n");
        
        prompt.push_str("严格要求：\n");
        prompt.push_str("1. type 必须是：feat, fix, docs, style, refactor, test, chore, perf, ci, build, revert\n");
        prompt.push_str("2. scope 可选，用括号包围，表示影响范围\n");
        prompt.push_str("3. subject 必须是中文，简洁描述变更内容，不超过50字\n");
        prompt.push_str("4. 禁止任何解释、分析或额外文字\n");
        prompt.push_str("5. 只输出一行标准格式的提交消息\n\n");
        
        prompt.push_str("禁止输出的错误格式示例：\n");
        prompt.push_str("- \"添加依赖：test，作用域：Cargo\" ❌\n");
        prompt.push_str("- \"根据分析，这是一个测试变更\" ❌\n");
        prompt.push_str("- \"变更类型：feat\" ❌\n\n");
        
        prompt.push_str("正确格式示例：\n");
        prompt.push_str("- \"feat(api): 添加用户认证功能\" ✅\n");
        prompt.push_str("- \"fix(ui): 修复按钮显示问题\" ✅\n");
        prompt.push_str("- \"test(core): 添加单元测试\" ✅\n\n");
        
        // 提供分析信息作为参考
        prompt.push_str(&format!("变更分析（仅供参考）：\n"));
        prompt.push_str(&format!("- 推荐类型：{}\n", analysis.change_type));
        if let Some(scope) = &analysis.scope {
            prompt.push_str(&format!("- 推荐作用域：{}\n", scope));
        }
        prompt.push_str(&format!("- 文件变更：{} 个\n", analysis.files_changed));
        
        prompt.push_str("\n现在直接输出符合格式的提交消息：\n\n");
        prompt.push_str("Diff 内容：\n");
        
        // 如果 diff 太长，截取关键部分
        if diff.len() > 5000 {
            prompt.push_str(&diff.chars().take(5000).collect::<String>());
            prompt.push_str("\n... (diff 内容已截断)");
        } else {
            prompt.push_str(diff);
        }
        
        Ok(prompt)
    }
    
    /// 验证提交消息格式
    fn validate_commit_message(&self, message: &str) -> Result<()> {
        let first_line = message.lines().next().unwrap_or("");
        
        if !COMMIT_FORMAT_REGEX.is_match(first_line) {
            anyhow::bail!(
                "提交消息格式不正确。期望格式：<type>(<scope>): <subject>\n实际：{}",
                first_line
            );
        }
        
        // 检查长度
        if first_line.chars().count() > 100 {
            anyhow::bail!("提交消息过长（{}字符），应不超过100字符", first_line.chars().count());
        }
        
        Ok(())
    }
    
    /// 清理提交消息
    fn clean_commit_message(&self, message: &str) -> String {
        // 只取第一行，去除多余空白
        message.lines()
            .next()
            .unwrap_or("")
            .trim()
            .to_string()
    }
}

#[async_trait]
impl Agent for CommitAgent {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn description(&self) -> &str {
        &self.description
    }
    
    fn capabilities(&self) -> Vec<AgentCapability> {
        vec![
            AgentCapability::GenerateCommit,
            AgentCapability::AnalyzeCode,
        ]
    }
    
    async fn initialize(&mut self, context: &AgentContext) -> Result<()> {
        // 初始化 AI 提供商
        use crate::core::ai::provider::ProviderFactory;
        
        let provider = ProviderFactory::create(&context.config.provider)?;
        self.provider = Some(Arc::from(provider));
        self.config = context.config.clone();
        self.status = AgentStatus::Ready;
        
        Ok(())
    }
    
    async fn execute(&self, task: AgentTask, context: &AgentContext) -> Result<AgentResult> {
        // 验证任务
        self.validate_task(&task)?;
        
        let start_time = Instant::now();
        
        let result = match task.task_type {
            TaskType::GenerateCommit => {
                // 生成提交消息
                let message = self.generate_commit_message(&task.input, context).await?;
                
                AgentResult {
                    success: true,
                    content: message,
                    duration_ms: start_time.elapsed().as_millis() as u64,
                    tokens_used: None, // TODO: 从 provider 获取 token 使用量
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

/// Diff 分析结果
#[derive(Debug, Default)]
struct DiffAnalysis {
    files_changed: usize,
    lines_added: usize,
    lines_deleted: usize,
    change_type: String,
    scope: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_commit_agent_creation() {
        let agent = CommitAgent::new();
        assert_eq!(agent.name(), "CommitAgent");
        assert!(agent.description().contains("Conventional Commits"));
    }
    
    #[test]
    fn test_commit_agent_capabilities() {
        let agent = CommitAgent::new();
        let caps = agent.capabilities();
        assert!(caps.contains(&AgentCapability::GenerateCommit));
        assert!(caps.contains(&AgentCapability::AnalyzeCode));
    }
    
    #[test]
    fn test_validate_commit_message() {
        let agent = CommitAgent::new();
        
        // 有效格式
        assert!(agent.validate_commit_message("feat(api): 添加用户认证").is_ok());
        assert!(agent.validate_commit_message("fix: 修复登录问题").is_ok());
        assert!(agent.validate_commit_message("docs(readme): 更新文档").is_ok());
        
        // 无效格式
        assert!(agent.validate_commit_message("invalid message").is_err());
        assert!(agent.validate_commit_message("feat 缺少冒号").is_err());
        assert!(agent.validate_commit_message("").is_err());
    }
    
    #[test]
    fn test_clean_commit_message() {
        let agent = CommitAgent::new();
        
        let message = "  feat(api): 添加功能  \n额外的行\n更多内容";
        assert_eq!(agent.clean_commit_message(message), "feat(api): 添加功能");
    }
    
    #[test]
    fn test_infer_change_type() {
        let agent = CommitAgent::new();
        
        assert_eq!(agent.infer_change_type("test_file.rs"), "test");
        assert_eq!(agent.infer_change_type("README.md"), "docs");
        assert_eq!(agent.infer_change_type("fix bug in login"), "fix");
        assert_eq!(agent.infer_change_type("add new feature"), "feat");
        assert_eq!(agent.infer_change_type("refactor code"), "refactor");
        assert_eq!(agent.infer_change_type("format code"), "style");
        assert_eq!(agent.infer_change_type("random changes"), "chore");
    }
    
    #[tokio::test]
    async fn test_commit_agent_task_validation() {
        let agent = CommitAgent::new();
        
        let valid_task = AgentTask::new(TaskType::GenerateCommit, "diff content");
        assert!(agent.validate_task(&valid_task).is_ok());
        
        let invalid_task = AgentTask::new(TaskType::GenerateCommit, "");
        assert!(agent.validate_task(&invalid_task).is_err());
    }
}