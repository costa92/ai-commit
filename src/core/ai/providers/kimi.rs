use crate::core::ai::provider::{AIProvider, ProviderConfig, StreamResponse};
use crate::core::ai::providers::openai_compat::OpenAICompatibleBase;
use anyhow::Result;
use async_trait::async_trait;

/// Kimi AI 提供商
pub struct KimiProvider {
    base: OpenAICompatibleBase,
}

impl Default for KimiProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl KimiProvider {
    pub fn new() -> Self {
        Self {
            base: OpenAICompatibleBase::new(),
        }
    }
}

#[async_trait]
impl AIProvider for KimiProvider {
    async fn generate(&self, prompt: &str, config: &ProviderConfig) -> Result<String> {
        self.base.generate_chat(prompt, config, "Kimi", None).await
    }

    async fn stream_generate(
        &self,
        prompt: &str,
        config: &ProviderConfig,
    ) -> Result<StreamResponse> {
        self.base.stream_chat(prompt, config, "Kimi", None).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kimi_provider_creation() {
        let _provider = KimiProvider::new();
    }

    #[test]
    fn test_kimi_default() {
        let _provider = KimiProvider::default();
    }
}
