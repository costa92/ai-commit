pub mod markdown;
pub mod json;
pub mod text;

pub use markdown::MarkdownFormatter;
pub use json::JsonFormatter;
pub use text::TextFormatter;

use crate::models::review::CodeReviewReport;
use crate::report::config::ReportConfig;
use anyhow::Result;
use async_trait::async_trait;

/// 报告格式化器 trait
#[async_trait]
pub trait ReportFormatter: Send + Sync {
    /// 格式化报告
    async fn format(&self, report: &CodeReviewReport, config: &ReportConfig) -> Result<String>;

    /// 获取格式化器名称
    fn name(&self) -> &str;

    /// 获取支持的文件扩展名
    fn file_extension(&self) -> &str;
}

/// 格式化辅助函数
pub mod utils {
    use crate::models::review::{Severity, RiskLevel};

    /// 将严重程度转换为字符串
    pub fn severity_to_string(severity: &Severity) -> &'static str {
        match severity {
            Severity::Critical => "Critical",
            Severity::High => "High",
            Severity::Medium => "Medium",
            Severity::Low => "Low",
            Severity::Info => "Info",
        }
    }

    /// 将风险等级转换为字符串
    pub fn risk_level_to_string(risk_level: &RiskLevel) -> &'static str {
        match risk_level {
            RiskLevel::Critical => "Critical",
            RiskLevel::High => "High",
            RiskLevel::Medium => "Medium",
            RiskLevel::Low => "Low",
        }
    }

    /// 将严重程度转换为表情符号
    pub fn severity_to_emoji(severity: &Severity) -> &'static str {
        match severity {
            Severity::Critical => "🔴",
            Severity::High => "🟠",
            Severity::Medium => "🟡",
            Severity::Low => "🔵",
            Severity::Info => "ℹ️",
        }
    }

    /// 将风险等级转换为表情符号
    pub fn risk_level_to_emoji(risk_level: &RiskLevel) -> &'static str {
        match risk_level {
            RiskLevel::Critical => "🔴",
            RiskLevel::High => "🟠",
            RiskLevel::Medium => "🟡",
            RiskLevel::Low => "🔵",
        }
    }

    /// 格式化持续时间
    pub fn format_duration(duration: &std::time::Duration) -> String {
        let secs = duration.as_secs();
        if secs < 60 {
            format!("{}s", secs)
        } else if secs < 3600 {
            format!("{}m {}s", secs / 60, secs % 60)
        } else {
            format!("{}h {}m {}s", secs / 3600, (secs % 3600) / 60, secs % 60)
        }
    }

    /// 格式化文件大小
    pub fn format_file_size(size: usize) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
        let mut size = size as f64;
        let mut unit_index = 0;

        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }

        if unit_index == 0 {
            format!("{} {}", size as usize, UNITS[unit_index])
        } else {
            format!("{:.1} {}", size, UNITS[unit_index])
        }
    }

    /// 格式化百分比
    pub fn format_percentage(value: f32) -> String {
        format!("{:.1}%", value * 100.0)
    }

    /// 生成进度条
    pub fn generate_progress_bar(value: f32, width: usize) -> String {
        let filled = (value * width as f32) as usize;
        let empty = width - filled;
        format!("[{}{}]", "█".repeat(filled), "░".repeat(empty))
    }
}