use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;

use ai_commit::notification::{
    NotificationMessage, NotificationPlatform, NotificationProvider, NotificationResult,
    NotificationSeverity, RetryManager, RetryStrategy, RetryConfig, RetryCondition,
    FailureLogger, FailureLoggerConfig, NotificationMonitor, MonitoringConfig,
};

/// Mock provider for testing retry functionality
struct MockRetryProvider {
    platform: NotificationPlatform,
    fail_count: Arc<AtomicU32>,
    max_failures: u32,
    failure_message: String,
}

impl MockRetryProvider {
    fn new(platform: NotificationPlatform, max_failures: u32, failure_message: String) -> Self {
        Self {
            platform,
            fail_count: Arc::new(AtomicU32::new(0)),
            max_failures,
            failure_message,
        }
    }

    fn get_attempt_count(&self) -> u32 {
        self.fail_count.load(Ordering::SeqCst)
    }
}

#[async_trait::async_trait]
impl NotificationProvider for MockRetryProvider {
    fn platform(&self) -> NotificationPlatform {
        self.platform.clone()
    }

    async fn send_notification(&self, message: &NotificationMessage) -> anyhow::Result<NotificationResult> {
        let current_count = self.fail_count.fetch_add(1, Ordering::SeqCst);

        if current_count < self.max_failures {
            anyhow::bail!("{}", self.failure_message);
        }

        Ok(NotificationResult::success(message.id.clone(), self.platform.clone()))
    }

    fn is_configured(&self) -> bool {
        true
    }

    fn supports_rich_content(&self) -> bool {
        false
    }
}

#[tokio::test]
async fn test_exponential_backoff_retry_success() {
    let config = RetryConfig {
        strategy: RetryStrategy::ExponentialBackoff {
            initial_delay: Duration::from_millis(10),
            max_delay: Duration::from_secs(1),
            backoff_multiplier: 2.0,
            max_retries: 3,
            jitter: false,
        },
        condition: RetryCondition::default(),
        enabled: true,
        pre_retry_hook: None,
        post_retry_hook: None,
    };

    let retry_manager = RetryManager::new(config);
    let provider = Arc::new(MockRetryProvider::new(
        NotificationPlatform::Email,
        2, // Fail first 2 attempts, succeed on 3rd
        "Network connection failed".to_string(),
    ));

    let message = NotificationMessage::new(
        "Test Retry".to_string(),
        "Testing retry functionality".to_string(),
        NotificationSeverity::Info,
        "/test/project".to_string(),
    );

    let start_time = std::time::Instant::now();
    let result = retry_manager.send_with_retry(provider.clone(), &message).await;
    let elapsed = start_time.elapsed();

    assert!(result.is_ok());
    let notification_result = result.unwrap();
    assert!(notification_result.success);
    assert_eq!(notification_result.retry_count, 2);
    assert_eq!(provider.get_attempt_count(), 3);

    // Verify exponential backoff timing (should be at least 10ms + 20ms = 30ms)
    assert!(elapsed >= Duration::from_millis(30));

    // Check retry statistics
    let stats = retry_manager.get_statistics().await;
    assert_eq!(stats.successful_retries, 1);
    assert_eq!(stats.total_retries, 2);
}

#[tokio::test]
async fn test_fixed_delay_retry_failure() {
    let config = RetryConfig {
        strategy: RetryStrategy::FixedDelay {
            delay: Duration::from_millis(5),
            max_retries: 2,
        },
        condition: RetryCondition::default(),
        enabled: true,
        pre_retry_hook: None,
        post_retry_hook: None,
    };

    let retry_manager = RetryManager::new(config);
    let provider = Arc::new(MockRetryProvider::new(
        NotificationPlatform::Feishu,
        5, // Always fail
        "Server error 500".to_string(),
    ));

    let message = NotificationMessage::new(
        "Test Failure".to_string(),
        "Testing retry failure".to_string(),
        NotificationSeverity::Error,
        "/test/project".to_string(),
    );

    let result = retry_manager.send_with_retry(provider.clone(), &message).await;

    assert!(result.is_err());
    assert_eq!(provider.get_attempt_count(), 2); // max_retries attempts

    // Check retry records
    let records = retry_manager.get_retry_records().await;
    assert_eq!(records.len(), 2);
    assert!(records.last().unwrap().final_attempt);

    // Check retry statistics
    let stats = retry_manager.get_statistics().await;
    assert_eq!(stats.failed_retries, 1);
    assert_eq!(stats.total_retries, 2);
}

