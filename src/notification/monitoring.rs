use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tokio::time::interval;
use uuid::Uuid;

use crate::notification::{NotificationPlatform, NotificationSeverity};
use crate::notification::failure_logger::{FailureLogger, FailureStatistics};
use crate::notification::retry::{RetryManager, RetryStatistics};

/// 监控指标类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetricType {
    /// 通知发送成功率
    SuccessRate,
    /// 通知发送失败率
    FailureRate,
    /// 平均重试次数
    AverageRetryCount,
    /// 平均响应时间
    AverageResponseTime,
    /// 错误率趋势
    ErrorRateTrend,
    /// 平台可用性
    PlatformAvailability,
    /// 队列长度
    QueueLength,
    /// 吞吐量
    Throughput,
}

/// 监控阈值
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringThreshold {
    pub metric_type: MetricType,
    pub platform: Option<NotificationPlatform>,
    pub warning_threshold: f64,
    pub critical_threshold: f64,
    pub evaluation_window: Duration,
    pub min_sample_size: u32,
}

/// 告警级别
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AlertLevel {
    Info,
    Warning,
    Critical,
}

/// 告警状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AlertStatus {
    Active,
    Resolved,
    Suppressed,
}

/// 告警记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id: String,
    pub alert_type: String,
    pub level: AlertLevel,
    pub status: AlertStatus,
    pub title: String,
    pub description: String,
    pub metric_type: MetricType,
    pub platform: Option<NotificationPlatform>,
    pub current_value: f64,
    pub threshold_value: f64,
    pub triggered_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub suppressed_until: Option<DateTime<Utc>>,
    pub metadata: HashMap<String, String>,
}

impl Alert {
    pub fn new(
        alert_type: String,
        level: AlertLevel,
        title: String,
        description: String,
        metric_type: MetricType,
        platform: Option<NotificationPlatform>,
        current_value: f64,
        threshold_value: f64,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            alert_type,
            level,
            status: AlertStatus::Active,
            title,
            description,
            metric_type,
            platform,
            current_value,
            threshold_value,
            triggered_at: Utc::now(),
            resolved_at: None,
            suppressed_until: None,
            metadata: HashMap::new(),
        }
    }

    pub fn resolve(&mut self) {
        self.status = AlertStatus::Resolved;
        self.resolved_at = Some(Utc::now());
    }

    pub fn suppress(&mut self, duration: Duration) {
        self.status = AlertStatus::Suppressed;
        self.suppressed_until = Some(Utc::now() + chrono::Duration::from_std(duration).unwrap());
    }

    pub fn is_suppressed(&self) -> bool {
        if let Some(suppressed_until) = self.suppressed_until {
            Utc::now() < suppressed_until
        } else {
            false
        }
    }
}

/// 监控统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringStatistics {
    pub total_notifications_sent: u64,
    pub successful_notifications: u64,
    pub failed_notifications: u64,
    pub success_rate: f64,
    pub failure_rate: f64,
    pub average_retry_count: f64,
    pub average_response_time: Duration,
    pub platform_statistics: HashMap<NotificationPlatform, PlatformMonitoringStats>,
    pub active_alerts: u32,
    pub resolved_alerts: u32,
    pub suppressed_alerts: u32,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformMonitoringStats {
    pub total_sent: u64,
    pub successful: u64,
    pub failed: u64,
    pub success_rate: f64,
    pub average_response_time: Duration,
    pub availability: f64,
    pub last_success: Option<DateTime<Utc>>,
    pub last_failure: Option<DateTime<Utc>>,
}

