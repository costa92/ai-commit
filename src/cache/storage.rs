use std::path::{Path, PathBuf};
use std::collections::HashMap;
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use std::time::Duration;
use sha2::{Sha256, Digest};

/// Filesystem cache entry metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
struct FsCacheMetadata {
    created_at: DateTime<Utc>,
    expires_at: Option<DateTime<Utc>>,
    access_count: u64,
    last_accessed: DateTime<Utc>,
    data_size: usize,
    checksum: String,
}

impl FsCacheMetadata {
    fn new(data_size: usize, checksum: String, ttl: Option<Duration>) -> Self {
        let now = Utc::now();
        let expires_at = ttl.map(|duration| {
            now + chrono::Duration::from_std(duration).unwrap()
        });

        Self {
            created_at: now,
            expires_at,
            access_count: 0,
            last_accessed: now,
            data_size,
            checksum,
        }
    }

    fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            Utc::now() > expires_at
        } else {
            false
        }
    }

    fn touch(&mut self) {
        self.access_count += 1;
        self.last_accessed = Utc::now();
    }
}

/// Filesystem-based cache manager
pub struct FsCacheManager {
    cache_dir: PathBuf,
    metadata_cache: HashMap<String, FsCacheMetadata>,
    compression_enabled: bool,
}

impl FsCacheManager {
    pub async fn new(cache_dir: &str) -> anyhow::Result<Self> {
        let cache_path = PathBuf::from(cache_dir);

        // Create cache directory if it doesn't exist
        if !cache_path.exists() {
            fs::create_dir_all(&cache_path).await?;
        }

        let mut manager = Self {
            cache_dir: cache_path,
            metadata_cache: HashMap::new(),
            compression_enabled: true,
        };

        // Load existing metadata
        manager.load_metadata().await?;

        Ok(manager)
    }

    /// Get data from filesystem cache
    pub async fn get<T>(&mut self, key: &str) -> Option<T>
    where
        T: for<'de> serde::Deserialize<'de>,
    {
        let safe_key = self.sanitize_key(key);

        // Check metadata first
        if let Some(metadata) = self.metadata_cache.get_mut(&safe_key) {
            if metadata.is_expired() {
                // Remove expired entry
                let _ = self.remove_internal(&safe_key).await;
                return None;
            }

            // Update access info
            metadata.touch();
        } else {
            return None;
        }

        // Read data from file
        let data_path = self.get_data_path(&safe_key);
        match self.read_data_file::<T>(&data_path).await {
            Ok(data) => {
                self.save_metadata().await.ok();
                Some(data)
            }
            Err(_) => {
                // File corrupted or missing, remove from metadata
                self.metadata_cache.remove(&safe_key);
                self.save_metadata().await.ok();
                None
            }
        }
    }

    /// Set data in filesystem cache
    pub async fn set<T>(&mut self, key: &str, data: T, ttl: Option<Duration>) -> anyhow::Result<()>
    where
        T: serde::Serialize,
    {
        let safe_key = self.sanitize_key(key);
        let data_path = self.get_data_path(&safe_key);

        // Serialize and write data
        let serialized_data = self.write_data_file(&data_path, &data).await?;

        // Calculate checksum
        let checksum = self.calculate_checksum(&serialized_data);

        // Update metadata
        let metadata = FsCacheMetadata::new(serialized_data.len(), checksum, ttl);
        self.metadata_cache.insert(safe_key, metadata);

        // Save metadata
        self.save_metadata().await?;

        Ok(())
    }

    /// Remove data from filesystem cache
    pub async fn remove(&mut self, key: &str) -> bool {
        let safe_key = self.sanitize_key(key);
        self.remove_internal(&safe_key).await
    }

    /// Check if key exists in cache
    pub async fn contains_key(&self, key: &str) -> bool {
        let safe_key = self.sanitize_key(key);

        if let Some(metadata) = self.metadata_cache.get(&safe_key) {
            if metadata.is_expired() {
                return false;
            }

            let data_path = self.get_data_path(&safe_key);
            data_path.exists()
        } else {
            false
        }
    }

    /// Clear all cache data
    pub async fn clear(&mut self) -> anyhow::Result<()> {
        // Remove all data files
        for key in self.metadata_cache.keys() {
            let data_path = self.get_data_path(key);
            if data_path.exists() {
                fs::remove_file(data_path).await.ok();
            }
        }

        // Clear metadata
        self.metadata_cache.clear();
        self.save_metadata().await?;

        Ok(())
    }

