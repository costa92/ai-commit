use anyhow::Result;
use async_trait::async_trait;
use futures_util::Stream;
use std::pin::Pin;

/// AI 提供商配置
#[derive(Debug, Clone)]
pub struct ProviderConfig {
    pub model: String,
    pub api_key: Option<String>,
    pub api_url: String,
    pub timeout_secs: u64,
    pub max_retries: u32,
    pub stream: bool,
}

impl Default for ProviderConfig {
    fn default() -> Self {
        Self {
            model: String::from("mistral"),
            api_key: None,
            api_url: String::from("http://localhost:11434"),
            timeout_secs: 30,
            max_retries: 3,
            stream: true,
        }
    }
}

/// 流式响应类型
pub type StreamResponse = Pin<Box<dyn Stream<Item = Result<String>> + Send>>;

/// AI 提供商接口
#[async_trait]
pub trait AIProvider: Send + Sync {
    /// 生成响应（非流式）
    async fn generate(&self, prompt: &str, config: &ProviderConfig) -> Result<String>;
    
    /// 生成响应（流式）
    async fn stream_generate(
        &self,
        prompt: &str,
        config: &ProviderConfig,
    ) -> Result<StreamResponse>;
}

/// AI 提供商工厂
pub struct ProviderFactory;

impl ProviderFactory {
    /// 根据名称创建提供商
    pub fn create(name: &str) -> Result<Box<dyn AIProvider>> {
        use crate::core::ai::providers::{DeepseekProvider, KimiProvider, OllamaProvider, SiliconFlowProvider};
        
        match name.to_lowercase().as_str() {
            "ollama" => Ok(Box::new(OllamaProvider::new())),
            "deepseek" => Ok(Box::new(DeepseekProvider::new())),
            "siliconflow" => Ok(Box::new(SiliconFlowProvider::new())),
            "kimi" => Ok(Box::new(KimiProvider::new())),
            _ => anyhow::bail!("Unknown AI provider: {}", name),
        }
    }
    
    /// 获取所有支持的提供商列表
    pub fn list_providers() -> Vec<&'static str> {
        vec!["ollama", "deepseek", "siliconflow", "kimi"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_config_default() {
        let config = ProviderConfig::default();
        assert_eq!(config.model, "mistral");
        assert_eq!(config.timeout_secs, 30);
        assert_eq!(config.max_retries, 3);
        assert!(config.stream);
    }

    #[test]
    fn test_provider_factory_list() {
        let providers = ProviderFactory::list_providers();
        assert!(providers.contains(&"ollama"));
        assert!(providers.contains(&"deepseek"));
        assert!(providers.contains(&"siliconflow"));
        assert!(providers.contains(&"kimi"));
    }

    #[test]
    fn test_provider_factory_create() {
        assert!(ProviderFactory::create("ollama").is_ok());
        assert!(ProviderFactory::create("deepseek").is_ok());
        assert!(ProviderFactory::create("siliconflow").is_ok());
        assert!(ProviderFactory::create("kimi").is_ok());
        assert!(ProviderFactory::create("unknown").is_err());
    }
}