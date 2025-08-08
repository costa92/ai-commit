use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::notification::{NotificationMessage, NotificationResult, NotificationPlatform, NotificationProvider};

/// 重试策略类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RetryStrategy {
    /// 固定延迟重试
    FixedDelay {
        delay: Duration,
        max_retries: u32,
    },
    /// 指数退避重试
    ExponentialBackoff {
        initial_delay: Duration,
        max_delay: Duration,
        backoff_multiplier: f64,
        max_retries: u32,
        jitter: bool,
    },
    /// 线性退避重试
    LinearBackoff {
        initial_delay: Duration,
        increment: Duration,
        max_delay: Duration,
        max_retries: u32,
    },
    /// 自定义延迟序列
    CustomDelays {
        delays: Vec<Duration>,
    },
}

impl Default for RetryStrategy {
    fn default() -> Self {
        Self::ExponentialBackoff {
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(60),
            backoff_multiplier: 2.0,
            max_retries: 3,
            jitter: true,
        }
    }
}

impl RetryStrategy {
    pub fn get_max_retries(&self) -> u32 {
        match self {
            RetryStrategy::FixedDelay { max_retries, .. } => *max_retries,
            RetryStrategy::ExponentialBackoff { max_retries, .. } => *max_retries,
            RetryStrategy::LinearBackoff { max_retries, .. } => *max_retries,
            RetryStrategy::CustomDelays { delays } => delays.len() as u32,
        }
    }
}

/// 重试条件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryCondition {
    /// 错误类型匹配
    pub error_patterns: Vec<String>,
    /// HTTP 状态码匹配
    pub http_status_codes: Vec<u16>,
    /// 是否重试网络错误
    pub retry_network_errors: bool,
    /// 是否重试超时错误
    pub retry_timeout_errors: bool,
    /// 是否重试服务器错误 (5xx)
    pub retry_server_errors: bool,
    /// 自定义重试判断函数名
    pub custom_condition: Option<String>,
}

impl Default for RetryCondition {
    fn default() -> Self {
        Self {
            error_patterns: vec![
                "connection".to_string(),
                "timeout".to_string(),
                "network".to_string(),
                "temporary".to_string(),
            ],
            http_status_codes: vec![429, 500, 502, 503, 504],
            retry_network_errors: true,
            retry_timeout_errors: true,
            retry_server_errors: true,
            custom_condition: None,
        }
    }
}

/// 重试配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// 重试策略
    pub strategy: RetryStrategy,
    /// 重试条件
    pub condition: RetryCondition,
    /// 是否启用重试
    pub enabled: bool,
    /// 重试前的预处理
    pub pre_retry_hook: Option<String>,
    /// 重试后的后处理
    pub post_retry_hook: Option<String>,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            strategy: RetryStrategy::default(),
            condition: RetryCondition::default(),
            enabled: true,
            pre_retry_hook: None,
            post_retry_hook: None,
        }
    }
}

/// 重试记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryRecord {
    pub id: String,
    pub message_id: String,
    pub platform: NotificationPlatform,
    pub attempt_number: u32,
    pub error_message: String,
    pub error_type: String,
    pub retry_delay: Duration,
    pub attempted_at: DateTime<Utc>,
    pub next_retry_at: Option<DateTime<Utc>>,
    pub final_attempt: bool,
}

impl RetryRecord {
    pub fn new(
        message_id: String,
        platform: NotificationPlatform,
        attempt_number: u32,
        error_message: String,
        error_type: String,
        retry_delay: Duration,
        next_retry_at: Option<DateTime<Utc>>,
        final_attempt: bool,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            message_id,
            platform,
            attempt_number,
            error_message,
            error_type,
            retry_delay,
            attempted_at: Utc::now(),
            next_retry_at,
            final_attempt,
        }
    }
}

/// 重试统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryStatistics {
    pub total_retries: u64,
    pub successful_retries: u64,
    pub failed_retries: u64,
    pub retry_rate: f64,
    pub average_retry_count: f64,
    pub platform_statistics: HashMap<NotificationPlatform, PlatformRetryStats>,
    pub error_type_statistics: HashMap<String, u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformRetryStats {
    pub total_attempts: u64,
    pub successful_retries: u64,
    pub failed_retries: u64,
    pub average_retry_delay: Duration,
    pub max_retry_delay: Duration,
}

/// 重试管理器
pub struct RetryManager {
    config: RetryConfig,
    retry_records: Arc<RwLock<Vec<RetryRecord>>>,
    statistics: Arc<RwLock<RetryStatistics>>,
}

impl RetryManager {
    pub fn new(config: RetryConfig) -> Self {
        Self {
            config,
            retry_records: Arc::new(RwLock::new(Vec::new())),
            statistics: Arc::new(RwLock::new(RetryStatistics {
                total_retries: 0,
                successful_retries: 0,
                failed_retries: 0,
                retry_rate: 0.0,
                average_retry_count: 0.0,
                platform_statistics: HashMap::new(),
                error_type_statistics: HashMap::new(),
            })),
        }
    }

