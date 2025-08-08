use serde::{Deserialize, Serialize};

/// Overall complexity metrics for a file or project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityMetrics {
    /// Total number of functions analyzed
    pub total_functions: u32,
    /// Average cyclomatic complexity across all functions
    pub average_cyclomatic_complexity: f64,
    /// Average cognitive complexity across all functions
    pub average_cognitive_complexity: f64,
    /// Average function length in lines
    pub average_function_length: f64,
    /// Average nesting depth
    pub average_nesting_depth: f64,
    /// Maximum cyclomatic complexity found
    pub max_cyclomatic_complexity: u32,
    /// Maximum cognitive complexity found
    pub max_cognitive_complexity: u32,
    /// Maximum function length found
    pub max_function_length: u32,
    /// Maximum nesting depth found
    pub max_nesting_depth: u32,
    /// Number of high-risk functions
    pub high_risk_functions: u32,
    /// Number of medium-risk functions
    pub medium_risk_functions: u32,
    /// Overall complexity score (0-100, lower is better)
    pub complexity_score: f64,
}

impl Default for ComplexityMetrics {
    fn default() -> Self {
        Self {
            total_functions: 0,
            average_cyclomatic_complexity: 0.0,
            average_cognitive_complexity: 0.0,
            average_function_length: 0.0,
            average_nesting_depth: 0.0,
            max_cyclomatic_complexity: 0,
            max_cognitive_complexity: 0,
            max_function_length: 0,
            max_nesting_depth: 0,
            high_risk_functions: 0,
            medium_risk_functions: 0,
            complexity_score: 0.0,
        }
    }
}

/// Risk level classification for functions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RiskLevel {
    /// Low complexity, well-structured code
    Low,
    /// Medium complexity, some areas for improvement
    Medium,
    /// High complexity, needs refactoring
    High,
    /// Critical complexity, urgent refactoring needed
    Critical,
}

impl RiskLevel {
    /// Get a human-readable description of the risk level
    pub fn description(&self) -> &'static str {
        match self {
            RiskLevel::Low => "Low risk - well-structured code",
            RiskLevel::Medium => "Medium risk - some complexity issues",
            RiskLevel::High => "High risk - needs refactoring",
            RiskLevel::Critical => "Critical risk - urgent refactoring required",
        }
    }

    /// Get a color code for UI display
    pub fn color_code(&self) -> &'static str {
        match self {
            RiskLevel::Low => "#28a745",      // Green
            RiskLevel::Medium => "#ffc107",   // Yellow
            RiskLevel::High => "#fd7e14",     // Orange
            RiskLevel::Critical => "#dc3545", // Red
        }
    }

    /// Get a numeric score for the risk level (1-4)
    pub fn score(&self) -> u32 {
        match self {
            RiskLevel::Low => 1,
            RiskLevel::Medium => 2,
            RiskLevel::High => 3,
            RiskLevel::Critical => 4,
        }
    }
}

impl std::fmt::Display for RiskLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RiskLevel::Low => write!(f, "Low"),
            RiskLevel::Medium => write!(f, "Medium"),
            RiskLevel::High => write!(f, "High"),
            RiskLevel::Critical => write!(f, "Critical"),
        }
    }
}

/// Complexity distribution statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityDistribution {
    /// Functions with low complexity
    pub low_complexity_functions: u32,
    /// Functions with medium complexity
    pub medium_complexity_functions: u32,
    /// Functions with high complexity
    pub high_complexity_functions: u32,
    /// Functions with critical complexity
    pub critical_complexity_functions: u32,
    /// Percentage of functions that are well-structured
    pub healthy_code_percentage: f64,
    /// Percentage of functions that need attention
    pub problematic_code_percentage: f64,
}

