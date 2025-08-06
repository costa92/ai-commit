use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Semaphore, RwLock};
use futures_util::future::join_all;
use serde::{Serialize, Deserialize};
use tracing::{info, warn, error, debug};

/// 并行处理器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParallelConfig {
    /// 最大并发任务数
    pub max_concurrent_tasks: usize,
    /// 线程池大小
    pub thread_pool_size: usize,
    /// 任务超时时间（秒）
    pub task_timeout_seconds: u64,
    /// 批处理大小
    pub batch_size: usize,
    /// 内存限制（MB）
    pub memory_limit_mb: usize,
    /// 是否启用任务分组
    pub enable_task_grouping: bool,
    /// 任务重试次数
    pub max_retries: u32,
    /// 重试延迟（毫秒）
    pub retry_delay_ms: u64,
}

impl Default for ParallelConfig {
    fn default() -> Self {
        Self {
            max_concurrent_tasks: num_cpus::get().max(4),
            thread_pool_size: num_cpus::get(),
            task_timeout_seconds: 300, // 5 minutes
            batch_size: 10,
            memory_limit_mb: 512,
            enable_task_grouping: true,
            max_retries: 3,
            retry_delay_ms: 1000,
        }
    }
}

/// 任务元数据
#[derive(Debug, Clone)]
pub struct TaskMetadata {
    pub id: String,
    pub priority: TaskPriority,
    pub estimated_duration: Option<Duration>,
    pub memory_requirement: Option<usize>,
    pub dependencies: Vec<String>,
    pub group: Option<String>,
}

/// 任务优先级
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum TaskPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

/// 任务执行结果
#[derive(Debug)]
pub struct TaskResult<T> {
    pub task_id: String,
    pub result: anyhow::Result<T>,
    pub execution_time: Duration,
    pub memory_used: Option<usize>,
    pub retry_count: u32,
}

/// 批处理任务
#[derive(Debug, Clone)]
pub struct BatchTask<T> {
    pub items: Vec<T>,
    pub metadata: TaskMetadata,
}

/// 并行处理统计信息
#[derive(Debug, Clone, Default)]
pub struct ParallelStats {
    pub total_tasks: u64,
    pub completed_tasks: u64,
    pub failed_tasks: u64,
    pub retried_tasks: u64,
    pub average_execution_time: Duration,
    pub peak_memory_usage: usize,
    pub current_active_tasks: usize,
    pub throughput_per_second: f64,
}

/// 资源监控器
#[derive(Debug)]
pub struct ResourceMonitor {
    memory_usage: Arc<RwLock<usize>>,
    active_tasks: Arc<RwLock<usize>>,
    config: ParallelConfig,
}

impl ResourceMonitor {
    pub fn new(config: ParallelConfig) -> Self {
        Self {
            memory_usage: Arc::new(RwLock::new(0)),
            active_tasks: Arc::new(RwLock::new(0)),
            config,
        }
    }

    pub async fn can_start_task(&self, memory_requirement: Option<usize>) -> bool {
        let current_memory = *self.memory_usage.read().await;
        let current_tasks = *self.active_tasks.read().await;

        // Check task limit
        if current_tasks >= self.config.max_concurrent_tasks {
            return false;
        }

        // Check memory limit
        if let Some(required_memory) = memory_requirement {
            let memory_limit = self.config.memory_limit_mb * 1024 * 1024; // Convert to bytes
            if current_memory + required_memory > memory_limit {
                return false;
            }
        }

        true
    }

    pub async fn start_task(&self, memory_requirement: Option<usize>) {
        if let Some(memory) = memory_requirement {
            let mut current_memory = self.memory_usage.write().await;
            *current_memory += memory;
        }

        let mut current_tasks = self.active_tasks.write().await;
        *current_tasks += 1;
    }

    pub async fn finish_task(&self, memory_requirement: Option<usize>) {
        if let Some(memory) = memory_requirement {
            let mut current_memory = self.memory_usage.write().await;
            *current_memory = current_memory.saturating_sub(memory);
        }

        let mut current_tasks = self.active_tasks.write().await;
        *current_tasks = current_tasks.saturating_sub(1);
    }

    pub async fn get_current_usage(&self) -> (usize, usize) {
        let memory = *self.memory_usage.read().await;
        let tasks = *self.active_tasks.read().await;
        (memory, tasks)
    }
}

/// 任务调度器
#[derive(Debug)]
pub struct TaskScheduler<T> {
    pending_tasks: Arc<RwLock<Vec<(TaskMetadata, T)>>>,
    config: ParallelConfig,
}

impl<T> TaskScheduler<T> {
    pub fn new(config: ParallelConfig) -> Self {
        Self {
            pending_tasks: Arc::new(RwLock::new(Vec::new())),
            config,
        }
    }

    pub async fn add_task(&self, metadata: TaskMetadata, task: T) {
        let mut tasks = self.pending_tasks.write().await;
        tasks.push((metadata, task));

        // Sort by priority (highest first)
        tasks.sort_by(|a, b| b.0.priority.cmp(&a.0.priority));
    }

    pub async fn add_tasks(&self, task_list: Vec<(TaskMetadata, T)>) {
        let mut tasks = self.pending_tasks.write().await;
        tasks.extend(task_list);

        // Sort by priority (highest first)
        tasks.sort_by(|a, b| b.0.priority.cmp(&a.0.priority));
    }

