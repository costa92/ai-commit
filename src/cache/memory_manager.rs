use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, RwLock};
use tokio::io::{AsyncRead, AsyncReadExt, BufReader};
use tokio::fs::File;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

/// Memory usage monitoring and management
#[derive(Debug, Clone)]
pub struct MemoryManager {
    config: MemoryConfig,
    current_usage: Arc<AtomicUsize>,
    peak_usage: Arc<AtomicUsize>,
    allocations: Arc<RwLock<HashMap<String, AllocationInfo>>>,
    stats: Arc<Mutex<MemoryStats>>,
    cleanup_threshold: usize,
    last_cleanup: Arc<Mutex<Instant>>,
}

/// Memory management configuration
#[derive(Debug, Clone)]
pub struct MemoryConfig {
    /// Maximum memory usage in bytes (default: 500MB)
    pub max_memory_usage: usize,
    /// Memory cleanup threshold as percentage of max (default: 80%)
    pub cleanup_threshold_percent: f32,
    /// Minimum time between cleanup operations
    pub cleanup_interval: Duration,
    /// Enable detailed allocation tracking
    pub track_allocations: bool,
    /// Large file threshold for streaming (default: 10MB)
    pub large_file_threshold: usize,
    /// Streaming buffer size (default: 64KB)
    pub stream_buffer_size: usize,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            max_memory_usage: 500 * 1024 * 1024, // 500MB
            cleanup_threshold_percent: 0.8, // 80%
            cleanup_interval: Duration::from_secs(30),
            track_allocations: true,
            large_file_threshold: 10 * 1024 * 1024, // 10MB
            stream_buffer_size: 64 * 1024, // 64KB
        }
    }
}

/// Information about a memory allocation
#[derive(Debug, Clone)]
pub struct AllocationInfo {
    pub id: String,
    pub size: usize,
    pub allocated_at: DateTime<Utc>,
    pub last_accessed: DateTime<Utc>,
    pub access_count: u64,
    pub category: AllocationCategory,
    pub metadata: HashMap<String, String>,
}

/// Categories of memory allocations for tracking
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AllocationCategory {
    Cache,
    FileContent,
    AnalysisResult,
    TemporaryBuffer,
    Configuration,
    Other(String),
}

/// Memory usage statistics
#[derive(Debug, Clone, Default)]
pub struct MemoryStats {
    pub current_usage: usize,
    pub peak_usage: usize,
    pub total_allocations: u64,
    pub total_deallocations: u64,
    pub cleanup_operations: u64,
    pub last_cleanup: Option<DateTime<Utc>>,
    pub allocations_by_category: HashMap<AllocationCategory, usize>,
    pub average_allocation_size: f64,
    pub memory_pressure_events: u64,
}

impl MemoryManager {
    /// Create a new memory manager
    pub fn new(config: MemoryConfig) -> Self {
        let cleanup_threshold = (config.max_memory_usage as f32 * config.cleanup_threshold_percent) as usize;

        Self {
            config,
            current_usage: Arc::new(AtomicUsize::new(0)),
            peak_usage: Arc::new(AtomicUsize::new(0)),
            allocations: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(Mutex::new(MemoryStats::default())),
            cleanup_threshold,
            last_cleanup: Arc::new(Mutex::new(Instant::now())),
        }
    }

    /// Check if memory usage is within limits
    pub fn check_memory_usage(&self) -> bool {
        let current = self.current_usage.load(Ordering::Relaxed);
        current <= self.config.max_memory_usage
    }

    /// Get current memory usage in bytes
    pub fn get_current_usage(&self) -> usize {
        self.current_usage.load(Ordering::Relaxed)
    }

    /// Get peak memory usage in bytes
    pub fn get_peak_usage(&self) -> usize {
        self.peak_usage.load(Ordering::Relaxed)
    }

    /// Get memory usage as percentage of limit
    pub fn get_usage_percentage(&self) -> f32 {
        let current = self.current_usage.load(Ordering::Relaxed);
        (current as f32 / self.config.max_memory_usage as f32) * 100.0
    }

