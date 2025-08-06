pub mod manager;
pub mod memory;
pub mod memory_manager;
pub mod storage;
pub mod strategy;
pub mod parallel;

pub use manager::{CacheManager, CacheEntry, CacheStats, CacheConfig};
pub use memory::{MemoryCache, MemoryCacheConfig, MemoryCacheStats, PerformanceMetrics};
pub use memory_manager::{
    MemoryManager, MemoryConfig, AllocationInfo, AllocationCategory, MemoryStats,
    MemoryPressure, StreamingFileReader, ManagedBuffer
};
pub use storage::{FsCacheManager, FsCacheStats, MaintenanceReport};
pub use strategy::{CacheStrategy, KeyStrategy, EvictionStrategy, InvalidationStrategy};
pub use parallel::{
    ParallelProcessor, ParallelConfig, TaskMetadata, TaskPriority, TaskResult,
    BatchTask, ParallelStats, ResourceMonitor, TaskScheduler
};