use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 提供商配置信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderInfo {
    /// 提供商名称
    pub name: String,
    /// 显示名称
    pub display_name: String,
    /// 默认 API URL
    pub default_url: String,
    /// 是否需要 API Key
    pub requires_api_key: bool,
    /// 默认模型
    pub default_model: String,
    /// 支持的模型列表
    pub supported_models: Vec<String>,
    /// API 格式类型
    pub api_format: ApiFormat,
    /// 环境变量前缀
    pub env_prefix: String,
    /// 描述
    pub description: String,
}

/// API 格式类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ApiFormat {
    /// OpenAI 兼容格式 (Deepseek, Kimi, SiliconFlow, OpenAI, Qwen)
    OpenAI,
    /// Ollama 格式
    Ollama,
    /// Anthropic Messages API 格式 (Claude)
    Anthropic,
    /// Google Generative AI 格式 (Gemini)
    Google,
    /// 自定义格式
    Custom,
}

/// 提供商配置文件结构
#[derive(Debug, Deserialize)]
struct ProvidersConfig {
    providers: Vec<ProviderConfig>,
}

/// 单个提供商配置
#[derive(Debug, Deserialize)]
struct ProviderConfig {
    name: String,
    display_name: String,
    default_url: String,
    requires_api_key: bool,
    default_model: String,
    supported_models: Vec<String>,
    api_format: ApiFormat,
    env_prefix: String,
    description: String,
}

impl From<ProviderConfig> for ProviderInfo {
    fn from(config: ProviderConfig) -> Self {
        Self {
            name: config.name,
            display_name: config.display_name,
            default_url: config.default_url,
            requires_api_key: config.requires_api_key,
            default_model: config.default_model,
            supported_models: config.supported_models,
            api_format: config.api_format,
            env_prefix: config.env_prefix,
            description: config.description,
        }
    }
}

/// 从配置文件加载提供商信息
fn load_providers_from_config() -> HashMap<String, ProviderInfo> {
    // 尝试从不同位置加载配置文件
    let config_paths = [
        "providers.toml",
        "config/providers.toml",
        "/etc/ai-commit/providers.toml",
    ];

    for path in &config_paths {
        if let Ok(content) = std::fs::read_to_string(path) {
            if let Ok(config) = toml::from_str::<ProvidersConfig>(&content) {
                let mut providers = HashMap::new();
                for provider_config in config.providers {
                    let provider_info: ProviderInfo = provider_config.into();
                    providers.insert(provider_info.name.clone(), provider_info);
                }
                return providers;
            }
        }
    }

    // 如果无法加载配置文件，使用默认配置
    get_default_providers()
}