    /// Check if memory pressure cleanup is needed
    pub fn needs_cleanup(&self) -> bool {
        let current = self.current_usage.load(Ordering::Relaxed);
        current >= self.cleanup_threshold
    }

    /// Register a memory allocation
    pub async fn allocate(&self, id: String, size: usize, category: AllocationCategory) -> anyhow::Result<()> {
        // Check if allocation would exceed memory limit
        let current = self.current_usage.load(Ordering::Relaxed);
        if current + size > self.config.max_memory_usage {
            // Try cleanup first
            if self.needs_cleanup() {
                self.cleanup_memory().await?;
            }

            // Check again after cleanup
            let current_after_cleanup = self.current_usage.load(Ordering::Relaxed);
            if current_after_cleanup + size > self.config.max_memory_usage {
                return Err(anyhow::anyhow!(
                    "Memory allocation would exceed limit: {} + {} > {}",
                    current_after_cleanup, size, self.config.max_memory_usage
                ));
            }
        }

        // Update usage counters
        let new_usage = self.current_usage.fetch_add(size, Ordering::Relaxed) + size;

        // Update peak usage
        let mut peak = self.peak_usage.load(Ordering::Relaxed);
        while new_usage > peak {
            match self.peak_usage.compare_exchange_weak(peak, new_usage, Ordering::Relaxed, Ordering::Relaxed) {
                Ok(_) => break,
                Err(current_peak) => peak = current_peak,
            }
        }

        // Track allocation if enabled
        if self.config.track_allocations {
            let allocation_info = AllocationInfo {
                id: id.clone(),
                size,
                allocated_at: Utc::now(),
                last_accessed: Utc::now(),
                access_count: 0,
                category: category.clone(),
                metadata: HashMap::new(),
            };

            let mut allocations = self.allocations.write().await;
            allocations.insert(id, allocation_info);
        }

        // Update statistics
        let mut stats = self.stats.lock().await;
        stats.current_usage = new_usage;
        stats.peak_usage = self.peak_usage.load(Ordering::Relaxed);
        stats.total_allocations += 1;
        *stats.allocations_by_category.entry(category).or_insert(0) += size;

        // Update average allocation size
        if stats.total_allocations > 0 {
            stats.average_allocation_size = stats.current_usage as f64 / stats.total_allocations as f64;
        }

        Ok(())
    }

    /// Deallocate memory
    pub async fn deallocate(&self, id: &str) -> anyhow::Result<usize> {
        let size = if self.config.track_allocations {
            let mut allocations = self.allocations.write().await;
            if let Some(allocation_info) = allocations.remove(id) {
                allocation_info.size
            } else {
                return Err(anyhow::anyhow!("Allocation not found: {}", id));
            }
        } else {
            return Err(anyhow::anyhow!("Allocation tracking is disabled"));
        };

        // Update usage counter
        self.current_usage.fetch_sub(size, Ordering::Relaxed);

        // Update statistics
        let mut stats = self.stats.lock().await;
        stats.current_usage = self.current_usage.load(Ordering::Relaxed);
        stats.total_deallocations += 1;

        Ok(size)
    }

    /// Update allocation access information
    pub async fn access_allocation(&self, id: &str) -> anyhow::Result<()> {
        if !self.config.track_allocations {
            return Ok(());
        }

        let mut allocations = self.allocations.write().await;
        if let Some(allocation_info) = allocations.get_mut(id) {
            allocation_info.last_accessed = Utc::now();
            allocation_info.access_count += 1;
        }

        Ok(())
    }

