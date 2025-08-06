use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use std::time::Duration;
use lru::LruCache;
use std::num::NonZeroUsize;

use crate::cache::storage::FsCacheManager;
use crate::cache::strategy::CacheStrategy;
use crate::cache::parallel::{ParallelProcessor, ParallelConfig, TaskResult};
use crate::cache::memory_manager::{MemoryManager, MemoryConfig, AllocationCategory, MemoryPressure};

/// Cache entry with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    pub data: Vec<u8>,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub access_count: u64,
    pub last_accessed: DateTime<Utc>,
    pub size: usize,
}

impl CacheEntry {
    pub fn new(data: Vec<u8>, ttl: Option<Duration>) -> Self {
        let now = Utc::now();
        let expires_at = ttl.map(|duration| {
            now + chrono::Duration::from_std(duration).unwrap()
        });

        Self {
            size: data.len(),
            data,
            created_at: now,
            expires_at,
            access_count: 0,
            last_accessed: now,
        }
    }

    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            Utc::now() > expires_at
        } else {
            false
        }
    }

    pub fn touch(&mut self) {
        self.access_count += 1;
        self.last_accessed = Utc::now();
    }
}

/// Cache statistics for monitoring
#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub memory_usage: usize,
    pub entry_count: usize,
}

impl CacheStats {
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }
}

/// Multi-level cache manager with memory management
pub struct CacheManager {
    memory_cache: Arc<Mutex<LruCache<String, CacheEntry>>>,
    fs_cache: Option<Arc<Mutex<FsCacheManager>>>,
    stats: Arc<Mutex<CacheStats>>,
    config: CacheConfig,
    strategy: CacheStrategy,
    parallel_processor: Option<Arc<ParallelProcessor>>,
    memory_manager: Arc<MemoryManager>,
}

#[derive(Debug, Clone)]
pub struct CacheConfig {
    pub memory_cache_size: usize,
    pub enable_fs_cache: bool,
    pub fs_cache_dir: String,
    pub default_ttl: Duration,
    pub max_memory_usage: usize, // in bytes
    pub cleanup_interval: Duration,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            memory_cache_size: 1000,
            enable_fs_cache: true,
            fs_cache_dir: ".cache".to_string(),
            default_ttl: Duration::from_secs(3600), // 1 hour
            max_memory_usage: 100 * 1024 * 1024, // 100MB
            cleanup_interval: Duration::from_secs(300), // 5 minutes
        }
    }
}

impl CacheManager {
    pub async fn new(config: CacheConfig) -> anyhow::Result<Self> {
        let memory_cache = Arc::new(Mutex::new(
            LruCache::new(NonZeroUsize::new(config.memory_cache_size).unwrap())
        ));

        let fs_cache = if config.enable_fs_cache {
            Some(Arc::new(Mutex::new(FsCacheManager::new(&config.fs_cache_dir).await?)))
        } else {
            None
        };

        let stats = Arc::new(Mutex::new(CacheStats::default()));
        let strategy = CacheStrategy::new(&config);

        // Create memory manager with cache-specific configuration
        let memory_config = MemoryConfig {
            max_memory_usage: config.max_memory_usage,
            cleanup_threshold_percent: 0.8,
            cleanup_interval: config.cleanup_interval,
            track_allocations: true,
            large_file_threshold: 10 * 1024 * 1024, // 10MB
            stream_buffer_size: 64 * 1024, // 64KB
        };
        let memory_manager = Arc::new(MemoryManager::new(memory_config));

        let manager = Self {
            memory_cache,
            fs_cache,
            stats,
            config,
            strategy,
            parallel_processor: None,
            memory_manager,
        };

        // Start background cleanup task
        manager.start_cleanup_task().await;

        Ok(manager)
    }

