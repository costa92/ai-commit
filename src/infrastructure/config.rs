use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use crate::infrastructure::error::ReviewError;

/// 应用程序配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// 基础配置
    pub general: GeneralConfig,

    /// AI 服务配置
    pub ai: AIConfig,

    /// 静态分析配置
    pub static_analysis: StaticAnalysisConfig,

    /// 敏感信息检测配置
    pub sensitive_detection: SensitiveDetectionConfig,

    /// 缓存配置
    pub cache: CacheConfig,

    /// 存储配置
    pub storage: StorageConfig,

    /// 消息队列配置
    pub messaging: MessagingConfig,

    /// 通知配置
    pub notification: NotificationConfig,

    /// 日志配置
    pub logging: LoggingConfig,

    /// 性能配置
    pub performance: PerformanceConfig,
}

/// 基础配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    pub project_name: String,
    pub version: String,
    pub environment: Environment,
    pub debug_mode: bool,
    pub max_file_size_mb: usize,
    pub supported_extensions: Vec<String>,
    pub excluded_paths: Vec<String>,
    pub temp_dir: PathBuf,
}

/// 环境类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Environment {
    Development,
    Testing,
    Staging,
    Production,
}

/// AI 服务配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIConfig {
    pub enabled: bool,
    pub default_provider: String,
    pub providers: HashMap<String, AIProviderConfig>,
    pub timeout_seconds: u64,
    pub max_retries: usize,
    pub temperature: f32,
    pub max_tokens: usize,
}

/// AI 提供商配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIProviderConfig {
    pub enabled: bool,
    pub api_url: String,
    pub api_key: Option<String>,
    pub model: String,
    pub timeout_seconds: u64,
    pub rate_limit_per_minute: usize,
}

/// 静态分析配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaticAnalysisConfig {
    pub enabled: bool,
    pub tools: HashMap<String, StaticAnalysisToolConfig>,
    pub parallel_execution: bool,
    pub timeout_seconds: u64,
    pub fail_on_error: bool,
}

/// 静态分析工具配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaticAnalysisToolConfig {
    pub enabled: bool,
    pub command: String,
    pub args: Vec<String>,
    pub timeout_seconds: u64,
    pub supported_languages: Vec<String>,
}

/// 敏感信息检测配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensitiveDetectionConfig {
    pub enabled: bool,
    pub patterns_file: Option<PathBuf>,
    pub whitelist_file: Option<PathBuf>,
    pub custom_patterns: Vec<SensitivePatternConfig>,
    pub min_confidence: f32,
    pub exclude_test_files: bool,
}

/// 敏感信息模式配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensitivePatternConfig {
    pub name: String,
    pub pattern: String,
    pub info_type: String,
    pub risk_level: String,
    pub confidence: f32,
    pub enabled: bool,
}

/// 缓存配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    pub enabled: bool,
    pub memory_cache_size: usize,
    pub file_cache_enabled: bool,
    pub file_cache_dir: PathBuf,
    pub default_ttl_seconds: u64,
    pub cleanup_interval_seconds: u64,
}

/// 存储配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub enabled: bool,
    pub storage_type: String,
    pub connection_string: Option<String>,
    pub database_name: String,
    pub collection_name: String,
    pub connection_pool_size: usize,
    pub timeout_seconds: u64,
}

/// 消息队列配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessagingConfig {
    pub enabled: bool,
    pub queue_type: String,
    pub brokers: Vec<String>,
    pub topics: HashMap<String, String>,
    pub consumer_group: String,
    pub batch_size: usize,
    pub timeout_seconds: u64,
}

/// 通知配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationConfig {
    pub enabled: bool,
    pub providers: HashMap<String, NotificationProviderConfig>,
    pub default_channels: Vec<String>,
    pub retry_attempts: usize,
    pub retry_delay_seconds: u64,
}

/// 通知提供商配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationProviderConfig {
    pub enabled: bool,
    pub webhook_url: Option<String>,
    pub api_key: Option<String>,
    pub template: Option<String>,
    pub timeout_seconds: u64,
}

/// 日志配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: String,
    pub output: String,
    pub file_path: Option<PathBuf>,
    pub max_file_size_mb: usize,
    pub max_files: usize,
    pub include_file_location: bool,
    pub include_thread_names: bool,
}

