use std::collections::HashMap;
use std::time::Duration;
use tokio::time::timeout;
use chrono::Utc;

use ai_commit::messaging::{
    MessagingManager, MessagingConfig, KafkaConfig, SerializationFormat, RetryConfig,
    ReportEvent, ReportEventType, EventMetadata, QueueType
};

#[tokio::test]
async fn test_messaging_manager_basic_functionality() {
    let config = MessagingConfig::default();
    let manager = MessagingManager::new(config);

    // Test basic functionality without actual Kafka connection
    assert!(!manager.is_enabled());
    assert_eq!(manager.get_available_queues().await.len(), 0);
}

#[tokio::test]
async fn test_report_event_creation_and_serialization() {
    let config = MessagingConfig::default();
    let manager = MessagingManager::new(config);

    // Create a test report event
    let event = ReportEvent::new(
        ReportEventType::ReportGenerated,
        "/test/project".to_string(),
        serde_json::json!({
            "report_id": "test-123",
            "score": 85,
            "issues_found": 3
        }),
    )
    .with_correlation_id("corr-456".to_string())
    .with_user_id("user-789".to_string())
    .with_tag("environment".to_string(), "test".to_string());

    // Test serialization
    let serialized = manager.serialize(&event).unwrap();
    assert!(!serialized.is_empty());

    // Test deserialization
    let deserialized: ReportEvent = manager.deserialize(&serialized).unwrap();
    assert_eq!(event.event_id, deserialized.event_id);
    assert_eq!(event.event_type, deserialized.event_type);
    assert_eq!(event.project_path, deserialized.project_path);
    assert_eq!(event.metadata.correlation_id, deserialized.metadata.correlation_id);
    assert_eq!(event.metadata.user_id, deserialized.metadata.user_id);
    assert_eq!(event.metadata.tags, deserialized.metadata.tags);
}

#[tokio::test]
async fn test_messaging_config_with_kafka() {
    let mut topic_mappings = HashMap::new();
    topic_mappings.insert(ReportEventType::ReportGenerated, "reports".to_string());
    topic_mappings.insert(ReportEventType::SecurityIssueDetected, "security-alerts".to_string());

    let kafka_config = KafkaConfig {
        brokers: vec!["localhost:9092".to_string(), "localhost:9093".to_string()],
        consumer_group_id: "test-group".to_string(),
        producer_config: HashMap::new(),
        consumer_config: HashMap::new(),
        topics: HashMap::new(),
    };

    let config = MessagingConfig {
        enabled_queues: vec![QueueType::Kafka],
        kafka: Some(kafka_config),
        topic_mappings,
        serialization_format: SerializationFormat::Json,
        retry_config: RetryConfig {
            max_retries: 5,
            initial_delay_ms: 500,
            max_delay_ms: 10000,
            backoff_multiplier: 1.5,
        },
    };

    let manager = MessagingManager::new(config);
    assert!(manager.is_enabled());
}

#[tokio::test]
async fn test_event_routing() {
    let mut topic_mappings = HashMap::new();
    topic_mappings.insert(ReportEventType::ReportGenerated, "reports".to_string());
    topic_mappings.insert(ReportEventType::SecurityIssueDetected, "security".to_string());

    let event = ReportEvent::new(
        ReportEventType::SecurityIssueDetected,
        "/test/project".to_string(),
        serde_json::json!({"severity": "high"}),
    );

    let routing = event.get_routing(&topic_mappings);
    assert_eq!(routing.topic, "security");
    assert_eq!(routing.partition_key, Some("/test/project".to_string()));
    assert!(routing.headers.contains_key("event_type"));
    assert!(routing.headers.contains_key("source"));
}

#[tokio::test]
async fn test_different_serialization_formats() {
    // Test JSON serialization
    let json_config = MessagingConfig {
        serialization_format: SerializationFormat::Json,
        ..Default::default()
    };
    let json_manager = MessagingManager::new(json_config);

    let event = ReportEvent::new(
        ReportEventType::AnalysisCompleted,
        "/test".to_string(),
        serde_json::json!({"test": "data"}),
    );

    let json_serialized = json_manager.serialize(&event).unwrap();
    let json_deserialized: ReportEvent = json_manager.deserialize(&json_serialized).unwrap();
    assert_eq!(event.event_id, json_deserialized.event_id);

    // Test MessagePack serialization
    let msgpack_config = MessagingConfig {
        serialization_format: SerializationFormat::MessagePack,
        ..Default::default()
    };
    let msgpack_manager = MessagingManager::new(msgpack_config);

    let msgpack_serialized = msgpack_manager.serialize(&event).unwrap();
    let msgpack_deserialized: ReportEvent = msgpack_manager.deserialize(&msgpack_serialized).unwrap();
    assert_eq!(event.event_id, msgpack_deserialized.event_id);

    // MessagePack should generally be more compact than JSON for structured data
    // (though this might not always be true for small objects)
    println!("JSON size: {}, MessagePack size: {}", json_serialized.len(), msgpack_serialized.len());
}

#[tokio::test]
async fn test_event_metadata_default() {
    let metadata = EventMetadata::default();

    assert_eq!(metadata.source, "ai-commit");
    assert_eq!(metadata.version, env!("CARGO_PKG_VERSION"));
    assert!(metadata.correlation_id.is_none());
    assert!(metadata.user_id.is_none());
    assert!(metadata.tags.is_empty());
}

