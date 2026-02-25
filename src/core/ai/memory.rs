use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// 项目提交约定
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CommitConventions {
    /// commit type 使用频率
    pub type_distribution: HashMap<String, u32>,
    /// scope 使用频率
    pub scope_distribution: HashMap<String, u32>,
    /// 主要提交语言 ("zh" / "en")
    pub language: String,
    /// 总 commit 数量 (用于统计)
    pub total_commits_analyzed: u32,
}

/// 用户修正记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitCorrection {
    /// 时间戳
    pub timestamp: String,
    /// AI 生成的原始消息
    pub ai_generated: String,
    /// 用户修正后的消息
    pub user_corrected: String,
    /// 修正的变更类型 (e.g., type/scope/subject changed)
    pub correction_type: Vec<String>,
}

/// 项目记忆
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProjectMemory {
    /// 提交约定
    pub conventions: CommitConventions,
    /// 最近的用户修正 (保留最多 20 条)
    pub corrections: Vec<CommitCorrection>,
    /// 最后更新时间
    pub last_updated: String,
}

impl ProjectMemory {
    const MAX_CORRECTIONS: usize = 20;

    /// 获取项目记忆存储目录
    pub fn memory_dir(project_path: &Path) -> Result<PathBuf> {
        let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Cannot find home directory"))?;
        let hash = compute_project_hash(project_path);
        let dir = home.join(".ai-commit").join("memory").join(hash);
        Ok(dir)
    }

    /// 从磁盘加载项目记忆
    pub fn load(project_path: &Path) -> Result<Self> {
        let dir = Self::memory_dir(project_path)?;
        let file = dir.join("memory.json");

        if !file.exists() {
            return Ok(Self::default());
        }

        let content = std::fs::read_to_string(&file)?;
        let memory: ProjectMemory = serde_json::from_str(&content)?;
        Ok(memory)
    }

    /// 保存项目记忆到磁盘
    pub fn save(&self, project_path: &Path) -> Result<()> {
        let dir = Self::memory_dir(project_path)?;
        std::fs::create_dir_all(&dir)?;

        let file = dir.join("memory.json");
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(file, content)?;
        Ok(())
    }

    /// 记录一次用户修正
    pub fn record_correction(&mut self, ai_generated: &str, user_corrected: &str) {
        if ai_generated == user_corrected {
            return; // 没有修正
        }

        let correction_type = detect_correction_type(ai_generated, user_corrected);

        self.corrections.push(CommitCorrection {
            timestamp: chrono::Utc::now().to_rfc3339(),
            ai_generated: ai_generated.to_string(),
            user_corrected: user_corrected.to_string(),
            correction_type,
        });

        // 保留最近 N 条
        if self.corrections.len() > Self::MAX_CORRECTIONS {
            let drain_count = self.corrections.len() - Self::MAX_CORRECTIONS;
            self.corrections.drain(..drain_count);
        }

        self.last_updated = chrono::Utc::now().to_rfc3339();
    }

    /// 记录一次成功的 commit (更新约定统计)
    pub fn record_commit(&mut self, message: &str) {
        if let Some((commit_type, scope)) = parse_commit_parts(message) {
            *self
                .conventions
                .type_distribution
                .entry(commit_type)
                .or_insert(0) += 1;

            if let Some(s) = scope {
                *self.conventions.scope_distribution.entry(s).or_insert(0) += 1;
            }

            self.conventions.total_commits_analyzed += 1;
        }

        self.last_updated = chrono::Utc::now().to_rfc3339();
    }

    /// 从 git log 初始化约定数据
    pub async fn initialize_from_git_log(&mut self) -> Result<()> {
        let output = tokio::process::Command::new("git")
            .args(["log", "-50", "--pretty=format:%s"])
            .output()
            .await?;

        if !output.status.success() {
            return Ok(()); // 静默失败
        }

        let log = String::from_utf8_lossy(&output.stdout);
        let mut zh_count = 0u32;
        let mut en_count = 0u32;

        for line in log.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            if let Some((commit_type, scope)) = parse_commit_parts(line) {
                *self
                    .conventions
                    .type_distribution
                    .entry(commit_type)
                    .or_insert(0) += 1;

                if let Some(s) = scope {
                    *self.conventions.scope_distribution.entry(s).or_insert(0) += 1;
                }

                self.conventions.total_commits_analyzed += 1;
            }

            // 检测语言
            if contains_cjk(line) {
                zh_count += 1;
            } else {
                en_count += 1;
            }
        }

        self.conventions.language = if zh_count > en_count {
            "zh".to_string()
        } else {
            "en".to_string()
        };

