use anyhow::Result;
use std::sync::Arc;

pub mod agents;
pub mod prompt;
pub mod provider;
pub mod providers;

pub use agents::{Agent, AgentManager, AgentContext, AgentTask, AgentConfig, TaskType};
pub use provider::{AIProvider, ProviderConfig, StreamResponse};
pub use prompt::{PromptBuilder, PromptTemplate};

/// AI 服务管理器
pub struct AIService {
    provider: Arc<dyn AIProvider>,
    prompt_builder: PromptBuilder,
}

impl AIService {
    /// 创建新的 AI 服务实例
    pub fn new(provider: Arc<dyn AIProvider>) -> Self {
        Self {
            provider,
            prompt_builder: PromptBuilder::new(),
        }
    }

    /// 生成提交消息
    pub async fn generate_commit_message(
        &self,
        diff: &str,
        config: &ProviderConfig,
    ) -> Result<String> {
        // 构建提示词
        let prompt = self.prompt_builder.build_commit_prompt(diff)?;
        
        // 调用 AI 提供商
        let response = self.provider.generate(&prompt, config).await?;
        
        // 验证和清理响应
        self.validate_and_clean_response(&response)
    }

    /// 生成标签说明
    pub async fn generate_tag_note(
        &self,
        changes: &str,
        version: &str,
        config: &ProviderConfig,
    ) -> Result<String> {
        let prompt = self.prompt_builder.build_tag_prompt(changes, version)?;
        self.provider.generate(&prompt, config).await
    }

    /// 验证和清理 AI 响应
    fn validate_and_clean_response(&self, response: &str) -> Result<String> {
        // 验证 Conventional Commits 格式
        if !self.is_valid_commit_format(response) {
            anyhow::bail!("Invalid commit message format");
        }
        
        // 清理响应
        Ok(self.clean_response(response))
    }

    /// 检查是否为有效的提交格式
    fn is_valid_commit_format(&self, message: &str) -> bool {
        use once_cell::sync::Lazy;
        use regex::Regex;
        
        static VALID_FORMAT: Lazy<Regex> = Lazy::new(|| {
            Regex::new(r"^(feat|fix|docs|style|refactor|test|chore)(\([^)]+\))?:\s+\S+.*$")
                .unwrap()
        });
        
        VALID_FORMAT.is_match(message.lines().next().unwrap_or(""))
    }

    /// 清理响应内容
    fn clean_response(&self, response: &str) -> String {
        response.lines()
            .next()
            .unwrap_or("")
            .trim()
            .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_commit_format() {
        let service = AIService::new(Arc::new(MockProvider));
        
        assert!(service.is_valid_commit_format("feat(api): 添加用户认证"));
        assert!(service.is_valid_commit_format("fix: 修复登录问题"));
        assert!(!service.is_valid_commit_format("invalid format"));
        assert!(!service.is_valid_commit_format(""));
    }

    #[test]
    fn test_clean_response() {
        let service = AIService::new(Arc::new(MockProvider));
        
        let response = "  feat(api): 添加新功能  \n额外的内容";
        assert_eq!(
            service.clean_response(response),
            "feat(api): 添加新功能"
        );
    }

    struct MockProvider;
    
    #[async_trait]
    impl AIProvider for MockProvider {
        async fn generate(&self, _prompt: &str, _config: &ProviderConfig) -> Result<String> {
            Ok("feat(test): 测试消息".to_string())
        }
        
        async fn stream_generate(
            &self,
            _prompt: &str,
            _config: &ProviderConfig,
        ) -> Result<StreamResponse> {
            unimplemented!()
        }
    }
}