#[tokio::test]
async fn test_report_event_convenience_methods() {
    let report_data = serde_json::json!({
        "report_id": "report-123",
        "overall_score": 92
    });

    let event = ReportEvent::report_generated(
        "/test/project".to_string(),
        "report-123".to_string(),
        report_data.clone(),
    );

    assert_eq!(event.event_type, ReportEventType::ReportGenerated);
    assert_eq!(event.project_path, "/test/project");
    assert_eq!(event.report_id, Some("report-123".to_string()));
    assert_eq!(event.payload, report_data);

    let analysis_event = ReportEvent::analysis_completed(
        "/test/project".to_string(),
        serde_json::json!({"analysis": "complete"}),
    );

    assert_eq!(analysis_event.event_type, ReportEventType::AnalysisCompleted);
    assert!(analysis_event.report_id.is_none());
}

// Mock tests for Kafka functionality (these would require actual Kafka setup to run fully)
#[cfg(feature = "messaging-kafka")]
mod kafka_tests {
    use super::*;
    use ai_commit::messaging::producers::{KafkaProducer, KafkaProducerConfig};
    use ai_commit::messaging::consumers::{KafkaConsumer, KafkaConsumerConfig};

    #[tokio::test]
    async fn test_kafka_producer_creation() {
        let config = KafkaProducerConfig::default();
        let producer = KafkaProducer::new(config);

        assert_eq!(producer.queue_type(), QueueType::Kafka);
        // Without actual Kafka connection, producer won't be available
        assert!(!producer.is_available());
    }

    #[tokio::test]
    async fn test_kafka_consumer_creation() {
        let config = KafkaConsumerConfig::default();
        let consumer = KafkaConsumer::new(config);

        assert_eq!(consumer.queue_type(), QueueType::Kafka);
        // Without actual Kafka connection, consumer won't be available
        assert!(!consumer.is_available());
    }

    #[tokio::test]
    async fn test_kafka_producer_health_check() {
        let config = KafkaProducerConfig::default();
        let producer = KafkaProducer::new(config);

        let health = producer.health_check().await.unwrap();
        assert!(!health.is_healthy);
        assert!(health.error_message.is_some());
        assert_eq!(health.metrics.messages_sent, 0);
    }

    #[tokio::test]
    async fn test_kafka_consumer_health_check() {
        let config = KafkaConsumerConfig::default();
        let consumer = KafkaConsumer::new(config);

        let health = consumer.health_check().await.unwrap();
        assert!(!health.is_healthy);
        assert!(health.error_message.is_some());
        assert_eq!(health.metrics.messages_consumed, 0);
    }
}

// Performance and stress tests
#[tokio::test]
async fn test_serialization_performance() {
    let config = MessagingConfig::default();
    let manager = MessagingManager::new(config);

    // Create a larger event for performance testing
    let large_payload = serde_json::json!({
        "files": (0..100).map(|i| format!("file_{}.rs", i)).collect::<Vec<_>>(),
        "issues": (0..50).map(|i| serde_json::json!({
            "id": i,
            "severity": "medium",
            "message": format!("Issue {} description", i),
            "file": format!("file_{}.rs", i % 10),
            "line": i * 10
        })).collect::<Vec<_>>(),
        "metrics": {
            "complexity": 45.7,
            "coverage": 78.2,
            "duplication": 12.3
        }
    });

    let event = ReportEvent::new(
        ReportEventType::ReportGenerated,
        "/large/project".to_string(),
        large_payload,
    );

    let start = std::time::Instant::now();
    let serialized = manager.serialize(&event).unwrap();
    let serialize_duration = start.elapsed();

    let start = std::time::Instant::now();
    let _deserialized: ReportEvent = manager.deserialize(&serialized).unwrap();
    let deserialize_duration = start.elapsed();

    println!("Serialization took: {:?}", serialize_duration);
    println!("Deserialization took: {:?}", deserialize_duration);
    println!("Serialized size: {} bytes", serialized.len());

    // Basic performance assertions (these are quite lenient)
    assert!(serialize_duration < Duration::from_millis(100));
    assert!(deserialize_duration < Duration::from_millis(100));
}

#[tokio::test]
async fn test_concurrent_serialization() {
    let config = MessagingConfig::default();
    let manager = std::sync::Arc::new(MessagingManager::new(config));

    let mut handles = Vec::new();

    for i in 0..10 {
        let manager_clone = manager.clone();
        let handle = tokio::spawn(async move {
            let event = ReportEvent::new(
                ReportEventType::AnalysisCompleted,
                format!("/project/{}", i),
                serde_json::json!({"thread": i}),
            );

            let serialized = manager_clone.serialize(&event).unwrap();
            let deserialized: ReportEvent = manager_clone.deserialize(&serialized).unwrap();

            assert_eq!(event.event_id, deserialized.event_id);
            assert_eq!(event.project_path, deserialized.project_path);
        });

        handles.push(handle);
    }

    // Wait for all tasks to complete
    for handle in handles {
        handle.await.unwrap();
    }
}