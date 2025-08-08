use async_trait::async_trait;
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::Utc;

use crate::messaging::{QueueType, MessageId};
use crate::messaging::events::{ReportEvent, EventProcessingResult, EventStatus};
use crate::messaging::consumers::{
    MessageConsumer, Message, ConsumerHealth, ConsumerMetrics,
    MessageHandler, EventHandler
};

#[cfg(feature = "messaging-kafka")]
use rdkafka::{
    consumer::{StreamConsumer, Consumer},
    ClientConfig, TopicPartitionList,
    message::Message as KafkaMessage,
};

/// Kafka message consumer implementation
pub struct KafkaConsumer {
    #[cfg(feature = "messaging-kafka")]
    consumer: Option<StreamConsumer>,

    #[cfg(not(feature = "messaging-kafka"))]
    consumer: Option<()>,

    config: KafkaConsumerConfig,
    metrics: Arc<RwLock<ConsumerMetrics>>,
    message_handler: Option<MessageHandler>,
    event_handler: Option<EventHandler>,
    is_connected: bool,
    is_consuming: bool,
}

/// Kafka consumer configuration
#[derive(Debug, Clone)]
pub struct KafkaConsumerConfig {
    /// Kafka broker addresses
    pub brokers: Vec<String>,

    /// Consumer group ID
    pub group_id: String,

    /// Consumer configuration options
    pub consumer_config: HashMap<String, String>,

    /// Topics to subscribe to
    pub topics: Vec<String>,

    /// Auto commit interval in milliseconds
    pub auto_commit_interval_ms: u64,

    /// Session timeout in milliseconds
    pub session_timeout_ms: u64,
}

impl KafkaConsumer {
    /// Create a new Kafka consumer
    pub fn new(config: KafkaConsumerConfig) -> Self {
        Self {
            consumer: None,
            config,
            metrics: Arc::new(RwLock::new(ConsumerMetrics::default())),
            message_handler: None,
            event_handler: None,
            is_connected: false,
            is_consuming: false,
        }
    }

    /// Initialize the Kafka consumer connection
    pub async fn initialize(&mut self) -> Result<()> {
        #[cfg(feature = "messaging-kafka")]
        {
            let mut client_config = ClientConfig::new();

            // Set broker list and group ID
            client_config.set("bootstrap.servers", self.config.brokers.join(","));
            client_config.set("group.id", &self.config.group_id);

            // Apply additional configuration
            for (key, value) in &self.config.consumer_config {
                client_config.set(key, value);
            }

            // Set default configurations if not provided
            if !self.config.consumer_config.contains_key("enable.auto.commit") {
                client_config.set("enable.auto.commit", "true");
            }

            if !self.config.consumer_config.contains_key("auto.commit.interval.ms") {
                client_config.set("auto.commit.interval.ms", self.config.auto_commit_interval_ms.to_string());
            }

            if !self.config.consumer_config.contains_key("session.timeout.ms") {
                client_config.set("session.timeout.ms", self.config.session_timeout_ms.to_string());
            }

            let consumer: StreamConsumer = client_config.create()
                .map_err(|e| anyhow::anyhow!("Failed to create Kafka consumer: {}", e))?;

            self.consumer = Some(consumer);
            self.is_connected = true;

            Ok(())
        }

        #[cfg(not(feature = "messaging-kafka"))]
        {
            anyhow::bail!("Kafka support not enabled. Enable the 'messaging-kafka' feature to use Kafka consumer.")
        }
    }

