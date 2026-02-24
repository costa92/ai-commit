use crate::config::Config;
use crate::languages::sensitive_info::{
    SensitiveInfoDetector, SensitiveInfoResult, SensitiveInfoSummary,
};
use crate::languages::static_analysis::{StaticAnalysisResult, StaticAnalysisService};
use crate::languages::{Language, LanguageAnalysisResult, LanguageAnalyzerFactory};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// 导入语言特定的 AI 审查器
use crate::languages::go::GoAIReviewer;
use crate::languages::rust::RustAIReviewer;

/// 增强的代码审查服务，支持 AI 审查和语言特定分析
pub struct CodeReviewService {
    config: Config,
    enable_ai_review: bool,
}

/// 增强的文件分析结果，包含 AI 审查结果和敏感信息检测
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileAnalysisResult {
    pub file_path: String,
    pub language: Language,
    pub analysis: LanguageAnalysisResult,
    pub static_analysis: Vec<StaticAnalysisResult>,
    pub ai_review: Option<AIReviewResult>,
    pub sensitive_info: Option<SensitiveInfoResult>,
}

/// AI 审查结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIReviewResult {
    pub review_type: String,
    pub overall_score: f32,
    pub summary: String,
    pub detailed_feedback: String,
    pub security_score: f32,
    pub performance_score: f32,
    pub maintainability_score: f32,
    pub recommendations: Vec<String>,
    pub learning_resources: Vec<String>,
}

/// 增强的代码审查报告，包含敏感信息检测
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeReviewReport {
    pub files: Vec<FileAnalysisResult>,
    pub summary: ReviewSummary,
    pub static_analysis_summary: StaticAnalysisSummary,
    pub ai_review_summary: Option<AIReviewSummary>,
    pub sensitive_info_summary: Option<SensitiveInfoSummary>,
}