#[tokio::test]
async fn test_linear_backoff_retry() {
    let config = RetryConfig {
        strategy: RetryStrategy::LinearBackoff {
            initial_delay: Duration::from_millis(10),
            increment: Duration::from_millis(5),
            max_delay: Duration::from_millis(50),
            max_retries: 3,
        },
        condition: RetryCondition::default(),
        enabled: true,
        pre_retry_hook: None,
        post_retry_hook: None,
    };

    let retry_manager = RetryManager::new(config);
    let provider = Arc::new(MockRetryProvider::new(
        NotificationPlatform::DingTalk,
        2,
        "Timeout error".to_string(),
    ));

    let message = NotificationMessage::new(
        "Test Linear Backoff".to_string(),
        "Testing linear backoff".to_string(),
        NotificationSeverity::Warning,
        "/test/project".to_string(),
    );

    let start_time = std::time::Instant::now();
    let result = retry_manager.send_with_retry(provider.clone(), &message).await;
    let elapsed = start_time.elapsed();

    assert!(result.is_ok());
    assert_eq!(provider.get_attempt_count(), 3);

    // Verify linear backoff timing (10ms + 15ms = 25ms minimum)
    assert!(elapsed >= Duration::from_millis(25));
}

#[tokio::test]
async fn test_custom_delays_retry() {
    let config = RetryConfig {
        strategy: RetryStrategy::CustomDelays {
            delays: vec![
                Duration::from_millis(5),
                Duration::from_millis(10),
                Duration::from_millis(20),
            ],
        },
        condition: RetryCondition::default(),
        enabled: true,
        pre_retry_hook: None,
        post_retry_hook: None,
    };

    let retry_manager = RetryManager::new(config);
    let provider = Arc::new(MockRetryProvider::new(
        NotificationPlatform::WeChat,
        3,
        "Rate limit exceeded".to_string(),
    ));

    let message = NotificationMessage::new(
        "Test Custom Delays".to_string(),
        "Testing custom delays".to_string(),
        NotificationSeverity::Critical,
        "/test/project".to_string(),
    );

    let start_time = std::time::Instant::now();
    let result = retry_manager.send_with_retry(provider.clone(), &message).await;
    let elapsed = start_time.elapsed();

    assert!(result.is_ok());
    assert_eq!(provider.get_attempt_count(), 4);

    // Verify custom delays timing (5ms + 10ms + 20ms = 35ms minimum)
    assert!(elapsed >= Duration::from_millis(35));
}

#[tokio::test]
async fn test_retry_condition_filtering() {
    let config = RetryConfig {
        strategy: RetryStrategy::FixedDelay {
            delay: Duration::from_millis(5),
            max_retries: 3,
        },
        condition: RetryCondition {
            error_patterns: vec!["network".to_string(), "timeout".to_string()],
            http_status_codes: vec![429, 500, 502, 503, 504],
            retry_network_errors: true,
            retry_timeout_errors: true,
            retry_server_errors: false,
            custom_condition: None,
        },
        enabled: true,
        pre_retry_hook: None,
        post_retry_hook: None,
    };

    let retry_manager = RetryManager::new(config);

    // Test network error (should retry)
    let network_provider = Arc::new(MockRetryProvider::new(
        NotificationPlatform::Email,
        5,
        "Network connection failed".to_string(),
    ));

    let message = NotificationMessage::new(
        "Test Network Error".to_string(),
        "Testing network error retry".to_string(),
        NotificationSeverity::Error,
        "/test/project".to_string(),
    );

    let result = retry_manager.send_with_retry(network_provider.clone(), &message).await;
    assert!(result.is_err());
    assert_eq!(network_provider.get_attempt_count(), 3); // Should retry

    // Test authentication error (should not retry based on our condition)
    let auth_provider = Arc::new(MockRetryProvider::new(
        NotificationPlatform::Feishu,
        5,
        "Authentication failed".to_string(),
    ));

    let result = retry_manager.send_with_retry(auth_provider.clone(), &message).await;
    assert!(result.is_err());
    assert_eq!(auth_provider.get_attempt_count(), 1); // Should not retry
}

