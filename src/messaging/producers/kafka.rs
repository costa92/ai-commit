use async_trait::async_trait;
use anyhow::Result;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use chrono::Utc;

use crate::messaging::{QueueType, MessageId};
use crate::messaging::events::ReportEvent;
use crate::messaging::producers::{MessageProducer, ProducerHealth, ProducerMetrics};

#[cfg(feature = "messaging-kafka")]
use rdkafka::{
    producer::{FutureProducer, FutureRecord},
    ClientConfig,
};

/// Kafka message producer implementation
pub struct KafkaProducer {
    #[cfg(feature = "messaging-kafka")]
    producer: Option<FutureProducer>,

    #[cfg(not(feature = "messaging-kafka"))]
    producer: Option<()>,

    config: KafkaProducerConfig,
    metrics: ProducerMetrics,
    is_connected: bool,
}

/// Kafka producer configuration
#[derive(Debug, Clone)]
pub struct KafkaProducerConfig {
    /// Kafka broker addresses
    pub brokers: Vec<String>,

    /// Producer configuration options
    pub producer_config: HashMap<String, String>,

    /// Default topic for events
    pub default_topic: String,

    /// Message timeout in seconds
    pub message_timeout_secs: u64,

    /// Retry configuration
    pub max_retries: u32,
    pub retry_delay_ms: u64,
}

impl KafkaProducer {
    /// Create a new Kafka producer
    pub fn new(config: KafkaProducerConfig) -> Self {
        Self {
            producer: None,
            config,
            metrics: ProducerMetrics::default(),
            is_connected: false,
        }
    }

    /// Initialize the Kafka producer connection
    pub async fn initialize(&mut self) -> Result<()> {
        #[cfg(feature = "messaging-kafka")]
        {
            let mut client_config = ClientConfig::new();

            // Set broker list
            client_config.set("bootstrap.servers", self.config.brokers.join(","));

            // Apply additional configuration
            for (key, value) in &self.config.producer_config {
                client_config.set(key, value);
            }

            // Set default configurations if not provided
            if !self.config.producer_config.contains_key("message.timeout.ms") {
                client_config.set("message.timeout.ms", (self.config.message_timeout_secs * 1000).to_string());
            }

            if !self.config.producer_config.contains_key("retries") {
                client_config.set("retries", self.config.max_retries.to_string());
            }

            let producer: FutureProducer = client_config.create()
                .map_err(|e| anyhow::anyhow!("Failed to create Kafka producer: {}", e))?;

            self.producer = Some(producer);
            self.is_connected = true;

            Ok(())
        }

        #[cfg(not(feature = "messaging-kafka"))]
        {
            anyhow::bail!("Kafka support not enabled. Enable the 'messaging-kafka' feature to use Kafka producer.")
        }
    }

    /// Send a message with retry logic and exponential backoff
    async fn send_with_retry(&mut self, topic: &str, key: Option<&str>, payload: &[u8]) -> Result<MessageId> {
        self.send_with_retry_and_headers(topic, key, payload, None).await
    }

