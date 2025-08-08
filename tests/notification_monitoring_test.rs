use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;

use ai_commit::notification::{
    NotificationPlatform, NotificationSeverity, NotificationMessage,
    FailureLogger, FailureLoggerConfig, RetryManager, RetryConfig, RetryStrategy,
    NotificationMonitor, MonitoringConfig, MonitoringThreshold, MetricType,
    AlertLevel, AlertStatus,
};

async fn create_test_monitor() -> NotificationMonitor {
    let failure_logger = Arc::new(FailureLogger::new(FailureLoggerConfig::default()));
    let retry_manager = Arc::new(RetryManager::new(RetryConfig::default()));

    let config = MonitoringConfig {
        enabled: true,
        collection_interval: Duration::from_millis(50),
        alert_evaluation_interval: Duration::from_millis(25),
        thresholds: vec![
            MonitoringThreshold {
                metric_type: MetricType::SuccessRate,
                platform: None,
                warning_threshold: 0.95,
                critical_threshold: 0.90,
                evaluation_window: Duration::from_minutes(1),
                min_sample_size: 1,
            },
            MonitoringThreshold {
                metric_type: MetricType::FailureRate,
                platform: Some(NotificationPlatform::Email),
                warning_threshold: 0.05,
                critical_threshold: 0.10,
                evaluation_window: Duration::from_minutes(1),
                min_sample_size: 1,
            },
        ],
        alert_cooldown: Duration::from_millis(100),
        max_alerts_per_hour: 100,
        enable_self_monitoring: false, // Disable for testing
        webhook_url: None,
        email_recipients: vec![],
    };

    NotificationMonitor::new(config, failure_logger, retry_manager)
}

#[tokio::test]
async fn test_monitoring_statistics_collection() {
    let monitor = create_test_monitor().await;

    // Simulate some failures and retries
    let failure_logger = monitor.failure_logger.clone();
    let retry_manager = monitor.retry_manager.clone();

    let message = NotificationMessage::new(
        "Test Stats".to_string(),
        "Testing statistics collection".to_string(),
        NotificationSeverity::Error,
        "/test/stats".to_string(),
    );

    // Log some failures
    for i in 0..5 {
        let error = anyhow::anyhow!("Test error {}", i);
        failure_logger.log_failure(
            &message,
            NotificationPlatform::Email,
            &error,
            i % 3,
            i >= 3,
        ).await.unwrap();
    }

    // Update retry statistics
    retry_manager.update_success_statistics(NotificationPlatform::Email, 2).await;
    retry_manager.update_failure_statistics(NotificationPlatform::Feishu, 1, "network".to_string()).await;

    // Wait for statistics collection
    tokio::time::sleep(Duration::from_millis(100)).await;

    let stats = monitor.get_statistics().await;

    assert!(stats.total_notifications_sent > 0);
    assert!(stats.failed_notifications > 0);
    assert!(stats.failure_rate > 0.0);
    assert!(!stats.platform_statistics.is_empty());
}

#[tokio::test]
async fn test_alert_creation_and_resolution() {
    let monitor = create_test_monitor().await;

    // Create a threshold that should trigger
    let threshold = MonitoringThreshold {
        metric_type: MetricType::SuccessRate,
        platform: None,
        warning_threshold: 0.95,
        critical_threshold: 0.90,
        evaluation_window: Duration::from_minutes(1),
        min_sample_size: 1,
    };

    // Create an alert
    let alert = monitor.create_alert(&threshold, AlertLevel::Critical, 0.85).await;

    assert_eq!(alert.level, AlertLevel::Critical);
    assert_eq!(alert.status, AlertStatus::Active);
    assert_eq!(alert.current_value, 0.85);
    assert_eq!(alert.threshold_value, 0.90);
    assert!(alert.title.contains("SuccessRate"));
    assert!(alert.description.contains("critical"));
}

