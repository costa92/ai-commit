pub mod detector;
pub mod go;
pub mod rust;
pub mod typescript;
pub mod generic;

pub use detector::{LanguageDetector, LanguageDetectionResult};

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Language {
    Go,
    Rust,
    TypeScript,
    JavaScript,
    Python,
    Java,
    C,
    Cpp,
    Unknown,
}

impl Language {
    pub fn from_extension(file_path: &str) -> Option<Self> {
        let extension = std::path::Path::new(file_path)
            .extension()?
            .to_str()?
            .to_lowercase();

        match extension.as_str() {
            "go" => Some(Language::Go),
            "rs" => Some(Language::Rust),
            "ts" => Some(Language::TypeScript),
            "js" | "jsx" => Some(Language::JavaScript),
            "py" => Some(Language::Python),
            "java" => Some(Language::Java),
            "c" => Some(Language::C),
            "cpp" | "cc" | "cxx" => Some(Language::Cpp),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum LanguageFeature {
    Package(String),
    Function(String),
    Struct(String),
    Interface(String),
    Class(String),
    Module(String),
}

pub trait LanguageAnalyzer {
    fn analyze_features(&self, content: &str) -> Vec<LanguageFeature>;
}