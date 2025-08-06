use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use std::time::Duration;
use lru::LruCache;
use std::num::NonZeroUsize;

use crate::cache::manager::CacheEntry;

/// Memory cache configuration
#[derive(Debug, Clone)]
pub struct MemoryCacheConfig {
    pub max_entries: usize,
    pub max_memory_bytes: usize,
    pub default_ttl: Duration,
    pub enable_compression: bool,
    pub compression_threshold: usize,
}

impl Default for MemoryCacheConfig {
    fn default() -> Self {
        Self {
            max_entries: 1000,
            max_memory_bytes: 100 * 1024 * 1024, // 100MB
            default_ttl: Duration::from_secs(3600), // 1 hour
            enable_compression: true,
            compression_threshold: 1024, // 1KB
        }
    }
}

/// Memory cache statistics
#[derive(Debug, Clone, Default)]
pub struct MemoryCacheStats {
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub memory_usage: usize,
    pub entry_count: usize,
    pub compression_ratio: f64,
    pub avg_access_time_ns: u64,
}

impl MemoryCacheStats {
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }

    pub fn memory_usage_mb(&self) -> f64 {
        self.memory_usage as f64 / (1024.0 * 1024.0)
    }

    pub fn avg_entry_size(&self) -> usize {
        if self.entry_count == 0 {
            0
        } else {
            self.memory_usage / self.entry_count
        }
    }
}

/// High-performance LRU memory cache
pub struct MemoryCache {
    cache: Arc<Mutex<LruCache<String, CacheEntry>>>,
    stats: Arc<Mutex<MemoryCacheStats>>,
    config: MemoryCacheConfig,
    access_times: Arc<Mutex<HashMap<String, Vec<u64>>>>, // For performance tracking
}

