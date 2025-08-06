pub mod manager;
pub mod producers;
pub mod consumers;
pub mod events;

pub use manager::{MessagingManager, MessagingConfig};
pub use events::{ReportEvent, ReportEventType, EventMetadata};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum QueueType {
    Kafka,
    RabbitMQ,
    Redis,
}

#[derive(Debug, Clone)]
pub struct MessageId {
    pub id: String,
    pub queue_type: QueueType,
}