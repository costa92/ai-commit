use std::collections::HashMap;
use std::path::PathBuf;
use tempfile::tempdir;
use tokio::fs;

use ai_commit::notification::{
    NotificationMessage, NotificationPlatform, NotificationSeverity,
    FailureLogger, FailureLoggerConfig, FailureType, ExportFormat,
};

#[tokio::test]
async fn test_failure_logging_basic() {
    let config = FailureLoggerConfig {
        enabled: true,
        log_to_stdout: false,
        structured_logging: false,
        ..Default::default()
    };

    let logger = FailureLogger::new(config);

    let message = NotificationMessage::new(
        "Test Failure".to_string(),
        "Testing failure logging".to_string(),
        NotificationSeverity::Error,
        "/test/project".to_string(),
    );

    let error = anyhow::anyhow!("Network connection failed");

    // Log the failure
    logger.log_failure(
        &message,
        NotificationPlatform::Email,
        &error,
        2,
        true,
    ).await.unwrap();

    // Verify the failure was recorded
    let records = logger.get_failure_records().await;
    assert_eq!(records.len(), 1);

    let record = &records[0];
    assert_eq!(record.message_id, message.id);
    assert_eq!(record.platform, NotificationPlatform::Email);
    assert!(matches!(record.failure_type, FailureType::NetworkFailure));
    assert_eq!(record.retry_count, 2);
    assert!(record.final_failure);
    assert!(record.error_message.contains("Network connection failed"));
    assert_eq!(record.context.project_path, "/test/project");
    assert_eq!(record.context.message_title, "Test Failure");
}

#[tokio::test]
async fn test_failure_type_classification() {
    let logger = FailureLogger::new(FailureLoggerConfig::default());

    let message = NotificationMessage::new(
        "Test".to_string(),
        "Content".to_string(),
        NotificationSeverity::Error,
        "/test".to_string(),
    );

    let test_cases = vec![
        ("Network connection failed", FailureType::NetworkFailure),
        ("Authentication failed", FailureType::AuthenticationFailure),
        ("Server error 500", FailureType::ServerError),
        ("Request timed out", FailureType::TimeoutError),
        ("Rate limit exceeded", FailureType::RateLimitExceeded),
        ("Invalid configuration", FailureType::ConfigurationError),
        ("Message format error", FailureType::MessageFormatError),
        ("Unknown error", FailureType::UnknownError),
    ];

    for (error_msg, expected_type) in test_cases {
        let error = anyhow::anyhow!("{}", error_msg);
        logger.log_failure(
            &message,
            NotificationPlatform::Email,
            &error,
            0,
            true,
        ).await.unwrap();
    }

    let records = logger.get_failure_records().await;
    assert_eq!(records.len(), 8);

    // Verify each failure type was classified correctly
    let expected_types = vec![
        FailureType::NetworkFailure,
        FailureType::AuthenticationFailure,
        FailureType::ServerError,
        FailureType::TimeoutError,
        FailureType::RateLimitExceeded,
        FailureType::ConfigurationError,
        FailureType::MessageFormatError,
        FailureType::UnknownError,
    ];

    for (i, expected_type) in expected_types.iter().enumerate() {
        assert!(std::mem::discriminant(&records[i].failure_type) == std::mem::discriminant(expected_type));
    }
}

#[tokio::test]
async fn test_failure_statistics() {
    let logger = FailureLogger::new(FailureLoggerConfig::default());

    let message = NotificationMessage::new(
        "Test Stats".to_string(),
        "Testing statistics".to_string(),
        NotificationSeverity::Warning,
        "/test/project".to_string(),
    );

    // Log multiple failures
    for i in 0..5 {
        let error = anyhow::anyhow!("Network error {}", i);
        logger.log_failure(
            &message,
            NotificationPlatform::Email,
            &error,
            i % 3, // Varying retry counts
            i >= 3, // Last 2 are final failures
        ).await.unwrap();
    }

    // Log failures for different platform
    for i in 0..3 {
        let error = anyhow::anyhow!("Server error {}", i);
        logger.log_failure(
            &message,
            NotificationPlatform::Feishu,
            &error,
            1,
            true,
        ).await.unwrap();
    }

    let stats = logger.get_statistics().await;

    assert_eq!(stats.total_failures, 8);
    assert_eq!(stats.final_failures, 5); // 2 from first loop + 3 from second
    assert_eq!(stats.retry_failures, 3); // 3 from first loop

    // Check platform statistics
    assert_eq!(stats.platform_failures.get(&NotificationPlatform::Email), Some(&5));
    assert_eq!(stats.platform_failures.get(&NotificationPlatform::Feishu), Some(&3));

    // Check failure type distribution
    assert!(stats.failure_type_distribution.contains_key("NetworkFailure"));
    assert!(stats.failure_type_distribution.contains_key("ServerError"));

    // Check average retry count
    let expected_avg = (0 + 1 + 2 + 0 + 1 + 1 + 1 + 1) as f64 / 8.0;
    assert!((stats.average_retry_count - expected_avg).abs() < 0.01);
}

