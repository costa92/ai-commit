use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 报告过滤条件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportFilter {
    pub project_path: Option<String>,
    pub project_path_pattern: Option<String>, // 支持模糊匹配
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub min_score: Option<f32>,
    pub max_score: Option<f32>,
    pub languages: Option<Vec<String>>,
    pub tags: Option<HashMap<String, String>>,
    pub user_id: Option<String>,
    pub min_files_analyzed: Option<usize>,
    pub max_files_analyzed: Option<usize>,
    pub min_issues: Option<usize>,
    pub max_issues: Option<usize>,
    pub issue_severity_filter: Option<IssueSeverityFilter>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub sort_by: Option<SortField>,
    pub sort_order: Option<SortOrder>,
    pub include_aggregations: bool, // 是否包含聚合统计信息
}

/// 问题严重程度过滤器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueSeverityFilter {
    pub min_critical: Option<usize>,
    pub max_critical: Option<usize>,
    pub min_high: Option<usize>,
    pub max_high: Option<usize>,
    pub min_medium: Option<usize>,
    pub max_medium: Option<usize>,
    pub min_low: Option<usize>,
    pub max_low: Option<usize>,
}

/// 排序字段
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SortField {
    CreatedAt,
    OverallScore,
    ProjectPath,
    FilesAnalyzed,
    IssuesCount,
    CriticalIssues,
    HighIssues,
    MediumIssues,
    LowIssues,
    UserId,
}

/// 排序顺序
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SortOrder {
    Asc,
    Desc,
}

/// 报告摘要信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportSummary {
    pub id: String,
    pub project_path: String,
    pub created_at: DateTime<Utc>,
    pub overall_score: f32,
    pub files_analyzed: usize,
    pub languages_detected: Vec<String>,
    pub issues_count: usize,
    pub critical_issues: usize,
    pub high_issues: usize,
    pub medium_issues: usize,
    pub low_issues: usize,
    pub tags: HashMap<String, String>,
    pub user_id: Option<String>,
    pub version: Option<String>,
}

/// 查询结果，包含报告列表和可选的聚合信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportQueryResult {
    pub reports: Vec<ReportSummary>,
    pub total_count: usize,
    pub aggregations: Option<ReportAggregations>,
}

/// 报告聚合统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportAggregations {
    pub score_stats: ScoreStatistics,
    pub language_distribution: HashMap<String, usize>,
    pub project_distribution: HashMap<String, usize>,
    pub user_distribution: HashMap<String, usize>,
    pub issue_severity_stats: IssueSeverityStatistics,
    pub time_series: Option<Vec<TimeSeriesPoint>>,
}

/// 评分统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreStatistics {
    pub min: f32,
    pub max: f32,
    pub avg: f32,
    pub median: Option<f32>,
    pub std_dev: Option<f32>,
    pub percentiles: HashMap<String, f32>, // "p25", "p50", "p75", "p90", "p95"
}

/// 问题严重程度统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueSeverityStatistics {
    pub total_critical: usize,
    pub total_high: usize,
    pub total_medium: usize,
    pub total_low: usize,
    pub avg_critical_per_report: f32,
    pub avg_high_per_report: f32,
    pub avg_medium_per_report: f32,
    pub avg_low_per_report: f32,
}

/// 时间序列数据点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSeriesPoint {
    pub timestamp: DateTime<Utc>,
    pub count: usize,
    pub avg_score: f32,
    pub total_issues: usize,
}

/// 高级查询构建器
#[derive(Debug, Clone)]
pub struct ReportQueryBuilder {
    filter: ReportFilter,
}

impl ReportQueryBuilder {
    pub fn new() -> Self {
        Self {
            filter: ReportFilter::default(),
        }
    }

    pub fn project_path(mut self, path: &str) -> Self {
        self.filter.project_path = Some(path.to_string());
        self
    }

    pub fn project_path_like(mut self, pattern: &str) -> Self {
        self.filter.project_path_pattern = Some(pattern.to_string());
        self
    }

    pub fn date_range(mut self, start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        self.filter.start_date = Some(start);
        self.filter.end_date = Some(end);
        self
    }

    pub fn score_range(mut self, min: f32, max: f32) -> Self {
        self.filter.min_score = Some(min);
        self.filter.max_score = Some(max);
        self
    }

    pub fn languages(mut self, languages: Vec<String>) -> Self {
        self.filter.languages = Some(languages);
        self
    }

    pub fn user_id(mut self, user_id: &str) -> Self {
        self.filter.user_id = Some(user_id.to_string());
        self
    }

    pub fn files_analyzed_range(mut self, min: usize, max: usize) -> Self {
        self.filter.min_files_analyzed = Some(min);
        self.filter.max_files_analyzed = Some(max);
        self
    }

    pub fn issues_range(mut self, min: usize, max: usize) -> Self {
        self.filter.min_issues = Some(min);
        self.filter.max_issues = Some(max);
        self
    }

    pub fn critical_issues_range(mut self, min: usize, max: usize) -> Self {
        let mut severity_filter = self.filter.issue_severity_filter.unwrap_or_default();
        severity_filter.min_critical = Some(min);
        severity_filter.max_critical = Some(max);
        self.filter.issue_severity_filter = Some(severity_filter);
        self
    }

    pub fn high_issues_range(mut self, min: usize, max: usize) -> Self {
        let mut severity_filter = self.filter.issue_severity_filter.unwrap_or_default();
        severity_filter.min_high = Some(min);
        severity_filter.max_high = Some(max);
        self.filter.issue_severity_filter = Some(severity_filter);
        self
    }

    pub fn tag(mut self, key: &str, value: &str) -> Self {
        let mut tags = self.filter.tags.unwrap_or_default();
        tags.insert(key.to_string(), value.to_string());
        self.filter.tags = Some(tags);
        self
    }

    pub fn limit(mut self, limit: usize) -> Self {
        self.filter.limit = Some(limit);
        self
    }

    pub fn offset(mut self, offset: usize) -> Self {
        self.filter.offset = Some(offset);
        self
    }

    pub fn sort_by(mut self, field: SortField, order: SortOrder) -> Self {
        self.filter.sort_by = Some(field);
        self.filter.sort_order = Some(order);
        self
    }

    pub fn with_aggregations(mut self) -> Self {
        self.filter.include_aggregations = true;
        self
    }

    pub fn build(self) -> ReportFilter {
        self.filter
    }
}

impl Default for ReportQueryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for IssueSeverityFilter {
    fn default() -> Self {
        Self {
            min_critical: None,
            max_critical: None,
            min_high: None,
            max_high: None,
            min_medium: None,
            max_medium: None,
            min_low: None,
            max_low: None,
        }
    }
}

impl Default for ReportFilter {
    fn default() -> Self {
        Self {
            project_path: None,
            project_path_pattern: None,
            start_date: None,
            end_date: None,
            min_score: None,
            max_score: None,
            languages: None,
            tags: None,
            user_id: None,
            min_files_analyzed: None,
            max_files_analyzed: None,
            min_issues: None,
            max_issues: None,
            issue_severity_filter: None,
            limit: Some(50),
            offset: Some(0),
            sort_by: Some(SortField::CreatedAt),
            sort_order: Some(SortOrder::Desc),
            include_aggregations: false,
        }
    }
}