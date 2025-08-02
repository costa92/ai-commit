use crate::languages::{Language, LanguageAnalysisResult, LanguageAnalyzerFactory};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 代码审查服务，用于自动识别语言并进行分析
pub struct CodeReviewService;

/// 文件分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileAnalysisResult {
    pub file_path: String,
    pub language: Language,
    pub analysis: LanguageAnalysisResult,
}

/// 代码审查报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeReviewReport {
    pub files: Vec<FileAnalysisResult>,
    pub summary: ReviewSummary,
}

/// 审查摘要
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewSummary {
    pub total_files: usize,
    pub languages_detected: HashMap<Language, usize>,
    pub total_features: usize,
    pub common_patterns: Vec<String>,
    pub overall_risks: Vec<String>,
    pub test_suggestions: Vec<String>,
}

impl CodeReviewService {
    pub fn new() -> Self {
        CodeReviewService
    }

    /// 自动识别文件语言
    pub fn detect_language(&self, file_path: &str) -> Language {
        Language::from_file_path(file_path)
    }

    /// 分析单个文件
    pub fn analyze_file(&self, file_path: &str, file_content: &str) -> FileAnalysisResult {
        let language = self.detect_language(file_path);
        let analyzer = LanguageAnalyzerFactory::create_analyzer(&language);

        // 将文件内容按行分割并分析
        let lines: Vec<&str> = file_content.lines().collect();

        // 只分析新增的行（在实际使用中，这些应该从git diff中提取）
        let added_lines: Vec<String> = lines.iter().map(|&line| line.to_string()).collect();

        let analysis = analyzer.analyze_file_changes(file_path, &added_lines);

        FileAnalysisResult {
            file_path: file_path.to_string(),
            language,
            analysis,
        }
    }

    /// 从Git diff分析变更的文件
    pub fn analyze_git_diff(&self, diff_content: &str) -> Vec<FileAnalysisResult> {
        let mut results = Vec::new();
        let parsed_diff = self.parse_git_diff(diff_content);

        for (file_path, added_lines) in parsed_diff {
            let language = self.detect_language(&file_path);
            let analyzer = LanguageAnalyzerFactory::create_analyzer(&language);
            let analysis = analyzer.analyze_file_changes(&file_path, &added_lines);

            results.push(FileAnalysisResult {
                file_path,
                language,
                analysis,
            });
        }

        results
    }

    /// 解析Git diff内容
    fn parse_git_diff(&self, diff_content: &str) -> Vec<(String, Vec<String>)> {
        let mut files = Vec::new();
        let mut current_file = String::new();
        let mut added_lines = Vec::new();

        for line in diff_content.lines() {
            if line.starts_with("diff --git") {
                // 保存前一个文件的结果
                if !current_file.is_empty() && !added_lines.is_empty() {
                    files.push((current_file.clone(), added_lines.clone()));
                }

                // 开始新文件
                current_file = String::new();
                added_lines.clear();
            } else if line.starts_with("+++") {
                // 提取文件路径
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    current_file = parts[1].trim_start_matches("b/").to_string();
                }
            } else if line.starts_with('+') && !line.starts_with("+++") {
                // 这是新增的行
                added_lines.push(line[1..].to_string()); // 移除'+'前缀
            }
        }

        // 添加最后一个文件
        if !current_file.is_empty() && !added_lines.is_empty() {
            files.push((current_file, added_lines));
        }

