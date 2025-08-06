use super::{ReportFormatter, utils};
use crate::models::review::CodeReviewReport;
use crate::report::config::ReportConfig;
use anyhow::Result;
use async_trait::async_trait;

/// 文本格式化器
pub struct TextFormatter {
    use_colors: bool,
}

impl TextFormatter {
    /// 创建新的文本格式化器
    pub fn new() -> Self {
        Self {
            use_colors: true,
        }
    }

    /// 创建不使用颜色的文本格式化器
    pub fn new_no_color() -> Self {
        Self {
            use_colors: false,
        }
    }

    /// 生成分隔线
    fn separator(&self, length: usize) -> String {
        "=".repeat(length)
    }

    /// 生成子分隔线
    fn sub_separator(&self, length: usize) -> String {
        "-".repeat(length)
    }

    /// 格式化严重程度
    fn format_severity(&self, severity: &crate::models::review::Severity) -> String {
        let severity_str = utils::severity_to_string(severity);
        if self.use_colors {
            match severity {
                crate::models::review::Severity::Critical => format!("\x1b[91m{}\x1b[0m", severity_str), // 红色
                crate::models::review::Severity::High => format!("\x1b[93m{}\x1b[0m", severity_str),     // 黄色
                crate::models::review::Severity::Medium => format!("\x1b[94m{}\x1b[0m", severity_str),   // 蓝色
                crate::models::review::Severity::Low => format!("\x1b[92m{}\x1b[0m", severity_str),      // 绿色
                crate::models::review::Severity::Info => format!("\x1b[96m{}\x1b[0m", severity_str),     // 青色
            }
        } else {
            severity_str.to_string()
        }
    }

    /// 格式化风险等级
    fn format_risk_level(&self, risk_level: &crate::models::review::RiskLevel) -> String {
        let risk_str = utils::risk_level_to_string(risk_level);
        if self.use_colors {
            match risk_level {
                crate::models::review::RiskLevel::Critical => format!("\x1b[91m{}\x1b[0m", risk_str), // 红色
                crate::models::review::RiskLevel::High => format!("\x1b[93m{}\x1b[0m", risk_str),     // 黄色
                crate::models::review::RiskLevel::Medium => format!("\x1b[94m{}\x1b[0m", risk_str),   // 蓝色
                crate::models::review::RiskLevel::Low => format!("\x1b[92m{}\x1b[0m", risk_str),      // 绿色
            }
        } else {
            risk_str.to_string()
        }
    }

    /// 生成摘要部分
    fn generate_summary(&self, report: &CodeReviewReport) -> String {
        let mut content = String::new();

        content.push_str(&self.separator(80));
        content.push('\n');
        content.push_str("                            CODE REVIEW REPORT\n");
        content.push_str(&self.separator(80));
        content.push('\n');
        content.push('\n');

        content.push_str(&format!("Project: {}\n", report.summary.project_path));
        content.push_str(&format!("Files Analyzed: {}\n", report.summary.files_analyzed));
        content.push_str(&format!("Languages: {}\n", report.summary.languages_detected.join(", ")));
        content.push_str(&format!("Analysis Duration: {}\n", utils::format_duration(&report.summary.analysis_duration)));
        content.push_str(&format!("Overall Score: {:.1}/10\n", report.overall_score));
        content.push_str(&format!("Generated: {}\n", report.summary.created_at.format("%Y-%m-%d %H:%M:%S UTC")));
        content.push('\n');

        content.push_str("ISSUE SUMMARY\n");
        content.push_str(&self.sub_separator(13));
        content.push('\n');
        content.push_str(&format!("Critical: {}\n", report.summary.critical_issues));
        content.push_str(&format!("High:     {}\n", report.summary.high_issues));
        content.push_str(&format!("Medium:   {}\n", report.summary.medium_issues));
        content.push_str(&format!("Low:      {}\n", report.summary.low_issues));
        content.push_str(&format!("Total:    {}\n", report.summary.total_issues));
        content.push('\n');

        if !report.recommendations.is_empty() {
            content.push_str("KEY RECOMMENDATIONS\n");
            content.push_str(&self.sub_separator(19));
            content.push('\n');
            for recommendation in &report.recommendations {
                content.push_str(&format!("• {}\n", recommendation));
            }
            content.push('\n');
        }

        content.push_str(&self.separator(80));
        content.push('\n');

        content
    }

