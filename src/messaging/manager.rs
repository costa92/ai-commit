use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use anyhow::Result;

use crate::messaging::{QueueType, MessageId};
use crate::messaging::events::{ReportEvent, ReportEventType};
use crate::messaging::event_bus::{EventBus, EventRoutingConfig};

/// Message producer trait for sending messages to queues
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
}

/// Message consumer trait for receiving messages from queues
#[async_trait]
pub trait MessageConsumer: Send + Sync {
    /// Get the queue type this consumer supports
    fn queue_type(&self) -> QueueType;

    /// Start consuming messages from a topic
    async fn start_consuming(&mut self, topic: &str) -> Result<()>;

    /// Stop consuming messages
    async fn stop_consuming(&mut self) -> Result<()>;

    /// Check if the consumer is available and configured
    fn is_available(&self) -> bool;
}

/// Configuration for messaging system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessagingConfig {
    /// Enabled queue types
    pub enabled_queues: Vec<QueueType>,

    /// Kafka configuration
    pub kafka: Option<KafkaConfig>,

    /// Default topic mappings for different event types
    pub topic_mappings: HashMap<ReportEventType, String>,

    /// Message serialization format
    pub serialization_format: SerializationFormat,

    /// Retry configuration
    pub retry_config: RetryConfig,
}

/// Kafka-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KafkaConfig {
    /// Kafka broker addresses
    pub brokers: Vec<String>,

    /// Consumer group ID
    pub consumer_group_id: String,

    /// Producer configuration
    pub producer_config: HashMap<String, String>,

    /// Consumer configuration
    pub consumer_config: HashMap<String, String>,

    /// Topic configurations
    pub topics: HashMap<String, TopicConfig>,
}

/// Topic configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicConfig {
    /// Number of partitions
    pub partitions: i32,

    /// Replication factor
    pub replication_factor: i16,

    /// Topic-specific configuration
    pub config: HashMap<String, String>,
}

/// Message serialization format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SerializationFormat {
    Json,
    MessagePack,
    Protobuf,
}

/// Retry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Maximum number of retries
    pub max_retries: u32,

    /// Initial retry delay in milliseconds
    pub initial_delay_ms: u64,

    /// Maximum retry delay in milliseconds
    pub max_delay_ms: u64,

    /// Backoff multiplier
    pub backoff_multiplier: f64,
}

/// Main messaging manager that coordinates producers and consumers
pub struct MessagingManager {
    /// Registered message producers
    producers: Arc<RwLock<HashMap<QueueType, Box<dyn MessageProducer>>>>,

    /// Registered message consumers
    consumers: Arc<RwLock<HashMap<QueueType, Box<dyn MessageConsumer>>>>,

    /// Event bus for local event handling
    event_bus: Option<Arc<EventBus>>,

    /// Messaging configuration
    config: MessagingConfig,

    /// Current serialization format
    serialization_format: SerializationFormat,
}

impl MessagingManager {
    /// Create a new messaging manager with configuration
    pub fn new(config: MessagingConfig) -> Self {
        Self {
            producers: Arc::new(RwLock::new(HashMap::new())),
            consumers: Arc::new(RwLock::new(HashMap::new())),
            event_bus: None,
            serialization_format: config.serialization_format.clone(),
            config,
        }
    }

    /// Create a new messaging manager with event bus enabled
    pub fn with_event_bus(config: MessagingConfig, event_routing_config: EventRoutingConfig) -> Self {
        let event_bus = Arc::new(EventBus::new(event_routing_config));

        Self {
            producers: Arc::new(RwLock::new(HashMap::new())),
            consumers: Arc::new(RwLock::new(HashMap::new())),
            event_bus: Some(event_bus),
            serialization_format: config.serialization_format.clone(),
            config,
        }
    }

    /// Register a message producer
    pub async fn register_producer(&self, producer: Box<dyn MessageProducer>) -> Result<()> {
        let queue_type = producer.queue_type();
        let mut producers = self.producers.write().await;
        producers.insert(queue_type, producer);
        Ok(())
    }

    /// Register a message consumer
    pub async fn register_consumer(&self, consumer: Box<dyn MessageConsumer>) -> Result<()> {
        let queue_type = consumer.queue_type();
        let mut consumers = self.consumers.write().await;
        consumers.insert(queue_type, consumer);
        Ok(())
    }