    pub async fn get_next_batch(&self, batch_size: usize) -> Vec<(TaskMetadata, T)> {
        let mut tasks = self.pending_tasks.write().await;
        let batch_size = batch_size.min(tasks.len());
        tasks.drain(0..batch_size).collect()
    }

    pub async fn get_tasks_by_group(&self, group: &str) -> Vec<(TaskMetadata, T)> {
        let mut tasks = self.pending_tasks.write().await;
        let mut group_tasks = Vec::new();
        let mut remaining_tasks = Vec::new();

        for (metadata, task) in tasks.drain(..) {
            if metadata.group.as_ref() == Some(&group.to_string()) {
                group_tasks.push((metadata, task));
            } else {
                remaining_tasks.push((metadata, task));
            }
        }

        *tasks = remaining_tasks;
        group_tasks
    }

    pub async fn get_tasks_by_priority(&self, priority: TaskPriority) -> Vec<(TaskMetadata, T)> {
        let mut tasks = self.pending_tasks.write().await;
        let mut priority_tasks = Vec::new();
        let mut remaining_tasks = Vec::new();

        for (metadata, task) in tasks.drain(..) {
            if metadata.priority == priority {
                priority_tasks.push((metadata, task));
            } else {
                remaining_tasks.push((metadata, task));
            }
        }

        *tasks = remaining_tasks;
        priority_tasks
    }

    pub async fn get_optimal_batch(&self, resource_monitor: &ResourceMonitor) -> Vec<(TaskMetadata, T)> {
        let mut tasks = self.pending_tasks.write().await;
        let mut batch = Vec::new();
        let mut remaining_tasks = Vec::new();
        let mut total_memory_requirement = 0;

        for (metadata, task) in tasks.drain(..) {
            let memory_req = metadata.memory_requirement.unwrap_or(0);

            // Check if we can add this task to the batch
            if resource_monitor.can_start_task(Some(total_memory_requirement + memory_req)).await
                && batch.len() < self.config.batch_size {
                total_memory_requirement += memory_req;
                batch.push((metadata, task));
            } else {
                remaining_tasks.push((metadata, task));
            }
        }

        *tasks = remaining_tasks;
        batch
    }

    pub async fn pending_count(&self) -> usize {
        self.pending_tasks.read().await.len()
    }

    pub async fn clear(&self) {
        let mut tasks = self.pending_tasks.write().await;
        tasks.clear();
    }

    pub async fn get_pending_groups(&self) -> Vec<String> {
        let tasks = self.pending_tasks.read().await;
        let mut groups = std::collections::HashSet::new();

        for (metadata, _) in tasks.iter() {
            if let Some(group) = &metadata.group {
                groups.insert(group.clone());
            }
        }

        groups.into_iter().collect()
    }
}

/// 并行处理器
pub struct ParallelProcessor {
    semaphore: Arc<Semaphore>,
    config: ParallelConfig,
    resource_monitor: Arc<ResourceMonitor>,
    stats: Arc<RwLock<ParallelStats>>,
    start_time: Instant,
}

impl ParallelProcessor {
    /// 创建新的并行处理器
    pub fn new(config: ParallelConfig) -> Self {
        let semaphore = Arc::new(Semaphore::new(config.max_concurrent_tasks));
        let resource_monitor = Arc::new(ResourceMonitor::new(config.clone()));
        let stats = Arc::new(RwLock::new(ParallelStats::default()));

        Self {
            semaphore,
            config,
            resource_monitor,
            stats,
            start_time: Instant::now(),
        }
    }

