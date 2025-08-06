use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// 报告配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportConfig {
    /// 报告格式
    pub format: ReportFormat,
    /// 输出路径
    pub output_path: Option<PathBuf>,
    /// 模板配置
    pub template: TemplateConfig,
    /// 包含的部分
    pub sections: ReportSections,
    /// 自定义选项
    pub options: ReportOptions,
}

/// 报告格式
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ReportFormat {
    Markdown,
    Json,
    Text,
    Html,
}

/// 模板配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateConfig {
    /// 模板名称
    pub name: String,
    /// 自定义模板路径
    pub custom_path: Option<PathBuf>,
    /// 模板变量
    pub variables: HashMap<String, String>,
}

/// 报告部分配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportSections {
    /// 包含摘要
    pub include_summary: bool,
    /// 包含静态分析结果
    pub include_static_analysis: bool,
    /// 包含 AI 审查结果
    pub include_ai_review: bool,
    /// 包含敏感信息检测
    pub include_sensitive_info: bool,
    /// 包含复杂度分析
    pub include_complexity: bool,
    /// 包含重复检测
    pub include_duplication: bool,
    /// 包含依赖分析
    pub include_dependency: bool,
    /// 包含覆盖率分析
    pub include_coverage: bool,
    /// 包含性能分析
    pub include_performance: bool,
    /// 包含趋势分析
    pub include_trend: bool,
    /// 包含统计信息
    pub include_statistics: bool,
    /// 包含建议
    pub include_recommendations: bool,
}

/// 报告选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportOptions {
    /// 显示详细信息
    pub verbose: bool,
    /// 包含代码片段
    pub include_code_snippets: bool,
    /// 最大问题数量
    pub max_issues_per_section: Option<usize>,
    /// 严重程度过滤
    pub severity_filter: Option<Vec<crate::models::review::Severity>>,
    /// 生成图表
    pub generate_charts: bool,
    /// 包含元数据
    pub include_metadata: bool,
    /// 颜色输出（仅文本格式）
    pub colored_output: bool,
}

impl Default for ReportConfig {
    fn default() -> Self {
        Self {
            format: ReportFormat::Markdown,
            output_path: None,
            template: TemplateConfig::default(),
            sections: ReportSections::default(),
            options: ReportOptions::default(),
        }
    }
}

impl Default for TemplateConfig {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            custom_path: None,
            variables: HashMap::new(),
        }
    }
}

impl Default for ReportSections {
    fn default() -> Self {
        Self {
            include_summary: true,
            include_static_analysis: true,
            include_ai_review: true,
            include_sensitive_info: true,
            include_complexity: true,
            include_duplication: true,
            include_dependency: true,
            include_coverage: true,
            include_performance: true,
            include_trend: true,
            include_statistics: true,
            include_recommendations: true,
        }
    }
}

impl Default for ReportOptions {
    fn default() -> Self {
        Self {
            verbose: false,
            include_code_snippets: true,
            max_issues_per_section: Some(50),
            severity_filter: None,
            generate_charts: false,
            include_metadata: true,
            colored_output: true,
        }
    }
}

impl std::fmt::Display for ReportFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReportFormat::Markdown => write!(f, "markdown"),
            ReportFormat::Json => write!(f, "json"),
            ReportFormat::Text => write!(f, "text"),
            ReportFormat::Html => write!(f, "html"),
        }
    }
}

impl std::str::FromStr for ReportFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "markdown" | "md" => Ok(ReportFormat::Markdown),
            "json" => Ok(ReportFormat::Json),
            "text" | "txt" => Ok(ReportFormat::Text),
            "html" => Ok(ReportFormat::Html),
            _ => Err(format!("Unsupported report format: {}", s)),
        }
    }
}