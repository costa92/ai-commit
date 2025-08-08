use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Report event that can be sent through messaging system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportEvent {
    /// Unique event identifier
    pub event_id: String,

    /// Type of the event
    pub event_type: ReportEventType,

    /// Event timestamp
    pub timestamp: DateTime<Utc>,

    /// Project path where the event occurred
    pub project_path: String,

    /// Associated report ID (if applicable)
    pub report_id: Option<String>,

    /// Event metadata
    pub metadata: EventMetadata,

    /// Event payload data
    pub payload: serde_json::Value,
}

/// Types of report events that can be generated
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ReportEventType {
    /// A new report has been generated
    ReportGenerated,

    /// Code analysis has been completed
    AnalysisCompleted,

    /// Quality threshold has been exceeded
    QualityThresholdExceeded,

    /// Security issue has been detected
    SecurityIssueDetected,

    /// Performance regression has been detected
    PerformanceRegressionDetected,

    /// Dependency vulnerability detected
    DependencyVulnerabilityDetected,

    /// Code complexity threshold exceeded
    ComplexityThresholdExceeded,

    /// Test coverage dropped below threshold
    CoverageThresholdExceeded,

    /// Custom event type
    Custom(String),
}

/// Metadata associated with an event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMetadata {
    /// Source system that generated the event
    pub source: String,

    /// Version of the source system
    pub version: String,

    /// Correlation ID for tracing related events
    pub correlation_id: Option<String>,

    /// User ID who triggered the event
    pub user_id: Option<String>,

    /// Additional tags for categorization
    pub tags: HashMap<String, String>,
}

/// Event routing information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventRouting {
    /// Target topic for the event
    pub topic: String,

    /// Partition key (optional)
    pub partition_key: Option<String>,

    /// Message headers
    pub headers: HashMap<String, String>,
}

/// Event processing status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventStatus {
    /// Event is pending processing
    Pending,

    /// Event is being processed
    Processing,

    /// Event has been processed successfully
    Completed,

    /// Event processing failed
    Failed(String),

    /// Event processing was retried
    Retried(u32),
}

/// Event processing result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventProcessingResult {
    /// Event ID
    pub event_id: String,

    /// Processing status
    pub status: EventStatus,

    /// Processing timestamp
    pub processed_at: DateTime<Utc>,

    /// Processing duration in milliseconds
    pub duration_ms: u64,

    /// Error message (if failed)
    pub error_message: Option<String>,

    /// Retry count
    pub retry_count: u32,
}

impl ReportEvent {
    /// Create a new report event
    pub fn new(
        event_type: ReportEventType,
        project_path: String,
        payload: serde_json::Value,
    ) -> Self {
        Self {
            event_id: uuid::Uuid::new_v4().to_string(),
            event_type,
            timestamp: Utc::now(),
            project_path,
            report_id: None,
            metadata: EventMetadata::default(),
            payload,
        }
    }

    /// Create a report generated event
    pub fn report_generated(
        project_path: String,
        report_id: String,
        report_data: serde_json::Value,
    ) -> Self {
        let mut event = Self::new(
            ReportEventType::ReportGenerated,
            project_path,
            report_data,
        );
        event.report_id = Some(report_id);
        event
    }

    /// Create an analysis completed event
    pub fn analysis_completed(
        project_path: String,
        analysis_results: serde_json::Value,
    ) -> Self {
        Self::new(
            ReportEventType::AnalysisCompleted,
            project_path,
            analysis_results,
        )
    }

    /// Create a quality threshold exceeded event
    pub fn quality_threshold_exceeded(
        project_path: String,
        threshold_data: serde_json::Value,
    ) -> Self {
        Self::new(
            ReportEventType::QualityThresholdExceeded,
            project_path,
            threshold_data,
        )
    }

    /// Create a security issue detected event
    pub fn security_issue_detected(
        project_path: String,
        security_data: serde_json::Value,
    ) -> Self {
        Self::new(
            ReportEventType::SecurityIssueDetected,
            project_path,
            security_data,
        )
    }

    /// Create a performance regression detected event
    pub fn performance_regression_detected(
        project_path: String,
        performance_data: serde_json::Value,
    ) -> Self {
        Self::new(
            ReportEventType::PerformanceRegressionDetected,
            project_path,
            performance_data,
        )
    }

