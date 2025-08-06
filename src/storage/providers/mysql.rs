use async_trait::async_trait;
use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use serde_json;
use sqlx::{MySqlPool, Row};
use std::collections::HashMap;
use tracing::{debug, error, info, warn};

use crate::models::review::CodeReviewReport;
use super::{StorageProvider, StorageType, StorageStats};
use crate::storage::models::{ReportFilter, ReportSummary, SortField, SortOrder};

/// MySQL 存储提供商
pub struct MySQLProvider {
    pool: MySqlPool,
    table_name: String,
}

impl MySQLProvider {
    /// 创建新的 MySQL 提供商
    pub async fn new(connection_string: &str, table_name: String) -> Result<Self> {
        let pool = MySqlPool::connect(connection_string).await?;

        let provider = Self {
            pool,
            table_name,
        };

        // 创建表结构
        provider.create_tables().await?;

        Ok(provider)
    }

    /// 创建数据库表
    async fn create_tables(&self) -> Result<()> {
        let create_table_sql = format!(
            r#"
            CREATE TABLE IF NOT EXISTS {} (
                id VARCHAR(36) PRIMARY KEY,
                project_path VARCHAR(500) NOT NULL,
                report_data LONGTEXT NOT NULL,
                created_at DATETIME NOT NULL,
                overall_score FLOAT NOT NULL,
                files_analyzed INT NOT NULL,
                languages_detected JSON,
                issues_count INT NOT NULL,
                critical_issues INT NOT NULL,
                high_issues INT NOT NULL,
                medium_issues INT NOT NULL,
                low_issues INT NOT NULL,
                tags JSON,
                analysis_duration_ms BIGINT,
                user_id VARCHAR(100),
                version VARCHAR(50),
                INDEX idx_project_path (project_path),
                INDEX idx_created_at (created_at),
                INDEX idx_overall_score (overall_score),
                INDEX idx_project_created (project_path, created_at)
            ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci
            "#,
            self.table_name
        );

        sqlx::query(&create_table_sql)
            .execute(&self.pool)
            .await?;

        info!("MySQL table '{}' created or verified", self.table_name);
        Ok(())
    }

    /// 构建查询条件
    fn build_where_clause(&self, filter: &ReportFilter) -> (String, Vec<String>) {
        let mut conditions = Vec::new();
        let mut params = Vec::new();

        if let Some(ref project_path) = filter.project_path {
            conditions.push("project_path = ?".to_string());
            params.push(project_path.clone());
        }

        if let Some(start_date) = filter.start_date {
            conditions.push("created_at >= ?".to_string());
            params.push(start_date.format("%Y-%m-%d %H:%M:%S").to_string());
        }

        if let Some(end_date) = filter.end_date {
            conditions.push("created_at <= ?".to_string());
            params.push(end_date.format("%Y-%m-%d %H:%M:%S").to_string());
        }

        if let Some(min_score) = filter.min_score {
            conditions.push("overall_score >= ?".to_string());
            params.push(min_score.to_string());
        }

        if let Some(max_score) = filter.max_score {
            conditions.push("overall_score <= ?".to_string());
            params.push(max_score.to_string());
        }

        if let Some(ref languages) = filter.languages {
            let language_conditions: Vec<String> = languages
                .iter()
                .map(|_| "JSON_CONTAINS(languages_detected, JSON_QUOTE(?))".to_string())
                .collect();
            if !language_conditions.is_empty() {
                conditions.push(format!("({})", language_conditions.join(" OR ")));
                params.extend(languages.clone());
            }
        }

        if let Some(ref tags) = filter.tags {
            for (key, value) in tags {
                conditions.push("JSON_EXTRACT(tags, CONCAT('$.', ?)) = ?".to_string());
                params.push(key.clone());
                params.push(value.clone());
            }
        }

        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", conditions.join(" AND "))
        };

        (where_clause, params)
    }

