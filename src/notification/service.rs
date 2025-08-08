use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::notification::{NotificationPlatform, NotificationSeverity};
use crate::notification::rule_engine::{NotificationRuleEngine, ProcessedNotification};
use crate::notification::retry::{RetryManager, RetryConfig as RetryManagerConfig};
use crate::notification::failure_logger::{FailureLogger, FailureLoggerConfig};
use crate::notification::monitoring::{NotificationMonitor, MonitoringConfig};

/// 通知消息结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationMessage {
    pub id: String,
    pub title: String,
    pub content: String,
    pub severity: NotificationSeverity,
    pub project_path: String,
    pub score: Option<f32>,
    pub timestamp: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
    pub template_data: HashMap<String, serde_json::Value>,
}

impl NotificationMessage {
    pub fn new(title: String, content: String, severity: NotificationSeverity, project_path: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            title,
            content,
            severity,
            project_path,
            score: None,
            timestamp: Utc::now(),
            metadata: HashMap::new(),
            template_data: HashMap::new(),
        }
    }

    pub fn with_score(mut self, score: f32) -> Self {
        self.score = Some(score);
        self
    }

    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }

    pub fn with_template_data(mut self, key: String, value: serde_json::Value) -> Self {
        self.template_data.insert(key, value);
        self
    }
}

/// 通知发送结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationResult {
    pub message_id: String,
    pub platform: NotificationPlatform,
    pub success: bool,
    pub error_message: Option<String>,
    pub sent_at: DateTime<Utc>,
    pub retry_count: u32,
}

impl NotificationResult {
    pub fn success(message_id: String, platform: NotificationPlatform) -> Self {
        Self {
            message_id,
            platform,
            success: true,
            error_message: None,
            sent_at: Utc::now(),
            retry_count: 0,
        }
    }

    pub fn failure(message_id: String, platform: NotificationPlatform, error: String, retry_count: u32) -> Self {
        Self {
            message_id,
            platform,
            success: false,
            error_message: Some(error),
            sent_at: Utc::now(),
            retry_count,
        }
    }
}

/// 通知提供商 trait
#[async_trait]
pub trait NotificationProvider: Send + Sync {
    fn platform(&self) -> NotificationPlatform;
    async fn send_notification(&self, message: &NotificationMessage) -> anyhow::Result<NotificationResult>;
    fn is_configured(&self) -> bool;
    fn supports_rich_content(&self) -> bool;
}

/// 通知配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationConfig {
    pub enabled_platforms: Vec<NotificationPlatform>,
    pub retry_config: RetryConfig,
    pub rate_limit: RateLimitConfig,
    pub template_config: TemplateConfig,
    pub rules: Vec<NotificationRule>,
}

impl Default for NotificationConfig {
    fn default() -> Self {
        Self {
            enabled_platforms: vec![],
            retry_config: RetryConfig::default(),
            rate_limit: RateLimitConfig::default(),
            template_config: TemplateConfig::default(),
            rules: vec![],
        }
    }
}

/// 重试配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    pub max_retries: u32,
    pub initial_delay: Duration,
    pub max_delay: Duration,
    pub backoff_multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(60),
            backoff_multiplier: 2.0,
        }
    }
}

/// 频率限制配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub max_notifications_per_minute: u32,
    pub max_notifications_per_hour: u32,
    pub burst_limit: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_notifications_per_minute: 10,
            max_notifications_per_hour: 100,
            burst_limit: 5,
        }
    }
}

/// 模板配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateConfig {
    pub default_template: String,
    pub custom_templates: HashMap<String, String>,
    pub enable_rich_formatting: bool,
}

impl Default for TemplateConfig {
    fn default() -> Self {
        Self {
            default_template: "default".to_string(),
            custom_templates: HashMap::new(),
            enable_rich_formatting: true,
        }
    }
}

/// 通知规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationRule {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub conditions: Vec<NotificationCondition>,
    pub platforms: Vec<NotificationPlatform>,
    pub template: Option<String>,
    pub aggregation: Option<AggregationConfig>,
}

/// 通知条件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationCondition {
    pub field: String,
    pub operator: ConditionOperator,
    pub value: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConditionOperator {
    Equals,
    NotEquals,
    GreaterThan,
    LessThan,
    Contains,
    NotContains,
    Regex,
}

/// 聚合配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregationConfig {
    pub window_duration: Duration,
    pub max_messages: u32,
    pub group_by: Vec<String>,
}