    /// Get cache statistics
    pub async fn stats(&self) -> FsCacheStats {
        let mut total_size = 0;
        let mut expired_count = 0;
        let mut total_access_count = 0;

        for metadata in self.metadata_cache.values() {
            total_size += metadata.data_size;
            total_access_count += metadata.access_count;

            if metadata.is_expired() {
                expired_count += 1;
            }
        }

        FsCacheStats {
            entry_count: self.metadata_cache.len(),
            total_size,
            expired_count,
            total_access_count,
        }
    }

    /// Cleanup expired entries
    pub async fn cleanup_expired(&mut self) -> anyhow::Result<usize> {
        let mut removed_count = 0;
        let expired_keys: Vec<String> = self.metadata_cache
            .iter()
            .filter(|(_, metadata)| metadata.is_expired())
            .map(|(key, _)| key.clone())
            .collect();

        for key in expired_keys {
            if self.remove_internal(&key).await {
                removed_count += 1;
            }
        }

        if removed_count > 0 {
            self.save_metadata().await?;
        }

        Ok(removed_count)
    }

    /// Perform maintenance operations
    pub async fn maintenance(&mut self) -> anyhow::Result<MaintenanceReport> {
        let mut report = MaintenanceReport::default();

        // Cleanup expired entries
        report.expired_removed = self.cleanup_expired().await?;

        // Verify data integrity
        let mut corrupted_keys = Vec::new();
        for (key, metadata) in &self.metadata_cache {
            let data_path = self.get_data_path(key);

            if !data_path.exists() {
                corrupted_keys.push(key.clone());
                continue;
            }

            // Verify checksum if possible
            if let Ok(data) = fs::read(&data_path).await {
                let checksum = self.calculate_checksum(&data);
                if checksum != metadata.checksum {
                    corrupted_keys.push(key.clone());
                }
            }
        }

        // Remove corrupted entries
        for key in corrupted_keys {
            self.remove_internal(&key).await;
            report.corrupted_removed += 1;
        }

        if report.corrupted_removed > 0 {
            self.save_metadata().await?;
        }

        Ok(report)
    }

    // Private helper methods

    async fn remove_internal(&mut self, safe_key: &str) -> bool {
        let data_path = self.get_data_path(safe_key);

        // Remove data file
        if data_path.exists() {
            fs::remove_file(data_path).await.ok();
        }

        // Remove from metadata
        self.metadata_cache.remove(safe_key).is_some()
    }

    fn sanitize_key(&self, key: &str) -> String {
        // Replace invalid filesystem characters
        key.chars()
            .map(|c| match c {
                '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
                c => c,
            })
            .collect()
    }

    fn get_data_path(&self, safe_key: &str) -> PathBuf {
        self.cache_dir.join(format!("{}.cache", safe_key))
    }

    fn get_metadata_path(&self) -> PathBuf {
        self.cache_dir.join("metadata.json")
    }

    async fn read_data_file<T>(&self, path: &Path) -> anyhow::Result<T>
    where
        T: for<'de> serde::Deserialize<'de>,
    {
        let mut file = fs::File::open(path).await?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).await?;

        // Decompress if needed
        let data = if self.compression_enabled && buffer.starts_with(b"COMP") {
            self.decompress_data(&buffer[4..])?
        } else {
            buffer
        };

