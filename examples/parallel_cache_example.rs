use std::time::Duration;
use tokio::time::sleep;
use ai_commit::cache::{
    CacheManager, CacheConfig, ParallelProcessor, ParallelConfig
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("=== AI-Commit Parallel Cache Processing Example ===\n");

    // Create cache configuration
    let cache_config = CacheConfig {
        memory_cache_size: 1000,
        enable_fs_cache: false, // Disable for this example
        max_memory_usage: 50 * 1024 * 1024, // 50MB
        ..Default::default()
    };

    // Create parallel processing configuration
    let parallel_config = ParallelConfig {
        max_concurrent_tasks: 4,
        batch_size: 5,
        task_timeout_seconds: 10,
        enable_task_grouping: true,
        max_retries: 2,
        ..Default::default()
    };

    // Create cache manager with parallel processing
    let cache_manager = CacheManager::new_with_parallel(cache_config, parallel_config).await?;

    println!("✅ Cache manager with parallel processing initialized");
    println!("   - Max concurrent tasks: 4");
    println!("   - Batch size: 5");
    println!("   - Parallel processing enabled: {}\n", cache_manager.is_parallel_enabled());

    // Example 1: Basic cache operations
    println!("💾 Example 1: Basic Cache Operations");

    // Populate cache with some data
    for i in 0..20 {
        let key = format!("key_{}", i);
        let value = format!("value_{}", i);
        cache_manager.set(&key, value, None).await?;
    }

    // Test cache retrieval
    let test_key = "key_5";
    if let Some(value) = cache_manager.get::<String>(test_key).await {
        println!("   - Retrieved {}: {}", test_key, value);
    }

    // Example 2: Direct parallel processor usage
    println!("\n⚡ Example 2: Direct Parallel Processor Usage");

    let parallel_processor = ParallelProcessor::new(ParallelConfig {
        max_concurrent_tasks: 6,
        batch_size: 3,
        enable_task_grouping: true,
        ..Default::default()
    });

    // Simulate file processing
    let files = vec![
        "file1.rs".to_string(),
        "file2.rs".to_string(),
        "file3.rs".to_string(),
        "file4.rs".to_string(),
        "file5.rs".to_string(),
    ];

    let processor_fn = |file_path: String| {
        async move {
            // Simulate processing time
            sleep(Duration::from_millis(100)).await;
            Ok(format!("processed_{}", file_path))
        }
    };

    let start_time = std::time::Instant::now();
    let results = parallel_processor.process_files_parallel(files, processor_fn).await;
    let duration = start_time.elapsed();

    println!("   - Processed {} files in {:?}", results.len(), duration);
    println!("   - Success rate: {}/{}",
        results.iter().filter(|r| r.result.is_ok()).count(),
        results.len()
    );

    for result in &results {
        println!("     - Task {}: {} ({}ms, {} retries)",
            result.task_id,
            if result.result.is_ok() { "✅ Success" } else { "❌ Failed" },
            result.execution_time.as_millis(),
            result.retry_count
        );
    }

    // Example 3: Batch processing
    println!("\n📦 Example 3: Batch Processing");

    let items = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

    let batch_processor_fn = |batch: Vec<i32>| {
        async move {
            sleep(Duration::from_millis(50)).await;
            Ok(batch.into_iter().map(|x| x * 2).collect::<Vec<i32>>())
        }
    };

    let start_time = std::time::Instant::now();
    let batch_results = parallel_processor.process_batches_parallel(items, batch_processor_fn).await;
    let duration = start_time.elapsed();

    println!("   - Processed {} batches in {:?}", batch_results.len(), duration);

    // Collect all processed items
    let mut all_processed = Vec::new();
    for result in batch_results {
        if let Ok(batch_data) = result.result {
            all_processed.extend(batch_data);
        }
    }

    all_processed.sort();
    println!("   - Processed items: {:?}", all_processed);

    // Display final statistics
    println!("\n📊 Final Statistics");

    let cache_stats = cache_manager.stats().await;
    println!("   Cache Statistics:");
    println!("     - Hits: {}", cache_stats.hits);
    println!("     - Misses: {}", cache_stats.misses);
    println!("     - Hit Rate: {:.2}%", cache_stats.hit_rate() * 100.0);
    println!("     - Memory Usage: {} bytes", cache_stats.memory_usage);
    println!("     - Entry Count: {}", cache_stats.entry_count);

    if let Some(parallel_stats) = cache_manager.get_parallel_stats().await {
        println!("   Parallel Processing Statistics:");
        println!("     - Total Tasks: {}", parallel_stats.total_tasks);
        println!("     - Completed Tasks: {}", parallel_stats.completed_tasks);
        println!("     - Failed Tasks: {}", parallel_stats.failed_tasks);
        println!("     - Retried Tasks: {}", parallel_stats.retried_tasks);
        println!("     - Average Execution Time: {:?}", parallel_stats.average_execution_time);
        println!("     - Throughput: {:.2} tasks/second", parallel_stats.throughput_per_second);
        println!("     - Current Active Tasks: {}", parallel_stats.current_active_tasks);
    }

    let processor_stats = parallel_processor.get_stats().await;
    println!("   Direct Processor Statistics:");
    println!("     - Total Tasks: {}", processor_stats.total_tasks);
    println!("     - Completed Tasks: {}", processor_stats.completed_tasks);
    println!("     - Failed Tasks: {}", processor_stats.failed_tasks);
    println!("     - Throughput: {:.2} tasks/second", processor_stats.throughput_per_second);

    println!("\n🎉 Parallel processing example completed successfully!");

    Ok(())
}