use std::collections::HashMap;
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use regex::Regex;
use once_cell::sync::Lazy;

use crate::ai::manager::{AIServiceManager, AIRequest, AIRequestType};
use crate::ai::reviewers::templates::ReviewPromptTemplates;
use crate::languages::{Language, LanguageFeature};

/// 语言特定 AI 审查器
pub struct LanguageSpecificReviewer {
    ai_service: AIServiceManager,
}

/// AI 审查结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIReviewResult {
    /// 文件路径
    pub file_path: String,

    /// 编程语言
    pub language: Language,

    /// 质量评分 (1-10)
    pub quality_score: f32,

    /// 主要问题列表
    pub issues: Vec<AIReviewIssue>,

    /// 优化建议列表
    pub suggestions: Vec<AIReviewSuggestion>,

    /// 最佳实践建议
    pub best_practices: Vec<String>,

    /// 学习资源推荐
    pub learning_resources: Vec<LearningResource>,

    /// 审查摘要
    pub summary: String,

    /// 语言特定分析结果
    pub language_specific_analysis: Option<LanguageSpecificAnalysis>,
}

/// AI 审查问题
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIReviewIssue {
    /// 问题类型
    pub issue_type: String,

    /// 问题描述
    pub description: String,

    /// 问题位置（行号）
    pub line_number: Option<u32>,

    /// 严重程度
    pub severity: IssueSeverity,

    /// 修复建议
    pub fix_suggestion: String,

    /// 代码示例
    pub code_example: Option<String>,
}

/// AI 审查建议
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIReviewSuggestion {
    /// 建议类型
    pub suggestion_type: String,

    /// 建议描述
    pub description: String,

    /// 优化原因
    pub reason: String,

    /// 实施方案
    pub implementation: String,

    /// 预期效果
    pub expected_impact: String,

    /// 代码示例
    pub code_example: Option<String>,
}

/// 学习资源
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningResource {
    /// 资源标题
    pub title: String,

    /// 资源链接
    pub url: String,

    /// 资源描述
    pub description: String,

    /// 资源类型
    pub resource_type: ResourceType,
}

/// 问题严重程度
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum IssueSeverity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

/// 资源类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResourceType {
    Documentation,
    Tutorial,
    Article,
    Video,
    Book,
    Tool,
}

/// 语言特定分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LanguageSpecificAnalysis {
    Go(GoAnalysisResult),
    Rust(RustAnalysisResult),
    TypeScript(TypeScriptAnalysisResult),
}

/// Go 语言特定分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoAnalysisResult {
    /// 错误处理分析
    pub error_handling: String,

    /// 并发安全分析
    pub concurrency_safety: String,

    /// 性能分析
    pub performance_analysis: String,

    /// 内存管理分析
    pub memory_management: String,
}

/// Rust 语言特定分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RustAnalysisResult {
    /// 所有权检查
    pub ownership_check: String,

    /// 借用检查
    pub borrow_check: String,

    /// 生命周期分析
    pub lifetime_analysis: String,

    /// 内存安全分析
    pub memory_safety: String,
}

/// TypeScript 语言特定分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeScriptAnalysisResult {
    /// 类型定义分析
    pub type_definition: String,

    /// 类型推断分析
    pub type_inference: String,

    /// 泛型使用分析
    pub generic_usage: String,

    /// 异步代码分析
    pub async_analysis: String,
}

// 正则表达式用于解析 AI 响应
static QUALITY_SCORE_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)(?:总分|评分|score)[:：]\s*(\d+(?:\.\d+)?)")
        .expect("Failed to compile quality score regex")
});

static ISSUE_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)(?:问题|issue)[:：]\s*(.+?)(?:\n|$)")
        .expect("Failed to compile issue regex")
});

impl LanguageSpecificReviewer {
    /// 创建新的语言特定审查器
    pub fn new(ai_service: AIServiceManager) -> Self {
        Self { ai_service }
    }