#[tokio::test]
async fn test_alert_suppression() {
    let monitor = create_test_monitor().await;

    // Create and add an alert
    let threshold = MonitoringThreshold {
        metric_type: MetricType::FailureRate,
        platform: Some(NotificationPlatform::Email),
        warning_threshold: 0.05,
        critical_threshold: 0.10,
        evaluation_window: Duration::from_minutes(1),
        min_sample_size: 1,
    };

    let mut alert = monitor.create_alert(&threshold, AlertLevel::Warning, 0.08).await;
    let alert_id = alert.id.clone();

    // Add alert to monitor
    {
        let mut alerts = monitor.alerts.write().await;
        alerts.push(alert.clone());
    }

    // Suppress the alert
    monitor.suppress_alert(&alert_id, Duration::from_minutes(30)).await.unwrap();

    // Verify alert is suppressed
    let alerts = monitor.get_active_alerts().await;
    let suppressed_alert = alerts.iter().find(|a| a.id == alert_id).unwrap();
    assert_eq!(suppressed_alert.status, AlertStatus::Suppressed);
    assert!(suppressed_alert.is_suppressed());
}

#[tokio::test]
async fn test_alert_resolution() {
    let monitor = create_test_monitor().await;

    // Create and add an alert
    let threshold = MonitoringThreshold {
        metric_type: MetricType::AverageRetryCount,
        platform: None,
        warning_threshold: 2.0,
        critical_threshold: 3.0,
        evaluation_window: Duration::from_minutes(1),
        min_sample_size: 1,
    };

    let alert = monitor.create_alert(&threshold, AlertLevel::Warning, 2.5).await;
    let alert_id = alert.id.clone();

    // Add alert to monitor
    {
        let mut alerts = monitor.alerts.write().await;
        alerts.push(alert);
    }

    // Resolve the alert
    monitor.resolve_alert(&alert_id).await.unwrap();

    // Verify alert is resolved
    let alerts = monitor.get_active_alerts().await;
    assert!(alerts.iter().find(|a| a.id == alert_id).is_none()); // Should not be in active alerts

    let history = monitor.get_alert_history().await;
    let resolved_alert = history.iter().find(|a| a.id == alert_id).unwrap();
    assert_eq!(resolved_alert.status, AlertStatus::Resolved);
    assert!(resolved_alert.resolved_at.is_some());
}

#[tokio::test]
async fn test_metric_value_calculation() {
    let monitor = create_test_monitor().await;

    // Create test statistics
    let mut stats = ai_commit::notification::MonitoringStatistics {
        total_notifications_sent: 100,
        successful_notifications: 90,
        failed_notifications: 10,
        success_rate: 0.90,
        failure_rate: 0.10,
        average_retry_count: 1.5,
        average_response_time: Duration::from_secs(2),
        platform_statistics: std::collections::HashMap::new(),
        active_alerts: 0,
        resolved_alerts: 0,
        suppressed_alerts: 0,
        last_updated: chrono::Utc::now(),
    };

    // Add platform-specific statistics
    stats.platform_statistics.insert(
        NotificationPlatform::Email,
        ai_commit::notification::PlatformMonitoringStats {
            total_sent: 50,
            successful: 48,
            failed: 2,
            success_rate: 0.96,
            average_response_time: Duration::from_secs(1),
            availability: 0.96,
            last_success: Some(chrono::Utc::now()),
            last_failure: None,
        },
    );

    // Test overall success rate
    let overall_success_rate = monitor.get_metric_value(&stats, &MetricType::SuccessRate, &None).await;
    assert_eq!(overall_success_rate, 0.90);

    // Test platform-specific success rate
    let email_success_rate = monitor.get_metric_value(
        &stats,
        &MetricType::SuccessRate,
        &Some(NotificationPlatform::Email),
    ).await;
    assert_eq!(email_success_rate, 0.96);

    // Test failure rate
    let failure_rate = monitor.get_metric_value(&stats, &MetricType::FailureRate, &None).await;
    assert_eq!(failure_rate, 0.10);

    // Test platform-specific failure rate
    let email_failure_rate = monitor.get_metric_value(
        &stats,
        &MetricType::FailureRate,
        &Some(NotificationPlatform::Email),
    ).await;
    assert_eq!(email_failure_rate, 0.04); // 1.0 - 0.96

    // Test average retry count
    let retry_count = monitor.get_metric_value(&stats, &MetricType::AverageRetryCount, &None).await;
    assert_eq!(retry_count, 1.5);

    // Test average response time
    let response_time = monitor.get_metric_value(&stats, &MetricType::AverageResponseTime, &None).await;
    assert_eq!(response_time, 2.0);

    // Test platform availability
    let email_availability = monitor.get_metric_value(
        &stats,
        &MetricType::PlatformAvailability,
        &Some(NotificationPlatform::Email),
    ).await;
    assert_eq!(email_availability, 0.96);

    // Test overall platform availability (average)
    let overall_availability = monitor.get_metric_value(&stats, &MetricType::PlatformAvailability, &None).await;
    assert_eq!(overall_availability, 0.96); // Only one platform in this test
}