/// 性能配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    pub max_concurrent_files: usize,
    pub max_memory_usage_mb: usize,
    pub analysis_timeout_seconds: u64,
    pub enable_profiling: bool,
    pub gc_interval_seconds: u64,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
            ai: AIConfig::default(),
            static_analysis: StaticAnalysisConfig::default(),
            sensitive_detection: SensitiveDetectionConfig::default(),
            cache: CacheConfig::default(),
            storage: StorageConfig::default(),
            messaging: MessagingConfig::default(),
            notification: NotificationConfig::default(),
            logging: LoggingConfig::default(),
            performance: PerformanceConfig::default(),
        }
    }
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            project_name: "ai-commit".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            environment: Environment::Development,
            debug_mode: false,
            max_file_size_mb: 10,
            supported_extensions: vec![
                "rs".to_string(), "go".to_string(), "ts".to_string(),
                "js".to_string(), "py".to_string(), "java".to_string(),
            ],
            excluded_paths: vec![
                "target/".to_string(), "node_modules/".to_string(),
                ".git/".to_string(), "vendor/".to_string(),
            ],
            temp_dir: std::env::temp_dir(),
        }
    }
}

impl Default for AIConfig {
    fn default() -> Self {
        let mut providers = HashMap::new();

        providers.insert("ollama".to_string(), AIProviderConfig {
            enabled: true,
            api_url: "http://localhost:11434".to_string(),
            api_key: None,
            model: "llama2".to_string(),
            timeout_seconds: 30,
            rate_limit_per_minute: 60,
        });

        providers.insert("deepseek".to_string(), AIProviderConfig {
            enabled: false,
            api_url: "https://api.deepseek.com".to_string(),
            api_key: None,
            model: "deepseek-coder".to_string(),
            timeout_seconds: 30,
            rate_limit_per_minute: 100,
        });

        Self {
            enabled: false,
            default_provider: "ollama".to_string(),
            providers,
            timeout_seconds: 30,
            max_retries: 3,
            temperature: 0.1,
            max_tokens: 2048,
        }
    }
}

impl Default for StaticAnalysisConfig {
    fn default() -> Self {
        let mut tools = HashMap::new();

        // Go 工具
        tools.insert("gofmt".to_string(), StaticAnalysisToolConfig {
            enabled: true,
            command: "gofmt".to_string(),
            args: vec!["-d".to_string()],
            timeout_seconds: 10,
            supported_languages: vec!["go".to_string()],
        });

        // Rust 工具
        tools.insert("rustfmt".to_string(), StaticAnalysisToolConfig {
            enabled: true,
            command: "rustfmt".to_string(),
            args: vec!["--check".to_string()],
            timeout_seconds: 10,
            supported_languages: vec!["rust".to_string()],
        });

        Self {
            enabled: true,
            tools,
            parallel_execution: true,
            timeout_seconds: 60,
            fail_on_error: false,
        }
    }
}

impl Default for SensitiveDetectionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            patterns_file: None,
            whitelist_file: None,
            custom_patterns: Vec::new(),
            min_confidence: 0.8,
            exclude_test_files: true,
        }
    }
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            memory_cache_size: 1000,
            file_cache_enabled: true,
            file_cache_dir: PathBuf::from(".cache"),
            default_ttl_seconds: 3600,
            cleanup_interval_seconds: 300,
        }
    }
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            storage_type: "sqlite".to_string(),
            connection_string: None,
            database_name: "ai_commit".to_string(),
            collection_name: "reviews".to_string(),
            connection_pool_size: 10,
            timeout_seconds: 30,
        }
    }
}

impl Default for MessagingConfig {
    fn default() -> Self {
        let mut topics = HashMap::new();
        topics.insert("report_generated".to_string(), "ai-commit-reports".to_string());
        topics.insert("analysis_completed".to_string(), "ai-commit-analysis".to_string());

        Self {
            enabled: false,
            queue_type: "kafka".to_string(),
            brokers: vec!["localhost:9092".to_string()],
            topics,
            consumer_group: "ai-commit-consumers".to_string(),
            batch_size: 100,
            timeout_seconds: 30,
        }
    }
}

impl Default for NotificationConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            providers: HashMap::new(),
            default_channels: Vec::new(),
            retry_attempts: 3,
            retry_delay_seconds: 5,
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            format: "pretty".to_string(),
            output: "stdout".to_string(),
            file_path: None,
            max_file_size_mb: 100,
            max_files: 5,
            include_file_location: true,
            include_thread_names: false,
        }
    }
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            max_concurrent_files: num_cpus::get(),
            max_memory_usage_mb: 1024,
            analysis_timeout_seconds: 300,
            enable_profiling: false,
            gc_interval_seconds: 60,
        }
    }
}