    /// 执行带重试的通知发送
    pub async fn send_with_retry(
        &self,
        provider: Arc<dyn NotificationProvider>,
        message: &NotificationMessage,
    ) -> anyhow::Result<NotificationResult> {
        if !self.config.enabled {
            return provider.send_notification(message).await;
        }

        let mut attempt = 0;
        let platform = provider.platform();

        loop {
            attempt += 1;

            // 执行预重试钩子
            if attempt > 1 {
                if let Some(hook) = &self.config.pre_retry_hook {
                    self.execute_hook(hook, message, attempt).await?;
                }
            }

            match provider.send_notification(message).await {
                Ok(mut result) => {
                    result.retry_count = attempt - 1;

                    // 更新统计信息
                    if attempt > 1 {
                        self.update_success_statistics(platform.clone(), attempt - 1).await;
                    }

                    // 执行后重试钩子
                    if attempt > 1 {
                        if let Some(hook) = &self.config.post_retry_hook {
                            self.execute_hook(hook, message, attempt).await?;
                        }
                    }

                    return Ok(result);
                }
                Err(e) => {
                    let error_type = self.classify_error(&e);
                    let should_retry = self.should_retry(&e, attempt).await;

                    if !should_retry {
                        // 记录最终失败
                        let record = RetryRecord::new(
                            message.id.clone(),
                            platform.clone(),
                            attempt,
                            e.to_string(),
                            error_type.clone(),
                            Duration::from_secs(0),
                            None,
                            true,
                        );

                        self.record_retry_attempt(record).await;
                        self.update_failure_statistics(platform.clone(), attempt, error_type).await;

                        return Err(e);
                    }

                    // 计算重试延迟
                    let delay = self.calculate_retry_delay(attempt).await;
                    let next_retry_at = Some(Utc::now() + chrono::Duration::from_std(delay).unwrap());

                    // 记录重试尝试
                    let record = RetryRecord::new(
                        message.id.clone(),
                        platform.clone(),
                        attempt,
                        e.to_string(),
                        error_type.clone(),
                        delay,
                        next_retry_at,
                        false,
                    );

                    self.record_retry_attempt(record).await;

                    log::warn!(
                        "Notification send failed (attempt {}), retrying in {:?}: {}",
                        attempt,
                        delay,
                        e
                    );

                    // 等待重试延迟
                    tokio::time::sleep(delay).await;
                }
            }
        }
    }

    /// 判断是否应该重试
    async fn should_retry(&self, error: &anyhow::Error, attempt: u32) -> bool {
        // 检查是否超过最大重试次数
        let max_retries = self.get_max_retries().await;
        if attempt >= max_retries {
            return false;
        }

        let error_str = error.to_string().to_lowercase();

        // 检查错误模式匹配
        for pattern in &self.config.condition.error_patterns {
            if error_str.contains(&pattern.to_lowercase()) {
                return true;
            }
        }

        // 检查网络错误
        if self.config.condition.retry_network_errors {
            if error_str.contains("network") || error_str.contains("connection") {
                return true;
            }
        }

        // 检查超时错误
        if self.config.condition.retry_timeout_errors {
            if error_str.contains("timeout") || error_str.contains("timed out") {
                return true;
            }
        }

        // 检查服务器错误
        if self.config.condition.retry_server_errors {
            if error_str.contains("server error") || error_str.contains("5") {
                return true;
            }
        }

        // 检查 HTTP 状态码
        for &status_code in &self.config.condition.http_status_codes {
            if error_str.contains(&status_code.to_string()) {
                return true;
            }
        }

        false
    }

    /// 计算重试延迟
    async fn calculate_retry_delay(&self, attempt: u32) -> Duration {
        match &self.config.strategy {
            RetryStrategy::FixedDelay { delay, .. } => *delay,
            RetryStrategy::ExponentialBackoff {
                initial_delay,
                max_delay,
                backoff_multiplier,
                jitter,
                ..
            } => {
                let base_delay = initial_delay.as_millis() as f64
                    * backoff_multiplier.powi((attempt - 1) as i32);

                let mut delay = Duration::from_millis(base_delay as u64);

                // 应用最大延迟限制
                if delay > *max_delay {
                    delay = *max_delay;
                }

                // 应用抖动
                if *jitter {
                    let jitter_range = delay.as_millis() as f64 * 0.1; // 10% 抖动
                    let jitter_offset = (rand::random::<f64>() - 0.5) * 2.0 * jitter_range;
                    let jittered_delay = (delay.as_millis() as f64 + jitter_offset).max(0.0);
                    delay = Duration::from_millis(jittered_delay as u64);
                }

                delay
            }
            RetryStrategy::LinearBackoff {
                initial_delay,
                increment,
                max_delay,
                ..
            } => {
                let delay = *initial_delay + *increment * (attempt - 1);
                std::cmp::min(delay, *max_delay)
            }
            RetryStrategy::CustomDelays { delays } => {
                let index = std::cmp::min((attempt - 1) as usize, delays.len() - 1);
                delays.get(index).copied().unwrap_or(Duration::from_secs(1))
            }
        }
    }