    /// Start the message consumption loop
    async fn start_consumption_loop(&mut self) -> Result<()> {
        #[cfg(feature = "messaging-kafka")]
        {
            if let Some(ref consumer) = self.consumer {
                use rdkafka::consumer::StreamConsumer;
                use futures_util::StreamExt;

                let mut message_stream = consumer.stream();

                while self.is_consuming {
                    match message_stream.next().await {
                        Some(Ok(kafka_message)) => {
                            if let Err(e) = self.process_kafka_message(kafka_message).await {
                                log::error!("Error processing Kafka message: {}", e);

                                let mut metrics = self.metrics.write().await;
                                metrics.messages_failed += 1;
                            }
                        }
                        Some(Err(e)) => {
                            log::error!("Error receiving Kafka message: {}", e);

                            let mut metrics = self.metrics.write().await;
                            metrics.messages_failed += 1;

                            // Brief pause before continuing
                            tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
                        }
                        None => {
                            log::warn!("Kafka message stream ended");
                            break;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Process a Kafka message
    #[cfg(feature = "messaging-kafka")]
    async fn process_kafka_message(&self, kafka_message: rdkafka::message::BorrowedMessage<'_>) -> Result<()> {
        let start_time = std::time::Instant::now();

        // Convert Kafka message to our Message format
        let message = self.convert_kafka_message(&kafka_message)?;

        // Update metrics
        {
            let mut metrics = self.metrics.write().await;
            metrics.messages_consumed += 1;
        }

        // Try to process as ReportEvent first
        if let Ok(report_event) = message.try_as_report_event() {
            if let Some(ref event_handler) = self.event_handler {
                match event_handler(report_event).await {
                    Ok(result) => {
                        log::debug!("Successfully processed report event: {:?}", result);

                        let mut metrics = self.metrics.write().await;
                        metrics.messages_processed += 1;

                        let duration_ms = start_time.elapsed().as_millis() as f64;
                        self.update_processing_latency(&mut metrics, duration_ms);
                    }
                    Err(e) => {
                        log::error!("Error processing report event: {}", e);

                        let mut metrics = self.metrics.write().await;
                        metrics.messages_failed += 1;

                        return Err(e);
                    }
                }
            }
        } else if let Some(ref message_handler) = self.message_handler {
            // Fall back to generic message handler
            match message_handler(message).await {
                Ok(_) => {
                    let mut metrics = self.metrics.write().await;
                    metrics.messages_processed += 1;

                    let duration_ms = start_time.elapsed().as_millis() as f64;
                    self.update_processing_latency(&mut metrics, duration_ms);
                }
                Err(e) => {
                    log::error!("Error processing message: {}", e);

                    let mut metrics = self.metrics.write().await;
                    metrics.messages_failed += 1;

                    return Err(e);
                }
            }
        }

        Ok(())
    }

    /// Convert Kafka message to our Message format
    #[cfg(feature = "messaging-kafka")]
    fn convert_kafka_message(&self, kafka_message: &rdkafka::message::BorrowedMessage<'_>) -> Result<Message> {
        let topic = kafka_message.topic().to_string();
        let partition = kafka_message.partition();
        let offset = kafka_message.offset();

        let message_id = MessageId::kafka(topic.clone(), partition, offset);

        let payload = kafka_message.payload()
            .unwrap_or(&[])
            .to_vec();

        let mut headers = HashMap::new();
        if let Some(kafka_headers) = kafka_message.headers() {
            for header in kafka_headers.iter() {
                if let Some(value) = header.value {
                    headers.insert(
                        header.key.to_string(),
                        String::from_utf8_lossy(value).to_string()
                    );
                }
            }
        }

        let timestamp = kafka_message.timestamp()
            .to_millis()
            .map(|millis| chrono::DateTime::from_timestamp_millis(millis))
            .flatten()
            .unwrap_or_else(|| Utc::now());

        Ok(Message {
            id: message_id,
            topic,
            payload,
            headers,
            timestamp,
            partition: Some(partition),
            offset: Some(offset),
        })
    }

    /// Update processing latency metrics
    fn update_processing_latency(&self, metrics: &mut ConsumerMetrics, duration_ms: f64) {
        let current_avg = metrics.avg_processing_latency_ms;
        let total_processed = metrics.messages_processed;

        if total_processed == 0 {
            return;
        }

        // Simple moving average
        let new_avg = (current_avg * (total_processed as f64) + duration_ms) / ((total_processed + 1) as f64);
        metrics.avg_processing_latency_ms = new_avg;
    }
}

#[async_trait]
impl MessageConsumer for KafkaConsumer {
    fn queue_type(&self) -> QueueType {
        QueueType::Kafka
    }

    async fn start_consuming(&mut self, topic: &str) -> Result<()> {
        if !self.is_connected {
            self.initialize().await?;
        }

        #[cfg(feature = "messaging-kafka")]
        {
            if let Some(ref consumer) = self.consumer {
                let topics = vec![topic];
                consumer.subscribe(&topics)
                    .map_err(|e| anyhow::anyhow!("Failed to subscribe to topic {}: {}", topic, e))?;

                // Update metrics
                {
                    let mut metrics = self.metrics.write().await;
                    if !metrics.active_topics.contains(&topic.to_string()) {
                        metrics.active_topics.push(topic.to_string());
                    }
                }

                self.is_consuming = true;

                // Start consumption loop in background
                let mut consumer_clone = KafkaConsumer {
                    consumer: self.consumer.clone(),
                    config: self.config.clone(),
                    metrics: self.metrics.clone(),
                    message_handler: None, // Will be set separately
                    event_handler: None,   // Will be set separately
                    is_connected: self.is_connected,
                    is_consuming: self.is_consuming,
                };

                tokio::spawn(async move {
                    if let Err(e) = consumer_clone.start_consumption_loop().await {
                        log::error!("Consumption loop error: {}", e);
                    }
                });

                Ok(())
            } else {
                anyhow::bail!("Kafka consumer not initialized")
            }
        }

        #[cfg(not(feature = "messaging-kafka"))]
        {
            anyhow::bail!("Kafka support not enabled")
        }
    }

    async fn stop_consuming(&mut self) -> Result<()> {
        self.is_consuming = false;

        #[cfg(feature = "messaging-kafka")]
        {
            if let Some(ref consumer) = self.consumer {
                consumer.unsubscribe();

                // Clear active topics
                let mut metrics = self.metrics.write().await;
                metrics.active_topics.clear();
            }
        }

        Ok(())
    }

    fn set_message_handler(&mut self, handler: MessageHandler) {
        self.message_handler = Some(handler);
    }

    fn set_event_handler(&mut self, handler: EventHandler) {
        self.event_handler = Some(handler);
    }

    fn is_available(&self) -> bool {
        #[cfg(feature = "messaging-kafka")]
        {
            self.is_connected && self.consumer.is_some()
        }

        #[cfg(not(feature = "messaging-kafka"))]
        {
            false
        }
    }

    async fn health_check(&self) -> Result<ConsumerHealth> {
        let metrics = self.metrics.read().await.clone();

        let mut health = ConsumerHealth {
            is_healthy: self.is_available() && self.is_consuming,
            checked_at: Utc::now(),
            error_message: None,
            metrics,
        };

        if !self.is_available() {
            health.error_message = Some("Kafka consumer not connected or not available".to_string());
        } else if !self.is_consuming {
            health.error_message = Some("Kafka consumer not actively consuming".to_string());
        }

        Ok(health)
    }

    async fn commit_message(&mut self, message: &Message) -> Result<()> {
        #[cfg(feature = "messaging-kafka")]
        {
            if let Some(ref consumer) = self.consumer {
                if let (Some(partition), Some(offset)) = (message.partition, message.offset) {
                    let mut topic_partition_list = TopicPartitionList::new();
                    topic_partition_list.add_partition_offset(&message.topic, partition, rdkafka::Offset::Offset(offset + 1))
                        .map_err(|e| anyhow::anyhow!("Failed to add partition offset: {}", e))?;

                    consumer.commit(&topic_partition_list, rdkafka::consumer::CommitMode::Sync)
                        .map_err(|e| anyhow::anyhow!("Failed to commit message: {}", e))?;
                }
            }
        }

        Ok(())
    }

    async fn close(&mut self) -> Result<()> {
        self.stop_consuming().await?;

        #[cfg(feature = "messaging-kafka")]
        {
            self.consumer = None;
        }

        self.is_connected = false;
        Ok(())
    }
}

impl Default for KafkaConsumerConfig {
    fn default() -> Self {
        let mut consumer_config = HashMap::new();
        consumer_config.insert("enable.auto.commit".to_string(), "true".to_string());
        consumer_config.insert("auto.offset.reset".to_string(), "earliest".to_string());
        consumer_config.insert("enable.partition.eof".to_string(), "false".to_string());

        Self {
            brokers: vec!["localhost:9092".to_string()],
            group_id: "ai-commit-consumer-group".to_string(),
            consumer_config,
            topics: vec!["ai-commit-events".to_string()],
            auto_commit_interval_ms: 5000,
            session_timeout_ms: 30000,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kafka_consumer_creation() {
        let config = KafkaConsumerConfig::default();
        let consumer = KafkaConsumer::new(config);

        assert_eq!(consumer.queue_type(), QueueType::Kafka);
        assert!(!consumer.is_available());
    }

    #[test]
    fn test_kafka_consumer_config_default() {
        let config = KafkaConsumerConfig::default();

        assert_eq!(config.brokers, vec!["localhost:9092"]);
        assert_eq!(config.group_id, "ai-commit-consumer-group");
        assert_eq!(config.topics, vec!["ai-commit-events"]);
        assert!(config.consumer_config.contains_key("enable.auto.commit"));
    }

    #[tokio::test]
    async fn test_health_check() {
        let config = KafkaConsumerConfig::default();
        let consumer = KafkaConsumer::new(config);

        let health = consumer.health_check().await.unwrap();
        assert!(!health.is_healthy);
        assert!(health.error_message.is_some());
    }
}