    /// 审查代码
    pub async fn review_code(
        &self,
        file_path: &str,
        language: &Language,
        code_content: &str,
        features: &[LanguageFeature],
    ) -> Result<AIReviewResult> {
        match language {
            Language::Go => self.review_go_code(file_path, code_content, features).await,
            Language::Rust => self.review_rust_code(file_path, code_content, features).await,
            Language::TypeScript | Language::JavaScript => {
                self.review_typescript_code(file_path, code_content, features).await
            }
            _ => self.review_generic_code(file_path, language, code_content).await,
        }
    }

    /// 审查 Go 代码
    async fn review_go_code(
        &self,
        file_path: &str,
        code_content: &str,
        _features: &[LanguageFeature],
    ) -> Result<AIReviewResult> {
        let prompt = ReviewPromptTemplates::go_review_template()
            .replace("{file_path}", file_path)
            .replace("{code_content}", code_content);

        let request = AIRequest {
            prompt,
            model: None,
            temperature: Some(0.3),
            max_tokens: Some(2048),
            context: crate::ai::manager::AIRequestContext {
                file_path: file_path.to_string(),
                language: "go".to_string(),
                request_type: AIRequestType::CodeReview,
                metadata: HashMap::new(),
            },
        };

        let response = self.ai_service.analyze_code(&request).await?;
        let mut result = self.parse_review_response(&response.content, file_path, &Language::Go)?;

        // 添加 Go 特定分析
        result.language_specific_analysis = Some(LanguageSpecificAnalysis::Go(
            self.extract_go_analysis(&response.content)
        ));

        Ok(result)
    }

    /// 审查 Rust 代码
    async fn review_rust_code(
        &self,
        file_path: &str,
        code_content: &str,
        _features: &[LanguageFeature],
    ) -> Result<AIReviewResult> {
        let prompt = ReviewPromptTemplates::rust_review_template()
            .replace("{file_path}", file_path)
            .replace("{code_content}", code_content);

        let request = AIRequest {
            prompt,
            model: None,
            temperature: Some(0.3),
            max_tokens: Some(2048),
            context: crate::ai::manager::AIRequestContext {
                file_path: file_path.to_string(),
                language: "rust".to_string(),
                request_type: AIRequestType::CodeReview,
                metadata: HashMap::new(),
            },
        };

        let response = self.ai_service.analyze_code(&request).await?;
        let mut result = self.parse_review_response(&response.content, file_path, &Language::Rust)?;

        // 添加 Rust 特定分析
        result.language_specific_analysis = Some(LanguageSpecificAnalysis::Rust(
            self.extract_rust_analysis(&response.content)
        ));

        Ok(result)
    }

    /// 审查 TypeScript 代码
    async fn review_typescript_code(
        &self,
        file_path: &str,
        code_content: &str,
        _features: &[LanguageFeature],
    ) -> Result<AIReviewResult> {
        let prompt = ReviewPromptTemplates::typescript_review_template()
            .replace("{file_path}", file_path)
            .replace("{code_content}", code_content);

        let request = AIRequest {
            prompt,
            model: None,
            temperature: Some(0.3),
            max_tokens: Some(2048),
            context: crate::ai::manager::AIRequestContext {
                file_path: file_path.to_string(),
                language: "typescript".to_string(),
                request_type: AIRequestType::CodeReview,
                metadata: HashMap::new(),
            },
        };

        let response = self.ai_service.analyze_code(&request).await?;
        let mut result = self.parse_review_response(&response.content, file_path, &Language::TypeScript)?;

        // 添加 TypeScript 特定分析
        result.language_specific_analysis = Some(LanguageSpecificAnalysis::TypeScript(
            self.extract_typescript_analysis(&response.content)
        ));

        Ok(result)
    }

