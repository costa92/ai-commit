use crate::core::ai::provider::{AIProvider, ProviderConfig, StreamResponse};
use crate::core::ai::providers::openai_compat::OpenAICompatibleBase;
use anyhow::Result;
use async_trait::async_trait;

/// 通义千问 (Qwen) 提供商
///
/// 阿里云 DashScope OpenAI 兼容模式，复用 OpenAICompatibleBase。
/// 默认 URL: https://dashscope.aliyuncs.com/compatible-mode/v1/chat/completions
/// 默认 model: qwen-turbo
/// 环境变量: AI_COMMIT_QWEN_API_KEY
pub struct QwenProvider {
    base: OpenAICompatibleBase,
}

impl Default for QwenProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl QwenProvider {
    pub fn new() -> Self {
        Self {
            base: OpenAICompatibleBase::new(),
        }
    }
}

#[async_trait]
impl AIProvider for QwenProvider {
    async fn generate(&self, prompt: &str, config: &ProviderConfig) -> Result<String> {
        self.base.generate_chat(prompt, config, "Qwen", None).await
    }

    async fn stream_generate(
        &self,
        prompt: &str,
        config: &ProviderConfig,
    ) -> Result<StreamResponse> {
        self.base.stream_chat(prompt, config, "Qwen", None).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_qwen_provider_creation() {
        let _provider = QwenProvider::new();
    }

    #[test]
    fn test_qwen_default() {
        let _provider = QwenProvider::default();
    }
}