    /// 构建排序子句
    fn build_order_clause(&self, filter: &ReportFilter) -> String {
        let sort_field = match filter.sort_by.as_ref().unwrap_or(&SortField::CreatedAt) {
            SortField::CreatedAt => "created_at",
            SortField::OverallScore => "overall_score",
            SortField::ProjectPath => "project_path",
            SortField::FilesAnalyzed => "files_analyzed",
            SortField::IssuesCount => "issues_count",
            SortField::CriticalIssues => "critical_issues",
            SortField::HighIssues => "high_issues",
            SortField::MediumIssues => "medium_issues",
            SortField::LowIssues => "low_issues",
            SortField::UserId => "user_id",
        };

        let sort_order = match filter.sort_order.as_ref().unwrap_or(&SortOrder::Desc) {
            SortOrder::Asc => "ASC",
            SortOrder::Desc => "DESC",
        };

        format!("ORDER BY {} {}", sort_field, sort_order)
    }

    /// 构建限制子句
    fn build_limit_clause(&self, filter: &ReportFilter) -> String {
        let limit = filter.limit.unwrap_or(50);
        let offset = filter.offset.unwrap_or(0);
        format!("LIMIT {} OFFSET {}", limit, offset)
    }

    /// 从行数据提取报告摘要
    fn extract_summary_from_row(&self, row: &sqlx::mysql::MySqlRow) -> Result<ReportSummary> {
        let languages_str: String = row.try_get("languages_detected")?;
        let languages_detected: Vec<String> = serde_json::from_str(&languages_str)
            .unwrap_or_else(|_| vec![]);

        let tags_str: Option<String> = row.try_get("tags")?;
        let tags: HashMap<String, String> = tags_str
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();

        let created_at: DateTime<Utc> = row.try_get("created_at")?;

        Ok(ReportSummary {
            id: row.try_get("id")?,
            project_path: row.try_get("project_path")?,
            created_at,
            overall_score: row.try_get("overall_score")?,
            files_analyzed: row.try_get::<i32, _>("files_analyzed")? as usize,
            languages_detected,
            issues_count: row.try_get::<i32, _>("issues_count")? as usize,
            critical_issues: row.try_get::<i32, _>("critical_issues")? as usize,
            high_issues: row.try_get::<i32, _>("high_issues")? as usize,
            medium_issues: row.try_get::<i32, _>("medium_issues")? as usize,
            low_issues: row.try_get::<i32, _>("low_issues")? as usize,
            tags,
            user_id: row.try_get("user_id").ok(),
            version: row.try_get("version").ok(),
        })
    }

    /// 执行数据库迁移
    pub async fn migrate(&self) -> Result<()> {
        // 检查表是否存在新列，如果不存在则添加
        let check_columns_sql = format!(
            r#"
            SELECT COLUMN_NAME
            FROM INFORMATION_SCHEMA.COLUMNS
            WHERE TABLE_SCHEMA = DATABASE()
            AND TABLE_NAME = '{}'
            AND COLUMN_NAME IN ('analysis_duration_ms', 'user_id', 'version')
            "#,
            self.table_name
        );

        let existing_columns: Vec<String> = sqlx::query(&check_columns_sql)
            .fetch_all(&self.pool)
            .await?
            .into_iter()
            .map(|row| row.try_get::<String, _>("COLUMN_NAME").unwrap_or_default())
            .collect();

        // 添加缺失的列
        let required_columns = vec![
            ("analysis_duration_ms", "BIGINT"),
            ("user_id", "VARCHAR(100)"),
            ("version", "VARCHAR(50)"),
        ];

        for (column_name, column_type) in required_columns {
            if !existing_columns.contains(&column_name.to_string()) {
                let alter_sql = format!(
                    "ALTER TABLE {} ADD COLUMN {} {}",
                    self.table_name, column_name, column_type
                );

                match sqlx::query(&alter_sql).execute(&self.pool).await {
                    Ok(_) => info!("Added column '{}' to table '{}'", column_name, self.table_name),
                    Err(e) => warn!("Failed to add column '{}': {}", column_name, e),
                }
            }
        }

        Ok(())
    }
}

