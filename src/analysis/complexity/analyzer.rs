use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

use crate::languages::{Language, LanguageFeature};
use super::calculator::{CyclomaticComplexityCalculator, CognitiveComplexityCalculator, FunctionLengthAnalyzer, NestingDepthAnalyzer};
use super::metrics::{ComplexityMetrics, RiskLevel};

/// Configuration for complexity analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityConfig {
    /// Maximum cyclomatic complexity before warning
    pub cyclomatic_threshold_warning: u32,
    /// Maximum cyclomatic complexity before error
    pub cyclomatic_threshold_error: u32,
    /// Maximum cognitive complexity before warning
    pub cognitive_threshold_warning: u32,
    /// Maximum cognitive complexity before error
    pub cognitive_threshold_error: u32,
    /// Maximum function length before warning
    pub function_length_threshold_warning: u32,
    /// Maximum function length before error
    pub function_length_threshold_error: u32,
    /// Maximum nesting depth before warning
    pub nesting_depth_threshold_warning: u32,
    /// Maximum nesting depth before error
    pub nesting_depth_threshold_error: u32,
    /// Whether to include comments in function length calculation
    pub include_comments_in_length: bool,
    /// Whether to include blank lines in function length calculation
    pub include_blank_lines_in_length: bool,
}

impl Default for ComplexityConfig {
    fn default() -> Self {
        Self {
            cyclomatic_threshold_warning: 10,
            cyclomatic_threshold_error: 20,
            cognitive_threshold_warning: 15,
            cognitive_threshold_error: 25,
            function_length_threshold_warning: 50,
            function_length_threshold_error: 100,
            nesting_depth_threshold_warning: 4,
            nesting_depth_threshold_error: 6,
            include_comments_in_length: false,
            include_blank_lines_in_length: false,
        }
    }
}

/// Represents a function's complexity metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionComplexity {
    /// Function name
    pub name: String,
    /// Starting line number
    pub line_start: u32,
    /// Ending line number
    pub line_end: u32,
    /// Cyclomatic complexity score
    pub cyclomatic_complexity: u32,
    /// Cognitive complexity score
    pub cognitive_complexity: u32,
    /// Function length in lines
    pub function_length: u32,
    /// Maximum nesting depth
    pub max_nesting_depth: u32,
    /// Overall risk level
    pub risk_level: RiskLevel,
    /// Specific issues found
    pub issues: Vec<ComplexityIssue>,
}

/// Represents a complexity issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityIssue {
    /// Type of complexity issue
    pub issue_type: ComplexityIssueType,
    /// Severity level
    pub severity: IssueSeverity,
    /// Description of the issue
    pub message: String,
    /// Line number where the issue occurs
    pub line_number: Option<u32>,
    /// Suggested fix
    pub suggestion: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComplexityIssueType {
    HighCyclomaticComplexity,
    HighCognitiveComplexity,
    LongFunction,
    DeepNesting,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IssueSeverity {
    Warning,
    Error,
    Critical,
}

/// Result of complexity analysis for a file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityResult {
    /// File path
    pub file_path: String,
    /// Language detected
    pub language: Language,
    /// Analysis timestamp
    pub timestamp: DateTime<Utc>,
    /// Individual function complexities
    pub functions: Vec<FunctionComplexity>,
    /// Overall file metrics
    pub overall_metrics: ComplexityMetrics,
    /// Complexity hotspots
    pub hotspots: Vec<ComplexityHotspot>,
    /// Refactoring recommendations
    pub recommendations: Vec<RefactoringRecommendation>,
}

/// Represents a complexity hotspot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityHotspot {
    /// Function name
    pub function_name: String,
    /// Line range
    pub line_range: (u32, u32),
    /// Hotspot score (higher = more problematic)
    pub score: f64,
    /// Primary complexity issue
    pub primary_issue: ComplexityIssueType,
    /// Description
    pub description: String,
}

