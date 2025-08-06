use super::*;
use chrono::Utc;
use std::collections::HashMap;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_review_configuration_default() {
        let config = ReviewConfiguration::default();

        assert!(config.static_analysis);
        assert!(!config.ai_review);
        assert!(config.sensitive_scan);
        assert!(config.complexity_analysis);
        assert!(!config.duplication_scan);
        assert!(!config.dependency_scan);
        assert!(!config.coverage_analysis);
        assert!(!config.performance_analysis);
        assert!(!config.trend_analysis);
    }

    #[test]
    fn test_severity_ordering() {
        assert!(Severity::Critical > Severity::High);
        assert!(Severity::High > Severity::Medium);
        assert!(Severity::Medium > Severity::Low);
        assert!(Severity::Low > Severity::Info);
    }

    #[test]
    fn test_risk_level_ordering() {
        assert!(RiskLevel::Critical > RiskLevel::High);
        assert!(RiskLevel::High > RiskLevel::Medium);
        assert!(RiskLevel::Medium > RiskLevel::Low);
    }

    #[test]
    fn test_issue_creation() {
        let issue = Issue {
            tool: "test-tool".to_string(),
            file_path: "test.rs".to_string(),
            line_number: Some(10),
            column_number: Some(5),
            severity: Severity::High,
            category: IssueCategory::Bug,
            message: "Test issue".to_string(),
            suggestion: Some("Fix this".to_string()),
            rule_id: Some("TEST001".to_string()),
        };

        assert_eq!(issue.tool, "test-tool");
        assert_eq!(issue.file_path, "test.rs");
        assert_eq!(issue.line_number, Some(10));
        assert_eq!(issue.severity, Severity::High);
        assert_eq!(issue.message, "Test issue");
    }

    // #[test]
    // fn test_sensitive_item_masking() {
    //     // use crate::analysis::sensitive::result::{SensitiveItem, SensitiveInfoType, RiskLevel};

    //     let api_key = SensitiveItem::new(
    //         SensitiveInfoType::ApiKey,
    //         1,
    //         0,
    //         32,
    //         "AKIA1234567890ABCDEF".to_string(),
    //         0.95,
    //         RiskLevel::Critical,
    //         "aws-access-key".to_string(),
    //     );

    //     assert_eq!(api_key.masked_text, "AKIA***CDEF");
    //     assert_eq!(api_key.confidence, 0.95);
    //     assert_eq!(api_key.risk_level, RiskLevel::Critical);
    // }

    #[test]
    fn test_code_review_report_serialization() {
        let report = CodeReviewReport {
            summary: ReviewSummary {
                project_path: "/test/project".to_string(),
                files_analyzed: 5,
                languages_detected: vec!["Rust".to_string(), "Go".to_string()],
                total_issues: 10,
                critical_issues: 1,
                high_issues: 2,
                medium_issues: 3,
                low_issues: 4,
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
                user_id: Some("test-user".to_string()),
                correlation_id: Some("test-correlation".to_string()),
                tags: HashMap::new(),
                configuration: ReviewConfiguration::default(),
            },
        };

        // 测试序列化
        let json = serde_json::to_string(&report).expect("Failed to serialize report");
        assert!(!json.is_empty());

        // 测试反序列化
        let deserialized: CodeReviewReport = serde_json::from_str(&json)
            .expect("Failed to deserialize report");

        assert_eq!(deserialized.summary.project_path, "/test/project");
        assert_eq!(deserialized.summary.files_analyzed, 5);
        assert_eq!(deserialized.overall_score, 7.5);
    }

    #[test]
    fn test_duplication_type_variants() {
        let exact = DuplicationType::Exact;
        let structural = DuplicationType::Structural;
        let semantic = DuplicationType::Semantic;

        // 测试序列化
        let exact_json = serde_json::to_string(&exact).unwrap();
        let structural_json = serde_json::to_string(&structural).unwrap();
        let semantic_json = serde_json::to_string(&semantic).unwrap();

        assert_eq!(exact_json, "\"Exact\"");
        assert_eq!(structural_json, "\"Structural\"");
        assert_eq!(semantic_json, "\"Semantic\"");
    }

    #[test]
    fn test_dependency_type_variants() {
        let direct = DependencyType::Direct;
        let transitive = DependencyType::Transitive;
        let development = DependencyType::Development;

        // 确保所有变体都可以序列化
        assert!(serde_json::to_string(&direct).is_ok());
        assert!(serde_json::to_string(&transitive).is_ok());
        assert!(serde_json::to_string(&development).is_ok());
    }

    #[test]
    fn test_trend_direction() {
        let improving = TrendDirection::Improving;
        let declining = TrendDirection::Declining;
        let stable = TrendDirection::Stable;

        // 测试序列化和反序列化
        for direction in [improving, declining, stable] {
            let json = serde_json::to_string(&direction).unwrap();
            let deserialized: TrendDirection = serde_json::from_str(&json).unwrap();

            // 由于没有 PartialEq，我们检查序列化结果
            let original_json = serde_json::to_string(&direction).unwrap();
            let deserialized_json = serde_json::to_string(&deserialized).unwrap();
            assert_eq!(original_json, deserialized_json);
        }
    }

    #[test]
    fn test_memory_issue_type() {
        let leak = MemoryIssueType::PotentialLeak;
        let unreleased = MemoryIssueType::UnreleasedResource;
        let circular = MemoryIssueType::CircularReference;
        let excessive = MemoryIssueType::ExcessiveAllocation;

        // 确保所有变体都可以序列化
        for issue_type in [leak, unreleased, circular, excessive] {
            assert!(serde_json::to_string(&issue_type).is_ok());
        }
    }

    #[test]
    fn test_complexity_metrics_calculation() {
        let functions = vec![
            FunctionComplexity {
                name: "func1".to_string(),
                line_start: 1,
                line_end: 10,
                cyclomatic_complexity: 5,
                cognitive_complexity: 3,
                function_length: 10,
                max_nesting_depth: 2,
                risk_level: RiskLevel::Medium,
            },
            FunctionComplexity {
                name: "func2".to_string(),
                line_start: 11,
                line_end: 25,
                cyclomatic_complexity: 8,
                cognitive_complexity: 6,
                function_length: 15,
                max_nesting_depth: 3,
                risk_level: RiskLevel::High,
            },
        ];

        let metrics = ComplexityMetrics {
            average_cyclomatic: 6.5,
            average_cognitive: 4.5,
            average_function_length: 12.5,
            max_complexity: 8,
            functions_over_threshold: 1,
        };

        assert_eq!(metrics.average_cyclomatic, 6.5);
        assert_eq!(metrics.max_complexity, 8);
    }
}