    /// 获取最大重试次数
    async fn get_max_retries(&self) -> u32 {
        match &self.config.strategy {
            RetryStrategy::FixedDelay { max_retries, .. } => *max_retries,
            RetryStrategy::ExponentialBackoff { max_retries, .. } => *max_retries,
            RetryStrategy::LinearBackoff { max_retries, .. } => *max_retries,
            RetryStrategy::CustomDelays { delays } => delays.len() as u32,
        }
    }

    /// 分类错误类型
    fn classify_error(&self, error: &anyhow::Error) -> String {
        let error_str = error.to_string().to_lowercase();

        if error_str.contains("timeout") || error_str.contains("timed out") {
            "timeout".to_string()
        } else if error_str.contains("network") || error_str.contains("connection") {
            "network".to_string()
        } else if error_str.contains("server") || error_str.contains("5") {
            "server_error".to_string()
        } else if error_str.contains("rate limit") || error_str.contains("429") {
            "rate_limit".to_string()
        } else if error_str.contains("auth") || error_str.contains("401") || error_str.contains("403") {
            "authentication".to_string()
        } else {
            "unknown".to_string()
        }
    }

    /// 记录重试尝试
    async fn record_retry_attempt(&self, record: RetryRecord) {
        let mut records = self.retry_records.write().await;
        records.push(record);

        // 保持记录数量在合理范围内
        if records.len() > 10000 {
            records.drain(0..1000);
        }
    }

    /// 更新成功统计信息
    async fn update_success_statistics(&self, platform: NotificationPlatform, retry_count: u32) {
        let mut stats = self.statistics.write().await;
        stats.total_retries += retry_count as u64;
        stats.successful_retries += 1;

        let platform_stats = stats.platform_statistics.entry(platform).or_insert(PlatformRetryStats {
            total_attempts: 0,
            successful_retries: 0,
            failed_retries: 0,
            average_retry_delay: Duration::from_secs(0),
            max_retry_delay: Duration::from_secs(0),
        });

        platform_stats.total_attempts += retry_count as u64;
        platform_stats.successful_retries += 1;

        // 重新计算重试率
        let total_attempts = stats.successful_retries + stats.failed_retries;
        if total_attempts > 0 {
            stats.retry_rate = stats.total_retries as f64 / total_attempts as f64;
            stats.average_retry_count = stats.total_retries as f64 / total_attempts as f64;
        }
    }

    /// 更新失败统计信息
    async fn update_failure_statistics(&self, platform: NotificationPlatform, retry_count: u32, error_type: String) {
        let mut stats = self.statistics.write().await;
        stats.total_retries += retry_count as u64;
        stats.failed_retries += 1;

        let platform_stats = stats.platform_statistics.entry(platform).or_insert(PlatformRetryStats {
            total_attempts: 0,
            successful_retries: 0,
            failed_retries: 0,
            average_retry_delay: Duration::from_secs(0),
            max_retry_delay: Duration::from_secs(0),
        });

        platform_stats.total_attempts += retry_count as u64;
        platform_stats.failed_retries += 1;

        // 更新错误类型统计
        *stats.error_type_statistics.entry(error_type).or_insert(0) += 1;

        // 重新计算重试率
        let total_attempts = stats.successful_retries + stats.failed_retries;
        if total_attempts > 0 {
            stats.retry_rate = stats.total_retries as f64 / total_attempts as f64;
            stats.average_retry_count = stats.total_retries as f64 / total_attempts as f64;
        }
    }

    /// 执行钩子函数
    async fn execute_hook(&self, hook: &str, message: &NotificationMessage, attempt: u32) -> anyhow::Result<()> {
        // 这里可以实现钩子函数的执行逻辑
        // 例如调用外部脚本、发送监控事件等
        log::debug!("Executing hook '{}' for message {} (attempt {})", hook, message.id, attempt);
        Ok(())
    }

    /// 获取重试记录
    pub async fn get_retry_records(&self) -> Vec<RetryRecord> {
        self.retry_records.read().await.clone()
    }

    /// 获取重试统计信息
    pub async fn get_statistics(&self) -> RetryStatistics {
        self.statistics.read().await.clone()
    }