#[tokio::test]
async fn test_monitoring_config_validation() {
    let config = MonitoringConfig {
        enabled: true,
        collection_interval: Duration::from_millis(100),
        alert_evaluation_interval: Duration::from_millis(50),
        thresholds: vec![
            MonitoringThreshold {
                metric_type: MetricType::SuccessRate,
                platform: None,
                warning_threshold: 0.95,
                critical_threshold: 0.90,
                evaluation_window: Duration::from_minutes(5),
                min_sample_size: 10,
            },
        ],
        alert_cooldown: Duration::from_minutes(15),
        max_alerts_per_hour: 10,
        enable_self_monitoring: true,
        webhook_url: Some("https://example.com/webhook".to_string()),
        email_recipients: vec!["admin@example.com".to_string()],
    };

    // Verify config values
    assert!(config.enabled);
    assert_eq!(config.collection_interval, Duration::from_millis(100));
    assert_eq!(config.alert_evaluation_interval, Duration::from_millis(50));
    assert_eq!(config.thresholds.len(), 1);
    assert_eq!(config.alert_cooldown, Duration::from_minutes(15));
    assert_eq!(config.max_alerts_per_hour, 10);
    assert!(config.enable_self_monitoring);
    assert!(config.webhook_url.is_some());
    assert_eq!(config.email_recipients.len(), 1);

    // Test threshold configuration
    let threshold = &config.thresholds[0];
    assert!(matches!(threshold.metric_type, MetricType::SuccessRate));
    assert!(threshold.platform.is_none());
    assert_eq!(threshold.warning_threshold, 0.95);
    assert_eq!(threshold.critical_threshold, 0.90);
    assert_eq!(threshold.evaluation_window, Duration::from_minutes(5));
    assert_eq!(threshold.min_sample_size, 10);
}

#[tokio::test]
async fn test_alert_cooldown_behavior() {
    let monitor = create_test_monitor().await;

    let threshold = MonitoringThreshold {
        metric_type: MetricType::SuccessRate,
        platform: None,
        warning_threshold: 0.95,
        critical_threshold: 0.90,
        evaluation_window: Duration::from_minutes(1),
        min_sample_size: 1,
    };

    // Create first alert
    let alert1 = monitor.create_alert(&threshold, AlertLevel::Critical, 0.85).await;
    let alert_key = format!("{:?}_{:?}", threshold.metric_type, threshold.platform);

    // Record alert time
    {
        let mut last_times = monitor.last_alert_times.write().await;
        last_times.insert(alert_key.clone(), chrono::Utc::now());
    }

    // Try to create another alert immediately (should be blocked by cooldown)
    let last_times = monitor.last_alert_times.read().await;
    let last_time = last_times.get(&alert_key).unwrap();
    let time_since_last = chrono::Utc::now().signed_duration_since(*last_time);
    let cooldown_duration = chrono::Duration::from_std(monitor.config.alert_cooldown).unwrap();

    // Should be within cooldown period
    assert!(time_since_last < cooldown_duration);

    // Wait for cooldown to expire
    tokio::time::sleep(monitor.config.alert_cooldown + Duration::from_millis(10)).await;

    // Now should be able to create another alert
    let time_since_last_after_wait = chrono::Utc::now().signed_duration_since(*last_time);
    assert!(time_since_last_after_wait > cooldown_duration);
}