    /// 生成静态分析结果
    fn generate_static_analysis(&self, report: &CodeReviewReport, config: &ReportConfig) -> String {
        if !config.sections.include_static_analysis || report.static_analysis_results.is_empty() {
            return String::new();
        }

        let mut content = String::new();
        content.push('\n');
        content.push_str("STATIC ANALYSIS RESULTS\n");
        content.push_str(&self.separator(23));
        content.push('\n');
        content.push('\n');

        for result in &report.static_analysis_results {
            content.push_str(&format!("{} - {}\n", result.tool_name, result.file_path));
            content.push_str(&self.sub_separator(50));
            content.push('\n');
            content.push('\n');

            let mut issues = result.issues.clone();

            // 应用严重程度过滤
            if let Some(ref severity_filter) = config.options.severity_filter {
                issues.retain(|issue| severity_filter.contains(&issue.severity));
            }

            // 应用最大问题数量限制
            if let Some(max_issues) = config.options.max_issues_per_section {
                issues.truncate(max_issues);
            }

            for issue in &issues {
                content.push_str(&format!("[{}] ", self.format_severity(&issue.severity)));
                if let Some(line) = issue.line_number {
                    content.push_str(&format!("Line {}: ", line));
                }
                content.push_str(&format!("{}\n", issue.message));

                if let Some(ref suggestion) = issue.suggestion {
                    content.push_str(&format!("   Suggestion: {}\n", suggestion));
                }
                content.push('\n');
            }
        }

        content
    }

    /// 生成 AI 审查结果
    fn generate_ai_review(&self, report: &CodeReviewReport, config: &ReportConfig) -> String {
        if !config.sections.include_ai_review || report.ai_review_results.is_empty() {
            return String::new();
        }

        let mut content = String::new();
        content.push_str("AI REVIEW RESULTS\n");
        content.push_str(&self.separator(17));
        content.push('\n');
        content.push('\n');

        for result in &report.ai_review_results {
            content.push_str(&format!("{} ({})\n", result.file_path, result.provider));
            content.push_str(&self.sub_separator(50));
            content.push('\n');
            content.push('\n');

            content.push_str(&format!("Quality Score: {:.1}/10\n", result.quality_score));
            content.push('\n');

            if !result.suggestions.is_empty() {
                content.push_str("Suggestions:\n");
                for suggestion in &result.suggestions {
                    content.push_str(&format!("• {}\n", suggestion));
                }
                content.push('\n');
            }

            if !result.learning_resources.is_empty() {
                content.push_str("Learning Resources:\n");
                for resource in &result.learning_resources {
                    content.push_str(&format!("• {}\n", resource));
                }
                content.push('\n');
            }
        }

        content
    }

    /// 生成敏感信息检测结果
    fn generate_sensitive_info(&self, report: &CodeReviewReport, config: &ReportConfig) -> String {
        if !config.sections.include_sensitive_info || report.sensitive_info_results.is_empty() {
            return String::new();
        }

        let mut content = String::new();
        content.push_str("SENSITIVE INFORMATION DETECTION\n");
        content.push_str(&self.separator(31));
        content.push('\n');
        content.push('\n');

        for result in &report.sensitive_info_results {
            content.push_str(&format!("{}\n", result.file_path));
            content.push_str(&self.sub_separator(50));
            content.push('\n');
            content.push('\n');

            for item in &result.items {
                content.push_str(&format!("[{}] {} at line {}\n",
                    self.format_risk_level(&item.risk_level),
                    item.info_type,
                    item.line_number
                ));
                content.push_str(&format!("Confidence: {}\n", utils::format_percentage(item.confidence)));
                content.push_str(&format!("Masked: {}\n", item.masked_text));

                if !item.recommendations.is_empty() {
                    content.push_str("Recommendations:\n");
                    for recommendation in &item.recommendations {
                        content.push_str(&format!("• {}\n", recommendation));
                    }
                }
                content.push('\n');
            }
        }

        content
    }

