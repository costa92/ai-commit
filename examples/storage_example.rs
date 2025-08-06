use ai_commit::models::review::{CodeReviewReport, ReviewSummary, ReviewMetadata, ReviewConfiguration};
use ai_commit::storage::{StorageManager, models::{ReportFilter, SortField, SortOrder}};
use ai_commit::storage::providers::{StorageConfig, StorageType};
use chrono::Utc;
use std::collections::HashMap;
use std::time::Duration;

#[cfg(feature = "storage-sqlite")]
use ai_commit::storage::providers::sqlite::SQLiteProvider;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化日志
    tracing_subscriber::fmt::init();

    println!("🚀 AI-Commit 存储管理框架示例");

    // 创建存储配置
    let mut config = StorageConfig::default();
    config.enabled = true;
    config.provider = StorageType::SQLite;
    config.connection_string = "sqlite://:memory:".to_string();
    config.table_name = Some("reports".to_string());

    // 创建存储管理器
    let mut manager = StorageManager::new(config.clone());

    #[cfg(feature = "storage-sqlite")]
    {
        // 创建并注册 SQLite 提供商
        let provider = SQLiteProvider::new(&config.connection_string, config.table_name.unwrap())
            .await?;

        manager.register_provider(Box::new(provider))?;

        println!("✅ SQLite 存储提供商已注册");
    }

    // 创建示例报告
    let test_report = create_example_report();
    println!("📝 创建了示例代码审查报告");

    // 存储报告
    let report_id = manager.store_report(&test_report).await?;
    println!("💾 报告已存储，ID: {}", report_id);

    // 检索报告
    let retrieved_report = manager.retrieve_report(&report_id).await?;
    if let Some(report) = retrieved_report {
        println!("📖 成功检索报告: {}", report.summary.project_path);
        println!("   - 整体评分: {:.1}", report.overall_score);
        println!("   - 分析文件数: {}", report.summary.files_analyzed);
        println!("   - 检测到的语言: {:?}", report.summary.languages_detected);
    }

    // 列出所有报告
    let filter = ReportFilter {
        limit: Some(10),
        sort_by: Some(SortField::CreatedAt),
        sort_order: Some(SortOrder::Desc),
        ..Default::default()
    };

    let summaries = manager.list_reports(&filter).await?;
    println!("📋 找到 {} 个报告:", summaries.len());

    for summary in &summaries {
        println!("   - {}: {} (评分: {:.1})",
                 summary.id,
                 summary.project_path,
                 summary.overall_score);
    }

    // 获取存储统计信息
    let stats = manager.get_storage_stats().await?;
    println!("📊 存储统计信息:");
    println!("   - 总报告数: {}", stats.total_reports);
    if let Some(avg_score) = stats.average_score {
        println!("   - 平均评分: {:.1}", avg_score);
    }
    if let Some(oldest) = stats.oldest_report {
        println!("   - 最早报告: {}", oldest.format("%Y-%m-%d %H:%M:%S"));
    }

    // 健康检查
    let health = manager.health_check().await?;
    println!("🏥 健康检查结果:");
    for (storage_type, is_healthy) in health {
        let status = if is_healthy { "✅ 健康" } else { "❌ 不健康" };
        println!("   - {:?}: {}", storage_type, status);
    }

    // 清理：删除示例报告
    manager.delete_report(&report_id).await?;
    println!("🗑️  示例报告已删除");

    println!("✨ 存储管理框架示例完成！");

    Ok(())
}

fn create_example_report() -> CodeReviewReport {
    CodeReviewReport {
        summary: ReviewSummary {
            project_path: "/example/rust-project".to_string(),
            files_analyzed: 15,
            languages_detected: vec!["rust".to_string(), "toml".to_string()],
            total_issues: 8,
            critical_issues: 0,
            high_issues: 1,
            medium_issues: 3,
            low_issues: 4,
            analysis_duration: Duration::from_secs(45),
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
        overall_score: 8.2,
        recommendations: vec![
            "考虑重构复杂度较高的函数".to_string(),
            "添加更多的单元测试".to_string(),
            "更新过时的依赖项".to_string(),
        ],
        metadata: ReviewMetadata {
            version: "0.1.0".to_string(),
            user_id: Some("example_user".to_string()),
            correlation_id: Some("example-correlation-123".to_string()),
            tags: {
                let mut tags = HashMap::new();
                tags.insert("environment".to_string(), "development".to_string());
                tags.insert("team".to_string(), "backend".to_string());
                tags
            },
            configuration: ReviewConfiguration {
                static_analysis: true,
                ai_review: false,
                sensitive_scan: true,
                complexity_analysis: true,
                duplication_scan: false,
                dependency_scan: true,
                coverage_analysis: false,
                performance_analysis: false,
                trend_analysis: false,
            },
        },
    }
}