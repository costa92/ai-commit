use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

use crate::messaging::events::{ReportEvent, ReportEventType, EventProcessingResult, EventStatus};
use crate::messaging::event_bus::{EventBus, EventHandler};

/// Async report processing request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsyncReportRequest {
    /// Unique request ID
    pub request_id: String,

    /// Request type
    pub request_type: AsyncRequestType,

    /// Request payload
    pub payload: serde_json::Value,

    /// Request timestamp
    pub created_at: DateTime<Utc>,

    /// Request priority
    pub priority: RequestPriority,

    /// Callback configuration
    pub callback: Option<CallbackConfig>,

    /// Request metadata
    pub metadata: HashMap<String, String>,
}

/// Types of async requests
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AsyncRequestType {
    /// Generate a code review report
    GenerateReport,

    /// Process analysis results
    ProcessAnalysis,

    /// Send notifications
    SendNotifications,

    /// Update storage
    UpdateStorage,

    /// Custom processing
    Custom(String),
}

/// Request priority levels
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RequestPriority {
    Low = 1,
    Normal = 2,
    High = 3,
    Critical = 4,
}

/// Callback configuration for async requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallbackConfig {
    /// Callback URL or identifier
    pub target: String,

    /// Callback type
    pub callback_type: CallbackType,

    /// Retry configuration
    pub retry_config: Option<CallbackRetryConfig>,
}

/// Types of callbacks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CallbackType {
    /// HTTP webhook
    HttpWebhook,

    /// Message queue
    MessageQueue,

    /// Event bus
    EventBus,

    /// Custom callback
    Custom(String),
}

/// Callback retry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallbackRetryConfig {
    /// Maximum retry attempts
    pub max_retries: u32,

    /// Initial delay in milliseconds
    pub initial_delay_ms: u64,

    /// Backoff multiplier
    pub backoff_multiplier: f64,
}

/// Async report processing result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsyncReportResult {
    /// Request ID
    pub request_id: String,

    /// Processing status
    pub status: ProcessingStatus,

    /// Result data
    pub result: Option<serde_json::Value>,

    /// Error message if failed
    pub error_message: Option<String>,

    /// Processing timestamps
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,

    /// Processing duration in milliseconds
    pub duration_ms: Option<u64>,

    /// Retry count
    pub retry_count: u32,
}

/// Processing status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ProcessingStatus {
    /// Request is queued for processing
    Queued,

    /// Request is being processed
    Processing,

    /// Request completed successfully
    Completed,

    /// Request failed
    Failed,

    /// Request was cancelled
    Cancelled,

    /// Request is being retried
    Retrying,
}

/// Async report processor trait
#[async_trait]
pub trait AsyncReportProcessor: Send + Sync {
    /// Process an async report request
    async fn process_request(&self, request: AsyncReportRequest) -> Result<AsyncReportResult>;

    /// Get processor name
    fn processor_name(&self) -> &str;

    /// Get supported request types
    fn supported_request_types(&self) -> Vec<AsyncRequestType>;

    /// Check if processor can handle the request
    fn can_handle(&self, request_type: &AsyncRequestType) -> bool {
        self.supported_request_types().contains(request_type)
    }
}

/// Async report processing manager
pub struct AsyncReportManager {
    /// Registered processors
    processors: Arc<RwLock<HashMap<AsyncRequestType, Arc<dyn AsyncReportProcessor>>>>,

    /// Request queue
    request_sender: mpsc::UnboundedSender<AsyncReportRequest>,
    request_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<AsyncReportRequest>>>>,

    /// Result storage
    results: Arc<RwLock<HashMap<String, AsyncReportResult>>>,

    /// Event bus for notifications
    event_bus: Option<Arc<EventBus>>,

    /// Processing configuration
    config: AsyncProcessingConfig,

    /// Processing statistics
    stats: Arc<RwLock<ProcessingStatistics>>,
}

/// Configuration for async processing
#[derive(Debug, Clone)]
pub struct AsyncProcessingConfig {
    /// Maximum concurrent workers
    pub max_workers: usize,

    /// Request timeout in seconds
    pub request_timeout_secs: u64,

    /// Maximum retry attempts
    pub max_retries: u32,

    /// Enable result persistence
    pub persist_results: bool,

    /// Result retention period in hours
    pub result_retention_hours: u64,
}