#[async_trait]
impl StorageProvider for MySQLProvider {
    fn storage_type(&self) -> StorageType {
        StorageType::MySQL
    }

    async fn store_report(&mut self, report: &CodeReviewReport) -> Result<String> {
        let report_id = uuid::Uuid::new_v4().to_string();
        let report_json = serde_json::to_string(report)?;
        let languages_json = serde_json::to_string(&report.summary.languages_detected)?;
        let tags_json = serde_json::to_string(&report.metadata.tags)?;

        let insert_sql = format!(
            r#"
            INSERT INTO {} (
                id, project_path, report_data, created_at, overall_score,
                files_analyzed, languages_detected, issues_count,
                critical_issues, high_issues, medium_issues, low_issues, tags,
                analysis_duration_ms, user_id, version
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            self.table_name
        );

        sqlx::query(&insert_sql)
            .bind(&report_id)
            .bind(&report.summary.project_path)
            .bind(&report_json)
            .bind(report.summary.created_at)
            .bind(report.overall_score)
            .bind(report.summary.files_analyzed as i32)
            .bind(&languages_json)
            .bind(report.summary.total_issues as i32)
            .bind(report.summary.critical_issues as i32)
            .bind(report.summary.high_issues as i32)
            .bind(report.summary.medium_issues as i32)
            .bind(report.summary.low_issues as i32)
            .bind(&tags_json)
            .bind(report.summary.analysis_duration.as_millis() as i64)
            .bind(report.metadata.user_id.as_ref().unwrap_or(&"unknown".to_string()))
            .bind(&report.metadata.version)
            .execute(&self.pool)
            .await?;

        debug!("Stored report with ID: {}", report_id);
        Ok(report_id)
    }

    async fn retrieve_report(&self, report_id: &str) -> Result<Option<CodeReviewReport>> {
        let select_sql = format!(
            "SELECT report_data FROM {} WHERE id = ?",
            self.table_name
        );

        let row: Option<(String,)> = sqlx::query_as(&select_sql)
            .bind(report_id)
            .fetch_optional(&self.pool)
            .await?;

        if let Some((report_json,)) = row {
            let report: CodeReviewReport = serde_json::from_str(&report_json)?;
            debug!("Retrieved report with ID: {}", report_id);
            Ok(Some(report))
        } else {
            debug!("Report with ID {} not found", report_id);
            Ok(None)
        }
    }

    async fn list_reports(&self, filter: &ReportFilter) -> Result<Vec<ReportSummary>> {
        let (where_clause, params) = self.build_where_clause(filter);
        let order_clause = self.build_order_clause(filter);
        let limit_clause = self.build_limit_clause(filter);

        let select_sql = format!(
            r#"
            SELECT id, project_path, created_at, overall_score, files_analyzed,
                   languages_detected, issues_count, critical_issues, high_issues,
                   medium_issues, low_issues, tags, user_id, version
            FROM {} {} {} {}
            "#,
            self.table_name, where_clause, order_clause, limit_clause
        );

        let mut query = sqlx::query(&select_sql);
        for param in params {
            query = query.bind(param);
        }

        let rows = query.fetch_all(&self.pool).await?;
        let mut summaries = Vec::new();

        for row in rows {
            match self.extract_summary_from_row(&row) {
                Ok(summary) => summaries.push(summary),
                Err(e) => {
                    error!("Failed to extract summary from row: {}", e);
                    continue;
                }
            }
        }

        debug!("Listed {} reports from MySQL", summaries.len());
        Ok(summaries)
    }

    async fn delete_report(&mut self, report_id: &str) -> Result<()> {
        let delete_sql = format!("DELETE FROM {} WHERE id = ?", self.table_name);

        let result = sqlx::query(&delete_sql)
            .bind(report_id)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() > 0 {
            debug!("Deleted report with ID: {}", report_id);
            Ok(())
        } else {
            Err(anyhow!("Report with ID {} not found", report_id))
        }
    }

    fn is_available(&self) -> bool {
        !self.pool.is_closed()
    }

    async fn health_check(&self) -> Result<bool> {
        match sqlx::query("SELECT 1").fetch_one(&self.pool).await {
            Ok(_) => {
                debug!("MySQL health check passed");
                Ok(true)
            }
            Err(e) => {
                error!("MySQL health check failed: {}", e);
                Ok(false)
            }
        }
    }

    async fn get_storage_stats(&self) -> Result<StorageStats> {
        let stats_sql = format!(
            r#"
            SELECT
                COUNT(*) as total_reports,
                MIN(created_at) as oldest_report,
                MAX(created_at) as newest_report,
                AVG(overall_score) as average_score
            FROM {}
            "#,
            self.table_name
        );

        let row = sqlx::query(&stats_sql).fetch_one(&self.pool).await?;

        let total_reports: i64 = row.try_get("total_reports")?;
        let oldest_report: Option<DateTime<Utc>> = row.try_get("oldest_report")?;
        let newest_report: Option<DateTime<Utc>> = row.try_get("newest_report")?;
        let average_score: Option<f64> = row.try_get("average_score")?;

        // 获取表大小信息
        let table_size_sql = format!(
            r#"
            SELECT
                ROUND(((data_length + index_length) / 1024 / 1024), 2) AS size_mb
            FROM information_schema.tables
            WHERE table_schema = DATABASE()
            AND table_name = '{}'
            "#,
            self.table_name
        );

        let storage_size_bytes = match sqlx::query(&table_size_sql).fetch_optional(&self.pool).await {
            Ok(Some(row)) => {
                let size_mb: Option<f64> = row.try_get("size_mb")?;
                (size_mb.unwrap_or(0.0) * 1024.0 * 1024.0) as u64
            }
            _ => 0,
        };

        let mut provider_specific = HashMap::new();
        provider_specific.insert(
            "table_name".to_string(),
            serde_json::Value::String(self.table_name.clone()),
        );
        provider_specific.insert(
            "engine".to_string(),
            serde_json::Value::String("InnoDB".to_string()),
        );
        provider_specific.insert(
            "charset".to_string(),
            serde_json::Value::String("utf8mb4".to_string()),
        );

        Ok(StorageStats {
            total_reports: total_reports as usize,
            storage_size_bytes,
            oldest_report,
            newest_report,
            average_score: average_score.map(|s| s as f32),
            provider_specific,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::review::{ReviewSummary, ReviewMetadata, ReviewConfiguration};
    use std::time::Duration;

    // 注意：这些测试需要运行中的 MySQL 实例
    // 可以使用 Docker: docker run -d -p 3306:3306 -e MYSQL_ROOT_PASSWORD=password -e MYSQL_DATABASE=ai_commit_test mysql:8.0

    async fn create_test_provider() -> Result<MySQLProvider> {
        let connection_string = "mysql://root:password@localhost:3306/ai_commit_test";
        let table_name = "test_reports".to_string();

        MySQLProvider::new(connection_string, table_name).await
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
    #[ignore] // 需要 MySQL 实例运行
    async fn test_store_and_retrieve_report() {
        let mut provider = create_test_provider().await.unwrap();
        let report = create_test_report();

        // Store report
        let report_id = provider.store_report(&report).await.unwrap();
        assert!(!report_id.is_empty());

        // Retrieve report
        let retrieved = provider.retrieve_report(&report_id).await.unwrap();
        assert!(retrieved.is_some());

        let retrieved_report = retrieved.unwrap();
        assert_eq!(retrieved_report.summary.project_path, report.summary.project_path);
        assert_eq!(retrieved_report.overall_score, report.overall_score);

        // Cleanup
        provider.delete_report(&report_id).await.unwrap();
    }

    #[tokio::test]
    #[ignore] // 需要 MySQL 实例运行
    async fn test_list_reports() {
        let mut provider = create_test_provider().await.unwrap();
        let report = create_test_report();

        // Store multiple reports
        let id1 = provider.store_report(&report).await.unwrap();
        let id2 = provider.store_report(&report).await.unwrap();

        // List all reports
        let filter = ReportFilter::default();
        let summaries = provider.list_reports(&filter).await.unwrap();
        assert!(summaries.len() >= 2);

        // Cleanup
        provider.delete_report(&id1).await.unwrap();
        provider.delete_report(&id2).await.unwrap();
    }

    #[tokio::test]
    #[ignore] // 需要 MySQL 实例运行
    async fn test_delete_report() {
        let mut provider = create_test_provider().await.unwrap();
        let report = create_test_report();

        // Store report
        let report_id = provider.store_report(&report).await.unwrap();

        // Delete report
        provider.delete_report(&report_id).await.unwrap();

        // Verify deletion
        let retrieved = provider.retrieve_report(&report_id).await.unwrap();
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    #[ignore] // 需要 MySQL 实例运行
    async fn test_health_check() {
        let provider = create_test_provider().await.unwrap();
        let healthy = provider.health_check().await.unwrap();
        assert!(healthy);
    }

    #[tokio::test]
    #[ignore] // 需要 MySQL 实例运行
    async fn test_storage_stats() {
        let mut provider = create_test_provider().await.unwrap();
        let report = create_test_report();

        // Store a report
        let report_id = provider.store_report(&report).await.unwrap();

        // Get stats
        let stats = provider.get_storage_stats().await.unwrap();
        assert!(stats.total_reports >= 1);

        // Cleanup
        provider.delete_report(&report_id).await.unwrap();
    }

    #[tokio::test]
    #[ignore] // 需要 MySQL 实例运行
    async fn test_migration() {
        let provider = create_test_provider().await.unwrap();

        // Test migration
        provider.migrate().await.unwrap();

        // Migration should be idempotent
        provider.migrate().await.unwrap();
    }

    #[tokio::test]
    #[ignore] // 需要 MySQL 实例运行
    async fn test_filter_by_project_path() {
        let mut provider = create_test_provider().await.unwrap();
        let mut report1 = create_test_report();
        report1.summary.project_path = "/project1".to_string();

        let mut report2 = create_test_report();
        report2.summary.project_path = "/project2".to_string();

        // Store reports
        let id1 = provider.store_report(&report1).await.unwrap();
        let id2 = provider.store_report(&report2).await.unwrap();

        // Filter by project path
        let mut filter = ReportFilter::default();
        filter.project_path = Some("/project1".to_string());

        let summaries = provider.list_reports(&filter).await.unwrap();
        assert_eq!(summaries.len(), 1);
        assert_eq!(summaries[0].project_path, "/project1");

        // Cleanup
        provider.delete_report(&id1).await.unwrap();
        provider.delete_report(&id2).await.unwrap();
    }

    #[tokio::test]
    #[ignore] // 需要 MySQL 实例运行
    async fn test_filter_by_score_range() {
        let mut provider = create_test_provider().await.unwrap();
        let mut report1 = create_test_report();
        report1.overall_score = 7.0;

        let mut report2 = create_test_report();
        report2.overall_score = 9.0;

        // Store reports
        let id1 = provider.store_report(&report1).await.unwrap();
        let id2 = provider.store_report(&report2).await.unwrap();

        // Filter by score range
        let mut filter = ReportFilter::default();
        filter.min_score = Some(8.0);

        let summaries = provider.list_reports(&filter).await.unwrap();
        assert_eq!(summaries.len(), 1);
        assert_eq!(summaries[0].overall_score, 9.0);

        // Cleanup
        provider.delete_report(&id1).await.unwrap();
        provider.delete_report(&id2).await.unwrap();
    }
}