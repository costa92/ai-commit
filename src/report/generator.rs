use crate::models::review::CodeReviewReport;
use crate::report::config::{ReportConfig, ReportFormat};
use crate::report::formatters::{ReportFormatter, MarkdownFormatter, JsonFormatter, TextFormatter};
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use tokio::fs;

/// 报告生成器
pub struct ReportGenerator {
    config: ReportConfig,
}

impl ReportGenerator {
    /// 创建新的报告生成器
    pub fn new(config: ReportConfig) -> Self {
        Self { config }
    }

    /// 生成报告
    pub async fn generate(&self, report: &CodeReviewReport) -> Result<GeneratedReport> {
        let formatter = self.create_formatter()?;
        let content = formatter.format(report, &self.config).await?;

        let output_path = self.determine_output_path(report)?;

        // 确保输出目录存在
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent).await
                .with_context(|| format!("Failed to create output directory: {:?}", parent))?;
        }

        // 写入文件
        fs::write(&output_path, &content).await
            .with_context(|| format!("Failed to write report to: {:?}", output_path))?;

        Ok(GeneratedReport {
            path: output_path,
            format: self.config.format.clone(),
            size: content.len(),
            content,
        })
    }

    /// 生成报告内容（不写入文件）
    pub async fn generate_content(&self, report: &CodeReviewReport) -> Result<String> {
        let formatter = self.create_formatter()?;
        formatter.format(report, &self.config).await
    }

    /// 创建格式化器
    fn create_formatter(&self) -> Result<Box<dyn ReportFormatter>> {
        match self.config.format {
            ReportFormat::Markdown => Ok(Box::new(MarkdownFormatter::new())),
            ReportFormat::Json => Ok(Box::new(JsonFormatter::new())),
            ReportFormat::Text => Ok(Box::new(TextFormatter::new())),
            ReportFormat::Html => {
                anyhow::bail!("HTML format is not yet implemented")
            }
        }
    }

    /// 确定输出路径
    fn determine_output_path(&self, report: &CodeReviewReport) -> Result<PathBuf> {
        if let Some(ref path) = self.config.output_path {
            return Ok(path.clone());
        }

        // 默认输出路径
        let project_name = Path::new(&report.summary.project_path)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("unknown");

        let timestamp = report.summary.created_at.format("%Y%m%d_%H%M%S");
        let filename = format!("code-review-{}-{}.{}",
            project_name,
            timestamp,
            self.get_file_extension()
        );

        Ok(PathBuf::from("code-review").join(filename))
    }

    /// 获取文件扩展名
    fn get_file_extension(&self) -> &str {
        match self.config.format {
            ReportFormat::Markdown => "md",
            ReportFormat::Json => "json",
            ReportFormat::Text => "txt",
            ReportFormat::Html => "html",
        }
    }
}

/// 生成的报告
#[derive(Debug, Clone)]
pub struct GeneratedReport {
    /// 报告文件路径
    pub path: PathBuf,
    /// 报告格式
    pub format: ReportFormat,
    /// 文件大小（字节）
    pub size: usize,
    /// 报告内容
    pub content: String,
}

impl GeneratedReport {
    /// 获取报告摘要信息
    pub fn summary(&self) -> ReportSummary {
        ReportSummary {
            path: self.path.clone(),
            format: self.format.clone(),
            size: self.size,
            lines: self.content.lines().count(),
        }
    }
}

/// 报告摘要
#[derive(Debug, Clone)]
pub struct ReportSummary {
    pub path: PathBuf,
    pub format: ReportFormat,
    pub size: usize,
    pub lines: usize,
}

impl std::fmt::Display for ReportSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Report generated: {} ({} format, {} bytes, {} lines)",
            self.path.display(),
            self.format,
            self.size,
            self.lines
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::review::*;
    use chrono::Utc;
    use std::collections::HashMap;
    use tempfile::TempDir;

    fn create_test_report() -> CodeReviewReport {
        CodeReviewReport {
            summary: ReviewSummary {
                project_path: "/test/project".to_string(),
                files_analyzed: 5,
                languages_detected: vec!["Rust".to_string(), "Go".to_string()],
                total_issues: 10,
                critical_issues: 1,
                high_issues: 2,
                medium_issues: 4,
                low_issues: 3,
                analysis_duration: std::time::Duration::from_secs(30),
                created_at: Utc::now(),
            },
            static_analysis_results: vec![],
            ai_review_results: vec![],
            sensitive_info_results: vec![],
            complexity_results: vec![],
            duplication_results: vec![],
            dependency_results: None,
            coverage_results: None,
            performance_results: vec![],
            trend_results: None,
            overall_score: 7.5,
            recommendations: vec!["Fix critical issues".to_string()],
            metadata: ReviewMetadata {
                version: "1.0.0".to_string(),
                user_id: None,
                correlation_id: None,
                tags: HashMap::new(),
                configuration: ReviewConfiguration::default(),
            },
        }
    }

    #[tokio::test]
    async fn test_generate_markdown_report() {
        let temp_dir = TempDir::new().unwrap();
        let output_path = temp_dir.path().join("test-report.md");

        let config = ReportConfig {
            format: ReportFormat::Markdown,
            output_path: Some(output_path.clone()),
            ..Default::default()
        };

        let generator = ReportGenerator::new(config);
        let report = create_test_report();

        let result = generator.generate(&report).await.unwrap();

        assert_eq!(result.path, output_path);
        assert_eq!(result.format, ReportFormat::Markdown);
        assert!(result.size > 0);
        assert!(output_path.exists());
    }

    #[tokio::test]
    async fn test_generate_json_report() {
        let temp_dir = TempDir::new().unwrap();
        let output_path = temp_dir.path().join("test-report.json");

        let config = ReportConfig {
            format: ReportFormat::Json,
            output_path: Some(output_path.clone()),
            ..Default::default()
        };

        let generator = ReportGenerator::new(config);
        let report = create_test_report();

        let result = generator.generate(&report).await.unwrap();

        assert_eq!(result.path, output_path);
        assert_eq!(result.format, ReportFormat::Json);
        assert!(result.size > 0);
        assert!(output_path.exists());

        // 验证 JSON 格式正确
        let content = tokio::fs::read_to_string(&output_path).await.unwrap();
        serde_json::from_str::<serde_json::Value>(&content).unwrap();
    }

    #[tokio::test]
    async fn test_generate_content_only() {
        let config = ReportConfig {
            format: ReportFormat::Text,
            ..Default::default()
        };

        let generator = ReportGenerator::new(config);
        let report = create_test_report();

        let content = generator.generate_content(&report).await.unwrap();
        assert!(!content.is_empty());
        assert!(content.contains("Code Review Report"));
    }

    #[tokio::test]
    async fn test_default_output_path() {
        let config = ReportConfig {
            format: ReportFormat::Markdown,
            output_path: None,
            ..Default::default()
        };

        let generator = ReportGenerator::new(config);
        let report = create_test_report();

        let path = generator.determine_output_path(&report).unwrap();
        assert!(path.to_string_lossy().contains("code-review"));
        assert!(path.to_string_lossy().ends_with(".md"));
    }
}