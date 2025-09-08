use once_cell::sync::Lazy;
use std::env;
use std::path::PathBuf;

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

// 环境变量批量读取结构（增加缓存机制）
pub struct EnvVars {
    provider: Option<String>,
    model: Option<String>,
    deepseek_api_key: Option<String>,
    deepseek_url: Option<String>,
    ollama_url: Option<String>,
    siliconflow_api_key: Option<String>,
    siliconflow_url: Option<String>,
    kimi_api_key: Option<String>,
    kimi_url: Option<String>,
    debug: Option<String>,
}

// 全局环境变量缓存
static ENV_VARS_CACHE: Lazy<std::sync::Mutex<Option<EnvVars>>> =
    Lazy::new(|| std::sync::Mutex::new(None));

impl EnvVars {
    fn load() -> Self {
        // 检查缓存
        {
            let cache = ENV_VARS_CACHE.lock().unwrap();
            if let Some(ref cached_vars) = *cache {
                return Self {
                    provider: cached_vars.provider.clone(),
                    model: cached_vars.model.clone(),
                    deepseek_api_key: cached_vars.deepseek_api_key.clone(),
                    deepseek_url: cached_vars.deepseek_url.clone(),
                    ollama_url: cached_vars.ollama_url.clone(),
                    siliconflow_api_key: cached_vars.siliconflow_api_key.clone(),
                    siliconflow_url: cached_vars.siliconflow_url.clone(),
                    kimi_api_key: cached_vars.kimi_api_key.clone(),
                    kimi_url: cached_vars.kimi_url.clone(),
                    debug: cached_vars.debug.clone(),
                };
            }
        }

        // 批量读取环境变量（优化：一次性读取所有需要的环境变量）
        let vars = Self {
            provider: env::var("AI_COMMIT_PROVIDER").ok(),
            model: env::var("AI_COMMIT_MODEL").ok(),
            deepseek_api_key: env::var("AI_COMMIT_DEEPSEEK_API_KEY").ok(),
            deepseek_url: env::var("AI_COMMIT_DEEPSEEK_URL").ok(),
            ollama_url: env::var("AI_COMMIT_OLLAMA_URL").ok(),
            siliconflow_api_key: env::var("AI_COMMIT_SILICONFLOW_API_KEY").ok(),
            siliconflow_url: env::var("AI_COMMIT_SILICONFLOW_URL").ok(),
            kimi_api_key: env::var("AI_COMMIT_KIMI_API_KEY").ok(),
            kimi_url: env::var("AI_COMMIT_KIMI_URL").ok(),
            debug: env::var("AI_COMMIT_DEBUG").ok(),
        };

        // 更新缓存
        *ENV_VARS_CACHE.lock().unwrap() = Some(EnvVars {
            provider: vars.provider.clone(),
            model: vars.model.clone(),
            deepseek_api_key: vars.deepseek_api_key.clone(),
            deepseek_url: vars.deepseek_url.clone(),
            ollama_url: vars.ollama_url.clone(),
            siliconflow_api_key: vars.siliconflow_api_key.clone(),
            siliconflow_url: vars.siliconflow_url.clone(),
            kimi_api_key: vars.kimi_api_key.clone(),
            kimi_url: vars.kimi_url.clone(),
            debug: vars.debug.clone(),
        });

        vars
    }

    // 清除缓存，用于测试
    pub fn clear_cache() {
        *ENV_VARS_CACHE.lock().unwrap() = None;
    }
}

#[derive(Debug, Clone, Default)]
pub struct Config {
    pub provider: String,
    pub model: String,
    pub deepseek_api_key: Option<String>,
    pub deepseek_url: String,
    pub ollama_url: String,
    pub siliconflow_api_key: Option<String>,
    pub siliconflow_url: String,
    pub kimi_api_key: Option<String>,
    pub kimi_url: String,
    pub debug: bool,
}

