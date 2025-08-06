use ai_commit::cache::{
    CacheManager, CacheConfig, MemoryManager, MemoryConfig, AllocationCategory, MemoryPressure
};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging (using println! for simplicity in example)
    // env_logger::init();

    println!("=== AI-Commit Memory Management Example ===\n");

    // Example 1: Basic Memory Manager Usage
    println!("1. Basic Memory Manager Usage");
    basic_memory_manager_example().await?;

    // Example 2: Cache Manager with Memory Management
    println!("\n2. Cache Manager with Memory Management");
    cache_with_memory_management_example().await?;

    // Example 3: Large File Streaming
    println!("\n3. Large File Streaming");
    large_file_streaming_example().await?;

    // Example 4: Memory Pressure Handling
    println!("\n4. Memory Pressure Handling");
    memory_pressure_example().await?;

    // Example 5: Managed Buffer Usage
    println!("\n5. Managed Buffer Usage");
    managed_buffer_example().await?;

    Ok(())
}

async fn basic_memory_manager_example() -> anyhow::Result<()> {
    let config = MemoryConfig {
        max_memory_usage: 10 * 1024 * 1024, // 10MB
        cleanup_threshold_percent: 0.8,
        cleanup_interval: Duration::from_secs(5),
        track_allocations: true,
        large_file_threshold: 1024 * 1024, // 1MB
        stream_buffer_size: 64 * 1024, // 64KB
    };

    let memory_manager = MemoryManager::new(config);

    println!("  Initial memory usage: {} bytes", memory_manager.get_current_usage());
    println!("  Memory limit: {} bytes", memory_manager.get_current_usage());

    // Allocate some memory
    memory_manager.allocate("test_allocation_1".to_string(), 1024 * 1024, AllocationCategory::Cache).await?;
    println!("  After 1MB allocation: {} bytes ({}%)",
             memory_manager.get_current_usage(),
             memory_manager.get_usage_percentage());

    memory_manager.allocate("test_allocation_2".to_string(), 2 * 1024 * 1024, AllocationCategory::FileContent).await?;
    println!("  After 2MB allocation: {} bytes ({}%)",
             memory_manager.get_current_usage(),
             memory_manager.get_usage_percentage());

    // Check memory pressure
    let pressure = memory_manager.get_memory_pressure();
    println!("  Memory pressure: {:?}", pressure);

    // Get detailed statistics
    let stats = memory_manager.get_stats().await;
    println!("  Total allocations: {}", stats.total_allocations);
    println!("  Peak usage: {} bytes", stats.peak_usage);

    // Deallocate memory
    let freed = memory_manager.deallocate("test_allocation_1").await?;
    println!("  Freed {} bytes, current usage: {} bytes", freed, memory_manager.get_current_usage());

    Ok(())
}

async fn cache_with_memory_management_example() -> anyhow::Result<()> {
    let config = CacheConfig {
        memory_cache_size: 100,
        enable_fs_cache: false,
        max_memory_usage: 5 * 1024 * 1024, // 5MB
        cleanup_interval: Duration::from_secs(10),
        ..Default::default()
    };

    let cache_manager = CacheManager::new(config).await?;

    println!("  Initial cache memory usage: {} bytes", cache_manager.get_memory_usage());

    // Store some data in cache
    let large_data = vec![0u8; 1024 * 1024]; // 1MB of data
    cache_manager.set("large_data_1", large_data.clone(), None).await?;

    println!("  After storing 1MB: {} bytes ({}%)",
             cache_manager.get_memory_usage(),
             cache_manager.get_memory_usage_percentage());

    // Store more data
    cache_manager.set("large_data_2", large_data.clone(), None).await?;
    cache_manager.set("large_data_3", large_data, None).await?;

    println!("  After storing 3MB total: {} bytes ({}%)",
             cache_manager.get_memory_usage(),
             cache_manager.get_memory_usage_percentage());

    // Check memory pressure
    let pressure = cache_manager.get_memory_pressure();
    println!("  Cache memory pressure: {:?}", pressure);

    // Get memory statistics
    let memory_stats = cache_manager.get_memory_stats().await;
    println!("  Memory cleanup operations: {}", memory_stats.cleanup_operations);
    println!("  Memory pressure events: {}", memory_stats.memory_pressure_events);

    // Force cleanup if needed
    if pressure == MemoryPressure::High || pressure == MemoryPressure::Critical {
        let freed = cache_manager.force_memory_cleanup().await?;
        println!("  Forced cleanup freed: {} bytes", freed);
        println!("  Memory usage after cleanup: {} bytes", cache_manager.get_memory_usage());
    }

    Ok(())
}

async fn large_file_streaming_example() -> anyhow::Result<()> {
    let config = MemoryConfig {
        max_memory_usage: 2 * 1024 * 1024, // 2MB
        large_file_threshold: 512 * 1024, // 512KB
        stream_buffer_size: 64 * 1024, // 64KB
        ..Default::default()
    };

    let memory_manager = MemoryManager::new(config);

    // Create a temporary large file for demonstration
    let temp_dir = tempfile::tempdir()?;
    let file_path = temp_dir.path().join("large_file.txt");
    let large_content = "A".repeat(1024 * 1024); // 1MB of 'A's
    tokio::fs::write(&file_path, &large_content).await?;

    let file_size = large_content.len();
    println!("  Created test file of size: {} bytes", file_size);

    // Check if file should be streamed
    let should_stream = memory_manager.should_stream_file(file_size);
    println!("  Should stream file: {}", should_stream);

    if should_stream {
        println!("  Streaming file...");
        let mut reader = memory_manager.stream_read_file(file_path.to_str().unwrap()).await?;

        let mut total_read = 0;
        let mut chunk_count = 0;

        while let Some(chunk) = reader.read_chunk().await? {
            total_read += chunk.len();
            chunk_count += 1;

            if chunk_count % 5 == 0 { // Print progress every 5 chunks
                println!("    Progress: {:.1}% ({} bytes read, {} chunks)",
                         reader.progress(), total_read, chunk_count);
            }

            // Simulate processing delay
            sleep(Duration::from_millis(10)).await;
        }

        println!("  Streaming complete: {} bytes read in {} chunks", total_read, chunk_count);
        println!("  Memory usage during streaming: {} bytes", memory_manager.get_current_usage());
    } else {
        println!("  File is small enough to load entirely into memory");
    }

    Ok(())
}