    /// 清理过期的重试记录
    pub async fn cleanup_old_records(&self, max_age: Duration) {
        let mut records = self.retry_records.write().await;
        let cutoff_time = Utc::now() - chrono::Duration::from_std(max_age).unwrap();

        records.retain(|record| record.attempted_at > cutoff_time);
    }

    /// 获取配置
    pub fn get_config(&self) -> &RetryConfig {
        &self.config
    }

    /// 更新配置
    pub fn update_config(&mut self, config: RetryConfig) {
        self.config = config;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    struct MockProvider {
        platform: NotificationPlatform,
        fail_count: Arc<AtomicU32>,
        max_failures: u32,
    }

    impl MockProvider {
        fn new(platform: NotificationPlatform, max_failures: u32) -> Self {
            Self {
                platform,
                fail_count: Arc::new(AtomicU32::new(0)),
                max_failures,
            }
        }
    }

    #[async_trait::async_trait]
    impl NotificationProvider for MockProvider {
        fn platform(&self) -> NotificationPlatform {
            self.platform.clone()
        }

        async fn send_notification(&self, message: &NotificationMessage) -> anyhow::Result<NotificationResult> {
            let current_count = self.fail_count.fetch_add(1, Ordering::SeqCst);

            if current_count < self.max_failures {
                anyhow::bail!("Mock network error");
            }

            Ok(NotificationResult::success(message.id.clone(), self.platform.clone()))
        }

        fn is_configured(&self) -> bool {
            true
        }

        fn supports_rich_content(&self) -> bool {
            false
        }
    }

    #[tokio::test]
    async fn test_exponential_backoff_calculation() {
        let config = RetryConfig {
            strategy: RetryStrategy::ExponentialBackoff {
                initial_delay: Duration::from_millis(100),
                max_delay: Duration::from_secs(10),
                backoff_multiplier: 2.0,
                max_retries: 5,
                jitter: false,
            },
            ..Default::default()
        };

        let manager = RetryManager::new(config);

        let delay1 = manager.calculate_retry_delay(1).await;
        let delay2 = manager.calculate_retry_delay(2).await;
        let delay3 = manager.calculate_retry_delay(3).await;

        assert_eq!(delay1, Duration::from_millis(100));
        assert_eq!(delay2, Duration::from_millis(200));
        assert_eq!(delay3, Duration::from_millis(400));
    }

    #[tokio::test]
    async fn test_retry_with_success() {
        let config = RetryConfig {
            strategy: RetryStrategy::FixedDelay {
                delay: Duration::from_millis(10),
                max_retries: 3,
            },
            ..Default::default()
        };

        let manager = RetryManager::new(config);
        let provider = Arc::new(MockProvider::new(NotificationPlatform::Email, 2));

        let message = NotificationMessage::new(
            "Test".to_string(),
            "Content".to_string(),
            crate::notification::NotificationSeverity::Info,
            "/test".to_string(),
        );

        let result = manager.send_with_retry(provider, &message).await;
        assert!(result.is_ok());

        let result = result.unwrap();
        assert!(result.success);
        assert_eq!(result.retry_count, 2);
    }

    #[tokio::test]
    async fn test_retry_with_failure() {
        let config = RetryConfig {
            strategy: RetryStrategy::FixedDelay {
                delay: Duration::from_millis(10),
                max_retries: 2,
            },
            ..Default::default()
        };

        let manager = RetryManager::new(config);
        let provider = Arc::new(MockProvider::new(NotificationPlatform::Email, 5));

        let message = NotificationMessage::new(
            "Test".to_string(),
            "Content".to_string(),
            crate::notification::NotificationSeverity::Info,
            "/test".to_string(),
        );

        let result = manager.send_with_retry(provider, &message).await;
        assert!(result.is_err());

        let records = manager.get_retry_records().await;
        assert_eq!(records.len(), 2);
        assert!(records.last().unwrap().final_attempt);
    }

    #[tokio::test]
    async fn test_error_classification() {
        let manager = RetryManager::new(RetryConfig::default());

        let timeout_error = anyhow::anyhow!("Request timed out");
        let network_error = anyhow::anyhow!("Network connection failed");
        let server_error = anyhow::anyhow!("Internal server error");

        assert_eq!(manager.classify_error(&timeout_error), "timeout");
        assert_eq!(manager.classify_error(&network_error), "network");
        assert_eq!(manager.classify_error(&server_error), "server_error");
    }

    #[tokio::test]
    async fn test_statistics_update() {
        let manager = RetryManager::new(RetryConfig::default());

        manager.update_success_statistics(NotificationPlatform::Email, 2).await;
        manager.update_failure_statistics(NotificationPlatform::Feishu, 3, "network".to_string()).await;

        let stats = manager.get_statistics().await;
        assert_eq!(stats.total_retries, 5);
        assert_eq!(stats.successful_retries, 1);
        assert_eq!(stats.failed_retries, 1);
        assert_eq!(stats.retry_rate, 2.5);
    }
}