pub mod manager;
pub mod tools;
pub mod result;
pub mod incremental;

pub use manager::{StaticAnalysisManager, StaticAnalysisConfig, IssueStatistics};
pub use result::{Issue, Severity, IssueCategory, StaticAnalysisResult};
pub use incremental::{IncrementalAnalysisManager, FileChangeDetector, GitDiffAnalyzer, AnalysisCache};

use async_trait::async_trait;
use crate::languages::Language;

#[async_trait]
pub trait StaticAnalysisTool: Send + Sync {
    fn name(&self) -> &str;
    fn supported_languages(&self) -> Vec<Language>;
    async fn analyze(&self, file_path: &str, content: &str) -> anyhow::Result<Vec<Issue>>;
    fn is_available(&self) -> bool;
}