impl ComplexityDistribution {
    /// Create a new distribution from complexity metrics
    pub fn from_metrics(metrics: &ComplexityMetrics, low_count: u32, critical_count: u32) -> Self {
        let total = metrics.total_functions;
        let problematic = metrics.high_risk_functions + metrics.medium_risk_functions;
        let healthy = total - problematic;

        Self {
            low_complexity_functions: low_count,
            medium_complexity_functions: metrics.medium_risk_functions,
            high_complexity_functions: metrics.high_risk_functions,
            critical_complexity_functions: critical_count,
            healthy_code_percentage: if total > 0 { (healthy as f64 / total as f64) * 100.0 } else { 0.0 },
            problematic_code_percentage: if total > 0 { (problematic as f64 / total as f64) * 100.0 } else { 0.0 },
        }
    }
}

/// Complexity trend data for tracking changes over time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityTrend {
    /// Timestamp of the measurement
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Complexity metrics at this point in time
    pub metrics: ComplexityMetrics,
    /// Change from previous measurement
    pub change: Option<ComplexityChange>,
}

/// Represents changes in complexity metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityChange {
    /// Change in average cyclomatic complexity
    pub cyclomatic_change: f64,
    /// Change in average cognitive complexity
    pub cognitive_change: f64,
    /// Change in average function length
    pub length_change: f64,
    /// Change in overall complexity score
    pub score_change: f64,
    /// Whether the change is an improvement
    pub is_improvement: bool,
}

impl ComplexityChange {
    /// Create a new complexity change by comparing two metrics
    pub fn from_comparison(current: &ComplexityMetrics, previous: &ComplexityMetrics) -> Self {
        let cyclomatic_change = current.average_cyclomatic_complexity - previous.average_cyclomatic_complexity;
        let cognitive_change = current.average_cognitive_complexity - previous.average_cognitive_complexity;
        let length_change = current.average_function_length - previous.average_function_length;
        let score_change = current.complexity_score - previous.complexity_score;

        // Lower scores are better, so negative change is improvement
        let is_improvement = score_change < 0.0;

        Self {
            cyclomatic_change,
            cognitive_change,
            length_change,
            score_change,
            is_improvement,
        }
    }
}

/// Complexity benchmark data for comparison
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityBenchmark {
    /// Language being benchmarked
    pub language: String,
    /// Project type (e.g., "web", "cli", "library")
    pub project_type: String,
    /// Recommended maximum cyclomatic complexity
    pub recommended_max_cyclomatic: u32,
    /// Recommended maximum cognitive complexity
    pub recommended_max_cognitive: u32,
    /// Recommended maximum function length
    pub recommended_max_length: u32,
    /// Recommended maximum nesting depth
    pub recommended_max_nesting: u32,
    /// Industry average complexity score
    pub industry_average_score: f64,
    /// Good complexity score threshold
    pub good_score_threshold: f64,
    /// Excellent complexity score threshold
    pub excellent_score_threshold: f64,
}

impl ComplexityBenchmark {
    /// Get default benchmark for a language
    pub fn for_language(language: &str) -> Self {
        match language.to_lowercase().as_str() {
            "rust" => Self {
                language: "Rust".to_string(),
                project_type: "general".to_string(),
                recommended_max_cyclomatic: 10,
                recommended_max_cognitive: 15,
                recommended_max_length: 50,
                recommended_max_nesting: 4,
                industry_average_score: 35.0,
                good_score_threshold: 25.0,
                excellent_score_threshold: 15.0,
            },
            "go" => Self {
                language: "Go".to_string(),
                project_type: "general".to_string(),
                recommended_max_cyclomatic: 10,
                recommended_max_cognitive: 15,
                recommended_max_length: 50,
                recommended_max_nesting: 4,
                industry_average_score: 30.0,
                good_score_threshold: 20.0,
                excellent_score_threshold: 12.0,
            },
            "typescript" | "javascript" => Self {
                language: "TypeScript/JavaScript".to_string(),
                project_type: "general".to_string(),
                recommended_max_cyclomatic: 10,
                recommended_max_cognitive: 15,
                recommended_max_length: 50,
                recommended_max_nesting: 4,
                industry_average_score: 40.0,
                good_score_threshold: 30.0,
                excellent_score_threshold: 20.0,
            },
            _ => Self {
                language: "Generic".to_string(),
                project_type: "general".to_string(),
                recommended_max_cyclomatic: 10,
                recommended_max_cognitive: 15,
                recommended_max_length: 50,
                recommended_max_nesting: 4,
                industry_average_score: 35.0,
                good_score_threshold: 25.0,
                excellent_score_threshold: 15.0,
            },
        }
    }