    /// Send a report event using the appropriate producer
    pub async fn send_report_event(&self, event: ReportEvent) -> Result<MessageId> {
        // Determine which queue to use (prefer Kafka if available)
        let queue_type = self.select_queue_for_event(&event.event_type)?;

        let mut producers = self.producers.write().await;
        if let Some(producer) = producers.get_mut(&queue_type) {
            if producer.is_available() {
                return producer.send_report_event(event).await;
            }
        }

        anyhow::bail!("No available producer for queue type: {:?}", queue_type)
    }

    /// Send a raw message to a specific topic
    pub async fn send_message(&self, queue_type: QueueType, topic: &str, message: &[u8]) -> Result<MessageId> {
        let mut producers = self.producers.write().await;
        if let Some(producer) = producers.get_mut(&queue_type) {
            if producer.is_available() {
                return producer.send_message(topic, message).await;
            }
        }

        anyhow::bail!("No available producer for queue type: {:?}", queue_type)
    }

    /// Start consuming messages from all registered consumers
    pub async fn start_all_consumers(&self) -> Result<()> {
        let mut consumers = self.consumers.write().await;

        for (queue_type, consumer) in consumers.iter_mut() {
            if consumer.is_available() {
                // Start consuming from default topics for this queue type
                if let Some(topics) = self.get_topics_for_queue(queue_type) {
                    for topic in topics {
                        consumer.start_consuming(&topic).await?;
                    }
                }
            }
        }

        Ok(())
    }

    /// Stop all consumers
    pub async fn stop_all_consumers(&self) -> Result<()> {
        let mut consumers = self.consumers.write().await;

        for consumer in consumers.values_mut() {
            consumer.stop_consuming().await?;
        }

        Ok(())
    }

    /// Serialize data based on configured format
    pub fn serialize<T: Serialize>(&self, data: &T) -> Result<Vec<u8>> {
        match self.serialization_format {
            SerializationFormat::Json => {
                Ok(serde_json::to_vec(data)?)
            },
            SerializationFormat::MessagePack => {
                Ok(rmp_serde::to_vec(data)?)
            },
            SerializationFormat::Protobuf => {
                // For now, fall back to JSON for protobuf
                // In a real implementation, you'd use protobuf serialization
                Ok(serde_json::to_vec(data)?)
            },
        }
    }

    /// Deserialize data based on configured format
    pub fn deserialize<T: for<'de> Deserialize<'de>>(&self, data: &[u8]) -> Result<T> {
        match self.serialization_format {
            SerializationFormat::Json => {
                Ok(serde_json::from_slice(data)?)
            },
            SerializationFormat::MessagePack => {
                Ok(rmp_serde::from_slice(data)?)
            },
            SerializationFormat::Protobuf => {
                // For now, fall back to JSON for protobuf
                Ok(serde_json::from_slice(data)?)
            },
        }
    }

    /// Get available queue types
    pub async fn get_available_queues(&self) -> Vec<QueueType> {
        let producers = self.producers.read().await;
        producers.keys().cloned().collect()
    }

    /// Check if messaging is enabled and configured
    pub fn is_enabled(&self) -> bool {
        !self.config.enabled_queues.is_empty()
    }

    /// Select the best queue for a given event type
    fn select_queue_for_event(&self, event_type: &ReportEventType) -> Result<QueueType> {
        // Prefer Kafka if available
        if self.config.enabled_queues.contains(&QueueType::Kafka) {
            return Ok(QueueType::Kafka);
        }

        // Fall back to other available queues
        if let Some(queue_type) = self.config.enabled_queues.first() {
            return Ok(queue_type.clone());
        }

        anyhow::bail!("No enabled queues configured")
    }

    /// Get topics for a specific queue type
    fn get_topics_for_queue(&self, queue_type: &QueueType) -> Option<Vec<String>> {
        match queue_type {
            QueueType::Kafka => {
                if let Some(kafka_config) = &self.config.kafka {
                    Some(kafka_config.topics.keys().cloned().collect())
                } else {
                    None
                }
            },
            _ => None,
        }
    }

    /// Get reference to the event bus if available
    pub fn event_bus(&self) -> Option<&Arc<EventBus>> {
        self.event_bus.as_ref()
    }