    /// Create cache manager with parallel processing enabled
    pub async fn new_with_parallel(config: CacheConfig, parallel_config: ParallelConfig) -> anyhow::Result<Self> {
        let memory_cache = Arc::new(Mutex::new(
            LruCache::new(NonZeroUsize::new(config.memory_cache_size).unwrap())
        ));

        let fs_cache = if config.enable_fs_cache {
            Some(Arc::new(Mutex::new(FsCacheManager::new(&config.fs_cache_dir).await?)))
        } else {
            None
        };

        let stats = Arc::new(Mutex::new(CacheStats::default()));
        let strategy = CacheStrategy::new(&config);
        let parallel_processor = Some(Arc::new(ParallelProcessor::new(parallel_config)));

        // Create memory manager with cache-specific configuration
        let memory_config = MemoryConfig {
            max_memory_usage: config.max_memory_usage,
            cleanup_threshold_percent: 0.8,
            cleanup_interval: config.cleanup_interval,
            track_allocations: true,
            large_file_threshold: 10 * 1024 * 1024, // 10MB
            stream_buffer_size: 64 * 1024, // 64KB
        };
        let memory_manager = Arc::new(MemoryManager::new(memory_config));

        let manager = Self {
            memory_cache,
            fs_cache,
            stats,
            config,
            strategy,
            parallel_processor,
            memory_manager,
        };

        // Start background cleanup task
        manager.start_cleanup_task().await;

        Ok(manager)
    }

    /// Get data from cache
    pub async fn get<T>(&self, key: &str) -> Option<T>
    where
        T: Clone + for<'de> serde::Deserialize<'de> + serde::Serialize,
    {
        // Try memory cache first
        if let Some(entry) = self.get_from_memory(key).await {
            if !entry.is_expired() {
                if let Ok(data) = self.deserialize_data::<T>(&entry.data) {
                    self.record_hit().await;
                    return Some(data);
                }
            } else {
                // Remove expired entry
                self.remove_from_memory(key).await;
            }
        }

        // Try filesystem cache
        if let Some(ref fs_cache) = self.fs_cache {
            let mut fs_cache_guard = fs_cache.lock().await;
            if let Some(data) = fs_cache_guard.get::<T>(key).await {
                // Promote to memory cache
                if let Ok(serialized) = self.serialize_data(&data) {
                    self.set_memory_cache(key, serialized, None).await;
                }
                self.record_hit().await;
                return Some(data);
            }
        }

        self.record_miss().await;
        None
    }

    /// Set data in cache
    pub async fn set<T>(&self, key: &str, data: T, ttl: Option<Duration>) -> anyhow::Result<()>
    where
        T: Clone + serde::Serialize,
    {
        let serialized = self.serialize_data(&data)?;
        let effective_ttl = ttl.or(Some(self.config.default_ttl));

        // Set in memory cache
        self.set_memory_cache(key, serialized.clone(), effective_ttl).await;

        // Set in filesystem cache if enabled
        if let Some(ref fs_cache) = self.fs_cache {
            let mut fs_cache_guard = fs_cache.lock().await;
            fs_cache_guard.set(key, data, effective_ttl).await?;
        }

        Ok(())
    }

    /// Remove data from cache
    pub async fn remove(&self, key: &str) -> bool {
        let memory_removed = self.remove_from_memory(key).await;

        let fs_removed = if let Some(ref fs_cache) = self.fs_cache {
            let mut fs_cache_guard = fs_cache.lock().await;
            fs_cache_guard.remove(key).await
        } else {
            false
        };

        memory_removed || fs_removed
    }

    /// Clear all cache data
    pub async fn clear(&self) -> anyhow::Result<()> {
        // Clear memory cache
        {
            let mut cache = self.memory_cache.lock().await;
            cache.clear();
        }

        // Clear filesystem cache
        if let Some(ref fs_cache) = self.fs_cache {
            let mut fs_cache_guard = fs_cache.lock().await;
            fs_cache_guard.clear().await?;
        }

        // Reset stats
        {
            let mut stats = self.stats.lock().await;
            *stats = CacheStats::default();
        }

        Ok(())
    }

    /// Get cache statistics
    pub async fn stats(&self) -> CacheStats {
        let mut stats = self.stats.lock().await.clone();

        // Update current memory usage and entry count
        let cache = self.memory_cache.lock().await;
        stats.entry_count = cache.len();
        stats.memory_usage = cache.iter()
            .map(|(_, entry)| entry.size)
            .sum();

        stats
    }

    /// Check if cache contains key
    pub async fn contains_key(&self, key: &str) -> bool {
        // Check memory cache
        {
            let cache = self.memory_cache.lock().await;
            if let Some(entry) = cache.peek(key) {
                if !entry.is_expired() {
                    return true;
                }
            }
        }

        // Check filesystem cache
        if let Some(ref fs_cache) = self.fs_cache {
            let fs_cache_guard = fs_cache.lock().await;
            return fs_cache_guard.contains_key(key).await;
        }

        false
    }

