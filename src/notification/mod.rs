pub mod service;
pub mod providers;
pub mod templates;

pub use service::{NotificationService, NotificationMessage, NotificationResult};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum NotificationPlatform {
    Feishu,
    WeChat,
    DingTalk,
    Email,
}

#[derive(Debug, Clone)]
pub enum NotificationSeverity {
    Info,
    Warning,
    Error,
    Critical,
}