        files
    }

    /// 分析多个文件并生成报告
    pub fn analyze_files(&self, file_paths: &[String]) -> CodeReviewReport {
        let mut file_results = Vec::new();

        for file_path in file_paths {
            // 尝试读取实际文件内容
            let file_content = match std::fs::read_to_string(file_path) {
                Ok(content) => content,
                Err(_) => {
                    // 如果读取失败，跳过该文件
                    eprintln!("警告: 无法读取文件 {}", file_path);
                    continue;
                }
            };

            let result = self.analyze_file(file_path, &file_content);
            file_results.push(result);
        }

        let summary = self.generate_summary(&file_results);

        CodeReviewReport {
            files: file_results,
            summary,
        }
    }

    /// 从Git diff生成完整的代码审查报告
    pub fn review_git_changes(&self, diff_content: &str) -> CodeReviewReport {
        let file_results = self.analyze_git_diff(diff_content);
        let summary = self.generate_summary(&file_results);

        CodeReviewReport {
            files: file_results,
            summary,
        }
    }

    /// 生成审查摘要
    fn generate_summary(&self, file_results: &[FileAnalysisResult]) -> ReviewSummary {
        let mut languages_detected = HashMap::new();
        let mut total_features = 0;
        let mut all_patterns = Vec::new();
        let mut all_risks = Vec::new();
        let mut all_test_suggestions = Vec::new();

        for result in file_results {
            // 统计语言
            *languages_detected
                .entry(result.language.clone())
                .or_insert(0) += 1;

            // 统计特征
            total_features += result.analysis.features.len();

            // 收集模式
            all_patterns.extend(result.analysis.change_patterns.clone());

            // 收集风险和测试建议
            let analyzer = LanguageAnalyzerFactory::create_analyzer(&result.language);
            all_risks.extend(analyzer.assess_risks(&result.analysis.features));
            all_test_suggestions
                .extend(analyzer.generate_test_suggestions(&result.analysis.features));
        }

        // 去重并排序
        all_patterns.sort();
        all_patterns.dedup();
        all_risks.sort();
        all_risks.dedup();
        all_test_suggestions.sort();
        all_test_suggestions.dedup();

        ReviewSummary {
            total_files: file_results.len(),
            languages_detected,
            total_features,
            common_patterns: all_patterns,
            overall_risks: all_risks,
            test_suggestions: all_test_suggestions,
        }
    }

    /// 格式化报告为可读文本，支持内容长度优化
    pub fn format_report(&self, report: &CodeReviewReport) -> String {
        let initial_report = self.format_report_internal(report, false);

        // 检查报告长度是否过长（超过 10000 字符）
        if initial_report.len() > 10000 {
            // 生成优化版本的报告
            self.format_report_internal(report, true)
        } else {
            initial_report
        }
    }

    /// 内部格式化方法，支持优化模式
    fn format_report_internal(&self, report: &CodeReviewReport, optimize: bool) -> String {
        let mut output = String::new();

        output.push_str("# 代码审查报告\n\n");

        // 摘要部分
        output.push_str("## 📊 摘要统计\n\n");
        output.push_str(&format!("- **总文件数**: {}\n", report.summary.total_files));
        output.push_str(&format!(
            "- **检测到的特征数**: {}\n",
            report.summary.total_features
        ));
        output.push_str("- **检测到的语言**:\n");

        for (language, count) in &report.summary.languages_detected {
            output.push_str(&format!("  - {}: {} 个文件\n", language.as_str(), count));
        }
        output.push('\n');

        // 变更模式 - 优化模式下限制数量
        if !report.summary.common_patterns.is_empty() {
            output.push_str("## 🔍 变更模式分析\n\n");
            let patterns = if optimize {
                // 优化模式：只显示前5个最重要的模式
                report
                    .summary
                    .common_patterns
                    .iter()
                    .take(5)
                    .collect::<Vec<_>>()
            } else {
                report.summary.common_patterns.iter().collect::<Vec<_>>()
            };

            for pattern in patterns {
                output.push_str(&format!("- {}\n", pattern));
            }

            if optimize && report.summary.common_patterns.len() > 5 {
                output.push_str(&format!(
                    "- ... 及其他 {} 个变更模式\n",
                    report.summary.common_patterns.len() - 5
                ));
            }
            output.push('\n');
        }

        // 风险评估 - 优化模式下限制数量
        if !report.summary.overall_risks.is_empty() {
            output.push_str("## ⚠️  风险评估\n\n");
            let risks = if optimize {
                // 优化模式：只显示前5个最重要的风险
                report
                    .summary
                    .overall_risks
                    .iter()
                    .take(5)
                    .collect::<Vec<_>>()
            } else {
                report.summary.overall_risks.iter().collect::<Vec<_>>()
            };

            for risk in risks {
                output.push_str(&format!("- {}\n", risk));
            }

            if optimize && report.summary.overall_risks.len() > 5 {
                output.push_str(&format!(
                    "- ... 及其他 {} 个风险项\n",
                    report.summary.overall_risks.len() - 5
                ));
            }
            output.push('\n');
        }

        // 测试建议 - 优化模式下限制数量
        if !report.summary.test_suggestions.is_empty() {
            output.push_str("## 🧪 测试建议\n\n");
            let suggestions = if optimize {
                // 优化模式：只显示前8个最重要的建议
                report
                    .summary
                    .test_suggestions
                    .iter()
                    .take(8)
                    .collect::<Vec<_>>()
            } else {
                report.summary.test_suggestions.iter().collect::<Vec<_>>()
            };

            for suggestion in suggestions {
                output.push_str(&format!("- {}\n", suggestion));
            }

            if optimize && report.summary.test_suggestions.len() > 8 {
                output.push_str(&format!(
                    "- ... 及其他 {} 个测试建议\n",
                    report.summary.test_suggestions.len() - 8
                ));
            }
            output.push('\n');
        }

        // 详细文件分析 - 优化模式下限制详细程度
        output.push_str("## 📁 详细文件分析\n\n");

        let files_to_show = if optimize {
            // 优化模式：只显示前10个文件
            report.files.iter().take(10).collect::<Vec<_>>()
        } else {
            report.files.iter().collect::<Vec<_>>()
        };

        for file_result in files_to_show {
            output.push_str(&format!(
                "### {} ({})\n\n",
                file_result.file_path,
                file_result.language.as_str()
            ));

            if !file_result.analysis.features.is_empty() {
                output.push_str("**检测到的特征**:\n");

                let features_to_show = if optimize {
                    // 优化模式：每个文件最多显示5个特征
                    file_result
                        .analysis
                        .features
                        .iter()
                        .take(5)
                        .collect::<Vec<_>>()
                } else {
                    file_result.analysis.features.iter().collect::<Vec<_>>()
                };

                for feature in features_to_show {
                    output.push_str(&format!(
                        "- **{}**: {} (行 {})\n",
                        feature.feature_type,
                        feature.name,
                        feature.line_number.unwrap_or(0)
                    ));
                }

                if optimize && file_result.analysis.features.len() > 5 {
                    output.push_str(&format!(
                        "- ... 及其他 {} 个特征\n",
                        file_result.analysis.features.len() - 5
                    ));
                }
                output.push('\n');
            }

            if !file_result.analysis.scope_suggestions.is_empty() {
                output.push_str("**作用域建议**: ");
                if optimize {
                    // 优化模式：最多显示前3个作用域建议
                    let suggestions_to_show: Vec<&String> = file_result
                        .analysis
                        .scope_suggestions
                        .iter()
                        .take(3)
                        .collect();
                    output.push_str(
                        &suggestions_to_show
                            .iter()
                            .map(|s| s.as_str())
                            .collect::<Vec<&str>>()
                            .join(", "),
                    );

                    if file_result.analysis.scope_suggestions.len() > 3 {
                        output.push_str(&format!(
                            ", ... (+{})",
                            file_result.analysis.scope_suggestions.len() - 3
                        ));
                    }
                } else {
                    output.push_str(&file_result.analysis.scope_suggestions.join(", "));
                }
                output.push_str("\n\n");
            }
        }

        // 如果优化模式下有省略的文件，添加说明
        if optimize && report.files.len() > 10 {
            output.push_str(&format!(
                "### ... 及其他 {} 个文件\n\n",
                report.files.len() - 10
            ));
            output.push_str("*为保持报告简洁，详细分析已限制在前10个文件。如需查看完整报告，请使用 `--review-format json` 获取完整数据。*\n\n");
        }

        output
    }
}

