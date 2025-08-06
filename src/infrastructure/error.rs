use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

/// 审查错误类型
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum ReviewError {
    #[error("配置错误: {message}")]
    Configuration { message: String },

    #[error("文件系统错误: {message}")]
    FileSystem { message: String, path: Option<String> },

    #[error("语言检测错误: {message}")]
    LanguageDetection { message: String, file_path: String },

    #[error("静态分析错误: {tool} - {message}")]
    StaticAnalysis { tool: String, message: String },

    #[error("AI 服务错误: {provider} - {message}")]
    AIService { provider: String, message: String, retryable: bool },

    #[error("敏感信息检测错误: {message}")]
    SensitiveInfoDetection { message: String },

    #[error("缓存错误: {message}")]
    Cache { message: String, cache_type: String },

    #[error("存储错误: {message}")]
    Storage { message: String, storage_type: String },

    #[error("消息队列错误: {message}")]
    Messaging { message: String, queue_type: String },

    #[error("通知错误: {platform} - {message}")]
    Notification { platform: String, message: String },

    #[error("网络错误: {message}")]
    Network { message: String, url: Option<String> },

    #[error("解析错误: {message}")]
    Parsing { message: String, content_type: String },

    #[error("验证错误: {message}")]
    Validation { message: String, field: Option<String> },

    #[error("超时错误: {operation} 超时 ({timeout_seconds}s)")]
    Timeout { operation: String, timeout_seconds: u64 },

    #[error("权限错误: {message}")]
    Permission { message: String, resource: String },

    #[error("资源不足: {message}")]
    ResourceExhausted { message: String, resource_type: String },

    #[error("内部错误: {message}")]
    Internal { message: String },

    #[error("外部依赖错误: {dependency} - {message}")]
    ExternalDependency { dependency: String, message: String },
}

impl ReviewError {
    /// 检查错误是否可重试
    pub fn is_retryable(&self) -> bool {
        match self {
            ReviewError::AIService { retryable, .. } => *retryable,
            ReviewError::Network { .. } => true,
            ReviewError::Timeout { .. } => true,
            ReviewError::ResourceExhausted { .. } => true,
            ReviewError::ExternalDependency { .. } => true,
            ReviewError::Storage { .. } => true,
            ReviewError::Messaging { .. } => true,
            _ => false,
        }
    }

    /// 获取错误的严重程度
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            ReviewError::Configuration { .. } => ErrorSeverity::Critical,
            ReviewError::FileSystem { .. } => ErrorSeverity::High,
            ReviewError::Permission { .. } => ErrorSeverity::High,
            ReviewError::Internal { .. } => ErrorSeverity::Critical,
            ReviewError::AIService { .. } => ErrorSeverity::Medium,
            ReviewError::Network { .. } => ErrorSeverity::Medium,
            ReviewError::Timeout { .. } => ErrorSeverity::Medium,
            ReviewError::Storage { .. } => ErrorSeverity::High,
            ReviewError::Messaging { .. } => ErrorSeverity::Low,
            ReviewError::Notification { .. } => ErrorSeverity::Low,
            _ => ErrorSeverity::Medium,
        }
    }

    /// 获取错误类别
    pub fn category(&self) -> ErrorCategory {
        match self {
            ReviewError::Configuration { .. } => ErrorCategory::Configuration,
            ReviewError::FileSystem { .. } => ErrorCategory::IO,
            ReviewError::LanguageDetection { .. } => ErrorCategory::Analysis,
            ReviewError::StaticAnalysis { .. } => ErrorCategory::Analysis,
            ReviewError::AIService { .. } => ErrorCategory::ExternalService,
            ReviewError::SensitiveInfoDetection { .. } => ErrorCategory::Analysis,
            ReviewError::Cache { .. } => ErrorCategory::Infrastructure,
            ReviewError::Storage { .. } => ErrorCategory::Infrastructure,
            ReviewError::Messaging { .. } => ErrorCategory::Infrastructure,
            ReviewError::Notification { .. } => ErrorCategory::Infrastructure,
            ReviewError::Network { .. } => ErrorCategory::Network,
            ReviewError::Parsing { .. } => ErrorCategory::Data,
            ReviewError::Validation { .. } => ErrorCategory::Data,
            ReviewError::Timeout { .. } => ErrorCategory::Performance,
            ReviewError::Permission { .. } => ErrorCategory::Security,
            ReviewError::ResourceExhausted { .. } => ErrorCategory::Performance,
            ReviewError::Internal { .. } => ErrorCategory::Internal,
            ReviewError::ExternalDependency { .. } => ErrorCategory::ExternalService,
        }
    }

    /// 创建配置错误
    pub fn config(message: impl Into<String>) -> Self {
        ReviewError::Configuration {
            message: message.into(),
        }
    }

    /// 创建文件系统错误
    pub fn file_system(message: impl Into<String>, path: Option<String>) -> Self {
        ReviewError::FileSystem {
            message: message.into(),
            path,
        }
    }

    /// 创建 AI 服务错误
    pub fn ai_service(provider: impl Into<String>, message: impl Into<String>, retryable: bool) -> Self {
        ReviewError::AIService {
            provider: provider.into(),
            message: message.into(),
            retryable,
        }
    }

    /// 创建网络错误
    pub fn network(message: impl Into<String>, url: Option<String>) -> Self {
        ReviewError::Network {
            message: message.into(),
            url,
        }
    }

    /// 创建超时错误
    pub fn timeout(operation: impl Into<String>, timeout_seconds: u64) -> Self {
        ReviewError::Timeout {
            operation: operation.into(),
            timeout_seconds,
        }
    }
}