/// AI 审查摘要
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIReviewSummary {
    pub total_files_reviewed: usize,
    pub average_score: f32,
    pub critical_issues: Vec<String>,
    pub common_patterns: Vec<String>,
    pub best_practices_violations: Vec<String>,
    pub recommended_actions: Vec<String>,
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

/// 静态分析摘要
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaticAnalysisSummary {
    pub tools_used: Vec<String>,
    pub total_issues: usize,
    pub issues_by_severity: HashMap<String, usize>,
    pub issues_by_tool: HashMap<String, usize>,
    pub execution_time: std::time::Duration,
    pub tools_unavailable: Vec<String>,
}

/// 审查配置选项
#[derive(Debug, Clone)]
pub struct ReviewOptions {
    pub enable_ai_review: bool,
    pub ai_review_types: Vec<String>, // "general", "security", "performance", "architecture"
    pub include_static_analysis: bool,
    pub detailed_feedback: bool,
    pub language_specific_rules: bool,
    pub enable_sensitive_info_detection: bool,
}

impl Default for ReviewOptions {
    fn default() -> Self {
        Self {
            enable_ai_review: true,
            ai_review_types: vec!["general".to_string()],
            include_static_analysis: true,
            detailed_feedback: true,
            language_specific_rules: true,
            enable_sensitive_info_detection: true,
        }
    }
}

impl CodeReviewService {
    pub fn new() -> Self {
        CodeReviewService {
            config: Config::new(),
            enable_ai_review: false,
        }
    }

    pub fn with_config(config: Config) -> Self {
        CodeReviewService {
            config,
            enable_ai_review: false,
        }
    }

    pub fn with_ai_review(mut self, enable: bool) -> Self {
        self.enable_ai_review = enable;
        self
    }

    /// 自动识别文件语言
    pub fn detect_language(&self, file_path: &str) -> Language {
        Language::from_file_path(file_path)
    }

    /// 分析单个文件（包含静态分析、AI审查和敏感信息检测）
    pub async fn analyze_file_with_options(
        &self,
        file_path: &str,
        file_content: &str,
        options: &ReviewOptions,
    ) -> FileAnalysisResult {
        let language = self.detect_language(file_path);
        let analyzer = LanguageAnalyzerFactory::create_analyzer(&language);

        // 将文件内容按行分割并分析
        let lines: Vec<&str> = file_content.lines().collect();
        let added_lines: Vec<String> = lines.iter().map(|&line| line.to_string()).collect();
        let analysis = analyzer.analyze_file_changes(file_path, &added_lines);

        // 运行静态分析
        let static_analysis = if options.include_static_analysis {
            let static_analysis_service = StaticAnalysisService::new();
            static_analysis_service
                .analyze_file(file_path, &language)
                .await
        } else {
            Vec::new()
        };

        // 运行 AI 审查
        let ai_review = if options.enable_ai_review && self.enable_ai_review {
            self.perform_ai_review(file_content, file_path, &language, &analysis, options)
                .await
                .ok()
        } else {
            None
        };

        // 运行敏感信息检测
        let sensitive_info = if options.enable_sensitive_info_detection {
            let detector = SensitiveInfoDetector::new();
            Some(detector.detect(file_path, file_content))
        } else {
            None
        };

        FileAnalysisResult {
            file_path: file_path.to_string(),
            language,
            analysis,
            static_analysis,
            ai_review,
            sensitive_info,
        }
    }

    /// 执行 AI 审查
    async fn perform_ai_review(
        &self,
        code_content: &str,
        file_path: &str,
        language: &Language,
        analysis: &LanguageAnalysisResult,
        options: &ReviewOptions,
    ) -> anyhow::Result<AIReviewResult> {
        match language {
            Language::Rust => {
                let reviewer = RustAIReviewer::new(self.config.clone());
                let result = reviewer
                    .review_code("comprehensive", &analysis.features, file_path)
                    .await?;

                Ok(AIReviewResult {
                    review_type: "rust_comprehensive".to_string(),
                    overall_score: result.overall_score,
                    summary: "Rust 代码审查完成".to_string(),
                    detailed_feedback: "详细的 Rust 代码审查反馈".to_string(),
                    security_score: 8.0,
                    performance_score: 8.0,
                    maintainability_score: result.overall_score,
                    recommendations: result.recommendations,
                    learning_resources: result.learning_resources,
                })
            }
            Language::Go => {
                let reviewer = GoAIReviewer::new(self.config.clone());
                let result = reviewer
                    .review_code("comprehensive", &analysis.features, file_path)
                    .await?;

                Ok(result)
            }
            Language::TypeScript | Language::JavaScript => {
                // 为 JS/TS 实现通用的 AI 审查
                self.perform_generic_ai_review(code_content, file_path, language, analysis, options)
                    .await
            }
            Language::Unknown => {
                // 通用审查
                self.perform_generic_ai_review(code_content, file_path, language, analysis, options)
                    .await
            }
        }
    }

    /// 通用 AI 审查（用于不支持语言特定审查的语言）
    async fn perform_generic_ai_review(
        &self,
        code_content: &str,
        file_path: &str,
        language: &Language,
        _analysis: &LanguageAnalysisResult,
        _options: &ReviewOptions,
    ) -> anyhow::Result<AIReviewResult> {
        // 构建通用的审查提示词
        let prompt = format!(
            r#"
请对以下 {} 代码进行审查，重点关注：
1. 代码质量和可读性
2. 潜在的安全问题
3. 性能优化建议
4. 最佳实践的应用

代码文件：{}

代码内容：
```
{}
```

请提供详细的审查反馈和改进建议。
"#,
            language.as_str(),
            file_path,
            code_content
        );

        // 调用 AI 服务
        let ai_response = crate::ai::generate_commit_message("", &self.config, &prompt).await?;

        Ok(AIReviewResult {
            review_type: format!("{}_generic", language.as_str()),
            overall_score: 7.5,
            summary: format!("{} 代码通用审查完成", language.as_str()),
            detailed_feedback: ai_response,
            security_score: 7.0,
            performance_score: 7.0,
            maintainability_score: 8.0,
            recommendations: vec!["建议进行更详细的代码审查".to_string()],
            learning_resources: vec![format!(
                "https://developer.mozilla.org/docs/{}",
                language.as_str()
            )],
        })
    }

    /// 从Git diff分析变更的文件（增强版本，包含敏感信息检测）
    pub async fn review_git_changes_with_options(
        &self,
        diff_content: &str,
        options: &ReviewOptions,
    ) -> CodeReviewReport {
        let mut files = Vec::new();
        let mut ai_reviews = Vec::new();
        let parsed_diff = self.parse_git_diff(diff_content);

        for (file_path, added_lines) in parsed_diff {
            let language = self.detect_language(&file_path);
            let analyzer = LanguageAnalyzerFactory::create_analyzer(&language);
            let analysis = analyzer.analyze_file_changes(&file_path, &added_lines);

            // 运行静态分析
            let static_analysis = if options.include_static_analysis {
                let static_analysis_service = StaticAnalysisService::new();
                static_analysis_service
                    .analyze_file(&file_path, &language)
                    .await
            } else {
                Vec::new()
            };

            // 运行 AI 审查
            let ai_review = if options.enable_ai_review && self.enable_ai_review {
                let code_content = added_lines.join("\n");
                match self
                    .perform_ai_review(&code_content, &file_path, &language, &analysis, options)
                    .await
                {
                    Ok(review) => {
                        ai_reviews.push(review.clone());
                        Some(review)
                    }
                    Err(_) => None,
                }
            } else {
                None
            };

            // 运行敏感信息检测
            let sensitive_info = if options.enable_sensitive_info_detection {
                let detector = SensitiveInfoDetector::new();
                let code_content = added_lines.join("\n");
                Some(detector.detect(&file_path, &code_content))
            } else {
                None
            };

            files.push(FileAnalysisResult {
                file_path,
                language,
                analysis,
                static_analysis,
                ai_review,
                sensitive_info,
            });
        }

        let summary = self.generate_summary(&files);
        let static_analysis_summary = self.generate_static_analysis_summary(&files);
        let ai_review_summary = if !ai_reviews.is_empty() {
            Some(self.generate_ai_review_summary(&ai_reviews))
        } else {
            None
        };
        let sensitive_info_summary = self.generate_sensitive_info_summary(&files);

        CodeReviewReport {
            files,
            summary,
            static_analysis_summary,
            ai_review_summary,
            sensitive_info_summary,
        }
    }

    /// 兼容性方法：保持原有接口
    pub async fn review_git_changes(&self, diff_content: &str) -> CodeReviewReport {
        let options = ReviewOptions::default();
        self.review_git_changes_with_options(diff_content, &options)
            .await
    }

    /// 分析指定文件列表（包含敏感信息检测）
    pub async fn analyze_files_with_options(
        &self,
        file_paths: &[String],
        options: &ReviewOptions,
    ) -> CodeReviewReport {
        let mut files = Vec::new();
        let mut ai_reviews = Vec::new();

        for file_path in file_paths {
            if let Ok(content) = tokio::fs::read_to_string(file_path).await {
                let result = self
                    .analyze_file_with_options(file_path, &content, options)
                    .await;

                if let Some(ref ai_review) = result.ai_review {
                    ai_reviews.push(ai_review.clone());
                }

                files.push(result);
            }
        }

        let summary = self.generate_summary(&files);
        let static_analysis_summary = self.generate_static_analysis_summary(&files);
        let ai_review_summary = if !ai_reviews.is_empty() {
            Some(self.generate_ai_review_summary(&ai_reviews))
        } else {
            None
        };
        let sensitive_info_summary = self.generate_sensitive_info_summary(&files);

        CodeReviewReport {
            files,
            summary,
            static_analysis_summary,
            ai_review_summary,
            sensitive_info_summary,
        }
    }

    /// 兼容性方法：保持原有接口
    pub async fn analyze_files(&self, file_paths: &[String]) -> CodeReviewReport {
        let options = ReviewOptions::default();
        self.analyze_files_with_options(file_paths, &options).await
    }

    /// 生成敏感信息摘要
    fn generate_sensitive_info_summary(
        &self,
        files: &[FileAnalysisResult],
    ) -> Option<SensitiveInfoSummary> {
        let mut all_sensitive_items = Vec::new();
        let mut has_sensitive_data = false;

        for file in files {
            if let Some(ref sensitive_info) = file.sensitive_info {
                if !sensitive_info.items.is_empty() {
                    has_sensitive_data = true;
                    all_sensitive_items.extend(sensitive_info.items.clone());
                }
            }
        }

        if !has_sensitive_data {
            return None;
        }

        // 重新计算整体摘要
        let mut types_detected = HashMap::new();
        let mut critical_count = 0;
        let mut high_count = 0;
        let mut medium_count = 0;
        let mut low_count = 0;

        for item in &all_sensitive_items {
            *types_detected.entry(item.info_type.clone()).or_insert(0) += 1;

            match item.info_type.risk_level() {
                crate::languages::sensitive_info::SensitiveRiskLevel::Critical => {
                    critical_count += 1
                }
                crate::languages::sensitive_info::SensitiveRiskLevel::High => high_count += 1,
                crate::languages::sensitive_info::SensitiveRiskLevel::Medium => medium_count += 1,
                crate::languages::sensitive_info::SensitiveRiskLevel::Low => low_count += 1,
            }
        }

        let overall_risk = if critical_count > 0 {
            crate::languages::sensitive_info::SensitiveRiskLevel::Critical
        } else if high_count > 0 {
            crate::languages::sensitive_info::SensitiveRiskLevel::High
        } else if medium_count > 0 {
            crate::languages::sensitive_info::SensitiveRiskLevel::Medium
        } else {
            crate::languages::sensitive_info::SensitiveRiskLevel::Low
        };

        Some(SensitiveInfoSummary {
            total_count: all_sensitive_items.len(),
            critical_count,
            high_count,
            medium_count,
            low_count,
            types_detected,
            overall_risk,
        })
    }

    /// 生成 AI 审查摘要
    fn generate_ai_review_summary(&self, ai_reviews: &[AIReviewResult]) -> AIReviewSummary {
        let total_files = ai_reviews.len();
        let average_score = if total_files > 0 {
            ai_reviews.iter().map(|r| r.overall_score).sum::<f32>() / total_files as f32
        } else {
            0.0
        };

        let mut critical_issues = Vec::new();
        let mut common_patterns = Vec::new();
        let mut recommended_actions = Vec::new();

        for review in ai_reviews {
            if review.security_score < 7.0 {
                critical_issues.push(format!("安全分数较低: {:.1}", review.security_score));
            }
            if review.performance_score < 7.0 {
                critical_issues.push(format!("性能分数较低: {:.1}", review.performance_score));
            }

            common_patterns.push(review.review_type.clone());
            recommended_actions.extend(review.recommendations.clone());
        }

        AIReviewSummary {
            total_files_reviewed: total_files,
            average_score,
            critical_issues,
            common_patterns,
            best_practices_violations: vec!["需要进一步分析".to_string()],
            recommended_actions,
        }
    }

    /// 生成传统摘要（保持兼容性）
    fn generate_summary(&self, files: &[FileAnalysisResult]) -> ReviewSummary {
        let mut languages_detected = HashMap::new();
        let mut total_features = 0;
        let mut all_patterns = Vec::new();
        let all_risks = Vec::new();
        let mut all_test_suggestions = Vec::new();

        for file in files {
            *languages_detected.entry(file.language.clone()).or_insert(0) += 1;
            total_features += file.analysis.features.len();
            all_patterns.extend(file.analysis.change_patterns.clone());

            // 从 AI 审查结果中提取风险和建议
            if let Some(ref ai_review) = file.ai_review {
                all_test_suggestions.extend(ai_review.recommendations.clone());
            }
        }

        ReviewSummary {
            total_files: files.len(),
            languages_detected,
            total_features,
            common_patterns: all_patterns,
            overall_risks: all_risks,
            test_suggestions: all_test_suggestions,
        }
    }

    /// 生成静态分析摘要
    fn generate_static_analysis_summary(
        &self,
        files: &[FileAnalysisResult],
    ) -> StaticAnalysisSummary {
        let start_time = std::time::Instant::now();
        let mut tools_used = std::collections::HashSet::new();
        let mut total_issues = 0;
        let mut issues_by_severity = HashMap::new();
        let mut issues_by_tool = HashMap::new();

        for file in files {
            for analysis in &file.static_analysis {
                tools_used.insert(format!("{:?}", analysis.tool));
                total_issues += analysis.issues.len();

                for issue in &analysis.issues {
                    let severity_str = format!("{:?}", issue.severity).to_lowercase();
                    *issues_by_severity.entry(severity_str).or_insert(0) += 1;
                }

                let tool_str = format!("{:?}", analysis.tool);
                *issues_by_tool.entry(tool_str).or_insert(0) += analysis.issues.len();
            }
        }

        StaticAnalysisSummary {
            tools_used: tools_used.into_iter().collect(),
            total_issues,
            issues_by_severity,
            issues_by_tool,
            execution_time: start_time.elapsed(),
            tools_unavailable: Vec::new(),
        }
    }

    /// 解析 Git diff
    fn parse_git_diff(&self, diff_content: &str) -> Vec<(String, Vec<String>)> {
        let mut results = Vec::new();
        let mut current_file = None;
        let mut added_lines = Vec::new();

        for line in diff_content.lines() {
            if line.starts_with("diff --git") {
                if let Some(file_path) = current_file.take() {
                    results.push((file_path, added_lines.clone()));
                    added_lines.clear();
                }

                if let Some(path) = self.extract_file_path(line) {
                    current_file = Some(path);
                }
            } else if line.starts_with('+') && !line.starts_with("+++") {
                added_lines.push(line[1..].to_string());
            }
        }

        if let Some(file_path) = current_file {
            results.push((file_path, added_lines));
        }

        results
    }

    /// 从 diff 行中提取文件路径
    fn extract_file_path(&self, line: &str) -> Option<String> {
        // 处理两种格式：
        // 1. diff --git a/file.ext b/file.ext
        // 2. diff --git "a/file with spaces.ext" "b/file with spaces.ext"

        if let Some(b_part) = line.split(" b/").nth(1) {
            // 标准格式：a/file b/file
            Some(b_part.to_string())
        } else if let Some(quoted_part) = line.split(" \"b/").nth(1) {
            // 带引号格式："a/file" "b/file"
            quoted_part
                .find('"')
                .map(|end_quote| quoted_part[..end_quote].to_string())
        } else {
            None
        }
    }

    /// 格式化报告（增强版本，包含敏感信息检测结果）
    pub fn format_enhanced_report(&self, report: &CodeReviewReport) -> String {
        let mut output = String::new();

        output.push_str("# 🔍 增强代码审查报告\n\n");

        // 基本统计
        output.push_str("## 📊 审查统计\n\n");
        output.push_str(&format!("- **总文件数**: {}\n", report.summary.total_files));
        output.push_str(&format!(
            "- **代码特征数**: {}\n",
            report.summary.total_features
        ));
        output.push_str(&format!(
            "- **静态分析问题**: {}\n",
            report.static_analysis_summary.total_issues
        ));

        if let Some(ref ai_summary) = report.ai_review_summary {
            output.push_str(&format!(
                "- **AI 审查文件数**: {}\n",
                ai_summary.total_files_reviewed
            ));
            output.push_str(&format!(
                "- **平均质量分数**: {:.1}/10\n",
                ai_summary.average_score
            ));
        }

        // 敏感信息统计
        if let Some(ref sensitive_summary) = report.sensitive_info_summary {
            output.push_str(&format!(
                "- **敏感信息检测**: {} 项 (风险等级: {} {})\n",
                sensitive_summary.total_count,
                sensitive_summary.overall_risk.emoji(),
                sensitive_summary.overall_risk.as_str()
            ));
        }

        output.push_str("\n## 🗣️ 检测到的编程语言\n\n");
        for (language, count) in &report.summary.languages_detected {
            output.push_str(&format!("- **{}**: {} 个文件\n", language.as_str(), count));
        }

        // 敏感信息摘要
        if let Some(ref sensitive_summary) = report.sensitive_info_summary {
            output.push_str("\n## 🚨 敏感信息检测摘要\n\n");

            output.push_str(&format!(
                "**总体风险**: {} {} ({} 项)\n\n",
                sensitive_summary.overall_risk.emoji(),
                sensitive_summary.overall_risk.as_str(),
                sensitive_summary.total_count
            ));

            // 风险等级分布
            output.push_str("### 📊 风险等级分布\n\n");
            if sensitive_summary.critical_count > 0 {
                output.push_str(&format!(
                    "- 🚨 **严重**: {} 项\n",
                    sensitive_summary.critical_count
                ));
            }
            if sensitive_summary.high_count > 0 {
                output.push_str(&format!(
                    "- ⚠️ **高**: {} 项\n",
                    sensitive_summary.high_count
                ));
            }
            if sensitive_summary.medium_count > 0 {
                output.push_str(&format!(
                    "- 🟡 **中等**: {} 项\n",
                    sensitive_summary.medium_count
                ));
            }
            if sensitive_summary.low_count > 0 {
                output.push_str(&format!(
                    "- ℹ️ **低**: {} 项\n",
                    sensitive_summary.low_count
                ));
            }

            // 敏感信息类型分布
            if !sensitive_summary.types_detected.is_empty() {
                output.push_str("\n### 🔍 检测到的敏感信息类型\n\n");
                for (info_type, count) in &sensitive_summary.types_detected {
                    output.push_str(&format!(
                        "- {} **{}**: {} 项 ({})\n",
                        info_type.risk_level().emoji(),
                        info_type.as_str(),
                        count,
                        info_type.risk_level().as_str()
                    ));
                }
            }

            output.push('\n');
        }

        // AI 审查摘要
        if let Some(ref ai_summary) = report.ai_review_summary {
            output.push_str("## 🤖 AI 审查摘要\n\n");

            if !ai_summary.critical_issues.is_empty() {
                output.push_str("### ⚠️ 关键问题\n\n");
                for issue in &ai_summary.critical_issues {
                    output.push_str(&format!("- {}\n", issue));
                }
                output.push('\n');
            }

            if !ai_summary.recommended_actions.is_empty() {
                output.push_str("### 💡 推荐操作\n\n");
                for action in &ai_summary.recommended_actions {
                    output.push_str(&format!("- {}\n", action));
                }
                output.push('\n');
            }
        }

        // 静态分析详情
        if report.static_analysis_summary.total_issues > 0
            || !report.static_analysis_summary.tools_used.is_empty()
        {
            output.push_str("## 🔍 静态分析问题\n\n");

            // 显示使用的工具
            if !report.static_analysis_summary.tools_used.is_empty() {
                output.push_str("### 🛠️ 执行的工具\n\n");
                for tool in &report.static_analysis_summary.tools_used {
                    output.push_str(&format!("- {}\n", tool));
                }
                output.push('\n');
            }

            // 按严重程度显示问题统计
            if !report.static_analysis_summary.issues_by_severity.is_empty() {
                output.push_str("### 按严重程度分类\n\n");
                for (severity, count) in &report.static_analysis_summary.issues_by_severity {
                    let emoji = match severity.as_str() {
                        "error" => "❌",
                        "warning" => "⚠️",
                        "info" => "ℹ️",
                        "style" => "🎨",
                        _ => "•",
                    };
                    output.push_str(&format!("- {} **{}**: {} 个\n", emoji, severity, count));
                }
                output.push('\n');
            }

            // 显示每个文件的静态分析问题
            for file in &report.files {
                if !file.static_analysis.is_empty() {
                    output.push_str(&format!("### 📄 {} 的问题\n\n", file.file_path));

                    for analysis in &file.static_analysis {
                        output.push_str(&format!("**{}**", analysis.tool.name()));
                        if analysis.success {
                            if analysis.issues.is_empty() {
                                output.push_str(" ✅ 无问题\n");
                            } else {
                                output.push_str(&format!(
                                    " 发现 {} 个问题:\n",
                                    analysis.issues.len()
                                ));
                            }
                        } else {
                            output.push_str(" ❌ 执行失败");
                            if let Some(ref error) = analysis.error_message {
                                output.push_str(&format!(" ({})", error));
                            }
                            output.push('\n');
                        }

                        for issue in &analysis.issues {
                            let location = if let (Some(line), Some(col)) =
                                (issue.line_number, issue.column)
                            {
                                format!("第{}行:{}列", line, col)
                            } else if let Some(line) = issue.line_number {
                                format!("第{}行", line)
                            } else {
                                "位置未知".to_string()
                            };

                            let severity_icon = match issue.severity {
                                crate::languages::static_analysis::IssueSeverity::Error => "❌",
                                crate::languages::static_analysis::IssueSeverity::Warning => "⚠️",
                                crate::languages::static_analysis::IssueSeverity::Info => "ℹ️",
                                crate::languages::static_analysis::IssueSeverity::Style => "🎨",
                            };

                            output.push_str(&format!(
                                "- {} **{}**: {} ({})\n",
                                severity_icon,
                                issue.severity.as_str(),
                                issue.message,
                                location
                            ));
                        }
                        output.push('\n');
                    }
                }
            }
        }

        // 详细文件分析（包含敏感信息）
        output.push_str("## 📁 详细文件分析\n\n");
        for file in &report.files {
            output.push_str(&format!("### 📄 {}\n\n", file.file_path));
            output.push_str(&format!("- **语言**: {}\n", file.language.as_str()));
            output.push_str(&format!("- **特征数**: {}\n", file.analysis.features.len()));

            // 敏感信息检测结果
            if let Some(ref sensitive_info) = file.sensitive_info {
                if !sensitive_info.items.is_empty() {
                    output.push_str(&format!(
                        "- **敏感信息**: {} 项 ({})\n",
                        sensitive_info.items.len(),
                        sensitive_info.summary.overall_risk.as_str()
                    ));

                    // 显示敏感信息详情
                    output.push_str("\n#### 🚨 检测到的敏感信息:\n\n");
                    for item in &sensitive_info.items {
                        output.push_str(&format!(
                            "- {} **{}** (第{}行): `{}` → `{}`\n",
                            item.info_type.risk_level().emoji(),
                            item.info_type.as_str(),
                            item.line_number,
                            item.matched_text,
                            item.masked_text
                        ));
                        output.push_str(&format!("  - 置信度: {:.1}%\n", item.confidence * 100.0));
                        output.push_str(&format!("  - 说明: {}\n", item.description));
                        if !item.recommendations.is_empty() {
                            output.push_str("  - 建议:\n");
                            for rec in &item.recommendations {
                                output.push_str(&format!("    - {}\n", rec));
                            }
                        }
                        output.push('\n');
                    }
                } else {
                    output.push_str("- **敏感信息**: ✅ 未检测到敏感信息\n");
                }
            }

            if let Some(ref ai_review) = file.ai_review {
                output.push_str(&format!(
                    "- **AI 评分**: {:.1}/10\n",
                    ai_review.overall_score
                ));
                output.push_str(&format!("- **审查类型**: {}\n", ai_review.review_type));

                if !ai_review.summary.is_empty() {
                    output.push_str(&format!("- **摘要**: {}\n", ai_review.summary));
                }
            }

            output.push('\n');
        }

        output
    }

    /// 兼容性方法：保持原有接口
    pub fn format_report(&self, report: &CodeReviewReport) -> String {
        self.format_enhanced_report(report)
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
    fn test_review_options_default() {
        let options = ReviewOptions::default();
        assert!(options.enable_ai_review);
        assert!(options.include_static_analysis);
        assert!(options.detailed_feedback);
        assert!(options.enable_sensitive_info_detection);
        assert_eq!(options.ai_review_types.len(), 1);
    }

    #[test]
    fn test_code_review_service_creation() {
        let service = CodeReviewService::new();
        assert!(!service.enable_ai_review);

        let service_with_ai = service.with_ai_review(true);
        assert!(service_with_ai.enable_ai_review);
    }

    #[test]
    fn test_language_detection() {
        let service = CodeReviewService::new();
        assert_eq!(service.detect_language("main.rs"), Language::Rust);
        assert_eq!(service.detect_language("main.go"), Language::Go);
        assert_eq!(service.detect_language("index.ts"), Language::TypeScript);
    }

    #[test]
    fn test_file_path_extraction() {
        let service = CodeReviewService::new();
        let line = "diff --git a/src/main.rs b/src/main.rs";
        let path = service.extract_file_path(line);
        assert_eq!(path, Some("src/main.rs".to_string()));
    }

    #[tokio::test]
    async fn test_ai_review_summary_generation() {
        let service = CodeReviewService::new();
        let ai_reviews = vec![AIReviewResult {
            review_type: "rust_comprehensive".to_string(),
            overall_score: 8.5,
            summary: "Good code".to_string(),
            detailed_feedback: "Detailed feedback".to_string(),
            security_score: 9.0,
            performance_score: 8.0,
            maintainability_score: 8.5,
            recommendations: vec!["Add tests".to_string()],
            learning_resources: vec!["https://doc.rust-lang.org".to_string()],
        }];

        let summary = service.generate_ai_review_summary(&ai_reviews);
        assert_eq!(summary.total_files_reviewed, 1);
        assert_eq!(summary.average_score, 8.5);
    }

    #[tokio::test]
    async fn test_sensitive_info_detection_in_file_analysis() {
        let service = CodeReviewService::new();
        let options = ReviewOptions {
            enable_ai_review: false,
            include_static_analysis: false,
            enable_sensitive_info_detection: true,
            ..ReviewOptions::default()
        };

        let file_content = r#"
        const config = {
            api_key: "sk-1234567890abcdef1234567890abcdef",
            password: "secretpassword123",
            email: "admin@company.com"
        };
        "#;

        let result = service
            .analyze_file_with_options("config.js", file_content, &options)
            .await;

        assert!(result.sensitive_info.is_some());
        let sensitive_info = result.sensitive_info.unwrap();
        assert_eq!(sensitive_info.items.len(), 3);

        // 检查检测到的敏感信息类型
        let info_types: Vec<_> = sensitive_info
            .items
            .iter()
            .map(|item| &item.info_type)
            .collect();
        assert!(info_types.contains(&&crate::languages::sensitive_info::SensitiveInfoType::ApiKey));
        assert!(
            info_types.contains(&&crate::languages::sensitive_info::SensitiveInfoType::Password)
        );
        assert!(info_types.contains(&&crate::languages::sensitive_info::SensitiveInfoType::Email));
    }

    #[tokio::test]
    async fn test_sensitive_info_summary_generation() {
        let service = CodeReviewService::new();

        // 创建包含敏感信息的测试数据
        let files = vec![FileAnalysisResult {
            file_path: "test.js".to_string(),
            language: crate::languages::Language::JavaScript,
            analysis: crate::languages::LanguageAnalysisResult {
                language: crate::languages::Language::JavaScript,
                features: vec![],
                scope_suggestions: vec![],
                change_patterns: vec![],
            },
            static_analysis: vec![],
            ai_review: None,
            sensitive_info: Some(crate::languages::sensitive_info::SensitiveInfoResult {
                file_path: "test.js".to_string(),
                items: vec![crate::languages::sensitive_info::SensitiveInfoItem {
                    info_type: crate::languages::sensitive_info::SensitiveInfoType::ApiKey,
                    line_number: 1,
                    column_start: 0,
                    column_end: 10,
                    matched_text: "api_key".to_string(),
                    masked_text: "***".to_string(),
                    confidence: 0.9,
                    description: "API Key detected".to_string(),
                    recommendations: vec!["Use environment variables".to_string()],
                }],
                summary: crate::languages::sensitive_info::SensitiveInfoSummary {
                    total_count: 1,
                    critical_count: 1,
                    high_count: 0,
                    medium_count: 0,
                    low_count: 0,
                    types_detected: std::collections::HashMap::new(),
                    overall_risk: crate::languages::sensitive_info::SensitiveRiskLevel::Critical,
                },
            }),
        }];

        let summary = service.generate_sensitive_info_summary(&files);
        assert!(summary.is_some());

        let summary = summary.unwrap();
        assert_eq!(summary.total_count, 1);
        assert_eq!(summary.critical_count, 1);
        assert!(matches!(
            summary.overall_risk,
            crate::languages::sensitive_info::SensitiveRiskLevel::Critical
        ));
    }

    #[tokio::test]
    async fn test_no_sensitive_info_detected() {
        let service = CodeReviewService::new();
        let options = ReviewOptions {
            enable_ai_review: false,
            include_static_analysis: false,
            enable_sensitive_info_detection: true,
            ..ReviewOptions::default()
        };

        let file_content = r#"
        function calculateSum(a, b) {
            return a + b;
        }
        const result = calculateSum(1, 2);
        "#;

        let result = service
            .analyze_file_with_options("safe.js", file_content, &options)
            .await;

        assert!(result.sensitive_info.is_some());
        let sensitive_info = result.sensitive_info.unwrap();
        assert_eq!(sensitive_info.items.len(), 0);
    }

    #[test]
    fn test_format_report_with_sensitive_info() {
        let service = CodeReviewService::new();

        let report = CodeReviewReport {
            files: vec![],
            summary: ReviewSummary {
                total_files: 1,
                languages_detected: std::collections::HashMap::new(),
                total_features: 0,
                common_patterns: vec![],
                overall_risks: vec![],
                test_suggestions: vec![],
            },
            static_analysis_summary: StaticAnalysisSummary {
                tools_used: vec![],
                total_issues: 0,
                issues_by_severity: std::collections::HashMap::new(),
                issues_by_tool: std::collections::HashMap::new(),
                execution_time: std::time::Duration::from_secs(0),
                tools_unavailable: vec![],
            },
            ai_review_summary: None,
            sensitive_info_summary: Some(crate::languages::sensitive_info::SensitiveInfoSummary {
                total_count: 2,
                critical_count: 1,
                high_count: 1,
                medium_count: 0,
                low_count: 0,
                types_detected: std::collections::HashMap::new(),
                overall_risk: crate::languages::sensitive_info::SensitiveRiskLevel::Critical,
            }),
        };

        let formatted = service.format_enhanced_report(&report);
        assert!(formatted.contains("敏感信息检测"));
        assert!(formatted.contains("🚨"));
        assert!(formatted.contains("严重"));
    }
}
