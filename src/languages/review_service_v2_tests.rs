#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::languages::{Language, LanguageFeature, LanguageAnalysisResult};
    use std::collections::HashMap;

    fn create_test_config() -> Config {
        Config {
            provider: "ollama".to_string(),
            model: "test-model".to_string(),
            debug: true,
            ..Default::default()
        }
    }

    fn create_test_language_feature(feature_type: &str, name: &str, line: usize) -> LanguageFeature {
        LanguageFeature {
            feature_type: feature_type.to_string(),
            name: name.to_string(),
            line_number: Some(line),
            description: format!("{} {}", feature_type, name),
        }
    }

    fn create_test_analysis_result(language: Language, features: Vec<LanguageFeature>) -> LanguageAnalysisResult {
        LanguageAnalysisResult {
            language,
            features,
            scope_suggestions: vec!["test".to_string()],
            change_patterns: vec!["test pattern".to_string()],
        }
    }

    #[test]
    fn test_review_options_default() {
        let options = ReviewOptions::default();
        assert!(options.enable_ai_review);
        assert_eq!(options.ai_review_types, vec!["general".to_string()]);
        assert!(options.include_static_analysis);
        assert!(options.detailed_feedback);
        assert!(options.language_specific_rules);
    }

    #[test]
    fn test_review_options_custom() {
        let options = ReviewOptions {
            enable_ai_review: false,
            ai_review_types: vec!["security".to_string(), "performance".to_string()],
            include_static_analysis: false,
            detailed_feedback: false,
            language_specific_rules: false,
        };
        
        assert!(!options.enable_ai_review);
        assert_eq!(options.ai_review_types.len(), 2);
        assert!(options.ai_review_types.contains(&"security".to_string()));
        assert!(options.ai_review_types.contains(&"performance".to_string()));
        assert!(!options.include_static_analysis);
        assert!(!options.detailed_feedback);
        assert!(!options.language_specific_rules);
    }

    #[test]
    fn test_code_review_service_creation() {
        let service = CodeReviewService::new();
        assert!(!service.enable_ai_review);

        let config = create_test_config();
        let service = CodeReviewService::with_config(config.clone());
        assert_eq!(service.config.provider, "ollama");
        assert!(!service.enable_ai_review);

        let service = CodeReviewService::with_config(config).with_ai_review(true);
        assert!(service.enable_ai_review);
    }

    #[test]
    fn test_code_review_service_default() {
        let service = CodeReviewService::default();
        assert!(!service.enable_ai_review);
    }

    #[test]
    fn test_detect_language() {
        let service = CodeReviewService::new();
        
        assert_eq!(service.detect_language("main.rs"), Language::Rust);
        assert_eq!(service.detect_language("main.go"), Language::Go);
        assert_eq!(service.detect_language("index.ts"), Language::TypeScript);
        assert_eq!(service.detect_language("app.js"), Language::JavaScript);
        assert_eq!(service.detect_language("config.json"), Language::Unknown);
    }

    #[test]
    fn test_generate_ai_review_summary() {
        let service = CodeReviewService::new();
        
        let ai_reviews = vec![
            AIReviewResult {
                review_type: "rust_comprehensive".to_string(),
                overall_score: 8.5,
                summary: "Good Rust code".to_string(),
                detailed_feedback: "Detailed feedback".to_string(),
                security_score: 9.0,
                performance_score: 8.0,
                maintainability_score: 8.5,
                recommendations: vec!["Add more tests".to_string()],
                learning_resources: vec!["https://doc.rust-lang.org".to_string()],
            },
            AIReviewResult {
                review_type: "go_performance".to_string(),
                overall_score: 7.0,
                summary: "Go performance review".to_string(),
                detailed_feedback: "Performance feedback".to_string(),
                security_score: 6.0,
                performance_score: 6.0,
                maintainability_score: 8.0,
                recommendations: vec!["Optimize goroutines".to_string()],
                learning_resources: vec!["https://golang.org".to_string()],
            },
        ];

        let summary = service.generate_ai_review_summary(&ai_reviews);
        
        assert_eq!(summary.total_files_reviewed, 2);
        assert_eq!(summary.average_score, 7.75); // (8.5 + 7.0) / 2
        assert!(summary.critical_issues.iter().any(|issue| issue.contains("安全分数较低")));
        assert!(summary.critical_issues.iter().any(|issue| issue.contains("性能分数较低")));
        assert!(summary.common_patterns.contains(&"rust_comprehensive".to_string()));
        assert!(summary.common_patterns.contains(&"go_performance".to_string()));
        assert!(summary.recommended_actions.contains(&"Add more tests".to_string()));
        assert!(summary.recommended_actions.contains(&"Optimize goroutines".to_string()));
    }

    #[test]
    fn test_generate_ai_review_summary_empty() {
        let service = CodeReviewService::new();
        let summary = service.generate_ai_review_summary(&[]);
        
        assert_eq!(summary.total_files_reviewed, 0);
        assert_eq!(summary.average_score, 0.0);
        assert!(summary.critical_issues.is_empty());
        assert!(summary.common_patterns.is_empty());
        assert!(summary.recommended_actions.is_empty());
    }

    #[test]
    fn test_generate_summary() {
        let service = CodeReviewService::new();
        
        let files = vec![
            FileAnalysisResult {
                file_path: "src/main.rs".to_string(),
                language: Language::Rust,
                analysis: create_test_analysis_result(
                    Language::Rust,
                    vec![
                        create_test_language_feature("function", "main", 1),
                        create_test_language_feature("struct", "Config", 5),
                    ]
                ),
                static_analysis: vec![],
                ai_review: Some(AIReviewResult {
                    review_type: "rust".to_string(),
                    overall_score: 8.0,
                    summary: "Good".to_string(),
                    detailed_feedback: "Detailed".to_string(),
                    security_score: 8.0,
                    performance_score: 8.0,
                    maintainability_score: 8.0,
                    recommendations: vec!["Test more".to_string()],
                    learning_resources: vec![],
                }),
            },
            FileAnalysisResult {
                file_path: "main.go".to_string(),
                language: Language::Go,
                analysis: create_test_analysis_result(
                    Language::Go,
                    vec![
                        create_test_language_feature("function", "main", 1),
                        create_test_language_feature("struct", "User", 10),
                        create_test_language_feature("interface", "Writer", 20),
                    ]
                ),
                static_analysis: vec![],
                ai_review: None,
            },
        ];

        let summary = service.generate_summary(&files);
        
        assert_eq!(summary.total_files, 2);
        assert_eq!(summary.languages_detected.get(&Language::Rust), Some(&1));
        assert_eq!(summary.languages_detected.get(&Language::Go), Some(&1));
        assert_eq!(summary.total_features, 5); // 2 + 3
        assert!(summary.common_patterns.contains(&"test pattern".to_string()));
        assert!(summary.test_suggestions.contains(&"Test more".to_string()));
    }

    #[test]
    fn test_generate_static_analysis_summary() {
        let service = CodeReviewService::new();
        
        let static_analysis_results = vec![
            crate::languages::static_analysis::StaticAnalysisResult {
                tool_name: "clippy".to_string(),
                severity: "warning".to_string(),
                message: "unused variable".to_string(),
                file_path: "src/main.rs".to_string(),
                line_number: Some(10),
                column_number: Some(5),
                rule_id: Some("unused_variables".to_string()),
                suggestion: Some("remove unused variable".to_string()),
            },
            crate::languages::static_analysis::StaticAnalysisResult {
                tool_name: "clippy".to_string(),
                severity: "error".to_string(),
                message: "type mismatch".to_string(),
                file_path: "src/lib.rs".to_string(),
                line_number: Some(20),
                column_number: Some(15),
                rule_id: Some("type_mismatch".to_string()),
                suggestion: None,
            },
            crate::languages::static_analysis::StaticAnalysisResult {
                tool_name: "gofmt".to_string(),
                severity: "info".to_string(),
                message: "formatting issue".to_string(),
                file_path: "main.go".to_string(),
                line_number: Some(5),
                column_number: None,
                rule_id: Some("formatting".to_string()),
                suggestion: Some("run gofmt".to_string()),
            },
        ];

        let files = vec![
            FileAnalysisResult {
                file_path: "src/main.rs".to_string(),
                language: Language::Rust,
                analysis: create_test_analysis_result(Language::Rust, vec![]),
                static_analysis: vec![static_analysis_results[0].clone()],
                ai_review: None,
            },
            FileAnalysisResult {
                file_path: "src/lib.rs".to_string(),
                language: Language::Rust,
                analysis: create_test_analysis_result(Language::Rust, vec![]),
                static_analysis: vec![static_analysis_results[1].clone()],
                ai_review: None,
            },
            FileAnalysisResult {
                file_path: "main.go".to_string(),
                language: Language::Go,
                analysis: create_test_analysis_result(Language::Go, vec![]),
                static_analysis: vec![static_analysis_results[2].clone()],
                ai_review: None,
            },
        ];

        let summary = service.generate_static_analysis_summary(&files);
        
        assert_eq!(summary.total_issues, 3);
        assert!(summary.tools_used.contains(&"clippy".to_string()));
        assert!(summary.tools_used.contains(&"gofmt".to_string()));
        assert_eq!(summary.issues_by_severity.get("warning"), Some(&1));
        assert_eq!(summary.issues_by_severity.get("error"), Some(&1));
        assert_eq!(summary.issues_by_severity.get("info"), Some(&1));
        assert_eq!(summary.issues_by_tool.get("clippy"), Some(&2));
        assert_eq!(summary.issues_by_tool.get("gofmt"), Some(&1));
        assert!(summary.execution_time.as_millis() > 0);
        assert!(summary.tools_unavailable.is_empty());
    }

    #[test]
    fn test_parse_git_diff() {
        let service = CodeReviewService::new();
        
        let diff_content = r#"diff --git a/src/main.rs b/src/main.rs
index 1234567..abcdefg 100644
--- a/src/main.rs
+++ b/src/main.rs
@@ -1,3 +1,6 @@
+use std::collections::HashMap;
+
 fn main() {
+    let map = HashMap::new();
     println!("Hello, world!");
 }
diff --git a/src/lib.rs b/src/lib.rs
index 9876543..fedcba9 100644
--- a/src/lib.rs
+++ b/src/lib.rs
@@ -1,2 +1,4 @@
+pub mod utils;
+
 pub fn hello() {
     println!("Hello from lib!");
"#;

        let results = service.parse_git_diff(diff_content);
        
        assert_eq!(results.len(), 2);
        
        let (main_file, main_lines) = &results[0];
        assert_eq!(main_file, "src/main.rs");
        assert_eq!(main_lines.len(), 3);
        assert_eq!(main_lines[0], "use std::collections::HashMap;");
        assert_eq!(main_lines[1], "");
        assert_eq!(main_lines[2], "    let map = HashMap::new();");
        
        let (lib_file, lib_lines) = &results[1];
        assert_eq!(lib_file, "src/lib.rs");
        assert_eq!(lib_lines.len(), 2);
        assert_eq!(lib_lines[0], "pub mod utils;");
        assert_eq!(lib_lines[1], "");
    }

    #[test]
    fn test_extract_file_path() {
        let service = CodeReviewService::new();
        
        let test_cases = vec![
            ("diff --git a/src/main.rs b/src/main.rs", Some("src/main.rs".to_string())),
            ("diff --git a/old/path.rs b/new/path.rs", Some("new/path.rs".to_string())),
            ("diff --git a/file.go b/file.go", Some("file.go".to_string())),
            ("not a diff line", None),
            ("diff --git a/file.js", None), // 缺少 b/ 部分
        ];

        for (line, expected) in test_cases {
            let result = service.extract_file_path(line);
            assert_eq!(result, expected, "Failed for line: {}", line);
        }
    }

    #[test]
    fn test_format_enhanced_report() {
        let service = CodeReviewService::new();
        
        let mut languages_detected = HashMap::new();
        languages_detected.insert(Language::Rust, 1);
        languages_detected.insert(Language::Go, 1);

        let mut issues_by_severity = HashMap::new();
        issues_by_severity.insert("warning".to_string(), 2);
        issues_by_severity.insert("error".to_string(), 1);

        let mut issues_by_tool = HashMap::new();
        issues_by_tool.insert("clippy".to_string(), 2);
        issues_by_tool.insert("gofmt".to_string(), 1);

        let report = CodeReviewReport {
            files: vec![
                FileAnalysisResult {
                    file_path: "src/main.rs".to_string(),
                    language: Language::Rust,
                    analysis: create_test_analysis_result(
                        Language::Rust,
                        vec![create_test_language_feature("function", "main", 1)]
                    ),
                    static_analysis: vec![],
                    ai_review: Some(AIReviewResult {
                        review_type: "rust_comprehensive".to_string(),
                        overall_score: 8.5,
                        summary: "Good Rust code".to_string(),
                        detailed_feedback: "Detailed feedback".to_string(),
                        security_score: 9.0,
                        performance_score: 8.0,
                        maintainability_score: 8.5,
                        recommendations: vec!["Add tests".to_string()],
                        learning_resources: vec![],
                    }),
                },
            ],
            summary: ReviewSummary {
                total_files: 1,
                languages_detected,
                total_features: 1,
                common_patterns: vec!["function addition".to_string()],
                overall_risks: vec!["no risks detected".to_string()],
                test_suggestions: vec!["add unit tests".to_string()],
            },
            static_analysis_summary: StaticAnalysisSummary {
                tools_used: vec!["clippy".to_string()],
                total_issues: 3,
                issues_by_severity,
                issues_by_tool,
                execution_time: std::time::Duration::from_millis(100),
                tools_unavailable: vec![],
            },
            ai_review_summary: Some(AIReviewSummary {
                total_files_reviewed: 1,
                average_score: 8.5,
                critical_issues: vec!["no critical issues".to_string()],
                common_patterns: vec!["rust patterns".to_string()],
                best_practices_violations: vec![],
                recommended_actions: vec!["add tests".to_string()],
            }),
        };

        let formatted = service.format_enhanced_report(&report);
        
        assert!(formatted.contains("# 🔍 增强代码审查报告"));
        assert!(formatted.contains("## 📊 审查统计"));
        assert!(formatted.contains("总文件数**: 1"));
        assert!(formatted.contains("代码特征数**: 1"));
        assert!(formatted.contains("静态分析问题**: 3"));
        assert!(formatted.contains("AI 审查文件数**: 1"));
        assert!(formatted.contains("平均质量分数**: 8.5/10"));
        assert!(formatted.contains("## 🗣️ 检测到的编程语言"));
        assert!(formatted.contains("**rust**: 1 个文件"));
        assert!(formatted.contains("**go**: 1 个文件"));
        assert!(formatted.contains("## 🤖 AI 审查摘要"));
        assert!(formatted.contains("### ⚠️ 关键问题"));
        assert!(formatted.contains("### 💡 推荐操作"));
        assert!(formatted.contains("## 📁 详细文件分析"));
        assert!(formatted.contains("### 📄 src/main.rs"));
        assert!(formatted.contains("**AI 评分**: 8.5/10"));
        assert!(formatted.contains("**审查类型**: rust_comprehensive"));
    }

    #[test]
    fn test_format_report_compatibility() {
        let service = CodeReviewService::new();
        
        let report = CodeReviewReport {
            files: vec![],
            summary: ReviewSummary {
                total_files: 0,
                languages_detected: HashMap::new(),
                total_features: 0,
                common_patterns: vec![],
                overall_risks: vec![],
                test_suggestions: vec![],
            },
            static_analysis_summary: StaticAnalysisSummary {
                tools_used: vec![],
                total_issues: 0,
                issues_by_severity: HashMap::new(),
                issues_by_tool: HashMap::new(),
                execution_time: std::time::Duration::from_millis(0),
                tools_unavailable: vec![],
            },
            ai_review_summary: None,
        };

        // 测试向后兼容性
        let formatted1 = service.format_enhanced_report(&report);
        let formatted2 = service.format_report(&report);
        
        assert_eq!(formatted1, formatted2);
    }

    #[tokio::test]
    async fn test_analyze_files_with_options() {
        let service = CodeReviewService::new();
        let options = ReviewOptions {
            enable_ai_review: false,
            ai_review_types: vec!["general".to_string()],
            include_static_analysis: false,
            detailed_feedback: true,
            language_specific_rules: true,
        };

        // 这个测试会尝试读取真实文件，所以我们只测试空文件列表
        let report = service.analyze_files_with_options(&[], &options).await;
        
        assert_eq!(report.files.len(), 0);
        assert_eq!(report.summary.total_files, 0);
        assert!(report.ai_review_summary.is_none());
    }

    #[tokio::test]
    async fn test_analyze_files_compatibility() {
        let service = CodeReviewService::new();
        
        // 测试向后兼容性
        let report = service.analyze_files(&[]).await;
        
        assert_eq!(report.files.len(), 0);
        assert_eq!(report.summary.total_files, 0);
    }

    #[tokio::test]
    async fn test_review_git_changes_with_options() {
        let service = CodeReviewService::new();
        let options = ReviewOptions {
            enable_ai_review: false,
            ai_review_types: vec!["general".to_string()],
            include_static_analysis: false,
            detailed_feedback: true,
            language_specific_rules: true,
        };

        // 测试空的 diff
        let report = service.review_git_changes_with_options("", &options).await;
        
        assert_eq!(report.files.len(), 0);
        assert_eq!(report.summary.total_files, 0);
        assert!(report.ai_review_summary.is_none());
    }

    #[tokio::test]
    async fn test_review_git_changes_compatibility() {
        let service = CodeReviewService::new();
        
        // 测试向后兼容性
        let report = service.review_git_changes("").await;
        
        assert_eq!(report.files.len(), 0);
        assert_eq!(report.summary.total_files, 0);
    }

    #[test]
    fn test_ai_review_result_serialization() {
        let result = AIReviewResult {
            review_type: "test".to_string(),
            overall_score: 8.5,
            summary: "Test summary".to_string(),
            detailed_feedback: "Test feedback".to_string(),
            security_score: 9.0,
            performance_score: 8.0,
            maintainability_score: 8.5,
            recommendations: vec!["Test recommendation".to_string()],
            learning_resources: vec!["https://example.com".to_string()],
        };

        // 测试序列化
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("test"));
        assert!(json.contains("8.5"));

        // 测试反序列化
        let deserialized: AIReviewResult = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.review_type, "test");
        assert_eq!(deserialized.overall_score, 8.5);
    }

    #[test]
    fn test_ai_review_summary_serialization() {
        let summary = AIReviewSummary {
            total_files_reviewed: 5,
            average_score: 7.8,
            critical_issues: vec!["Issue 1".to_string()],
            common_patterns: vec!["Pattern 1".to_string()],
            best_practices_violations: vec!["Violation 1".to_string()],
            recommended_actions: vec!["Action 1".to_string()],
        };

        // 测试序列化
        let json = serde_json::to_string(&summary).unwrap();
        assert!(json.contains("5"));
        assert!(json.contains("7.8"));

        // 测试反序列化
        let deserialized: AIReviewSummary = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.total_files_reviewed, 5);
        assert_eq!(deserialized.average_score, 7.8);
    }

    #[test]
    fn test_code_review_report_serialization() {
        let report = CodeReviewReport {
            files: vec![],
            summary: ReviewSummary {
                total_files: 1,
                languages_detected: HashMap::new(),
                total_features: 2,
                common_patterns: vec!["test".to_string()],
                overall_risks: vec![],
                test_suggestions: vec![],
            },
            static_analysis_summary: StaticAnalysisSummary {
                tools_used: vec!["test".to_string()],
                total_issues: 0,
                issues_by_severity: HashMap::new(),
                issues_by_tool: HashMap::new(),
                execution_time: std::time::Duration::from_millis(100),
                tools_unavailable: vec![],
            },
            ai_review_summary: None,
        };

        // 测试序列化
        let json = serde_json::to_string(&report).unwrap();
        assert!(json.contains("1"));
        assert!(json.contains("2"));

        // 测试反序列化
        let deserialized: CodeReviewReport = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.summary.total_files, 1);
        assert_eq!(deserialized.summary.total_features, 2);
    }
}