    /// Publish an event to the local event bus
    pub async fn publish_event(&self, event: ReportEvent) -> Result<usize> {
        if let Some(event_bus) = &self.event_bus {
            event_bus.publish(event).await
        } else {
            anyhow::bail!("Event bus not enabled")
        }
    }

    /// Register an event handler with the event bus
    pub async fn register_event_handler(&self, handler: Arc<dyn crate::messaging::event_bus::EventHandler>) -> Result<()> {
        if let Some(event_bus) = &self.event_bus {
            event_bus.register_handler(handler).await
        } else {
            anyhow::bail!("Event bus not enabled")
        }
    }

    /// Send event to both external queues and local event bus
    pub async fn broadcast_event(&self, event: ReportEvent) -> Result<(Option<MessageId>, Option<usize>)> {
        let mut external_result = None;
        let mut local_result = None;

        // Send to external queue if available
        if self.is_enabled() {
            match self.send_report_event(event.clone()).await {
                Ok(message_id) => external_result = Some(message_id),
                Err(e) => log::warn!("Failed to send event to external queue: {}", e),
            }
        }

        // Send to local event bus if available
        if let Some(event_bus) = &self.event_bus {
            match event_bus.publish(event).await {
                Ok(subscriber_count) => local_result = Some(subscriber_count),
                Err(e) => log::warn!("Failed to publish event to local event bus: {}", e),
            }
        }

        Ok((external_result, local_result))
    }

    /// Get event bus statistics
    pub async fn get_event_bus_statistics(&self) -> Option<crate::messaging::event_bus::EventBusStatistics> {
        if let Some(event_bus) = &self.event_bus {
            Some(event_bus.get_statistics().await)
        } else {
            None
        }
    }

    /// Shutdown the messaging manager and all its components
    pub async fn shutdown(&self) -> Result<()> {
        // Stop all consumers
        self.stop_all_consumers().await?;

        // Shutdown event bus if available
        if let Some(event_bus) = &self.event_bus {
            event_bus.shutdown().await?;
        }

        log::info!("Messaging manager shutdown completed");
        Ok(())
    }
}

impl Default for MessagingConfig {
    fn default() -> Self {
        let mut topic_mappings = HashMap::new();
        topic_mappings.insert(ReportEventType::ReportGenerated, "report-generated".to_string());
        topic_mappings.insert(ReportEventType::AnalysisCompleted, "analysis-completed".to_string());
        topic_mappings.insert(ReportEventType::QualityThresholdExceeded, "quality-alert".to_string());
        topic_mappings.insert(ReportEventType::SecurityIssueDetected, "security-alert".to_string());
        topic_mappings.insert(ReportEventType::PerformanceRegressionDetected, "performance-alert".to_string());

        Self {
            enabled_queues: vec![],
            kafka: None,
            topic_mappings,
            serialization_format: SerializationFormat::Json,
            retry_config: RetryConfig {
                max_retries: 3,
                initial_delay_ms: 1000,
                max_delay_ms: 30000,
                backoff_multiplier: 2.0,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::messaging::events::{ReportEvent, ReportEventType, EventMetadata};
    use chrono::Utc;

    #[tokio::test]
    async fn test_messaging_manager_creation() {
        let config = MessagingConfig::default();
        let manager = MessagingManager::new(config);

        assert!(!manager.is_enabled());
        assert_eq!(manager.get_available_queues().await.len(), 0);
    }

    #[tokio::test]
    async fn test_serialization() {
        let config = MessagingConfig::default();
        let manager = MessagingManager::new(config);

        let event = ReportEvent {
            event_id: "test-123".to_string(),
            event_type: ReportEventType::ReportGenerated,
            timestamp: Utc::now(),
            project_path: "/test/project".to_string(),
            report_id: Some("report-456".to_string()),
            metadata: EventMetadata {
                source: "test".to_string(),
                version: "1.0.0".to_string(),
                correlation_id: None,
                user_id: None,
                tags: HashMap::new(),
            },
            payload: serde_json::json!({"test": "data"}),
        };

        let serialized = manager.serialize(&event).unwrap();
        let deserialized: ReportEvent = manager.deserialize(&serialized).unwrap();

        assert_eq!(event.event_id, deserialized.event_id);
        assert_eq!(event.event_type, deserialized.event_type);
    }
}