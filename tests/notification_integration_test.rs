use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tempfile::tempdir;

use ai_commit::notification::{
    NotificationService, NotificationConfig, NotificationMessage, NotificationProvider,
    NotificationResult, NotificationPlatform, NotificationSeverity, RetryConfig,
    FailureLoggerConfig, MonitoringConfig, MonitoringThreshold, MetricType, AlertLevel,
    ExportFormat,
};

/// Comprehensive mock provider for integration testing
struct IntegrationMockProvider {
    platform: NotificationPlatform,
    call_count: Arc<AtomicU32>,
    failure_pattern: Vec<bool>, // true = fail, false = succeed
    response_delay: Duration,
}

impl IntegrationMockProvider {
    fn new(platform: NotificationPlatform, failure_pattern: Vec<bool>, response_delay: Duration) -> Self {
        Self {
            platform,
            call_count: Arc::new(AtomicU32::new(0)),
            failure_pattern,
            response_delay,
        }
    }

    fn get_call_count(&self) -> u32 {
        self.call_count.load(Ordering::SeqCst)
    }
}

#[async_trait::async_trait]
impl NotificationProvider for IntegrationMockProvider {
    fn platform(&self) -> NotificationPlatform {
        self.platform.clone()
    }

    async fn send_notification(&self, message: &NotificationMessage) -> anyhow::Result<NotificationResult> {
        let call_index = self.call_count.fetch_add(1, Ordering::SeqCst) as usize;

        // Simulate response delay
        if !self.response_delay.is_zero() {
            tokio::time::sleep(self.response_delay).await;
        }

        // Check if this call should fail based on the pattern
        let should_fail = self.failure_pattern.get(call_index).copied().unwrap_or(false);

        if should_fail {
            anyhow::bail!("Mock provider failure at call {}", call_index + 1);
        }

        Ok(NotificationResult::success(message.id.clone(), self.platform.clone()))
    }

    fn is_configured(&self) -> bool {
        true
    }

    fn supports_rich_content(&self) -> bool {
        true
    }
}

