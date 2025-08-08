use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use anyhow::Result;
use async_trait::async_trait;

use crate::messaging::events::{ReportEvent, ReportEventType, EventProcessingResult, EventStatus};

/// Event handler trait for processing events
#[async_trait]
pub trait EventHandler: Send + Sync {
    /// Handle a report event
    async fn handle_event(&self, event: ReportEvent) -> Result<EventProcessingResult>;

    /// Get the event types this handler is interested in
    fn interested_event_types(&self) -> Vec<ReportEventType>;

    /// Get handler name for logging and debugging
    fn handler_name(&self) -> &str;
}

/// Event bus for publishing and subscribing to events
pub struct EventBus {
    /// Event publishers for different event types
    publishers: Arc<RwLock<HashMap<ReportEventType, broadcast::Sender<ReportEvent>>>>,

    /// Registered event handlers
    handlers: Arc<RwLock<Vec<Arc<dyn EventHandler>>>>,

    /// Event routing configuration
    routing_config: EventRoutingConfig,
}

/// Configuration for event routing
#[derive(Debug, Clone)]
pub struct EventRoutingConfig {
    /// Buffer size for event channels
    pub channel_buffer_size: usize,

    /// Whether to enable event persistence
    pub enable_persistence: bool,

    /// Maximum retry attempts for failed events
    pub max_retry_attempts: u32,

    /// Retry delay in milliseconds
    pub retry_delay_ms: u64,
}

/// Event subscription handle
pub struct EventSubscription {
    /// Event type being subscribed to
    pub event_type: ReportEventType,

    /// Receiver for events
    pub receiver: broadcast::Receiver<ReportEvent>,

    /// Handler for processing events
    pub handler: Arc<dyn EventHandler>,
}

impl EventBus {
    /// Create a new event bus
    pub fn new(config: EventRoutingConfig) -> Self {
        Self {
            publishers: Arc::new(RwLock::new(HashMap::new())),
            handlers: Arc::new(RwLock::new(Vec::new())),
            routing_config: config,
        }
    }

    /// Publish an event to all subscribers
    pub async fn publish(&self, event: ReportEvent) -> Result<usize> {
        let event_type = event.event_type.clone();
        let mut publishers = self.publishers.write().await;

        // Create publisher if it doesn't exist
        if !publishers.contains_key(&event_type) {
            let (sender, _) = broadcast::channel(self.routing_config.channel_buffer_size);
            publishers.insert(event_type.clone(), sender);
        }

        // Send event to all subscribers
        if let Some(sender) = publishers.get(&event_type) {
            match sender.send(event.clone()) {
                Ok(subscriber_count) => {
                    log::debug!("Published event {} to {} subscribers", event.event_id, subscriber_count);
                    Ok(subscriber_count)
                }
                Err(e) => {
                    log::error!("Failed to publish event {}: {}", event.event_id, e);
                    anyhow::bail!("Failed to publish event: {}", e)
                }
            }
        } else {
            Ok(0)
        }
    }

    /// Subscribe to events of a specific type
    pub async fn subscribe(&self, event_type: ReportEventType) -> Result<broadcast::Receiver<ReportEvent>> {
        let mut publishers = self.publishers.write().await;

        // Create publisher if it doesn't exist
        if !publishers.contains_key(&event_type) {
            let (sender, _) = broadcast::channel(self.routing_config.channel_buffer_size);
            publishers.insert(event_type.clone(), sender);
        }

        // Get receiver for the event type
        if let Some(sender) = publishers.get(&event_type) {
            Ok(sender.subscribe())
        } else {
            anyhow::bail!("Failed to create subscription for event type: {:?}", event_type)
        }
    }

    /// Register an event handler
    pub async fn register_handler(&self, handler: Arc<dyn EventHandler>) -> Result<()> {
        let mut handlers = self.handlers.write().await;
        handlers.push(handler.clone());

        // Subscribe the handler to its interested event types
        for event_type in handler.interested_event_types() {
            let receiver = self.subscribe(event_type.clone()).await?;

            // Spawn a task to handle events for this handler
            let handler_clone = handler.clone();
            let routing_config = self.routing_config.clone();

            tokio::spawn(async move {
                Self::handle_events_for_handler(handler_clone, receiver, routing_config).await;
            });
        }

        log::info!("Registered event handler: {}", handler.handler_name());
        Ok(())
    }