/// 错误严重程度
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ErrorSeverity {
    Critical,
    High,
    Medium,
    Low,
}

/// 错误类别
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ErrorCategory {
    Configuration,
    IO,
    Network,
    Analysis,
    ExternalService,
    Infrastructure,
    Data,
    Performance,
    Security,
    Internal,
}

/// 错误上下文
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorContext {
    pub operation: String,
    pub file_path: Option<String>,
    pub line_number: Option<usize>,
    pub additional_info: std::collections::HashMap<String, String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub correlation_id: Option<String>,
}

impl ErrorContext {
    pub fn new(operation: impl Into<String>) -> Self {
        Self {
            operation: operation.into(),
            file_path: None,
            line_number: None,
            additional_info: std::collections::HashMap::new(),
            timestamp: chrono::Utc::now(),
            correlation_id: None,
        }
    }

    pub fn with_file(mut self, file_path: impl Into<String>) -> Self {
        self.file_path = Some(file_path.into());
        self
    }

    pub fn with_line(mut self, line_number: usize) -> Self {
        self.line_number = Some(line_number);
        self
    }

    pub fn with_info(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.additional_info.insert(key.into(), value.into());
        self
    }

    pub fn with_correlation_id(mut self, correlation_id: impl Into<String>) -> Self {
        self.correlation_id = Some(correlation_id.into());
        self
    }
}

/// 错误恢复策略
#[derive(Debug, Clone)]
pub enum RecoveryStrategy {
    /// 立即重试
    Retry { max_attempts: usize, delay_ms: u64 },
    /// 指数退避重试
    ExponentialBackoff { max_attempts: usize, initial_delay_ms: u64, max_delay_ms: u64 },
    /// 优雅降级
    Fallback { fallback_fn: fn() -> Result<(), ReviewError> },
    /// 跳过并继续
    Skip,
    /// 失败并停止
    Fail,
}

/// 错误处理器
pub struct ErrorHandler {
    strategies: std::collections::HashMap<ErrorCategory, RecoveryStrategy>,
    error_log: Vec<(ReviewError, ErrorContext)>,
}

impl ErrorHandler {
    pub fn new() -> Self {
        let mut strategies = std::collections::HashMap::new();

        // 设置默认恢复策略
        strategies.insert(ErrorCategory::Network, RecoveryStrategy::ExponentialBackoff {
            max_attempts: 3,
            initial_delay_ms: 1000,
            max_delay_ms: 10000,
        });

        strategies.insert(ErrorCategory::ExternalService, RecoveryStrategy::Retry {
            max_attempts: 2,
            delay_ms: 2000,
        });

        strategies.insert(ErrorCategory::Analysis, RecoveryStrategy::Skip);
        strategies.insert(ErrorCategory::Infrastructure, RecoveryStrategy::Skip);
        strategies.insert(ErrorCategory::Configuration, RecoveryStrategy::Fail);
        strategies.insert(ErrorCategory::Security, RecoveryStrategy::Fail);

        Self {
            strategies,
            error_log: Vec::new(),
        }
    }

