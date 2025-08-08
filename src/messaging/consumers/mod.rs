pub mod kafka;

pub use kafka::KafkaConsumer;

use async_trait::async_trait;
use anyhow::Result;
use std::future::Future;
use std::pin::Pin;
use crate::messaging::{QueueType, MessageId};
use crate::messaging::events::{ReportEvent, EventProcessingResult};

/// Message received from a queue
#[derive(Debug, Clone)]
pub struct Message {
    /// Message ID
    pub id: MessageId,

    /// Topic the message was received from
    pub topic: String,

    /// Message payload
    pub payload: Vec<u8>,

    /// Message headers
    pub headers: std::collections::HashMap<String, String>,

    /// Message timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,

    /// Partition (if applicable)
    pub partition: Option<i32>,

    /// Offset (if applicable)
    pub offset: Option<i64>,
}

/// Message processing handler function type
pub type MessageHandler = Box<dyn Fn(Message) -> Pin<Box<dyn Future<Output = Result<()>> + Send>> + Send + Sync>;

/// Event processing handler function type
pub type EventHandler = Box<dyn Fn(ReportEvent) -> Pin<Box<dyn Future<Output = Result<EventProcessingResult>> + Send>> + Send + Sync>;

/// Base message consumer trait
#[async_trait]
pub trait MessageConsumer: Send + Sync {
    /// Get the queue type this consumer supports
    fn queue_type(&self) -> QueueType;

    /// Start consuming messages from a topic
    async fn start_consuming(&mut self, topic: &str) -> Result<()>;

    /// Stop consuming messages
    async fn stop_consuming(&mut self) -> Result<()>;

    /// Set message handler
    fn set_message_handler(&mut self, handler: MessageHandler);

    /// Set event handler for report events
    fn set_event_handler(&mut self, handler: EventHandler);

    /// Check if the consumer is available and configured
    fn is_available(&self) -> bool;

    /// Get consumer health status
    async fn health_check(&self) -> Result<ConsumerHealth>;

    /// Commit message offset (if applicable)
    async fn commit_message(&mut self, message: &Message) -> Result<()>;

    /// Close the consumer and clean up resources
    async fn close(&mut self) -> Result<()>;
}

/// Consumer health information
#[derive(Debug, Clone)]
pub struct ConsumerHealth {
    /// Whether the consumer is healthy
    pub is_healthy: bool,

    /// Health check timestamp
    pub checked_at: chrono::DateTime<chrono::Utc>,

    /// Error message if unhealthy
    pub error_message: Option<String>,

    /// Additional metrics
    pub metrics: ConsumerMetrics,
}

/// Consumer metrics
#[derive(Debug, Clone, Default)]
pub struct ConsumerMetrics {
    /// Total messages consumed
    pub messages_consumed: u64,

    /// Total messages processed successfully
    pub messages_processed: u64,

    /// Total messages failed
    pub messages_failed: u64,

    /// Average processing latency in milliseconds
    pub avg_processing_latency_ms: f64,

    /// Current lag (if applicable)
    pub lag: Option<i64>,

    /// Topics being consumed
    pub active_topics: Vec<String>,
}

impl Default for ConsumerHealth {
    fn default() -> Self {
        Self {
            is_healthy: false,
            checked_at: chrono::Utc::now(),
            error_message: None,
            metrics: ConsumerMetrics::default(),
        }
    }
}

impl Message {
    /// Try to deserialize the message payload as a ReportEvent
    pub fn try_as_report_event(&self) -> Result<ReportEvent> {
        serde_json::from_slice(&self.payload)
            .map_err(|e| anyhow::anyhow!("Failed to deserialize ReportEvent: {}", e))
    }

    /// Get a header value
    pub fn get_header(&self, key: &str) -> Option<&String> {
        self.headers.get(key)
    }

    /// Check if message has a specific header
    pub fn has_header(&self, key: &str) -> bool {
        self.headers.contains_key(key)
    }
}