impl Default for CodeReviewService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_detection() {
        let service = CodeReviewService::new();

        assert_eq!(service.detect_language("main.go"), Language::Go);
        assert_eq!(
            service.detect_language("component.tsx"),
            Language::TypeScript
        );
        assert_eq!(service.detect_language("script.js"), Language::JavaScript);
        assert_eq!(service.detect_language("lib.rs"), Language::Rust);
        assert_eq!(service.detect_language("config.json"), Language::Unknown);
    }

    #[test]
    fn test_file_analysis() {
        let service = CodeReviewService::new();
        let go_content = r#"
package main

import "fmt"

func main() {
    fmt.Println("Hello, World!")
}
"#;

        let result = service.analyze_file("main.go", go_content);
        assert_eq!(result.language, Language::Go);
        assert_eq!(result.file_path, "main.go");
        assert!(!result.analysis.features.is_empty());
    }

    #[test]
    fn test_git_diff_parsing() {
        let service = CodeReviewService::new();
        let diff_content = r#"
diff --git a/src/main.go b/src/main.go
index 1234567..abcdefg 100644
--- a/src/main.go
+++ b/src/main.go
@@ -1,3 +1,5 @@
 package main
 
+import "fmt"
+
 func main() {
"#;

        let results = service.analyze_git_diff(diff_content);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].file_path, "src/main.go");
        assert_eq!(results[0].language, Language::Go);
    }

    #[test]
    fn test_report_generation() {
        let service = CodeReviewService::new();
        let files = vec![
            "src/main.go".to_string(),
            "components/Button.tsx".to_string(),
            "utils/helper.js".to_string(),
        ];

        let report = service.analyze_files(&files);
        assert_eq!(report.files.len(), 0); // 文件不存在，所以结果为0
        assert!(report.summary.languages_detected.is_empty());

        let formatted = service.format_report(&report);
        assert!(formatted.contains("代码审查报告"));
        assert!(formatted.contains("摘要统计"));
    }

    #[test]
    fn test_content_length_optimization() {
        let service = CodeReviewService::new();

        // 创建一个包含大量内容的模拟报告
        let mut large_report = CodeReviewReport {
            files: Vec::new(),
            summary: ReviewSummary {
                total_files: 50,
                languages_detected: {
                    let mut map = HashMap::new();
                    map.insert(Language::Rust, 30);
                    map.insert(Language::Go, 20);
                    map
                },
                total_features: 500,
                common_patterns: (0..20)
                    .map(|i| {
                        format!(
                            "变更模式 {} - 这是一个很长的描述，包含了详细的技术细节和实现建议",
                            i
                        )
                    })
                    .collect(),
                overall_risks: (0..15)
                    .map(|i| {
                        format!(
                            "风险项 {} - 这是一个详细的风险描述，包含了潜在的影响和建议的缓解措施",
                            i
                        )
                    })
                    .collect(),
                test_suggestions: (0..25)
                    .map(|i| {
                        format!(
                            "测试建议 {} - 这是一个详细的测试策略建议，包含了具体的实现方法",
                            i
                        )
                    })
                    .collect(),
            },
        };

        // 添加大量文件分析结果
        for i in 0..20 {
            large_report.files.push(FileAnalysisResult {
                file_path: format!("src/file_{}.rs", i),
                language: Language::Rust,
                analysis: LanguageAnalysisResult {
                    language: Language::Rust,
                    features: (0..10)
                        .map(|j| crate::languages::LanguageFeature {
                            feature_type: "function".to_string(),
                            name: format!("function_{}_{}", i, j),
                            line_number: Some(j + 1),
                            description: format!("这是一个详细的函数描述 {} {}", i, j),
                        })
                        .collect(),
                    scope_suggestions: vec![
                        "rust".to_string(),
                        "module".to_string(),
                        format!("file_{}", i),
                        "function".to_string(),
                        "implementation".to_string(),
                    ],
                    change_patterns: vec![format!("模式 {}", i)],
                },
            });
        }

        // 测试初始报告长度
        let initial_report = service.format_report_internal(&large_report, false);
        assert!(initial_report.len() > 10000, "初始报告应该超过长度阈值");

        // 测试优化报告长度
        let optimized_report = service.format_report_internal(&large_report, true);
        assert!(
            optimized_report.len() < initial_report.len(),
            "优化后的报告应该更短"
        );

        // 测试自动优化功能
        let auto_optimized = service.format_report(&large_report);
        assert_eq!(
            auto_optimized, optimized_report,
            "自动优化应该产生相同的结果"
        );

        // 验证优化报告包含省略提示
        assert!(
            optimized_report.contains("及其他"),
            "优化报告应该包含省略提示"
        );
        assert!(
            optimized_report.contains("为保持报告简洁"),
            "优化报告应该包含说明文字"
        );
    }

    #[test]
    fn test_small_content_no_optimization() {
        let service = CodeReviewService::new();

        // 创建一个小的报告
        let small_report = CodeReviewReport {
            files: vec![FileAnalysisResult {
                file_path: "src/main.rs".to_string(),
                language: Language::Rust,
                analysis: LanguageAnalysisResult {
                    language: Language::Rust,
                    features: vec![crate::languages::LanguageFeature {
                        feature_type: "function".to_string(),
                        name: "main".to_string(),
                        line_number: Some(1),
                        description: "主函数".to_string(),
                    }],
                    scope_suggestions: vec!["main".to_string()],
                    change_patterns: vec!["函数变更".to_string()],
                },
            }],
            summary: ReviewSummary {
                total_files: 1,
                languages_detected: {
                    let mut map = HashMap::new();
                    map.insert(Language::Rust, 1);
                    map
                },
                total_features: 1,
                common_patterns: vec!["函数变更".to_string()],
                overall_risks: vec!["无风险".to_string()],
                test_suggestions: vec!["添加测试".to_string()],
            },
        };

        let report = service.format_report(&small_report);
        let unoptimized_report = service.format_report_internal(&small_report, false);

        // 小报告应该不触发优化
        assert_eq!(report, unoptimized_report, "小报告不应该被优化");
        assert!(!report.contains("及其他"), "小报告不应该包含省略提示");
    }
}
