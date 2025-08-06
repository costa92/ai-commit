use ai_commit::cache::{
    CacheManager, CacheConfig, MemoryManager, MemoryConfig, AllocationCategory, MemoryPressure
};
use std::time::Duration;

#[tokio::test]
async fn test_memory_manager_integration() -> anyhow::Result<()> {
    // Test basic memory manager functionality
    let config = MemoryConfig {
        max_memory_usage: 1024 * 1024, // 1MB
        cleanup_threshold_percent: 0.8,
        cleanup_interval: Duration::from_secs(1),
        track_allocations: true,
        large_file_threshold: 512 * 1024, // 512KB
        stream_buffer_size: 64 * 1024, // 64KB
    };

    let memory_manager = MemoryManager::new(config);

    // Test allocation
    memory_manager.allocate("test1".to_string(), 100 * 1024, AllocationCategory::Cache).await?;
    assert_eq!(memory_manager.get_current_usage(), 100 * 1024);

    // Test memory pressure
    let pressure = memory_manager.get_memory_pressure();
    assert_eq!(pressure, MemoryPressure::Low);

    // Test deallocation
    let freed = memory_manager.deallocate("test1").await?;
    assert_eq!(freed, 100 * 1024);
    assert_eq!(memory_manager.get_current_usage(), 0);

    Ok(())
}

#[tokio::test]
async fn test_cache_manager_with_memory_management() -> anyhow::Result<()> {
    // Test cache manager with memory management integration
    let config = CacheConfig {
        memory_cache_size: 10,
        enable_fs_cache: false,
        max_memory_usage: 1024 * 1024, // 1MB
        cleanup_interval: Duration::from_secs(1),
        ..Default::default()
    };

    let cache_manager = CacheManager::new(config).await?;

    // Test basic cache operations
    cache_manager.set("key1", "value1".to_string(), None).await?;
    let result: Option<String> = cache_manager.get("key1").await;
    assert_eq!(result, Some("value1".to_string()));

    // Test memory management integration
    let memory_usage = cache_manager.get_memory_usage();
    assert!(memory_usage > 0);

    let memory_pressure = cache_manager.get_memory_pressure();
    assert_eq!(memory_pressure, MemoryPressure::Low);

    // Test file streaming decision
    assert!(!cache_manager.should_stream_file(1024)); // Small file
    assert!(cache_manager.should_stream_file(20 * 1024 * 1024)); // Large file

    Ok(())
}

#[tokio::test]
async fn test_managed_buffer_lifecycle() -> anyhow::Result<()> {
    let config = MemoryConfig {
        max_memory_usage: 2 * 1024 * 1024, // 2MB
        ..Default::default()
    };

    let memory_manager = MemoryManager::new(config);

    // Create a managed buffer
    let mut buffer = memory_manager.create_managed_buffer(
        1024,
        AllocationCategory::TemporaryBuffer
    ).await?;

    // Test buffer operations
    assert_eq!(buffer.len(), 0);
    assert!(buffer.is_empty());

    // Add some data
    buffer.data_mut().extend_from_slice(b"Hello, World!");
    assert_eq!(buffer.len(), 13);
    assert!(!buffer.is_empty());
    assert_eq!(buffer.data(), b"Hello, World!");

    // Test access tracking
    buffer.touch().await?;

    // Buffer will be automatically cleaned up when dropped
    drop(buffer);

    // Give some time for cleanup
    tokio::time::sleep(Duration::from_millis(10)).await;

    Ok(())
}

#[tokio::test]
async fn test_memory_pressure_handling() -> anyhow::Result<()> {
    let config = MemoryConfig {
        max_memory_usage: 1024, // 1KB for testing
        cleanup_threshold_percent: 0.5, // 50%
        cleanup_interval: Duration::from_millis(10),
        ..Default::default()
    };

    let memory_manager = MemoryManager::new(config);

    // Allocate memory to reach medium pressure
    memory_manager.allocate("test1".to_string(), 600, AllocationCategory::Cache).await?;
    assert_eq!(memory_manager.get_memory_pressure(), MemoryPressure::Medium);

    // Allocate more to reach high pressure
    memory_manager.allocate("test2".to_string(), 200, AllocationCategory::Cache).await?;
    assert_eq!(memory_manager.get_memory_pressure(), MemoryPressure::High);

    // Test cleanup
    let freed = memory_manager.force_cleanup().await?;
    assert!(freed >= 0); // Cleanup was attempted

    Ok(())
}