    /// Compare metrics against this benchmark
    pub fn compare(&self, metrics: &ComplexityMetrics) -> ComplexityComparison {
        let score_rating = if metrics.complexity_score <= self.excellent_score_threshold {
            ScoreRating::Excellent
        } else if metrics.complexity_score <= self.good_score_threshold {
            ScoreRating::Good
        } else if metrics.complexity_score <= self.industry_average_score {
            ScoreRating::Average
        } else {
            ScoreRating::BelowAverage
        };

        ComplexityComparison {
            score_rating,
            vs_industry_average: metrics.complexity_score - self.industry_average_score,
            recommendations: self.generate_recommendations(metrics),
        }
    }

    fn generate_recommendations(&self, metrics: &ComplexityMetrics) -> Vec<String> {
        let mut recommendations = Vec::new();

        if metrics.average_cyclomatic_complexity > self.recommended_max_cyclomatic as f64 {
            recommendations.push(format!(
                "Average cyclomatic complexity ({:.1}) exceeds recommended maximum ({}). Consider breaking down complex functions.",
                metrics.average_cyclomatic_complexity, self.recommended_max_cyclomatic
            ));
        }

        if metrics.average_cognitive_complexity > self.recommended_max_cognitive as f64 {
            recommendations.push(format!(
                "Average cognitive complexity ({:.1}) exceeds recommended maximum ({}). Simplify control flow and reduce nesting.",
                metrics.average_cognitive_complexity, self.recommended_max_cognitive
            ));
        }

        if metrics.average_function_length > self.recommended_max_length as f64 {
            recommendations.push(format!(
                "Average function length ({:.1} lines) exceeds recommended maximum ({}). Extract smaller, focused functions.",
                metrics.average_function_length, self.recommended_max_length
            ));
        }

        if metrics.high_risk_functions > 0 {
            recommendations.push(format!(
                "{} functions have high complexity and need immediate attention.",
                metrics.high_risk_functions
            ));
        }

        if recommendations.is_empty() {
            recommendations.push("Code complexity is within acceptable ranges. Keep up the good work!".to_string());
        }

        recommendations
    }
}

/// Comparison result against benchmark
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityComparison {
    /// Overall score rating
    pub score_rating: ScoreRating,
    /// Difference from industry average (negative is better)
    pub vs_industry_average: f64,
    /// Specific recommendations
    pub recommendations: Vec<String>,
}

/// Score rating categories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScoreRating {
    Excellent,
    Good,
    Average,
    BelowAverage,
}

impl ScoreRating {
    pub fn description(&self) -> &'static str {
        match self {
            ScoreRating::Excellent => "Excellent - Very low complexity",
            ScoreRating::Good => "Good - Low complexity",
            ScoreRating::Average => "Average - Moderate complexity",
            ScoreRating::BelowAverage => "Below Average - High complexity",
        }
    }

    pub fn color_code(&self) -> &'static str {
        match self {
            ScoreRating::Excellent => "#28a745",   // Green
            ScoreRating::Good => "#20c997",        // Teal
            ScoreRating::Average => "#ffc107",     // Yellow
            ScoreRating::BelowAverage => "#dc3545", // Red
        }
    }
}

impl std::fmt::Display for ScoreRating {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.description())
    }
}