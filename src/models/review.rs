use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 代码审查报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeReviewReport {
    pub summary: ReviewSummary,
    pub static_analysis_results: Vec<StaticAnalysisResult>,
    pub ai_review_results: Vec<AIReviewResult>,
    pub sensitive_info_results: Vec<SensitiveInfoResult>,
    pub complexity_results: Vec<ComplexityResult>,
    pub duplication_results: Vec<DuplicationResult>,
    pub dependency_results: Option<DependencyAnalysisResult>,
    pub coverage_results: Option<CoverageAnalysisResult>,
    pub performance_results: Vec<PerformanceAnalysisResult>,
    pub trend_results: Option<TrendAnalysisResult>,
    pub overall_score: f32,
    pub recommendations: Vec<String>,
    pub metadata: ReviewMetadata,
}

/// 审查摘要
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewSummary {
    pub project_path: String,
    pub files_analyzed: usize,
    pub languages_detected: Vec<String>,
    pub total_issues: usize,
    pub critical_issues: usize,
    pub high_issues: usize,
    pub medium_issues: usize,
    pub low_issues: usize,
    pub analysis_duration: std::time::Duration,
    pub created_at: DateTime<Utc>,
}

/// 审查元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewMetadata {
    pub version: String,
    pub user_id: Option<String>,
    pub correlation_id: Option<String>,
    pub tags: HashMap<String, String>,
    pub configuration: ReviewConfiguration,
}

/// 审查配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewConfiguration {
    pub static_analysis: bool,
    pub ai_review: bool,
    pub sensitive_scan: bool,
    pub complexity_analysis: bool,
    pub duplication_scan: bool,
    pub dependency_scan: bool,
    pub coverage_analysis: bool,
    pub performance_analysis: bool,
    pub trend_analysis: bool,
}

/// 静态分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaticAnalysisResult {
    pub tool_name: String,
    pub file_path: String,
    pub issues: Vec<Issue>,
    pub execution_time: std::time::Duration,
}

/// AI 审查结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIReviewResult {
    pub provider: String,
    pub model: String,
    pub file_path: String,
    pub quality_score: f32,
    pub suggestions: Vec<String>,
    pub learning_resources: Vec<String>,
    pub execution_time: std::time::Duration,
}

/// 敏感信息结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensitiveInfoResult {
    pub file_path: String,
    pub items: Vec<SensitiveItem>,
    pub summary: SensitiveSummary,
}

/// 敏感信息项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensitiveItem {
    pub info_type: String,
    pub line_number: usize,
    pub column_start: usize,
    pub column_end: usize,
    pub matched_text: String,
    pub masked_text: String,
    pub confidence: f32,
    pub risk_level: RiskLevel,
    pub recommendations: Vec<String>,
}

/// 敏感信息摘要
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensitiveSummary {
    pub total_items: usize,
    pub critical_items: usize,
    pub high_items: usize,
    pub medium_items: usize,
    pub low_items: usize,
    pub types_detected: Vec<String>,
}

/// 复杂度分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityResult {
    pub file_path: String,
    pub functions: Vec<FunctionComplexity>,
    pub overall_metrics: ComplexityMetrics,
    pub hotspots: Vec<ComplexityHotspot>,
    pub recommendations: Vec<String>,
}

/// 函数复杂度
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionComplexity {
    pub name: String,
    pub line_start: usize,
    pub line_end: usize,
    pub cyclomatic_complexity: usize,
    pub cognitive_complexity: usize,
    pub function_length: usize,
    pub max_nesting_depth: usize,
    pub risk_level: RiskLevel,
}

/// 复杂度指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityMetrics {
    pub average_cyclomatic: f32,
    pub average_cognitive: f32,
    pub average_function_length: f32,
    pub max_complexity: usize,
    pub functions_over_threshold: usize,
}

/// 复杂度热点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityHotspot {
    pub function_name: String,
    pub file_path: String,
    pub line_number: usize,
    pub complexity_score: f32,
    pub issues: Vec<String>,
}

/// 重复检测结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicationResult {
    pub file_path: String,
    pub duplications: Vec<CodeDuplication>,
    pub duplication_percentage: f32,
    pub recommendations: Vec<String>,
}

/// 代码重复
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeDuplication {
    pub duplication_type: DuplicationType,
    pub source_location: CodeLocation,
    pub target_locations: Vec<CodeLocation>,
    pub similarity_score: f32,
    pub lines_count: usize,
}

/// 重复类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DuplicationType {
    Exact,
    Structural,
    Semantic,
}

/// 代码位置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeLocation {
    pub file_path: String,
    pub line_start: usize,
    pub line_end: usize,
    pub content_hash: String,
}

/// 依赖分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyAnalysisResult {
    pub package_manager: String,
    pub dependencies: Vec<Dependency>,
    pub vulnerabilities: Vec<Vulnerability>,
    pub license_issues: Vec<LicenseIssue>,
    pub outdated_dependencies: Vec<OutdatedDependency>,
    pub summary: DependencySummary,
}

/// 依赖项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    pub name: String,
    pub version: String,
    pub dependency_type: DependencyType,
    pub license: Option<String>,
    pub vulnerabilities: Vec<String>,
}

/// 依赖类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DependencyType {
    Direct,
    Transitive,
    Development,
}

/// 漏洞信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vulnerability {
    pub cve_id: String,
    pub severity: Severity,
    pub description: String,
    pub affected_versions: String,
    pub fixed_version: Option<String>,
    pub recommendations: Vec<String>,
}