/// 通知队列项
#[derive(Debug, Clone)]
struct NotificationQueueItem {
    message: NotificationMessage,
    platforms: Vec<NotificationPlatform>,
    retry_count: u32,
    scheduled_at: DateTime<Utc>,
}

/// 通知服务
pub struct NotificationService {
    providers: HashMap<NotificationPlatform, Arc<dyn NotificationProvider>>,
    config: NotificationConfig,
    queue: Arc<RwLock<Vec<NotificationQueueItem>>>,
    rate_limiter: Arc<RwLock<RateLimiter>>,
    rule_engine: Arc<NotificationRuleEngine>,
    retry_manager: Arc<RetryManager>,
    failure_logger: Arc<FailureLogger>,
    monitor: Option<Arc<NotificationMonitor>>,
}

impl NotificationService {
    pub fn new(config: NotificationConfig) -> Self {
        // Convert the old RetryConfig to the new RetryManagerConfig
        let retry_manager_config = RetryManagerConfig {
            strategy: crate::notification::retry::RetryStrategy::ExponentialBackoff {
                initial_delay: config.retry_config.initial_delay,
                max_delay: config.retry_config.max_delay,
                backoff_multiplier: config.retry_config.backoff_multiplier,
                max_retries: config.retry_config.max_retries,
                jitter: true,
            },
            condition: crate::notification::retry::RetryCondition::default(),
            enabled: true,
            pre_retry_hook: None,
            post_retry_hook: None,
        };

        let retry_manager = Arc::new(RetryManager::new(retry_manager_config));
        let failure_logger = Arc::new(FailureLogger::new(FailureLoggerConfig::default()));

        Self {
            providers: HashMap::new(),
            config,
            queue: Arc::new(RwLock::new(Vec::new())),
            rate_limiter: Arc::new(RwLock::new(RateLimiter::new())),
            rule_engine: Arc::new(NotificationRuleEngine::new()),
            retry_manager,
            failure_logger,
            monitor: None,
        }
    }

    /// 创建带监控的通知服务
    pub fn new_with_monitoring(
        config: NotificationConfig,
        monitoring_config: MonitoringConfig,
        failure_logger_config: FailureLoggerConfig,
    ) -> Self {
        // Convert the old RetryConfig to the new RetryManagerConfig
        let retry_manager_config = RetryManagerConfig {
            strategy: crate::notification::retry::RetryStrategy::ExponentialBackoff {
                initial_delay: config.retry_config.initial_delay,
                max_delay: config.retry_config.max_delay,
                backoff_multiplier: config.retry_config.backoff_multiplier,
                max_retries: config.retry_config.max_retries,
                jitter: true,
            },
            condition: crate::notification::retry::RetryCondition::default(),
            enabled: true,
            pre_retry_hook: None,
            post_retry_hook: None,
        };

        let retry_manager = Arc::new(RetryManager::new(retry_manager_config));
        let failure_logger = Arc::new(FailureLogger::new(failure_logger_config));
        let monitor = Arc::new(NotificationMonitor::new(
            monitoring_config,
            failure_logger.clone(),
            retry_manager.clone(),
        ));

        Self {
            providers: HashMap::new(),
            config,
            queue: Arc::new(RwLock::new(Vec::new())),
            rate_limiter: Arc::new(RwLock::new(RateLimiter::new())),
            rule_engine: Arc::new(NotificationRuleEngine::new()),
            retry_manager,
            failure_logger,
            monitor: Some(monitor),
        }
    }

    /// 注册通知提供商
    pub fn register_provider(&mut self, provider: Arc<dyn NotificationProvider>) {
        let platform = provider.platform();
        self.providers.insert(platform, provider);
    }

    /// 发送通知
    pub async fn send_notification(&self, message: NotificationMessage) -> anyhow::Result<Vec<NotificationResult>> {
        // 使用规则引擎处理通知
        let processed_notifications = self.rule_engine.process_notification(message).await?;

        if processed_notifications.is_empty() {
            return Ok(vec![]);
        }

        let mut all_results = Vec::new();

        // 发送每个处理后的通知
        for processed in processed_notifications {
            let mut results = Vec::new();

            for platform in &processed.platforms {
                if let Some(provider) = self.providers.get(platform) {
                    if provider.is_configured() {
                        // 如果是聚合消息，使用聚合后的内容
                        let message_to_send = if processed.aggregated_messages.len() > 1 {
                            self.create_aggregated_message(&processed)?
                        } else {
                            processed.original_message.clone()
                        };

                        match self.retry_manager.send_with_retry(provider.clone(), &message_to_send).await {
                            Ok(result) => {
                                results.push(result);
                            },
                            Err(e) => {
                                // 记录失败到失败日志
                                if let Err(log_err) = self.failure_logger.log_failure(
                                    &message_to_send,
                                    platform.clone(),
                                    &e,
                                    self.retry_manager.get_config().strategy.get_max_retries(),
                                    true,
                                ).await {
                                    log::error!("Failed to log notification failure: {}", log_err);
                                }

                                log::error!("Failed to send notification to {:?}: {}", platform, e);
                                results.push(NotificationResult::failure(
                                    message_to_send.id.clone(),
                                    platform.clone(),
                                    e.to_string(),
                                    self.retry_manager.get_config().strategy.get_max_retries(),
                                ));
                            }
                        }
                    }
                }
            }

            all_results.extend(results);
        }

        Ok(all_results)
    }