#[tokio::test]
async fn test_retry_disabled() {
    let config = RetryConfig {
        strategy: RetryStrategy::FixedDelay {
            delay: Duration::from_millis(5),
            max_retries: 3,
        },
        condition: RetryCondition::default(),
        enabled: false, // Disabled
        pre_retry_hook: None,
        post_retry_hook: None,
    };

    let retry_manager = RetryManager::new(config);
    let provider = Arc::new(MockRetryProvider::new(
        NotificationPlatform::Email,
        5,
        "Network error".to_string(),
    ));

    let message = NotificationMessage::new(
        "Test Disabled Retry".to_string(),
        "Testing disabled retry".to_string(),
        NotificationSeverity::Info,
        "/test/project".to_string(),
    );

    let result = retry_manager.send_with_retry(provider.clone(), &message).await;
    assert!(result.is_err());
    assert_eq!(provider.get_attempt_count(), 1); // No retries
}

#[tokio::test]
async fn test_failure_logging_integration() {
    let failure_logger = Arc::new(FailureLogger::new(FailureLoggerConfig {
        enabled: true,
        log_to_stdout: false,
        structured_logging: false,
        ..Default::default()
    }));

    let retry_manager = RetryManager::new(RetryConfig::default());

    let provider = Arc::new(MockRetryProvider::new(
        NotificationPlatform::Email,
        5, // Always fail
        "Network connection timeout".to_string(),
    ));

    let message = NotificationMessage::new(
        "Test Failure Logging".to_string(),
        "Testing failure logging integration".to_string(),
        NotificationSeverity::Error,
        "/test/project".to_string(),
    );

    // Attempt to send (will fail)
    let result = retry_manager.send_with_retry(provider.clone(), &message).await;
    assert!(result.is_err());

    // Log the failure
    let error = result.unwrap_err();
    failure_logger.log_failure(
        &message,
        NotificationPlatform::Email,
        &error,
        3,
        true,
    ).await.unwrap();

    // Verify failure was logged
    let records = failure_logger.get_failure_records().await;
    assert_eq!(records.len(), 1);

    let record = &records[0];
    assert_eq!(record.message_id, message.id);
    assert_eq!(record.platform, NotificationPlatform::Email);
    assert_eq!(record.retry_count, 3);
    assert!(record.final_failure);
    assert!(record.error_message.contains("Network connection timeout"));

    // Check failure statistics
    let stats = failure_logger.get_statistics().await;
    assert_eq!(stats.total_failures, 1);
    assert_eq!(stats.final_failures, 1);
    assert_eq!(stats.retry_failures, 0);
}

#[tokio::test]
async fn test_monitoring_integration() {
    let failure_logger = Arc::new(FailureLogger::new(FailureLoggerConfig::default()));
    let retry_manager = Arc::new(RetryManager::new(RetryConfig::default()));

    let monitoring_config = MonitoringConfig {
        enabled: true,
        collection_interval: Duration::from_millis(100),
        alert_evaluation_interval: Duration::from_millis(50),
        ..Default::default()
    };

    let monitor = NotificationMonitor::new(
        monitoring_config,
        failure_logger.clone(),
        retry_manager.clone(),
    );

    // Simulate some failures to trigger monitoring
    let provider = Arc::new(MockRetryProvider::new(
        NotificationPlatform::Email,
        5,
        "Server error".to_string(),
    ));

    let message = NotificationMessage::new(
        "Test Monitoring".to_string(),
        "Testing monitoring integration".to_string(),
        NotificationSeverity::Error,
        "/test/project".to_string(),
    );

    // Generate some failures
    for _ in 0..3 {
        let result = retry_manager.send_with_retry(provider.clone(), &message).await;
        if let Err(error) = result {
            failure_logger.log_failure(
                &message,
                NotificationPlatform::Email,
                &error,
                3,
                true,
            ).await.unwrap();
        }
    }

    // Wait a bit for statistics to be collected
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Check monitoring statistics
    let stats = monitor.get_statistics().await;
    assert!(stats.total_retries > 0);
    assert!(stats.failed_retries > 0);
    assert!(stats.failure_rate > 0.0);
}

