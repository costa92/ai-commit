use super::ReportFormatter;
use crate::models::review::CodeReviewReport;
use crate::report::config::ReportConfig;
use anyhow::Result;
use async_trait::async_trait;

/// JSON 格式化器
pub struct JsonFormatter;

impl JsonFormatter {
    /// 创建新的 JSON 格式化器
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl ReportFormatter for JsonFormatter {
    async fn format(&self, report: &CodeReviewReport, config: &ReportConfig) -> Result<String> {
        // 根据配置过滤报告内容
        let mut filtered_report = report.clone();

        // 根据配置移除不需要的部分
        if !config.sections.include_static_analysis {
            filtered_report.static_analysis_results.clear();
        }

        if !config.sections.include_ai_review {
            filtered_report.ai_review_results.clear();
        }

        if !config.sections.include_sensitive_info {
            filtered_report.sensitive_info_results.clear();
        }

        if !config.sections.include_complexity {
            filtered_report.complexity_results.clear();
        }

        if !config.sections.include_duplication {
            filtered_report.duplication_results.clear();
        }

        if !config.sections.include_dependency {
            filtered_report.dependency_results = None;
        }

        if !config.sections.include_coverage {
            filtered_report.coverage_results = None;
        }

        if !config.sections.include_performance {
            filtered_report.performance_results.clear();
        }

        if !config.sections.include_trend {
            filtered_report.trend_results = None;
        }

        if !config.sections.include_recommendations {
            filtered_report.recommendations.clear();
        }

        // 应用严重程度过滤
        if let Some(ref severity_filter) = config.options.severity_filter {
            for static_result in &mut filtered_report.static_analysis_results {
                static_result.issues.retain(|issue| severity_filter.contains(&issue.severity));
            }
        }

        // 应用最大问题数量限制
        if let Some(max_issues) = config.options.max_issues_per_section {
            for static_result in &mut filtered_report.static_analysis_results {
                static_result.issues.truncate(max_issues);
            }
        }

        // 序列化为 JSON
        let json_string = if config.options.verbose {
            serde_json::to_string_pretty(&filtered_report)?
        } else {
            serde_json::to_string(&filtered_report)?
        };

        Ok(json_string)
    }

    fn name(&self) -> &str {
        "json"
    }

    fn file_extension(&self) -> &str {
        "json"
    }
}

impl Default for JsonFormatter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::review::*;
    use chrono::Utc;
    use std::collections::HashMap;

    fn create_test_report() -> CodeReviewReport {
        CodeReviewReport {
            summary: ReviewSummary {
                project_path: "/test/project".to_string(),
                files_analyzed: 5,
                languages_detected: vec!["Rust".to_string()],
                total_issues: 2,
                critical_issues: 0,
                high_issues: 1,
                medium_issues: 1,
                low_issues: 0,
                analysis_duration: std::time::Duration::from_secs(30),
                created_at: Utc::now(),
            },
            static_analysis_results: vec![
                StaticAnalysisResult {
                    tool_name: "rustfmt".to_string(),
                    file_path: "src/main.rs".to_string(),
                    issues: vec![
                        Issue {
                            tool: "rustfmt".to_string(),
                            file_path: "src/main.rs".to_string(),
                            line_number: Some(10),
                            column_number: Some(5),
                            severity: Severity::Medium,
                            category: IssueCategory::Style,
                            message: "Code formatting issue".to_string(),
                            suggestion: Some("Run rustfmt".to_string()),
                            rule_id: Some("format".to_string()),
                        },
                        Issue {
                            tool: "clippy".to_string(),
                            file_path: "src/main.rs".to_string(),
                            line_number: Some(20),
                            column_number: Some(10),
                            severity: Severity::High,
                            category: IssueCategory::Bug,
                            message: "Potential bug detected".to_string(),
                            suggestion: Some("Fix the logic".to_string()),
                            rule_id: Some("clippy::bug".to_string()),
                        }
                    ],
                    execution_time: std::time::Duration::from_millis(100),
                }
            ],
            ai_review_results: vec![],
            sensitive_info_results: vec![],
            complexity_results: vec![],
            duplication_results: vec![],
            dependency_results: None,
            coverage_results: None,
            performance_results: vec![],
            trend_results: None,
            overall_score: 7.5,
            recommendations: vec!["Fix formatting issues".to_string()],
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
    async fn test_json_formatting() {
        let formatter = JsonFormatter::new();
        let report = create_test_report();
        let config = ReportConfig::default();

        let result = formatter.format(&report, &config).await.unwrap();

        // 验证是否为有效的 JSON
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert!(parsed.is_object());

        // 验证包含基本字段
        assert!(result.contains("summary"));
        assert!(result.contains("overall_score"));
        assert!(result.contains("static_analysis_results"));
    }

    #[tokio::test]
    async fn test_json_filtering() {
        let formatter = JsonFormatter::new();
        let report = create_test_report();

        let mut config = ReportConfig::default();
        config.sections.include_static_analysis = false;
        config.sections.include_ai_review = false;

        let result = formatter.format(&report, &config).await.unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        // 验证静态分析结果被过滤掉
        assert_eq!(parsed["static_analysis_results"].as_array().unwrap().len(), 0);
        assert_eq!(parsed["ai_review_results"].as_array().unwrap().len(), 0);
    }

    #[tokio::test]
    async fn test_severity_filtering() {
        let formatter = JsonFormatter::new();
        let report = create_test_report();

        let mut config = ReportConfig::default();
        config.options.severity_filter = Some(vec![Severity::High]);

        let result = formatter.format(&report, &config).await.unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        // 验证只包含高严重程度的问题
        let issues = &parsed["static_analysis_results"][0]["issues"];
        assert_eq!(issues.as_array().unwrap().len(), 1);
        assert_eq!(issues[0]["severity"], "High");
    }

    #[tokio::test]
    async fn test_max_issues_limit() {
        let formatter = JsonFormatter::new();
        let report = create_test_report();

        let mut config = ReportConfig::default();
        config.options.max_issues_per_section = Some(1);

        let result = formatter.format(&report, &config).await.unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        // 验证问题数量被限制
        let issues = &parsed["static_analysis_results"][0]["issues"];
        assert_eq!(issues.as_array().unwrap().len(), 1);
    }

    #[tokio::test]
    async fn test_pretty_json() {
        let formatter = JsonFormatter::new();
        let report = create_test_report();

        let mut config = ReportConfig::default();
        config.options.verbose = true;

        let result = formatter.format(&report, &config).await.unwrap();

        // 验证格式化的 JSON 包含换行符和缩进
        assert!(result.contains('\n'));
        assert!(result.contains("  "));
    }
}