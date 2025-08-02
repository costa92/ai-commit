use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 语言特定的代码特征
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageFeature {
    pub feature_type: String,
    pub name: String,
    pub line_number: Option<usize>,
    pub description: String,
}

/// 语言分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageAnalysisResult {
    pub language: Language,
    pub features: Vec<LanguageFeature>,
    pub scope_suggestions: Vec<String>,
    pub change_patterns: Vec<String>,
}

/// 支持的编程语言枚举
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Language {
    Go,
    TypeScript,
    JavaScript,
    Rust,
    Unknown,
}

impl Language {
    pub fn from_file_path(path: &str) -> Self {
        let extension = std::path::Path::new(path)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");

        match extension {
            "go" => Language::Go,
            "ts" | "tsx" => Language::TypeScript,
            "js" | "jsx" => Language::JavaScript,
            "rs" => Language::Rust,
            _ => Language::Unknown,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Language::Go => "go",
            Language::TypeScript => "typescript",
            Language::JavaScript => "javascript",
            Language::Rust => "rust",
            Language::Unknown => "unknown",
        }
    }
}

/// 语言分析器特征
pub trait LanguageAnalyzer {
    /// 获取支持的语言类型
    fn language(&self) -> Language;
    
    /// 分析代码行，提取语言特定特征
    fn analyze_line(&self, line: &str, line_number: usize) -> Vec<LanguageFeature>;
    
    /// 从文件路径提取作用域建议
    fn extract_scope_suggestions(&self, file_path: &str) -> Vec<String>;
    
    /// 分析变更模式，识别常见的变更类型
    fn analyze_change_patterns(&self, features: &[LanguageFeature]) -> Vec<String>;
    
    /// 生成语言特定的测试建议
    fn generate_test_suggestions(&self, features: &[LanguageFeature]) -> Vec<String>;
    
    /// 生成语言特定的风险评估
    fn assess_risks(&self, features: &[LanguageFeature]) -> Vec<String>;
    
    /// 完整分析文件变更
    fn analyze_file_changes(&self, file_path: &str, added_lines: &[String]) -> LanguageAnalysisResult {
        let mut features = Vec::new();
        
        for (index, line) in added_lines.iter().enumerate() {
            let line_features = self.analyze_line(line, index + 1);
            features.extend(line_features);
        }
        
        let scope_suggestions = self.extract_scope_suggestions(file_path);
        let change_patterns = self.analyze_change_patterns(&features);
        
        LanguageAnalysisResult {
            language: self.language(),
            features,
            scope_suggestions,
            change_patterns,
        }
    }
}

/// 语言分析器工厂
pub struct LanguageAnalyzerFactory;

pub mod go;
pub mod typescript;
pub mod javascript;
pub mod rust;
pub mod generic;
pub mod review_service;

// 导出review_service的主要类型
pub use review_service::{CodeReviewService, CodeReviewReport, FileAnalysisResult, ReviewSummary};

impl LanguageAnalyzerFactory {
    pub fn create_analyzer(language: &Language) -> Box<dyn LanguageAnalyzer> {
        match language {
            Language::Go => Box::new(go::GoAnalyzer::new()),
            Language::TypeScript => Box::new(typescript::TypeScriptAnalyzer::new()),
            Language::JavaScript => Box::new(javascript::JavaScriptAnalyzer::new()),
            Language::Rust => Box::new(rust::RustAnalyzer::new()),
            Language::Unknown => Box::new(generic::GenericAnalyzer::new()),
        }
    }
    
    pub fn get_supported_languages() -> Vec<Language> {
        vec![
            Language::Go,
            Language::TypeScript,
            Language::JavaScript,
            Language::Rust,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_from_file_path() {
        assert_eq!(Language::from_file_path("main.go"), Language::Go);
        assert_eq!(Language::from_file_path("component.tsx"), Language::TypeScript);
        assert_eq!(Language::from_file_path("script.ts"), Language::TypeScript);
        assert_eq!(Language::from_file_path("app.js"), Language::JavaScript);
        assert_eq!(Language::from_file_path("component.jsx"), Language::JavaScript);
        assert_eq!(Language::from_file_path("main.rs"), Language::Rust);
        assert_eq!(Language::from_file_path("config.json"), Language::Unknown);
    }

    #[test]
    fn test_language_as_str() {
        assert_eq!(Language::Go.as_str(), "go");
        assert_eq!(Language::TypeScript.as_str(), "typescript");
        assert_eq!(Language::JavaScript.as_str(), "javascript");
        assert_eq!(Language::Rust.as_str(), "rust");
        assert_eq!(Language::Unknown.as_str(), "unknown");
    }

    #[test]
    fn test_supported_languages() {
        let languages = LanguageAnalyzerFactory::get_supported_languages();
        assert!(languages.contains(&Language::Go));
        assert!(languages.contains(&Language::TypeScript));
        assert!(languages.contains(&Language::JavaScript));
        assert!(languages.contains(&Language::Rust));
    }
}