    /// 批量发送通知
    pub async fn send_batch_notifications(&self, messages: Vec<NotificationMessage>) -> anyhow::Result<Vec<Vec<NotificationResult>>> {
        let mut all_results = Vec::new();

        for message in messages {
            let results = self.send_notification(message).await?;
            all_results.push(results);
        }

        Ok(all_results)
    }

    /// 应用通知规则
    async fn apply_rules(&self, message: &NotificationMessage) -> anyhow::Result<Vec<NotificationPlatform>> {
        let mut applicable_platforms = Vec::new();

        for rule in &self.config.rules {
            if !rule.enabled {
                continue;
            }

            if self.evaluate_conditions(&rule.conditions, message).await? {
                applicable_platforms.extend(rule.platforms.clone());
            }
        }

        // 如果没有规则匹配，使用默认平台
        if applicable_platforms.is_empty() {
            applicable_platforms = self.config.enabled_platforms.clone();
        }

        // 去重
        applicable_platforms.sort();
        applicable_platforms.dedup();

        Ok(applicable_platforms)
    }

    /// 评估通知条件
    async fn evaluate_conditions(&self, conditions: &[NotificationCondition], message: &NotificationMessage) -> anyhow::Result<bool> {
        for condition in conditions {
            if !self.evaluate_condition(condition, message).await? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// 评估单个条件
    async fn evaluate_condition(&self, condition: &NotificationCondition, message: &NotificationMessage) -> anyhow::Result<bool> {
        let field_value = match condition.field.as_str() {
            "severity" => serde_json::to_value(&message.severity)?,
            "score" => serde_json::to_value(&message.score)?,
            "project_path" => serde_json::to_value(&message.project_path)?,
            "title" => serde_json::to_value(&message.title)?,
            "content" => serde_json::to_value(&message.content)?,
            field => message.metadata.get(field)
                .map(|v| serde_json::to_value(v))
                .transpose()?
                .unwrap_or(serde_json::Value::Null),
        };

        match condition.operator {
            ConditionOperator::Equals => Ok(field_value == condition.value),
            ConditionOperator::NotEquals => Ok(field_value != condition.value),
            ConditionOperator::GreaterThan => {
                if let (Some(field_num), Some(condition_num)) = (field_value.as_f64(), condition.value.as_f64()) {
                    Ok(field_num > condition_num)
                } else {
                    Ok(false)
                }
            },
            ConditionOperator::LessThan => {
                if let (Some(field_num), Some(condition_num)) = (field_value.as_f64(), condition.value.as_f64()) {
                    Ok(field_num < condition_num)
                } else {
                    Ok(false)
                }
            },
            ConditionOperator::Contains => {
                if let (Some(field_str), Some(condition_str)) = (field_value.as_str(), condition.value.as_str()) {
                    Ok(field_str.contains(condition_str))
                } else {
                    Ok(false)
                }
            },
            ConditionOperator::NotContains => {
                if let (Some(field_str), Some(condition_str)) = (field_value.as_str(), condition.value.as_str()) {
                    Ok(!field_str.contains(condition_str))
                } else {
                    Ok(true)
                }
            },
            ConditionOperator::Regex => {
                if let (Some(field_str), Some(pattern)) = (field_value.as_str(), condition.value.as_str()) {
                    match regex::Regex::new(pattern) {
                        Ok(re) => Ok(re.is_match(field_str)),
                        Err(_) => Ok(false),
                    }
                } else {
                    Ok(false)
                }
            },
        }
    }



    /// 检查频率限制
    async fn check_rate_limit(&self) -> anyhow::Result<bool> {
        let mut limiter = self.rate_limiter.write().await;
        Ok(limiter.allow_request())
    }

    /// 获取配置
    pub fn get_config(&self) -> &NotificationConfig {
        &self.config
    }

    /// 更新配置
    pub fn update_config(&mut self, config: NotificationConfig) {
        self.config = config;
    }

    /// 获取已注册的提供商
    pub fn get_providers(&self) -> Vec<NotificationPlatform> {
        self.providers.keys().cloned().collect()
    }

    /// 检查提供商是否可用
    pub fn is_provider_available(&self, platform: &NotificationPlatform) -> bool {
        self.providers.get(platform)
            .map(|provider| provider.is_configured())
            .unwrap_or(false)
    }

    /// 创建聚合消息
    fn create_aggregated_message(&self, processed: &ProcessedNotification) -> anyhow::Result<NotificationMessage> {
        let message_count = processed.aggregated_messages.len();
        let first_message = &processed.original_message;

        // 创建聚合标题
        let aggregated_title = if message_count > 1 {
            format!("{} (聚合了 {} 条消息)", first_message.title, message_count)
        } else {
            first_message.title.clone()
        };

        // 创建聚合内容
        let mut aggregated_content = format!("聚合了 {} 条通知消息:\n\n", message_count);

        for (i, msg) in processed.aggregated_messages.iter().enumerate() {
            aggregated_content.push_str(&format!("{}. [{}] {}\n",
                i + 1,
                format!("{:?}", msg.severity),
                msg.content
            ));

            if i < processed.aggregated_messages.len() - 1 {
                aggregated_content.push('\n');
            }
        }

        // 计算平均分数
        let avg_score = if processed.aggregated_messages.iter().any(|m| m.score.is_some()) {
            let scores: Vec<f32> = processed.aggregated_messages
                .iter()
                .filter_map(|m| m.score)
                .collect();

            if !scores.is_empty() {
                Some(scores.iter().sum::<f32>() / scores.len() as f32)
            } else {
                None
            }
        } else {
            None
        };

        // 合并元数据
        let mut merged_metadata = first_message.metadata.clone();
        merged_metadata.insert("aggregated_count".to_string(), message_count.to_string());
        merged_metadata.insert("aggregation_rule".to_string(), processed.rule.name.clone());

        // 合并模板数据
        let mut merged_template_data = first_message.template_data.clone();
        merged_template_data.insert("aggregated_messages".to_string(),
            serde_json::to_value(&processed.aggregated_messages)?);

        Ok(NotificationMessage {
            id: processed.id.clone(),
            title: aggregated_title,
            content: aggregated_content,
            severity: first_message.severity.clone(),
            project_path: first_message.project_path.clone(),
            score: avg_score,
            timestamp: processed.processed_at,
            metadata: merged_metadata,
            template_data: merged_template_data,
        })
    }

    /// 获取规则引擎
    pub fn get_rule_engine(&self) -> Arc<NotificationRuleEngine> {
        self.rule_engine.clone()
    }

    /// 添加通知规则
    pub async fn add_rule(&self, rule: crate::notification::rule_engine::NotificationRule) -> anyhow::Result<()> {
        self.rule_engine.add_rule(rule).await
    }

    /// 更新通知规则
    pub async fn update_rule(&self, rule_id: &str, rule: crate::notification::rule_engine::NotificationRule) -> anyhow::Result<()> {
        self.rule_engine.update_rule(rule_id, rule).await
    }

    /// 删除通知规则
    pub async fn remove_rule(&self, rule_id: &str) -> anyhow::Result<()> {
        self.rule_engine.remove_rule(rule_id).await
    }

    /// 获取所有规则
    pub async fn get_rules(&self) -> Vec<crate::notification::rule_engine::NotificationRule> {
        self.rule_engine.get_rules().await
    }

    /// 获取规则引擎统计信息
    pub async fn get_rule_statistics(&self) -> crate::notification::rule_engine::RuleEngineStatistics {
        self.rule_engine.get_rule_statistics().await
    }

    /// 获取重试管理器
    pub fn get_retry_manager(&self) -> Arc<RetryManager> {
        self.retry_manager.clone()
    }

    /// 获取失败日志记录器
    pub fn get_failure_logger(&self) -> Arc<FailureLogger> {
        self.failure_logger.clone()
    }

    /// 获取监控系统
    pub fn get_monitor(&self) -> Option<Arc<NotificationMonitor>> {
        self.monitor.clone()
    }

    /// 启动监控系统
    pub async fn start_monitoring(&self) -> anyhow::Result<()> {
        if let Some(ref monitor) = self.monitor {
            monitor.start_monitoring().await
        } else {
            log::warn!("Monitoring system not configured");
            Ok(())
        }
    }

    /// 获取重试统计信息
    pub async fn get_retry_statistics(&self) -> crate::notification::retry::RetryStatistics {
        self.retry_manager.get_statistics().await
    }

    /// 获取失败统计信息
    pub async fn get_failure_statistics(&self) -> crate::notification::failure_logger::FailureStatistics {
        self.failure_logger.get_statistics().await
    }

    /// 获取监控统计信息
    pub async fn get_monitoring_statistics(&self) -> Option<crate::notification::monitoring::MonitoringStatistics> {
        if let Some(ref monitor) = self.monitor {
            Some(monitor.get_statistics().await)
        } else {
            None
        }
    }

    /// 获取活跃告警
    pub async fn get_active_alerts(&self) -> Vec<crate::notification::monitoring::Alert> {
        if let Some(ref monitor) = self.monitor {
            monitor.get_active_alerts().await
        } else {
            vec![]
        }
    }

    /// 清理过期记录
    pub async fn cleanup_old_records(&self, max_age: Duration) -> anyhow::Result<()> {
        // 清理重试记录
        self.retry_manager.cleanup_old_records(max_age).await;

        // 清理失败记录
        self.failure_logger.cleanup_old_records().await?;

        Ok(())
    }

    /// 导出失败记录
    pub async fn export_failure_records(
        &self,
        file_path: &std::path::PathBuf,
        format: crate::notification::failure_logger::ExportFormat,
    ) -> anyhow::Result<()> {
        self.failure_logger.export_records(file_path, format).await
    }

    /// 抑制告警
    pub async fn suppress_alert(&self, alert_id: &str, duration: Duration) -> anyhow::Result<()> {
        if let Some(ref monitor) = self.monitor {
            monitor.suppress_alert(alert_id, duration).await
        } else {
            anyhow::bail!("Monitoring system not configured")
        }
    }

    /// 解决告警
    pub async fn resolve_alert(&self, alert_id: &str) -> anyhow::Result<()> {
        if let Some(ref monitor) = self.monitor {
            monitor.resolve_alert(alert_id).await
        } else {
            anyhow::bail!("Monitoring system not configured")
        }
    }
}

/// 简单的频率限制器
struct RateLimiter {
    requests: Vec<DateTime<Utc>>,
}

impl RateLimiter {
    fn new() -> Self {
        Self {
            requests: Vec::new(),
        }
    }

    fn allow_request(&mut self) -> bool {
        let now = Utc::now();

        // 清理过期的请求记录
        self.requests.retain(|&timestamp| {
            now.signed_duration_since(timestamp).num_minutes() < 1
        });

        // 检查是否超过限制
        if self.requests.len() >= 10 { // 每分钟最多10个请求
            return false;
        }

        self.requests.push(now);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_notification_message_creation() {
        let message = NotificationMessage::new(
            "Test Title".to_string(),
            "Test Content".to_string(),
            NotificationSeverity::Info,
            "/test/project".to_string(),
        );

        assert_eq!(message.title, "Test Title");
        assert_eq!(message.content, "Test Content");
        assert_eq!(message.project_path, "/test/project");
        assert!(message.score.is_none());
    }

    #[tokio::test]
    async fn test_notification_message_with_score() {
        let message = NotificationMessage::new(
            "Test Title".to_string(),
            "Test Content".to_string(),
            NotificationSeverity::Warning,
            "/test/project".to_string(),
        ).with_score(8.5);

        assert_eq!(message.score, Some(8.5));
    }

    #[tokio::test]
    async fn test_notification_result_success() {
        let result = NotificationResult::success(
            "test-id".to_string(),
            NotificationPlatform::Feishu,
        );

        assert!(result.success);
        assert!(result.error_message.is_none());
        assert_eq!(result.retry_count, 0);
    }

    #[tokio::test]
    async fn test_notification_result_failure() {
        let result = NotificationResult::failure(
            "test-id".to_string(),
            NotificationPlatform::Email,
            "Connection failed".to_string(),
            2,
        );

        assert!(!result.success);
        assert_eq!(result.error_message, Some("Connection failed".to_string()));
        assert_eq!(result.retry_count, 2);
    }

    #[tokio::test]
    async fn test_rate_limiter() {
        let mut limiter = RateLimiter::new();

        // 应该允许前10个请求
        for _ in 0..10 {
            assert!(limiter.allow_request());
        }

        // 第11个请求应该被拒绝
        assert!(!limiter.allow_request());
    }
}