        self.last_updated = chrono::Utc::now().to_rfc3339();
        Ok(())
    }

    /// 生成用于 prompt 注入的上下文文本
    pub fn to_prompt_context(&self) -> String {
        let mut parts = Vec::new();

        // 常用 type
        if !self.conventions.type_distribution.is_empty() {
            let mut types: Vec<_> = self.conventions.type_distribution.iter().collect();
            types.sort_by(|a, b| b.1.cmp(a.1));
            let top_types: Vec<String> = types
                .iter()
                .take(5)
                .map(|(t, c)| format!("{}({})", t, c))
                .collect();
            parts.push(format!("常用类型: {}", top_types.join(", ")));
        }

        // 常用 scope
        if !self.conventions.scope_distribution.is_empty() {
            let mut scopes: Vec<_> = self.conventions.scope_distribution.iter().collect();
            scopes.sort_by(|a, b| b.1.cmp(a.1));
            let top_scopes: Vec<String> = scopes
                .iter()
                .take(5)
                .map(|(s, c)| format!("{}({})", s, c))
                .collect();
            parts.push(format!("常用 scope: {}", top_scopes.join(", ")));
        }

        // 语言偏好
        if !self.conventions.language.is_empty() {
            let lang = if self.conventions.language == "zh" {
                "中文"
            } else {
                "English"
            };
            parts.push(format!("语言偏好: {}", lang));
        }

        // 最近的修正示例 (最多 3 条)
        if !self.corrections.is_empty() {
            parts.push("最近修正:".to_string());
            for correction in self.corrections.iter().rev().take(3) {
                parts.push(format!(
                    "  AI: {} → 用户: {}",
                    correction.ai_generated, correction.user_corrected
                ));
            }
        }

        if parts.is_empty() {
            String::new()
        } else {
            format!("\n项目上下文:\n{}\n", parts.join("\n"))
        }
    }

    /// 重置项目记忆
    pub fn reset(project_path: &Path) -> Result<()> {
        let dir = Self::memory_dir(project_path)?;
        if dir.exists() {
            std::fs::remove_dir_all(&dir)?;
        }
        Ok(())
    }

    /// 显示项目记忆摘要
    pub fn display_summary(&self) -> String {
        let mut lines = Vec::new();

        lines.push(format!(
            "Total commits analyzed: {}",
            self.conventions.total_commits_analyzed
        ));

        if !self.conventions.language.is_empty() {
            lines.push(format!("Language: {}", self.conventions.language));
        }

        if !self.conventions.type_distribution.is_empty() {
            lines.push("Type distribution:".to_string());
            let mut types: Vec<_> = self.conventions.type_distribution.iter().collect();
            types.sort_by(|a, b| b.1.cmp(a.1));
            for (t, c) in types {
                lines.push(format!("  {}: {}", t, c));
            }
        }

        if !self.conventions.scope_distribution.is_empty() {
            lines.push("Scope distribution:".to_string());
            let mut scopes: Vec<_> = self.conventions.scope_distribution.iter().collect();
            scopes.sort_by(|a, b| b.1.cmp(a.1));
            for (s, c) in scopes.iter().take(10) {
                lines.push(format!("  {}: {}", s, c));
            }
        }

        lines.push(format!("Corrections recorded: {}", self.corrections.len()));

        if !self.last_updated.is_empty() {
            lines.push(format!("Last updated: {}", self.last_updated));
        }

        lines.join("\n")
    }
}