    /// 审查通用代码
    async fn review_generic_code(
        &self,
        file_path: &str,
        language: &Language,
        code_content: &str,
    ) -> Result<AIReviewResult> {
        let language_str = format!("{:?}", language).to_lowercase();
        let prompt = ReviewPromptTemplates::generic_review_template()
            .replace("{file_path}", file_path)
            .replace("{language}", &language_str)
            .replace("{code_content}", code_content);

        let request = AIRequest {
            prompt,
            model: None,
            temperature: Some(0.3),
            max_tokens: Some(2048),
            context: crate::ai::manager::AIRequestContext {
                file_path: file_path.to_string(),
                language: language_str,
                request_type: AIRequestType::CodeReview,
                metadata: HashMap::new(),
            },
        };

        let response = self.ai_service.analyze_code(&request).await?;
        self.parse_review_response(&response.content, file_path, language)
    }

    /// 解析审查响应
    fn parse_review_response(
        &self,
        response: &str,
        file_path: &str,
        language: &Language,
    ) -> Result<AIReviewResult> {
        // 提取质量评分
        let quality_score = self.extract_quality_score(response);

        // 提取问题列表
        let issues = self.extract_issues(response);

        // 提取建议列表
        let suggestions = self.extract_suggestions(response);

        // 提取最佳实践
        let best_practices = self.extract_best_practices(response);

        // 提取学习资源
        let learning_resources = self.extract_learning_resources(response, language);

        // 生成摘要
        let summary = self.generate_summary(&issues, &suggestions, quality_score);

        Ok(AIReviewResult {
            file_path: file_path.to_string(),
            language: language.clone(),
            quality_score,
            issues,
            suggestions,
            best_practices,
            learning_resources,
            summary,
            language_specific_analysis: None,
        })
    }

    /// 提取质量评分
    fn extract_quality_score(&self, response: &str) -> f32 {
        if let Some(captures) = QUALITY_SCORE_REGEX.captures(response) {
            if let Some(score_str) = captures.get(1) {
                if let Ok(score) = score_str.as_str().parse::<f32>() {
                    return score.clamp(1.0, 10.0);
                }
            }
        }
        5.0 // 默认评分
    }

    /// 提取问题列表
    fn extract_issues(&self, response: &str) -> Vec<AIReviewIssue> {
        let mut issues = Vec::new();

        // 简单的问题提取逻辑，实际实现可能需要更复杂的解析
        let lines: Vec<&str> = response.lines().collect();
        let mut in_issues_section = false;

        for line in lines {
            if line.contains("主要问题") || line.contains("问题") {
                in_issues_section = true;
                continue;
            }

            if in_issues_section && (line.contains("优化建议") || line.contains("最佳实践")) {
                break;
            }

            if in_issues_section && line.trim().starts_with("1.") || line.trim().starts_with("-") {
                if let Some(issue) = self.parse_issue_line(line) {
                    issues.push(issue);
                }
            }
        }

        issues
    }

    /// 解析问题行
    fn parse_issue_line(&self, line: &str) -> Option<AIReviewIssue> {
        // 简化的问题解析逻辑
        if line.contains("**") {
            let parts: Vec<&str> = line.split("**").collect();
            if parts.len() >= 3 {
                let issue_type = parts[1].to_string();
                let description = parts.get(2).unwrap_or(&"").trim_start_matches(" - ").to_string();

                return Some(AIReviewIssue {
                    issue_type,
                    description,
                    line_number: None,
                    severity: IssueSeverity::Medium,
                    fix_suggestion: "请参考 AI 建议进行修复".to_string(),
                    code_example: None,
                });
            }
        }
        None
    }

    /// 提取建议列表
    fn extract_suggestions(&self, response: &str) -> Vec<AIReviewSuggestion> {
        let mut suggestions = Vec::new();

        // 简化的建议提取逻辑
        let lines: Vec<&str> = response.lines().collect();
        let mut in_suggestions_section = false;

        for line in lines {
            if line.contains("优化建议") || line.contains("建议") {
                in_suggestions_section = true;
                continue;
            }

            if in_suggestions_section && (line.contains("最佳实践") || line.contains("学习资源")) {
                break;
            }

            if in_suggestions_section && (line.trim().starts_with("1.") || line.trim().starts_with("-")) {
                if let Some(suggestion) = self.parse_suggestion_line(line) {
                    suggestions.push(suggestion);
                }
            }
        }

        suggestions
    }