/// 监控配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    pub enabled: bool,
    pub collection_interval: Duration,
    pub alert_evaluation_interval: Duration,
    pub thresholds: Vec<MonitoringThreshold>,
    pub alert_cooldown: Duration,
    pub max_alerts_per_hour: u32,
    pub enable_self_monitoring: bool,
    pub webhook_url: Option<String>,
    pub email_recipients: Vec<String>,
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            collection_interval: Duration::from_secs(60),
            alert_evaluation_interval: Duration::from_secs(30),
            thresholds: vec![
                MonitoringThreshold {
                    metric_type: MetricType::SuccessRate,
                    platform: None,
                    warning_threshold: 0.95,
                    critical_threshold: 0.90,
                    evaluation_window: Duration::from_secs(300), // 5 minutes
                    min_sample_size: 10,
                },
                MonitoringThreshold {
                    metric_type: MetricType::FailureRate,
                    platform: None,
                    warning_threshold: 0.05,
                    critical_threshold: 0.10,
                    evaluation_window: Duration::from_secs(300), // 5 minutes
                    min_sample_size: 10,
                },
                MonitoringThreshold {
                    metric_type: MetricType::AverageRetryCount,
                    platform: None,
                    warning_threshold: 2.0,
                    critical_threshold: 3.0,
                    evaluation_window: Duration::from_secs(600), // 10 minutes
                    min_sample_size: 5,
                },
            ],
            alert_cooldown: Duration::from_secs(900), // 15 minutes
            max_alerts_per_hour: 10,
            enable_self_monitoring: true,
            webhook_url: None,
            email_recipients: vec![],
        }
    }
}

/// 通知监控系统
#[derive(Clone)]
pub struct NotificationMonitor {
    config: MonitoringConfig,
    failure_logger: Arc<FailureLogger>,
    retry_manager: Arc<RetryManager>,
    statistics: Arc<RwLock<MonitoringStatistics>>,
    alerts: Arc<RwLock<Vec<Alert>>>,
    alert_history: Arc<RwLock<Vec<Alert>>>,
    last_alert_times: Arc<RwLock<HashMap<String, DateTime<Utc>>>>,
}