    /// 并行处理文件列表
    pub async fn process_files_parallel<F, R, Fut>(&self, files: Vec<String>, processor: F) -> Vec<TaskResult<R>>
    where
        F: Fn(String) -> Fut + Send + Sync + Clone + 'static,
        Fut: std::future::Future<Output = anyhow::Result<R>> + Send + 'static,
        R: Send + 'static,
    {
        info!("Starting parallel processing of {} files", files.len());

        let mut tasks = Vec::new();
        let total_files = files.len();

        // Update total tasks count
        {
            let mut stats = self.stats.write().await;
            stats.total_tasks += total_files as u64;
        }

        for (index, file_path) in files.into_iter().enumerate() {
            let semaphore = self.semaphore.clone();
            let processor = processor.clone();
            let resource_monitor = self.resource_monitor.clone();
            let stats = self.stats.clone();
            let config = self.config.clone();
            let task_id = format!("file_task_{}", index);

            let task = tokio::spawn(async move {
                let start_time = Instant::now();
                let mut retry_count = 0;

                loop {
                    // Wait for resource availability
                    while !resource_monitor.can_start_task(None).await {
                        tokio::time::sleep(Duration::from_millis(100)).await;
                    }

                    // Acquire semaphore permit
                    let _permit = semaphore.acquire().await.unwrap();

                    // Start task monitoring
                    resource_monitor.start_task(None).await;

                    // Update active tasks count
                    {
                        let mut stats_guard = stats.write().await;
                        stats_guard.current_active_tasks += 1;
                    }

                    debug!("Processing file: {}", file_path);

                    // Execute the task with timeout
                    let timeout_duration = Duration::from_secs(config.task_timeout_seconds);
                    let result = tokio::time::timeout(
                        timeout_duration,
                        processor(file_path.clone())
                    ).await;

                    // Finish task monitoring
                    resource_monitor.finish_task(None).await;

                    // Update active tasks count
                    {
                        let mut stats_guard = stats.write().await;
                        stats_guard.current_active_tasks = stats_guard.current_active_tasks.saturating_sub(1);
                    }

                    let execution_time = start_time.elapsed();

                    match result {
                        Ok(Ok(value)) => {
                            // Success
                            {
                                let mut stats_guard = stats.write().await;
                                stats_guard.completed_tasks += 1;
                                stats_guard.average_execution_time =
                                    (stats_guard.average_execution_time * (stats_guard.completed_tasks - 1) as u32 + execution_time) / stats_guard.completed_tasks as u32;
                            }

                            return TaskResult {
                                task_id,
                                result: Ok(value),
                                execution_time,
                                memory_used: None,
                                retry_count,
                            };
                        }
                        Ok(Err(e)) => {
                            // Error
                            retry_count += 1;

                            if retry_count <= config.max_retries {
                                warn!("Task {} failed (attempt {}), retrying: {}", task_id, retry_count, e);

                                {
                                    let mut stats_guard = stats.write().await;
                                    stats_guard.retried_tasks += 1;
                                }

                                // Wait before retry
                                tokio::time::sleep(Duration::from_millis(config.retry_delay_ms)).await;
                                continue;
                            } else {
                                // Max retries exceeded
                                error!("Task {} failed after {} retries: {}", task_id, retry_count, e);

                                {
                                    let mut stats_guard = stats.write().await;
                                    stats_guard.failed_tasks += 1;
                                }

                                return TaskResult {
                                    task_id,
                                    result: Err(e),
                                    execution_time,
                                    memory_used: None,
                                    retry_count,
                                };
                            }
                        }
                        Err(_) => {
                            // Timeout
                            retry_count += 1;

                            if retry_count <= config.max_retries {
                                warn!("Task {} timed out (attempt {}), retrying", task_id, retry_count);

                                {
                                    let mut stats_guard = stats.write().await;
                                    stats_guard.retried_tasks += 1;
                                }

                                // Wait before retry
                                tokio::time::sleep(Duration::from_millis(config.retry_delay_ms)).await;
                                continue;
                            } else {
                                // Max retries exceeded
                                let error = anyhow::anyhow!("Task timed out after {} seconds", config.task_timeout_seconds);
                                error!("Task {} failed after {} retries: {}", task_id, retry_count, error);

                                {
                                    let mut stats_guard = stats.write().await;
                                    stats_guard.failed_tasks += 1;
                                }

                                return TaskResult {
                                    task_id,
                                    result: Err(error),
                                    execution_time,
                                    memory_used: None,
                                    retry_count,
                                };
                            }
                        }
                    }
                }
            });

            tasks.push(task);
        }

        // Wait for all tasks to complete
        let results = join_all(tasks).await;

        // Collect results and handle join errors
        let mut task_results = Vec::new();
        for (index, result) in results.into_iter().enumerate() {
            match result {
                Ok(task_result) => task_results.push(task_result),
                Err(join_error) => {
                    error!("Task {} panicked: {}", index, join_error);
                    task_results.push(TaskResult {
                        task_id: format!("file_task_{}", index),
                        result: Err(anyhow::anyhow!("Task panicked: {}", join_error)),
                        execution_time: Duration::from_secs(0),
                        memory_used: None,
                        retry_count: 0,
                    });
                }
            }
        }

        // Update throughput statistics
        {
            let mut stats = self.stats.write().await;
            let elapsed = self.start_time.elapsed();
            stats.throughput_per_second = stats.completed_tasks as f64 / elapsed.as_secs_f64();
        }

        info!("Completed parallel processing of {} files", total_files);
        task_results
    }

