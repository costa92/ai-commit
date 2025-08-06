use async_trait::async_trait;
use anyhow::{anyhow, Result};
use chrono::Utc;
use futures_util::TryStreamExt;
use mongodb::{
    bson::{doc, Document, DateTime as BsonDateTime},
    options::{ClientOptions, FindOptions},
    Client, Collection, Database,
};
use serde_json;
use std::collections::HashMap;
use tracing::{debug, error, info, warn};

use crate::models::review::CodeReviewReport;
use super::{StorageProvider, StorageType, StorageStats};
use crate::storage::models::{ReportFilter, ReportSummary, SortField, SortOrder};

/// MongoDB 存储提供商
pub struct MongoDBProvider {
    client: Client,
    database: Database,
    collection: Collection<Document>,
    collection_name: String,
}

impl MongoDBProvider {
    /// 创建新的 MongoDB 提供商
    pub async fn new(
        connection_string: &str,
        database_name: &str,
        collection_name: String,
    ) -> Result<Self> {
        // 解析连接字符串
        let client_options = ClientOptions::parse(connection_string).await?;
        let client = Client::with_options(client_options)?;

        // 获取数据库和集合
        let database = client.database(database_name);
        let collection = database.collection::<Document>(&collection_name);

        let provider = Self {
            client,
            database,
            collection,
            collection_name,
        };

        // 创建索引
        provider.create_indexes().await?;

        Ok(provider)
    }

    /// 创建数据库索引
    async fn create_indexes(&self) -> Result<()> {
        use mongodb::options::IndexOptions;
        use mongodb::IndexModel;

        let indexes = vec![
            IndexModel::builder()
                .keys(doc! { "project_path": 1 })
                .options(IndexOptions::builder().name("idx_project_path".to_string()).build())
                .build(),
            IndexModel::builder()
                .keys(doc! { "created_at": 1 })
                .options(IndexOptions::builder().name("idx_created_at".to_string()).build())
                .build(),
            IndexModel::builder()
                .keys(doc! { "overall_score": 1 })
                .options(IndexOptions::builder().name("idx_overall_score".to_string()).build())
                .build(),
            IndexModel::builder()
                .keys(doc! { "project_path": 1, "created_at": -1 })
                .options(IndexOptions::builder().name("idx_project_created".to_string()).build())
                .build(),
        ];

        match self.collection.create_indexes(indexes, None).await {
            Ok(_) => {
                info!("MongoDB indexes created for collection '{}'", self.collection_name);
                Ok(())
            }
            Err(e) => {
                warn!("Failed to create MongoDB indexes: {}", e);
                // 不要因为索引创建失败而终止，可能索引已经存在
                Ok(())
            }
        }
    }

    /// 将报告转换为 MongoDB 文档
    fn serialize_report(&self, report: &CodeReviewReport, report_id: &str) -> Result<Document> {
        let report_json = serde_json::to_string(report)?;
        let languages_bson = mongodb::bson::to_bson(&report.summary.languages_detected)?;
        let tags_bson = mongodb::bson::to_bson(&report.metadata.tags)?;

        let created_at_bson = BsonDateTime::from_millis(report.summary.created_at.timestamp_millis());

        let doc = doc! {
            "_id": report_id,
            "project_path": &report.summary.project_path,
            "report_data": &report_json,
            "created_at": created_at_bson,
            "overall_score": report.overall_score,
            "files_analyzed": report.summary.files_analyzed as i64,
            "languages_detected": languages_bson,
            "issues_count": report.summary.total_issues as i64,
            "critical_issues": report.summary.critical_issues as i64,
            "high_issues": report.summary.high_issues as i64,
            "medium_issues": report.summary.medium_issues as i64,
            "low_issues": report.summary.low_issues as i64,
            "tags": tags_bson,
            "analysis_duration_ms": report.summary.analysis_duration.as_millis() as i64,
            "user_id": report.metadata.user_id.as_ref().unwrap_or(&"unknown".to_string()),
            "version": &report.metadata.version,
        };

        Ok(doc)
    }

    /// 从 MongoDB 文档反序列化报告
    fn deserialize_report(&self, document: &Document) -> Result<CodeReviewReport> {
        let report_data = document
            .get_str("report_data")
            .map_err(|e| anyhow!("Failed to get report_data: {}", e))?;

        let report: CodeReviewReport = serde_json::from_str(report_data)?;
        Ok(report)
    }