    /// 解析建议行
    fn parse_suggestion_line(&self, line: &str) -> Option<AIReviewSuggestion> {
        // 简化的建议解析逻辑
        if line.contains("**") {
            let parts: Vec<&str> = line.split("**").collect();
            if parts.len() >= 3 {
                let suggestion_type = parts[1].to_string();
                let description = parts.get(2).unwrap_or(&"").trim_start_matches(" - ").to_string();

                return Some(AIReviewSuggestion {
                    suggestion_type,
                    description,
                    reason: "提升代码质量".to_string(),
                    implementation: "请参考 AI 建议实施".to_string(),
                    expected_impact: "改善代码质量和性能".to_string(),
                    code_example: None,
                });
            }
        }
        None
    }

    /// 提取最佳实践
    fn extract_best_practices(&self, response: &str) -> Vec<String> {
        let mut practices = Vec::new();

        let lines: Vec<&str> = response.lines().collect();
        let mut in_practices_section = false;

        for line in lines {
            if line.contains("最佳实践") {
                in_practices_section = true;
                continue;
            }

            if in_practices_section && line.contains("学习资源") {
                break;
            }

            if in_practices_section && (line.trim().starts_with("-") || line.trim().starts_with("*")) {
                let practice = line.trim().trim_start_matches("-").trim_start_matches("*").trim();
                if !practice.is_empty() {
                    practices.push(practice.to_string());
                }
            }
        }

        practices
    }

    /// 提取学习资源
    fn extract_learning_resources(&self, response: &str, language: &Language) -> Vec<LearningResource> {
        let mut resources = Vec::new();

        // 根据语言添加默认学习资源
        match language {
            Language::Go => {
                resources.push(LearningResource {
                    title: "Go 官方文档".to_string(),
                    url: "https://golang.org/doc/".to_string(),
                    description: "Go 语言官方文档和教程".to_string(),
                    resource_type: ResourceType::Documentation,
                });
                resources.push(LearningResource {
                    title: "Effective Go".to_string(),
                    url: "https://golang.org/doc/effective_go".to_string(),
                    description: "Go 语言最佳实践指南".to_string(),
                    resource_type: ResourceType::Article,
                });
            }
            Language::Rust => {
                resources.push(LearningResource {
                    title: "The Rust Programming Language".to_string(),
                    url: "https://doc.rust-lang.org/book/".to_string(),
                    description: "Rust 官方教程书籍".to_string(),
                    resource_type: ResourceType::Book,
                });
                resources.push(LearningResource {
                    title: "Rust by Example".to_string(),
                    url: "https://doc.rust-lang.org/rust-by-example/".to_string(),
                    description: "通过示例学习 Rust".to_string(),
                    resource_type: ResourceType::Tutorial,
                });
            }
            Language::TypeScript => {
                resources.push(LearningResource {
                    title: "TypeScript 官方文档".to_string(),
                    url: "https://www.typescriptlang.org/docs/".to_string(),
                    description: "TypeScript 官方文档".to_string(),
                    resource_type: ResourceType::Documentation,
                });
                resources.push(LearningResource {
                    title: "TypeScript Deep Dive".to_string(),
                    url: "https://basarat.gitbook.io/typescript/".to_string(),
                    description: "TypeScript 深入学习指南".to_string(),
                    resource_type: ResourceType::Book,
                });
            }
            _ => {
                resources.push(LearningResource {
                    title: "编程最佳实践".to_string(),
                    url: "https://github.com/topics/best-practices".to_string(),
                    description: "通用编程最佳实践资源".to_string(),
                    resource_type: ResourceType::Article,
                });
            }
        }

        resources
    }

