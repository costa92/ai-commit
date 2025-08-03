use crate::languages::{Language, LanguageAnalyzer, LanguageFeature, LanguageAnalysisResult};
use once_cell::sync::Lazy;
use regex::Regex;

pub mod ai_reviewer;
pub mod analyzer;
pub mod prompts;

#[cfg(test)]
mod tests;

pub use ai_reviewer::RustAIReviewer;
pub use analyzer::RustAnalyzer;

// Rust 语言特定的正则表达式
static RUST_FN_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^\s*(?:pub\s+)?(?:async\s+)?(?:unsafe\s+)?fn\s+(\w+)").unwrap());
static RUST_STRUCT_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^\s*(?:pub\s+)?struct\s+(\w+)").unwrap());
static RUST_ENUM_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^\s*(?:pub\s+)?enum\s+(\w+)").unwrap());
static RUST_TRAIT_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^\s*(?:pub\s+)?trait\s+(\w+)").unwrap());
static RUST_IMPL_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^\s*impl\s+(?:<[^>]*>\s+)?(?:(\w+)\s+for\s+)?(\w+)").unwrap());
static RUST_MOD_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^\s*(?:pub\s+)?mod\s+(\w+)").unwrap());
static RUST_USE_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\s*use\s+([^;]+)").unwrap());
static RUST_CONST_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^\s*(?:pub\s+)?const\s+(\w+)").unwrap());
static RUST_STATIC_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^\s*(?:pub\s+)?static\s+(\w+)").unwrap());
static RUST_TYPE_ALIAS_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^\s*(?:pub\s+)?type\s+(\w+)\s*=").unwrap());

/// Rust 特定的代码特征类型
#[derive(Debug, Clone)]
pub enum RustFeatureType {
    Function,
    Method,
    Struct,
    Enum,
    Trait,
    Impl,
    Module,
    Use,
    Const,
    Static,
    TypeAlias,
    Macro,
    Test,
}

impl RustFeatureType {
    pub fn as_str(&self) -> &str {
        match self {
            RustFeatureType::Function => "function",
            RustFeatureType::Method => "method",
            RustFeatureType::Struct => "struct",
            RustFeatureType::Enum => "enum",
            RustFeatureType::Trait => "trait",
            RustFeatureType::Impl => "impl",
            RustFeatureType::Module => "module",
            RustFeatureType::Use => "use",
            RustFeatureType::Const => "const",
            RustFeatureType::Static => "static",
            RustFeatureType::TypeAlias => "type_alias",
            RustFeatureType::Macro => "macro",
            RustFeatureType::Test => "test",
        }
    }
}

/// Rust 代码分析助手函数
pub fn extract_rust_feature(line: &str, line_number: usize) -> Option<LanguageFeature> {
    if let Some(caps) = RUST_FN_REGEX.captures(line) {
        return Some(LanguageFeature {
            feature_type: RustFeatureType::Function.as_str().to_string(),
            name: caps.get(1).map(|m| m.as_str().to_string()).unwrap_or_default(),
            line_number: Some(line_number),
            description: format!("Rust function: {}", line.trim()),
        });
    }

    if let Some(caps) = RUST_STRUCT_REGEX.captures(line) {
        return Some(LanguageFeature {
            feature_type: RustFeatureType::Struct.as_str().to_string(),
            name: caps.get(1).map(|m| m.as_str().to_string()).unwrap_or_default(),
            line_number: Some(line_number),
            description: format!("Rust struct: {}", line.trim()),
        });
    }

    if let Some(caps) = RUST_ENUM_REGEX.captures(line) {
        return Some(LanguageFeature {
            feature_type: RustFeatureType::Enum.as_str().to_string(),
            name: caps.get(1).map(|m| m.as_str().to_string()).unwrap_or_default(),
            line_number: Some(line_number),
            description: format!("Rust enum: {}", line.trim()),
        });
    }

    if let Some(caps) = RUST_TRAIT_REGEX.captures(line) {
        return Some(LanguageFeature {
            feature_type: RustFeatureType::Trait.as_str().to_string(),
            name: caps.get(1).map(|m| m.as_str().to_string()).unwrap_or_default(),
            line_number: Some(line_number),
            description: format!("Rust trait: {}", line.trim()),
        });
    }

    if RUST_IMPL_REGEX.is_match(line) {
        return Some(LanguageFeature {
            feature_type: RustFeatureType::Impl.as_str().to_string(),
            name: "impl_block".to_string(),
            line_number: Some(line_number),
            description: format!("Rust impl block: {}", line.trim()),
        });
    }

    if let Some(caps) = RUST_USE_REGEX.captures(line) {
        return Some(LanguageFeature {
            feature_type: RustFeatureType::Use.as_str().to_string(),
            name: caps.get(1).map(|m| m.as_str().to_string()).unwrap_or_default(),
            line_number: Some(line_number),
            description: format!("Rust use statement: {}", line.trim()),
        });
    }

    None
}