    /// Set correlation ID for event tracing
    pub fn with_correlation_id(mut self, correlation_id: String) -> Self {
        self.metadata.correlation_id = Some(correlation_id);
        self
    }

    /// Set user ID
    pub fn with_user_id(mut self, user_id: String) -> Self {
        self.metadata.user_id = Some(user_id);
        self
    }

    /// Add a tag
    pub fn with_tag(mut self, key: String, value: String) -> Self {
        self.metadata.tags.insert(key, value);
        self
    }

    /// Get event routing information
    pub fn get_routing(&self, topic_mappings: &HashMap<ReportEventType, String>) -> EventRouting {
        let topic = topic_mappings
            .get(&self.event_type)
            .cloned()
            .unwrap_or_else(|| "default".to_string());

        let partition_key = Some(self.project_path.clone());

        let mut headers = HashMap::new();
        headers.insert("event_type".to_string(), format!("{:?}", self.event_type));
        headers.insert("source".to_string(), self.metadata.source.clone());
        headers.insert("version".to_string(), self.metadata.version.clone());

        if let Some(correlation_id) = &self.metadata.correlation_id {
            headers.insert("correlation_id".to_string(), correlation_id.clone());
        }

        EventRouting {
            topic,
            partition_key,
            headers,
        }
    }
}

impl Default for EventMetadata {
    fn default() -> Self {
        Self {
            source: "ai-commit".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            correlation_id: None,
            user_id: None,
            tags: HashMap::new(),
        }
    }
}

impl std::fmt::Display for ReportEventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReportEventType::ReportGenerated => write!(f, "report_generated"),
            ReportEventType::AnalysisCompleted => write!(f, "analysis_completed"),
            ReportEventType::QualityThresholdExceeded => write!(f, "quality_threshold_exceeded"),
            ReportEventType::SecurityIssueDetected => write!(f, "security_issue_detected"),
            ReportEventType::PerformanceRegressionDetected => write!(f, "performance_regression_detected"),
            ReportEventType::DependencyVulnerabilityDetected => write!(f, "dependency_vulnerability_detected"),
            ReportEventType::ComplexityThresholdExceeded => write!(f, "complexity_threshold_exceeded"),
            ReportEventType::CoverageThresholdExceeded => write!(f, "coverage_threshold_exceeded"),
            ReportEventType::Custom(name) => write!(f, "custom_{}", name),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_report_event_creation() {
        let payload = serde_json::json!({"test": "data"});
        let event = ReportEvent::new(
            ReportEventType::ReportGenerated,
            "/test/project".to_string(),
            payload.clone(),
        );

        assert_eq!(event.event_type, ReportEventType::ReportGenerated);
        assert_eq!(event.project_path, "/test/project");
        assert_eq!(event.payload, payload);
        assert!(event.event_id.len() > 0);
    }

    #[test]
    fn test_event_with_metadata() {
        let event = ReportEvent::new(
            ReportEventType::AnalysisCompleted,
            "/test/project".to_string(),
            serde_json::json!({}),
        )
        .with_correlation_id("corr-123".to_string())
        .with_user_id("user-456".to_string())
        .with_tag("environment".to_string(), "production".to_string());

        assert_eq!(event.metadata.correlation_id, Some("corr-123".to_string()));
        assert_eq!(event.metadata.user_id, Some("user-456".to_string()));
        assert_eq!(event.metadata.tags.get("environment"), Some(&"production".to_string()));
    }

    #[test]
    fn test_event_routing() {
        let mut topic_mappings = HashMap::new();
        topic_mappings.insert(ReportEventType::ReportGenerated, "reports".to_string());

        let event = ReportEvent::new(
            ReportEventType::ReportGenerated,
            "/test/project".to_string(),
            serde_json::json!({}),
        );

        let routing = event.get_routing(&topic_mappings);
        assert_eq!(routing.topic, "reports");
        assert_eq!(routing.partition_key, Some("/test/project".to_string()));
        assert!(routing.headers.contains_key("event_type"));
    }

    #[test]
    fn test_event_type_display() {
        assert_eq!(ReportEventType::ReportGenerated.to_string(), "report_generated");
        assert_eq!(ReportEventType::SecurityIssueDetected.to_string(), "security_issue_detected");
        assert_eq!(ReportEventType::Custom("test".to_string()).to_string(), "custom_test");
    }
}