#[tokio::test]
async fn test_retry_record_cleanup() {
    let retry_manager = RetryManager::new(RetryConfig::default());

    let provider = Arc::new(MockRetryProvider::new(
        NotificationPlatform::Email,
        5,
        "Test error".to_string(),
    ));

    let message = NotificationMessage::new(
        "Test Cleanup".to_string(),
        "Testing record cleanup".to_string(),
        NotificationSeverity::Info,
        "/test/project".to_string(),
    );

    // Generate some retry records
    let _ = retry_manager.send_with_retry(provider, &message).await;

    // Verify records exist
    let records_before = retry_manager.get_retry_records().await;
    assert!(!records_before.is_empty());

    // Cleanup with very short max age (should remove all records)
    retry_manager.cleanup_old_records(Duration::from_millis(1)).await;

    // Wait a bit to ensure records are old enough
    tokio::time::sleep(Duration::from_millis(10)).await;

    // Cleanup again
    retry_manager.cleanup_old_records(Duration::from_millis(1)).await;

    // Verify records were cleaned up
    let records_after = retry_manager.get_retry_records().await;
    assert!(records_after.len() <= records_before.len());
}

#[tokio::test]
async fn test_concurrent_retry_operations() {
    let retry_manager = Arc::new(RetryManager::new(RetryConfig {
        strategy: RetryStrategy::FixedDelay {
            delay: Duration::from_millis(10),
            max_retries: 2,
        },
        ..Default::default()
    }));

    let mut handles = Vec::new();

    // Start multiple concurrent retry operations
    for i in 0..5 {
        let manager = retry_manager.clone();
        let provider = Arc::new(MockRetryProvider::new(
            NotificationPlatform::Email,
            1, // Succeed on second attempt
            format!("Concurrent error {}", i),
        ));

        let message = NotificationMessage::new(
            format!("Concurrent Test {}", i),
            "Testing concurrent operations".to_string(),
            NotificationSeverity::Info,
            "/test/project".to_string(),
        );

        let handle = tokio::spawn(async move {
            manager.send_with_retry(provider, &message).await
        });

        handles.push(handle);
    }

    // Wait for all operations to complete
    let results = futures::future::join_all(handles).await;

    // Verify all operations completed successfully
    for result in results {
        let retry_result = result.unwrap();
        assert!(retry_result.is_ok());
        let notification_result = retry_result.unwrap();
        assert!(notification_result.success);
        assert_eq!(notification_result.retry_count, 1);
    }

    // Check final statistics
    let stats = retry_manager.get_statistics().await;
    assert_eq!(stats.successful_retries, 5);
    assert_eq!(stats.total_retries, 5); // Each operation retried once
}

#[tokio::test]
async fn test_retry_timeout_behavior() {
    let config = RetryConfig {
        strategy: RetryStrategy::FixedDelay {
            delay: Duration::from_millis(100),
            max_retries: 3,
        },
        ..Default::default()
    };

    let retry_manager = RetryManager::new(config);
    let provider = Arc::new(MockRetryProvider::new(
        NotificationPlatform::Email,
        5, // Always fail
        "Timeout error".to_string(),
    ));

    let message = NotificationMessage::new(
        "Test Timeout".to_string(),
        "Testing timeout behavior".to_string(),
        NotificationSeverity::Error,
        "/test/project".to_string(),
    );

    // Set a timeout that should allow completion
    let result = timeout(
        Duration::from_secs(1),
        retry_manager.send_with_retry(provider.clone(), &message)
    ).await;

    assert!(result.is_ok()); // Should not timeout
    let retry_result = result.unwrap();
    assert!(retry_result.is_err()); // But should fail due to max retries
    assert_eq!(provider.get_attempt_count(), 3);
}