pub mod service;
pub mod providers;
pub mod templates;
pub mod rule_engine;
pub mod retry;
pub mod failure_logger;
pub mod monitoring;

pub use service::{
    NotificationService, NotificationMessage, NotificationResult, NotificationProvider,
    NotificationConfig, RetryConfig, RateLimitConfig, TemplateConfig, NotificationRule,
    NotificationCondition, ConditionOperator, AggregationConfig
};
pub use templates::TemplateEngine;
pub use rule_engine::{
    NotificationRuleEngine, NotificationRule as RuleEngineRule, ProcessedNotification,
    RuleEngineStatistics, RuleRateLimit
};
pub use retry::{
    RetryManager, RetryStrategy, RetryCondition, RetryRecord, RetryStatistics,
    PlatformRetryStats
};
pub use failure_logger::{
    FailureLogger, FailureLoggerConfig, FailureRecord, FailureStatistics, FailureType,
    FailureContext, ExportFormat
};
pub use monitoring::{
    NotificationMonitor, MonitoringConfig, MonitoringThreshold, Alert, AlertLevel,
    AlertStatus, MetricType, MonitoringStatistics, PlatformMonitoringStats
};

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
pub enum NotificationPlatform {
    Feishu,
    WeChat,
    DingTalk,
    Email,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum NotificationSeverity {
    Info,
    Warning,
    Error,
    Critical,
}