    /// 构建 MongoDB 查询过滤器
    fn build_filter(&self, filter: &ReportFilter) -> Document {
        let mut mongo_filter = Document::new();

        if let Some(ref project_path) = filter.project_path {
            mongo_filter.insert("project_path", project_path);
        }

        if let Some(start_date) = filter.start_date {
            let start_date_bson = BsonDateTime::from_millis(start_date.timestamp_millis());
            let date_filter = mongo_filter
                .entry("created_at".to_string())
                .or_insert_with(|| Document::new().into())
                .as_document_mut()
                .unwrap();
            date_filter.insert("$gte", start_date_bson);
        }

        if let Some(end_date) = filter.end_date {
            let end_date_bson = BsonDateTime::from_millis(end_date.timestamp_millis());
            let date_filter = mongo_filter
                .entry("created_at".to_string())
                .or_insert_with(|| Document::new().into())
                .as_document_mut()
                .unwrap();
            date_filter.insert("$lte", end_date_bson);
        }

        if let Some(min_score) = filter.min_score {
            let score_filter = mongo_filter
                .entry("overall_score".to_string())
                .or_insert_with(|| Document::new().into())
                .as_document_mut()
                .unwrap();
            score_filter.insert("$gte", min_score);
        }

        if let Some(max_score) = filter.max_score {
            let score_filter = mongo_filter
                .entry("overall_score".to_string())
                .or_insert_with(|| Document::new().into())
                .as_document_mut()
                .unwrap();
            score_filter.insert("$lte", max_score);
        }

        if let Some(ref languages) = filter.languages {
            mongo_filter.insert("languages_detected", doc! { "$in": languages });
        }

        if let Some(ref tags) = filter.tags {
            for (key, value) in tags {
                mongo_filter.insert(format!("tags.{}", key), value);
            }
        }

        mongo_filter
    }

    /// 构建 MongoDB 排序选项
    fn build_sort(&self, filter: &ReportFilter) -> Document {
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
            SortOrder::Asc => 1,
            SortOrder::Desc => -1,
        };

        doc! { sort_field: sort_order }
    }

    /// 从文档提取报告摘要
    fn extract_summary_from_document(&self, document: &Document) -> Result<ReportSummary> {
        let languages_detected: Vec<String> = document
            .get_array("languages_detected")
            .map_err(|e| anyhow!("Failed to get languages_detected: {}", e))?
            .iter()
            .filter_map(|bson| bson.as_str().map(|s| s.to_string()))
            .collect();

        let tags: HashMap<String, String> = document
            .get_document("tags")
            .map(|doc| {
                doc.iter()
                    .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                    .collect()
            })
            .unwrap_or_default();

        let created_at_bson = document
            .get_datetime("created_at")
            .map_err(|e| anyhow!("Failed to get created_at: {}", e))?;
        let created_at = chrono::DateTime::from_timestamp_millis(created_at_bson.timestamp_millis())
            .unwrap_or_else(|| Utc::now())
            .with_timezone(&Utc);

        Ok(ReportSummary {
            id: document
                .get_str("_id")
                .map_err(|e| anyhow!("Failed to get _id: {}", e))?
                .to_string(),
            project_path: document
                .get_str("project_path")
                .map_err(|e| anyhow!("Failed to get project_path: {}", e))?
                .to_string(),
            created_at,
            overall_score: document
                .get_f64("overall_score")
                .map_err(|e| anyhow!("Failed to get overall_score: {}", e))? as f32,
            files_analyzed: document
                .get_i64("files_analyzed")
                .map_err(|e| anyhow!("Failed to get files_analyzed: {}", e))? as usize,
            languages_detected,
            issues_count: document
                .get_i64("issues_count")
                .map_err(|e| anyhow!("Failed to get issues_count: {}", e))? as usize,
            critical_issues: document
                .get_i64("critical_issues")
                .map_err(|e| anyhow!("Failed to get critical_issues: {}", e))? as usize,
            high_issues: document
                .get_i64("high_issues")
                .map_err(|e| anyhow!("Failed to get high_issues: {}", e))? as usize,
            medium_issues: document
                .get_i64("medium_issues")
                .map_err(|e| anyhow!("Failed to get medium_issues: {}", e))? as usize,
            low_issues: document
                .get_i64("low_issues")
                .map_err(|e| anyhow!("Failed to get low_issues: {}", e))? as usize,
            tags,
            user_id: document.get_str("user_id").ok().map(|s| s.to_string()),
            version: document.get_str("version").ok().map(|s| s.to_string()),
        })
    }
}