#[tokio::test]
async fn test_complete_retry_and_error_handling_flow() {
    // Create temporary directory for logs
    let temp_dir = tempdir().unwrap();
    let log_file = temp_dir.path().join("integration_test.log");

    // Configure comprehensive notification service
    let notification_config = NotificationConfig {
        enabled_platforms: vec![NotificationPlatform::Email, NotificationPlatform::Feishu],
        retry_config: RetryConfig {
            max_retries: 3,
            initial_delay: Duration::from_millis(10),
            max_delay: Duration::from_millis(100),
            backoff_multiplier: 2.0,
        },
        ..Default::default()
    };

    let failure_logger_config = FailureLoggerConfig {
        enabled: true,
        log_file_path: Some(log_file.clone()),
        log_to_stdout: false,
        structured_logging: true,
        max_records: 1000,
        retention_days: 7,
        include_sensitive_data: false,
        cleanup_interval_hours: 1,
    };

    let monitoring_config = MonitoringConfig {
        enabled: true,
        collection_interval: Duration::from_millis(50),
        alert_evaluation_interval: Duration::from_millis(25),
        thresholds: vec![
            MonitoringThreshold {
                metric_type: MetricType::SuccessRate,
                platform: None,
                warning_threshold: 0.80,
                critical_threshold: 0.60,
                evaluation_window: Duration::from_minutes(1),
                min_sample_size: 1,
            },
            MonitoringThreshold {
                metric_type: MetricType::FailureRate,
                platform: Some(NotificationPlatform::Email),
                warning_threshold: 0.20,
                critical_threshold: 0.40,
                evaluation_window: Duration::from_minutes(1),
                min_sample_size: 1,
            },
        ],
        alert_cooldown: Duration::from_millis(100),
        max_alerts_per_hour: 50,
        enable_self_monitoring: true,
        webhook_url: None,
        email_recipients: vec![],
    };

    // Create notification service with full monitoring
    let mut service = NotificationService::new_with_monitoring(
        notification_config,
        monitoring_config,
        failure_logger_config,
    );

    // Register mock providers with different failure patterns
    let email_provider = Arc::new(IntegrationMockProvider::new(
        NotificationPlatform::Email,
        vec![true, true, false], // Fail first 2 attempts, succeed on 3rd
        Duration::from_millis(5),
    ));

    let feishu_provider = Arc::new(IntegrationMockProvider::new(
        NotificationPlatform::Feishu,
        vec![true, true, true, true], // Always fail (to test final failure)
        Duration::from_millis(3),
    ));

    service.register_provider(email_provider.clone());
    service.register_provider(feishu_provider.clone());

    // Create test messages
    let successful_message = NotificationMessage::new(
        "Integration Test Success".to_string(),
        "This message should succeed after retries".to_string(),
        NotificationSeverity::Info,
        "/test/integration/success".to_string(),
    ).with_metadata("test_type".to_string(), "integration".to_string())
     .with_metadata("expected_outcome".to_string(), "success".to_string());

    let failed_message = NotificationMessage::new(
        "Integration Test Failure".to_string(),
        "This message should fail after all retries".to_string(),
        NotificationSeverity::Error,
        "/test/integration/failure".to_string(),
    ).with_metadata("test_type".to_string(), "integration".to_string())
     .with_metadata("expected_outcome".to_string(), "failure".to_string());

    // Send notifications and collect results
    let success_results = service.send_notification(successful_message.clone()).await.unwrap();
    let failure_results = service.send_notification(failed_message.clone()).await.unwrap();

    // Verify Email provider results (should succeed after retries)
    let email_success_result = success_results.iter()
        .find(|r| r.platform == NotificationPlatform::Email)
        .unwrap();
    assert!(email_success_result.success);
    assert_eq!(email_success_result.retry_count, 2); // Failed 2 times, succeeded on 3rd
    assert_eq!(email_provider.get_call_count(), 3);

    // Verify Feishu provider results (should fail after all retries)
    let feishu_failure_result = failure_results.iter()
        .find(|r| r.platform == NotificationPlatform::Feishu)
        .unwrap();
    assert!(!feishu_failure_result.success);
    assert_eq!(feishu_failure_result.retry_count, 3); // Max retries reached
    assert_eq!(feishu_provider.get_call_count(), 4); // Initial + 3 retries

    // Wait for statistics collection and processing
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Verify retry statistics
    let retry_stats = service.get_retry_statistics().await;
    assert_eq!(retry_stats.successful_retries, 1); // Email succeeded
    assert_eq!(retry_stats.failed_retries, 1); // Feishu failed
    assert_eq!(retry_stats.total_retries, 5); // 2 for email + 3 for feishu
    assert!(retry_stats.retry_rate > 0.0);

    // Verify failure logging
    let failure_stats = service.get_failure_statistics().await;
    assert_eq!(failure_stats.total_failures, 6); // All individual failure attempts
    assert_eq!(failure_stats.final_failures, 1); // Only Feishu final failure
    assert_eq!(failure_stats.retry_failures, 5); // 2 email retries + 3 feishu retries

    // Check platform-specific failure statistics
    assert!(failure_stats.platform_failures.contains_key(&NotificationPlatform::Email));
    assert!(failure_stats.platform_failures.contains_key(&NotificationPlatform::Feishu));
    assert_eq!(failure_stats.platform_failures[&NotificationPlatform::Email], 2);
    assert_eq!(failure_stats.platform_failures[&NotificationPlatform::Feishu], 4);

    // Verify failure records were logged
    let failure_logger = service.get_failure_logger();
    let failure_records = failure_logger.get_failure_records().await;
    assert_eq!(failure_records.len(), 6); // All failure attempts

    // Check that log file was created and contains structured data
    assert!(log_file.exists());
    let log_content = tokio::fs::read_to_string(&log_file).await.unwrap();
    assert!(!log_content.is_empty());

    // Verify each line is valid JSON (structured logging)
    for line in log_content.lines() {
        if !line.trim().is_empty() {
            let _: serde_json::Value = serde_json::from_str(line).unwrap();
        }
    }

    // Verify monitoring statistics
    if let Some(monitoring_stats) = service.get_monitoring_statistics().await {
        assert!(monitoring_stats.total_notifications_sent > 0);
        assert!(monitoring_stats.failed_notifications > 0);
        assert!(monitoring_stats.successful_notifications > 0);
        assert!(monitoring_stats.failure_rate > 0.0);
        assert!(monitoring_stats.success_rate < 1.0);

        // Check platform-specific monitoring stats
        assert!(!monitoring_stats.platform_statistics.is_empty());
        assert!(monitoring_stats.platform_statistics.contains_key(&NotificationPlatform::Email));
        assert!(monitoring_stats.platform_statistics.contains_key(&NotificationPlatform::Feishu));
    }

    // Verify alerts were generated due to high failure rate
    let active_alerts = service.get_active_alerts().await;
    assert!(!active_alerts.is_empty());

    // Check for failure rate alert
    let failure_rate_alert = active_alerts.iter()
        .find(|alert| matches!(alert.metric_type, MetricType::FailureRate));
    assert!(failure_rate_alert.is_some());

    let alert = failure_rate_alert.unwrap();
    assert!(matches!(alert.level, AlertLevel::Warning | AlertLevel::Critical));
    assert!(alert.current_value > alert.threshold_value);
}