impl MemoryCache {
    pub fn new(config: MemoryCacheConfig) -> Self {
        let cache = Arc::new(Mutex::new(
            LruCache::new(NonZeroUsize::new(config.max_entries).unwrap())
        ));

        Self {
            cache,
            stats: Arc::new(Mutex::new(MemoryCacheStats::default())),
            config,
            access_times: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Get data from memory cache
    pub async fn get<T>(&self, key: &str) -> Option<T>
    where
        T: Clone + for<'de> serde::Deserialize<'de>,
    {
        let start_time = std::time::Instant::now();

        let result = {
            let mut cache = self.cache.lock().await;

            if let Some(entry) = cache.get_mut(key) {
                if entry.is_expired() {
                    // Remove expired entry
                    cache.pop(key);
                    None
                } else {
                    // Update access info
                    entry.touch();

                    // Deserialize data
                    match self.deserialize_data::<T>(&entry.data) {
                        Ok(data) => Some(data),
                        Err(_) => {
                            // Remove corrupted entry
                            cache.pop(key);
                            None
                        }
                    }
                }
            } else {
                None
            }
        };

        let access_time = start_time.elapsed().as_nanos() as u64;

        // Update statistics
        {
            let mut stats = self.stats.lock().await;
            if result.is_some() {
                stats.hits += 1;
            } else {
                stats.misses += 1;
            }

            // Update average access time
            let total_accesses = stats.hits + stats.misses;
            stats.avg_access_time_ns = (stats.avg_access_time_ns * (total_accesses - 1) + access_time) / total_accesses;
        }

        // Track access times for performance analysis
        {
            let mut access_times = self.access_times.lock().await;
            access_times.entry(key.to_string())
                .or_insert_with(Vec::new)
                .push(access_time);

            // Keep only recent access times (last 100)
            if let Some(times) = access_times.get_mut(key) {
                if times.len() > 100 {
                    times.drain(0..times.len() - 100);
                }
            }
        }

        result
    }

    /// Set data in memory cache
    pub async fn set<T>(&self, key: &str, data: T, ttl: Option<Duration>) -> anyhow::Result<()>
    where
        T: Clone + serde::Serialize,
    {
        let serialized = self.serialize_data(&data)?;
        let compressed = if self.config.enable_compression && serialized.len() >= self.config.compression_threshold {
            self.compress_data(&serialized)?
        } else {
            serialized.clone()
        };

        // Calculate compression ratio before moving compressed data
        let compression_ratio = if self.config.enable_compression && serialized.len() >= self.config.compression_threshold {
            Some(compressed.len() as f64 / serialized.len() as f64)
        } else {
            None
        };

        let entry = CacheEntry::new(compressed, ttl.or(Some(self.config.default_ttl)));

        // Check if we need to evict entries before inserting
        if self.should_evict(&entry).await {
            self.evict_entries().await;
        }

        {
            let mut cache = self.cache.lock().await;
            cache.put(key.to_string(), entry);
        }

        // Update compression ratio statistics
        if let Some(ratio) = compression_ratio {
            let mut stats = self.stats.lock().await;
            stats.compression_ratio = (stats.compression_ratio + ratio) / 2.0;
        }

        Ok(())
    }

    /// Remove data from memory cache
    pub async fn remove(&self, key: &str) -> bool {
        let mut cache = self.cache.lock().await;
        cache.pop(key).is_some()
    }

    /// Check if key exists in cache
    pub async fn contains_key(&self, key: &str) -> bool {
        let cache = self.cache.lock().await;
        if let Some(entry) = cache.peek(key) {
            !entry.is_expired()
        } else {
            false
        }
    }

    /// Get all keys in cache
    pub async fn keys(&self) -> Vec<String> {
        let cache = self.cache.lock().await;
        cache.iter()
            .filter(|(_, entry)| !entry.is_expired())
            .map(|(key, _)| key.clone())
            .collect()
    }

    /// Clear all cache data
    pub async fn clear(&self) {
        let mut cache = self.cache.lock().await;
        cache.clear();

        let mut stats = self.stats.lock().await;
        *stats = MemoryCacheStats::default();

        let mut access_times = self.access_times.lock().await;
        access_times.clear();
    }

    /// Get cache statistics
    pub async fn stats(&self) -> MemoryCacheStats {
        let mut stats = self.stats.lock().await.clone();

        // Update current memory usage and entry count
        let cache = self.cache.lock().await;
        stats.entry_count = cache.len();
        stats.memory_usage = cache.iter()
            .map(|(_, entry)| entry.size)
            .sum();

        stats
    }

    /// Get detailed performance metrics
    pub async fn performance_metrics(&self) -> PerformanceMetrics {
        let stats = self.stats().await;
        let access_times = self.access_times.lock().await;

        let mut all_times = Vec::new();
        for times in access_times.values() {
            all_times.extend(times);
        }

        all_times.sort_unstable();

        let p50 = if all_times.is_empty() { 0 } else { all_times[all_times.len() / 2] };
        let p95 = if all_times.is_empty() { 0 } else { all_times[all_times.len() * 95 / 100] };
        let p99 = if all_times.is_empty() { 0 } else { all_times[all_times.len() * 99 / 100] };

        PerformanceMetrics {
            hit_rate: stats.hit_rate(),
            memory_usage_mb: stats.memory_usage_mb(),
            avg_entry_size: stats.avg_entry_size(),
            compression_ratio: stats.compression_ratio,
            access_time_p50_ns: p50,
            access_time_p95_ns: p95,
            access_time_p99_ns: p99,
            eviction_rate: if stats.hits + stats.misses == 0 { 0.0 } else {
                stats.evictions as f64 / (stats.hits + stats.misses) as f64
            },
        }
    }

    /// Cleanup expired entries
    pub async fn cleanup_expired(&self) -> usize {
        let mut cache = self.cache.lock().await;
        let mut removed_count = 0;

        let expired_keys: Vec<String> = cache.iter()
            .filter(|(_, entry)| entry.is_expired())
            .map(|(key, _)| key.clone())
            .collect();

        for key in expired_keys {
            cache.pop(&key);
            removed_count += 1;
        }

        removed_count
    }

    /// Warm up cache with frequently accessed keys
    pub async fn warmup<T>(&self, data: Vec<(String, T)>) -> anyhow::Result<usize>
    where
        T: Clone + serde::Serialize,
    {
        let mut warmed_count = 0;

        for (key, value) in data {
            if !self.contains_key(&key).await {
                self.set(&key, value, None).await?;
                warmed_count += 1;
            }
        }

        Ok(warmed_count)
    }

    // Private helper methods

    async fn should_evict(&self, new_entry: &CacheEntry) -> bool {
        let cache = self.cache.lock().await;
        let current_usage: usize = cache.iter()
            .map(|(_, entry)| entry.size)
            .sum();

        current_usage + new_entry.size > self.config.max_memory_bytes
    }

    async fn evict_entries(&self) {
        let mut cache = self.cache.lock().await;
        let mut stats = self.stats.lock().await;

        // Evict least recently used entries until we're under the limit
        while !cache.is_empty() {
            let current_usage: usize = cache.iter()
                .map(|(_, entry)| entry.size)
                .sum();

            if current_usage <= self.config.max_memory_bytes * 8 / 10 { // 80% of limit
                break;
            }

            cache.pop_lru();
            stats.evictions += 1;
        }
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

    fn compress_data(&self, data: &[u8]) -> anyhow::Result<Vec<u8>> {
        use flate2::write::GzEncoder;
        use flate2::Compression;
        use std::io::Write;

        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(data)?;
        Ok(encoder.finish()?)
    }
}

/// Detailed performance metrics
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub hit_rate: f64,
    pub memory_usage_mb: f64,
    pub avg_entry_size: usize,
    pub compression_ratio: f64,
    pub access_time_p50_ns: u64,
    pub access_time_p95_ns: u64,
    pub access_time_p99_ns: u64,
    pub eviction_rate: f64,
}

impl PerformanceMetrics {
    pub fn access_time_p50_ms(&self) -> f64 {
        self.access_time_p50_ns as f64 / 1_000_000.0
    }

    pub fn access_time_p95_ms(&self) -> f64 {
        self.access_time_p95_ns as f64 / 1_000_000.0
    }

    pub fn access_time_p99_ms(&self) -> f64 {
        self.access_time_p99_ns as f64 / 1_000_000.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration as TokioDuration};

    #[tokio::test]
    async fn test_memory_cache_basic_operations() {
        let config = MemoryCacheConfig::default();
        let cache = MemoryCache::new(config);

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
    async fn test_memory_cache_expiration() {
        let config = MemoryCacheConfig::default();
        let cache = MemoryCache::new(config);

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
    async fn test_memory_cache_lru_eviction() {
        let config = MemoryCacheConfig {
            max_entries: 2,
            ..Default::default()
        };
        let cache = MemoryCache::new(config);

        // Fill cache to capacity
        cache.set("key1", "value1".to_string(), None).await.unwrap();
        cache.set("key2", "value2".to_string(), None).await.unwrap();

        // Access key1 to make it more recently used
        let _: Option<String> = cache.get("key1").await;

        // Add third item, should evict key2 (least recently used)
        cache.set("key3", "value3".to_string(), None).await.unwrap();

        // key1 and key3 should exist, key2 should be evicted
        assert!(cache.contains_key("key1").await);
        assert!(!cache.contains_key("key2").await);
        assert!(cache.contains_key("key3").await);
    }

    #[tokio::test]
    async fn test_memory_cache_stats() {
        let config = MemoryCacheConfig::default();
        let cache = MemoryCache::new(config);

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

    #[tokio::test]
    async fn test_memory_cache_compression() {
        let config = MemoryCacheConfig {
            enable_compression: true,
            compression_threshold: 10, // Very low threshold for testing
            ..Default::default()
        };
        let cache = MemoryCache::new(config);

        // Set large data that should be compressed
        let large_data = "x".repeat(1000);
        cache.set("large_key", large_data.clone(), None).await.unwrap();

        // Should be able to retrieve the data correctly
        let result: Option<String> = cache.get("large_key").await;
        assert_eq!(result, Some(large_data));

        // Check that compression ratio is recorded
        let metrics = cache.performance_metrics().await;
        assert!(metrics.compression_ratio > 0.0);
        assert!(metrics.compression_ratio < 1.0); // Should be compressed
    }

    #[tokio::test]
    async fn test_memory_cache_warmup() {
        let config = MemoryCacheConfig::default();
        let cache = MemoryCache::new(config);

        let warmup_data = vec![
            ("key1".to_string(), "value1".to_string()),
            ("key2".to_string(), "value2".to_string()),
            ("key3".to_string(), "value3".to_string()),
        ];

        let warmed_count = cache.warmup(warmup_data).await.unwrap();
        assert_eq!(warmed_count, 3);

        // All keys should be available
        let result: Option<String> = cache.get("key1").await;
        assert_eq!(result, Some("value1".to_string()));

        let result: Option<String> = cache.get("key2").await;
        assert_eq!(result, Some("value2".to_string()));

        let result: Option<String> = cache.get("key3").await;
        assert_eq!(result, Some("value3".to_string()));
    }

    #[tokio::test]
    async fn test_memory_cache_cleanup() {
        let config = MemoryCacheConfig::default();
        let cache = MemoryCache::new(config);

        // Set some data with short TTL
        cache.set("key1", "value1".to_string(), Some(Duration::from_millis(50))).await.unwrap();
        cache.set("key2", "value2".to_string(), None).await.unwrap();

        // Wait for expiration
        sleep(TokioDuration::from_millis(100)).await;

        // Run cleanup
        let removed_count = cache.cleanup_expired().await;
        assert_eq!(removed_count, 1);

        // Verify only non-expired data remains
        let result: Option<String> = cache.get("key1").await;
        assert_eq!(result, None);

        let result: Option<String> = cache.get("key2").await;
        assert_eq!(result, Some("value2".to_string()));
    }
}