use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::notification::{NotificationMessage, NotificationPlatform, NotificationSeverity};

/// 失败记录类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FailureType {
    /// 网络连接失败
    NetworkFailure,
    /// 认证失败
    AuthenticationFailure,
    /// 服务器错误
    ServerError,
    /// 超时错误
    TimeoutError,
    /// 频率限制
    RateLimitExceeded,
    /// 配置错误
    ConfigurationError,
    /// 消息格式错误
    MessageFormatError,
    /// 未知错误
    UnknownError,
}

impl From<&str> for FailureType {
    fn from(error_str: &str) -> Self {
        let error_lower = error_str.to_lowercase();

        if error_lower.contains("network") || error_lower.contains("connection") {
            FailureType::NetworkFailure
        } else if error_lower.contains("auth") || error_lower.contains("401") || error_lower.contains("403") {
            FailureType::AuthenticationFailure
        } else if error_lower.contains("server") || error_lower.contains("5") {
            FailureType::ServerError
        } else if error_lower.contains("timeout") || error_lower.contains("timed out") {
            FailureType::TimeoutError
        } else if error_lower.contains("rate limit") || error_lower.contains("429") {
            FailureType::RateLimitExceeded
        } else if error_lower.contains("config") || error_lower.contains("invalid") {
            FailureType::ConfigurationError
        } else if error_lower.contains("format") || error_lower.contains("parse") {
            FailureType::MessageFormatError
        } else {
            FailureType::UnknownError
        }
    }
}

/// 失败记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailureRecord {
    pub id: String,
    pub message_id: String,
    pub platform: NotificationPlatform,
    pub failure_type: FailureType,
    pub error_message: String,
    pub error_details: Option<String>,
    pub stack_trace: Option<String>,
    pub retry_count: u32,
    pub final_failure: bool,
    pub occurred_at: DateTime<Utc>,
    pub message_metadata: HashMap<String, String>,
    pub context: FailureContext,
}

/// 失败上下文信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailureContext {
    pub project_path: String,
    pub message_title: String,
    pub message_severity: NotificationSeverity,
    pub user_id: Option<String>,
    pub correlation_id: Option<String>,
    pub environment: String,
    pub version: String,
}

impl FailureRecord {
    pub fn new(
        message: &NotificationMessage,
        platform: NotificationPlatform,
        error: &anyhow::Error,
        retry_count: u32,
        final_failure: bool,
    ) -> Self {
        let error_message = error.to_string();
        let failure_type = FailureType::from(error_message.as_str());

        Self {
            id: Uuid::new_v4().to_string(),
            message_id: message.id.clone(),
            platform,
            failure_type,
            error_message: error_message.clone(),
            error_details: Some(format!("{:#}", error)),
            stack_trace: None, // Backtrace handling simplified for compatibility
            retry_count,
            final_failure,
            occurred_at: Utc::now(),
            message_metadata: message.metadata.clone(),
            context: FailureContext {
                project_path: message.project_path.clone(),
                message_title: message.title.clone(),
                message_severity: message.severity.clone(),
                user_id: message.metadata.get("user_id").cloned(),
                correlation_id: message.metadata.get("correlation_id").cloned(),
                environment: std::env::var("ENVIRONMENT").unwrap_or_else(|_| "unknown".to_string()),
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
        }
    }
}

/// 失败统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailureStatistics {
    pub total_failures: u64,
    pub final_failures: u64,
    pub retry_failures: u64,
    pub failure_rate: f64,
    pub platform_failures: HashMap<NotificationPlatform, u64>,
    pub failure_type_distribution: HashMap<String, u64>,
    pub hourly_failure_counts: HashMap<String, u64>,
    pub top_error_messages: Vec<(String, u64)>,
    pub average_retry_count: f64,
    pub last_updated: DateTime<Utc>,
}

impl Default for FailureStatistics {
    fn default() -> Self {
        Self {
            total_failures: 0,
            final_failures: 0,
            retry_failures: 0,
            failure_rate: 0.0,
            platform_failures: HashMap::new(),
            failure_type_distribution: HashMap::new(),
            hourly_failure_counts: HashMap::new(),
            top_error_messages: Vec::new(),
            average_retry_count: 0.0,
            last_updated: Utc::now(),
        }
    }
}

/// 失败日志配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailureLoggerConfig {
    /// 是否启用失败日志记录
    pub enabled: bool,
    /// 日志文件路径
    pub log_file_path: Option<PathBuf>,
    /// 是否记录到标准日志
    pub log_to_stdout: bool,
    /// 是否记录到结构化日志
    pub structured_logging: bool,
    /// 最大记录数量
    pub max_records: usize,
    /// 记录保留时间（天）
    pub retention_days: u32,
    /// 是否记录敏感信息
    pub include_sensitive_data: bool,
    /// 自动清理间隔（小时）
    pub cleanup_interval_hours: u32,
}