#[tokio::test]
async fn test_failure_query_filtering() {
    let logger = FailureLogger::new(FailureLoggerConfig::default());

    let message1 = NotificationMessage::new(
        "Test 1".to_string(),
        "Content 1".to_string(),
        NotificationSeverity::Error,
        "/test1".to_string(),
    );

    let message2 = NotificationMessage::new(
        "Test 2".to_string(),
        "Content 2".to_string(),
        NotificationSeverity::Warning,
        "/test2".to_string(),
    );

    // Log failures with different characteristics
    let network_error = anyhow::anyhow!("Network connection failed");
    let server_error = anyhow::anyhow!("Internal server error");

    logger.log_failure(&message1, NotificationPlatform::Email, &network_error, 1, true).await.unwrap();
    logger.log_failure(&message2, NotificationPlatform::Feishu, &server_error, 2, false).await.unwrap();
    logger.log_failure(&message1, NotificationPlatform::Email, &network_error, 0, true).await.unwrap();

    // Query all records
    let all_records = logger.query_failures(None, None, None, None, false).await;
    assert_eq!(all_records.len(), 3);

    // Query by platform
    let email_records = logger.query_failures(Some(NotificationPlatform::Email), None, None, None, false).await;
    assert_eq!(email_records.len(), 2);

    let feishu_records = logger.query_failures(Some(NotificationPlatform::Feishu), None, None, None, false).await;
    assert_eq!(feishu_records.len(), 1);

    // Query by failure type
    let network_records = logger.query_failures(None, Some(FailureType::NetworkFailure), None, None, false).await;
    assert_eq!(network_records.len(), 2);

    let server_records = logger.query_failures(None, Some(FailureType::ServerError), None, None, false).await;
    assert_eq!(server_records.len(), 1);

    // Query final failures only
    let final_failures = logger.query_failures(None, None, None, None, true).await;
    assert_eq!(final_failures.len(), 2);

    // Query by time range
    let now = chrono::Utc::now();
    let one_hour_ago = now - chrono::Duration::hours(1);
    let one_hour_later = now + chrono::Duration::hours(1);

    let time_filtered = logger.query_failures(None, None, Some(one_hour_ago), Some(one_hour_later), false).await;
    assert_eq!(time_filtered.len(), 3); // All records should be within this range
}

#[tokio::test]
async fn test_failure_logging_to_file() {
    let temp_dir = tempdir().unwrap();
    let log_file = temp_dir.path().join("test_failures.log");

    let config = FailureLoggerConfig {
        enabled: true,
        log_file_path: Some(log_file.clone()),
        log_to_stdout: false,
        structured_logging: false,
        ..Default::default()
    };

    let logger = FailureLogger::new(config);

    let message = NotificationMessage::new(
        "File Test".to_string(),
        "Testing file logging".to_string(),
        NotificationSeverity::Critical,
        "/test/project".to_string(),
    );

    let error = anyhow::anyhow!("Test file logging error");

    // Log the failure
    logger.log_failure(
        &message,
        NotificationPlatform::DingTalk,
        &error,
        1,
        true,
    ).await.unwrap();

    // Verify file was created and contains the log entry
    assert!(log_file.exists());

    let file_content = fs::read_to_string(&log_file).await.unwrap();
    assert!(file_content.contains("NOTIFICATION FAILURE"));
    assert!(file_content.contains("File Test"));
    assert!(file_content.contains("DingTalk"));
    assert!(file_content.contains("Test file logging error"));
    assert!(file_content.contains("/test/project"));
}