/// 计算项目路径的短 hash
fn compute_project_hash(path: &Path) -> String {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    path.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

/// 解析 commit message 中的 type 和 scope
fn parse_commit_parts(message: &str) -> Option<(String, Option<String>)> {
    let message = message.trim();

    // 匹配 type(scope): subject 或 type: subject
    let re = regex::Regex::new(
        r"^(feat|fix|docs|style|refactor|test|chore|perf|ci|build|revert)(?:\(([^)]+)\))?:\s+",
    )
    .ok()?;

    let caps = re.captures(message)?;
    let commit_type = caps.get(1)?.as_str().to_string();
    let scope = caps.get(2).map(|m| m.as_str().to_string());

    Some((commit_type, scope))
}

/// 检测 commit message 中是否包含 CJK 字符
fn contains_cjk(text: &str) -> bool {
    text.chars().any(|c| {
        ('\u{4E00}'..='\u{9FFF}').contains(&c) // CJK Unified Ideographs
            || ('\u{3400}'..='\u{4DBF}').contains(&c) // CJK Unified Ideographs Extension A
    })
}

/// 检测修正类型
fn detect_correction_type(original: &str, corrected: &str) -> Vec<String> {
    let mut types = Vec::new();

    let orig_parts = parse_commit_parts(original);
    let corr_parts = parse_commit_parts(corrected);

    match (orig_parts, corr_parts) {
        (Some((ot, os)), Some((ct, cs))) => {
            if ot != ct {
                types.push("type".to_string());
            }
            if os != cs {
                types.push("scope".to_string());
            }
            // subject 总是可能不同
            types.push("subject".to_string());
        }
        _ => {
            types.push("format".to_string());
        }
    }

    types
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_commit_parts() {
        assert_eq!(
            parse_commit_parts("feat(api): add user auth"),
            Some(("feat".to_string(), Some("api".to_string())))
        );

        assert_eq!(
            parse_commit_parts("fix: resolve null pointer"),
            Some(("fix".to_string(), None))
        );

        assert_eq!(parse_commit_parts("random text"), None);
        assert_eq!(parse_commit_parts(""), None);
    }

    #[test]
    fn test_contains_cjk() {
        assert!(contains_cjk("feat(api): 添加用户认证"));
        assert!(!contains_cjk("feat(api): add user auth"));
        assert!(contains_cjk("混合 mixed 文本"));
    }

    #[test]
    fn test_compute_project_hash() {
        let hash1 = compute_project_hash(Path::new("/foo/bar"));
        let hash2 = compute_project_hash(Path::new("/foo/baz"));
        assert_ne!(hash1, hash2);
        assert_eq!(hash1.len(), 16);
    }

    #[test]
    fn test_record_correction() {
        let mut memory = ProjectMemory::default();
        memory.record_correction("feat(api): add endpoint", "feat(auth): add login endpoint");

        assert_eq!(memory.corrections.len(), 1);
        assert_eq!(
            memory.corrections[0].ai_generated,
            "feat(api): add endpoint"
        );
        assert_eq!(
            memory.corrections[0].user_corrected,
            "feat(auth): add login endpoint"
        );
    }

    #[test]
    fn test_record_correction_no_change() {
        let mut memory = ProjectMemory::default();
        memory.record_correction("feat: same message", "feat: same message");
        assert!(memory.corrections.is_empty());
    }

    #[test]
    fn test_record_correction_max_limit() {
        let mut memory = ProjectMemory::default();
        for i in 0..25 {
            memory.record_correction(
                &format!("feat: original {}", i),
                &format!("feat: corrected {}", i),
            );
        }
        assert_eq!(memory.corrections.len(), ProjectMemory::MAX_CORRECTIONS);
    }

    #[test]
    fn test_record_commit() {
        let mut memory = ProjectMemory::default();
        memory.record_commit("feat(api): add endpoint");
        memory.record_commit("feat(ui): add button");
        memory.record_commit("fix(api): fix null");

        assert_eq!(memory.conventions.type_distribution.get("feat"), Some(&2));
        assert_eq!(memory.conventions.type_distribution.get("fix"), Some(&1));
        assert_eq!(memory.conventions.scope_distribution.get("api"), Some(&2));
        assert_eq!(memory.conventions.scope_distribution.get("ui"), Some(&1));
        assert_eq!(memory.conventions.total_commits_analyzed, 3);
    }

    #[test]
    fn test_to_prompt_context_empty() {
        let memory = ProjectMemory::default();
        assert!(memory.to_prompt_context().is_empty());
    }

    #[test]
    fn test_to_prompt_context_with_data() {
        let mut memory = ProjectMemory::default();
        memory.record_commit("feat(api): add endpoint");
        memory.record_commit("fix(api): fix bug");

        let context = memory.to_prompt_context();
        assert!(context.contains("常用类型"));
        assert!(context.contains("常用 scope"));
    }

    #[test]
    fn test_display_summary() {
        let mut memory = ProjectMemory::default();
        memory.record_commit("feat(api): add endpoint");
        memory.record_commit("fix(ui): fix button");

        let summary = memory.display_summary();
        assert!(summary.contains("Total commits analyzed: 2"));
        assert!(summary.contains("feat"));
        assert!(summary.contains("fix"));
    }

    #[test]
    fn test_detect_correction_type() {
        let types = detect_correction_type("feat(api): add endpoint", "fix(api): add endpoint");
        assert!(types.contains(&"type".to_string()));

        let types = detect_correction_type("feat(api): add endpoint", "feat(auth): add endpoint");
        assert!(types.contains(&"scope".to_string()));
    }

    #[test]
    fn test_memory_dir() {
        let dir = ProjectMemory::memory_dir(Path::new("/test/project"));
        assert!(dir.is_ok());
        let dir = dir.unwrap();
        assert!(dir.to_string_lossy().contains(".ai-commit/memory"));
    }

    #[test]
    fn test_save_and_load() {
        let tmp_dir = std::env::temp_dir().join("ai-commit-test-memory");
        let _ = std::fs::remove_dir_all(&tmp_dir);
        std::fs::create_dir_all(&tmp_dir).unwrap();

        let mut memory = ProjectMemory::default();
        memory.record_commit("feat(api): test save");
        memory.record_correction("feat: old", "fix: new");

        // Save to a temp location by writing directly
        let file = tmp_dir.join("memory.json");
        let content = serde_json::to_string_pretty(&memory).unwrap();
        std::fs::write(&file, &content).unwrap();

        // Load back
        let loaded: ProjectMemory =
            serde_json::from_str(&std::fs::read_to_string(&file).unwrap()).unwrap();

        assert_eq!(loaded.conventions.total_commits_analyzed, 1);
        assert_eq!(loaded.corrections.len(), 1);

        let _ = std::fs::remove_dir_all(&tmp_dir);
    }
}