    /// 生成摘要
    fn generate_summary(&self, issues: &[AIReviewIssue], suggestions: &[AIReviewSuggestion], quality_score: f32) -> String {
        let issue_count = issues.len();
        let suggestion_count = suggestions.len();

        let quality_level = match quality_score {
            9.0..=10.0 => "优秀",
            7.0..=8.9 => "良好",
            5.0..=6.9 => "一般",
            3.0..=4.9 => "较差",
            _ => "很差",
        };

        format!(
            "代码质量评分：{:.1}/10 ({})\n发现 {} 个问题，提供 {} 条优化建议。",
            quality_score, quality_level, issue_count, suggestion_count
        )
    }

    /// 提取 Go 特定分析
    fn extract_go_analysis(&self, response: &str) -> GoAnalysisResult {
        GoAnalysisResult {
            error_handling: self.extract_section(response, "错误处理").unwrap_or_else(|| "未检测到特定的错误处理问题".to_string()),
            concurrency_safety: self.extract_section(response, "并发安全").unwrap_or_else(|| "未检测到并发安全问题".to_string()),
            performance_analysis: self.extract_section(response, "性能").unwrap_or_else(|| "未检测到明显的性能问题".to_string()),
            memory_management: self.extract_section(response, "内存").unwrap_or_else(|| "内存使用看起来合理".to_string()),
        }
    }

    /// 提取 Rust 特定分析
    fn extract_rust_analysis(&self, response: &str) -> RustAnalysisResult {
        RustAnalysisResult {
            ownership_check: self.extract_section(response, "所有权").unwrap_or_else(|| "所有权使用正确".to_string()),
            borrow_check: self.extract_section(response, "借用").unwrap_or_else(|| "借用检查通过".to_string()),
            lifetime_analysis: self.extract_section(response, "生命周期").unwrap_or_else(|| "生命周期管理合理".to_string()),
            memory_safety: self.extract_section(response, "内存安全").unwrap_or_else(|| "内存安全性良好".to_string()),
        }
    }

    /// 提取 TypeScript 特定分析
    fn extract_typescript_analysis(&self, response: &str) -> TypeScriptAnalysisResult {
        TypeScriptAnalysisResult {
            type_definition: self.extract_section(response, "类型定义").unwrap_or_else(|| "类型定义合理".to_string()),
            type_inference: self.extract_section(response, "类型推断").unwrap_or_else(|| "类型推断正常".to_string()),
            generic_usage: self.extract_section(response, "泛型").unwrap_or_else(|| "泛型使用恰当".to_string()),
            async_analysis: self.extract_section(response, "异步").unwrap_or_else(|| "异步代码处理正确".to_string()),
        }
    }

    /// 提取响应中的特定部分
    fn extract_section(&self, response: &str, section_name: &str) -> Option<String> {
        let lines: Vec<&str> = response.lines().collect();
        let mut in_section = false;
        let mut section_content = Vec::new();

        for line in lines {
            if line.contains(section_name) {
                in_section = true;
                continue;
            }

            if in_section && (line.starts_with("###") || line.starts_with("##")) {
                break;
            }

            if in_section && !line.trim().is_empty() {
                section_content.push(line.trim());
            }
        }

        if section_content.is_empty() {
            None
        } else {
            Some(section_content.join(" "))
        }
    }
}

/// Go AI 审查器
pub type GoAIReviewer = LanguageSpecificReviewer;

/// Rust AI 审查器
pub type RustAIReviewer = LanguageSpecificReviewer;