#[tokio::test]
async fn test_failure_logging_structured() {
    let temp_dir = tempdir().unwrap();
    let log_file = temp_dir.path().join("structured_failures.log");

    let config = FailureLoggerConfig {
        enabled: true,
        log_file_path: Some(log_file.clone()),
        log_to_stdout: false,
        structured_logging: true,
        ..Default::default()
    };

    let logger = FailureLogger::new(config);

    let message = NotificationMessage::new(
        "Structured Test".to_string(),
        "Testing structured logging".to_string(),
        NotificationSeverity::Info,
        "/test/structured".to_string(),
    );

    let error = anyhow::anyhow!("Structured logging test error");

    // Log the failure
    logger.log_failure(
        &message,
        NotificationPlatform::WeChat,
        &error,
        0,
        false,
    ).await.unwrap();

    // Verify file contains JSON structure
    assert!(log_file.exists());

    let file_content = fs::read_to_string(&log_file).await.unwrap();

    // Parse as JSON to verify structure
    let json_record: serde_json::Value = serde_json::from_str(&file_content.trim()).unwrap();

    assert_eq!(json_record["message_id"], message.id);
    assert_eq!(json_record["platform"], "WeChat");
    assert_eq!(json_record["error_message"], "Structured logging test error");
    assert_eq!(json_record["retry_count"], 0);
    assert_eq!(json_record["final_failure"], false);
    assert_eq!(json_record["context"]["project_path"], "/test/structured");
    assert_eq!(json_record["context"]["message_title"], "Structured Test");
}

#[tokio::test]
async fn test_failure_record_cleanup() {
    let mut config = FailureLoggerConfig::default();
    config.retention_days = 0; // Immediate expiration for testing

    let logger = FailureLogger::new(config);

    let message = NotificationMessage::new(
        "Cleanup Test".to_string(),
        "Testing cleanup".to_string(),
        NotificationSeverity::Error,
        "/test/cleanup".to_string(),
    );

    // Log some failures
    for i in 0..5 {
        let error = anyhow::anyhow!("Cleanup test error {}", i);
        logger.log_failure(
            &message,
            NotificationPlatform::Email,
            &error,
            0,
            true,
        ).await.unwrap();
    }

    // Verify records exist
    let records_before = logger.get_failure_records().await;
    assert_eq!(records_before.len(), 5);

    // Wait a bit to ensure records are old enough
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

    // Cleanup old records
    let removed_count = logger.cleanup_old_records().await.unwrap();
    assert_eq!(removed_count, 5);

    // Verify records were removed
    let records_after = logger.get_failure_records().await;
    assert_eq!(records_after.len(), 0);
}

#[tokio::test]
async fn test_failure_export_json() {
    let logger = FailureLogger::new(FailureLoggerConfig::default());

    let message = NotificationMessage::new(
        "Export Test".to_string(),
        "Testing export functionality".to_string(),
        NotificationSeverity::Warning,
        "/test/export".to_string(),
    );

    // Log some failures
    for i in 0..3 {
        let error = anyhow::anyhow!("Export test error {}", i);
        logger.log_failure(
            &message,
            NotificationPlatform::Feishu,
            &error,
            i,
            i == 2,
        ).await.unwrap();
    }

    let temp_dir = tempdir().unwrap();
    let export_file = temp_dir.path().join("export_test.json");

    // Export to JSON
    logger.export_records(&export_file, ExportFormat::Json).await.unwrap();

    // Verify export file
    assert!(export_file.exists());

    let exported_content = fs::read_to_string(&export_file).await.unwrap();
    let exported_records: Vec<serde_json::Value> = serde_json::from_str(&exported_content).unwrap();

    assert_eq!(exported_records.len(), 3);

    for (i, record) in exported_records.iter().enumerate() {
        assert_eq!(record["retry_count"], i);
        assert_eq!(record["final_failure"], i == 2);
        assert!(record["error_message"].as_str().unwrap().contains(&format!("Export test error {}", i)));
    }
}