#[async_trait]
impl StorageProvider for MongoDBProvider {
    fn storage_type(&self) -> StorageType {
        StorageType::MongoDB
    }

    async fn store_report(&mut self, report: &CodeReviewReport) -> Result<String> {
        let report_id = uuid::Uuid::new_v4().to_string();
        let document = self.serialize_report(report, &report_id)?;

        match self.collection.insert_one(document, None).await {
            Ok(_) => {
                debug!("Stored report with ID: {}", report_id);
                Ok(report_id)
            }
            Err(e) => {
                error!("Failed to store report in MongoDB: {}", e);
                Err(anyhow!("Failed to store report: {}", e))
            }
        }
    }

    async fn retrieve_report(&self, report_id: &str) -> Result<Option<CodeReviewReport>> {
        let filter = doc! { "_id": report_id };

        match self.collection.find_one(filter, None).await {
            Ok(Some(document)) => {
                let report = self.deserialize_report(&document)?;
                debug!("Retrieved report with ID: {}", report_id);
                Ok(Some(report))
            }
            Ok(None) => {
                debug!("Report with ID {} not found", report_id);
                Ok(None)
            }
            Err(e) => {
                error!("Failed to retrieve report from MongoDB: {}", e);
                Err(anyhow!("Failed to retrieve report: {}", e))
            }
        }
    }

    async fn list_reports(&self, filter: &ReportFilter) -> Result<Vec<ReportSummary>> {
        let mongo_filter = self.build_filter(filter);
        let sort = self.build_sort(filter);

        let find_options = FindOptions::builder()
            .sort(sort)
            .limit(filter.limit.unwrap_or(50) as i64)
            .skip(filter.offset.unwrap_or(0) as u64)
            .projection(doc! {
                "_id": 1,
                "project_path": 1,
                "created_at": 1,
                "overall_score": 1,
                "files_analyzed": 1,
                "languages_detected": 1,
                "issues_count": 1,
                "critical_issues": 1,
                "high_issues": 1,
                "medium_issues": 1,
                "low_issues": 1,
                "tags": 1,
                "user_id": 1,
                "version": 1
            })
            .build();

        match self.collection.find(mongo_filter, find_options).await {
            Ok(mut cursor) => {
                let mut summaries = Vec::new();

                while let Ok(Some(document)) = cursor.try_next().await {
                    match self.extract_summary_from_document(&document) {
                        Ok(summary) => summaries.push(summary),
                        Err(e) => {
                            error!("Failed to extract summary from document: {}", e);
                            continue;
                        }
                    }
                }

                debug!("Listed {} reports from MongoDB", summaries.len());
                Ok(summaries)
            }
            Err(e) => {
                error!("Failed to list reports from MongoDB: {}", e);
                Err(anyhow!("Failed to list reports: {}", e))
            }
        }
    }

    async fn delete_report(&mut self, report_id: &str) -> Result<()> {
        let filter = doc! { "_id": report_id };

        match self.collection.delete_one(filter, None).await {
            Ok(result) => {
                if result.deleted_count > 0 {
                    debug!("Deleted report with ID: {}", report_id);
                    Ok(())
                } else {
                    Err(anyhow!("Report with ID {} not found", report_id))
                }
            }
            Err(e) => {
                error!("Failed to delete report from MongoDB: {}", e);
                Err(anyhow!("Failed to delete report: {}", e))
            }
        }
    }

    fn is_available(&self) -> bool {
        // MongoDB client doesn't have a simple is_closed method like SQLite
        // We'll implement this in health_check instead
        true
    }

    async fn health_check(&self) -> Result<bool> {
        match self.database.run_command(doc! { "ping": 1 }, None).await {
            Ok(_) => {
                debug!("MongoDB health check passed");
                Ok(true)
            }
            Err(e) => {
                error!("MongoDB health check failed: {}", e);
                Ok(false)
            }
        }
    }

