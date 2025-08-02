use crate::languages::{Language, LanguageAnalyzer, LanguageAnalyzerFactory, LanguageAnalysisResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

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
            *languages_detected.entry(result.language.clone()).or_insert(0) += 1;
            
            // 统计特征
            total_features += result.analysis.features.len();
            
            // 收集模式
            all_patterns.extend(result.analysis.change_patterns.clone());
            
            // 收集风险和测试建议
            let analyzer = LanguageAnalyzerFactory::create_analyzer(&result.language);
            all_risks.extend(analyzer.assess_risks(&result.analysis.features));
            all_test_suggestions.extend(analyzer.generate_test_suggestions(&result.analysis.features));
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

    /// 格式化报告为可读文本
    pub fn format_report(&self, report: &CodeReviewReport) -> String {
        let mut output = String::new();
        
        output.push_str("# 代码审查报告\n\n");
        
        // 摘要部分
        output.push_str("## 📊 摘要统计\n\n");
        output.push_str(&format!("- **总文件数**: {}\n", report.summary.total_files));
        output.push_str(&format!("- **检测到的特征数**: {}\n", report.summary.total_features));
        output.push_str("- **检测到的语言**:\n");
        
        for (language, count) in &report.summary.languages_detected {
            output.push_str(&format!("  - {}: {} 个文件\n", language.as_str(), count));
        }
        output.push_str("\n");
        
        // 变更模式
        if !report.summary.common_patterns.is_empty() {
            output.push_str("## 🔍 变更模式分析\n\n");
            for pattern in &report.summary.common_patterns {
                output.push_str(&format!("- {}\n", pattern));
            }
            output.push_str("\n");
        }
        
        // 风险评估
        if !report.summary.overall_risks.is_empty() {
            output.push_str("## ⚠️  风险评估\n\n");
            for risk in &report.summary.overall_risks {
                output.push_str(&format!("- {}\n", risk));
            }
            output.push_str("\n");
        }
        
        // 测试建议
        if !report.summary.test_suggestions.is_empty() {
            output.push_str("## 🧪 测试建议\n\n");
            for suggestion in &report.summary.test_suggestions {
                output.push_str(&format!("- {}\n", suggestion));
            }
            output.push_str("\n");
        }
        
        // 详细文件分析
        output.push_str("## 📁 详细文件分析\n\n");
        for file_result in &report.files {
            output.push_str(&format!("### {} ({})\n\n", 
                file_result.file_path, 
                file_result.language.as_str()
            ));
            
            if !file_result.analysis.features.is_empty() {
                output.push_str("**检测到的特征**:\n");
                for feature in &file_result.analysis.features {
                    output.push_str(&format!("- **{}**: {} (行 {})\n", 
                        feature.feature_type,
                        feature.name,
                        feature.line_number.unwrap_or(0)
                    ));
                }
                output.push_str("\n");
            }
            
            if !file_result.analysis.scope_suggestions.is_empty() {
                output.push_str("**作用域建议**: ");
                output.push_str(&file_result.analysis.scope_suggestions.join(", "));
                output.push_str("\n\n");
            }
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
        assert_eq!(service.detect_language("component.tsx"), Language::TypeScript);
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
        assert_eq!(report.files.len(), 3);
        assert!(!report.summary.languages_detected.is_empty());
        
        let formatted = service.format_report(&report);
        assert!(formatted.contains("代码审查报告"));
        assert!(formatted.contains("摘要统计"));
    }
}