impl NotificationMonitor {
    pub fn new(
        config: MonitoringConfig,
        failure_logger: Arc<FailureLogger>,
        retry_manager: Arc<RetryManager>,
    ) -> Self {
        Self {
            config,
            failure_logger,
            retry_manager,
            statistics: Arc::new(RwLock::new(MonitoringStatistics {
                total_notifications_sent: 0,
                successful_notifications: 0,
                failed_notifications: 0,
                success_rate: 1.0,
                failure_rate: 0.0,
                average_retry_count: 0.0,
                average_response_time: Duration::from_secs(0),
                platform_statistics: HashMap::new(),
                active_alerts: 0,
                resolved_alerts: 0,
                suppressed_alerts: 0,
                last_updated: Utc::now(),
            })),
            alerts: Arc::new(RwLock::new(Vec::new())),
            alert_history: Arc::new(RwLock::new(Vec::new())),
            last_alert_times: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 启动监控
    pub async fn start_monitoring(&self) -> anyhow::Result<()> {
        if !self.config.enabled {
            log::info!("Notification monitoring is disabled");
            return Ok(());
        }

        log::info!("Starting notification monitoring system");

        // 启动统计信息收集
        let statistics_collector = {
            let monitor = self.clone();
            tokio::spawn(async move { monitor.start_statistics_collection().await })
        };

        // 启动告警评估
        let alert_evaluator = {
            let monitor = self.clone();
            tokio::spawn(async move { monitor.start_alert_evaluation().await })
        };

        // 启动自监控
        let self_monitor = if self.config.enable_self_monitoring {
            let monitor = self.clone();
            Some(tokio::spawn(async move { monitor.start_self_monitoring().await }))
        } else {
            None
        };

        // 等待所有任务完成
        tokio::select! {
            result = statistics_collector => {
                log::error!("Statistics collection task ended: {:?}", result);
            }
            result = alert_evaluator => {
                log::error!("Alert evaluation task ended: {:?}", result);
            }
            result = async {
                if let Some(monitor) = self_monitor {
                    monitor.await
                } else {
                    std::future::pending().await
                }
            } => {
                log::error!("Self monitoring task ended: {:?}", result);
            }
        }

        Ok(())
    }

    /// 启动统计信息收集
    async fn start_statistics_collection(&self) -> anyhow::Result<()> {
        let mut interval = interval(self.config.collection_interval);
        let statistics = self.statistics.clone();
        let failure_logger = self.failure_logger.clone();
        let retry_manager = self.retry_manager.clone();

        loop {
            interval.tick().await;

            // 收集失败统计信息
            let failure_stats = failure_logger.get_statistics().await;
            let retry_stats = retry_manager.get_statistics().await;

            // 更新监控统计信息
            let mut stats = statistics.write().await;
            self.update_monitoring_statistics(&mut stats, &failure_stats, &retry_stats).await;
        }
    }

    /// 启动告警评估
    async fn start_alert_evaluation(&self) -> anyhow::Result<()> {
        let mut interval = interval(self.config.alert_evaluation_interval);
        let statistics = self.statistics.clone();
        let alerts = self.alerts.clone();
        let alert_history = self.alert_history.clone();
        let last_alert_times = self.last_alert_times.clone();
        let thresholds = self.config.thresholds.clone();
        let alert_cooldown = self.config.alert_cooldown;

        loop {
            interval.tick().await;

            let stats = statistics.read().await;
            let mut current_alerts = alerts.write().await;
            let mut history = alert_history.write().await;
            let mut last_times = last_alert_times.write().await;

            // 评估每个阈值
            for threshold in &thresholds {
                let metric_value = self.get_metric_value(&stats, &threshold.metric_type, &threshold.platform).await;

                // 检查是否需要触发告警
                let alert_level = if metric_value >= threshold.critical_threshold {
                    Some(AlertLevel::Critical)
                } else if metric_value >= threshold.warning_threshold {
                    Some(AlertLevel::Warning)
                } else {
                    None
                };

                if let Some(level) = alert_level {
                    let alert_key = format!("{:?}_{:?}", threshold.metric_type, threshold.platform);

                    // 检查冷却时间
                    let should_alert = if let Some(last_time) = last_times.get(&alert_key) {
                        Utc::now().signed_duration_since(*last_time) > chrono::Duration::from_std(alert_cooldown).unwrap()
                    } else {
                        true
                    };

                    if should_alert {
                        let alert = self.create_alert(&threshold, level, metric_value).await;

                        log::warn!("Triggered alert: {} - {}", alert.title, alert.description);

                        current_alerts.push(alert.clone());
                        history.push(alert.clone());
                        last_times.insert(alert_key, Utc::now());

                        // 发送告警通知
                        if let Err(e) = self.send_alert_notification(&alert).await {
                            log::error!("Failed to send alert notification: {}", e);
                        }
                    }
                } else {
                    // 检查是否需要解决现有告警
                    for alert in current_alerts.iter_mut() {
                        if alert.status == AlertStatus::Active &&
                           std::mem::discriminant(&alert.metric_type) == std::mem::discriminant(&threshold.metric_type) &&
                           alert.platform == threshold.platform {
                            alert.resolve();
                            log::info!("Resolved alert: {}", alert.title);
                        }
                    }
                }
            }

            // 清理已解决的告警
            current_alerts.retain(|alert| alert.status == AlertStatus::Active || alert.is_suppressed());

            // 更新告警统计
            let active_count = current_alerts.iter().filter(|a| a.status == AlertStatus::Active).count() as u32;
            let suppressed_count = current_alerts.iter().filter(|a| a.status == AlertStatus::Suppressed).count() as u32;
            let resolved_count = history.iter().filter(|a| a.status == AlertStatus::Resolved).count() as u32;

            drop(stats);
            let mut stats = statistics.write().await;
            stats.active_alerts = active_count;
            stats.suppressed_alerts = suppressed_count;
            stats.resolved_alerts = resolved_count;
        }
    }

    /// 启动自监控
    async fn start_self_monitoring(&self) -> anyhow::Result<()> {
        let mut interval = interval(Duration::from_secs(300)); // 5 minutes

        loop {
            interval.tick().await;

            // 监控自身健康状态
            let health_status = self.check_self_health().await;

            if !health_status.is_healthy {
                log::warn!("Notification monitoring system health check failed: {}", health_status.message);

                // 创建自监控告警
                let alert = Alert::new(
                    "self_monitoring".to_string(),
                    AlertLevel::Warning,
                    "Notification Monitor Health Issue".to_string(),
                    health_status.message,
                    MetricType::PlatformAvailability,
                    None,
                    0.0,
                    1.0,
                );

                let mut alerts = self.alerts.write().await;
                alerts.push(alert);
            }
        }
    }

    /// 更新监控统计信息
    async fn update_monitoring_statistics(
        &self,
        stats: &mut MonitoringStatistics,
        failure_stats: &FailureStatistics,
        retry_stats: &RetryStatistics,
    ) {
        // 计算总体统计
        stats.total_notifications_sent = failure_stats.total_failures + retry_stats.successful_retries;
        stats.successful_notifications = retry_stats.successful_retries;
        stats.failed_notifications = failure_stats.final_failures;

        if stats.total_notifications_sent > 0 {
            stats.success_rate = stats.successful_notifications as f64 / stats.total_notifications_sent as f64;
            stats.failure_rate = stats.failed_notifications as f64 / stats.total_notifications_sent as f64;
        }

        stats.average_retry_count = retry_stats.average_retry_count;
        stats.last_updated = Utc::now();

        // 更新平台统计
        for (platform, platform_retry_stats) in &retry_stats.platform_statistics {
            let platform_stats = stats.platform_statistics.entry(platform.clone()).or_insert(PlatformMonitoringStats {
                total_sent: 0,
                successful: 0,
                failed: 0,
                success_rate: 1.0,
                average_response_time: Duration::from_secs(0),
                availability: 1.0,
                last_success: None,
                last_failure: None,
            });

            platform_stats.total_sent = platform_retry_stats.total_attempts;
            platform_stats.successful = platform_retry_stats.successful_retries;
            platform_stats.failed = platform_retry_stats.failed_retries;

            if platform_stats.total_sent > 0 {
                platform_stats.success_rate = platform_stats.successful as f64 / platform_stats.total_sent as f64;
                platform_stats.availability = platform_stats.success_rate;
            }

            platform_stats.average_response_time = platform_retry_stats.average_retry_delay;
        }
    }

    /// 获取指标值
    async fn get_metric_value(
        &self,
        stats: &MonitoringStatistics,
        metric_type: &MetricType,
        platform: &Option<NotificationPlatform>,
    ) -> f64 {
        match metric_type {
            MetricType::SuccessRate => {
                if let Some(platform) = platform {
                    stats.platform_statistics.get(platform)
                        .map(|s| s.success_rate)
                        .unwrap_or(1.0)
                } else {
                    stats.success_rate
                }
            }
            MetricType::FailureRate => {
                if let Some(platform) = platform {
                    stats.platform_statistics.get(platform)
                        .map(|s| 1.0 - s.success_rate)
                        .unwrap_or(0.0)
                } else {
                    stats.failure_rate
                }
            }
            MetricType::AverageRetryCount => stats.average_retry_count,
            MetricType::AverageResponseTime => stats.average_response_time.as_secs_f64(),
            MetricType::PlatformAvailability => {
                if let Some(platform) = platform {
                    stats.platform_statistics.get(platform)
                        .map(|s| s.availability)
                        .unwrap_or(1.0)
                } else {
                    // 计算所有平台的平均可用性
                    if stats.platform_statistics.is_empty() {
                        1.0
                    } else {
                        let total_availability: f64 = stats.platform_statistics.values()
                            .map(|s| s.availability)
                            .sum();
                        total_availability / stats.platform_statistics.len() as f64
                    }
                }
            }
            _ => 0.0, // 其他指标类型的默认值
        }
    }

    /// 创建告警
    async fn create_alert(
        &self,
        threshold: &MonitoringThreshold,
        level: AlertLevel,
        current_value: f64,
    ) -> Alert {
        let threshold_value = match level {
            AlertLevel::Critical => threshold.critical_threshold,
            AlertLevel::Warning => threshold.warning_threshold,
            AlertLevel::Info => threshold.warning_threshold,
        };

        let platform_str = threshold.platform.as_ref()
            .map(|p| format!(" for {:?}", p))
            .unwrap_or_default();

        let title = format!("{:?} {:?} Alert{}", threshold.metric_type, level, platform_str);
        let description = format!(
            "{:?} has reached {} level: current value {:.3}, threshold {:.3}",
            threshold.metric_type,
            match level {
                AlertLevel::Critical => "critical",
                AlertLevel::Warning => "warning",
                AlertLevel::Info => "info",
            },
            current_value,
            threshold_value
        );

        Alert::new(
            format!("{:?}", threshold.metric_type),
            level,
            title,
            description,
            threshold.metric_type.clone(),
            threshold.platform.clone(),
            current_value,
            threshold_value,
        )
    }

    /// 发送告警通知
    async fn send_alert_notification(&self, alert: &Alert) -> anyhow::Result<()> {
        // 这里可以实现发送告警通知的逻辑
        // 例如发送到 webhook、邮件等

        if let Some(ref webhook_url) = self.config.webhook_url {
            let client = reqwest::Client::new();
            let payload = serde_json::json!({
                "alert_id": alert.id,
                "title": alert.title,
                "description": alert.description,
                "level": alert.level,
                "metric_type": alert.metric_type,
                "platform": alert.platform,
                "current_value": alert.current_value,
                "threshold_value": alert.threshold_value,
                "triggered_at": alert.triggered_at,
            });

            let response = client
                .post(webhook_url)
                .json(&payload)
                .send()
                .await?;

            if !response.status().is_success() {
                anyhow::bail!("Webhook request failed with status: {}", response.status());
            }
        }

        // 记录告警到日志
        log::warn!(
            target: "notification_alert",
            "Alert triggered: {} - {} - level: {:?} - metric_type: {:?} - platform: {:?} - current_value: {} - threshold_value: {}",
            alert.title,
            alert.description,
            alert.level,
            alert.metric_type,
            alert.platform,
            alert.current_value,
            alert.threshold_value
        );

        Ok(())
    }

    /// 检查自身健康状态
    async fn check_self_health(&self) -> HealthStatus {
        // 检查各个组件的健康状态
        let mut issues = Vec::new();

        // 检查统计信息更新时间
        let stats = self.statistics.read().await;
        let time_since_update = Utc::now().signed_duration_since(stats.last_updated);
        if time_since_update > chrono::Duration::minutes(10) {
            issues.push("Statistics not updated for more than 10 minutes".to_string());
        }

        // 检查告警数量
        let alerts = self.alerts.read().await;
        if alerts.len() > 100 {
            issues.push("Too many active alerts".to_string());
        }

        // 检查内存使用
        // 这里可以添加内存使用检查逻辑

        if issues.is_empty() {
            HealthStatus {
                is_healthy: true,
                message: "All systems operational".to_string(),
            }
        } else {
            HealthStatus {
                is_healthy: false,
                message: issues.join("; "),
            }
        }
    }

    /// 获取监控统计信息
    pub async fn get_statistics(&self) -> MonitoringStatistics {
        self.statistics.read().await.clone()
    }

    /// 获取活跃告警
    pub async fn get_active_alerts(&self) -> Vec<Alert> {
        let alerts = self.alerts.read().await;
        alerts.iter()
            .filter(|alert| alert.status == AlertStatus::Active)
            .cloned()
            .collect()
    }

    /// 获取告警历史
    pub async fn get_alert_history(&self) -> Vec<Alert> {
        self.alert_history.read().await.clone()
    }

    /// 抑制告警
    pub async fn suppress_alert(&self, alert_id: &str, duration: Duration) -> anyhow::Result<()> {
        let mut alerts = self.alerts.write().await;

        if let Some(alert) = alerts.iter_mut().find(|a| a.id == alert_id) {
            alert.suppress(duration);
            log::info!("Suppressed alert {} for {:?}", alert_id, duration);
            Ok(())
        } else {
            anyhow::bail!("Alert not found: {}", alert_id)
        }
    }

    /// 解决告警
    pub async fn resolve_alert(&self, alert_id: &str) -> anyhow::Result<()> {
        let mut alerts = self.alerts.write().await;

        if let Some(alert) = alerts.iter_mut().find(|a| a.id == alert_id) {
            alert.resolve();
            log::info!("Manually resolved alert {}", alert_id);
            Ok(())
        } else {
            anyhow::bail!("Alert not found: {}", alert_id)
        }
    }

    /// 获取配置
    pub fn get_config(&self) -> &MonitoringConfig {
        &self.config
    }

    /// 更新配置
    pub fn update_config(&mut self, config: MonitoringConfig) {
        self.config = config;
    }
}

/// 健康状态
#[derive(Debug, Clone)]
struct HealthStatus {
    is_healthy: bool,
    message: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::notification::failure_logger::FailureLoggerConfig;
    use crate::notification::retry::RetryConfig;

    #[tokio::test]
    async fn test_alert_creation() {
        let threshold = MonitoringThreshold {
            metric_type: MetricType::SuccessRate,
            platform: Some(NotificationPlatform::Email),
            warning_threshold: 0.95,
            critical_threshold: 0.90,
            evaluation_window: Duration::from_secs(300), // 5 minutes
            min_sample_size: 10,
        };

        let failure_logger = Arc::new(FailureLogger::new(FailureLoggerConfig::default()));
        let retry_manager = Arc::new(RetryManager::new(RetryConfig::default()));
        let monitor = NotificationMonitor::new(MonitoringConfig::default(), failure_logger, retry_manager);

        let alert = monitor.create_alert(&threshold, AlertLevel::Critical, 0.85).await;

        assert_eq!(alert.level, AlertLevel::Critical);
        assert_eq!(alert.status, AlertStatus::Active);
        assert_eq!(alert.current_value, 0.85);
        assert_eq!(alert.threshold_value, 0.90);
        assert_eq!(alert.platform, Some(NotificationPlatform::Email));
    }

    #[tokio::test]
    async fn test_alert_suppression() {
        let failure_logger = Arc::new(FailureLogger::new(FailureLoggerConfig::default()));
        let retry_manager = Arc::new(RetryManager::new(RetryConfig::default()));
        let monitor = NotificationMonitor::new(MonitoringConfig::default(), failure_logger, retry_manager);

        let mut alert = Alert::new(
            "test".to_string(),
            AlertLevel::Warning,
            "Test Alert".to_string(),
            "Test Description".to_string(),
            MetricType::SuccessRate,
            None,
            0.8,
            0.9,
        );

        assert_eq!(alert.status, AlertStatus::Active);
        assert!(!alert.is_suppressed());

        alert.suppress(Duration::from_secs(1800)); // 30 minutes

        assert_eq!(alert.status, AlertStatus::Suppressed);
        assert!(alert.is_suppressed());
    }

    #[tokio::test]
    async fn test_metric_value_calculation() {
        let failure_logger = Arc::new(FailureLogger::new(FailureLoggerConfig::default()));
        let retry_manager = Arc::new(RetryManager::new(RetryConfig::default()));
        let monitor = NotificationMonitor::new(MonitoringConfig::default(), failure_logger, retry_manager);

        let mut stats = MonitoringStatistics {
            total_notifications_sent: 100,
            successful_notifications: 95,
            failed_notifications: 5,
            success_rate: 0.95,
            failure_rate: 0.05,
            average_retry_count: 1.2,
            average_response_time: Duration::from_secs(2),
            platform_statistics: HashMap::new(),
            active_alerts: 0,
            resolved_alerts: 0,
            suppressed_alerts: 0,
            last_updated: Utc::now(),
        };

        stats.platform_statistics.insert(NotificationPlatform::Email, PlatformMonitoringStats {
            total_sent: 50,
            successful: 48,
            failed: 2,
            success_rate: 0.96,
            average_response_time: Duration::from_secs(1),
            availability: 0.96,
            last_success: Some(Utc::now()),
            last_failure: None,
        });

        let overall_success_rate = monitor.get_metric_value(&stats, &MetricType::SuccessRate, &None).await;
        assert_eq!(overall_success_rate, 0.95);

        let email_success_rate = monitor.get_metric_value(&stats, &MetricType::SuccessRate, &Some(NotificationPlatform::Email)).await;
        assert_eq!(email_success_rate, 0.96);

        let failure_rate = monitor.get_metric_value(&stats, &MetricType::FailureRate, &None).await;
        assert_eq!(failure_rate, 0.05);

        let retry_count = monitor.get_metric_value(&stats, &MetricType::AverageRetryCount, &None).await;
        assert_eq!(retry_count, 1.2);
    }
}