/// 配置管理器
pub struct ConfigManager {
    config: AppConfig,
    config_sources: Vec<ConfigSource>,
    watchers: Vec<Box<dyn ConfigWatcher>>,
}

/// 配置源
#[derive(Debug, Clone)]
pub enum ConfigSource {
    File(PathBuf),
    Environment,
    CommandLine(HashMap<String, String>),
    Default,
}

/// 配置监听器
pub trait ConfigWatcher: Send + Sync {
    fn on_config_changed(&self, config: &AppConfig) -> Result<(), ReviewError>;
}

impl ConfigManager {
    pub fn new() -> Self {
        Self {
            config: AppConfig::default(),
            config_sources: vec![ConfigSource::Default],
            watchers: Vec::new(),
        }
    }

    /// 添加配置源
    pub fn add_source(mut self, source: ConfigSource) -> Self {
        self.config_sources.push(source);
        self
    }

    /// 添加配置监听器
    pub fn add_watcher(mut self, watcher: Box<dyn ConfigWatcher>) -> Self {
        self.watchers.push(watcher);
        self
    }

    /// 加载配置
    pub fn load(&mut self) -> Result<(), ReviewError> {
        let mut config = AppConfig::default();

        // 按优先级顺序加载配置源
        for source in &self.config_sources {
            match source {
                ConfigSource::File(path) => {
                    self.load_from_file(&mut config, path)?;
                },
                ConfigSource::Environment => {
                    self.load_from_environment(&mut config)?;
                },
                ConfigSource::CommandLine(args) => {
                    self.load_from_command_line(&mut config, args)?;
                },
                ConfigSource::Default => {
                    // 默认配置已经设置
                },
            }
        }

        // 验证配置
        self.validate_config(&config)?;

        self.config = config;
        self.notify_watchers()?;

        Ok(())
    }

    /// 从文件加载配置
    fn load_from_file(&self, config: &mut AppConfig, path: &PathBuf) -> Result<(), ReviewError> {
        if !path.exists() {
            return Ok(()); // 文件不存在时跳过
        }

        let content = std::fs::read_to_string(path)
            .map_err(|e| ReviewError::file_system(
                format!("无法读取配置文件: {}", e),
                Some(path.to_string_lossy().to_string())
            ))?;

        let file_config: AppConfig = match path.extension().and_then(|s| s.to_str()) {
            Some("toml") => toml::from_str(&content)
                .map_err(|e| ReviewError::config(format!("TOML 解析错误: {}", e)))?,
            Some("yaml") | Some("yml") => serde_yaml::from_str(&content)
                .map_err(|e| ReviewError::config(format!("YAML 解析错误: {}", e)))?,
            Some("json") => serde_json::from_str(&content)
                .map_err(|e| ReviewError::config(format!("JSON 解析错误: {}", e)))?,
            _ => return Err(ReviewError::config("不支持的配置文件格式")),
        };

        // 合并配置
        self.merge_config(config, file_config);

        Ok(())
    }

    /// 从环境变量加载配置
    fn load_from_environment(&self, config: &mut AppConfig) -> Result<(), ReviewError> {
        // AI 配置
        if let Ok(enabled) = std::env::var("AI_COMMIT_AI_ENABLED") {
            config.ai.enabled = enabled.parse().unwrap_or(false);
        }

        if let Ok(provider) = std::env::var("AI_COMMIT_AI_PROVIDER") {
            config.ai.default_provider = provider;
        }

        // 存储配置
        if let Ok(enabled) = std::env::var("AI_COMMIT_STORAGE_ENABLED") {
            config.storage.enabled = enabled.parse().unwrap_or(false);
        }

        if let Ok(connection_string) = std::env::var("AI_COMMIT_STORAGE_CONNECTION") {
            config.storage.connection_string = Some(connection_string);
        }

        // 日志配置
        if let Ok(level) = std::env::var("AI_COMMIT_LOG_LEVEL") {
            config.logging.level = level;
        }

        // 通知配置
        if let Ok(enabled) = std::env::var("AI_COMMIT_NOTIFICATION_ENABLED") {
            config.notification.enabled = enabled.parse().unwrap_or(false);
        }

        Ok(())
    }