/// 许可证问题
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseIssue {
    pub dependency_name: String,
    pub license: String,
    pub issue_type: LicenseIssueType,
    pub description: String,
    pub recommendations: Vec<String>,
}

/// 许可证问题类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LicenseIssueType {
    Incompatible,
    Unknown,
    Restrictive,
}

/// 过时依赖
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutdatedDependency {
    pub name: String,
    pub current_version: String,
    pub latest_version: String,
    pub update_type: UpdateType,
    pub breaking_changes: bool,
    pub recommendations: Vec<String>,
}

/// 更新类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UpdateType {
    Major,
    Minor,
    Patch,
}

/// 依赖摘要
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencySummary {
    pub total_dependencies: usize,
    pub direct_dependencies: usize,
    pub transitive_dependencies: usize,
    pub vulnerabilities_count: usize,
    pub critical_vulnerabilities: usize,
    pub high_vulnerabilities: usize,
    pub license_issues_count: usize,
    pub outdated_count: usize,
}

/// 覆盖率分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageAnalysisResult {
    pub tool_name: String,
    pub overall_coverage: CoverageMetrics,
    pub file_coverage: Vec<FileCoverage>,
    pub uncovered_lines: Vec<UncoveredLine>,
    pub recommendations: Vec<String>,
}

/// 覆盖率指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageMetrics {
    pub line_coverage: f32,
    pub branch_coverage: f32,
    pub function_coverage: f32,
    pub statement_coverage: f32,
}

/// 文件覆盖率
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileCoverage {
    pub file_path: String,
    pub line_coverage: f32,
    pub branch_coverage: f32,
    pub uncovered_lines: Vec<usize>,
}

/// 未覆盖行
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UncoveredLine {
    pub file_path: String,
    pub line_number: usize,
    pub content: String,
    pub reason: String,
}

/// 性能分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceAnalysisResult {
    pub file_path: String,
    pub antipatterns: Vec<PerformanceAntipattern>,
    pub complexity_analysis: AlgorithmComplexity,
    pub memory_issues: Vec<MemoryIssue>,
    pub recommendations: Vec<String>,
}

/// 性能反模式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceAntipattern {
    pub pattern_type: String,
    pub line_number: usize,
    pub description: String,
    pub severity: Severity,
    pub impact: String,
    pub fix_suggestion: String,
}

/// 算法复杂度
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlgorithmComplexity {
    pub time_complexity: String,
    pub space_complexity: String,
    pub nested_loops_count: usize,
    pub recursion_depth: usize,
}

/// 内存问题
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryIssue {
    pub issue_type: MemoryIssueType,
    pub line_number: usize,
    pub description: String,
    pub severity: Severity,
    pub fix_suggestion: String,
}

/// 内存问题类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MemoryIssueType {
    PotentialLeak,
    UnreleasedResource,
    CircularReference,
    ExcessiveAllocation,
}

/// 趋势分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendAnalysisResult {
    pub quality_trend: QualityTrend,
    pub metrics_history: Vec<QualitySnapshot>,
    pub regressions: Vec<QualityRegression>,
    pub technical_debt: TechnicalDebtMetrics,
    pub predictions: Vec<QualityPrediction>,
}

/// 质量趋势
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityTrend {
    pub direction: TrendDirection,
    pub change_rate: f32,
    pub confidence: f32,
    pub time_period: std::time::Duration,
}

/// 趋势方向
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrendDirection {
    Improving,
    Declining,
    Stable,
}

/// 质量快照
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualitySnapshot {
    pub timestamp: DateTime<Utc>,
    pub overall_score: f32,
    pub metrics: HashMap<String, f32>,
    pub commit_hash: Option<String>,
}

/// 质量回归
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityRegression {
    pub metric_name: String,
    pub previous_value: f32,
    pub current_value: f32,
    pub change_percentage: f32,
    pub severity: Severity,
    pub detected_at: DateTime<Utc>,
}

/// 技术债务指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechnicalDebtMetrics {
    pub total_debt_hours: f32,
    pub debt_ratio: f32,
    pub debt_categories: HashMap<String, f32>,
    pub payback_recommendations: Vec<String>,
}

/// 质量预测
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityPrediction {
    pub metric_name: String,
    pub predicted_value: f32,
    pub confidence: f32,
    pub time_horizon: std::time::Duration,
    pub factors: Vec<String>,
}

/// 问题
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    pub tool: String,
    pub file_path: String,
    pub line_number: Option<usize>,
    pub column_number: Option<usize>,
    pub severity: Severity,
    pub category: IssueCategory,
    pub message: String,
    pub suggestion: Option<String>,
    pub rule_id: Option<String>,
}

/// 严重程度
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

/// 风险等级
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum RiskLevel {
    Critical,
    High,
    Medium,
    Low,
}

/// 问题类别
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IssueCategory {
    Style,
    Bug,
    Security,
    Performance,
    Maintainability,
    Complexity,
    Duplication,
    Coverage,
    Dependency,
}

impl Default for ReviewConfiguration {
    fn default() -> Self {
        Self {
            static_analysis: true,
            ai_review: false,
            sensitive_scan: true,
            complexity_analysis: true,
            duplication_scan: false,
            dependency_scan: false,
            coverage_analysis: false,
            performance_analysis: false,
            trend_analysis: false,
        }
    }
}