use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use crate::models::review::CodeReviewReport;
use super::models::{ReportFilter, ReportSummary, ReportQueryResult};
use super::providers::{StorageProvider, StorageType, StorageConfig, StorageStats};

/// 存储管理器
pub struct StorageManager {
    providers: HashMap<StorageType, Arc<RwLock<Box<dyn StorageProvider>>>>,
    config: StorageConfig,
    active_provider: Option<StorageType>,
    connection_pool: Option<Arc<ConnectionPool>>,
}

/// 连接池管理
pub struct ConnectionPool {
    max_connections: usize,
    active_connections: Arc<RwLock<usize>>,
    connection_timeout: std::time::Duration,
}

impl StorageManager {
    /// 创建新的存储管理器
    pub fn new(config: StorageConfig) -> Self {
        let connection_pool = if config.connection_pool_size.is_some() {
            Some(Arc::new(ConnectionPool::new(
                config.connection_pool_size.unwrap_or(10),
                std::time::Duration::from_secs(config.connection_timeout_seconds.unwrap_or(30)),
            )))
        } else {
            None
        };

        Self {
            providers: HashMap::new(),
            active_provider: if config.enabled { Some(config.provider.clone()) } else { None },
            config,
            connection_pool,
        }
    }

    /// 注册存储提供商
    pub fn register_provider(&mut self, provider: Box<dyn StorageProvider>) -> Result<()> {
        let storage_type = provider.storage_type();

        if !provider.is_available() {
            warn!("Storage provider {:?} is not available", storage_type);
            return Err(anyhow!("Storage provider {:?} is not available", storage_type));
        }

        info!("Registering storage provider: {:?}", storage_type);
        self.providers.insert(storage_type, Arc::new(RwLock::new(provider)));

        Ok(())
    }

    /// 获取活跃的存储提供商
    async fn get_active_provider(&self) -> Result<Arc<RwLock<Box<dyn StorageProvider>>>> {
        let provider_type = self.active_provider
            .as_ref()
            .ok_or_else(|| anyhow!("No active storage provider configured"))?;

        self.providers
            .get(provider_type)
            .cloned()
            .ok_or_else(|| anyhow!("Storage provider {:?} not found", provider_type))
    }

    /// 存储报告
    pub async fn store_report(&self, report: &CodeReviewReport) -> Result<String> {
        if !self.config.enabled {
            return Err(anyhow!("Storage is disabled"));
        }

        let provider = self.get_active_provider().await?;
        let mut provider_guard = provider.write().await;

        debug!("Storing report for project: {}", report.summary.project_path);

        match provider_guard.store_report(report).await {
            Ok(report_id) => {
                info!("Successfully stored report with ID: {}", report_id);
                Ok(report_id)
            }
            Err(e) => {
                error!("Failed to store report: {}", e);
                Err(e)
            }
        }
    }

    /// 检索报告
    pub async fn retrieve_report(&self, report_id: &str) -> Result<Option<CodeReviewReport>> {
        if !self.config.enabled {
            return Err(anyhow!("Storage is disabled"));
        }

        let provider = self.get_active_provider().await?;
        let provider_guard = provider.read().await;

        debug!("Retrieving report with ID: {}", report_id);

        match provider_guard.retrieve_report(report_id).await {
            Ok(report) => {
                if report.is_some() {
                    info!("Successfully retrieved report with ID: {}", report_id);
                } else {
                    warn!("Report with ID {} not found", report_id);
                }
                Ok(report)
            }
            Err(e) => {
                error!("Failed to retrieve report {}: {}", report_id, e);
                Err(e)
            }
        }
    }

    /// 列出报告
    pub async fn list_reports(&self, filter: &ReportFilter) -> Result<Vec<ReportSummary>> {
        if !self.config.enabled {
            return Err(anyhow!("Storage is disabled"));
        }

        let provider = self.get_active_provider().await?;
        let provider_guard = provider.read().await;

        debug!("Listing reports with filter: {:?}", filter);

        match provider_guard.list_reports(filter).await {
            Ok(reports) => {
                info!("Successfully listed {} reports", reports.len());
                Ok(reports)
            }
            Err(e) => {
                error!("Failed to list reports: {}", e);
                Err(e)
            }
        }
    }

    /// 查询报告（支持聚合和高级过滤）
    pub async fn query_reports(&self, filter: &ReportFilter) -> Result<ReportQueryResult> {
        if !self.config.enabled {
            return Err(anyhow!("Storage is disabled"));
        }

        let provider = self.get_active_provider().await?;
        let provider_guard = provider.read().await;

        debug!("Querying reports with filter: {:?}", filter);

        match provider_guard.query_reports(filter).await {
            Ok(result) => {
                info!("Successfully queried {} reports with {} total",
                      result.reports.len(), result.total_count);
                Ok(result)
            }
            Err(e) => {
                error!("Failed to query reports: {}", e);
                Err(e)
            }
        }
    }

    /// 删除报告
    pub async fn delete_report(&self, report_id: &str) -> Result<()> {
        if !self.config.enabled {
            return Err(anyhow!("Storage is disabled"));
        }

        let provider = self.get_active_provider().await?;
        let mut provider_guard = provider.write().await;

        debug!("Deleting report with ID: {}", report_id);

        match provider_guard.delete_report(report_id).await {
            Ok(()) => {
                info!("Successfully deleted report with ID: {}", report_id);
                Ok(())
            }
            Err(e) => {
                error!("Failed to delete report {}: {}", report_id, e);
                Err(e)
            }
        }
    }