/// TypeScript AI 审查器
pub type TypeScriptAIReviewer = LanguageSpecificReviewer;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::manager::AIConfig;

    #[test]
    fn test_extract_quality_score() {
        let reviewer = create_test_reviewer();

        // 测试不同格式的评分提取
        assert_eq!(reviewer.extract_quality_score("总分：8.5分"), 8.5);
        assert_eq!(reviewer.extract_quality_score("评分: 7"), 7.0);
        assert_eq!(reviewer.extract_quality_score("Score: 9.2"), 9.2);
        assert_eq!(reviewer.extract_quality_score("没有评分"), 5.0); // 默认值
    }

    #[test]
    fn test_parse_issue_line() {
        let reviewer = create_test_reviewer();

        let issue_line = "1. **性能问题** - 循环中存在重复计算";
        let issue = reviewer.parse_issue_line(issue_line);

        assert!(issue.is_some());
        let issue = issue.unwrap();
        assert_eq!(issue.issue_type, "性能问题");
        assert!(issue.description.contains("循环中存在重复计算"));
    }

    #[test]
    fn test_parse_suggestion_line() {
        let reviewer = create_test_reviewer();

        let suggestion_line = "1. **缓存优化** - 添加结果缓存以提升性能";
        let suggestion = reviewer.parse_suggestion_line(suggestion_line);

        assert!(suggestion.is_some());
        let suggestion = suggestion.unwrap();
        assert_eq!(suggestion.suggestion_type, "缓存优化");
        assert!(suggestion.description.contains("添加结果缓存"));
    }

    #[test]
    fn test_generate_summary() {
        let reviewer = create_test_reviewer();

        let issues = vec![
            AIReviewIssue {
                issue_type: "测试问题".to_string(),
                description: "测试描述".to_string(),
                line_number: None,
                severity: IssueSeverity::Medium,
                fix_suggestion: "测试建议".to_string(),
                code_example: None,
            }
        ];

        let suggestions = vec![
            AIReviewSuggestion {
                suggestion_type: "测试建议".to_string(),
                description: "测试描述".to_string(),
                reason: "测试原因".to_string(),
                implementation: "测试实施".to_string(),
                expected_impact: "测试影响".to_string(),
                code_example: None,
            }
        ];

        let summary = reviewer.generate_summary(&issues, &suggestions, 8.0);
        assert!(summary.contains("8.0/10"));
        assert!(summary.contains("良好"));
        assert!(summary.contains("1 个问题"));
        assert!(summary.contains("1 条优化建议"));
    }

    #[test]
    fn test_extract_learning_resources() {
        let reviewer = create_test_reviewer();

        // 测试 Go 语言资源
        let go_resources = reviewer.extract_learning_resources("", &Language::Go);
        assert!(!go_resources.is_empty());
        assert!(go_resources.iter().any(|r| r.title.contains("Go 官方文档")));

        // 测试 Rust 语言资源
        let rust_resources = reviewer.extract_learning_resources("", &Language::Rust);
        assert!(!rust_resources.is_empty());
        assert!(rust_resources.iter().any(|r| r.title.contains("The Rust Programming Language")));

        // 测试 TypeScript 语言资源
        let ts_resources = reviewer.extract_learning_resources("", &Language::TypeScript);
        assert!(!ts_resources.is_empty());
        assert!(ts_resources.iter().any(|r| r.title.contains("TypeScript 官方文档")));
    }

    #[test]
    fn test_extract_section() {
        let reviewer = create_test_reviewer();

        let response = r#"
### 错误处理分析
错误处理实现得很好，使用了适当的错误类型。

### 性能分析
性能方面存在一些优化空间。

### 其他分析
其他内容。
"#;

        let error_handling = reviewer.extract_section(response, "错误处理");
        assert!(error_handling.is_some());
        assert!(error_handling.unwrap().contains("错误处理实现得很好"));

        let performance = reviewer.extract_section(response, "性能");
        assert!(performance.is_some());
        assert!(performance.unwrap().contains("性能方面存在一些优化空间"));

        let non_existent = reviewer.extract_section(response, "不存在的部分");
        assert!(non_existent.is_none());
    }

    fn create_test_reviewer() -> LanguageSpecificReviewer {
        let config = AIConfig::default();
        let ai_service = AIServiceManager::new(config).unwrap();
        LanguageSpecificReviewer::new(ai_service)
    }
}