    /// Handle events for a specific handler
    async fn handle_events_for_handler(
        handler: Arc<dyn EventHandler>,
        mut receiver: broadcast::Receiver<ReportEvent>,
        config: EventRoutingConfig,
    ) {
        loop {
            match receiver.recv().await {
                Ok(event) => {
                    let start_time = std::time::Instant::now();
                    let mut retry_count = 0;

                    loop {
                        match handler.handle_event(event.clone()).await {
                            Ok(result) => {
                                let duration = start_time.elapsed();
                                log::debug!(
                                    "Handler '{}' processed event {} in {:?} (status: {:?})",
                                    handler.handler_name(),
                                    event.event_id,
                                    duration,
                                    result.status
                                );
                                break;
                            }
                            Err(e) => {
                                retry_count += 1;
                                log::warn!(
                                    "Handler '{}' failed to process event {} (attempt {}/{}): {}",
                                    handler.handler_name(),
                                    event.event_id,
                                    retry_count,
                                    config.max_retry_attempts,
                                    e
                                );

                                if retry_count >= config.max_retry_attempts {
                                    log::error!(
                                        "Handler '{}' failed to process event {} after {} attempts",
                                        handler.handler_name(),
                                        event.event_id,
                                        retry_count
                                    );
                                    break;
                                }

                                // Wait before retrying
                                tokio::time::sleep(std::time::Duration::from_millis(config.retry_delay_ms)).await;
                            }
                        }
                    }
                }
                Err(broadcast::error::RecvError::Lagged(skipped)) => {
                    log::warn!(
                        "Handler '{}' lagged behind, skipped {} events",
                        handler.handler_name(),
                        skipped
                    );
                }
                Err(broadcast::error::RecvError::Closed) => {
                    log::info!("Event channel closed for handler '{}'", handler.handler_name());
                    break;
                }
            }
        }
    }

    /// Get statistics about the event bus
    pub async fn get_statistics(&self) -> EventBusStatistics {
        let publishers = self.publishers.read().await;
        let handlers = self.handlers.read().await;

        let mut event_type_counts = HashMap::new();
        for (event_type, sender) in publishers.iter() {
            event_type_counts.insert(event_type.clone(), sender.receiver_count());
        }

        EventBusStatistics {
            total_event_types: publishers.len(),
            total_handlers: handlers.len(),
            event_type_subscriber_counts: event_type_counts,
        }
    }

    /// Shutdown the event bus
    pub async fn shutdown(&self) -> Result<()> {
        let mut publishers = self.publishers.write().await;
        publishers.clear();

        let mut handlers = self.handlers.write().await;
        handlers.clear();

        log::info!("Event bus shutdown completed");
        Ok(())
    }
}

/// Statistics about the event bus
#[derive(Debug, Clone)]
pub struct EventBusStatistics {
    /// Total number of event types
    pub total_event_types: usize,

    /// Total number of registered handlers
    pub total_handlers: usize,

    /// Number of subscribers for each event type
    pub event_type_subscriber_counts: HashMap<ReportEventType, usize>,
}

impl Default for EventRoutingConfig {
    fn default() -> Self {
        Self {
            channel_buffer_size: 1000,
            enable_persistence: false,
            max_retry_attempts: 3,
            retry_delay_ms: 1000,
        }
    }
}

/// Example event handler for logging events
pub struct LoggingEventHandler {
    name: String,
    interested_types: Vec<ReportEventType>,
}

impl LoggingEventHandler {
    pub fn new(name: String, interested_types: Vec<ReportEventType>) -> Self {
        Self {
            name,
            interested_types,
        }
    }
}

#[async_trait]
impl EventHandler for LoggingEventHandler {
    async fn handle_event(&self, event: ReportEvent) -> Result<EventProcessingResult> {
        log::info!(
            "LoggingEventHandler '{}' received event: {} (type: {:?}, project: {})",
            self.name,
            event.event_id,
            event.event_type,
            event.project_path
        );

        Ok(EventProcessingResult {
            event_id: event.event_id,
            status: EventStatus::Completed,
            processed_at: chrono::Utc::now(),
            duration_ms: 1, // Minimal processing time for logging
            error_message: None,
            retry_count: 0,
        })
    }

    fn interested_event_types(&self) -> Vec<ReportEventType> {
        self.interested_types.clone()
    }

