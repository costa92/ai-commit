use std::collections::HashMap;
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use regex::Regex;
use once_cell::sync::Lazy;

use crate::ai::reviewers::language_specific::{
    AIReviewResult, AIReviewIssue, AIReviewSuggestion, LearningResource,
    IssueSeverity, ResourceType
};
use crate::languages::Language;

/// AI 审查结果处理器
pub struct AIReviewResultProcessor;

/// 质量评分算法配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityScoringConfig {
    /// 问题权重配置
    pub issue_weights: HashMap<IssueSeverity, f32>,

    /// 基础分数
    pub base_score: f32,

    /// 最大扣分
    pub max_deduction: f32,

    /// 建议奖励分数
    pub suggestion_bonus: f32,
}

/// 标准化的审查结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandardizedReviewResult {
    /// 原始审查结果
    pub original_result: AIReviewResult,

    /// 标准化质量评分
    pub normalized_score: f32,

    /// 评分详情
    pub scoring_details: ScoringDetails,

    /// 分类后的问题
    pub categorized_issues: CategorizedIssues,

    /// 优先级排序的建议
    pub prioritized_suggestions: Vec<PrioritizedSuggestion>,

    /// 增强的学习资源
    pub enhanced_learning_resources: Vec<EnhancedLearningResource>,

    /// 质量趋势分析
    pub quality_trend: Option<QualityTrend>,
}

/// 评分详情
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoringDetails {
    /// 基础分数
    pub base_score: f32,

    /// 问题扣分
    pub issue_deductions: HashMap<IssueSeverity, f32>,

    /// 总扣分
    pub total_deduction: f32,

    /// 建议奖励分
    pub suggestion_bonus: f32,

    /// 最终分数
    pub final_score: f32,

    /// 评分说明
    pub explanation: String,
}

/// 分类后的问题
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategorizedIssues {
    /// 安全问题
    pub security_issues: Vec<AIReviewIssue>,

    /// 性能问题
    pub performance_issues: Vec<AIReviewIssue>,

    /// 可维护性问题
    pub maintainability_issues: Vec<AIReviewIssue>,

    /// 代码风格问题
    pub style_issues: Vec<AIReviewIssue>,

    /// 逻辑问题
    pub logic_issues: Vec<AIReviewIssue>,

    /// 其他问题
    pub other_issues: Vec<AIReviewIssue>,
}

/// 优先级建议
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrioritizedSuggestion {
    /// 原始建议
    pub suggestion: AIReviewSuggestion,

    /// 优先级分数 (1-10, 10最高)
    pub priority_score: f32,

    /// 实施难度 (1-5, 5最难)
    pub implementation_difficulty: u8,

    /// 预期收益 (1-5, 5最高)
    pub expected_benefit: u8,

    /// 优先级说明
    pub priority_explanation: String,
}

/// 增强的学习资源
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedLearningResource {
    /// 原始资源
    pub resource: LearningResource,

    /// 相关性分数 (1-10)
    pub relevance_score: f32,

    /// 难度等级 (1-5)
    pub difficulty_level: u8,

    /// 预计学习时间（分钟）
    pub estimated_time_minutes: u32,

    /// 标签
    pub tags: Vec<String>,
}

/// 质量趋势
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityTrend {
    /// 当前分数
    pub current_score: f32,

    /// 历史分数（如果有）
    pub historical_scores: Vec<f32>,

    /// 趋势方向
    pub trend_direction: TrendDirection,

    /// 趋势强度 (0-1)
    pub trend_strength: f32,

    /// 趋势分析
    pub trend_analysis: String,
}

/// 趋势方向
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrendDirection {
    Improving,
    Stable,
    Declining,
    Unknown,
}

// 正则表达式用于解析和分类
static SECURITY_KEYWORDS: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)(security|安全|vulnerability|漏洞|injection|注入|xss|csrf|sql|auth|认证|权限)")
        .expect("Failed to compile security keywords regex")
});

static PERFORMANCE_KEYWORDS: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)(performance|性能|slow|慢|optimization|优化|memory|内存|cpu|cache|缓存)")
        .expect("Failed to compile performance keywords regex")
});

static MAINTAINABILITY_KEYWORDS: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)(maintainability|可维护性|readable|可读性|complex|复杂|refactor|重构|duplicate|重复)")
        .expect("Failed to compile maintainability keywords regex")
});

static STYLE_KEYWORDS: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)(style|风格|format|格式|naming|命名|convention|约定|lint)")
        .expect("Failed to compile style keywords regex")
});