impl Default for FailureLoggerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            log_file_path: Some(PathBuf::from("logs/notification_failures.log")),
            log_to_stdout: true,
            structured_logging: true,
            max_records: 10000,
            retention_days: 30,
            include_sensitive_data: false,
            cleanup_interval_hours: 24,
        }
    }
}

/// 失败日志记录器
pub struct FailureLogger {
    config: FailureLoggerConfig,
    records: Arc<RwLock<Vec<FailureRecord>>>,
    statistics: Arc<RwLock<FailureStatistics>>,
    error_message_counts: Arc<RwLock<HashMap<String, u64>>>,
}

impl FailureLogger {
    pub fn new(config: FailureLoggerConfig) -> Self {
        Self {
            config,
            records: Arc::new(RwLock::new(Vec::new())),
            statistics: Arc::new(RwLock::new(FailureStatistics::default())),
            error_message_counts: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 记录失败
    pub async fn log_failure(
        &self,
        message: &NotificationMessage,
        platform: NotificationPlatform,
        error: &anyhow::Error,
        retry_count: u32,
        final_failure: bool,
    ) -> anyhow::Result<()> {
        if !self.config.enabled {
            return Ok(());
        }

        let record = FailureRecord::new(message, platform, error, retry_count, final_failure);

        // 记录到内存
        {
            let mut records = self.records.write().await;
            records.push(record.clone());

            // 限制记录数量
            if records.len() > self.config.max_records {
                records.drain(0..1000);
            }
        }

        // 更新统计信息
        self.update_statistics(&record).await;

        // 记录到文件
        if let Some(ref log_file_path) = self.config.log_file_path {
            self.write_to_file(&record, log_file_path).await?;
        }

        // 记录到标准输出
        if self.config.log_to_stdout {
            self.write_to_stdout(&record).await;
        }

        // 结构化日志记录
        if self.config.structured_logging {
            self.write_structured_log(&record).await;
        }

        Ok(())
    }

    /// 更新统计信息
    async fn update_statistics(&self, record: &FailureRecord) {
        let mut stats = self.statistics.write().await;

        stats.total_failures += 1;

        if record.final_failure {
            stats.final_failures += 1;
        } else {
            stats.retry_failures += 1;
        }

        // 更新平台失败统计
        *stats.platform_failures.entry(record.platform.clone()).or_insert(0) += 1;

        // 更新失败类型分布
        let failure_type_str = format!("{:?}", record.failure_type);
        *stats.failure_type_distribution.entry(failure_type_str).or_insert(0) += 1;

        // 更新小时级失败统计
        let hour_key = record.occurred_at.format("%Y-%m-%d-%H").to_string();
        *stats.hourly_failure_counts.entry(hour_key).or_insert(0) += 1;

        // 更新错误消息计数
        {
            let mut error_counts = self.error_message_counts.write().await;
            *error_counts.entry(record.error_message.clone()).or_insert(0) += 1;

            // 更新 top 错误消息
            let mut top_errors: Vec<(String, u64)> = error_counts.iter()
                .map(|(msg, count)| (msg.clone(), *count))
                .collect();
            top_errors.sort_by(|a, b| b.1.cmp(&a.1));
            top_errors.truncate(10);
            stats.top_error_messages = top_errors;
        }

        // 计算平均重试次数
        if stats.total_failures > 0 {
            let total_retries: u64 = {
                let records = self.records.read().await;
                records.iter().map(|r| r.retry_count as u64).sum()
            };
            stats.average_retry_count = total_retries as f64 / stats.total_failures as f64;
        }

        stats.last_updated = Utc::now();
    }

    /// 写入文件
    async fn write_to_file(&self, record: &FailureRecord, file_path: &PathBuf) -> anyhow::Result<()> {
        // 确保目录存在
        if let Some(parent) = file_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let log_entry = if self.config.structured_logging {
            serde_json::to_string(record)?
        } else {
            self.format_human_readable(record)
        };

        // 异步写入文件
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(file_path)?;

        writeln!(file, "{}", log_entry)?;
        file.flush()?;

        Ok(())
    }

    /// 写入标准输出
    async fn write_to_stdout(&self, record: &FailureRecord) {
        let log_entry = self.format_human_readable(record);
        println!("{}", log_entry);
    }

    /// 结构化日志记录
    async fn write_structured_log(&self, record: &FailureRecord) {
        log::error!(
            target: "notification_failure",
            "Notification failure: {:?} - {:?} - {} - {} retries - message_id: {} - platform: {:?} - failure_type: {:?} - project_path: {} - message_title: {} - severity: {:?} - environment: {} - version: {}",
            record.platform,
            record.failure_type,
            record.error_message,
            record.retry_count,
            record.message_id,
            record.platform,
            record.failure_type,
            record.context.project_path,
            record.context.message_title,
            record.context.message_severity,
            record.context.environment,
            record.context.version
        );
    }

    /// 格式化为人类可读格式
    fn format_human_readable(&self, record: &FailureRecord) -> String {
        let sensitive_data = if self.config.include_sensitive_data {
            format!("\n  Error Details: {}", record.error_details.as_deref().unwrap_or("N/A"))
        } else {
            String::new()
        };

        format!(
            "[{}] NOTIFICATION FAILURE\n\
            ID: {}\n\
            Message ID: {}\n\
            Platform: {:?}\n\
            Failure Type: {:?}\n\
            Error: {}\n\
            Retry Count: {}\n\
            Final Failure: {}\n\
            Project: {}\n\
            Title: {}\n\
            Severity: {:?}\n\
            Environment: {}\n\
            Version: {}{}\n\
            ---",
            record.occurred_at.format("%Y-%m-%d %H:%M:%S UTC"),
            record.id,
            record.message_id,
            record.platform,
            record.failure_type,
            record.error_message,
            record.retry_count,
            record.final_failure,
            record.context.project_path,
            record.context.message_title,
            record.context.message_severity,
            record.context.environment,
            record.context.version,
            sensitive_data
        )
    }

    /// 获取失败记录
    pub async fn get_failure_records(&self) -> Vec<FailureRecord> {
        self.records.read().await.clone()
    }

    /// 获取失败统计信息
    pub async fn get_statistics(&self) -> FailureStatistics {
        self.statistics.read().await.clone()
    }

    /// 按条件查询失败记录
    pub async fn query_failures(
        &self,
        platform: Option<NotificationPlatform>,
        failure_type: Option<FailureType>,
        start_time: Option<DateTime<Utc>>,
        end_time: Option<DateTime<Utc>>,
        final_failures_only: bool,
    ) -> Vec<FailureRecord> {
        let records = self.records.read().await;

        records.iter()
            .filter(|record| {
                if let Some(ref p) = platform {
                    if record.platform != *p {
                        return false;
                    }
                }

                if let Some(ref ft) = failure_type {
                    if std::mem::discriminant(&record.failure_type) != std::mem::discriminant(ft) {
                        return false;
                    }
                }

                if let Some(start) = start_time {
                    if record.occurred_at < start {
                        return false;
                    }
                }

                if let Some(end) = end_time {
                    if record.occurred_at > end {
                        return false;
                    }
                }

                if final_failures_only && !record.final_failure {
                    return false;
                }

                true
            })
            .cloned()
            .collect()
    }

    /// 清理过期记录
    pub async fn cleanup_old_records(&self) -> anyhow::Result<usize> {
        let cutoff_time = Utc::now() - chrono::Duration::days(self.config.retention_days as i64);

        let mut records = self.records.write().await;
        let initial_count = records.len();

        records.retain(|record| record.occurred_at > cutoff_time);

        let removed_count = initial_count - records.len();

        if removed_count > 0 {
            log::info!("Cleaned up {} old failure records", removed_count);
        }

        Ok(removed_count)
    }

    /// 导出失败记录
    pub async fn export_records(&self, file_path: &PathBuf, format: ExportFormat) -> anyhow::Result<()> {
        let records = self.records.read().await;

        match format {
            ExportFormat::Json => {
                let json_data = serde_json::to_string_pretty(&*records)?;
                tokio::fs::write(file_path, json_data).await?;
            }
            ExportFormat::Csv => {
                let mut csv_content = String::new();
                csv_content.push_str("ID,Message ID,Platform,Failure Type,Error Message,Retry Count,Final Failure,Occurred At,Project Path,Message Title,Severity\n");

                for record in records.iter() {
                    csv_content.push_str(&format!(
                        "{},{},{:?},{:?},{},{},{},{},{},{},{:?}\n",
                        record.id,
                        record.message_id,
                        record.platform,
                        record.failure_type,
                        record.error_message.replace(',', ";"),
                        record.retry_count,
                        record.final_failure,
                        record.occurred_at.format("%Y-%m-%d %H:%M:%S"),
                        record.context.project_path,
                        record.context.message_title.replace(',', ";"),
                        record.context.message_severity
                    ));
                }

                tokio::fs::write(file_path, csv_content).await?;
            }
        }

        Ok(())
    }

    /// 获取配置
    pub fn get_config(&self) -> &FailureLoggerConfig {
        &self.config
    }

    /// 更新配置
    pub fn update_config(&mut self, config: FailureLoggerConfig) {
        self.config = config;
    }
}

/// 导出格式
#[derive(Debug, Clone)]
pub enum ExportFormat {
    Json,
    Csv,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::notification::NotificationSeverity;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_failure_record_creation() {
        let message = NotificationMessage::new(
            "Test Title".to_string(),
            "Test Content".to_string(),
            NotificationSeverity::Error,
            "/test/project".to_string(),
        );

        let error = anyhow::anyhow!("Network connection failed");
        let record = FailureRecord::new(&message, NotificationPlatform::Email, &error, 2, true);

        assert_eq!(record.message_id, message.id);
        assert_eq!(record.platform, NotificationPlatform::Email);
        assert!(matches!(record.failure_type, FailureType::NetworkFailure));
        assert_eq!(record.retry_count, 2);
        assert!(record.final_failure);
    }

    #[tokio::test]
    async fn test_failure_type_classification() {
        assert!(matches!(FailureType::from("Network connection failed"), FailureType::NetworkFailure));
        assert!(matches!(FailureType::from("Authentication failed"), FailureType::AuthenticationFailure));
        assert!(matches!(FailureType::from("Server error 500"), FailureType::ServerError));
        assert!(matches!(FailureType::from("Request timed out"), FailureType::TimeoutError));
        assert!(matches!(FailureType::from("Rate limit exceeded"), FailureType::RateLimitExceeded));
        assert!(matches!(FailureType::from("Unknown error"), FailureType::UnknownError));
    }

    #[tokio::test]
    async fn test_failure_logging() {
        let temp_dir = tempdir().unwrap();
        let log_file = temp_dir.path().join("test_failures.log");

        let config = FailureLoggerConfig {
            enabled: true,
            log_file_path: Some(log_file.clone()),
            log_to_stdout: false,
            structured_logging: false,
            ..Default::default()
        };

        let logger = FailureLogger::new(config);

        let message = NotificationMessage::new(
            "Test Title".to_string(),
            "Test Content".to_string(),
            NotificationSeverity::Warning,
            "/test/project".to_string(),
        );

        let error = anyhow::anyhow!("Test error");
        logger.log_failure(&message, NotificationPlatform::Feishu, &error, 1, false).await.unwrap();

        // 验证记录被添加
        let records = logger.get_failure_records().await;
        assert_eq!(records.len(), 1);

        // 验证文件被创建
        assert!(log_file.exists());

        // 验证统计信息
        let stats = logger.get_statistics().await;
        assert_eq!(stats.total_failures, 1);
        assert_eq!(stats.retry_failures, 1);
        assert_eq!(stats.final_failures, 0);
    }

    #[tokio::test]
    async fn test_query_failures() {
        let logger = FailureLogger::new(FailureLoggerConfig::default());

        let message1 = NotificationMessage::new(
            "Test 1".to_string(),
            "Content 1".to_string(),
            NotificationSeverity::Error,
            "/test1".to_string(),
        );

        let message2 = NotificationMessage::new(
            "Test 2".to_string(),
            "Content 2".to_string(),
            NotificationSeverity::Warning,
            "/test2".to_string(),
        );

        let error1 = anyhow::anyhow!("Network error");
        let error2 = anyhow::anyhow!("Server error");

        logger.log_failure(&message1, NotificationPlatform::Email, &error1, 1, true).await.unwrap();
        logger.log_failure(&message2, NotificationPlatform::Feishu, &error2, 2, false).await.unwrap();

        // 查询所有记录
        let all_records = logger.query_failures(None, None, None, None, false).await;
        assert_eq!(all_records.len(), 2);

        // 查询特定平台
        let email_records = logger.query_failures(Some(NotificationPlatform::Email), None, None, None, false).await;
        assert_eq!(email_records.len(), 1);

        // 查询最终失败
        let final_failures = logger.query_failures(None, None, None, None, true).await;
        assert_eq!(final_failures.len(), 1);
    }

    #[tokio::test]
    async fn test_cleanup_old_records() {
        let mut config = FailureLoggerConfig::default();
        config.retention_days = 0; // 立即过期

        let logger = FailureLogger::new(config);

        let message = NotificationMessage::new(
            "Test".to_string(),
            "Content".to_string(),
            NotificationSeverity::Info,
            "/test".to_string(),
        );

        let error = anyhow::anyhow!("Test error");
        logger.log_failure(&message, NotificationPlatform::DingTalk, &error, 0, true).await.unwrap();

        // 等待一小段时间确保记录过期
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        let removed_count = logger.cleanup_old_records().await.unwrap();
        assert_eq!(removed_count, 1);

        let records = logger.get_failure_records().await;
        assert_eq!(records.len(), 0);
    }
}