        bincode::deserialize(&data).map_err(|e| anyhow::anyhow!("Deserialization error: {}", e))
    }

    async fn write_data_file<T>(&self, path: &Path, data: &T) -> anyhow::Result<Vec<u8>>
    where
        T: serde::Serialize,
    {
        let serialized = bincode::serialize(data)
            .map_err(|e| anyhow::anyhow!("Serialization error: {}", e))?;

        // Compress if beneficial
        let (final_data, compressed) = if self.compression_enabled && serialized.len() > 1024 {
            match self.compress_data(&serialized) {
                Ok(compressed) if compressed.len() < serialized.len() => {
                    let mut with_header = b"COMP".to_vec();
                    with_header.extend_from_slice(&compressed);
                    (with_header, true)
                }
                _ => (serialized.clone(), false),
            }
        } else {
            (serialized.clone(), false)
        };

        // Write to file
        let mut file = fs::File::create(path).await?;
        file.write_all(&final_data).await?;
        file.sync_all().await?;

        Ok(if compressed { serialized } else { final_data })
    }

    fn compress_data(&self, data: &[u8]) -> anyhow::Result<Vec<u8>> {
        use flate2::write::GzEncoder;
        use flate2::Compression;
        use std::io::Write;

        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(data)?;
        Ok(encoder.finish()?)
    }

    fn decompress_data(&self, data: &[u8]) -> anyhow::Result<Vec<u8>> {
        use flate2::read::GzDecoder;
        use std::io::Read;

        let mut decoder = GzDecoder::new(data);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed)?;
        Ok(decompressed)
    }

    fn calculate_checksum(&self, data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }

    async fn load_metadata(&mut self) -> anyhow::Result<()> {
        let metadata_path = self.get_metadata_path();

        if metadata_path.exists() {
            let content = fs::read_to_string(metadata_path).await?;
            self.metadata_cache = serde_json::from_str(&content)
                .unwrap_or_else(|_| HashMap::new());
        }

        Ok(())
    }

    async fn save_metadata(&self) -> anyhow::Result<()> {
        let metadata_path = self.get_metadata_path();
        let content = serde_json::to_string_pretty(&self.metadata_cache)?;
        fs::write(metadata_path, content).await?;
        Ok(())
    }
}

/// Filesystem cache statistics
#[derive(Debug, Clone)]
pub struct FsCacheStats {
    pub entry_count: usize,
    pub total_size: usize,
    pub expired_count: usize,
    pub total_access_count: u64,
}

/// Maintenance operation report
#[derive(Debug, Clone, Default)]
pub struct MaintenanceReport {
    pub expired_removed: usize,
    pub corrupted_removed: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_fs_cache_basic_operations() {
        let temp_dir = TempDir::new().unwrap();
        let cache_dir = temp_dir.path().to_str().unwrap();

        let mut cache = FsCacheManager::new(cache_dir).await.unwrap();

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
    async fn test_fs_cache_persistence() {
        let temp_dir = TempDir::new().unwrap();
        let cache_dir = temp_dir.path().to_str().unwrap();

        // Create cache and set data
        {
            let mut cache = FsCacheManager::new(cache_dir).await.unwrap();
            cache.set("persistent_key", "persistent_value".to_string(), None).await.unwrap();
        }

        // Create new cache instance and verify data persists
        {
            let mut cache = FsCacheManager::new(cache_dir).await.unwrap();
            let result: Option<String> = cache.get("persistent_key").await;
            assert_eq!(result, Some("persistent_value".to_string()));
        }
    }

    #[tokio::test]
    async fn test_fs_cache_expiration() {
        let temp_dir = TempDir::new().unwrap();
        let cache_dir = temp_dir.path().to_str().unwrap();

        let mut cache = FsCacheManager::new(cache_dir).await.unwrap();

        // Set with short TTL
        cache.set("key1", "value1".to_string(), Some(Duration::from_millis(100))).await.unwrap();

        // Should be available immediately
        let result: Option<String> = cache.get("key1").await;
        assert_eq!(result, Some("value1".to_string()));

        // Wait for expiration
        tokio::time::sleep(tokio::time::Duration::from_millis(150)).await;

        // Should be expired
        let result: Option<String> = cache.get("key1").await;
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn test_fs_cache_maintenance() {
        let temp_dir = TempDir::new().unwrap();
        let cache_dir = temp_dir.path().to_str().unwrap();

        let mut cache = FsCacheManager::new(cache_dir).await.unwrap();

        // Set some data with short TTL
        cache.set("key1", "value1".to_string(), Some(Duration::from_millis(50))).await.unwrap();
        cache.set("key2", "value2".to_string(), None).await.unwrap();

        // Wait for expiration
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Run maintenance
        let report = cache.maintenance().await.unwrap();
        assert_eq!(report.expired_removed, 1);

        // Verify only non-expired data remains
        let result: Option<String> = cache.get("key1").await;
        assert_eq!(result, None);

        let result: Option<String> = cache.get("key2").await;
        assert_eq!(result, Some("value2".to_string()));
    }
}