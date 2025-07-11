use std::env;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Config {
    pub provider: String,
    pub model: String,
    pub deepseek_api_key: Option<String>,
    pub deepseek_url: String,
    pub ollama_url: String,
    pub siliconflow_api_key: Option<String>,
    pub siliconflow_url: String,
}

impl Config {
    pub fn new() -> Self {
        // 默认配置
        let mut config = Config {
            provider: "ollama".to_string(),
            model: "mistral".to_string(),
            deepseek_api_key: None,
            deepseek_url: "https://api.deepseek.com/v1/chat/completions".to_string(),
            ollama_url: "http://localhost:11434/api/generate".to_string(),
            siliconflow_api_key: None,
            siliconflow_url: "https://api.siliconflow.cn/v1/chat/completions".to_string(),
        };

        // 加载配置文件
        #[cfg(not(test))]
        config.load_from_env_file();
        // 加载环境变量（覆盖配置文件）
        config.load_from_env();

        config
    }

    pub fn load_from_env_file(&mut self) {
        // 尝试从用户主目录加载
        if let Ok(home) = env::var("HOME") {
            let user_env_path = PathBuf::from(format!("{}/.ai-commit/.env", home));
            if user_env_path.exists() {
                dotenvy::from_path(user_env_path).ok();
            }
        }

        // 尝试从当前目录加载
        dotenvy::dotenv().ok();
    }

    pub fn load_from_env(&mut self) {
        // 加载环境变量，不覆盖已有值
        if let Ok(provider) = env::var("AI_COMMIT_PROVIDER") {
            self.provider = provider;
        }
        if let Ok(model) = env::var("AI_COMMIT_MODEL") {
            self.model = model;
        }
        if let Ok(api_key) = env::var("AI_COMMIT_DEEPSEEK_API_KEY") {
            self.deepseek_api_key = Some(api_key);
        }
        if let Ok(url) = env::var("AI_COMMIT_DEEPSEEK_URL") {
            self.deepseek_url = url;
        }
        if let Ok(url) = env::var("AI_COMMIT_OLLAMA_URL") {
            self.ollama_url = url;
        }
        if let Ok(api_key) = env::var("AI_COMMIT_SILICONFLOW_API_KEY") {
            self.siliconflow_api_key = Some(api_key);
        }
        if let Ok(url) = env::var("AI_COMMIT_SILICONFLOW_URL") {
            self.siliconflow_url = url;
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

        let config = Config::new();
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
}