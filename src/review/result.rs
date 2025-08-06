use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

use crate::languages::Language;
use crate::models::CodeReviewReport;

/// 审查请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewRequest {
    pub project_path: String,
    pub files: Vec<String>,
    pub options: ReviewOptions,
    pub correlation_id: Option<String>,
    pub user_id: Option<String>,
    pub tags: HashMap<String, String>,
}

/// 审查选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewOptions {
    // 基础分析选项
    pub static_analysis: bool,
    pub ai_review: bool,
    pub sensitive_scan: bool,

    // 高级分析选项
    pub complexity_analysis: bool,
    pub duplication_scan: bool,
    pub dependency_scan: bool,
    pub coverage_analysis: bool,
    pub performance_analysis: bool,
    pub trend_analysis: bool,

    // AI 配置
    pub ai_provider: Option<String>,
    pub ai_model: Option<String>,

    // 报告选项
    pub report_format: ReportFormat,
    pub output_path: Option<String>,
    pub store_report: bool,

    // 通知选项
    pub enable_notifications: bool,
    pub notification_channels: Vec<String>,

    // 消息队列选项
    pub enable_messaging: bool,

    // 质量快照选项
    pub record_snapshot: bool,
}

/// 报告格式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReportFormat {
    Markdown,
    Json,
    Text,
    Html,
}

impl Default for ReportFormat {
    fn default() -> Self {
        ReportFormat::Markdown
    }
}

impl Default for ReviewOptions {
    fn default() -> Self {
        Self {
            static_analysis: true,
            ai_review: false,
            sensitive_scan: true,
            complexity_analysis: true,
            duplication_scan: false,
            dependency_scan: false,
            coverage_analysis: false,
            performance_analysis: false,
            trend_analysis: false,
            ai_provider: None,
            ai_model: None,
            report_format: ReportFormat::default(),
            output_path: None,
            store_report: false,
            enable_notifications: false,
            notification_channels: Vec::new(),
            enable_messaging: false,
            record_snapshot: false,
        }
    }
}

/// 审查结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewResult {
    pub report: CodeReviewReport,
    pub metadata: ReviewMetadata,
}

/// 审查元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewMetadata {
    pub duration: Duration,
    pub files_analyzed: usize,
    pub languages_detected: Vec<Language>,
    pub started_at: DateTime<Utc>,
    pub completed_at: DateTime<Utc>,
    pub success: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl ReviewMetadata {
    pub fn new(files_analyzed: usize, languages_detected: Vec<Language>) -> Self {
        let now = Utc::now();
        Self {
            duration: Duration::from_secs(0),
            files_analyzed,
            languages_detected,
            started_at: now,
            completed_at: now,
            success: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self.completed_at = self.started_at + chrono::Duration::from_std(duration).unwrap_or_default();
        self
    }

    pub fn with_error(mut self, error: String) -> Self {
        self.errors.push(error);
        self.success = false;
        self
    }

    pub fn with_warning(mut self, warning: String) -> Self {
        self.warnings.push(warning);
        self
    }
}