async fn memory_pressure_example() -> anyhow::Result<()> {
    let config = MemoryConfig {
        max_memory_usage: 1024 * 1024, // 1MB limit for demonstration
        cleanup_threshold_percent: 0.7, // 70%
        cleanup_interval: Duration::from_millis(100),
        ..Default::default()
    };

    let memory_manager = MemoryManager::new(config);

    println!("  Memory limit: {} bytes", 1024 * 1024);

    // Gradually increase memory usage to demonstrate pressure levels
    let allocation_sizes = vec![200 * 1024, 300 * 1024, 200 * 1024, 400 * 1024]; // 200KB, 300KB, 200KB, 400KB

    for (i, size) in allocation_sizes.iter().enumerate() {
        let allocation_id = format!("pressure_test_{}", i);

        match memory_manager.allocate(allocation_id, *size, AllocationCategory::TemporaryBuffer).await {
            Ok(_) => {
                let usage = memory_manager.get_current_usage();
                let percentage = memory_manager.get_usage_percentage();
                let pressure = memory_manager.get_memory_pressure();

                println!("  Allocation {}: {} bytes allocated", i + 1, size);
                println!("    Current usage: {} bytes ({:.1}%)", usage, percentage);
                println!("    Memory pressure: {:?}", pressure);

                // Trigger cleanup if pressure is high
                if pressure == MemoryPressure::High || pressure == MemoryPressure::Critical {
                    println!("    High memory pressure detected, triggering cleanup...");
                    let freed = memory_manager.force_cleanup().await?;
                    println!("    Cleanup freed: {} bytes", freed);
                    println!("    Usage after cleanup: {} bytes", memory_manager.get_current_usage());
                }
            }
            Err(e) => {
                println!("  Allocation {} failed: {}", i + 1, e);
                println!("    Attempting cleanup and retry...");

                let freed = memory_manager.force_cleanup().await?;
                println!("    Cleanup freed: {} bytes", freed);

                // Retry allocation
                if let Ok(_) = memory_manager.allocate(allocation_id, *size, AllocationCategory::TemporaryBuffer).await {
                    println!("    Retry successful after cleanup");
                } else {
                    println!("    Retry failed even after cleanup");
                }
            }
        }

        // Small delay between allocations
        sleep(Duration::from_millis(50)).await;
    }

    // Final statistics
    let stats = memory_manager.get_stats().await;
    println!("  Final statistics:");
    println!("    Total allocations: {}", stats.total_allocations);
    println!("    Total deallocations: {}", stats.total_deallocations);
    println!("    Cleanup operations: {}", stats.cleanup_operations);
    println!("    Memory pressure events: {}", stats.memory_pressure_events);

    Ok(())
}

async fn managed_buffer_example() -> anyhow::Result<()> {
    let config = MemoryConfig {
        max_memory_usage: 5 * 1024 * 1024, // 5MB
        ..Default::default()
    };

    let memory_manager = MemoryManager::new(config);

    println!("  Creating managed buffers for different use cases...");

    // Create a buffer for analysis results
    let mut analysis_buffer = memory_manager.create_managed_buffer(
        1024 * 1024, // 1MB
        AllocationCategory::AnalysisResult
    ).await?;

    println!("  Created analysis buffer: {} bytes capacity", analysis_buffer.data().capacity());

    // Simulate writing analysis data
    let analysis_data = b"Analysis result: Code quality score: 85/100\n";
    analysis_buffer.data_mut().extend_from_slice(analysis_data);
    analysis_buffer.data_mut().extend_from_slice(&vec![b'X'; 1024]); // Add some bulk data

    println!("  Analysis buffer now contains: {} bytes", analysis_buffer.len());

    // Update access information
    analysis_buffer.touch().await?;

    // Create a temporary buffer for file processing
    let mut temp_buffer = memory_manager.create_managed_buffer(
        512 * 1024, // 512KB
        AllocationCategory::TemporaryBuffer
    ).await?;

    // Simulate file content processing
    let file_content = b"fn main() {\n    println!(\"Hello, world!\");\n}\n";
    temp_buffer.data_mut().extend_from_slice(file_content);
    temp_buffer.data_mut().extend_from_slice(&vec![b' '; 1024]); // Add padding

    println!("  Temporary buffer contains: {} bytes", temp_buffer.len());

    // Check memory usage
    println!("  Current memory usage: {} bytes ({}%)",
             memory_manager.get_current_usage(),
             memory_manager.get_usage_percentage());

    // Get allocations by category
    let allocations_by_category = memory_manager.get_allocations_by_category().await;
    for (category, allocations) in allocations_by_category {
        println!("  Category {:?}: {} allocations", category, allocations.len());
        for allocation in allocations {
            println!("    - {} bytes, accessed {} times",
                     allocation.size, allocation.access_count);
        }
    }

    // Buffers will be automatically cleaned up when they go out of scope
    println!("  Buffers will be automatically deallocated when dropped");

    Ok(())
}