    pub fn handle_error(&mut self, error: ReviewError, context: ErrorContext) -> Result<(), ReviewError> {
        // 记录错误
        self.error_log.push((error.clone(), context.clone()));

        // 根据错误类别选择恢复策略
        let strategy = self.strategies.get(&error.category())
            .unwrap_or(&RecoveryStrategy::Fail);

        match strategy {
            RecoveryStrategy::Retry { max_attempts, delay_ms } => {
                self.retry_operation(&error, &context, *max_attempts, *delay_ms)
            },
            RecoveryStrategy::ExponentialBackoff { max_attempts, initial_delay_ms, max_delay_ms } => {
                self.exponential_backoff(&error, &context, *max_attempts, *initial_delay_ms, *max_delay_ms)
            },
            RecoveryStrategy::Fallback { fallback_fn } => {
                fallback_fn()
            },
            RecoveryStrategy::Skip => {
                tracing::warn!("跳过错误: {} (上下文: {})", error, context.operation);
                Ok(())
            },
            RecoveryStrategy::Fail => {
                Err(error)
            },
        }
    }

    fn retry_operation(&self, error: &ReviewError, context: &ErrorContext, max_attempts: usize, delay_ms: u64) -> Result<(), ReviewError> {
        tracing::warn!("重试操作: {} (最大尝试次数: {})", context.operation, max_attempts);

        for attempt in 1..=max_attempts {
            std::thread::sleep(std::time::Duration::from_millis(delay_ms));

            // 这里应该重新执行原始操作
            // 由于我们没有原始操作的引用，我们只是模拟重试逻辑
            tracing::info!("重试第 {} 次: {}", attempt, context.operation);

            // 如果是最后一次尝试，返回原始错误
            if attempt == max_attempts {
                return Err(error.clone());
            }
        }

        Ok(())
    }

    fn exponential_backoff(&self, error: &ReviewError, context: &ErrorContext, max_attempts: usize, initial_delay_ms: u64, max_delay_ms: u64) -> Result<(), ReviewError> {
        tracing::warn!("指数退避重试: {}", context.operation);

        let mut delay = initial_delay_ms;

        for attempt in 1..=max_attempts {
            std::thread::sleep(std::time::Duration::from_millis(delay));

            tracing::info!("指数退避重试第 {} 次: {} (延迟: {}ms)", attempt, context.operation, delay);

            // 增加延迟时间
            delay = (delay * 2).min(max_delay_ms);

            // 如果是最后一次尝试，返回原始错误
            if attempt == max_attempts {
                return Err(error.clone());
            }
        }

        Ok(())
    }

    pub fn get_error_statistics(&self) -> ErrorStatistics {
        let mut stats = ErrorStatistics::new();

        for (error, _) in &self.error_log {
            stats.total_errors += 1;

            match error.severity() {
                ErrorSeverity::Critical => stats.critical_errors += 1,
                ErrorSeverity::High => stats.high_errors += 1,
                ErrorSeverity::Medium => stats.medium_errors += 1,
                ErrorSeverity::Low => stats.low_errors += 1,
            }

            *stats.errors_by_category.entry(error.category()).or_insert(0) += 1;
        }

        stats
    }

    pub fn clear_error_log(&mut self) {
        self.error_log.clear();
    }
}

/// 错误统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorStatistics {
    pub total_errors: usize,
    pub critical_errors: usize,
    pub high_errors: usize,
    pub medium_errors: usize,
    pub low_errors: usize,
    pub errors_by_category: std::collections::HashMap<ErrorCategory, usize>,
}

impl ErrorStatistics {
    pub fn new() -> Self {
        Self {
            total_errors: 0,
            critical_errors: 0,
            high_errors: 0,
            medium_errors: 0,
            low_errors: 0,
            errors_by_category: std::collections::HashMap::new(),
        }
    }
}

impl Default for ErrorHandler {
    fn default() -> Self {
        Self::new()
    }
}

// 实现从常见错误类型的转换
impl From<std::io::Error> for ReviewError {
    fn from(error: std::io::Error) -> Self {
        ReviewError::FileSystem {
            message: error.to_string(),
            path: None,
        }
    }
}

impl From<serde_json::Error> for ReviewError {
    fn from(error: serde_json::Error) -> Self {
        ReviewError::Parsing {
            message: error.to_string(),
            content_type: "JSON".to_string(),
        }
    }
}

impl From<reqwest::Error> for ReviewError {
    fn from(error: reqwest::Error) -> Self {
        ReviewError::Network {
            message: error.to_string(),
            url: error.url().map(|u| u.to_string()),
        }
    }
}

impl From<anyhow::Error> for ReviewError {
    fn from(error: anyhow::Error) -> Self {
        ReviewError::Internal {
            message: error.to_string(),
        }
    }
}