    /// Get cache keys (memory cache only for performance)
    pub async fn keys(&self) -> Vec<String> {
        let cache = self.memory_cache.lock().await;
        cache.iter()
            .filter(|(_, entry)| !entry.is_expired())
            .map(|(key, _)| key.clone())
            .collect()
    }



    /// Get parallel processor statistics
    pub async fn get_parallel_stats(&self) -> Option<crate::cache::parallel::ParallelStats> {
        if let Some(ref processor) = self.parallel_processor {
            Some(processor.get_stats().await)
        } else {
            None
        }
    }

    /// Check if parallel processing is enabled
    pub fn is_parallel_enabled(&self) -> bool {
        self.parallel_processor.is_some()
    }

    /// Get memory manager for advanced memory operations
    pub fn memory_manager(&self) -> Arc<MemoryManager> {
        Arc::clone(&self.memory_manager)
    }

    /// Get current memory usage from memory manager
    pub fn get_memory_usage(&self) -> usize {
        self.memory_manager.get_current_usage()
    }

    /// Get memory usage percentage
    pub fn get_memory_usage_percentage(&self) -> f32 {
        self.memory_manager.get_usage_percentage()
    }

    /// Get memory pressure level
    pub fn get_memory_pressure(&self) -> MemoryPressure {
        self.memory_manager.get_memory_pressure()
    }

    /// Check if a file should be streamed based on size
    pub fn should_stream_file(&self, file_size: usize) -> bool {
        self.memory_manager.should_stream_file(file_size)
    }

    /// Create a streaming file reader for large files
    pub async fn stream_read_file(&self, file_path: &str) -> anyhow::Result<crate::cache::memory_manager::StreamingFileReader> {
        self.memory_manager.stream_read_file(file_path).await
    }

    /// Create a managed buffer for large data processing
    pub async fn create_managed_buffer(&self, size: usize, category: AllocationCategory) -> anyhow::Result<crate::cache::memory_manager::ManagedBuffer> {
        self.memory_manager.create_managed_buffer(size, category).await
    }

    /// Force memory cleanup
    pub async fn force_memory_cleanup(&self) -> anyhow::Result<usize> {
        self.memory_manager.force_cleanup().await
    }

    /// Get detailed memory statistics
    pub async fn get_memory_stats(&self) -> crate::cache::memory_manager::MemoryStats {
        self.memory_manager.get_stats().await
    }

    // Private helper methods

    async fn get_from_memory(&self, key: &str) -> Option<CacheEntry> {
        let mut cache = self.memory_cache.lock().await;
        if let Some(entry) = cache.get_mut(key) {
            entry.touch();
            Some(entry.clone())
        } else {
            None
        }
    }

    async fn set_memory_cache(&self, key: &str, data: Vec<u8>, ttl: Option<Duration>) {
        let entry = CacheEntry::new(data, ttl);

        // Register allocation with memory manager
        let allocation_id = format!("cache_entry_{}", key);
        if let Err(e) = self.memory_manager.allocate(
            allocation_id.clone(),
            entry.size,
            AllocationCategory::Cache
        ).await {
            // If allocation fails due to memory pressure, try cleanup and retry
            if self.memory_manager.get_memory_pressure() == MemoryPressure::Critical {
                let _ = self.memory_manager.force_cleanup().await;
                let _ = self.evict_entries().await;

                // Retry allocation after cleanup
                if let Err(_) = self.memory_manager.allocate(
                    allocation_id,
                    entry.size,
                    AllocationCategory::Cache
                ).await {
                    log::warn!("Failed to allocate memory for cache entry: {}", e);
                    return;
                }
            } else {
                log::warn!("Failed to allocate memory for cache entry: {}", e);
                return;
            }
        }

        let mut cache = self.memory_cache.lock().await;

        // Check memory limits before inserting
        if self.should_evict(&entry).await {
            self.evict_entries().await;
        }

        cache.put(key.to_string(), entry);
    }

    async fn remove_from_memory(&self, key: &str) -> bool {
        let mut cache = self.memory_cache.lock().await;
        let removed = cache.pop(key).is_some();

        if removed {
            // Deallocate from memory manager
            let allocation_id = format!("cache_entry_{}", key);
            let _ = self.memory_manager.deallocate(&allocation_id).await;
        }

        removed
    }

