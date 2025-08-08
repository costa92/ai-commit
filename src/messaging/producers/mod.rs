pub mod kafka;

pub use kafka::KafkaProducer;

use async_trait::async_trait;
use anyhow::Result;
use crate::messaging::{QueueType, MessageId};
use crate::messaging::events::ReportEvent;

/// Base message producer trait
#[async_trait]
pub trait MessageProducer: Send + Sync {
    /// Get the queue type this producer supports
    fn queue_type(&self) -> QueueType;

    /// Send a raw message to a topic
    async fn send_message(&mut self, topic: &str, message: &[u8]) -> Result<MessageId>;

    /// Send a report event (convenience method)
    async fn send_report_event(&mut self, event: ReportEvent) -> Result<MessageId>;

    /// Check if the producer is available and configured
    fn is_available(&self) -> bool;

    /// Get producer health status
    async fn health_check(&self) -> Result<ProducerHealth>;

    /// Close the producer and clean up resources
    async fn close(&mut self) -> Result<()>;
}

/// Producer health information
#[derive(Debug, Clone)]
pub struct ProducerHealth {
    /// Whether the producer is healthy
    pub is_healthy: bool,

    /// Health check timestamp
    pub checked_at: chrono::DateTime<chrono::Utc>,

    /// Error message if unhealthy
    pub error_message: Option<String>,

    /// Additional metrics
    pub metrics: ProducerMetrics,
}

/// Producer metrics
#[derive(Debug, Clone, Default)]
pub struct ProducerMetrics {
    /// Total messages sent
    pub messages_sent: u64,

    /// Total messages failed
    pub messages_failed: u64,

    /// Average send latency in milliseconds
    pub avg_send_latency_ms: f64,

    /// Current queue size (if applicable)
    pub queue_size: Option<u64>,
}

impl Default for ProducerHealth {
    fn default() -> Self {
        Self {
            is_healthy: false,
            checked_at: chrono::Utc::now(),
            error_message: None,
            metrics: ProducerMetrics::default(),
        }
    }
}