/// 获取默认提供商配置（硬编码备份）
fn get_default_providers() -> HashMap<String, ProviderInfo> {
    let mut providers = HashMap::new();

    // Ollama 配置
    providers.insert(
        "ollama".to_string(),
        ProviderInfo {
            name: "ollama".to_string(),
            display_name: "Ollama".to_string(),
            default_url: "http://localhost:11434/api/generate".to_string(),
            requires_api_key: false,
            default_model: "mistral".to_string(),
            supported_models: vec![
                "mistral".to_string(),
                "llama3".to_string(),
                "qwen2".to_string(),
                "codellama".to_string(),
                "gemma".to_string(),
                "phi3".to_string(),
            ],
            api_format: ApiFormat::Ollama,
            env_prefix: "AI_COMMIT_OLLAMA".to_string(),
            description: "本地 Ollama 服务，无需 API Key".to_string(),
        },
    );

    // Deepseek 配置
    providers.insert(
        "deepseek".to_string(),
        ProviderInfo {
            name: "deepseek".to_string(),
            display_name: "Deepseek".to_string(),
            default_url: "https://api.deepseek.com/v1/chat/completions".to_string(),
            requires_api_key: true,
            default_model: "deepseek-chat".to_string(),
            supported_models: vec!["deepseek-chat".to_string(), "deepseek-coder".to_string()],
            api_format: ApiFormat::OpenAI,
            env_prefix: "AI_COMMIT_DEEPSEEK".to_string(),
            description: "深度求索 AI 服务，需要 API Key".to_string(),
        },
    );

    // SiliconFlow 配置
    providers.insert(
        "siliconflow".to_string(),
        ProviderInfo {
            name: "siliconflow".to_string(),
            display_name: "SiliconFlow".to_string(),
            default_url: "https://api.siliconflow.cn/v1/chat/completions".to_string(),
            requires_api_key: true,
            default_model: "qwen/Qwen2-7B-Instruct".to_string(),
            supported_models: vec![
                "qwen/Qwen2-7B-Instruct".to_string(),
                "qwen/Qwen2-72B-Instruct".to_string(),
                "deepseek-ai/deepseek-coder-6.7b-instruct".to_string(),
                "01-ai/Yi-34B-Chat-4bits".to_string(),
            ],
            api_format: ApiFormat::OpenAI,
            env_prefix: "AI_COMMIT_SILICONFLOW".to_string(),
            description: "硅基流动 AI 服务，需要 API Key".to_string(),
        },
    );

    // Kimi 配置
    providers.insert(
        "kimi".to_string(),
        ProviderInfo {
            name: "kimi".to_string(),
            display_name: "Kimi".to_string(),
            default_url: "https://api.moonshot.cn/v1/chat/completions".to_string(),
            requires_api_key: true,
            default_model: "moonshot-v1-8k".to_string(),
            supported_models: vec![
                "moonshot-v1-8k".to_string(),
                "moonshot-v1-32k".to_string(),
                "moonshot-v1-128k".to_string(),
            ],
            api_format: ApiFormat::OpenAI,
            env_prefix: "AI_COMMIT_KIMI".to_string(),
            description: "月之暗面 Kimi AI 服务，需要 API Key".to_string(),
        },
    );

    // OpenAI 配置
    providers.insert(
        "openai".to_string(),
        ProviderInfo {
            name: "openai".to_string(),
            display_name: "OpenAI".to_string(),
            default_url: "https://api.openai.com/v1/chat/completions".to_string(),
            requires_api_key: true,
            default_model: "gpt-4o-mini".to_string(),
            supported_models: vec![
                "gpt-4o-mini".to_string(),
                "gpt-4o".to_string(),
                "gpt-4-turbo".to_string(),
                "o1-mini".to_string(),
            ],
            api_format: ApiFormat::OpenAI,
            env_prefix: "AI_COMMIT_OPENAI".to_string(),
            description: "OpenAI GPT 系列模型，需要 API Key".to_string(),
        },
    );

    // Claude 配置
    providers.insert(
        "claude".to_string(),
        ProviderInfo {
            name: "claude".to_string(),
            display_name: "Claude".to_string(),
            default_url: "https://api.anthropic.com/v1/messages".to_string(),
            requires_api_key: true,
            default_model: "claude-sonnet-4-20250514".to_string(),
            supported_models: vec![
                "claude-sonnet-4-20250514".to_string(),
                "claude-haiku-4-5-20251001".to_string(),
                "claude-opus-4-6".to_string(),
            ],
            api_format: ApiFormat::Anthropic,
            env_prefix: "AI_COMMIT_CLAUDE".to_string(),
            description: "Anthropic Claude 系列模型，需要 API Key".to_string(),
        },
    );

    // Gemini 配置
    providers.insert(
        "gemini".to_string(),
        ProviderInfo {
            name: "gemini".to_string(),
            display_name: "Gemini".to_string(),
            default_url: "https://generativelanguage.googleapis.com/v1beta".to_string(),
            requires_api_key: true,
            default_model: "gemini-2.0-flash".to_string(),
            supported_models: vec![
                "gemini-2.0-flash".to_string(),
                "gemini-2.0-flash-lite".to_string(),
                "gemini-1.5-pro".to_string(),
            ],
            api_format: ApiFormat::Google,
            env_prefix: "AI_COMMIT_GEMINI".to_string(),
            description: "Google Gemini 系列模型，需要 API Key".to_string(),
        },
    );

    // Qwen 配置
    providers.insert(
        "qwen".to_string(),
        ProviderInfo {
            name: "qwen".to_string(),
            display_name: "Qwen".to_string(),
            default_url: "https://dashscope.aliyuncs.com/compatible-mode/v1/chat/completions"
                .to_string(),
            requires_api_key: true,
            default_model: "qwen-turbo".to_string(),
            supported_models: vec![
                "qwen-turbo".to_string(),
                "qwen-plus".to_string(),
                "qwen-max".to_string(),
            ],
            api_format: ApiFormat::OpenAI,
            env_prefix: "AI_COMMIT_QWEN".to_string(),
            description: "阿里云通义千问 AI 服务，需要 API Key".to_string(),
        },
    );

    providers
}

/// 全局提供商配置映射
pub static PROVIDER_REGISTRY: Lazy<HashMap<String, ProviderInfo>> =
    Lazy::new(load_providers_from_config);

impl ProviderInfo {
    /// 获取 API URL 环境变量名
    pub fn url_env_var(&self) -> String {
        format!("{}_URL", self.env_prefix)
    }

    /// 获取 API Key 环境变量名  
    pub fn api_key_env_var(&self) -> String {
        format!("{}_API_KEY", self.env_prefix)
    }

    /// 验证提供商是否配置正确
    pub fn validate(&self, api_key: Option<&str>) -> anyhow::Result<()> {
        if self.requires_api_key && api_key.is_none() {
            anyhow::bail!(
                "{} API key is required but not set. Please set {} environment variable or in .env file",
                self.display_name,
                self.api_key_env_var()
            );
        }
        Ok(())
    }
}

/// 提供商注册表操作
pub struct ProviderRegistry;