    /// 批量并行处理
    pub async fn process_batches_parallel<T, F, R, Fut>(
        &self,
        items: Vec<T>,
        processor: F,
    ) -> Vec<TaskResult<Vec<R>>>
    where
        T: Send + 'static + Clone,
        F: Fn(Vec<T>) -> Fut + Send + Sync + Clone + 'static,
        Fut: std::future::Future<Output = anyhow::Result<Vec<R>>> + Send + 'static,
        R: Send + 'static,
    {
        if items.is_empty() {
            return Vec::new();
        }

        info!("Starting batch parallel processing of {} items", items.len());

        // Split items into batches
        let batches: Vec<Vec<T>> = items
            .chunks(self.config.batch_size)
            .map(|chunk| chunk.to_vec())
            .collect();

        let batch_count = batches.len();
        info!("Created {} batches with batch size {}", batch_count, self.config.batch_size);

        // Update total tasks count
        {
            let mut stats = self.stats.write().await;
            stats.total_tasks += batch_count as u64;
        }

        let mut tasks = Vec::new();

        for (batch_index, batch) in batches.into_iter().enumerate() {
            let semaphore = self.semaphore.clone();
            let processor = processor.clone();
            let resource_monitor = self.resource_monitor.clone();
            let stats = self.stats.clone();
            let config = self.config.clone();
            let task_id = format!("batch_task_{}", batch_index);

            let task = tokio::spawn(async move {
                let start_time = Instant::now();
                let mut retry_count = 0;

                loop {
                    // Wait for resource availability
                    while !resource_monitor.can_start_task(None).await {
                        tokio::time::sleep(Duration::from_millis(100)).await;
                    }

                    // Acquire semaphore permit
                    let _permit = semaphore.acquire().await.unwrap();

                    // Start task monitoring
                    resource_monitor.start_task(None).await;

                    // Update active tasks count
                    {
                        let mut stats_guard = stats.write().await;
                        stats_guard.current_active_tasks += 1;
                    }

                    debug!("Processing batch {} with {} items", batch_index, batch.len());

                    // Execute the batch with timeout
                    let timeout_duration = Duration::from_secs(config.task_timeout_seconds);
                    let result = tokio::time::timeout(
                        timeout_duration,
                        processor(batch.clone())
                    ).await;

                    // Finish task monitoring
                    resource_monitor.finish_task(None).await;

                    // Update active tasks count
                    {
                        let mut stats_guard = stats.write().await;
                        stats_guard.current_active_tasks = stats_guard.current_active_tasks.saturating_sub(1);
                    }

                    let execution_time = start_time.elapsed();

                    match result {
                        Ok(Ok(results)) => {
                            // Success
                            {
                                let mut stats_guard = stats.write().await;
                                stats_guard.completed_tasks += 1;
                                stats_guard.average_execution_time =
                                    (stats_guard.average_execution_time * (stats_guard.completed_tasks - 1) as u32 + execution_time) / stats_guard.completed_tasks as u32;
                            }

                            return TaskResult {
                                task_id,
                                result: Ok(results),
                                execution_time,
                                memory_used: None,
                                retry_count,
                            };
                        }
                        Ok(Err(e)) => {
                            // Error
                            retry_count += 1;

                            if retry_count <= config.max_retries {
                                warn!("Batch task {} failed (attempt {}), retrying: {}", task_id, retry_count, e);

                                {
                                    let mut stats_guard = stats.write().await;
                                    stats_guard.retried_tasks += 1;
                                }

                                // Wait before retry
                                tokio::time::sleep(Duration::from_millis(config.retry_delay_ms)).await;
                                continue;
                            } else {
                                // Max retries exceeded
                                error!("Batch task {} failed after {} retries: {}", task_id, retry_count, e);

                                {
                                    let mut stats_guard = stats.write().await;
                                    stats_guard.failed_tasks += 1;
                                }

                                return TaskResult {
                                    task_id,
                                    result: Err(e),
                                    execution_time,
                                    memory_used: None,
                                    retry_count,
                                };
                            }
                        }
                        Err(_) => {
                            // Timeout
                            retry_count += 1;

                            if retry_count <= config.max_retries {
                                warn!("Batch task {} timed out (attempt {}), retrying", task_id, retry_count);

                                {
                                    let mut stats_guard = stats.write().await;
                                    stats_guard.retried_tasks += 1;
                                }

                                // Wait before retry
                                tokio::time::sleep(Duration::from_millis(config.retry_delay_ms)).await;
                                continue;
                            } else {
                                // Max retries exceeded
                                let error = anyhow::anyhow!("Batch task timed out after {} seconds", config.task_timeout_seconds);
                                error!("Batch task {} failed after {} retries: {}", task_id, retry_count, error);

                                {
                                    let mut stats_guard = stats.write().await;
                                    stats_guard.failed_tasks += 1;
                                }

                                return TaskResult {
                                    task_id,
                                    result: Err(error),
                                    execution_time,
                                    memory_used: None,
                                    retry_count,
                                };
                            }
                        }
                    }
                }
            });

            tasks.push(task);
        }

        // Wait for all batch tasks to complete
        let results = join_all(tasks).await;

        // Collect results and handle join errors
        let mut task_results = Vec::new();
        for (index, result) in results.into_iter().enumerate() {
            match result {
                Ok(task_result) => task_results.push(task_result),
                Err(join_error) => {
                    error!("Batch task {} panicked: {}", index, join_error);
                    task_results.push(TaskResult {
                        task_id: format!("batch_task_{}", index),
                        result: Err(anyhow::anyhow!("Batch task panicked: {}", join_error)),
                        execution_time: Duration::from_secs(0),
                        memory_used: None,
                        retry_count: 0,
                    });
                }
            }
        }

        // Update throughput statistics
        {
            let mut stats = self.stats.write().await;
            let elapsed = self.start_time.elapsed();
            stats.throughput_per_second = stats.completed_tasks as f64 / elapsed.as_secs_f64();
        }

        info!("Completed batch parallel processing of {} batches", batch_count);
        task_results
    }

    /// 获取处理统计信息
    pub async fn get_stats(&self) -> ParallelStats {
        let mut stats = self.stats.read().await.clone();

        // Update current resource usage
        let (memory_usage, active_tasks) = self.resource_monitor.get_current_usage().await;
        stats.peak_memory_usage = stats.peak_memory_usage.max(memory_usage);
        stats.current_active_tasks = active_tasks;

        // Update throughput
        let elapsed = self.start_time.elapsed();
        if elapsed.as_secs() > 0 {
            stats.throughput_per_second = stats.completed_tasks as f64 / elapsed.as_secs_f64();
        }

        stats
    }