/// Processing statistics
#[derive(Debug, Clone, Default)]
pub struct ProcessingStatistics {
    /// Total requests processed
    pub total_requests: u64,

    /// Successful requests
    pub successful_requests: u64,

    /// Failed requests
    pub failed_requests: u64,

    /// Average processing time in milliseconds
    pub avg_processing_time_ms: f64,

    /// Current queue size
    pub current_queue_size: usize,

    /// Active workers
    pub active_workers: usize,
}

impl AsyncReportManager {
    /// Create a new async report manager
    pub fn new(config: AsyncProcessingConfig, event_bus: Option<Arc<EventBus>>) -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();

        Self {
            processors: Arc::new(RwLock::new(HashMap::new())),
            request_sender: sender,
            request_receiver: Arc::new(RwLock::new(Some(receiver))),
            results: Arc::new(RwLock::new(HashMap::new())),
            event_bus,
            config,
            stats: Arc::new(RwLock::new(ProcessingStatistics::default())),
        }
    }

    /// Register an async report processor
    pub async fn register_processor(&self, processor: Arc<dyn AsyncReportProcessor>) -> Result<()> {
        let mut processors = self.processors.write().await;

        for request_type in processor.supported_request_types() {
            processors.insert(request_type, processor.clone());
        }

        log::info!("Registered async processor: {}", processor.processor_name());
        Ok(())
    }

    /// Submit an async report request
    pub async fn submit_request(&self, request: AsyncReportRequest) -> Result<String> {
        let request_id = request.request_id.clone();

        // Create initial result entry
        let result = AsyncReportResult {
            request_id: request_id.clone(),
            status: ProcessingStatus::Queued,
            result: None,
            error_message: None,
            started_at: Utc::now(),
            completed_at: None,
            duration_ms: None,
            retry_count: 0,
        };

        {
            let mut results = self.results.write().await;
            results.insert(request_id.clone(), result);
        }

        // Send request to processing queue
        self.request_sender.send(request)
            .map_err(|e| anyhow::anyhow!("Failed to queue request: {}", e))?;

        // Update statistics
        {
            let mut stats = self.stats.write().await;
            stats.total_requests += 1;
            stats.current_queue_size += 1;
        }

        log::debug!("Submitted async request: {}", request_id);
        Ok(request_id)
    }

    /// Get request result
    pub async fn get_result(&self, request_id: &str) -> Option<AsyncReportResult> {
        let results = self.results.read().await;
        results.get(request_id).cloned()
    }

    /// Start processing workers
    pub async fn start_workers(&self) -> Result<()> {
        let mut receiver_guard = self.request_receiver.write().await;
        let receiver = receiver_guard.take()
            .ok_or_else(|| anyhow::anyhow!("Workers already started"))?;

        // Create shared receiver
        let shared_receiver = Arc::new(tokio::sync::Mutex::new(receiver));

        // Spawn worker tasks
        for worker_id in 0..self.config.max_workers {
            let receiver = shared_receiver.clone();
            let processors = self.processors.clone();
            let results = self.results.clone();
            let event_bus = self.event_bus.clone();
            let config = self.config.clone();
            let stats = self.stats.clone();

            tokio::spawn(async move {
                Self::worker_loop(
                    worker_id,
                    receiver,
                    processors,
                    results,
                    event_bus,
                    config,
                    stats,
                ).await;
            });
        }

        log::info!("Started {} async processing workers", self.config.max_workers);
        Ok(())
    }

    /// Worker loop for processing requests
    async fn worker_loop(
        worker_id: usize,
        receiver: Arc<tokio::sync::Mutex<mpsc::UnboundedReceiver<AsyncReportRequest>>>,
        processors: Arc<RwLock<HashMap<AsyncRequestType, Arc<dyn AsyncReportProcessor>>>>,
        results: Arc<RwLock<HashMap<String, AsyncReportResult>>>,
        event_bus: Option<Arc<EventBus>>,
        config: AsyncProcessingConfig,
        stats: Arc<RwLock<ProcessingStatistics>>,
    ) {
        log::info!("Worker {} started", worker_id);

        loop {
            // Try to receive a request
            let request = {
                let mut receiver_guard = receiver.lock().await;
                receiver_guard.recv().await
            };

            match request {
                Some(request) => {
                    let start_time = std::time::Instant::now();

                    // Update statistics
                    {
                        let mut stats_guard = stats.write().await;
                        stats_guard.active_workers += 1;
                        stats_guard.current_queue_size = stats_guard.current_queue_size.saturating_sub(1);
                    }

                    // Process the request
                    let result = Self::process_single_request(
                        &request,
                        &processors,
                        &config,
                    ).await;

                    let processing_duration = start_time.elapsed();

                    // Update result
                    {
                        let mut results_guard = results.write().await;
                        if let Some(stored_result) = results_guard.get_mut(&request.request_id) {
                            stored_result.status = result.status.clone();
                            stored_result.result = result.result.clone();
                            stored_result.error_message = result.error_message.clone();
                            stored_result.completed_at = Some(Utc::now());
                            stored_result.duration_ms = Some(processing_duration.as_millis() as u64);
                            stored_result.retry_count = result.retry_count;
                        }
                    }

                    // Update statistics
                    {
                        let mut stats_guard = stats.write().await;
                        stats_guard.active_workers = stats_guard.active_workers.saturating_sub(1);

                        match result.status {
                            ProcessingStatus::Completed => stats_guard.successful_requests += 1,
                            ProcessingStatus::Failed => stats_guard.failed_requests += 1,
                            _ => {}
                        }

                        // Update average processing time
                        let total_processed = stats_guard.successful_requests + stats_guard.failed_requests;
                        if total_processed > 0 {
                            let current_avg = stats_guard.avg_processing_time_ms;
                            let new_duration = processing_duration.as_millis() as f64;
                            stats_guard.avg_processing_time_ms =
                                (current_avg * (total_processed - 1) as f64 + new_duration) / total_processed as f64;
                        }
                    }

                    // Publish event if event bus is available
                    if let Some(event_bus) = &event_bus {
                        let event = ReportEvent::new(
                            ReportEventType::AnalysisCompleted,
                            request.metadata.get("project_path").cloned().unwrap_or_default(),
                            serde_json::to_value(&result).unwrap_or_default(),
                        );

                        if let Err(e) = event_bus.publish(event).await {
                            log::warn!("Failed to publish processing result event: {}", e);
                        }
                    }

                    log::debug!(
                        "Worker {} processed request {} in {:?} (status: {:?})",
                        worker_id,
                        request.request_id,
                        processing_duration,
                        result.status
                    );
                }
                None => {
                    log::info!("Worker {} shutting down - channel closed", worker_id);
                    break;
                }
            }
        }
    }

    /// Process a single request
    async fn process_single_request(
        request: &AsyncReportRequest,
        processors: &Arc<RwLock<HashMap<AsyncRequestType, Arc<dyn AsyncReportProcessor>>>>,
        config: &AsyncProcessingConfig,
    ) -> AsyncReportResult {
        let processors_guard = processors.read().await;

        if let Some(processor) = processors_guard.get(&request.request_type) {
            // Process with timeout
            let timeout_duration = std::time::Duration::from_secs(config.request_timeout_secs);

            match tokio::time::timeout(timeout_duration, processor.process_request(request.clone())).await {
                Ok(Ok(result)) => result,
                Ok(Err(e)) => AsyncReportResult {
                    request_id: request.request_id.clone(),
                    status: ProcessingStatus::Failed,
                    result: None,
                    error_message: Some(e.to_string()),
                    started_at: request.created_at,
                    completed_at: Some(Utc::now()),
                    duration_ms: None,
                    retry_count: 0,
                },
                Err(_) => AsyncReportResult {
                    request_id: request.request_id.clone(),
                    status: ProcessingStatus::Failed,
                    result: None,
                    error_message: Some("Request timeout".to_string()),
                    started_at: request.created_at,
                    completed_at: Some(Utc::now()),
                    duration_ms: None,
                    retry_count: 0,
                },
            }
        } else {
            AsyncReportResult {
                request_id: request.request_id.clone(),
                status: ProcessingStatus::Failed,
                result: None,
                error_message: Some(format!("No processor found for request type: {:?}", request.request_type)),
                started_at: request.created_at,
                completed_at: Some(Utc::now()),
                duration_ms: None,
                retry_count: 0,
            }
        }
    }

    /// Get processing statistics
    pub async fn get_statistics(&self) -> ProcessingStatistics {
        self.stats.read().await.clone()
    }

    /// Cleanup old results
    pub async fn cleanup_old_results(&self) -> Result<usize> {
        let cutoff_time = Utc::now() - chrono::Duration::hours(self.config.result_retention_hours as i64);
        let mut results = self.results.write().await;

        let initial_count = results.len();
        results.retain(|_, result| {
            result.started_at > cutoff_time
        });

        let cleaned_count = initial_count - results.len();
        log::debug!("Cleaned up {} old async processing results", cleaned_count);

        Ok(cleaned_count)
    }
}

