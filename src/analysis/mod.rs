pub mod static_analysis;
// pub mod sensitive;
// pub mod complexity;
// pub mod duplication;
// pub mod dependency;
// pub mod coverage;
// pub mod performance;

// Re-export commonly used types
pub use static_analysis::{StaticAnalysisManager, StaticAnalysisTool, Issue, Severity};
// pub use sensitive::{SensitiveInfoDetector, SensitiveInfoResult, SensitiveInfoType, RiskLevel};
// pub use complexity::{ComplexityAnalyzer, ComplexityResult, FunctionComplexity};
// pub use duplication::{DuplicationDetector, DuplicationResult};
// pub use dependency::{DependencyAnalyzer, DependencyAnalysisResult};
// pub use coverage::{CoverageAnalyzer, CoverageReport};
// pub use performance::{PerformanceAnalyzer, PerformanceAnalysisResult};