    /// 等待所有任务完成
    pub async fn wait_for_completion(&self) {
        loop {
            let (_, active_tasks) = self.resource_monitor.get_current_usage().await;
            if active_tasks == 0 {
                break;
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }

    /// 获取配置信息
    pub fn get_config(&self) -> &ParallelConfig {
        &self.config
    }

    /// 使用任务调度器进行分组并行处理
    pub async fn process_with_scheduler<T, F, R, Fut>(
        &self,
        tasks: Vec<(TaskMetadata, T)>,
        processor: F,
    ) -> Vec<TaskResult<R>>
    where
        T: Send + 'static + Clone,
        F: Fn(T) -> Fut + Send + Sync + Clone + 'static,
        Fut: std::future::Future<Output = anyhow::Result<R>> + Send + 'static,
        R: Send + 'static,
    {
        if tasks.is_empty() {
            return Vec::new();
        }

        info!("Starting scheduled parallel processing of {} tasks", tasks.len());

        let scheduler = TaskScheduler::new(self.config.clone());

        // Add all tasks to scheduler
        scheduler.add_tasks(tasks).await;

        let mut all_results = Vec::new();
        let mut active_tasks = Vec::new();

        // Process tasks in optimal batches
        while scheduler.pending_count().await > 0 || !active_tasks.is_empty() {
            // Get optimal batch based on resource availability
            let batch = scheduler.get_optimal_batch(&self.resource_monitor).await;

            if !batch.is_empty() {
                // Update total tasks count
                {
                    let mut stats = self.stats.write().await;
                    stats.total_tasks += batch.len() as u64;
                }

                // Process batch
                for (metadata, task_data) in batch {
                    let semaphore = self.semaphore.clone();
                    let processor = processor.clone();
                    let resource_monitor = self.resource_monitor.clone();
                    let stats = self.stats.clone();
                    let config = self.config.clone();
                    let task_id = metadata.id.clone();
                    let memory_req = metadata.memory_requirement;

                    let task_handle = tokio::spawn(async move {
                        let start_time = Instant::now();
                        let mut retry_count = 0;

                        loop {
                            // Wait for resource availability
                            while !resource_monitor.can_start_task(memory_req).await {
                                tokio::time::sleep(Duration::from_millis(100)).await;
                            }

                            // Acquire semaphore permit
                            let _permit = semaphore.acquire().await.unwrap();

                            // Start task monitoring
                            resource_monitor.start_task(memory_req).await;

                            // Update active tasks count
                            {
                                let mut stats_guard = stats.write().await;
                                stats_guard.current_active_tasks += 1;
                            }

                            debug!("Processing scheduled task: {}", task_id);

                            // Execute the task with timeout
                            let timeout_duration = Duration::from_secs(config.task_timeout_seconds);
                            let result = tokio::time::timeout(
                                timeout_duration,
                                processor(task_data.clone())
                            ).await;

                            // Finish task monitoring
                            resource_monitor.finish_task(memory_req).await;

                            // Update active tasks count
                            {
                                let mut stats_guard = stats.write().await;
                                stats_guard.current_active_tasks = stats_guard.current_active_tasks.saturating_sub(1);
                            }

                            let execution_time = start_time.elapsed();

                            match result {
                                Ok(Ok(value)) => {
                                    // Success
                                    {
                                        let mut stats_guard = stats.write().await;
                                        stats_guard.completed_tasks += 1;
                                        stats_guard.average_execution_time =
                                            (stats_guard.average_execution_time * (stats_guard.completed_tasks - 1) as u32 + execution_time) / stats_guard.completed_tasks as u32;
                                    }

                                    return TaskResult {
                                        task_id,
                                        result: Ok(value),
                                        execution_time,
                                        memory_used: memory_req,
                                        retry_count,
                                    };
                                }
                                Ok(Err(e)) => {
                                    // Error
                                    retry_count += 1;

                                    if retry_count <= config.max_retries {
                                        warn!("Scheduled task {} failed (attempt {}), retrying: {}", task_id, retry_count, e);

                                        {
                                            let mut stats_guard = stats.write().await;
                                            stats_guard.retried_tasks += 1;
                                        }

                                        // Wait before retry
                                        tokio::time::sleep(Duration::from_millis(config.retry_delay_ms)).await;
                                        continue;
                                    } else {
                                        // Max retries exceeded
                                        error!("Scheduled task {} failed after {} retries: {}", task_id, retry_count, e);

                                        {
                                            let mut stats_guard = stats.write().await;
                                            stats_guard.failed_tasks += 1;
                                        }

                                        return TaskResult {
                                            task_id,
                                            result: Err(e),
                                            execution_time,
                                            memory_used: memory_req,
                                            retry_count,
                                        };
                                    }
                                }
                                Err(_) => {
                                    // Timeout
                                    retry_count += 1;

                                    if retry_count <= config.max_retries {
                                        warn!("Scheduled task {} timed out (attempt {}), retrying", task_id, retry_count);

                                        {
                                            let mut stats_guard = stats.write().await;
                                            stats_guard.retried_tasks += 1;
                                        }

                                        // Wait before retry
                                        tokio::time::sleep(Duration::from_millis(config.retry_delay_ms)).await;
                                        continue;
                                    } else {
                                        // Max retries exceeded
                                        let error = anyhow::anyhow!("Scheduled task timed out after {} seconds", config.task_timeout_seconds);
                                        error!("Scheduled task {} failed after {} retries: {}", task_id, retry_count, error);

                                        {
                                            let mut stats_guard = stats.write().await;
                                            stats_guard.failed_tasks += 1;
                                        }

                                        return TaskResult {
                                            task_id,
                                            result: Err(error),
                                            execution_time,
                                            memory_used: memory_req,
                                            retry_count,
                                        };
                                    }
                                }
                            }
                        }
                    });

                    active_tasks.push(task_handle);
                }
            }

            // Check for completed tasks
            let mut completed_indices = Vec::new();
            for (index, task_handle) in active_tasks.iter().enumerate() {
                if task_handle.is_finished() {
                    completed_indices.push(index);
                }
            }

            // Collect completed results
            for index in completed_indices.into_iter().rev() {
                let task_handle = active_tasks.remove(index);
                match task_handle.await {
                    Ok(task_result) => all_results.push(task_result),
                    Err(join_error) => {
                        error!("Scheduled task panicked: {}", join_error);
                        all_results.push(TaskResult {
                            task_id: format!("scheduled_task_{}", index),
                            result: Err(anyhow::anyhow!("Task panicked: {}", join_error)),
                            execution_time: Duration::from_secs(0),
                            memory_used: None,
                            retry_count: 0,
                        });
                    }
                }
            }

            // Small delay to prevent busy waiting
            if scheduler.pending_count().await > 0 || !active_tasks.is_empty() {
                tokio::time::sleep(Duration::from_millis(50)).await;
            }
        }

        // Wait for any remaining active tasks
        for task_handle in active_tasks {
            match task_handle.await {
                Ok(task_result) => all_results.push(task_result),
                Err(join_error) => {
                    error!("Scheduled task panicked: {}", join_error);
                    all_results.push(TaskResult {
                        task_id: "unknown_scheduled_task".to_string(),
                        result: Err(anyhow::anyhow!("Task panicked: {}", join_error)),
                        execution_time: Duration::from_secs(0),
                        memory_used: None,
                        retry_count: 0,
                    });
                }
            }
        }

        // Update throughput statistics
        {
            let mut stats = self.stats.write().await;
            let elapsed = self.start_time.elapsed();
            stats.throughput_per_second = stats.completed_tasks as f64 / elapsed.as_secs_f64();
        }

        info!("Completed scheduled parallel processing of {} tasks", all_results.len());
        all_results
    }

    /// 按组并行处理任务
    pub async fn process_by_groups<T, F, R, Fut>(
        &self,
        tasks: Vec<(TaskMetadata, T)>,
        processor: F,
    ) -> std::collections::HashMap<String, Vec<TaskResult<R>>>
    where
        T: Send + 'static + Clone,
        F: Fn(T) -> Fut + Send + Sync + Clone + 'static,
        Fut: std::future::Future<Output = anyhow::Result<R>> + Send + 'static,
        R: Send + 'static,
    {
        if tasks.is_empty() {
            return std::collections::HashMap::new();
        }

        info!("Starting group-based parallel processing of {} tasks", tasks.len());

        let scheduler = TaskScheduler::new(self.config.clone());
        scheduler.add_tasks(tasks).await;

        let groups = scheduler.get_pending_groups().await;
        let mut group_results = std::collections::HashMap::new();

        for group in groups {
            info!("Processing group: {}", group);
            let group_tasks = scheduler.get_tasks_by_group(&group).await;
            let results = self.process_with_scheduler(group_tasks, processor.clone()).await;
            group_results.insert(group, results);
        }

        // Process ungrouped tasks
        let remaining_tasks = scheduler.get_next_batch(scheduler.pending_count().await).await;
        if !remaining_tasks.is_empty() {
            info!("Processing {} ungrouped tasks", remaining_tasks.len());
            let results = self.process_with_scheduler(remaining_tasks, processor).await;
            group_results.insert("ungrouped".to_string(), results);
        }

        info!("Completed group-based parallel processing");
        group_results
    }

    /// 获取资源监控器的引用
    pub fn get_resource_monitor(&self) -> &Arc<ResourceMonitor> {
        &self.resource_monitor
    }

    /// 重置统计信息
    pub async fn reset_stats(&self) {
        let mut stats = self.stats.write().await;
        *stats = ParallelStats::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[tokio::test]
    async fn test_parallel_processor_creation() {
        let config = ParallelConfig::default();
        let processor = ParallelProcessor::new(config.clone());

        assert_eq!(processor.get_config().max_concurrent_tasks, config.max_concurrent_tasks);

        let stats = processor.get_stats().await;
        assert_eq!(stats.total_tasks, 0);
        assert_eq!(stats.completed_tasks, 0);
    }

    #[tokio::test]
    async fn test_resource_monitor() {
        let config = ParallelConfig {
            max_concurrent_tasks: 2,
            memory_limit_mb: 1, // 1MB limit
            ..Default::default()
        };

        let monitor = ResourceMonitor::new(config);

        // Should be able to start task initially
        assert!(monitor.can_start_task(Some(512 * 1024)).await); // 512KB

        // Start a task
        monitor.start_task(Some(512 * 1024)).await;

        // Should still be able to start another small task
        assert!(monitor.can_start_task(Some(256 * 1024)).await); // 256KB

        // Should not be able to start a large task that exceeds memory limit
        assert!(!monitor.can_start_task(Some(800 * 1024)).await); // 800KB (would exceed 1MB total)

        // Finish the task
        monitor.finish_task(Some(512 * 1024)).await;

        // Should be able to start large task now
        assert!(monitor.can_start_task(Some(800 * 1024)).await);
    }

    #[tokio::test]
    async fn test_task_scheduler() {
        let config = ParallelConfig::default();
        let scheduler = TaskScheduler::new(config);

        // Add tasks with different priorities
        let high_priority_task = TaskMetadata {
            id: "high".to_string(),
            priority: TaskPriority::High,
            estimated_duration: None,
            memory_requirement: None,
            dependencies: Vec::new(),
            group: None,
        };

        let low_priority_task = TaskMetadata {
            id: "low".to_string(),
            priority: TaskPriority::Low,
            estimated_duration: None,
            memory_requirement: None,
            dependencies: Vec::new(),
            group: None,
        };

        scheduler.add_task(low_priority_task, "low_task").await;
        scheduler.add_task(high_priority_task, "high_task").await;

        // Get next batch - should return high priority task first
        let batch = scheduler.get_next_batch(1).await;
        assert_eq!(batch.len(), 1);
        assert_eq!(batch[0].0.id, "high");

        // Remaining task should be low priority
        let batch = scheduler.get_next_batch(1).await;
        assert_eq!(batch.len(), 1);
        assert_eq!(batch[0].0.id, "low");
    }

    #[tokio::test]
    async fn test_parallel_file_processing() {
        let config = ParallelConfig {
            max_concurrent_tasks: 2,
            task_timeout_seconds: 5,
            max_retries: 1,
            ..Default::default()
        };

        let processor = ParallelProcessor::new(config);
        let counter = Arc::new(AtomicUsize::new(0));

        let files = vec![
            "file1.txt".to_string(),
            "file2.txt".to_string(),
            "file3.txt".to_string(),
        ];

        let counter_clone = counter.clone();
        let processor_fn = move |file_path: String| {
            let counter = counter_clone.clone();
            async move {
                counter.fetch_add(1, Ordering::SeqCst);
                tokio::time::sleep(Duration::from_millis(100)).await;
                Ok(format!("processed_{}", file_path))
            }
        };

        let results = processor.process_files_parallel(files, processor_fn).await;

        assert_eq!(results.len(), 3);
        assert_eq!(counter.load(Ordering::SeqCst), 3);

        // Check that all tasks succeeded
        for result in &results {
            assert!(result.result.is_ok());
            assert!(result.result.as_ref().unwrap().starts_with("processed_"));
        }

        let stats = processor.get_stats().await;
        assert_eq!(stats.completed_tasks, 3);
        assert_eq!(stats.failed_tasks, 0);
    }

    #[tokio::test]
    async fn test_batch_processing() {
        let config = ParallelConfig {
            batch_size: 2,
            max_concurrent_tasks: 2,
            ..Default::default()
        };

        let processor = ParallelProcessor::new(config);

        let items = vec![1, 2, 3, 4, 5];

        let processor_fn = move |batch: Vec<i32>| {
            async move {
                tokio::time::sleep(Duration::from_millis(50)).await;
                Ok(batch.into_iter().map(|x| x * 2).collect::<Vec<i32>>())
            }
        };

        let results = processor.process_batches_parallel(items, processor_fn).await;

        // Should create 3 batches: [1,2], [3,4], [5]
        assert_eq!(results.len(), 3);

        // Collect all processed items
        let mut all_processed = Vec::new();
        for result in results {
            assert!(result.result.is_ok());
            all_processed.extend(result.result.unwrap());
        }

        all_processed.sort();
        assert_eq!(all_processed, vec![2, 4, 6, 8, 10]);
    }

    #[tokio::test]
    async fn test_task_scheduler_enhancements() {
        let config = ParallelConfig::default();
        let scheduler = TaskScheduler::new(config);

        // Test adding multiple tasks
        let tasks = vec![
            (TaskMetadata {
                id: "task1".to_string(),
                priority: TaskPriority::High,
                estimated_duration: None,
                memory_requirement: Some(1024),
                dependencies: Vec::new(),
                group: Some("group1".to_string()),
            }, "data1"),
            (TaskMetadata {
                id: "task2".to_string(),
                priority: TaskPriority::Low,
                estimated_duration: None,
                memory_requirement: Some(2048),
                dependencies: Vec::new(),
                group: Some("group1".to_string()),
            }, "data2"),
            (TaskMetadata {
                id: "task3".to_string(),
                priority: TaskPriority::Normal,
                estimated_duration: None,
                memory_requirement: None,
                dependencies: Vec::new(),
                group: Some("group2".to_string()),
            }, "data3"),
        ];

        scheduler.add_tasks(tasks).await;
        assert_eq!(scheduler.pending_count().await, 3);

        // Test getting tasks by group
        let group1_tasks = scheduler.get_tasks_by_group("group1").await;
        assert_eq!(group1_tasks.len(), 2);

        // Test getting pending groups
        let groups = scheduler.get_pending_groups().await;
        assert!(groups.contains(&"group2".to_string()));

        // Test getting tasks by priority
        let normal_tasks = scheduler.get_tasks_by_priority(TaskPriority::Normal).await;
        assert_eq!(normal_tasks.len(), 1);
        assert_eq!(normal_tasks[0].0.id, "task3");
    }

    #[tokio::test]
    async fn test_optimal_batch_selection() {
        let config = ParallelConfig {
            batch_size: 2,
            memory_limit_mb: 1, // 1MB limit
            ..Default::default()
        };

        let scheduler = TaskScheduler::new(config.clone());
        let resource_monitor = ResourceMonitor::new(config);

        // Add tasks with different memory requirements
        let tasks = vec![
            (TaskMetadata {
                id: "small_task".to_string(),
                priority: TaskPriority::High,
                estimated_duration: None,
                memory_requirement: Some(256 * 1024), // 256KB
                dependencies: Vec::new(),
                group: None,
            }, "small_data"),
            (TaskMetadata {
                id: "large_task".to_string(),
                priority: TaskPriority::High,
                estimated_duration: None,
                memory_requirement: Some(800 * 1024), // 800KB
                dependencies: Vec::new(),
                group: None,
            }, "large_data"),
            (TaskMetadata {
                id: "medium_task".to_string(),
                priority: TaskPriority::Normal,
                estimated_duration: None,
                memory_requirement: Some(512 * 1024), // 512KB
                dependencies: Vec::new(),
                group: None,
            }, "medium_data"),
        ];

        scheduler.add_tasks(tasks).await;

        // Get optimal batch - should select tasks that fit within memory limit
        let batch = scheduler.get_optimal_batch(&resource_monitor).await;

        // Should get small_task and medium_task (total 768KB < 1MB)
        // but not large_task (would exceed limit)
        assert!(batch.len() <= 2);

        let total_memory: usize = batch.iter()
            .map(|(metadata, _)| metadata.memory_requirement.unwrap_or(0))
            .sum();

        assert!(total_memory <= 1024 * 1024); // Should not exceed 1MB
    }

    #[tokio::test]
    async fn test_scheduled_parallel_processing() {
        let config = ParallelConfig {
            max_concurrent_tasks: 2,
            batch_size: 2,
            ..Default::default()
        };

        let processor = ParallelProcessor::new(config);
        let counter = Arc::new(AtomicUsize::new(0));

        // Create tasks with metadata
        let tasks = vec![
            (TaskMetadata {
                id: "scheduled_task_1".to_string(),
                priority: TaskPriority::High,
                estimated_duration: None,
                memory_requirement: None,
                dependencies: Vec::new(),
                group: Some("test_group".to_string()),
            }, "data1".to_string()),
            (TaskMetadata {
                id: "scheduled_task_2".to_string(),
                priority: TaskPriority::Normal,
                estimated_duration: None,
                memory_requirement: None,
                dependencies: Vec::new(),
                group: Some("test_group".to_string()),
            }, "data2".to_string()),
        ];

        let counter_clone = counter.clone();
        let processor_fn = move |data: String| {
            let counter = counter_clone.clone();
            async move {
                counter.fetch_add(1, Ordering::SeqCst);
                tokio::time::sleep(Duration::from_millis(50)).await;
                Ok(format!("processed_{}", data))
            }
        };

        let results = processor.process_with_scheduler(tasks, processor_fn).await;

        assert_eq!(results.len(), 2);
        assert_eq!(counter.load(Ordering::SeqCst), 2);

        // Check that all tasks succeeded
        for result in &results {
            assert!(result.result.is_ok());
            assert!(result.result.as_ref().unwrap().starts_with("processed_"));
        }
    }

    #[tokio::test]
    async fn test_group_based_processing() {
        let config = ParallelConfig {
            max_concurrent_tasks: 3,
            batch_size: 2,
            ..Default::default()
        };

        let processor = ParallelProcessor::new(config);

        // Create tasks with different groups
        let tasks = vec![
            (TaskMetadata {
                id: "group1_task1".to_string(),
                priority: TaskPriority::High,
                estimated_duration: None,
                memory_requirement: None,
                dependencies: Vec::new(),
                group: Some("group1".to_string()),
            }, 1),
            (TaskMetadata {
                id: "group1_task2".to_string(),
                priority: TaskPriority::Normal,
                estimated_duration: None,
                memory_requirement: None,
                dependencies: Vec::new(),
                group: Some("group1".to_string()),
            }, 2),
            (TaskMetadata {
                id: "group2_task1".to_string(),
                priority: TaskPriority::High,
                estimated_duration: None,
                memory_requirement: None,
                dependencies: Vec::new(),
                group: Some("group2".to_string()),
            }, 3),
        ];

        let processor_fn = move |data: i32| {
            async move {
                tokio::time::sleep(Duration::from_millis(30)).await;
                Ok(data * 2)
            }
        };

        let group_results = processor.process_by_groups(tasks, processor_fn).await;

        // Should have results for both groups
        assert!(group_results.contains_key("group1"));
        assert!(group_results.contains_key("group2"));

        // Group1 should have 2 results, Group2 should have 1
        assert_eq!(group_results["group1"].len(), 2);
        assert_eq!(group_results["group2"].len(), 1);

        // Check results
        for result in &group_results["group1"] {
            assert!(result.result.is_ok());
        }
        for result in &group_results["group2"] {
            assert!(result.result.is_ok());
        }
    }

    #[tokio::test]
    async fn test_task_retry_mechanism() {
        let config = ParallelConfig {
            max_retries: 2,
            retry_delay_ms: 10, // Short delay for testing
            task_timeout_seconds: 1,
            ..Default::default()
        };

        let processor = ParallelProcessor::new(config);
        let attempt_counter = Arc::new(AtomicUsize::new(0));

        let files = vec!["failing_file.txt".to_string()];

        let counter_clone = attempt_counter.clone();
        let processor_fn = move |_file_path: String| {
            let counter = counter_clone.clone();
            async move {
                let attempt = counter.fetch_add(1, Ordering::SeqCst);
                if attempt < 2 {
                    // Fail first two attempts
                    anyhow::bail!("Simulated failure on attempt {}", attempt + 1);
                } else {
                    // Succeed on third attempt
                    Ok("success".to_string())
                }
            }
        };

        let results = processor.process_files_parallel(files, processor_fn).await;

        assert_eq!(results.len(), 1);
        assert!(results[0].result.is_ok());
        assert_eq!(results[0].retry_count, 2);
        assert_eq!(attempt_counter.load(Ordering::SeqCst), 3);
    }
}