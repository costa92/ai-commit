use serde::{Deserialize, Serialize};
use std::time::Duration;

/// 问题严重程度
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Critical => write!(f, "Critical"),
            Severity::High => write!(f, "High"),
            Severity::Medium => write!(f, "Medium"),
            Severity::Low => write!(f, "Low"),
            Severity::Info => write!(f, "Info"),
        }
    }
}

/// 问题类别
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
    Custom(String),
}

impl std::fmt::Display for IssueCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IssueCategory::Style => write!(f, "Style"),
            IssueCategory::Bug => write!(f, "Bug"),
            IssueCategory::Security => write!(f, "Security"),
            IssueCategory::Performance => write!(f, "Performance"),
            IssueCategory::Maintainability => write!(f, "Maintainability"),
            IssueCategory::Complexity => write!(f, "Complexity"),
            IssueCategory::Duplication => write!(f, "Duplication"),
            IssueCategory::Coverage => write!(f, "Coverage"),
            IssueCategory::Dependency => write!(f, "Dependency"),
            IssueCategory::Custom(name) => write!(f, "{}", name),
        }
    }
}

/// 代码问题
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
    pub code_snippet: Option<String>,
}

impl Issue {
    pub fn new(
        tool: String,
        file_path: String,
        severity: Severity,
        category: IssueCategory,
        message: String,
    ) -> Self {
        Self {
            tool,
            file_path,
            line_number: None,
            column_number: None,
            severity,
            category,
            message,
            suggestion: None,
            rule_id: None,
            code_snippet: None,
        }
    }

    pub fn with_location(mut self, line: usize, column: Option<usize>) -> Self {
        self.line_number = Some(line);
        self.column_number = column;
        self
    }

    pub fn with_suggestion(mut self, suggestion: String) -> Self {
        self.suggestion = Some(suggestion);
        self
    }

    pub fn with_rule_id(mut self, rule_id: String) -> Self {
        self.rule_id = Some(rule_id);
        self
    }

    pub fn with_code_snippet(mut self, snippet: String) -> Self {
        self.code_snippet = Some(snippet);
        self
    }
}

/// 静态分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaticAnalysisResult {
    pub tool_name: String,
    pub file_path: String,
    pub issues: Vec<Issue>,
    pub execution_time: Duration,
    pub success: bool,
    pub error_message: Option<String>,
}

impl StaticAnalysisResult {
    pub fn new(tool_name: String, file_path: String) -> Self {
        Self {
            tool_name,
            file_path,
            issues: Vec::new(),
            execution_time: Duration::from_secs(0),
            success: true,
            error_message: None,
        }
    }

    pub fn with_issues(mut self, issues: Vec<Issue>) -> Self {
        self.issues = issues;
        self
    }

    pub fn with_execution_time(mut self, duration: Duration) -> Self {
        self.execution_time = duration;
        self
    }

    pub fn with_error(mut self, error: String) -> Self {
        self.success = false;
        self.error_message = Some(error);
        self
    }

    pub fn critical_issues(&self) -> Vec<&Issue> {
        self.issues.iter().filter(|issue| issue.severity == Severity::Critical).collect()
    }

    pub fn high_issues(&self) -> Vec<&Issue> {
        self.issues.iter().filter(|issue| issue.severity == Severity::High).collect()
    }

    pub fn medium_issues(&self) -> Vec<&Issue> {
        self.issues.iter().filter(|issue| issue.severity == Severity::Medium).collect()
    }

    pub fn low_issues(&self) -> Vec<&Issue> {
        self.issues.iter().filter(|issue| issue.severity == Severity::Low).collect()
    }

    pub fn issues_by_category(&self, category: &IssueCategory) -> Vec<&Issue> {
        self.issues.iter().filter(|issue| {
            match (&issue.category, category) {
                (IssueCategory::Custom(a), IssueCategory::Custom(b)) => a == b,
                (a, b) => std::mem::discriminant(a) == std::mem::discriminant(b),
            }
        }).collect()
    }
}