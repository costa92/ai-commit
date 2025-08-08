pub mod detector;
pub mod result;

pub use detector::{
    DuplicationDetector, DuplicationConfig, ExactDuplicationDetector,
    StructuralDuplicationDetector, CrossFileDuplicationDetector,
    RefactoringSuggestionGenerator, ProcessedContent, CacheStats,
    CrossFilePerformanceStats, CrossFileCacheStats
};
pub use result::{
    DuplicationResult, CodeDuplication, CodeBlock, DuplicationType,
    RiskLevel, RefactoringPriority, DuplicationSummary, TypeStatistics,
    RiskStatistics, HotspotFile, RefactoringSuggestion, SuggestionType,
    ComplexityLevel, HotspotLevel, DuplicationDistribution, FileSizeDistribution,
    TypeDistribution, RiskDistribution, DensityDistribution, DetailedDuplicationReport,
    ProjectOverview, DuplicationGrade, KeyMetrics, HotspotAnalysis, HotspotPattern,
    TrendAnalysis, HistoricalDataPoint, TrendDirection, PredictedValue,
    ImprovementRecommendation, RecommendationPriority, ImplementationDifficulty
};