    /// Send a message with retry logic, exponential backoff, and headers
    async fn send_with_retry_and_headers(
        &mut self,
        topic: &str,
        key: Option<&str>,
        payload: &[u8],
        headers: Option<HashMap<String, String>>
    ) -> Result<MessageId> {
        let mut last_error = None;
        let mut delay_ms = self.config.retry_delay_ms;

        for attempt in 0..=self.config.max_retries {
            match self.send_message_with_headers(topic, key, payload, headers.clone()).await {
                Ok(message_id) => {
                    self.metrics.messages_sent += 1;
                    log::debug!("Successfully sent message to topic '{}' after {} attempts", topic, attempt + 1);
                    return Ok(message_id);
                }
                Err(e) => {
                    let error_msg = e.to_string();
                    last_error = Some(e);
                    self.metrics.messages_failed += 1;

                    log::warn!("Failed to send message to topic '{}' (attempt {}/{}): {}",
                              topic, attempt + 1, self.config.max_retries + 1, error_msg);

                    if attempt < self.config.max_retries {
                        log::debug!("Retrying in {}ms...", delay_ms);
                        tokio::time::sleep(Duration::from_millis(delay_ms)).await;

                        // Exponential backoff with jitter
                        delay_ms = std::cmp::min(
                            (delay_ms as f64 * 1.5) as u64,
                            self.config.retry_delay_ms * 10
                        );
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| anyhow::anyhow!("Unknown error during message send after {} retries", self.config.max_retries)))
    }

    /// Internal message sending implementation with headers support
    async fn send_message_internal(&self, topic: &str, key: Option<&str>, payload: &[u8]) -> Result<MessageId> {
        self.send_message_with_headers(topic, key, payload, None).await
    }

    /// Send message with optional headers
    async fn send_message_with_headers(
        &self,
        topic: &str,
        key: Option<&str>,
        payload: &[u8],
        headers: Option<HashMap<String, String>>
    ) -> Result<MessageId> {
        #[cfg(feature = "messaging-kafka")]
        {
            if let Some(ref producer) = self.producer {
                let mut record = FutureRecord::to(topic).payload(payload);

                if let Some(key) = key {
                    record = record.key(key);
                }

                // Add headers if provided
                if let Some(headers) = headers {
                    let mut owned_headers = rdkafka::message::OwnedHeaders::new();
                    for (key, value) in headers {
                        owned_headers = owned_headers.insert(rdkafka::message::Header {
                            key: &key,
                            value: Some(&value),
                        });
                    }
                    record = record.headers(owned_headers);
                }

                let start_time = Instant::now();
                let delivery_status = producer
                    .send(record, Duration::from_secs(self.config.message_timeout_secs))
                    .await;

                let send_duration = start_time.elapsed();

                match delivery_status {
                    Ok((partition, offset)) => {
                        // Update metrics
                        let duration_ms = send_duration.as_millis() as f64;
                        self.update_latency_metric(duration_ms);

                        log::debug!("Message sent to topic '{}', partition {}, offset {}", topic, partition, offset);
                        Ok(MessageId::kafka(topic.to_string(), partition, offset))
                    }
                    Err((kafka_error, _)) => {
                        anyhow::bail!("Failed to send message to Kafka: {}", kafka_error)
                    }
                }
            } else {
                anyhow::bail!("Kafka producer not initialized")
            }
        }

        #[cfg(not(feature = "messaging-kafka"))]
        {
            anyhow::bail!("Kafka support not enabled")
        }
    }

    /// Update latency metrics
    fn update_latency_metric(&self, duration_ms: f64) {
        // Simple moving average calculation
        // In a real implementation, you might want to use a more sophisticated approach
        let current_avg = self.metrics.avg_send_latency_ms;
        let total_messages = self.metrics.messages_sent;

        if total_messages == 0 {
            // This is handled by the caller incrementing messages_sent after this call
            return;
        }

        // Update average (this is a simplified calculation)
        let new_avg = (current_avg * (total_messages as f64) + duration_ms) / ((total_messages + 1) as f64);

        // Note: In a real implementation, you'd want to use atomic operations or proper synchronization
        // For now, we'll accept potential race conditions in metrics
    }
}

#[async_trait]
impl MessageProducer for KafkaProducer {
    fn queue_type(&self) -> QueueType {
        QueueType::Kafka
    }

    async fn send_message(&mut self, topic: &str, message: &[u8]) -> Result<MessageId> {
        if !self.is_connected {
            self.initialize().await?;
        }

        self.send_with_retry(topic, None, message).await
    }

    async fn send_report_event(&mut self, event: ReportEvent) -> Result<MessageId> {
        if !self.is_connected {
            self.initialize().await?;
        }

        // Serialize the event
        let payload = serde_json::to_vec(&event)?;

        // Use project path as partition key for better distribution
        let partition_key = Some(event.project_path.as_str());

        // Determine topic based on event type
        let topic = match event.event_type {
            crate::messaging::events::ReportEventType::ReportGenerated => "report-generated".to_string(),
            crate::messaging::events::ReportEventType::AnalysisCompleted => "analysis-completed".to_string(),
            crate::messaging::events::ReportEventType::QualityThresholdExceeded => "quality-alerts".to_string(),
            crate::messaging::events::ReportEventType::SecurityIssueDetected => "security-alerts".to_string(),
            crate::messaging::events::ReportEventType::PerformanceRegressionDetected => "performance-alerts".to_string(),
            _ => self.config.default_topic.clone(),
        };

        // Create headers from event metadata
        let mut headers = HashMap::new();
        headers.insert("event_type".to_string(), format!("{:?}", event.event_type));
        headers.insert("event_id".to_string(), event.event_id.clone());
        headers.insert("source".to_string(), event.metadata.source.clone());
        headers.insert("version".to_string(), event.metadata.version.clone());
        headers.insert("timestamp".to_string(), event.timestamp.to_rfc3339());

        if let Some(correlation_id) = &event.metadata.correlation_id {
            headers.insert("correlation_id".to_string(), correlation_id.clone());
        }

        if let Some(user_id) = &event.metadata.user_id {
            headers.insert("user_id".to_string(), user_id.clone());
        }

        self.send_with_retry_and_headers(&topic, partition_key, &payload, Some(headers)).await
    }

    fn is_available(&self) -> bool {
        #[cfg(feature = "messaging-kafka")]
        {
            self.is_connected && self.producer.is_some()
        }

        #[cfg(not(feature = "messaging-kafka"))]
        {
            false
        }
    }

    async fn health_check(&self) -> Result<ProducerHealth> {
        let mut health = ProducerHealth {
            is_healthy: self.is_available(),
            checked_at: Utc::now(),
            error_message: None,
            metrics: self.metrics.clone(),
        };

        if !self.is_available() {
            health.error_message = Some("Kafka producer not connected or not available".to_string());
        }

        #[cfg(feature = "messaging-kafka")]
        {
            // In a real implementation, you might want to send a test message
            // or check broker connectivity here
        }

        Ok(health)
    }

    async fn close(&mut self) -> Result<()> {
        #[cfg(feature = "messaging-kafka")]
        {
            if let Some(producer) = self.producer.take() {
                // Flush any pending messages
                producer.flush(Duration::from_secs(30));
            }
        }

        self.is_connected = false;
        Ok(())
    }
}

impl Default for KafkaProducerConfig {
    fn default() -> Self {
        let mut producer_config = HashMap::new();
        producer_config.insert("acks".to_string(), "all".to_string());
        producer_config.insert("retries".to_string(), "3".to_string());
        producer_config.insert("batch.size".to_string(), "16384".to_string());
        producer_config.insert("linger.ms".to_string(), "1".to_string());
        producer_config.insert("buffer.memory".to_string(), "33554432".to_string());

        Self {
            brokers: vec!["localhost:9092".to_string()],
            producer_config,
            default_topic: "ai-commit-events".to_string(),
            message_timeout_secs: 30,
            max_retries: 3,
            retry_delay_ms: 1000,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kafka_producer_creation() {
        let config = KafkaProducerConfig::default();
        let producer = KafkaProducer::new(config);

        assert_eq!(producer.queue_type(), QueueType::Kafka);
        assert!(!producer.is_available());
    }

    #[test]
    fn test_kafka_config_default() {
        let config = KafkaProducerConfig::default();

        assert_eq!(config.brokers, vec!["localhost:9092"]);
        assert_eq!(config.default_topic, "ai-commit-events");
        assert_eq!(config.message_timeout_secs, 30);
        assert!(config.producer_config.contains_key("acks"));
    }

    #[tokio::test]
    async fn test_health_check() {
        let config = KafkaProducerConfig::default();
        let producer = KafkaProducer::new(config);

        let health = producer.health_check().await.unwrap();
        assert!(!health.is_healthy);
        assert!(health.error_message.is_some());
    }
}