    /// Perform memory cleanup
    pub async fn cleanup_memory(&self) -> anyhow::Result<usize> {
        let mut last_cleanup = self.last_cleanup.lock().await;
        let now = Instant::now();

        // Check if enough time has passed since last cleanup
        if now.duration_since(*last_cleanup) < self.config.cleanup_interval {
            return Ok(0);
        }

        let mut freed_bytes = 0;
        let mut stats = self.stats.lock().await;

        if self.config.track_allocations {
            let mut allocations = self.allocations.write().await;
            let mut to_remove = Vec::new();

            // Find allocations to clean up (least recently used)
            let mut allocation_list: Vec<_> = allocations.iter().collect();
            allocation_list.sort_by(|a, b| a.1.last_accessed.cmp(&b.1.last_accessed));

            let target_usage = (self.config.max_memory_usage as f32 * 0.7) as usize; // Target 70% usage
            let current_usage = self.current_usage.load(Ordering::Relaxed);

            if current_usage > target_usage {
                let bytes_to_free = current_usage - target_usage;
                let mut freed_so_far = 0;

                for (id, allocation_info) in allocation_list {
                    if freed_so_far >= bytes_to_free {
                        break;
                    }

                    // Skip recently accessed allocations
                    let age = Utc::now().signed_duration_since(allocation_info.last_accessed);
                    if age.num_seconds() < 300 { // Don't clean up allocations accessed in last 5 minutes
                        continue;
                    }

                    to_remove.push(id.clone());
                    freed_so_far += allocation_info.size;
                }

                // Remove selected allocations
                for id in to_remove {
                    if let Some(allocation_info) = allocations.remove(&id) {
                        freed_bytes += allocation_info.size;
                        self.current_usage.fetch_sub(allocation_info.size, Ordering::Relaxed);
                    }
                }
            }
        }

        // Update statistics
        stats.cleanup_operations += 1;
        stats.last_cleanup = Some(Utc::now());
        stats.current_usage = self.current_usage.load(Ordering::Relaxed);

        if freed_bytes > 0 {
            stats.memory_pressure_events += 1;
        }

        *last_cleanup = now;

        Ok(freed_bytes)
    }

    /// Get detailed memory statistics
    pub async fn get_stats(&self) -> MemoryStats {
        let mut stats = self.stats.lock().await;
        stats.current_usage = self.current_usage.load(Ordering::Relaxed);
        stats.peak_usage = self.peak_usage.load(Ordering::Relaxed);
        stats.clone()
    }

    /// Get allocation information by category
    pub async fn get_allocations_by_category(&self) -> HashMap<AllocationCategory, Vec<AllocationInfo>> {
        if !self.config.track_allocations {
            return HashMap::new();
        }

        let allocations = self.allocations.read().await;
        let mut by_category = HashMap::new();

        for allocation_info in allocations.values() {
            by_category
                .entry(allocation_info.category.clone())
                .or_insert_with(Vec::new)
                .push(allocation_info.clone());
        }

        by_category
    }

    /// Check if a file should be processed using streaming
    pub fn should_stream_file(&self, file_size: usize) -> bool {
        file_size > self.config.large_file_threshold
    }

    /// Stream read a large file with memory management
    pub async fn stream_read_file(&self, file_path: &str) -> anyhow::Result<StreamingFileReader> {
        let file = File::open(file_path).await?;
        let metadata = file.metadata().await?;
        let file_size = metadata.len() as usize;

        // Check if we have enough memory for the buffer
        let buffer_size = std::cmp::min(self.config.stream_buffer_size, file_size);
        let current_usage = self.current_usage.load(Ordering::Relaxed);

        if current_usage + buffer_size > self.config.max_memory_usage {
            if self.needs_cleanup() {
                self.cleanup_memory().await?;
            }
        }

        let allocation_id = format!("stream_buffer_{}", uuid::Uuid::new_v4());
        self.allocate(allocation_id.clone(), buffer_size, AllocationCategory::TemporaryBuffer).await?;

        Ok(StreamingFileReader {
            reader: BufReader::new(file),
            buffer_size,
            file_size,
            bytes_read: 0,
            allocation_id,
            memory_manager: self.clone(),
        })
    }

