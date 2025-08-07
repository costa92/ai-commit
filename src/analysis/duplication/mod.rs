pub mod detector;
pub mod result;

pub use detector::{
    DuplicationDetector, DuplicationConfig, ExactDuplicationDetector,
    StructuralDuplicationDetector, CrossFileDuplicationDetector,
    RefactoringSuggestionGenerator, ProcessedContent
};
pub use result::{
    DuplicationResult, CodeDuplication, CodeBlock, DuplicationType,
    RiskLevel, RefactoringPriority, DuplicationSummary, TypeStatistics,
    RiskStatistics, HotspotFile, RefactoringSuggestion, SuggestionType,
    ComplexityLevel
};