impl AIReviewResultProcessor {
    /// 处理和标准化审查结果
    pub fn process_review_result(
        &self,
        result: AIReviewResult,
        config: Option<QualityScoringConfig>,
    ) -> Result<StandardizedReviewResult> {
        let config = config.unwrap_or_else(|| self.default_scoring_config());

        // 标准化质量评分
        let (normalized_score, scoring_details) = self.normalize_quality_score(&result, &config)?;

        // 分类问题
        let categorized_issues = self.categorize_issues(&result.issues);

        // 优先级排序建议
        let prioritized_suggestions = self.prioritize_suggestions(&result.suggestions);

        // 增强学习资源
        let enhanced_learning_resources = self.enhance_learning_resources(
            &result.learning_resources,
            &result.language,
            &result.issues,
        );

        // 质量趋势分析（如果有历史数据）
        let quality_trend = self.analyze_quality_trend(normalized_score, None);

        Ok(StandardizedReviewResult {
            original_result: result,
            normalized_score,
            scoring_details,
            categorized_issues,
            prioritized_suggestions,
            enhanced_learning_resources,
            quality_trend,
        })
    }

    /// 标准化质量评分
    fn normalize_quality_score(
        &self,
        result: &AIReviewResult,
        config: &QualityScoringConfig,
    ) -> Result<(f32, ScoringDetails)> {
        let mut base_score = config.base_score;
        let mut issue_deductions = HashMap::new();
        let mut total_deduction: f32 = 0.0;

        // 计算问题扣分
        for issue in &result.issues {
            let weight = config.issue_weights.get(&issue.severity).unwrap_or(&1.0);
            let deduction = weight * 0.5; // 每个问题基础扣分0.5，根据严重程度调整

            *issue_deductions.entry(issue.severity.clone()).or_insert(0.0) += deduction;
            total_deduction += deduction;
        }

        // 限制最大扣分
        total_deduction = total_deduction.min(config.max_deduction);

        // 计算建议奖励分
        let suggestion_bonus = (result.suggestions.len() as f32 * config.suggestion_bonus).min(1.0);

        // 计算最终分数
        let final_score = (base_score - total_deduction + suggestion_bonus).clamp(1.0, 10.0);

        let explanation = self.generate_scoring_explanation(
            base_score,
            total_deduction,
            suggestion_bonus,
            final_score,
            &result.issues,
        );

        let scoring_details = ScoringDetails {
            base_score,
            issue_deductions,
            total_deduction,
            suggestion_bonus,
            final_score,
            explanation,
        };

        Ok((final_score, scoring_details))
    }

    /// 分类问题
    fn categorize_issues(&self, issues: &[AIReviewIssue]) -> CategorizedIssues {
        let mut categorized = CategorizedIssues {
            security_issues: Vec::new(),
            performance_issues: Vec::new(),
            maintainability_issues: Vec::new(),
            style_issues: Vec::new(),
            logic_issues: Vec::new(),
            other_issues: Vec::new(),
        };

        for issue in issues {
            let issue_text = format!("{} {}", issue.issue_type, issue.description);

            if SECURITY_KEYWORDS.is_match(&issue_text) {
                categorized.security_issues.push(issue.clone());
            } else if PERFORMANCE_KEYWORDS.is_match(&issue_text) {
                categorized.performance_issues.push(issue.clone());
            } else if MAINTAINABILITY_KEYWORDS.is_match(&issue_text) {
                categorized.maintainability_issues.push(issue.clone());
            } else if STYLE_KEYWORDS.is_match(&issue_text) {
                categorized.style_issues.push(issue.clone());
            } else if issue_text.to_lowercase().contains("logic") || issue_text.contains("逻辑") {
                categorized.logic_issues.push(issue.clone());
            } else {
                categorized.other_issues.push(issue.clone());
            }
        }

        categorized
    }

    /// 优先级排序建议
    fn prioritize_suggestions(&self, suggestions: &[AIReviewSuggestion]) -> Vec<PrioritizedSuggestion> {
        let mut prioritized: Vec<PrioritizedSuggestion> = suggestions
            .iter()
            .map(|suggestion| {
                let (priority_score, difficulty, benefit, explanation) =
                    self.calculate_suggestion_priority(suggestion);

                PrioritizedSuggestion {
                    suggestion: suggestion.clone(),
                    priority_score,
                    implementation_difficulty: difficulty,
                    expected_benefit: benefit,
                    priority_explanation: explanation,
                }
            })
            .collect();

        // 按优先级分数排序
        prioritized.sort_by(|a, b| b.priority_score.partial_cmp(&a.priority_score).unwrap());

        prioritized
    }