    async fn should_evict(&self, new_entry: &CacheEntry) -> bool {
        let cache = self.memory_cache.lock().await;
        let current_usage: usize = cache.iter()
            .map(|(_, entry)| entry.size)
            .sum();

        current_usage + new_entry.size > self.config.max_memory_usage
    }

    async fn evict_entries(&self) {
        let mut cache = self.memory_cache.lock().await;
        let mut stats = self.stats.lock().await;

        // Evict least recently used entries until we're under the limit
        while !cache.is_empty() {
            let current_usage: usize = cache.iter()
                .map(|(_, entry)| entry.size)
                .sum();

            if current_usage <= self.config.max_memory_usage * 8 / 10 { // 80% of limit
                break;
            }

            if let Some((key, _)) = cache.pop_lru() {
                // Deallocate from memory manager
                let allocation_id = format!("cache_entry_{}", key);
                let _ = self.memory_manager.deallocate(&allocation_id).await;
                stats.evictions += 1;
            }
        }
    }

    async fn record_hit(&self) {
        let mut stats = self.stats.lock().await;
        stats.hits += 1;
    }

    async fn record_miss(&self) {
        let mut stats = self.stats.lock().await;
        stats.misses += 1;
    }

    fn serialize_data<T>(&self, data: &T) -> anyhow::Result<Vec<u8>>
    where
        T: serde::Serialize,
    {
        bincode::serialize(data).map_err(|e| anyhow::anyhow!("Serialization error: {}", e))
    }

    fn deserialize_data<T>(&self, data: &[u8]) -> anyhow::Result<T>
    where
        T: for<'de> serde::Deserialize<'de>,
    {
        bincode::deserialize(data).map_err(|e| anyhow::anyhow!("Deserialization error: {}", e))
    }

    async fn start_cleanup_task(&self) {
        let memory_cache = Arc::clone(&self.memory_cache);
        let cleanup_interval = self.config.cleanup_interval;

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(cleanup_interval);

            loop {
                interval.tick().await;

                // Clean up expired entries
                let mut cache = memory_cache.lock().await;
                let expired_keys: Vec<String> = cache.iter()
                    .filter(|(_, entry)| entry.is_expired())
                    .map(|(key, _)| key.clone())
                    .collect();

                for key in expired_keys {
                    cache.pop(&key);
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration as TokioDuration};

    #[tokio::test]
    async fn test_cache_basic_operations() {
        let config = CacheConfig {
            enable_fs_cache: false,
            ..Default::default()
        };

        let cache = CacheManager::new(config).await.unwrap();

        // Test set and get
        cache.set("key1", "value1".to_string(), None).await.unwrap();
        let result: Option<String> = cache.get("key1").await;
        assert_eq!(result, Some("value1".to_string()));

        // Test miss
        let result: Option<String> = cache.get("nonexistent").await;
        assert_eq!(result, None);

        // Test contains_key
        assert!(cache.contains_key("key1").await);
        assert!(!cache.contains_key("nonexistent").await);

        // Test remove
        assert!(cache.remove("key1").await);
        let result: Option<String> = cache.get("key1").await;
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn test_cache_expiration() {
        let config = CacheConfig {
            enable_fs_cache: false,
            ..Default::default()
        };

        let cache = CacheManager::new(config).await.unwrap();

        // Set with short TTL
        cache.set("key1", "value1".to_string(), Some(Duration::from_millis(100))).await.unwrap();

        // Should be available immediately
        let result: Option<String> = cache.get("key1").await;
        assert_eq!(result, Some("value1".to_string()));

        // Wait for expiration
        sleep(TokioDuration::from_millis(150)).await;

        // Should be expired
        let result: Option<String> = cache.get("key1").await;
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn test_cache_stats() {
        let config = CacheConfig {
            enable_fs_cache: false,
            ..Default::default()
        };

        let cache = CacheManager::new(config).await.unwrap();

        // Initial stats
        let stats = cache.stats().await;
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 0);

        // Set and get (hit)
        cache.set("key1", "value1".to_string(), None).await.unwrap();
        let _: Option<String> = cache.get("key1").await;

        // Miss
        let _: Option<String> = cache.get("nonexistent").await;

        let stats = cache.stats().await;
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.hit_rate(), 0.5);
    }
}