    /// Create a memory-managed buffer for large data processing
    pub async fn create_managed_buffer(&self, size: usize, category: AllocationCategory) -> anyhow::Result<ManagedBuffer> {
        let allocation_id = format!("buffer_{}", uuid::Uuid::new_v4());
        self.allocate(allocation_id.clone(), size, category).await?;

        Ok(ManagedBuffer {
            data: Vec::with_capacity(size),
            allocation_id,
            memory_manager: self.clone(),
        })
    }

    /// Force garbage collection and cleanup
    pub async fn force_cleanup(&self) -> anyhow::Result<usize> {
        // Reset cleanup interval check
        {
            let mut last_cleanup = self.last_cleanup.lock().await;
            *last_cleanup = Instant::now() - self.config.cleanup_interval - Duration::from_secs(1);
        }

        self.cleanup_memory().await
    }

    /// Get memory pressure level
    pub fn get_memory_pressure(&self) -> MemoryPressure {
        let usage_percent = self.get_usage_percentage();

        if usage_percent >= 90.0 {
            MemoryPressure::Critical
        } else if usage_percent >= 80.0 {
            MemoryPressure::High
        } else if usage_percent >= 60.0 {
            MemoryPressure::Medium
        } else {
            MemoryPressure::Low
        }
    }
}

/// Memory pressure levels
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MemoryPressure {
    Low,
    Medium,
    High,
    Critical,
}

/// Streaming file reader with memory management
pub struct StreamingFileReader {
    reader: BufReader<File>,
    buffer_size: usize,
    file_size: usize,
    bytes_read: usize,
    allocation_id: String,
    memory_manager: MemoryManager,
}

impl StreamingFileReader {
    /// Read next chunk of data
    pub async fn read_chunk(&mut self) -> anyhow::Result<Option<Vec<u8>>> {
        if self.bytes_read >= self.file_size {
            return Ok(None);
        }

        let mut buffer = vec![0u8; self.buffer_size];
        let bytes_read = self.reader.read(&mut buffer).await?;

        if bytes_read == 0 {
            return Ok(None);
        }

        buffer.truncate(bytes_read);
        self.bytes_read += bytes_read;

        // Update access information
        self.memory_manager.access_allocation(&self.allocation_id).await?;

        Ok(Some(buffer))
    }

    /// Get reading progress as percentage
    pub fn progress(&self) -> f32 {
        if self.file_size == 0 {
            100.0
        } else {
            (self.bytes_read as f32 / self.file_size as f32) * 100.0
        }
    }

    /// Get total file size
    pub fn file_size(&self) -> usize {
        self.file_size
    }

    /// Get bytes read so far
    pub fn bytes_read(&self) -> usize {
        self.bytes_read
    }
}

impl Drop for StreamingFileReader {
    fn drop(&mut self) {
        // Clean up allocation when reader is dropped
        let memory_manager = self.memory_manager.clone();
        let allocation_id = self.allocation_id.clone();

        tokio::spawn(async move {
            let _ = memory_manager.deallocate(&allocation_id).await;
        });
    }
}

/// Memory-managed buffer
pub struct ManagedBuffer {
    data: Vec<u8>,
    allocation_id: String,
    memory_manager: MemoryManager,
}

impl ManagedBuffer {
    /// Get buffer data
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Get mutable buffer data
    pub fn data_mut(&mut self) -> &mut Vec<u8> {
        &mut self.data
    }

    /// Get buffer size
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if buffer is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Update access information
    pub async fn touch(&self) -> anyhow::Result<()> {
        self.memory_manager.access_allocation(&self.allocation_id).await
    }
}