    /// 计算建议优先级
    fn calculate_suggestion_priority(&self, suggestion: &AIReviewSuggestion) -> (f32, u8, u8, String) {
        let suggestion_text = format!("{} {} {}",
            suggestion.suggestion_type,
            suggestion.description,
            suggestion.reason
        );

        let mut priority_score: f32 = 5.0; // 基础分数
        let mut difficulty = 3; // 默认难度
        let mut benefit = 3; // 默认收益

        // 根据关键词调整优先级
        if SECURITY_KEYWORDS.is_match(&suggestion_text) {
            priority_score += 3.0;
            benefit = 5;
        }

        if PERFORMANCE_KEYWORDS.is_match(&suggestion_text) {
            priority_score += 2.0;
            benefit = 4;
        }

        if suggestion_text.to_lowercase().contains("easy") || suggestion_text.contains("简单") {
            difficulty = 1;
            priority_score += 1.0;
        } else if suggestion_text.to_lowercase().contains("complex") || suggestion_text.contains("复杂") {
            difficulty = 5;
            priority_score -= 1.0;
        }

        priority_score = priority_score.clamp(1.0, 10.0);

        let explanation = format!(
            "基于安全性、性能影响和实施难度评估，优先级为 {:.1}/10",
            priority_score
        );

        (priority_score, difficulty, benefit, explanation)
    }

    /// 增强学习资源
    fn enhance_learning_resources(
        &self,
        resources: &[LearningResource],
        language: &Language,
        issues: &[AIReviewIssue],
    ) -> Vec<EnhancedLearningResource> {
        let mut enhanced: Vec<EnhancedLearningResource> = resources
            .iter()
            .map(|resource| {
                let (relevance_score, difficulty_level, estimated_time, tags) =
                    self.analyze_learning_resource(resource, language, issues);

                EnhancedLearningResource {
                    resource: resource.clone(),
                    relevance_score,
                    difficulty_level,
                    estimated_time_minutes: estimated_time,
                    tags,
                }
            })
            .collect();

        // 按相关性排序
        enhanced.sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap());