#[tokio::test]
async fn test_failure_export_csv() {
    let logger = FailureLogger::new(FailureLoggerConfig::default());

    let message = NotificationMessage::new(
        "CSV Export Test".to_string(),
        "Testing CSV export".to_string(),
        NotificationSeverity::Critical,
        "/test/csv".to_string(),
    );

    // Log a failure
    let error = anyhow::anyhow!("CSV export test error");
    logger.log_failure(
        &message,
        NotificationPlatform::DingTalk,
        &error,
        1,
        true,
    ).await.unwrap();

    let temp_dir = tempdir().unwrap();
    let export_file = temp_dir.path().join("export_test.csv");

    // Export to CSV
    logger.export_records(&export_file, ExportFormat::Csv).await.unwrap();

    // Verify export file
    assert!(export_file.exists());

    let exported_content = fs::read_to_string(&export_file).await.unwrap();
    let lines: Vec<&str> = exported_content.lines().collect();

    assert_eq!(lines.len(), 2); // Header + 1 data row

    // Check header
    assert!(lines[0].contains("ID,Message ID,Platform,Failure Type"));

    // Check data row
    let data_row = lines[1];
    assert!(data_row.contains("DingTalk"));
    assert!(data_row.contains("CSV export test error"));
    assert!(data_row.contains("1")); // retry_count
    assert!(data_row.contains("true")); // final_failure
    assert!(data_row.contains("/test/csv"));
    assert!(data_row.contains("CSV Export Test"));
}

#[tokio::test]
async fn test_failure_logging_disabled() {
    let config = FailureLoggerConfig {
        enabled: false,
        ..Default::default()
    };

    let logger = FailureLogger::new(config);

    let message = NotificationMessage::new(
        "Disabled Test".to_string(),
        "Testing disabled logging".to_string(),
        NotificationSeverity::Error,
        "/test/disabled".to_string(),
    );

    let error = anyhow::anyhow!("This should not be logged");

    // Attempt to log (should be ignored)
    logger.log_failure(
        &message,
        NotificationPlatform::Email,
        &error,
        0,
        true,
    ).await.unwrap();

    // Verify no records were created
    let records = logger.get_failure_records().await;
    assert_eq!(records.len(), 0);

    let stats = logger.get_statistics().await;
    assert_eq!(stats.total_failures, 0);
}

#[tokio::test]
async fn test_failure_logging_with_metadata() {
    let logger = FailureLogger::new(FailureLoggerConfig::default());

    let mut message = NotificationMessage::new(
        "Metadata Test".to_string(),
        "Testing metadata logging".to_string(),
        NotificationSeverity::Info,
        "/test/metadata".to_string(),
    );

    // Add metadata
    message = message
        .with_metadata("user_id".to_string(), "user123".to_string())
        .with_metadata("correlation_id".to_string(), "corr456".to_string())
        .with_metadata("custom_field".to_string(), "custom_value".to_string());

    let error = anyhow::anyhow!("Metadata test error");

    // Log the failure
    logger.log_failure(
        &message,
        NotificationPlatform::WeChat,
        &error,
        0,
        false,
    ).await.unwrap();

    // Verify metadata was preserved
    let records = logger.get_failure_records().await;
    assert_eq!(records.len(), 1);

    let record = &records[0];
    assert_eq!(record.message_metadata.get("user_id"), Some(&"user123".to_string()));
    assert_eq!(record.message_metadata.get("correlation_id"), Some(&"corr456".to_string()));
    assert_eq!(record.message_metadata.get("custom_field"), Some(&"custom_value".to_string()));

    // Verify context includes metadata
    assert_eq!(record.context.user_id, Some("user123".to_string()));
    assert_eq!(record.context.correlation_id, Some("corr456".to_string()));
}

#[tokio::test]
async fn test_concurrent_failure_logging() {
    let logger = Arc::new(FailureLogger::new(FailureLoggerConfig::default()));

    let mut handles = Vec::new();

    // Start multiple concurrent logging operations
    for i in 0..10 {
        let logger_clone = logger.clone();
        let handle = tokio::spawn(async move {
            let message = NotificationMessage::new(
                format!("Concurrent Test {}", i),
                "Testing concurrent logging".to_string(),
                NotificationSeverity::Error,
                format!("/test/concurrent/{}", i),
            );

            let error = anyhow::anyhow!("Concurrent error {}", i);

            logger_clone.log_failure(
                &message,
                NotificationPlatform::Email,
                &error,
                i % 3,
                i % 2 == 0,
            ).await
        });

        handles.push(handle);
    }

    // Wait for all operations to complete
    let results = futures::future::join_all(handles).await;

    // Verify all operations succeeded
    for result in results {
        assert!(result.unwrap().is_ok());
    }

    // Verify all records were logged
    let records = logger.get_failure_records().await;
    assert_eq!(records.len(), 10);

    // Verify statistics are correct
    let stats = logger.get_statistics().await;
    assert_eq!(stats.total_failures, 10);
    assert_eq!(stats.final_failures, 5); // Even indices
    assert_eq!(stats.retry_failures, 5); // Odd indices
}