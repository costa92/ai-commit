use crate::languages::{Language, LanguageAnalyzer, LanguageFeature, LanguageAnalysisResult};
use once_cell::sync::Lazy;
use regex::Regex;

pub mod ai_reviewer;
pub mod analyzer;
pub mod prompts;

#[cfg(test)]
mod tests;

pub use ai_reviewer::GoAIReviewer;
pub use analyzer::GoAnalyzer;

// Go 语言特定的正则表达式
static GO_PACKAGE_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\s*package\s+(\w+)").unwrap());
static GO_FUNC_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\s*func\s+(\w*\s*)?\(").unwrap());
static GO_STRUCT_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^\s*type\s+(\w+)\s+struct").unwrap());
static GO_INTERFACE_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^\s*type\s+(\w+)\s+interface").unwrap());
static GO_CONST_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\s*const\s+(\w+)").unwrap());
static GO_VAR_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\s*var\s+(\w+)").unwrap());
static GO_IMPORT_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"^\s*import\s+(?:"([^"]+)"|(\w+)\s+"([^"]+)")"#).unwrap());
static GO_METHOD_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^\s*func\s+\([^)]+\)\s+(\w+)").unwrap());

/// Go 特定的代码特征类型
#[derive(Debug, Clone)]
pub enum GoFeatureType {
    Package,
    Function,
    Method,
    Struct,
    Interface,
    Const,
    Var,
    Import,
    Type,
    Goroutine,
    Channel,
    Test,
    Benchmark,
}

impl GoFeatureType {
    pub fn as_str(&self) -> &str {
        match self {
            GoFeatureType::Package => "package",
            GoFeatureType::Function => "function",
            GoFeatureType::Method => "method",
            GoFeatureType::Struct => "struct",
            GoFeatureType::Interface => "interface",
            GoFeatureType::Const => "const",
            GoFeatureType::Var => "var",
            GoFeatureType::Import => "import",
            GoFeatureType::Type => "type",
            GoFeatureType::Goroutine => "goroutine",
            GoFeatureType::Channel => "channel",
            GoFeatureType::Test => "test",
            GoFeatureType::Benchmark => "benchmark",
        }
    }
}

/// Go 代码分析助手函数
pub fn extract_go_feature(line: &str, line_number: usize) -> Option<LanguageFeature> {
    if let Some(caps) = GO_PACKAGE_REGEX.captures(line) {
        return Some(LanguageFeature {
            feature_type: GoFeatureType::Package.as_str().to_string(),
            name: caps.get(1).map(|m| m.as_str().to_string()).unwrap_or_default(),
            line_number: Some(line_number),
            description: format!("Go package: {}", line.trim()),
        });
    }

    if let Some(_caps) = GO_FUNC_REGEX.captures(line) {
        let func_name = extract_function_name(line);
        return Some(LanguageFeature {
            feature_type: GoFeatureType::Function.as_str().to_string(),
            name: func_name.unwrap_or("anonymous".to_string()),
            line_number: Some(line_number),
            description: format!("Go function: {}", line.trim()),
        });
    }

    if let Some(caps) = GO_METHOD_REGEX.captures(line) {
        return Some(LanguageFeature {
            feature_type: GoFeatureType::Method.as_str().to_string(),
            name: caps.get(1).map(|m| format!("method {}", m.as_str())).unwrap_or_default(),
            line_number: Some(line_number),
            description: format!("Go method: {}", line.trim()),
        });
    }

    if let Some(caps) = GO_STRUCT_REGEX.captures(line) {
        return Some(LanguageFeature {
            feature_type: GoFeatureType::Struct.as_str().to_string(),
            name: caps.get(1).map(|m| m.as_str().to_string()).unwrap_or_default(),
            line_number: Some(line_number),
            description: format!("Go struct: {}", line.trim()),
        });
    }

    if let Some(caps) = GO_INTERFACE_REGEX.captures(line) {
        return Some(LanguageFeature {
            feature_type: GoFeatureType::Interface.as_str().to_string(),
            name: caps.get(1).map(|m| m.as_str().to_string()).unwrap_or_default(),
            line_number: Some(line_number),
            description: format!("Go interface: {}", line.trim()),
        });
    }

    if let Some(caps) = GO_IMPORT_REGEX.captures(line) {
        let import_path = caps.get(1)
            .or_else(|| caps.get(3))
            .map(|m| m.as_str().to_string())
            .unwrap_or_default();
        return Some(LanguageFeature {
            feature_type: GoFeatureType::Import.as_str().to_string(),
            name: import_path,
            line_number: Some(line_number),
            description: format!("Go import: {}", line.trim()),
        });
    }

    None
}

/// 提取函数名
fn extract_function_name(line: &str) -> Option<String> {
    if let Some(_caps) = GO_FUNC_REGEX.captures(line) {
        // 提取完整的函数声明
        let func_part = line.split('(').next().unwrap_or(line);
        let name = func_part.replace("func", "").trim().to_string();
        if name.is_empty() {
            Some("anonymous".to_string())
        } else {
            Some(name)
        }
    } else {
        None
    }
}