impl Config {
    pub fn new() -> Self {
        // 确保环境变量已加载
        #[cfg(not(test))]
        ensure_env_loaded();

        // 默认配置
        let config = Config {
            provider: "ollama".to_owned(),
            model: "mistral".to_owned(),
            deepseek_api_key: None,
            deepseek_url: "https://api.deepseek.com/v1/chat/completions".to_owned(),
            ollama_url: "http://localhost:11434/api/generate".to_owned(),
            siliconflow_api_key: None,
            siliconflow_url: "https://api.siliconflow.cn/v1/chat/completions".to_owned(),
            kimi_api_key: None,
            kimi_url: "https://api.moonshot.cn/v1/chat/completions".to_owned(),
            debug: false,
        };

        // 在非测试环境下加载环境变量
        #[cfg(not(test))]
        {
            let mut config = config;
            config.load_from_env();
            config
        }

        #[cfg(test)]
        config
    }

    pub fn load_from_env(&mut self) {
        let env_vars = EnvVars::load();

        if let Some(provider) = env_vars.provider {
            self.provider = provider;
        }
        if let Some(model) = env_vars.model {
            self.model = model;
        }
        if let Some(api_key) = env_vars.deepseek_api_key {
            self.deepseek_api_key = Some(api_key);
        }
        if let Some(url) = env_vars.deepseek_url {
            self.deepseek_url = url;
        }
        if let Some(url) = env_vars.ollama_url {
            self.ollama_url = url;
        }
        if let Some(api_key) = env_vars.siliconflow_api_key {
            self.siliconflow_api_key = Some(api_key);
        }
        if let Some(url) = env_vars.siliconflow_url {
            self.siliconflow_url = url;
        }
        if let Some(api_key) = env_vars.kimi_api_key {
            self.kimi_api_key = Some(api_key);
        }
        if let Some(url) = env_vars.kimi_url {
            self.kimi_url = url;
        }
        if let Some(debug) = env_vars.debug {
            self.debug = debug.to_lowercase() == "true" || debug == "1";
        }
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

    pub fn validate(&self) -> anyhow::Result<()> {
        // 验证配置的有效性
        match self.provider.as_str() {
            "deepseek" => {
                if self.deepseek_api_key.is_none() {
                    anyhow::bail!("Deepseek API key is required but not set. Please set AI_COMMIT_DEEPSEEK_API_KEY environment variable or in .env file");
                }
            }
            "siliconflow" => {
                if self.siliconflow_api_key.is_none() {
                    anyhow::bail!("SiliconFlow API key is required but not set. Please set AI_COMMIT_SILICONFLOW_API_KEY environment variable or in .env file");
                }
            }
            "kimi" => {
                if self.kimi_api_key.is_none() {
                    anyhow::bail!("Kimi API key is required but not set. Please set AI_COMMIT_KIMI_API_KEY environment variable or in .env file");
                }
            }
            "ollama" => {
                // Ollama 使用本地服务，不需要 API key
            }
            _ => {
                anyhow::bail!("Unsupported provider: {}", self.provider);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    fn clear_env() {
        env::remove_var("AI_COMMIT_PROVIDER");
        env::remove_var("AI_COMMIT_MODEL");
        env::remove_var("AI_COMMIT_DEEPSEEK_API_KEY");
        env::remove_var("AI_COMMIT_DEEPSEEK_URL");
        env::remove_var("AI_COMMIT_OLLAMA_URL");
        env::remove_var("AI_COMMIT_SILICONFLOW_API_KEY");
        env::remove_var("AI_COMMIT_SILICONFLOW_URL");
        env::remove_var("AI_COMMIT_KIMI_API_KEY");
        env::remove_var("AI_COMMIT_KIMI_URL");
        env::remove_var("AI_COMMIT_DEBUG");
    }

    #[test]
    fn test_config_defaults() {
        clear_env();
        let config = Config::new();
        assert_eq!(config.provider, "ollama");
        assert_eq!(config.model, "mistral");
        assert!(config.deepseek_api_key.is_none());
        assert_eq!(
            config.deepseek_url,
            "https://api.deepseek.com/v1/chat/completions"
        );
        assert_eq!(config.ollama_url, "http://localhost:11434/api/generate");
        assert!(config.siliconflow_api_key.is_none());
        assert_eq!(
            config.siliconflow_url,
            "https://api.siliconflow.cn/v1/chat/completions"
        );
        assert!(config.kimi_api_key.is_none());
        assert_eq!(config.kimi_url, "https://api.moonshot.cn/v1/chat/completions");
        clear_env();
    }

    #[test]
    fn test_config_from_env() {
        clear_env();
        env::set_var("AI_COMMIT_PROVIDER", "deepseek");
        env::set_var("AI_COMMIT_MODEL", "gpt-4");
        env::set_var("AI_COMMIT_DEEPSEEK_API_KEY", "test-key");
        env::set_var("AI_COMMIT_DEEPSEEK_URL", "https://test.api.deepseek.com");
        env::set_var("AI_COMMIT_OLLAMA_URL", "http://localhost:8080");

        let mut config = Config::new();
        config.load_from_env();

        assert_eq!(config.provider, "deepseek");
        assert_eq!(config.model, "gpt-4");
        assert_eq!(config.deepseek_api_key, Some("test-key".to_string()));
        assert_eq!(config.deepseek_url, "https://test.api.deepseek.com");
        assert_eq!(config.ollama_url, "http://localhost:8080");

        clear_env();
    }

    #[test]
    fn test_config_validation() {
        clear_env();
        let mut config = Config::new();

        // 测试默认的 ollama provider
        assert!(config.validate().is_ok());

        // 测试 deepseek provider 没有 API key
        config.provider = "deepseek".to_string();
        config.deepseek_api_key = None;
        assert!(config.validate().is_err());

        // 测试 deepseek provider 有 API key
        config.deepseek_api_key = Some("test-key".to_string());
        assert!(config.validate().is_ok());

        // 测试 siliconflow provider 没有 API key
        config.provider = "siliconflow".to_string();
        config.siliconflow_api_key = None;
        assert!(config.validate().is_err());

        // 测试 siliconflow provider 有 API key
        config.siliconflow_api_key = Some("test-key".to_string());
        assert!(config.validate().is_ok());

        // 测试不支持的 provider
        config.provider = "unsupported".to_string();
        assert!(config.validate().is_err());
        clear_env();
    }

    #[test]
    fn test_ensure_env_loaded_function() {
        // 测试环境加载函数
        ensure_env_loaded();

        // 多次调用应该是安全的
        ensure_env_loaded();
        ensure_env_loaded();

        // 验证函数不会崩溃
    }

    #[test]
    fn test_env_vars_batch_loading() {
        clear_env();

        // 设置一些环境变量
        env::set_var("AI_COMMIT_PROVIDER", "siliconflow");
        env::set_var("AI_COMMIT_MODEL", "qwen-plus");
        env::set_var("AI_COMMIT_SILICONFLOW_API_KEY", "test-siliconflow-key");
        env::set_var(
            "AI_COMMIT_SILICONFLOW_URL",
            "https://custom.siliconflow.com",
        );

        let env_vars = EnvVars::load();

        assert_eq!(env_vars.provider, Some("siliconflow".to_string()));
        assert_eq!(env_vars.model, Some("qwen-plus".to_string()));
        assert_eq!(
            env_vars.siliconflow_api_key,
            Some("test-siliconflow-key".to_string())
        );
        assert_eq!(
            env_vars.siliconflow_url,
            Some("https://custom.siliconflow.com".to_string())
        );
        assert_eq!(env_vars.deepseek_api_key, None);

        clear_env();
    }

    #[test]
    fn test_config_update_from_args() {
        clear_env();
        let mut config = Config::new();

        // 模拟命令行参数
        let mut args = crate::cli::args::Args::default();
        args.provider = "deepseek".to_string();
        args.model = "deepseek-chat".to_string();

        config.update_from_args(&args);

        assert_eq!(config.provider, "deepseek");
        assert_eq!(config.model, "deepseek-chat");

        clear_env();
    }

    #[test]
    fn test_config_update_from_empty_args() {
        clear_env();
        let mut config = Config::new();
        let original_provider = config.provider.clone();
        let original_model = config.model.clone();

        // 空参数不应该覆盖默认值
        let args = crate::cli::args::Args::default();

        config.update_from_args(&args);

        // 应该保持原始值
        assert_eq!(config.provider, original_provider);
        assert_eq!(config.model, original_model);

        clear_env();
    }

    #[test]
    fn test_config_priority_order() {
        clear_env();

        // 设置环境变量
        env::set_var("AI_COMMIT_PROVIDER", "env_provider");
        env::set_var("AI_COMMIT_MODEL", "env_model");

        let mut config = Config::new();

        // 环境变量应该被加载
        assert_eq!(config.provider, "env_provider");
        assert_eq!(config.model, "env_model");

        // 命令行参数应该覆盖环境变量
        let mut args = crate::cli::args::Args::default();
        args.provider = "cli_provider".to_string();
        args.model = "cli_model".to_string();

        config.update_from_args(&args);

        assert_eq!(config.provider, "cli_provider");
        assert_eq!(config.model, "cli_model");

        clear_env();
    }

    #[test]
    fn test_all_providers_validation() {
        clear_env();

        // 测试所有支持的提供商
        let test_cases = vec![
            ("ollama", None, true),
            ("deepseek", None, false),
            ("deepseek", Some("test-key"), true),
            ("siliconflow", None, false),
            ("siliconflow", Some("test-key"), true),
            ("kimi", None, false),
            ("kimi", Some("test-key"), true),
            ("unknown", None, false),
        ];

        for (provider, api_key, should_pass) in test_cases {
            let mut config = Config::new();
            config.provider = provider.to_string();

            if provider == "deepseek" {
                config.deepseek_api_key = api_key.map(String::from);
            } else if provider == "siliconflow" {
                config.siliconflow_api_key = api_key.map(String::from);
            } else if provider == "kimi" {
                config.kimi_api_key = api_key.map(String::from);
            }

            let result = config.validate();

            if should_pass {
                assert!(
                    result.is_ok(),
                    "Provider '{}' with key {:?} should pass validation",
                    provider,
                    api_key
                );
            } else {
                assert!(
                    result.is_err(),
                    "Provider '{}' with key {:?} should fail validation",
                    provider,
                    api_key
                );
            }
        }

        clear_env();
    }

    #[test]
    fn test_config_default_values() {
        clear_env();
        let config = Config::new();

        // 验证所有默认值
        assert_eq!(config.provider, "ollama");
        assert_eq!(config.model, "mistral");
        assert_eq!(
            config.deepseek_url,
            "https://api.deepseek.com/v1/chat/completions"
        );
        assert_eq!(config.ollama_url, "http://localhost:11434/api/generate");
        assert_eq!(
            config.siliconflow_url,
            "https://api.siliconflow.cn/v1/chat/completions"
        );
        assert!(config.deepseek_api_key.is_none());
        assert!(config.siliconflow_api_key.is_none());
        assert!(config.kimi_api_key.is_none());
        assert_eq!(config.kimi_url, "https://api.moonshot.cn/v1/chat/completions");

        clear_env();
    }

    #[test]
    fn test_config_clone() {
        clear_env();
        let config1 = Config::new();
        let config2 = config1.clone();

        assert_eq!(config1.provider, config2.provider);
        assert_eq!(config1.model, config2.model);
        assert_eq!(config1.deepseek_api_key, config2.deepseek_api_key);
        assert_eq!(config1.deepseek_url, config2.deepseek_url);
        assert_eq!(config1.ollama_url, config2.ollama_url);
        assert_eq!(config1.siliconflow_api_key, config2.siliconflow_api_key);
        assert_eq!(config1.siliconflow_url, config2.siliconflow_url);

        clear_env();
    }

    #[test]
    fn test_config_debug_format() {
        clear_env();
        let config = Config::new();
        let debug_str = format!("{:?}", config);

        // 验证 Debug 格式包含关键信息
        assert!(debug_str.contains("Config"));
        assert!(debug_str.contains("provider"));
        assert!(debug_str.contains("model"));

        clear_env();
    }

    #[test]
    fn test_env_loading_singleton() {
        clear_env();

        // 设置环境变量
        env::set_var("AI_COMMIT_PROVIDER", "test_singleton");

        // 多次调用 ensure_env_loaded
        ensure_env_loaded();
        ensure_env_loaded();
        ensure_env_loaded();

        // 环境变量应该只被加载一次
        let provider = env::var("AI_COMMIT_PROVIDER").unwrap();
        assert_eq!(provider, "test_singleton");

        clear_env();
    }

    #[test]
    fn test_partial_env_vars() {
        clear_env();

        // 只设置部分环境变量
        env::set_var("AI_COMMIT_PROVIDER", "deepseek");
        env::set_var("AI_COMMIT_DEEPSEEK_API_KEY", "partial-key");
        // 不设置其他变量

        let config = Config::new();

        // 应该使用环境变量覆盖默认值，未设置的保持默认
        assert_eq!(config.provider, "deepseek");
        assert_eq!(config.deepseek_api_key, Some("partial-key".to_string()));
        assert_eq!(config.model, "mistral"); // 默认值
        assert_eq!(
            config.deepseek_url,
            "https://api.deepseek.com/v1/chat/completions"
        ); // 默认值

        clear_env();
    }

    #[test]
    fn test_error_messages() {
        clear_env();

        // 测试各种错误消息
        let mut config = Config::new();

        // Deepseek 没有 API key
        config.provider = "deepseek".to_string();
        config.deepseek_api_key = None;
        let err = config.validate().unwrap_err();
        assert!(err.to_string().contains("Deepseek API key"));

        // SiliconFlow 没有 API key
        config.provider = "siliconflow".to_string();
        config.siliconflow_api_key = None;
        let err = config.validate().unwrap_err();
        assert!(err.to_string().contains("SiliconFlow API key"));

        // 不支持的提供商
        config.provider = "unknown_provider".to_string();
        let err = config.validate().unwrap_err();
        assert!(err.to_string().contains("Unsupported provider"));

        clear_env();
    }

    #[test]
    fn test_debug_mode_default() {
        clear_env();
        let config = Config::new();
        assert!(!config.debug);
        clear_env();
    }

    #[test]
    fn test_debug_mode_from_env_true() {
        clear_env();
        EnvVars::clear_cache();
        env::set_var("AI_COMMIT_DEBUG", "true");

        let mut config = Config::new();
        config.load_from_env();

        assert!(config.debug);
        clear_env();
    }

    #[test]
    fn test_debug_mode_from_env_one() {
        clear_env();
        EnvVars::clear_cache();
        env::set_var("AI_COMMIT_DEBUG", "1");

        let mut config = Config::new();
        config.load_from_env();

        assert!(config.debug);
        clear_env();
    }

    #[test]
    fn test_debug_mode_from_env_false() {
        clear_env();
        EnvVars::clear_cache();
        env::set_var("AI_COMMIT_DEBUG", "false");

        let mut config = Config::new();
        config.load_from_env();

        assert!(!config.debug);
        clear_env();
    }

    #[test]
    fn test_debug_mode_from_env_zero() {
        clear_env();
        EnvVars::clear_cache();
        env::set_var("AI_COMMIT_DEBUG", "0");

        let mut config = Config::new();
        config.load_from_env();

        assert!(!config.debug);
        clear_env();
    }

    #[test]
    fn test_debug_mode_from_env_invalid() {
        clear_env();
        EnvVars::clear_cache();
        env::set_var("AI_COMMIT_DEBUG", "invalid");

        let mut config = Config::new();
        config.load_from_env();

        assert!(!config.debug);
        clear_env();
    }

    #[test]
    fn test_debug_mode_case_insensitive() {
        clear_env();

        // 测试大写 TRUE
        EnvVars::clear_cache();
        env::set_var("AI_COMMIT_DEBUG", "TRUE");
        let mut config = Config::new();
        config.load_from_env();
        assert!(config.debug);

        // 测试混合大小写 True
        EnvVars::clear_cache();
        env::set_var("AI_COMMIT_DEBUG", "True");
        config = Config::new();
        config.load_from_env();
        assert!(config.debug);

        // 测试大写 FALSE
        EnvVars::clear_cache();
        env::set_var("AI_COMMIT_DEBUG", "FALSE");
        config = Config::new();
        config.load_from_env();
        assert!(!config.debug);

        clear_env();
    }

    #[test]
    fn test_debug_mode_in_env_vars_cache() {
        clear_env();
        EnvVars::clear_cache();

        env::set_var("AI_COMMIT_DEBUG", "true");
        env::set_var("AI_COMMIT_PROVIDER", "deepseek");

        let env_vars = EnvVars::load();

        assert_eq!(env_vars.debug, Some("true".to_string()));
        assert_eq!(env_vars.provider, Some("deepseek".to_string()));

        clear_env();
    }
} // 测试注释2
  // 添加更多内容到config
