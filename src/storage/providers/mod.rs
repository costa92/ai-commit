use async_trait::async_trait;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::models::review::CodeReviewReport;
use super::models::{ReportFilter, ReportSummary, ReportQueryResult};

#[cfg(feature = "storage-sqlite")]
pub mod sqlite;

#[cfg(feature = "storage-mysql")]
pub mod mysql;

#[cfg(feature = "storage-mysql")]
pub use mysql::MySQLProvider;

#[cfg(feature = "storage-mongodb")]
pub mod mongodb;

#[cfg(feature = "storage-mongodb")]
pub use mongodb::MongoDBProvider;

/// 存储类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StorageType {
    MongoDB,
    MySQL,
    PostgreSQL,
    SQLite,
}

/// 存储提供商 trait
#[async_trait]
pub trait StorageProvider: Send + Sync {
    /// 获取存储类型
    fn storage_type(&self) -> StorageType;

    /// 存储报告
    async fn store_report(&mut self, report: &CodeReviewReport) -> Result<String>;

    /// 检索报告
    async fn retrieve_report(&self, report_id: &str) -> Result<Option<CodeReviewReport>>;

    /// 列出报告
    async fn list_reports(&self, filter: &ReportFilter) -> Result<Vec<ReportSummary>>;

    /// 查询报告（支持聚合和高级过滤）
    async fn query_reports(&self, filter: &ReportFilter) -> Result<ReportQueryResult> {
        // 默认实现，调用 list_reports 并包装结果
        let reports = self.list_reports(filter).await?;
        let total_count = reports.len();

        Ok(ReportQueryResult {
            reports,
            total_count,
            aggregations: None, // 基础实现不提供聚合
        })
    }

    /// 删除报告
    async fn delete_report(&mut self, report_id: &str) -> Result<()>;

    /// 检查存储是否可用
    fn is_available(&self) -> bool;

    /// 健康检查
    async fn health_check(&self) -> Result<bool>;

    /// 获取存储统计信息
    async fn get_storage_stats(&self) -> Result<StorageStats>;
}

/// 存储统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageStats {
    pub total_reports: usize,
    pub storage_size_bytes: u64,
    pub oldest_report: Option<chrono::DateTime<chrono::Utc>>,
    pub newest_report: Option<chrono::DateTime<chrono::Utc>>,
    pub average_score: Option<f32>,
    pub provider_specific: HashMap<String, serde_json::Value>,
}

/// 存储配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub enabled: bool,
    pub provider: StorageType,
    pub connection_string: String,
    pub database_name: String,
    pub collection_name: Option<String>, // For MongoDB
    pub table_name: Option<String>,      // For SQL databases
    pub connection_pool_size: Option<usize>,
    pub connection_timeout_seconds: Option<u64>,
    pub retry_attempts: Option<usize>,
    pub backup_enabled: bool,
    pub backup_interval_hours: Option<u64>,
    pub encryption_enabled: bool,
    pub compression_enabled: bool,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            provider: StorageType::SQLite,
            connection_string: "sqlite://./reports.db".to_string(),
            database_name: "ai_commit_reports".to_string(),
            collection_name: Some("reports".to_string()),
            table_name: Some("reports".to_string()),
            connection_pool_size: Some(10),
            connection_timeout_seconds: Some(30),
            retry_attempts: Some(3),
            backup_enabled: false,
            backup_interval_hours: Some(24),
            encryption_enabled: false,
            compression_enabled: true,
        }
    }
}