    /// 健康检查
    pub async fn health_check(&self) -> Result<HashMap<StorageType, bool>> {
        let mut results = HashMap::new();

        for (storage_type, provider) in &self.providers {
            let provider_guard = provider.read().await;
            match provider_guard.health_check().await {
                Ok(healthy) => {
                    results.insert(storage_type.clone(), healthy);
                }
                Err(e) => {
                    error!("Health check failed for {:?}: {}", storage_type, e);
                    results.insert(storage_type.clone(), false);
                }
            }
        }

        Ok(results)
    }

    /// 获取存储统计信息
    pub async fn get_storage_stats(&self) -> Result<StorageStats> {
        if !self.config.enabled {
            return Err(anyhow!("Storage is disabled"));
        }

        let provider = self.get_active_provider().await?;
        let provider_guard = provider.read().await;

        provider_guard.get_storage_stats().await
    }

    /// 切换存储提供商
    pub async fn switch_provider(&mut self, provider_type: StorageType) -> Result<()> {
        if !self.providers.contains_key(&provider_type) {
            return Err(anyhow!("Storage provider {:?} not registered", provider_type));
        }

        // 健康检查新提供商
        let provider = self.providers.get(&provider_type).unwrap();
        let provider_guard = provider.read().await;

        if !provider_guard.health_check().await? {
            return Err(anyhow!("Storage provider {:?} is not healthy", provider_type));
        }

        info!("Switching storage provider to: {:?}", provider_type);
        self.active_provider = Some(provider_type);
        self.config.provider = provider_type;

        Ok(())
    }

    /// 获取配置
    pub fn get_config(&self) -> &StorageConfig {
        &self.config
    }

    /// 更新配置
    pub fn update_config(&mut self, config: StorageConfig) {
        self.config = config;
        self.active_provider = if self.config.enabled {
            Some(self.config.provider.clone())
        } else {
            None
        };
    }

    /// 是否启用存储
    pub fn is_enabled(&self) -> bool {
        self.config.enabled && self.active_provider.is_some()
    }
}

impl ConnectionPool {
    pub fn new(max_connections: usize, connection_timeout: std::time::Duration) -> Self {
        Self {
            max_connections,
            active_connections: Arc::new(RwLock::new(0)),
            connection_timeout,
        }
    }

    pub async fn acquire_connection(&self) -> Result<ConnectionGuard> {
        let mut active = self.active_connections.write().await;

        if *active >= self.max_connections {
            return Err(anyhow!("Connection pool exhausted"));
        }

        *active += 1;
        Ok(ConnectionGuard {
            pool: self.active_connections.clone(),
        })
    }

    pub fn max_connections(&self) -> usize {
        self.max_connections
    }

    pub async fn active_connections(&self) -> usize {
        *self.active_connections.read().await
    }
}

/// 连接守卫，自动释放连接
pub struct ConnectionGuard {
    pool: Arc<RwLock<usize>>,
}

impl Drop for ConnectionGuard {
    fn drop(&mut self) {
        let pool = self.pool.clone();
        tokio::spawn(async move {
            let mut active = pool.write().await;
            if *active > 0 {
                *active -= 1;
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::review::{CodeReviewReport, ReviewSummary, ReviewMetadata, ReviewConfiguration};
    use chrono::Utc;
    use async_trait::async_trait;

    struct MockStorageProvider {
        storage_type: StorageType,
        available: bool,
    }

    #[async_trait]
    impl StorageProvider for MockStorageProvider {
        fn storage_type(&self) -> StorageType {
            self.storage_type.clone()
        }

        async fn store_report(&mut self, _report: &CodeReviewReport) -> Result<String> {
            Ok("mock-report-id".to_string())
        }

        async fn retrieve_report(&self, _report_id: &str) -> Result<Option<CodeReviewReport>> {
            Ok(None)
        }

        async fn list_reports(&self, _filter: &ReportFilter) -> Result<Vec<ReportSummary>> {
            Ok(vec![])
        }

        async fn delete_report(&mut self, _report_id: &str) -> Result<()> {
            Ok(())
        }

        fn is_available(&self) -> bool {
            self.available
        }

        async fn health_check(&self) -> Result<bool> {
            Ok(self.available)
        }

        async fn get_storage_stats(&self) -> Result<StorageStats> {
            Ok(StorageStats {
                total_reports: 0,
                storage_size_bytes: 0,
                oldest_report: None,
                newest_report: None,
                average_score: None,
                provider_specific: HashMap::new(),
            })
        }
    }

    #[tokio::test]
    async fn test_storage_manager_creation() {
        let config = StorageConfig::default();
        let manager = StorageManager::new(config);

        assert!(!manager.is_enabled());
    }

    #[tokio::test]
    async fn test_provider_registration() {
        let mut config = StorageConfig::default();
        config.enabled = true;
        config.provider = StorageType::SQLite;

        let mut manager = StorageManager::new(config);

        let provider = Box::new(MockStorageProvider {
            storage_type: StorageType::SQLite,
            available: true,
        });

        assert!(manager.register_provider(provider).is_ok());
        assert!(manager.is_enabled());
    }

    #[tokio::test]
    async fn test_connection_pool() {
        let pool = ConnectionPool::new(2, std::time::Duration::from_secs(30));

        assert_eq!(pool.max_connections(), 2);
        assert_eq!(pool.active_connections().await, 0);

        let _guard1 = pool.acquire_connection().await.unwrap();
        assert_eq!(pool.active_connections().await, 1);

        let _guard2 = pool.acquire_connection().await.unwrap();
        assert_eq!(pool.active_connections().await, 2);

        // Should fail when pool is exhausted
        assert!(pool.acquire_connection().await.is_err());
    }
}