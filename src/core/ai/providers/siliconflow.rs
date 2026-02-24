use crate::core::ai::provider::{AIProvider, ProviderConfig, StreamResponse};
use crate::core::ai::providers::openai_compat::OpenAICompatibleBase;
use anyhow::Result;
use async_trait::async_trait;

/// SiliconFlow AI 提供商
pub struct SiliconFlowProvider {
    base: OpenAICompatibleBase,
}

impl Default for SiliconFlowProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl SiliconFlowProvider {
    pub fn new() -> Self {
        Self {
            base: OpenAICompatibleBase::new(),
        }
    }
}

#[async_trait]
impl AIProvider for SiliconFlowProvider {
    async fn generate(&self, prompt: &str, config: &ProviderConfig) -> Result<String> {
        self.base
            .generate_chat(prompt, config, "SiliconFlow", Some(0.9))
            .await
    }

    async fn stream_generate(
        &self,
        prompt: &str,
        config: &ProviderConfig,
    ) -> Result<StreamResponse> {
        self.base
            .stream_chat(prompt, config, "SiliconFlow", Some(0.9))
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_siliconflow_provider_creation() {
        let _provider = SiliconFlowProvider::new();
    }

    #[test]
    fn test_siliconflow_default() {
        let _provider = SiliconFlowProvider::default();
    }
}