#[tokio::test]
async fn test_platform_specific_monitoring() {
    let failure_logger = Arc::new(FailureLogger::new(FailureLoggerConfig::default()));
    let retry_manager = Arc::new(RetryManager::new(RetryConfig::default()));

    let config = MonitoringConfig {
        enabled: true,
        collection_interval: Duration::from_millis(50),
        alert_evaluation_interval: Duration::from_millis(25),
        thresholds: vec![
            MonitoringThreshold {
                metric_type: MetricType::SuccessRate,
                platform: Some(NotificationPlatform::Email),
                warning_threshold: 0.95,
                critical_threshold: 0.90,
                evaluation_window: Duration::from_minutes(1),
                min_sample_size: 1,
            },
            MonitoringThreshold {
                metric_type: MetricType::FailureRate,
                platform: Some(NotificationPlatform::Feishu),
                warning_threshold: 0.05,
                critical_threshold: 0.10,
                evaluation_window: Duration::from_minutes(1),
                min_sample_size: 1,
            },
        ],
        ..Default::default()
    };

    let monitor = NotificationMonitor::new(config, failure_logger.clone(), retry_manager.clone());

    // Simulate platform-specific failures
    let message = NotificationMessage::new(
        "Platform Test".to_string(),
        "Testing platform-specific monitoring".to_string(),
        NotificationSeverity::Error,
        "/test/platform".to_string(),
    );

    // Log failures for Email platform
    for i in 0..3 {
        let error = anyhow::anyhow!("Email error {}", i);
        failure_logger.log_failure(
            &message,
            NotificationPlatform::Email,
            &error,
            1,
            true,
        ).await.unwrap();
    }

    // Log failures for Feishu platform
    for i in 0..2 {
        let error = anyhow::anyhow!("Feishu error {}", i);
        failure_logger.log_failure(
            &message,
            NotificationPlatform::Feishu,
            &error,
            0,
            true,
        ).await.unwrap();
    }

    // Update platform statistics
    retry_manager.update_failure_statistics(NotificationPlatform::Email, 3, "network".to_string()).await;
    retry_manager.update_failure_statistics(NotificationPlatform::Feishu, 2, "server".to_string()).await;

    // Wait for statistics collection
    tokio::time::sleep(Duration::from_millis(100)).await;

    let stats = monitor.get_statistics().await;

    // Verify platform-specific statistics
    assert!(stats.platform_statistics.contains_key(&NotificationPlatform::Email));
    assert!(stats.platform_statistics.contains_key(&NotificationPlatform::Feishu));

    let email_stats = stats.platform_statistics.get(&NotificationPlatform::Email).unwrap();
    let feishu_stats = stats.platform_statistics.get(&NotificationPlatform::Feishu).unwrap();

    assert!(email_stats.failed > 0);
    assert!(feishu_stats.failed > 0);
}

#[tokio::test]
async fn test_monitoring_disabled() {
    let failure_logger = Arc::new(FailureLogger::new(FailureLoggerConfig::default()));
    let retry_manager = Arc::new(RetryManager::new(RetryConfig::default()));

    let config = MonitoringConfig {
        enabled: false, // Disabled
        ..Default::default()
    };

    let monitor = NotificationMonitor::new(config, failure_logger, retry_manager);

    // Try to start monitoring (should return immediately)
    let result = timeout(Duration::from_millis(100), monitor.start_monitoring()).await;

    // Should complete quickly since monitoring is disabled
    assert!(result.is_ok());
    assert!(result.unwrap().is_ok());
}