impl ProviderRegistry {
    /// 获取所有已注册的提供商
    pub fn list_providers() -> Vec<&'static str> {
        PROVIDER_REGISTRY.keys().map(|s| s.as_str()).collect()
    }

    /// 获取提供商信息
    pub fn get_provider(name: &str) -> Option<&'static ProviderInfo> {
        PROVIDER_REGISTRY.get(name)
    }

    /// 检查提供商是否存在
    pub fn exists(name: &str) -> bool {
        PROVIDER_REGISTRY.contains_key(name)
    }

    /// 获取所有提供商信息
    pub fn get_all() -> &'static HashMap<String, ProviderInfo> {
        &PROVIDER_REGISTRY
    }

    /// 重新加载提供商配置（用于热重载）
    pub fn reload() -> anyhow::Result<()> {
        anyhow::bail!("热重载功能未实现，需要重启应用程序以加载新配置")
    }

    /// 获取提供商配置文件路径信息
    pub fn get_config_info() -> String {
        format!(
            "提供商配置将从以下位置按顺序加载：\n\
            1. ./providers.toml (当前目录)\n\
            2. ./config/providers.toml\n\
            3. /etc/ai-commit/providers.toml (系统配置)\n\
            4. 内置默认配置 (备用)\n\
            \n\
            当前加载的提供商数量: {}",
            PROVIDER_REGISTRY.len()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_registry_basic() {
        let providers = ProviderRegistry::list_providers();
        assert!(providers.contains(&"ollama"));
        assert!(providers.contains(&"deepseek"));
        assert!(providers.contains(&"siliconflow"));
        assert!(providers.contains(&"kimi"));
        assert!(providers.contains(&"openai"));
        assert!(providers.contains(&"claude"));
        assert!(providers.contains(&"gemini"));
        assert!(providers.contains(&"qwen"));
    }

    #[test]
    fn test_provider_info_validation() {
        let ollama = ProviderRegistry::get_provider("ollama").unwrap();
        assert!(ollama.validate(None).is_ok()); // Ollama 不需要 API key

        let deepseek = ProviderRegistry::get_provider("deepseek").unwrap();
        assert!(deepseek.validate(None).is_err()); // Deepseek 需要 API key
        assert!(deepseek.validate(Some("test-key")).is_ok());
    }

    #[test]
    fn test_provider_env_vars() {
        let kimi = ProviderRegistry::get_provider("kimi").unwrap();
        assert_eq!(kimi.url_env_var(), "AI_COMMIT_KIMI_URL");
        assert_eq!(kimi.api_key_env_var(), "AI_COMMIT_KIMI_API_KEY");
    }

    #[test]
    fn test_api_formats() {
        let ollama = ProviderRegistry::get_provider("ollama").unwrap();
        assert_eq!(ollama.api_format, ApiFormat::Ollama);

        let deepseek = ProviderRegistry::get_provider("deepseek").unwrap();
        assert_eq!(deepseek.api_format, ApiFormat::OpenAI);

        let openai = ProviderRegistry::get_provider("openai").unwrap();
        assert_eq!(openai.api_format, ApiFormat::OpenAI);

        let claude = ProviderRegistry::get_provider("claude").unwrap();
        assert_eq!(claude.api_format, ApiFormat::Anthropic);

        let gemini = ProviderRegistry::get_provider("gemini").unwrap();
        assert_eq!(gemini.api_format, ApiFormat::Google);

        let qwen = ProviderRegistry::get_provider("qwen").unwrap();
        assert_eq!(qwen.api_format, ApiFormat::OpenAI);
    }

    #[test]
    fn test_provider_models() {
        let ollama = ProviderRegistry::get_provider("ollama").unwrap();
        assert!(ollama.supported_models.contains(&"mistral".to_string()));

        let kimi = ProviderRegistry::get_provider("kimi").unwrap();
        assert!(kimi
            .supported_models
            .contains(&"moonshot-v1-8k".to_string()));
    }

    #[test]
    fn test_default_providers_fallback() {
        let default_providers = get_default_providers();
        assert!(default_providers.contains_key("ollama"));
        assert!(default_providers.contains_key("deepseek"));
        assert!(default_providers.contains_key("siliconflow"));
        assert!(default_providers.contains_key("kimi"));
        assert!(default_providers.contains_key("openai"));
        assert!(default_providers.contains_key("claude"));
        assert!(default_providers.contains_key("gemini"));
        assert!(default_providers.contains_key("qwen"));
    }

    #[test]
    fn test_config_info() {
        let info = ProviderRegistry::get_config_info();
        assert!(info.contains("providers.toml"));
        assert!(info.contains("当前加载的提供商数量"));
    }

    #[test]
    fn test_provider_config_conversion() {
        let config = ProviderConfig {
            name: "test".to_string(),
            display_name: "Test Provider".to_string(),
            default_url: "https://test.com/api".to_string(),
            requires_api_key: true,
            default_model: "test-model".to_string(),
            supported_models: vec!["model1".to_string(), "model2".to_string()],
            api_format: ApiFormat::OpenAI,
            env_prefix: "AI_COMMIT_TEST".to_string(),
            description: "Test provider".to_string(),
        };

        let info: ProviderInfo = config.into();
        assert_eq!(info.name, "test");
        assert_eq!(info.display_name, "Test Provider");
        assert!(info.requires_api_key);
    }
}
