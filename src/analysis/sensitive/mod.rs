pub mod config;
pub mod detector;
pub mod masking;
pub mod patterns;
pub mod predefined_patterns;
pub mod result;

pub use config::{
    SensitiveConfig, ConfigManager, DetectorConfig, WhitelistConfig,
    CustomRulesConfig, WhitelistEntryConfig, CustomRuleConfig,
    MaskingConfigMap, RiskAssessmentConfig, ConfigValidationError
};
pub use detector::{
    SensitiveInfoDetector, SensitiveDetectorConfig, WhitelistEntry,
    DetectionStatistics, AggregatedSensitiveResult
};
pub use masking::{
    SensitiveInfoMasker, MaskingConfig, MaskingAlgorithm,
    RiskAssessor, RiskAssessment, SecurityRecommendation,
    RecommendationPriority, RecommendationCategory, RiskThresholds
};
pub use patterns::{SensitivePattern, PatternEngine, PatternMatch};
pub use predefined_patterns::{PredefinedPatterns, PatternStatistics};
pub use result::{
    SensitiveInfoResult, SensitiveInfoType, RiskLevel,
    SensitiveItem, SensitiveSummary
};