        enhanced
    }

    /// 分析学习资源
    fn analyze_learning_resource(
        &self,
        resource: &LearningResource,
        language: &Language,
        issues: &[AIReviewIssue],
    ) -> (f32, u8, u32, Vec<String>) {
        let mut relevance_score: f32 = 5.0;
        let mut difficulty_level = 3;
        let mut estimated_time = 30; // 默认30分钟
        let mut tags = Vec::new();

        // 根据资源类型调整
        match resource.resource_type {
            ResourceType::Documentation => {
                relevance_score += 1.0;
                difficulty_level = 2;
                estimated_time = 15;
                tags.push("官方文档".to_string());
            }
            ResourceType::Tutorial => {
                relevance_score += 2.0;
                difficulty_level = 2;
                estimated_time = 45;
                tags.push("教程".to_string());
            }
            ResourceType::Article => {
                relevance_score += 1.5;
                difficulty_level = 3;
                estimated_time = 20;
                tags.push("文章".to_string());
            }
            ResourceType::Video => {
                relevance_score += 1.0;
                difficulty_level = 2;
                estimated_time = 60;
                tags.push("视频".to_string());
            }
            ResourceType::Book => {
                relevance_score += 0.5;
                difficulty_level = 4;
                estimated_time = 300;
                tags.push("书籍".to_string());
            }
            ResourceType::Tool => {
                relevance_score += 1.5;
                difficulty_level = 3;
                estimated_time = 30;
                tags.push("工具".to_string());
            }
        }

        // 根据语言调整相关性
        let language_str = format!("{:?}", language).to_lowercase();
        if resource.title.to_lowercase().contains(&language_str) ||
           resource.description.to_lowercase().contains(&language_str) {
            relevance_score += 2.0;
            tags.push(format!("{:?}", language));
        }

        // 根据问题类型调整相关性
        for issue in issues {
            if resource.description.to_lowercase().contains(&issue.issue_type.to_lowercase()) {
                relevance_score += 1.0;
                tags.push(issue.issue_type.clone());
            }
        }

        relevance_score = relevance_score.clamp(1.0, 10.0);

        (relevance_score, difficulty_level, estimated_time, tags)
    }

    /// 分析质量趋势
    fn analyze_quality_trend(
        &self,
        current_score: f32,
        historical_scores: Option<Vec<f32>>,
    ) -> Option<QualityTrend> {
        let historical_scores = historical_scores.unwrap_or_default();

        if historical_scores.is_empty() {
            return Some(QualityTrend {
                current_score,
                historical_scores,
                trend_direction: TrendDirection::Unknown,
                trend_strength: 0.0,
                trend_analysis: "暂无历史数据，无法分析趋势".to_string(),
            });
        }

        let (trend_direction, trend_strength) = self.calculate_trend(&historical_scores, current_score);
        let trend_analysis = self.generate_trend_analysis(&trend_direction, trend_strength, current_score);

        Some(QualityTrend {
            current_score,
            historical_scores,
            trend_direction,
            trend_strength,
            trend_analysis,
        })
    }

    /// 计算趋势
    fn calculate_trend(&self, historical_scores: &[f32], current_score: f32) -> (TrendDirection, f32) {
        if historical_scores.len() < 2 {
            return (TrendDirection::Unknown, 0.0);
        }

        let recent_scores: Vec<f32> = historical_scores.iter()
            .rev()
            .take(5)
            .cloned()
            .collect();

        let avg_recent = recent_scores.iter().sum::<f32>() / recent_scores.len() as f32;
        let score_diff = current_score - avg_recent;

        let trend_direction = if score_diff > 0.5 {
            TrendDirection::Improving
        } else if score_diff < -0.5 {
            TrendDirection::Declining
        } else {
            TrendDirection::Stable
        };

        let trend_strength = (score_diff.abs() / 10.0).clamp(0.0, 1.0);

        (trend_direction, trend_strength)
    }

    /// 生成趋势分析
    fn generate_trend_analysis(&self, direction: &TrendDirection, strength: f32, current_score: f32) -> String {
        match direction {
            TrendDirection::Improving => {
                format!("代码质量呈上升趋势，当前分数 {:.1}，趋势强度 {:.1}/1.0", current_score, strength)
            }
            TrendDirection::Declining => {
                format!("代码质量呈下降趋势，当前分数 {:.1}，需要关注，趋势强度 {:.1}/1.0", current_score, strength)
            }
            TrendDirection::Stable => {
                format!("代码质量保持稳定，当前分数 {:.1}，变化不大", current_score)
            }
            TrendDirection::Unknown => {
                format!("暂无足够历史数据分析趋势，当前分数 {:.1}", current_score)
            }
        }
    }

    /// 生成评分说明
    fn generate_scoring_explanation(
        &self,
        base_score: f32,
        total_deduction: f32,
        suggestion_bonus: f32,
        final_score: f32,
        issues: &[AIReviewIssue],
    ) -> String {
        let critical_count = issues.iter().filter(|i| matches!(i.severity, IssueSeverity::Critical)).count();
        let high_count = issues.iter().filter(|i| matches!(i.severity, IssueSeverity::High)).count();
        let medium_count = issues.iter().filter(|i| matches!(i.severity, IssueSeverity::Medium)).count();

        format!(
            "基础分数 {:.1}，发现 {} 个严重问题、{} 个高级问题、{} 个中等问题，总扣分 {:.1}，建议奖励 {:.1}，最终得分 {:.1}",
            base_score, critical_count, high_count, medium_count, total_deduction, suggestion_bonus, final_score
        )
    }

    /// 默认评分配置
    fn default_scoring_config(&self) -> QualityScoringConfig {
        let mut issue_weights = HashMap::new();
        issue_weights.insert(IssueSeverity::Critical, 3.0);
        issue_weights.insert(IssueSeverity::High, 2.0);
        issue_weights.insert(IssueSeverity::Medium, 1.0);
        issue_weights.insert(IssueSeverity::Low, 0.5);
        issue_weights.insert(IssueSeverity::Info, 0.1);

        QualityScoringConfig {
            issue_weights,
            base_score: 8.0,
            max_deduction: 6.0,
            suggestion_bonus: 0.1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::reviewers::language_specific::{AIReviewResult, AIReviewIssue, AIReviewSuggestion};

    #[test]
    fn test_normalize_quality_score() {
        let processor = AIReviewResultProcessor;
        let config = processor.default_scoring_config();

        let result = create_test_review_result();
        let (normalized_score, scoring_details) = processor.normalize_quality_score(&result, &config).unwrap();

        assert!(normalized_score >= 1.0 && normalized_score <= 10.0);
        assert!(scoring_details.final_score == normalized_score);
        assert!(!scoring_details.explanation.is_empty());
    }

    #[test]
    fn test_categorize_issues() {
        let processor = AIReviewResultProcessor;

        let issues = vec![
            AIReviewIssue {
                issue_type: "Security Vulnerability".to_string(),
                description: "SQL injection risk".to_string(),
                line_number: Some(10),
                severity: IssueSeverity::Critical,
                fix_suggestion: "Use parameterized queries".to_string(),
                code_example: None,
            },
            AIReviewIssue {
                issue_type: "Performance Issue".to_string(),
                description: "Slow database query".to_string(),
                line_number: Some(20),
                severity: IssueSeverity::High,
                fix_suggestion: "Add database index".to_string(),
                code_example: None,
            },
        ];

        let categorized = processor.categorize_issues(&issues);

        assert_eq!(categorized.security_issues.len(), 1);
        assert_eq!(categorized.performance_issues.len(), 1);
        assert_eq!(categorized.security_issues[0].issue_type, "Security Vulnerability");
        assert_eq!(categorized.performance_issues[0].issue_type, "Performance Issue");
    }

    #[test]
    fn test_prioritize_suggestions() {
        let processor = AIReviewResultProcessor;

        let suggestions = vec![
            AIReviewSuggestion {
                suggestion_type: "Security Enhancement".to_string(),
                description: "Add input validation".to_string(),
                reason: "Prevent security vulnerabilities".to_string(),
                implementation: "Use validation library".to_string(),
                expected_impact: "High security improvement".to_string(),
                code_example: None,
            },
            AIReviewSuggestion {
                suggestion_type: "Style Improvement".to_string(),
                description: "Fix formatting".to_string(),
                reason: "Improve readability".to_string(),
                implementation: "Run formatter".to_string(),
                expected_impact: "Better code style".to_string(),
                code_example: None,
            },
        ];

        let prioritized = processor.prioritize_suggestions(&suggestions);

        assert_eq!(prioritized.len(), 2);
        // Security suggestion should have higher priority
        assert!(prioritized[0].priority_score > prioritized[1].priority_score);
        assert_eq!(prioritized[0].suggestion.suggestion_type, "Security Enhancement");
    }

    #[test]
    fn test_calculate_trend() {
        let processor = AIReviewResultProcessor;

        // Test improving trend
        let historical_scores = vec![6.0, 6.5, 7.0, 7.5];
        let current_score = 8.0;
        let (direction, strength) = processor.calculate_trend(&historical_scores, current_score);

        assert!(matches!(direction, TrendDirection::Improving));
        assert!(strength > 0.0);

        // Test declining trend
        let historical_scores = vec![8.0, 7.5, 7.0, 6.5];
        let current_score = 6.0;
        let (direction, strength) = processor.calculate_trend(&historical_scores, current_score);

        assert!(matches!(direction, TrendDirection::Declining));
        assert!(strength > 0.0);

        // Test stable trend
        let historical_scores = vec![7.0, 7.1, 6.9, 7.0];
        let current_score = 7.0;
        let (direction, _) = processor.calculate_trend(&historical_scores, current_score);

        assert!(matches!(direction, TrendDirection::Stable));
    }

    #[test]
    fn test_process_review_result() {
        let processor = AIReviewResultProcessor;
        let result = create_test_review_result();

        let processed = processor.process_review_result(result, None).unwrap();

        assert!(processed.normalized_score >= 1.0 && processed.normalized_score <= 10.0);
        assert!(!processed.scoring_details.explanation.is_empty());
        assert!(!processed.prioritized_suggestions.is_empty());
        assert!(!processed.enhanced_learning_resources.is_empty());
    }

    fn create_test_review_result() -> AIReviewResult {
        AIReviewResult {
            file_path: "test.rs".to_string(),
            language: Language::Rust,
            quality_score: 7.5,
            issues: vec![
                AIReviewIssue {
                    issue_type: "Performance".to_string(),
                    description: "Inefficient loop".to_string(),
                    line_number: Some(10),
                    severity: IssueSeverity::Medium,
                    fix_suggestion: "Use iterator".to_string(),
                    code_example: None,
                }
            ],
            suggestions: vec![
                AIReviewSuggestion {
                    suggestion_type: "Optimization".to_string(),
                    description: "Use Vec::with_capacity".to_string(),
                    reason: "Reduce allocations".to_string(),
                    implementation: "Pre-allocate vector".to_string(),
                    expected_impact: "Better performance".to_string(),
                    code_example: None,
                }
            ],
            best_practices: vec!["Use idiomatic Rust".to_string()],
            learning_resources: vec![
                LearningResource {
                    title: "Rust Book".to_string(),
                    url: "https://doc.rust-lang.org/book/".to_string(),
                    description: "Official Rust tutorial".to_string(),
                    resource_type: ResourceType::Book,
                }
            ],
            summary: "Good code quality".to_string(),
            language_specific_analysis: None,
        }
    }
}