pub mod manager;
pub mod producers;
pub mod consumers;
pub mod events;
pub mod event_bus;
pub mod async_processor;

pub use manager::{MessagingManager, MessagingConfig, KafkaConfig, RetryConfig, SerializationFormat};
pub use events::{ReportEvent, ReportEventType, EventMetadata, EventStatus, EventProcessingResult};
pub use producers::{MessageProducer, ProducerHealth, ProducerMetrics};
pub use consumers::{MessageConsumer, Message, ConsumerHealth, ConsumerMetrics, MessageHandler, EventHandler};
pub use event_bus::{EventBus, EventHandler as EventBusHandler, EventRoutingConfig, EventBusStatistics, LoggingEventHandler};
pub use async_processor::{
    AsyncReportManager, AsyncReportProcessor, AsyncReportRequest, AsyncReportResult,
    AsyncRequestType, RequestPriority, ProcessingStatus, AsyncProcessingConfig,
    ReportGenerationProcessor
};

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum QueueType {
    Kafka,
    RabbitMQ,
    Redis,
}

#[derive(Debug, Clone)]
pub struct MessageId {
    pub id: String,
    pub queue_type: QueueType,
    pub topic: Option<String>,
    pub partition: Option<i32>,
    pub offset: Option<i64>,
}

impl MessageId {
    /// Create a new Kafka message ID
    pub fn kafka(topic: String, partition: i32, offset: i64) -> Self {
        Self {
            id: format!("{}:{}:{}", topic, partition, offset),
            queue_type: QueueType::Kafka,
            topic: Some(topic),
            partition: Some(partition),
            offset: Some(offset),
        }
    }

    /// Create a generic message ID
    pub fn generic(queue_type: QueueType, id: String) -> Self {
        Self {
            id,
            queue_type,
            topic: None,
            partition: None,
            offset: None,
        }
    }
}