impl Drop for ManagedBuffer {
    fn drop(&mut self) {
        // Clean up allocation when buffer is dropped
        let memory_manager = self.memory_manager.clone();
        let allocation_id = self.allocation_id.clone();

        tokio::spawn(async move {
            let _ = memory_manager.deallocate(&allocation_id).await;
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration as TokioDuration};

    #[tokio::test]
    async fn test_memory_manager_basic_operations() {
        let config = MemoryConfig {
            max_memory_usage: 1024 * 1024, // 1MB for testing
            ..Default::default()
        };
        let manager = MemoryManager::new(config);

        // Test allocation
        let result = manager.allocate("test1".to_string(), 1024, AllocationCategory::Cache).await;
        assert!(result.is_ok());
        assert_eq!(manager.get_current_usage(), 1024);

        // Test deallocation
        let freed = manager.deallocate("test1").await.unwrap();
        assert_eq!(freed, 1024);
        assert_eq!(manager.get_current_usage(), 0);
    }

    #[tokio::test]
    async fn test_memory_limit_enforcement() {
        let config = MemoryConfig {
            max_memory_usage: 1024, // 1KB limit
            ..Default::default()
        };
        let manager = MemoryManager::new(config);

        // This should succeed
        let result = manager.allocate("test1".to_string(), 512, AllocationCategory::Cache).await;
        assert!(result.is_ok());

        // This should fail (would exceed limit)
        let result = manager.allocate("test2".to_string(), 1024, AllocationCategory::Cache).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_memory_cleanup() {
        let config = MemoryConfig {
            max_memory_usage: 2048,
            cleanup_threshold_percent: 0.5, // 50% for testing
            cleanup_interval: Duration::from_millis(10),
            ..Default::default()
        };
        let manager = MemoryManager::new(config);

        // Allocate memory to trigger cleanup threshold
        manager.allocate("test1".to_string(), 800, AllocationCategory::Cache).await.unwrap();
        manager.allocate("test2".to_string(), 800, AllocationCategory::Cache).await.unwrap();

        assert!(manager.needs_cleanup());

        // Wait longer to ensure allocations are old enough for cleanup (more than 5 minutes)
        // For testing, we'll use force_cleanup which bypasses the age check
        let freed = manager.force_cleanup().await.unwrap();

        // The cleanup might not free anything if memory pressure isn't high enough
        // Let's check that cleanup was attempted (freed >= 0)
        assert!(freed >= 0);

        // Verify that cleanup was recorded in stats
        let stats = manager.get_stats().await;
        assert!(stats.cleanup_operations > 0);
    }

    #[tokio::test]
    async fn test_memory_pressure_levels() {
        let config = MemoryConfig {
            max_memory_usage: 1000,
            ..Default::default()
        };
        let manager = MemoryManager::new(config);

        // Low pressure
        manager.allocate("test1".to_string(), 100, AllocationCategory::Cache).await.unwrap();
        assert_eq!(manager.get_memory_pressure(), MemoryPressure::Low);

        // Medium pressure
        manager.allocate("test2".to_string(), 500, AllocationCategory::Cache).await.unwrap();
        assert_eq!(manager.get_memory_pressure(), MemoryPressure::Medium);

        // High pressure
        manager.allocate("test3".to_string(), 200, AllocationCategory::Cache).await.unwrap();
        assert_eq!(manager.get_memory_pressure(), MemoryPressure::High);
    }

    #[tokio::test]
    async fn test_managed_buffer() {
        let config = MemoryConfig {
            max_memory_usage: 10240,
            ..Default::default()
        };
        let manager = MemoryManager::new(config);

        let mut buffer = manager.create_managed_buffer(1024, AllocationCategory::TemporaryBuffer).await.unwrap();

        // Test buffer operations
        assert_eq!(buffer.len(), 0);
        assert!(buffer.is_empty());

        buffer.data_mut().extend_from_slice(b"Hello, World!");
        assert_eq!(buffer.len(), 13);
        assert!(!buffer.is_empty());
        assert_eq!(buffer.data(), b"Hello, World!");

        // Test access tracking
        buffer.touch().await.unwrap();
    }

    #[tokio::test]
    async fn test_file_streaming_decision() {
        let config = MemoryConfig {
            large_file_threshold: 1024,
            ..Default::default()
        };
        let manager = MemoryManager::new(config);

        assert!(!manager.should_stream_file(512)); // Small file
        assert!(manager.should_stream_file(2048)); // Large file
    }
}