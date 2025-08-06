use std::time::Duration;
use tokio::time::sleep;
use ai_commit::cache::{
    ParallelProcessor, ParallelConfig, TaskMetadata, TaskPriority
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("=== Enhanced Parallel Processing Example ===\n");

    // Create enhanced parallel processing configuration
    let parallel_config = ParallelConfig {
        max_concurrent_tasks: 6,
        batch_size: 3,
        task_timeout_seconds: 10,
        enable_task_grouping: true,
        max_retries: 2,
        memory_limit_mb: 2, // 2MB limit for demonstration
        ..Default::default()
    };

    let processor = ParallelProcessor::new(parallel_config);

    println!("✅ Enhanced parallel processor initialized");
    println!("   - Max concurrent tasks: 6");
    println!("   - Batch size: 3");
    println!("   - Memory limit: 2MB");
    println!("   - Task grouping enabled: true\n");

    // Example 1: Scheduled parallel processing with task metadata
    println!("🚀 Example 1: Scheduled Parallel Processing with Metadata");

    let tasks_with_metadata = vec![
        (TaskMetadata {
            id: "high_priority_task_1".to_string(),
            priority: TaskPriority::High,
            estimated_duration: Some(Duration::from_millis(100)),
            memory_requirement: Some(256 * 1024), // 256KB
            dependencies: Vec::new(),
            group: Some("critical_group".to_string()),
        }, "critical_data_1".to_string()),

        (TaskMetadata {
            id: "high_priority_task_2".to_string(),
            priority: TaskPriority::High,
            estimated_duration: Some(Duration::from_millis(150)),
            memory_requirement: Some(512 * 1024), // 512KB
            dependencies: Vec::new(),
            group: Some("critical_group".to_string()),
        }, "critical_data_2".to_string()),

        (TaskMetadata {
            id: "normal_task_1".to_string(),
            priority: TaskPriority::Normal,
            estimated_duration: Some(Duration::from_millis(80)),
            memory_requirement: Some(128 * 1024), // 128KB
            dependencies: Vec::new(),
            group: Some("normal_group".to_string()),
        }, "normal_data_1".to_string()),

        (TaskMetadata {
            id: "low_priority_task_1".to_string(),
            priority: TaskPriority::Low,
            estimated_duration: Some(Duration::from_millis(200)),
            memory_requirement: Some(64 * 1024), // 64KB
            dependencies: Vec::new(),
            group: Some("background_group".to_string()),
        }, "background_data_1".to_string()),
    ];

    let processor_fn = |data: String| {
        async move {
            // Simulate processing time based on data type
            let delay = if data.contains("critical") {
                50
            } else if data.contains("normal") {
                80
            } else {
                120
            };

            sleep(Duration::from_millis(delay)).await;
            Ok(format!("processed_{}", data))
        }
    };

    let start_time = std::time::Instant::now();
    let results = processor.process_with_scheduler(tasks_with_metadata, processor_fn).await;
    let duration = start_time.elapsed();

    println!("   - Processed {} tasks in {:?}", results.len(), duration);
    println!("   - Success rate: {}/{}",
        results.iter().filter(|r| r.result.is_ok()).count(),
        results.len()
    );

    for result in &results {
        println!("     - Task {}: {} ({}ms, {} retries, memory: {:?})",
            result.task_id,
            if result.result.is_ok() { "✅ Success" } else { "❌ Failed" },
            result.execution_time.as_millis(),
            result.retry_count,
            result.memory_used.map(|m| format!("{}KB", m / 1024)).unwrap_or("N/A".to_string())
        );
    }

    // Example 2: Group-based parallel processing
    println!("\n📦 Example 2: Group-based Parallel Processing");

    let grouped_tasks = vec![
        (TaskMetadata {
            id: "frontend_task_1".to_string(),
            priority: TaskPriority::High,
            estimated_duration: None,
            memory_requirement: Some(300 * 1024),
            dependencies: Vec::new(),
            group: Some("frontend".to_string()),
        }, 1),

        (TaskMetadata {
            id: "frontend_task_2".to_string(),
            priority: TaskPriority::Normal,
            estimated_duration: None,
            memory_requirement: Some(200 * 1024),
            dependencies: Vec::new(),
            group: Some("frontend".to_string()),
        }, 2),

        (TaskMetadata {
            id: "backend_task_1".to_string(),
            priority: TaskPriority::High,
            estimated_duration: None,
            memory_requirement: Some(400 * 1024),
            dependencies: Vec::new(),
            group: Some("backend".to_string()),
        }, 3),

        (TaskMetadata {
            id: "backend_task_2".to_string(),
            priority: TaskPriority::Normal,
            estimated_duration: None,
            memory_requirement: Some(350 * 1024),
            dependencies: Vec::new(),
            group: Some("backend".to_string()),
        }, 4),

        (TaskMetadata {
            id: "database_task_1".to_string(),
            priority: TaskPriority::Critical,
            estimated_duration: None,
            memory_requirement: Some(600 * 1024),
            dependencies: Vec::new(),
            group: Some("database".to_string()),
        }, 5),
    ];

    let group_processor_fn = |data: i32| {
        async move {
            // Simulate different processing times for different groups
            let delay = match data {
                1..=2 => 60,  // Frontend tasks
                3..=4 => 100, // Backend tasks
                5 => 150,     // Database tasks
                _ => 80,
            };

            sleep(Duration::from_millis(delay)).await;
            Ok(data * 10)
        }
    };

    let start_time = std::time::Instant::now();
    let group_results = processor.process_by_groups(grouped_tasks, group_processor_fn).await;
    let duration = start_time.elapsed();

    println!("   - Processed {} groups in {:?}", group_results.len(), duration);

    for (group_name, group_results) in &group_results {
        println!("   - Group '{}': {} tasks", group_name, group_results.len());
        for result in group_results {
            println!("     - Task {}: {} ({}ms)",
                result.task_id,
                if result.result.is_ok() {
                    format!("✅ Result: {}", result.result.as_ref().unwrap())
                } else {
                    "❌ Failed".to_string()
                },
                result.execution_time.as_millis()
            );
        }
    }

    // Example 3: Resource monitoring demonstration
    println!("\n📊 Example 3: Resource Monitoring");

    let resource_monitor = processor.get_resource_monitor();
    let (memory_usage, active_tasks) = resource_monitor.get_current_usage().await;

    println!("   - Current memory usage: {} bytes", memory_usage);
    println!("   - Current active tasks: {}", active_tasks);

    // Display final statistics
    println!("\n📈 Final Statistics");

    let stats = processor.get_stats().await;
    println!("   Parallel Processing Statistics:");
    println!("     - Total Tasks: {}", stats.total_tasks);
    println!("     - Completed Tasks: {}", stats.completed_tasks);
    println!("     - Failed Tasks: {}", stats.failed_tasks);
    println!("     - Retried Tasks: {}", stats.retried_tasks);
    println!("     - Average Execution Time: {:?}", stats.average_execution_time);
    println!("     - Peak Memory Usage: {} bytes", stats.peak_memory_usage);
    println!("     - Throughput: {:.2} tasks/second", stats.throughput_per_second);
    println!("     - Current Active Tasks: {}", stats.current_active_tasks);

    // Example 4: Performance comparison
    println!("\n⚡ Example 4: Performance Comparison");

    // Sequential processing
    let sequential_data = vec!["seq1", "seq2", "seq3", "seq4", "seq5"];
    let start_time = std::time::Instant::now();

    let mut sequential_results = Vec::new();
    for data in &sequential_data {
        sleep(Duration::from_millis(100)).await;
        sequential_results.push(format!("processed_{}", data));
    }
    let sequential_duration = start_time.elapsed();

    // Parallel processing
    let parallel_files = sequential_data.iter().map(|s| s.to_string()).collect();
    let parallel_processor_fn = |file_path: String| {
        async move {
            sleep(Duration::from_millis(100)).await;
            Ok(format!("processed_{}", file_path))
        }
    };

    let start_time = std::time::Instant::now();
    let parallel_results = processor.process_files_parallel(parallel_files, parallel_processor_fn).await;
    let parallel_duration = start_time.elapsed();

    println!("   - Sequential processing: {:?} for {} items", sequential_duration, sequential_results.len());
    println!("   - Parallel processing: {:?} for {} items", parallel_duration, parallel_results.len());
    println!("   - Speedup: {:.2}x", sequential_duration.as_secs_f64() / parallel_duration.as_secs_f64());

    println!("\n🎉 Enhanced parallel processing example completed successfully!");

    Ok(())
}