    /// 从命令行参数加载配置
    fn load_from_command_line(&self, config: &mut AppConfig, args: &HashMap<String, String>) -> Result<(), ReviewError> {
        for (key, value) in args {
            match key.as_str() {
                "ai-enabled" => {
                    config.ai.enabled = value.parse().unwrap_or(false);
                },
                "ai-provider" => {
                    config.ai.default_provider = value.clone();
                },
                "storage-enabled" => {
                    config.storage.enabled = value.parse().unwrap_or(false);
                },
                "log-level" => {
                    config.logging.level = value.clone();
                },
                "debug" => {
                    config.general.debug_mode = value.parse().unwrap_or(false);
                },
                _ => {
                    // 忽略未知参数
                },
            }
        }

        Ok(())
    }

    /// 合并配置
    fn merge_config(&self, base: &mut AppConfig, other: AppConfig) {
        // 这里应该实现智能合并逻辑
        // 为了简化，我们直接覆盖非默认值

        if other.ai.enabled != AIConfig::default().enabled {
            base.ai.enabled = other.ai.enabled;
        }

        if other.ai.default_provider != AIConfig::default().default_provider {
            base.ai.default_provider = other.ai.default_provider;
        }

        // 合并 AI 提供商配置
        for (key, value) in other.ai.providers {
            base.ai.providers.insert(key, value);
        }

        // 类似地合并其他配置...
    }

    /// 验证配置
    fn validate_config(&self, config: &AppConfig) -> Result<(), ReviewError> {
        // 验证 AI 配置
        if config.ai.enabled {
            if !config.ai.providers.contains_key(&config.ai.default_provider) {
                return Err(ReviewError::config(
                    format!("默认 AI 提供商 '{}' 未配置", config.ai.default_provider)
                ));
            }
        }

        // 验证存储配置
        if config.storage.enabled && config.storage.connection_string.is_none() {
            return Err(ReviewError::config("存储已启用但未提供连接字符串"));
        }

        // 验证性能配置
        if config.performance.max_concurrent_files == 0 {
            return Err(ReviewError::config("最大并发文件数不能为 0"));
        }

        if config.performance.max_memory_usage_mb < 100 {
            return Err(ReviewError::config("最大内存使用量不能小于 100MB"));
        }

        Ok(())
    }

    /// 通知监听器
    fn notify_watchers(&self) -> Result<(), ReviewError> {
        for watcher in &self.watchers {
            watcher.on_config_changed(&self.config)?;
        }
        Ok(())
    }

    /// 获取配置
    pub fn get_config(&self) -> &AppConfig {
        &self.config
    }

    /// 热重载配置
    pub fn reload(&mut self) -> Result<(), ReviewError> {
        self.load()
    }

    /// 保存配置到文件
    pub fn save_to_file(&self, path: &PathBuf, format: ConfigFormat) -> Result<(), ReviewError> {
        let content = match format {
            ConfigFormat::Toml => toml::to_string_pretty(&self.config)
                .map_err(|e| ReviewError::config(format!("TOML 序列化错误: {}", e)))?,
            ConfigFormat::Yaml => serde_yaml::to_string(&self.config)
                .map_err(|e| ReviewError::config(format!("YAML 序列化错误: {}", e)))?,
            ConfigFormat::Json => serde_json::to_string_pretty(&self.config)
                .map_err(|e| ReviewError::config(format!("JSON 序列化错误: {}", e)))?,
        };

        std::fs::write(path, content)
            .map_err(|e| ReviewError::file_system(
                format!("无法写入配置文件: {}", e),
                Some(path.to_string_lossy().to_string())
            ))?;

        Ok(())
    }
}

/// 配置文件格式
#[derive(Debug, Clone)]
pub enum ConfigFormat {
    Toml,
    Yaml,
    Json,
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.general.project_name, "ai-commit");
        assert!(!config.ai.enabled);
        assert!(config.static_analysis.enabled);
        assert!(config.sensitive_detection.enabled);
    }

    #[test]
    fn test_config_manager_creation() {
        let manager = ConfigManager::new();
        assert_eq!(manager.config_sources.len(), 1);
        assert!(matches!(manager.config_sources[0], ConfigSource::Default));
    }

    #[test]
    fn test_environment_detection() {
        let env = Environment::Development;
        assert_eq!(env, Environment::Development);

        let json = serde_json::to_string(&env).unwrap();
        let deserialized: Environment = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, Environment::Development);
    }
}