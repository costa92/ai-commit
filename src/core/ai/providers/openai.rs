use crate::core::ai::provider::{AIProvider, ProviderConfig, StreamResponse};
use crate::core::ai::providers::openai_compat::OpenAICompatibleBase;
use anyhow::Result;
use async_trait::async_trait;

/// OpenAI 提供商
///
/// 直接使用 OpenAI API，复用 OpenAICompatibleBase。
/// 默认 URL: https://api.openai.com/v1/chat/completions
/// 默认 model: gpt-4o-mini
/// 环境变量: AI_COMMIT_OPENAI_API_KEY
pub struct OpenAIProvider {
    base: OpenAICompatibleBase,
}

impl Default for OpenAIProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl OpenAIProvider {
    pub fn new() -> Self {
        Self {
            base: OpenAICompatibleBase::new(),
        }
    }
}

#[async_trait]
impl AIProvider for OpenAIProvider {
    async fn generate(&self, prompt: &str, config: &ProviderConfig) -> Result<String> {
        self.base
            .generate_chat(prompt, config, "OpenAI", None)
            .await
    }

    async fn stream_generate(
        &self,
        prompt: &str,
        config: &ProviderConfig,
    ) -> Result<StreamResponse> {
        self.base.stream_chat(prompt, config, "OpenAI", None).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openai_provider_creation() {
        let _provider = OpenAIProvider::new();
    }

    #[test]
    fn test_openai_default() {
        let _provider = OpenAIProvider::default();
    }
}
