use async_trait::async_trait;
use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use serde_json;
use sqlx::{Row, SqlitePool};
use std::collections::HashMap;
use tracing::{debug, error, info};

use crate::models::review::CodeReviewReport;
use super::{StorageProvider, StorageType, StorageStats};
use crate::storage::models::{ReportFilter, ReportSummary, SortField, SortOrder};

/// SQLite 存储提供商
pub struct SQLiteProvider {
    pool: SqlitePool,
    table_name: String,
}

impl SQLiteProvider {
    /// 创建新的 SQLite 提供商
    pub async fn new(connection_string: &str, table_name: String) -> Result<Self> {
        let pool = SqlitePool::connect(connection_string).await?;

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
                id TEXT PRIMARY KEY,
                project_path TEXT NOT NULL,
                report_data TEXT NOT NULL,
                created_at DATETIME NOT NULL,
                overall_score REAL NOT NULL,
                files_analyzed INTEGER NOT NULL,
                languages_detected TEXT NOT NULL,
                issues_count INTEGER NOT NULL,
                critical_issues INTEGER NOT NULL,
                high_issues INTEGER NOT NULL,
                medium_issues INTEGER NOT NULL,
                low_issues INTEGER NOT NULL,
                tags TEXT
            )
            "#,
            self.table_name
        );

        sqlx::query(&create_table_sql)
            .execute(&self.pool)
            .await?;

        // Create indexes separately
        let indexes = vec![
            format!("CREATE INDEX IF NOT EXISTS idx_{}_project_path ON {} (project_path)", self.table_name, self.table_name),
            format!("CREATE INDEX IF NOT EXISTS idx_{}_created_at ON {} (created_at)", self.table_name, self.table_name),
            format!("CREATE INDEX IF NOT EXISTS idx_{}_overall_score ON {} (overall_score)", self.table_name, self.table_name),
        ];

        for index_sql in indexes {
            sqlx::query(&index_sql)
                .execute(&self.pool)
                .await?;
        }

        info!("SQLite table '{}' created or verified", self.table_name);
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
            params.push(start_date.to_rfc3339());
        }

        if let Some(end_date) = filter.end_date {
            conditions.push("created_at <= ?".to_string());
            params.push(end_date.to_rfc3339());
        }

        if let Some(min_score) = filter.min_score {
            conditions.push("overall_score >= ?".to_string());
            params.push(min_score.to_string());
        }

        if let Some(max_score) = filter.max_score {
            conditions.push("overall_score <= ?".to_string());
            params.push(max_score.to_string());
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
            SortField::UserId => "id", // SQLite doesn't have user_id column, fallback to id
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
    fn extract_summary_from_row(&self, row: &sqlx::sqlite::SqliteRow) -> Result<ReportSummary> {
        let languages_str: String = row.try_get("languages_detected")?;
        let languages_detected: Vec<String> = serde_json::from_str(&languages_str)
            .unwrap_or_else(|_| vec![]);

        let tags_str: Option<String> = row.try_get("tags")?;
        let tags: HashMap<String, String> = tags_str
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();

        let created_at_str: String = row.try_get("created_at")?;
        let created_at = DateTime::parse_from_rfc3339(&created_at_str)
            .map_err(|e| anyhow!("Failed to parse created_at: {}", e))?
            .with_timezone(&Utc);

        Ok(ReportSummary {
            id: row.try_get("id")?,
            project_path: row.try_get("project_path")?,
            created_at,
            overall_score: row.try_get("overall_score")?,
            files_analyzed: row.try_get::<i64, _>("files_analyzed")? as usize,
            languages_detected,
            issues_count: row.try_get::<i64, _>("issues_count")? as usize,
            critical_issues: row.try_get::<i64, _>("critical_issues")? as usize,
            high_issues: row.try_get::<i64, _>("high_issues")? as usize,
            medium_issues: row.try_get::<i64, _>("medium_issues")? as usize,
            low_issues: row.try_get::<i64, _>("low_issues")? as usize,
            tags,
            user_id: None, // SQLite provider doesn't store user_id in summary query
            version: None, // SQLite provider doesn't store version in summary query
        })
    }
}

#[async_trait]
impl StorageProvider for SQLiteProvider {
    fn storage_type(&self) -> StorageType {
        StorageType::SQLite
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
                critical_issues, high_issues, medium_issues, low_issues, tags
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            self.table_name
        );

        sqlx::query(&insert_sql)
            .bind(&report_id)
            .bind(&report.summary.project_path)
            .bind(&report_json)
            .bind(report.summary.created_at.to_rfc3339())
            .bind(report.overall_score)
            .bind(report.summary.files_analyzed as i64)
            .bind(&languages_json)
            .bind(report.summary.total_issues as i64)
            .bind(report.summary.critical_issues as i64)
            .bind(report.summary.high_issues as i64)
            .bind(report.summary.medium_issues as i64)
            .bind(report.summary.low_issues as i64)
            .bind(&tags_json)
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
                   medium_issues, low_issues, tags
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

        debug!("Listed {} reports", summaries.len());
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
            Ok(_) => Ok(true),
            Err(e) => {
                error!("SQLite health check failed: {}", e);
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
        let oldest_report_str: Option<String> = row.try_get("oldest_report")?;
        let newest_report_str: Option<String> = row.try_get("newest_report")?;
        let average_score: Option<f64> = row.try_get("average_score")?;

        let oldest_report = oldest_report_str
            .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
            .map(|dt| dt.with_timezone(&Utc));

        let newest_report = newest_report_str
            .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
            .map(|dt| dt.with_timezone(&Utc));

        let mut provider_specific = HashMap::new();
        provider_specific.insert("database_file".to_string(),
            serde_json::Value::String("sqlite database".to_string()));

        Ok(StorageStats {
            total_reports: total_reports as usize,
            storage_size_bytes: 0, // SQLite doesn't easily provide this info
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
    use tempfile::NamedTempFile;

    async fn create_test_provider() -> SQLiteProvider {
        // Use in-memory database for tests
        let connection_string = "sqlite://:memory:".to_string();
        SQLiteProvider::new(&connection_string, "test_reports".to_string()).await.unwrap()
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
    async fn test_store_and_retrieve_report() {
        let mut provider = create_test_provider().await;
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
    }

    #[tokio::test]
    async fn test_list_reports() {
        let mut provider = create_test_provider().await;
        let report = create_test_report();

        // Store multiple reports
        let _id1 = provider.store_report(&report).await.unwrap();
        let _id2 = provider.store_report(&report).await.unwrap();

        // List all reports
        let filter = ReportFilter::default();
        let summaries = provider.list_reports(&filter).await.unwrap();
        assert_eq!(summaries.len(), 2);
    }

    #[tokio::test]
    async fn test_delete_report() {
        let mut provider = create_test_provider().await;
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
    async fn test_health_check() {
        let provider = create_test_provider().await;
        let healthy = provider.health_check().await.unwrap();
        assert!(healthy);
    }

    #[tokio::test]
    async fn test_storage_stats() {
        let mut provider = create_test_provider().await;
        let report = create_test_report();

        // Store a report
        let _report_id = provider.store_report(&report).await.unwrap();

        // Get stats
        let stats = provider.get_storage_stats().await.unwrap();
        assert_eq!(stats.total_reports, 1);
        assert!(stats.average_score.is_some());
    }
}