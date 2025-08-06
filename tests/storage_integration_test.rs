use ai_commit::models::review::{CodeReviewReport, ReviewSummary, ReviewMetadata, ReviewConfiguration};
use ai_commit::storage::{StorageManager, models::{ReportFilter, SortField, SortOrder}};
use ai_commit::storage::providers::{StorageConfig, StorageType};
use chrono::Utc;
use std::collections::HashMap;
use std::time::Duration;

#[cfg(feature = "storage-sqlite")]
use ai_commit::storage::providers::sqlite::SQLiteProvider;

#[cfg(feature = "storage-sqlite")]
#[tokio::test]
async fn test_storage_manager_with_sqlite() {
    // 创建测试配置
    let mut config = StorageConfig::default();
    config.enabled = true;
    config.provider = StorageType::SQLite;
    config.connection_string = "sqlite://:memory:".to_string();
    config.table_name = Some("test_reports".to_string());

    // 创建存储管理器
    let mut manager = StorageManager::new(config.clone());

    // 创建并注册 SQLite 提供商
    let provider = SQLiteProvider::new(&config.connection_string, config.table_name.unwrap())
        .await
        .expect("Failed to create SQLite provider");

    manager.register_provider(Box::new(provider))
        .expect("Failed to register provider");

    // 验证管理器已启用
    assert!(manager.is_enabled());

    // 创建测试报告
    let test_report = create_test_report();

    // 存储报告
    let report_id = manager.store_report(&test_report)
        .await
        .expect("Failed to store report");

    assert!(!report_id.is_empty());

    // 检索报告
    let retrieved_report = manager.retrieve_report(&report_id)
        .await
        .expect("Failed to retrieve report");

    assert!(retrieved_report.is_some());
    let retrieved = retrieved_report.unwrap();
    assert_eq!(retrieved.summary.project_path, test_report.summary.project_path);
    assert_eq!(retrieved.overall_score, test_report.overall_score);

    // 列出报告
    let filter = ReportFilter {
        project_path: Some("/test/project".to_string()),
        limit: Some(10),
        sort_by: Some(SortField::CreatedAt),
        sort_order: Some(SortOrder::Desc),
        ..Default::default()
    };

    let summaries = manager.list_reports(&filter)
        .await
        .expect("Failed to list reports");

    assert_eq!(summaries.len(), 1);
    assert_eq!(summaries[0].project_path, "/test/project");

    // 获取存储统计信息
    let stats = manager.get_storage_stats()
        .await
        .expect("Failed to get storage stats");

    assert_eq!(stats.total_reports, 1);
    assert!(stats.average_score.is_some());

    // 健康检查
    let health = manager.health_check().await.expect("Health check failed");
    assert!(health.get(&StorageType::SQLite).unwrap_or(&false));

    // 删除报告
    manager.delete_report(&report_id)
        .await
        .expect("Failed to delete report");

    // 验证删除
    let deleted_report = manager.retrieve_report(&report_id)
        .await
        .expect("Failed to check deleted report");

    assert!(deleted_report.is_none());
}

fn create_test_report() -> CodeReviewReport {
    CodeReviewReport {
        summary: ReviewSummary {
            project_path: "/test/project".to_string(),
            files_analyzed: 5,
            languages_detected: vec!["rust".to_string(), "javascript".to_string()],
            total_issues: 10,
            critical_issues: 1,
            high_issues: 2,
            medium_issues: 3,
            low_issues: 4,
            analysis_duration: Duration::from_secs(30),
            created_at: Utc::now(),
        },
        static_analysis_results: vec![],
        ai_review_results: vec![],
        sensitive_info_results: vec![],
        complexity_results: vec![],
        duplication_results: vec![],
        dependency_results: None,
        coverage_results: None,
        performance_results: vec![],
        trend_results: None,
        overall_score: 8.5,
        recommendations: vec!["Fix critical issues".to_string()],
        metadata: ReviewMetadata {
            version: "0.1.0".to_string(),
            user_id: Some("test_user".to_string()),
            correlation_id: None,
            tags: HashMap::new(),
            configuration: ReviewConfiguration::default(),
        },
    }
}

#[tokio::test]
async fn test_storage_manager_disabled() {
    // 创建禁用的配置
    let config = StorageConfig {
        enabled: false,
        ..Default::default()
    };

    let manager = StorageManager::new(config);
    assert!(!manager.is_enabled());

    // 尝试存储报告应该失败
    let test_report = create_test_report();
    let result = manager.store_report(&test_report).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Storage is disabled"));
}

#[tokio::test]
async fn test_storage_manager_no_provider() {
    // 创建启用但没有注册提供商的配置
    let mut config = StorageConfig::default();
    config.enabled = true;

    let manager = StorageManager::new(config);

    // 尝试存储报告应该失败
    let test_report = create_test_report();
    let result = manager.store_report(&test_report).await;
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    println!("Actual error message: {}", error_msg);
    assert!(error_msg.contains("No active storage provider") ||
            error_msg.contains("Storage is disabled") ||
            error_msg.contains("Storage provider") && error_msg.contains("not found"));
}