    fn handler_name(&self) -> &str {
        &self.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::messaging::events::{ReportEvent, ReportEventType, EventMetadata};
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[tokio::test]
    async fn test_event_bus_creation() {
        let config = EventRoutingConfig::default();
        let event_bus = EventBus::new(config);

        let stats = event_bus.get_statistics().await;
        assert_eq!(stats.total_event_types, 0);
        assert_eq!(stats.total_handlers, 0);
    }

    #[tokio::test]
    async fn test_event_publishing_and_subscription() {
        let config = EventRoutingConfig::default();
        let event_bus = EventBus::new(config);

        // Subscribe to events
        let mut receiver = event_bus.subscribe(ReportEventType::ReportGenerated).await.unwrap();

        // Create and publish an event
        let event = ReportEvent::new(
            ReportEventType::ReportGenerated,
            "/test/project".to_string(),
            serde_json::json!({"test": "data"}),
        );

        let subscriber_count = event_bus.publish(event.clone()).await.unwrap();
        assert_eq!(subscriber_count, 1);

        // Receive the event
        let received_event = receiver.recv().await.unwrap();
        assert_eq!(received_event.event_id, event.event_id);
        assert_eq!(received_event.event_type, event.event_type);
    }

    #[tokio::test]
    async fn test_event_handler_registration() {
        let config = EventRoutingConfig::default();
        let event_bus = EventBus::new(config);

        // Create a test handler
        let handler = Arc::new(LoggingEventHandler::new(
            "test-handler".to_string(),
            vec![ReportEventType::ReportGenerated],
        ));

        // Register the handler
        event_bus.register_handler(handler).await.unwrap();

        let stats = event_bus.get_statistics().await;
        assert_eq!(stats.total_handlers, 1);
        assert_eq!(stats.total_event_types, 1);
    }

    #[tokio::test]
    async fn test_multiple_subscribers() {
        let config = EventRoutingConfig::default();
        let event_bus = EventBus::new(config);

        // Create multiple subscribers
        let _receiver1 = event_bus.subscribe(ReportEventType::ReportGenerated).await.unwrap();
        let _receiver2 = event_bus.subscribe(ReportEventType::ReportGenerated).await.unwrap();
        let _receiver3 = event_bus.subscribe(ReportEventType::SecurityIssueDetected).await.unwrap();

        // Publish an event
        let event = ReportEvent::new(
            ReportEventType::ReportGenerated,
            "/test/project".to_string(),
            serde_json::json!({"test": "data"}),
        );

        let subscriber_count = event_bus.publish(event).await.unwrap();
        assert_eq!(subscriber_count, 2); // Only 2 subscribers for ReportGenerated

        let stats = event_bus.get_statistics().await;
        assert_eq!(stats.total_event_types, 2);
    }

    /// Test handler that counts processed events
    struct CountingEventHandler {
        name: String,
        count: Arc<AtomicUsize>,
        interested_types: Vec<ReportEventType>,
    }

    impl CountingEventHandler {
        fn new(name: String, interested_types: Vec<ReportEventType>) -> Self {
            Self {
                name,
                count: Arc::new(AtomicUsize::new(0)),
                interested_types,
            }
        }

        fn get_count(&self) -> usize {
            self.count.load(Ordering::Relaxed)
        }
    }

    #[async_trait]
    impl EventHandler for CountingEventHandler {
        async fn handle_event(&self, event: ReportEvent) -> Result<EventProcessingResult> {
            self.count.fetch_add(1, Ordering::Relaxed);

            Ok(EventProcessingResult {
                event_id: event.event_id,
                status: EventStatus::Completed,
                processed_at: chrono::Utc::now(),
                duration_ms: 1,
                error_message: None,
                retry_count: 0,
            })
        }

        fn interested_event_types(&self) -> Vec<ReportEventType> {
            self.interested_types.clone()
        }

        fn handler_name(&self) -> &str {
            &self.name
        }
    }

    #[tokio::test]
    async fn test_event_handler_processing() {
        let config = EventRoutingConfig::default();
        let event_bus = EventBus::new(config);

        // Create a counting handler
        let handler = Arc::new(CountingEventHandler::new(
            "counting-handler".to_string(),
            vec![ReportEventType::ReportGenerated],
        ));

        let handler_clone = handler.clone();

        // Register the handler
        event_bus.register_handler(handler_clone).await.unwrap();

        // Give the handler time to set up
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        // Publish multiple events
        for i in 0..5 {
            let event = ReportEvent::new(
                ReportEventType::ReportGenerated,
                format!("/test/project/{}", i),
                serde_json::json!({"test": i}),
            );

            event_bus.publish(event).await.unwrap();
        }

        // Give handlers time to process events
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;

        // Check that all events were processed
        assert_eq!(handler.get_count(), 5);
    }
}