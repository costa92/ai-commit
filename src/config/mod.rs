use once_cell::sync::Lazy;
use std::env;
use std::path::PathBuf;

pub mod providers;
pub use providers::{ProviderRegistry, ProviderInfo, ApiFormat};

// 全局环境加载状态
static ENV_LOADED: Lazy<()> = Lazy::new(|| {
    // 尝试从用户主目录加载
    if let Ok(home) = env::var("HOME") {
        let user_env_path = PathBuf::from(home).join(".ai-commit/.env");
        if user_env_path.exists() {
            dotenvy::from_path(user_env_path).ok();
        }
    }

    // 尝试从当前目录加载
    dotenvy::dotenv().ok();
});

// 确保环境变量已加载（公开 API）
pub fn ensure_env_loaded() {
    Lazy::force(&ENV_LOADED);
}

/// 简化的配置结构体 - 只使用统一环境变量
#[derive(Debug, Clone, Default)]
pub struct Config {
    pub provider: String,
    pub model: String,
    pub debug: bool,
}

impl Config {
    pub fn new() -> Self {
        // 确保环境变量已加载
        #[cfg(not(test))]
        ensure_env_loaded();

        let config = Config {
            provider: env::var("AI_COMMIT_PROVIDER").unwrap_or_else(|_| "ollama".to_string()),
            model: env::var("AI_COMMIT_MODEL").unwrap_or_else(|_| "mistral".to_string()),
            debug: env::var("AI_COMMIT_DEBUG")
                .map(|v| v.to_lowercase() == "true" || v == "1")
                .unwrap_or(false),
        };

        config
    }

    pub fn update_from_args(&mut self, args: &crate::cli::args::Args) {
        // 命令行参数优先级最高
        if !args.provider.is_empty() {
            self.provider = args.provider.clone();
        }
        if !args.model.is_empty() {
            self.model = args.model.clone();
        }
    }

    /// 获取当前提供商的 API Key
    pub fn get_api_key(&self) -> Option<String> {
        env::var("AI_COMMIT_PROVIDER_API_KEY").ok()
    }

    /// 获取当前提供商的 URL
    pub fn get_url(&self) -> String {
        env::var("AI_COMMIT_PROVIDER_URL")
            .unwrap_or_else(|_| {
                // 使用提供商默认URL
                ProviderRegistry::get_provider(&self.provider)
                    .map(|info| info.default_url.clone())
                    .unwrap_or_default()
            })
    }

    /// 验证当前提供商配置
    pub fn validate(&self) -> anyhow::Result<()> {
        let provider_info = ProviderRegistry::get_provider(&self.provider)
            .ok_or_else(|| anyhow::anyhow!("Unsupported provider: {}", self.provider))?;

        if provider_info.requires_api_key && self.get_api_key().is_none() {
            anyhow::bail!(
                "{} requires API key. Please set AI_COMMIT_PROVIDER_API_KEY environment variable",
                provider_info.display_name
            );
        }

        Ok(())
    }

    /// 获取当前提供商信息
    pub fn get_current_provider_info(&self) -> Option<&'static ProviderInfo> {
        ProviderRegistry::get_provider(&self.provider)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    fn clear_env() {
        env::remove_var("AI_COMMIT_PROVIDER");
        env::remove_var("AI_COMMIT_MODEL");
        env::remove_var("AI_COMMIT_DEBUG");
        env::remove_var("AI_COMMIT_PROVIDER_API_KEY");
        env::remove_var("AI_COMMIT_PROVIDER_URL");
    }

    #[test]
    fn test_config_defaults() {
        clear_env();
        let config = Config::new();
        assert_eq!(config.provider, "ollama");
        assert_eq!(config.model, "mistral");
        assert!(!config.debug);
        clear_env();
    }

    #[test]
    fn test_config_from_env() {
        clear_env();
        env::set_var("AI_COMMIT_PROVIDER", "deepseek");
        env::set_var("AI_COMMIT_MODEL", "deepseek-chat");
        env::set_var("AI_COMMIT_DEBUG", "true");
        env::set_var("AI_COMMIT_PROVIDER_API_KEY", "test-key");
        env::set_var("AI_COMMIT_PROVIDER_URL", "https://custom.api.com");

        let config = Config::new();
        assert_eq!(config.provider, "deepseek");
        assert_eq!(config.model, "deepseek-chat");
        assert!(config.debug);
        assert_eq!(config.get_api_key(), Some("test-key".to_string()));
        assert_eq!(config.get_url(), "https://custom.api.com");

        clear_env();
    }

    #[test]
    fn test_validation() {
        clear_env();
        
        // ollama 不需要 API Key
        env::set_var("AI_COMMIT_PROVIDER", "ollama");
        let config = Config::new();
        assert!(config.validate().is_ok());

        // deepseek 需要 API Key
        env::set_var("AI_COMMIT_PROVIDER", "deepseek");
        let config = Config::new();
        assert!(config.validate().is_err());

        // 设置 API Key 后应该通过
        env::set_var("AI_COMMIT_PROVIDER_API_KEY", "test-key");
        let config = Config::new();
        assert!(config.validate().is_ok());

        clear_env();
    }
}