#[tokio::test]
async fn test_retry_strategy_effectiveness() {
    let temp_dir = tempdir().unwrap();

    // Test different retry strategies
    let strategies = vec![
        ("FixedDelay", RetryConfig {
            max_retries: 3,
            initial_delay: Duration::from_millis(5),
            max_delay: Duration::from_millis(50),
            backoff_multiplier: 1.0, // Fixed delay
        }),
        ("ExponentialBackoff", RetryConfig {
            max_retries: 3,
            initial_delay: Duration::from_millis(5),
            max_delay: Duration::from_millis(50),
            backoff_multiplier: 2.0, // Exponential
        }),
    ];

    for (strategy_name, retry_config) in strategies {
        let notification_config = NotificationConfig {
            enabled_platforms: vec![NotificationPlatform::Email],
            retry_config,
            ..Default::default()
        };

        let service = NotificationService::new(notification_config);

        // Provider that succeeds on the 3rd attempt
        let provider = Arc::new(IntegrationMockProvider::new(
            NotificationPlatform::Email,
            vec![true, true, false], // Fail twice, then succeed
            Duration::from_millis(1),
        ));

        let mut service = service;
        service.register_provider(provider.clone());

        let message = NotificationMessage::new(
            format!("Test {}", strategy_name),
            format!("Testing {} strategy", strategy_name),
            NotificationSeverity::Info,
            format!("/test/strategy/{}", strategy_name.to_lowercase()),
        );

        let start_time = std::time::Instant::now();
        let results = service.send_notification(message).await.unwrap();
        let elapsed = start_time.elapsed();

        // Verify success
        assert_eq!(results.len(), 1);
        let result = &results[0];
        assert!(result.success);
        assert_eq!(result.retry_count, 2);
        assert_eq!(provider.get_call_count(), 3);

        // Verify timing characteristics
        match strategy_name {
            "FixedDelay" => {
                // Fixed delay: 5ms + 5ms = 10ms minimum
                assert!(elapsed >= Duration::from_millis(10));
            }
            "ExponentialBackoff" => {
                // Exponential: 5ms + 10ms = 15ms minimum
                assert!(elapsed >= Duration::from_millis(15));
            }
            _ => {}
        }

        println!("Strategy {}: elapsed {:?}, calls: {}",
                strategy_name, elapsed, provider.get_call_count());
    }
}

#[tokio::test]
async fn test_failure_recovery_and_cleanup() {
    let temp_dir = tempdir().unwrap();
    let log_file = temp_dir.path().join("recovery_test.log");

    let service = NotificationService::new_with_monitoring(
        NotificationConfig {
            enabled_platforms: vec![NotificationPlatform::Email],
            ..Default::default()
        },
        MonitoringConfig::default(),
        FailureLoggerConfig {
            enabled: true,
            log_file_path: Some(log_file.clone()),
            retention_days: 0, // Immediate expiration for testing
            ..Default::default()
        },
    );

    // Provider that always fails initially
    let provider = Arc::new(IntegrationMockProvider::new(
        NotificationPlatform::Email,
        vec![true, true, true, true], // Always fail
        Duration::from_millis(1),
    ));

    let mut service = service;
    service.register_provider(provider);

    let message = NotificationMessage::new(
        "Recovery Test".to_string(),
        "Testing failure recovery".to_string(),
        NotificationSeverity::Error,
        "/test/recovery".to_string(),
    );

    // Generate failures
    let _results = service.send_notification(message).await.unwrap();

    // Verify failures were logged
    let failure_logger = service.get_failure_logger();
    let records_before = failure_logger.get_failure_records().await;
    assert!(!records_before.is_empty());

    // Wait a bit to ensure records are old enough for cleanup
    tokio::time::sleep(Duration::from_millis(10)).await;

    // Perform cleanup
    let removed_count = service.cleanup_old_records(Duration::from_millis(1)).await.unwrap();
    assert!(removed_count > 0);

    // Verify records were cleaned up
    let records_after = failure_logger.get_failure_records().await;
    assert!(records_after.len() < records_before.len());
}