#[tokio::test]
async fn test_alert_history_tracking() {
    let monitor = create_test_monitor().await;

    let threshold = MonitoringThreshold {
        metric_type: MetricType::SuccessRate,
        platform: None,
        warning_threshold: 0.95,
        critical_threshold: 0.90,
        evaluation_window: Duration::from_minutes(1),
        min_sample_size: 1,
    };

    // Create multiple alerts
    let mut alert_ids = Vec::new();
    for i in 0..3 {
        let alert = monitor.create_alert(&threshold, AlertLevel::Warning, 0.92 - (i as f64 * 0.01)).await;
        alert_ids.push(alert.id.clone());

        // Add to both active alerts and history
        {
            let mut alerts = monitor.alerts.write().await;
            let mut history = monitor.alert_history.write().await;
            alerts.push(alert.clone());
            history.push(alert);
        }
    }

    // Resolve some alerts
    monitor.resolve_alert(&alert_ids[0]).await.unwrap();
    monitor.resolve_alert(&alert_ids[1]).await.unwrap();

    // Check active alerts (should have 1)
    let active_alerts = monitor.get_active_alerts().await;
    assert_eq!(active_alerts.len(), 1);
    assert_eq!(active_alerts[0].id, alert_ids[2]);

    // Check alert history (should have all 3)
    let history = monitor.get_alert_history().await;
    assert_eq!(history.len(), 3);

    // Verify resolved alerts have resolved_at timestamp
    let resolved_alerts: Vec<_> = history.iter()
        .filter(|a| a.status == AlertStatus::Resolved)
        .collect();
    assert_eq!(resolved_alerts.len(), 2);

    for alert in resolved_alerts {
        assert!(alert.resolved_at.is_some());
    }
}

#[tokio::test]
async fn test_concurrent_monitoring_operations() {
    let monitor = Arc::new(create_test_monitor().await);

    let mut handles = Vec::new();

    // Start multiple concurrent operations
    for i in 0..5 {
        let monitor_clone = monitor.clone();
        let handle = tokio::spawn(async move {
            let threshold = MonitoringThreshold {
                metric_type: MetricType::SuccessRate,
                platform: None,
                warning_threshold: 0.95,
                critical_threshold: 0.90,
                evaluation_window: Duration::from_minutes(1),
                min_sample_size: 1,
            };

            let alert = monitor_clone.create_alert(&threshold, AlertLevel::Warning, 0.92).await;
            let alert_id = alert.id.clone();

            // Add alert
            {
                let mut alerts = monitor_clone.alerts.write().await;
                alerts.push(alert);
            }

            // Suppress or resolve randomly
            if i % 2 == 0 {
                monitor_clone.suppress_alert(&alert_id, Duration::from_minutes(10)).await
            } else {
                monitor_clone.resolve_alert(&alert_id).await
            }
        });

        handles.push(handle);
    }

    // Wait for all operations to complete
    let results = futures::future::join_all(handles).await;

    // Verify all operations succeeded
    for result in results {
        assert!(result.unwrap().is_ok());
    }

    // Check final state
    let active_alerts = monitor.get_active_alerts().await;
    let history = monitor.get_alert_history().await;

    // Should have some combination of suppressed and resolved alerts
    assert!(active_alerts.len() + history.iter().filter(|a| a.status == AlertStatus::Resolved).count() == 5);
}

#[tokio::test]
async fn test_self_monitoring_health_check() {
    let failure_logger = Arc::new(FailureLogger::new(FailureLoggerConfig::default()));
    let retry_manager = Arc::new(RetryManager::new(RetryConfig::default()));

    let config = MonitoringConfig {
        enabled: true,
        enable_self_monitoring: true,
        collection_interval: Duration::from_millis(50),
        alert_evaluation_interval: Duration::from_millis(25),
        ..Default::default()
    };

    let monitor = NotificationMonitor::new(config, failure_logger, retry_manager);

    // Test health check
    let health_status = monitor.check_self_health().await;

    // Should be healthy initially
    assert!(health_status.is_healthy);
    assert_eq!(health_status.message, "All systems operational");

    // Simulate unhealthy condition by adding too many alerts
    {
        let mut alerts = monitor.alerts.write().await;
        for i in 0..150 { // More than the 100 alert limit
            let threshold = MonitoringThreshold {
                metric_type: MetricType::SuccessRate,
                platform: None,
                warning_threshold: 0.95,
                critical_threshold: 0.90,
                evaluation_window: Duration::from_minutes(1),
                min_sample_size: 1,
            };

            let alert = monitor.create_alert(&threshold, AlertLevel::Warning, 0.92).await;
            alerts.push(alert);
        }
    }

    // Check health again
    let unhealthy_status = monitor.check_self_health().await;
    assert!(!unhealthy_status.is_healthy);
    assert!(unhealthy_status.message.contains("Too many active alerts"));
}