    /// 生成复杂度分析结果
    fn generate_complexity(&self, report: &CodeReviewReport, config: &ReportConfig) -> String {
        if !config.sections.include_complexity || report.complexity_results.is_empty() {
            return String::new();
        }

        let mut content = String::new();
        content.push_str("COMPLEXITY ANALYSIS\n");
        content.push_str(&self.separator(19));
        content.push('\n');
        content.push('\n');

        for result in &report.complexity_results {
            content.push_str(&format!("{}\n", result.file_path));
            content.push_str(&self.sub_separator(50));
            content.push('\n');
            content.push('\n');

            content.push_str("Overall Metrics:\n");
            content.push_str(&format!("- Average Cyclomatic Complexity: {:.1}\n", result.overall_metrics.average_cyclomatic));
            content.push_str(&format!("- Average Cognitive Complexity: {:.1}\n", result.overall_metrics.average_cognitive));
            content.push_str(&format!("- Functions Over Threshold: {}\n", result.overall_metrics.functions_over_threshold));
            content.push('\n');

            if !result.hotspots.is_empty() {
                content.push_str("Complexity Hotspots:\n");
                for hotspot in &result.hotspots {
                    content.push_str(&format!("- {} (Line {}) - Score: {:.1}\n",
                        hotspot.function_name,
                        hotspot.line_number,
                        hotspot.complexity_score
                    ));
                }
                content.push('\n');
            }
        }

        content
    }
}

#[async_trait]
impl ReportFormatter for TextFormatter {
    async fn format(&self, report: &CodeReviewReport, config: &ReportConfig) -> Result<String> {
        let mut content = String::new();

        // 根据配置决定是否使用颜色
        let formatter = if config.options.colored_output && self.use_colors {
            TextFormatter::new()
        } else {
            TextFormatter::new_no_color()
        };

        // 生成摘要
        if config.sections.include_summary {
            content.push_str(&formatter.generate_summary(report));
        }

        // 生成静态分析结果
        content.push_str(&formatter.generate_static_analysis(report, config));

        // 生成 AI 审查结果
        content.push_str(&formatter.generate_ai_review(report, config));

        // 生成敏感信息检测结果
        content.push_str(&formatter.generate_sensitive_info(report, config));

        // 生成复杂度分析结果
        content.push_str(&formatter.generate_complexity(report, config));

        Ok(content)
    }

    fn name(&self) -> &str {
        "text"
    }

    fn file_extension(&self) -> &str {
        "txt"
    }
}

impl Default for TextFormatter {
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
    async fn test_text_formatting() {
        let formatter = TextFormatter::new();
        let report = create_test_report();
        let config = ReportConfig::default();

        let result = formatter.format(&report, &config).await.unwrap();

        assert!(result.contains("CODE REVIEW REPORT"));
        assert!(result.contains("Project: /test/project"));
        assert!(result.contains("Files Analyzed: 5"));
        assert!(result.contains("Overall Score: 7.5"));
        assert!(result.contains("Fix formatting issues"));
    }

    #[tokio::test]
    async fn test_no_color_formatting() {
        let formatter = TextFormatter::new_no_color();
        let report = create_test_report();
        let config = ReportConfig::default();

        let result = formatter.format(&report, &config).await.unwrap();

        // 验证不包含 ANSI 颜色代码
        assert!(!result.contains("\x1b["));
        assert!(result.contains("Medium"));
    }

    #[tokio::test]
    async fn test_colored_output_config() {
        let formatter = TextFormatter::new();
        let report = create_test_report();

        let mut config = ReportConfig::default();
        config.options.colored_output = false;

        let result = formatter.format(&report, &config).await.unwrap();

        // 当配置禁用颜色时，应该不包含颜色代码
        assert!(!result.contains("\x1b["));
    }

    #[tokio::test]
    async fn test_section_filtering() {
        let formatter = TextFormatter::new();
        let report = create_test_report();

        let mut config = ReportConfig::default();
        config.sections.include_static_analysis = false;

        let result = formatter.format(&report, &config).await.unwrap();

        // 验证静态分析部分被过滤掉
        assert!(!result.contains("STATIC ANALYSIS RESULTS"));
    }
}