#[tokio::test]
async fn test_export_and_analysis_capabilities() {
    let temp_dir = tempdir().unwrap();

    let service = NotificationService::new_with_monitoring(
        NotificationConfig {
            enabled_platforms: vec![NotificationPlatform::Email, NotificationPlatform::Feishu],
            ..Default::default()
        },
        MonitoringConfig::default(),
        FailureLoggerConfig::default(),
    );

    // Providers with different failure patterns
    let email_provider = Arc::new(IntegrationMockProvider::new(
        NotificationPlatform::Email,
        vec![true, false], // Fail once, then succeed
        Duration::from_millis(1),
    ));

    let feishu_provider = Arc::new(IntegrationMockProvider::new(
        NotificationPlatform::Feishu,
        vec![true, true, true], // Always fail
        Duration::from_millis(1),
    ));

    let mut service = service;
    service.register_provider(email_provider);
    service.register_provider(feishu_provider);

    // Generate mixed results
    for i in 0..3 {
        let message = NotificationMessage::new(
            format!("Export Test {}", i),
            format!("Testing export functionality {}", i),
            NotificationSeverity::Warning,
            format!("/test/export/{}", i),
        );

        let _results = service.send_notification(message).await.unwrap();
    }

    // Export failure records in different formats
    let json_export = temp_dir.path().join("failures.json");
    let csv_export = temp_dir.path().join("failures.csv");

    service.export_failure_records(&json_export, ExportFormat::Json).await.unwrap();
    service.export_failure_records(&csv_export, ExportFormat::Csv).await.unwrap();

    // Verify exports were created
    assert!(json_export.exists());
    assert!(csv_export.exists());

    // Verify JSON export content
    let json_content = tokio::fs::read_to_string(&json_export).await.unwrap();
    let exported_records: Vec<serde_json::Value> = serde_json::from_str(&json_content).unwrap();
    assert!(!exported_records.is_empty());

    // Verify CSV export content
    let csv_content = tokio::fs::read_to_string(&csv_export).await.unwrap();
    let csv_lines: Vec<&str> = csv_content.lines().collect();
    assert!(csv_lines.len() > 1); // Header + data rows
    assert!(csv_lines[0].contains("ID,Message ID,Platform")); // Header check

    // Verify comprehensive statistics
    let retry_stats = service.get_retry_statistics().await;
    let failure_stats = service.get_failure_statistics().await;

    assert!(retry_stats.total_retries > 0);
    assert!(failure_stats.total_failures > 0);
    assert!(!failure_stats.platform_failures.is_empty());
    assert!(!failure_stats.failure_type_distribution.is_empty());
}

#[tokio::test]
async fn test_concurrent_notification_handling() {
    let service = Arc::new(NotificationService::new_with_monitoring(
        NotificationConfig {
            enabled_platforms: vec![NotificationPlatform::Email],
            ..Default::default()
        },
        MonitoringConfig::default(),
        FailureLoggerConfig::default(),
    ));

    // Provider with mixed success/failure pattern
    let provider = Arc::new(IntegrationMockProvider::new(
        NotificationPlatform::Email,
        vec![false, true, false, true, false], // Alternating success/failure
        Duration::from_millis(5),
    ));

    {
        let mut service_mut = Arc::try_unwrap(service.clone()).unwrap_or_else(|arc| {
            // If we can't unwrap, create a new service with the same config
            let mut new_service = NotificationService::new_with_monitoring(
                NotificationConfig {
                    enabled_platforms: vec![NotificationPlatform::Email],
                    ..Default::default()
                },
                MonitoringConfig::default(),
                FailureLoggerConfig::default(),
            );
            new_service.register_provider(provider.clone());
            return new_service;
        });
        service_mut.register_provider(provider.clone());
        // This approach won't work due to Arc ownership. Let's use a different approach.
    }

    // Create a new service for this test
    let mut service = NotificationService::new_with_monitoring(
        NotificationConfig {
            enabled_platforms: vec![NotificationPlatform::Email],
            ..Default::default()
        },
        MonitoringConfig::default(),
        FailureLoggerConfig::default(),
    );
    service.register_provider(provider.clone());
    let service = Arc::new(service);

    let mut handles = Vec::new();

    // Start multiple concurrent notification operations
    for i in 0..5 {
        let service_clone = service.clone();
        let handle = tokio::spawn(async move {
            let message = NotificationMessage::new(
                format!("Concurrent Test {}", i),
                format!("Testing concurrent handling {}", i),
                NotificationSeverity::Info,
                format!("/test/concurrent/{}", i),
            );

            service_clone.send_notification(message).await
        });

        handles.push(handle);
    }

    // Wait for all operations to complete
    let results = futures::future::join_all(handles).await;

    // Verify all operations completed
    let mut successful_notifications = 0;
    let mut failed_notifications = 0;

    for result in results {
        let notification_results = result.unwrap().unwrap();
        for notification_result in notification_results {
            if notification_result.success {
                successful_notifications += 1;
            } else {
                failed_notifications += 1;
            }
        }
    }

    // Based on the alternating pattern, we should have mixed results
    assert!(successful_notifications > 0);
    assert!(failed_notifications > 0);
    assert_eq!(successful_notifications + failed_notifications, 5);

    // Verify provider was called for all notifications
    assert_eq!(provider.get_call_count(), 5);

    // Wait for statistics processing
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Verify final statistics reflect concurrent operations
    let retry_stats = service.get_retry_statistics().await;
    let failure_stats = service.get_failure_statistics().await;

    assert_eq!(retry_stats.successful_retries as u32, successful_notifications);
    assert_eq!(failure_stats.final_failures as u32, failed_notifications);
}