    async fn get_storage_stats(&self) -> Result<StorageStats> {
        // 使用聚合管道获取统计信息
        let pipeline = vec![
            doc! {
                "$group": {
                    "_id": null,
                    "total_reports": { "$sum": 1 },
                    "oldest_report": { "$min": "$created_at" },
                    "newest_report": { "$max": "$created_at" },
                    "average_score": { "$avg": "$overall_score" }
                }
            }
        ];

        match self.collection.aggregate(pipeline, None).await {
            Ok(mut cursor) => {
                if let Ok(Some(document)) = cursor.try_next().await {
                    let total_reports = document.get_i32("total_reports").unwrap_or(0) as usize;
                    let oldest_report = document
                        .get_datetime("oldest_report")
                        .ok()
                        .and_then(|dt| chrono::DateTime::from_timestamp_millis(dt.timestamp_millis()))
                        .map(|dt| dt.with_timezone(&Utc));
                    let newest_report = document
                        .get_datetime("newest_report")
                        .ok()
                        .and_then(|dt| chrono::DateTime::from_timestamp_millis(dt.timestamp_millis()))
                        .map(|dt| dt.with_timezone(&Utc));
                    let average_score = document.get_f64("average_score").ok().map(|s| s as f32);

                    // 获取集合统计信息
                    let mut provider_specific = HashMap::new();
                    provider_specific.insert(
                        "collection_name".to_string(),
                        serde_json::Value::String(self.collection_name.clone()),
                    );
                    provider_specific.insert(
                        "database_name".to_string(),
                        serde_json::Value::String(self.database.name().to_string()),
                    );

                    // 尝试获取集合大小信息
                    if let Ok(stats_result) = self
                        .database
                        .run_command(doc! { "collStats": &self.collection_name }, None)
                        .await
                    {
                        if let Ok(size) = stats_result.get_i64("size") {
                            provider_specific.insert(
                                "collection_size_bytes".to_string(),
                                serde_json::Value::Number(serde_json::Number::from(size)),
                            );
                        }
                        if let Ok(count) = stats_result.get_i64("count") {
                            provider_specific.insert(
                                "document_count".to_string(),
                                serde_json::Value::Number(serde_json::Number::from(count)),
                            );
                        }
                    }

                    Ok(StorageStats {
                        total_reports,
                        storage_size_bytes: 0, // Will be filled from collection stats if available
                        oldest_report,
                        newest_report,
                        average_score,
                        provider_specific,
                    })
                } else {
                    // 没有数据的情况
                    let mut provider_specific = HashMap::new();
                    provider_specific.insert(
                        "collection_name".to_string(),
                        serde_json::Value::String(self.collection_name.clone()),
                    );
                    provider_specific.insert(
                        "database_name".to_string(),
                        serde_json::Value::String(self.database.name().to_string()),
                    );

                    Ok(StorageStats {
                        total_reports: 0,
                        storage_size_bytes: 0,
                        oldest_report: None,
                        newest_report: None,
                        average_score: None,
                        provider_specific,
                    })
                }
            }
            Err(e) => {
                error!("Failed to get MongoDB storage stats: {}", e);
                Err(anyhow!("Failed to get storage stats: {}", e))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::review::{ReviewSummary, ReviewMetadata, ReviewConfiguration};
    use std::time::Duration;

    // 注意：这些测试需要运行中的 MongoDB 实例
    // 可以使用 Docker: docker run -d -p 27017:27017 mongo:latest

    async fn create_test_provider() -> Result<MongoDBProvider> {
        let connection_string = "mongodb://localhost:27017";
        let database_name = "ai_commit_test";
        let collection_name = "test_reports".to_string();

        MongoDBProvider::new(connection_string, database_name, collection_name).await
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
    #[ignore] // 需要 MongoDB 实例运行
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
    #[ignore] // 需要 MongoDB 实例运行
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
    #[ignore] // 需要 MongoDB 实例运行
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
    #[ignore] // 需要 MongoDB 实例运行
    async fn test_health_check() {
        let provider = create_test_provider().await.unwrap();
        let healthy = provider.health_check().await.unwrap();
        assert!(healthy);
    }

    #[tokio::test]
    #[ignore] // 需要 MongoDB 实例运行
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
}