/// Represents a refactoring recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefactoringRecommendation {
    /// Function name to refactor
    pub function_name: String,
    /// Line range
    pub line_range: (u32, u32),
    /// Priority (1-10, higher = more urgent)
    pub priority: u32,
    /// Refactoring technique
    pub technique: RefactoringTechnique,
    /// Description of the recommendation
    pub description: String,
    /// Expected benefit
    pub expected_benefit: String,
    /// Estimated effort
    pub estimated_effort: EffortLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RefactoringTechnique {
    ExtractMethod,
    ReduceNesting,
    SimplifyConditionals,
    BreakDownFunction,
    ExtractClass,
    ReplaceConditionalWithPolymorphism,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EffortLevel {
    Low,
    Medium,
    High,
}

/// Main complexity analyzer
pub struct ComplexityAnalyzer {
    cyclomatic_calculator: CyclomaticComplexityCalculator,
    cognitive_calculator: CognitiveComplexityCalculator,
    function_analyzer: FunctionLengthAnalyzer,
    nesting_analyzer: NestingDepthAnalyzer,
    config: ComplexityConfig,
}

impl ComplexityAnalyzer {
    /// Create a new complexity analyzer with default configuration
    pub fn new() -> Self {
        Self::with_config(ComplexityConfig::default())
    }

    /// Create a new complexity analyzer with custom configuration
    pub fn with_config(config: ComplexityConfig) -> Self {
        Self {
            cyclomatic_calculator: CyclomaticComplexityCalculator::new(),
            cognitive_calculator: CognitiveComplexityCalculator::new(),
            function_analyzer: FunctionLengthAnalyzer::new(),
            nesting_analyzer: NestingDepthAnalyzer::new(),
            config,
        }
    }

    /// Analyze complexity for a file
    pub fn analyze_file(&self, file_path: &str, content: &str, language: &Language) -> anyhow::Result<ComplexityResult> {
        let functions = self.extract_functions(content, language)?;
        let mut function_complexities = Vec::new();

        for function in functions {
            let cyclomatic = self.cyclomatic_calculator.calculate(&function, language)?;
            let cognitive = self.cognitive_calculator.calculate(&function, language)?;
            let length = self.function_analyzer.analyze(&function, &self.config)?;
            let nesting = self.nesting_analyzer.analyze(&function, language)?;

            let issues = self.identify_issues(cyclomatic, cognitive, length, nesting);
            let risk_level = self.calculate_risk_level(cyclomatic, cognitive, length, nesting);

            function_complexities.push(FunctionComplexity {
                name: function.name.clone(),
                line_start: function.line_start,
                line_end: function.line_end,
                cyclomatic_complexity: cyclomatic,
                cognitive_complexity: cognitive,
                function_length: length,
                max_nesting_depth: nesting,
                risk_level,
                issues,
            });
        }

        let overall_metrics = self.calculate_overall_metrics(&function_complexities);
        let hotspots = self.identify_hotspots(&function_complexities);
        let recommendations = self.generate_recommendations(&function_complexities);

        Ok(ComplexityResult {
            file_path: file_path.to_string(),
            language: language.clone(),
            timestamp: Utc::now(),
            functions: function_complexities,
            overall_metrics,
            hotspots,
            recommendations,
        })
    }

    /// Extract function information from code
    fn extract_functions(&self, content: &str, language: &Language) -> anyhow::Result<Vec<FunctionInfo>> {
        // This is a simplified implementation
        // In a real implementation, we would use proper AST parsing
        let mut functions = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        match language {
            Language::Rust => self.extract_rust_functions(&lines, &mut functions),
            Language::Go => self.extract_go_functions(&lines, &mut functions),
            Language::TypeScript | Language::JavaScript => self.extract_ts_functions(&lines, &mut functions),
            _ => self.extract_generic_functions(&lines, &mut functions),
        }

        Ok(functions)
    }

    fn extract_rust_functions(&self, lines: &[&str], functions: &mut Vec<FunctionInfo>) {
        let mut current_function: Option<FunctionInfo> = None;
        let mut brace_count = 0;

        for (i, line) in lines.iter().enumerate() {
            let line_num = (i + 1) as u32;
            let trimmed = line.trim();

            // Look for function definitions
            if trimmed.starts_with("fn ") || trimmed.contains(" fn ") {
                if let Some(name) = self.extract_rust_function_name(trimmed) {
                    if let Some(mut func) = current_function.take() {
                        func.line_end = line_num - 1;
                        func.content = lines[func.line_start as usize - 1..func.line_end as usize].join("\n");
                        functions.push(func);
                    }

                    current_function = Some(FunctionInfo {
                        name,
                        line_start: line_num,
                        line_end: line_num,
                        content: String::new(),
                    });
                    brace_count = 0;
                }
            }

            if current_function.is_some() {
                brace_count += line.matches('{').count() as i32;
                brace_count -= line.matches('}').count() as i32;

                if brace_count == 0 && line.contains('}') {
                    if let Some(mut func) = current_function.take() {
                        func.line_end = line_num;
                        func.content = lines[func.line_start as usize - 1..=func.line_end as usize - 1].join("\n");
                        functions.push(func);
                    }
                }
            }
        }

        // Handle case where file ends without closing brace
        if let Some(mut func) = current_function {
            func.line_end = lines.len() as u32;
            func.content = lines[func.line_start as usize - 1..].join("\n");
            functions.push(func);
        }
    }

    fn extract_rust_function_name(&self, line: &str) -> Option<String> {
        if let Some(fn_pos) = line.find("fn ") {
            let after_fn = &line[fn_pos + 3..];
            if let Some(paren_pos) = after_fn.find('(') {
                let name = after_fn[..paren_pos].trim();
                if !name.is_empty() {
                    return Some(name.to_string());
                }
            }
        }
        None
    }

    fn extract_go_functions(&self, lines: &[&str], functions: &mut Vec<FunctionInfo>) {
        // Similar implementation for Go functions
        // This is a simplified version
        for (i, line) in lines.iter().enumerate() {
            let line_num = (i + 1) as u32;
            let trimmed = line.trim();

            if trimmed.starts_with("func ") {
                if let Some(name) = self.extract_go_function_name(trimmed) {
                    // For simplicity, assume single-line functions or use brace counting
                    functions.push(FunctionInfo {
                        name,
                        line_start: line_num,
                        line_end: line_num + 10, // Simplified
                        content: line.to_string(),
                    });
                }
            }
        }
    }

    fn extract_go_function_name(&self, line: &str) -> Option<String> {
        if let Some(func_pos) = line.find("func ") {
            let after_func = &line[func_pos + 5..];
            if let Some(paren_pos) = after_func.find('(') {
                let name_part = after_func[..paren_pos].trim();
                // Handle receiver methods like "func (r *Receiver) MethodName"
                if name_part.starts_with('(') {
                    if let Some(end_paren) = name_part.find(')') {
                        let method_name = name_part[end_paren + 1..].trim();
                        return Some(method_name.to_string());
                    }
                } else {
                    return Some(name_part.to_string());
                }
            }
        }
        None
    }

    fn extract_ts_functions(&self, lines: &[&str], functions: &mut Vec<FunctionInfo>) {
        // Similar implementation for TypeScript/JavaScript functions
        for (i, line) in lines.iter().enumerate() {
            let line_num = (i + 1) as u32;
            let trimmed = line.trim();

            if trimmed.starts_with("function ") || trimmed.contains("function ") {
                if let Some(name) = self.extract_ts_function_name(trimmed) {
                    functions.push(FunctionInfo {
                        name,
                        line_start: line_num,
                        line_end: line_num + 10, // Simplified
                        content: line.to_string(),
                    });
                }
            }
        }
    }

    fn extract_ts_function_name(&self, line: &str) -> Option<String> {
        if let Some(func_pos) = line.find("function ") {
            let after_func = &line[func_pos + 9..];
            if let Some(paren_pos) = after_func.find('(') {
                let name = after_func[..paren_pos].trim();
                if !name.is_empty() {
                    return Some(name.to_string());
                }
            }
        }
        None
    }

    fn extract_generic_functions(&self, lines: &[&str], functions: &mut Vec<FunctionInfo>) {
        // Generic function extraction for unknown languages
        for (i, line) in lines.iter().enumerate() {
            let line_num = (i + 1) as u32;
            let trimmed = line.trim();

            // Look for common function patterns
            if trimmed.contains("def ") || trimmed.contains("function ") || trimmed.contains("fn ") {
                functions.push(FunctionInfo {
                    name: format!("function_at_line_{}", line_num),
                    line_start: line_num,
                    line_end: line_num + 5, // Simplified
                    content: line.to_string(),
                });
            }
        }
    }

    fn identify_issues(&self, cyclomatic: u32, cognitive: u32, length: u32, nesting: u32) -> Vec<ComplexityIssue> {
        let mut issues = Vec::new();

        // Check cyclomatic complexity
        if cyclomatic >= self.config.cyclomatic_threshold_error {
            issues.push(ComplexityIssue {
                issue_type: ComplexityIssueType::HighCyclomaticComplexity,
                severity: IssueSeverity::Error,
                message: format!("Cyclomatic complexity ({}) exceeds error threshold ({})", cyclomatic, self.config.cyclomatic_threshold_error),
                line_number: None,
                suggestion: Some("Consider breaking this function into smaller functions".to_string()),
            });
        } else if cyclomatic >= self.config.cyclomatic_threshold_warning {
            issues.push(ComplexityIssue {
                issue_type: ComplexityIssueType::HighCyclomaticComplexity,
                severity: IssueSeverity::Warning,
                message: format!("Cyclomatic complexity ({}) exceeds warning threshold ({})", cyclomatic, self.config.cyclomatic_threshold_warning),
                line_number: None,
                suggestion: Some("Consider simplifying this function".to_string()),
            });
        }

        // Check cognitive complexity
        if cognitive >= self.config.cognitive_threshold_error {
            issues.push(ComplexityIssue {
                issue_type: ComplexityIssueType::HighCognitiveComplexity,
                severity: IssueSeverity::Error,
                message: format!("Cognitive complexity ({}) exceeds error threshold ({})", cognitive, self.config.cognitive_threshold_error),
                line_number: None,
                suggestion: Some("Reduce nesting and simplify control flow".to_string()),
            });
        } else if cognitive >= self.config.cognitive_threshold_warning {
            issues.push(ComplexityIssue {
                issue_type: ComplexityIssueType::HighCognitiveComplexity,
                severity: IssueSeverity::Warning,
                message: format!("Cognitive complexity ({}) exceeds warning threshold ({})", cognitive, self.config.cognitive_threshold_warning),
                line_number: None,
                suggestion: Some("Consider reducing cognitive load".to_string()),
            });
        }

        // Check function length
        if length >= self.config.function_length_threshold_error {
            issues.push(ComplexityIssue {
                issue_type: ComplexityIssueType::LongFunction,
                severity: IssueSeverity::Error,
                message: format!("Function length ({} lines) exceeds error threshold ({})", length, self.config.function_length_threshold_error),
                line_number: None,
                suggestion: Some("Break this function into smaller, more focused functions".to_string()),
            });
        } else if length >= self.config.function_length_threshold_warning {
            issues.push(ComplexityIssue {
                issue_type: ComplexityIssueType::LongFunction,
                severity: IssueSeverity::Warning,
                message: format!("Function length ({} lines) exceeds warning threshold ({})", length, self.config.function_length_threshold_warning),
                line_number: None,
                suggestion: Some("Consider shortening this function".to_string()),
            });
        }

        // Check nesting depth
        if nesting >= self.config.nesting_depth_threshold_error {
            issues.push(ComplexityIssue {
                issue_type: ComplexityIssueType::DeepNesting,
                severity: IssueSeverity::Error,
                message: format!("Nesting depth ({}) exceeds error threshold ({})", nesting, self.config.nesting_depth_threshold_error),
                line_number: None,
                suggestion: Some("Reduce nesting by using early returns or extracting methods".to_string()),
            });
        } else if nesting >= self.config.nesting_depth_threshold_warning {
            issues.push(ComplexityIssue {
                issue_type: ComplexityIssueType::DeepNesting,
                severity: IssueSeverity::Warning,
                message: format!("Nesting depth ({}) exceeds warning threshold ({})", nesting, self.config.nesting_depth_threshold_warning),
                line_number: None,
                suggestion: Some("Consider reducing nesting depth".to_string()),
            });
        }

        issues
    }

    fn calculate_risk_level(&self, cyclomatic: u32, cognitive: u32, length: u32, nesting: u32) -> RiskLevel {
        let mut risk_score = 0;

        // Weight different complexity metrics
        if cyclomatic >= self.config.cyclomatic_threshold_error { risk_score += 3; }
        else if cyclomatic >= self.config.cyclomatic_threshold_warning { risk_score += 1; }

        if cognitive >= self.config.cognitive_threshold_error { risk_score += 3; }
        else if cognitive >= self.config.cognitive_threshold_warning { risk_score += 1; }

        if length >= self.config.function_length_threshold_error { risk_score += 2; }
        else if length >= self.config.function_length_threshold_warning { risk_score += 1; }

        if nesting >= self.config.nesting_depth_threshold_error { risk_score += 2; }
        else if nesting >= self.config.nesting_depth_threshold_warning { risk_score += 1; }

        match risk_score {
            0 => RiskLevel::Low,
            1..=3 => RiskLevel::Medium,
            4..=6 => RiskLevel::High,
            _ => RiskLevel::Critical,
        }
    }

    fn calculate_overall_metrics(&self, functions: &[FunctionComplexity]) -> ComplexityMetrics {
        if functions.is_empty() {
            return ComplexityMetrics::default();
        }

        let total_functions = functions.len() as u32;
        let avg_cyclomatic = functions.iter().map(|f| f.cyclomatic_complexity).sum::<u32>() as f64 / total_functions as f64;
        let avg_cognitive = functions.iter().map(|f| f.cognitive_complexity).sum::<u32>() as f64 / total_functions as f64;
        let avg_length = functions.iter().map(|f| f.function_length).sum::<u32>() as f64 / total_functions as f64;
        let avg_nesting = functions.iter().map(|f| f.max_nesting_depth).sum::<u32>() as f64 / total_functions as f64;

        let max_cyclomatic = functions.iter().map(|f| f.cyclomatic_complexity).max().unwrap_or(0);
        let max_cognitive = functions.iter().map(|f| f.cognitive_complexity).max().unwrap_or(0);
        let max_length = functions.iter().map(|f| f.function_length).max().unwrap_or(0);
        let max_nesting = functions.iter().map(|f| f.max_nesting_depth).max().unwrap_or(0);

        let high_risk_functions = functions.iter().filter(|f| matches!(f.risk_level, RiskLevel::High | RiskLevel::Critical)).count() as u32;
        let medium_risk_functions = functions.iter().filter(|f| matches!(f.risk_level, RiskLevel::Medium)).count() as u32;

        ComplexityMetrics {
            total_functions,
            average_cyclomatic_complexity: avg_cyclomatic,
            average_cognitive_complexity: avg_cognitive,
            average_function_length: avg_length,
            average_nesting_depth: avg_nesting,
            max_cyclomatic_complexity: max_cyclomatic,
            max_cognitive_complexity: max_cognitive,
            max_function_length: max_length,
            max_nesting_depth: max_nesting,
            high_risk_functions,
            medium_risk_functions,
            complexity_score: self.calculate_complexity_score(avg_cyclomatic, avg_cognitive, avg_length, avg_nesting),
        }
    }

    fn calculate_complexity_score(&self, avg_cyclomatic: f64, avg_cognitive: f64, avg_length: f64, avg_nesting: f64) -> f64 {
        // Weighted complexity score (0-100, lower is better)
        let cyclomatic_weight = 0.3;
        let cognitive_weight = 0.4;
        let length_weight = 0.2;
        let nesting_weight = 0.1;

        let cyclomatic_score = (avg_cyclomatic / self.config.cyclomatic_threshold_warning as f64).min(1.0) * 100.0;
        let cognitive_score = (avg_cognitive / self.config.cognitive_threshold_warning as f64).min(1.0) * 100.0;
        let length_score = (avg_length / self.config.function_length_threshold_warning as f64).min(1.0) * 100.0;
        let nesting_score = (avg_nesting / self.config.nesting_depth_threshold_warning as f64).min(1.0) * 100.0;

        cyclomatic_score * cyclomatic_weight +
        cognitive_score * cognitive_weight +
        length_score * length_weight +
        nesting_score * nesting_weight
    }

    fn identify_hotspots(&self, functions: &[FunctionComplexity]) -> Vec<ComplexityHotspot> {
        let mut hotspots = Vec::new();

        for function in functions {
            if matches!(function.risk_level, RiskLevel::High | RiskLevel::Critical) {
                let score = self.calculate_hotspot_score(function);
                let primary_issue = self.identify_primary_issue(function);
                let description = self.generate_hotspot_description(function, &primary_issue);

                hotspots.push(ComplexityHotspot {
                    function_name: function.name.clone(),
                    line_range: (function.line_start, function.line_end),
                    score,
                    primary_issue,
                    description,
                });
            }
        }

        // Sort by score (highest first)
        hotspots.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

        hotspots
    }

    fn calculate_hotspot_score(&self, function: &FunctionComplexity) -> f64 {
        let cyclomatic_ratio = function.cyclomatic_complexity as f64 / self.config.cyclomatic_threshold_warning as f64;
        let cognitive_ratio = function.cognitive_complexity as f64 / self.config.cognitive_threshold_warning as f64;
        let length_ratio = function.function_length as f64 / self.config.function_length_threshold_warning as f64;
        let nesting_ratio = function.max_nesting_depth as f64 / self.config.nesting_depth_threshold_warning as f64;

        (cyclomatic_ratio * 0.3 + cognitive_ratio * 0.4 + length_ratio * 0.2 + nesting_ratio * 0.1) * 100.0
    }

    fn identify_primary_issue(&self, function: &FunctionComplexity) -> ComplexityIssueType {
        let cyclomatic_ratio = function.cyclomatic_complexity as f64 / self.config.cyclomatic_threshold_warning as f64;
        let cognitive_ratio = function.cognitive_complexity as f64 / self.config.cognitive_threshold_warning as f64;
        let length_ratio = function.function_length as f64 / self.config.function_length_threshold_warning as f64;
        let nesting_ratio = function.max_nesting_depth as f64 / self.config.nesting_depth_threshold_warning as f64;

        if cognitive_ratio >= cyclomatic_ratio && cognitive_ratio >= length_ratio && cognitive_ratio >= nesting_ratio {
            ComplexityIssueType::HighCognitiveComplexity
        } else if cyclomatic_ratio >= length_ratio && cyclomatic_ratio >= nesting_ratio {
            ComplexityIssueType::HighCyclomaticComplexity
        } else if length_ratio >= nesting_ratio {
            ComplexityIssueType::LongFunction
        } else {
            ComplexityIssueType::DeepNesting
        }
    }

    fn generate_hotspot_description(&self, function: &FunctionComplexity, primary_issue: &ComplexityIssueType) -> String {
        match primary_issue {
            ComplexityIssueType::HighCyclomaticComplexity => {
                format!("Function '{}' has high cyclomatic complexity ({}), indicating many decision points",
                    function.name, function.cyclomatic_complexity)
            },
            ComplexityIssueType::HighCognitiveComplexity => {
                format!("Function '{}' has high cognitive complexity ({}), making it difficult to understand",
                    function.name, function.cognitive_complexity)
            },
            ComplexityIssueType::LongFunction => {
                format!("Function '{}' is too long ({} lines), violating single responsibility principle",
                    function.name, function.function_length)
            },
            ComplexityIssueType::DeepNesting => {
                format!("Function '{}' has deep nesting ({}), reducing readability",
                    function.name, function.max_nesting_depth)
            },
        }
    }

    fn generate_recommendations(&self, functions: &[FunctionComplexity]) -> Vec<RefactoringRecommendation> {
        let mut recommendations = Vec::new();

        for function in functions {
            if matches!(function.risk_level, RiskLevel::Medium | RiskLevel::High | RiskLevel::Critical) {
                let priority = match function.risk_level {
                    RiskLevel::Critical => 10,
                    RiskLevel::High => 7,
                    RiskLevel::Medium => 4,
                    RiskLevel::Low => 1,
                };

                let (technique, description, benefit, effort) = self.recommend_refactoring_technique(function);

                recommendations.push(RefactoringRecommendation {
                    function_name: function.name.clone(),
                    line_range: (function.line_start, function.line_end),
                    priority,
                    technique,
                    description,
                    expected_benefit: benefit,
                    estimated_effort: effort,
                });
            }
        }

        // Sort by priority (highest first)
        recommendations.sort_by(|a, b| b.priority.cmp(&a.priority));

        recommendations
    }

    fn recommend_refactoring_technique(&self, function: &FunctionComplexity) -> (RefactoringTechnique, String, String, EffortLevel) {
        let primary_issue = self.identify_primary_issue(function);

        match primary_issue {
            ComplexityIssueType::HighCyclomaticComplexity => (
                RefactoringTechnique::ExtractMethod,
                format!("Extract smaller methods from '{}' to reduce decision points", function.name),
                "Reduced cyclomatic complexity, improved testability".to_string(),
                EffortLevel::Medium,
            ),
            ComplexityIssueType::HighCognitiveComplexity => (
                RefactoringTechnique::SimplifyConditionals,
                format!("Simplify conditional logic in '{}' to reduce cognitive load", function.name),
                "Improved readability and maintainability".to_string(),
                EffortLevel::Medium,
            ),
            ComplexityIssueType::LongFunction => (
                RefactoringTechnique::BreakDownFunction,
                format!("Break down '{}' into smaller, focused functions", function.name),
                "Better separation of concerns, improved readability".to_string(),
                EffortLevel::High,
            ),
            ComplexityIssueType::DeepNesting => (
                RefactoringTechnique::ReduceNesting,
                format!("Reduce nesting in '{}' using early returns or guard clauses", function.name),
                "Improved readability, reduced cognitive complexity".to_string(),
                EffortLevel::Low,
            ),
        }
    }
}

impl Default for ComplexityAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// Represents function information extracted from code
#[derive(Debug, Clone)]
pub struct FunctionInfo {
    pub name: String,
    pub line_start: u32,
    pub line_end: u32,
    pub content: String,
}