impl Default for AsyncProcessingConfig {
    fn default() -> Self {
        Self {
            max_workers: num_cpus::get(),
            request_timeout_secs: 300, // 5 minutes
            max_retries: 3,
            persist_results: true,
            result_retention_hours: 24,
        }
    }
}

/// Example processor for generating reports
pub struct ReportGenerationProcessor {
    name: String,
}

impl ReportGenerationProcessor {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

#[async_trait]
impl AsyncReportProcessor for ReportGenerationProcessor {
    async fn process_request(&self, request: AsyncReportRequest) -> Result<AsyncReportResult> {
        log::info!("Processing report generation request: {}", request.request_id);

        // Simulate report generation work
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        let result_data = serde_json::json!({
            "report_id": format!("report_{}", request.request_id),
            "generated_at": Utc::now(),
            "status": "completed"
        });

        Ok(AsyncReportResult {
            request_id: request.request_id,
            status: ProcessingStatus::Completed,
            result: Some(result_data),
            error_message: None,
            started_at: request.created_at,
            completed_at: Some(Utc::now()),
            duration_ms: Some(100),
            retry_count: 0,
        })
    }

    fn processor_name(&self) -> &str {
        &self.name
    }

    fn supported_request_types(&self) -> Vec<AsyncRequestType> {
        vec![AsyncRequestType::GenerateReport]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_async_report_manager_creation() {
        let config = AsyncProcessingConfig::default();
        let manager = AsyncReportManager::new(config, None);

        let stats = manager.get_statistics().await;
        assert_eq!(stats.total_requests, 0);
        assert_eq!(stats.active_workers, 0);
    }

    #[tokio::test]
    async fn test_processor_registration() {
        let config = AsyncProcessingConfig::default();
        let manager = AsyncReportManager::new(config, None);

        let processor = Arc::new(ReportGenerationProcessor::new("test-processor".to_string()));
        manager.register_processor(processor).await.unwrap();

        // Verify processor is registered by checking if it can handle the request type
        let processors = manager.processors.read().await;
        assert!(processors.contains_key(&AsyncRequestType::GenerateReport));
    }

    #[tokio::test]
    async fn test_request_submission() {
        let config = AsyncProcessingConfig::default();
        let manager = AsyncReportManager::new(config, None);

        let request = AsyncReportRequest {
            request_id: "test-123".to_string(),
            request_type: AsyncRequestType::GenerateReport,
            payload: serde_json::json!({"test": "data"}),
            created_at: Utc::now(),
            priority: RequestPriority::Normal,
            callback: None,
            metadata: HashMap::new(),
        };

        let request_id = manager.submit_request(request).await.unwrap();
        assert_eq!(request_id, "test-123");

        // Check that result was created
        let result = manager.get_result(&request_id).await;
        assert!(result.is_some());
        assert_eq!(result.unwrap().status, ProcessingStatus::Queued);
    }

    #[tokio::test]
    async fn test_async_processing() {
        let config = AsyncProcessingConfig {
            max_workers: 1,
            ..Default::default()
        };
        let manager = AsyncReportManager::new(config, None);

        // Register processor
        let processor = Arc::new(ReportGenerationProcessor::new("test-processor".to_string()));
        manager.register_processor(processor).await.unwrap();

        // Start workers
        manager.start_workers().await.unwrap();

        // Submit request
        let request = AsyncReportRequest {
            request_id: "test-456".to_string(),
            request_type: AsyncRequestType::GenerateReport,
            payload: serde_json::json!({"test": "data"}),
            created_at: Utc::now(),
            priority: RequestPriority::Normal,
            callback: None,
            metadata: HashMap::new(),
        };

        let request_id = manager.submit_request(request).await.unwrap();

        // Wait for processing
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;

        // Check result
        let result = manager.get_result(&request_id).await;
        assert!(result.is_some());

        let result = result.unwrap();
        assert_eq!(result.status, ProcessingStatus::Completed);
        assert!(result.result.is_some());
        assert!(result.completed_at.is_some());
    }
}