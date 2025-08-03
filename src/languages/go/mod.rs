use crate::languages::LanguageFeature;
use once_cell::sync::Lazy;
use regex::Regex;

pub mod ai_reviewer;
pub mod analyzer;
pub mod prompts;

#[cfg(test)]
mod tests;

pub use ai_reviewer::GoAIReviewer;
pub use analyzer::GoAnalyzer;

// Go 语言特定的正则表达式 - 改进错误处理
static GO_PACKAGE_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^\s*package\s+(\w+)")
        .expect("Failed to compile GO_PACKAGE_REGEX - this is a bug in the regex pattern")
});
static GO_FUNC_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^\s*func\s+(\w*\s*)?\(")
        .expect("Failed to compile GO_FUNC_REGEX - this is a bug in the regex pattern")
});
static GO_STRUCT_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^\s*type\s+(\w+)\s+struct")
        .expect("Failed to compile GO_STRUCT_REGEX - this is a bug in the regex pattern")
});
static GO_INTERFACE_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^\s*type\s+(\w+)\s+interface")
        .expect("Failed to compile GO_INTERFACE_REGEX - this is a bug in the regex pattern")
});
#[allow(dead_code)]
static GO_CONST_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^\s*const\s+(\w+)")
        .expect("Failed to compile GO_CONST_REGEX - this is a bug in the regex pattern")
});
#[allow(dead_code)]
static GO_VAR_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^\s*var\s+(\w+)")
        .expect("Failed to compile GO_VAR_REGEX - this is a bug in the regex pattern")
});
static GO_IMPORT_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"^\s*import\s+(?:"([^"]+)"|(\w+)\s+"([^"]+)")"#)
        .expect("Failed to compile GO_IMPORT_REGEX - this is a bug in the regex pattern")
});
static GO_METHOD_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^\s*func\s+\([^)]+\)\s+(\w+)")
        .expect("Failed to compile GO_METHOD_REGEX - this is a bug in the regex pattern")
});

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

/// Go 代码分析助手函数 - 改进错误处理
pub fn extract_go_feature(line: &str, line_number: usize) -> Option<LanguageFeature> {
    // 安全检查：确保输入不为空且不会导致正则表达式问题
    let line = line.trim();
    if line.is_empty() {
        return None;
    }

    if let Some(caps) = GO_PACKAGE_REGEX.captures(line) {
        return Some(LanguageFeature {
            feature_type: GoFeatureType::Package.as_str().to_string(),
            name: caps
                .get(1)
                .map(|m| m.as_str().to_string())
                .unwrap_or_else(|| "unknown".to_string()),
            line_number: Some(line_number),
            description: format!("Go package: {}", line),
        });
    }

    if GO_FUNC_REGEX.is_match(line) {
        let func_name = extract_function_name(line);
        return Some(LanguageFeature {
            feature_type: GoFeatureType::Function.as_str().to_string(),
            name: func_name.unwrap_or_else(|| "anonymous".to_string()),
            line_number: Some(line_number),
            description: format!("Go function: {}", line),
        });
    }

    if let Some(caps) = GO_METHOD_REGEX.captures(line) {
        return Some(LanguageFeature {
            feature_type: GoFeatureType::Method.as_str().to_string(),
            name: caps
                .get(1)
                .map(|m| format!("method {}", m.as_str()))
                .unwrap_or_else(|| "method unknown".to_string()),
            line_number: Some(line_number),
            description: format!("Go method: {}", line),
        });
    }

    if let Some(caps) = GO_STRUCT_REGEX.captures(line) {
        return Some(LanguageFeature {
            feature_type: GoFeatureType::Struct.as_str().to_string(),
            name: caps
                .get(1)
                .map(|m| m.as_str().to_string())
                .unwrap_or_else(|| "unknown".to_string()),
            line_number: Some(line_number),
            description: format!("Go struct: {}", line),
        });
    }

    if let Some(caps) = GO_INTERFACE_REGEX.captures(line) {
        return Some(LanguageFeature {
            feature_type: GoFeatureType::Interface.as_str().to_string(),
            name: caps
                .get(1)
                .map(|m| m.as_str().to_string())
                .unwrap_or_else(|| "unknown".to_string()),
            line_number: Some(line_number),
            description: format!("Go interface: {}", line),
        });
    }

    if let Some(caps) = GO_IMPORT_REGEX.captures(line) {
        let import_path = caps
            .get(1)
            .or_else(|| caps.get(3))
            .map(|m| m.as_str().to_string())
            .unwrap_or_else(|| "unknown".to_string());
        return Some(LanguageFeature {
            feature_type: GoFeatureType::Import.as_str().to_string(),
            name: import_path,
            line_number: Some(line_number),
            description: format!("Go import: {}", line),
        });
    }

    None
}

/// 提取函数名 - 改进的错误处理
fn extract_function_name(line: &str) -> Option<String> {
    if let Some(_caps) = GO_FUNC_REGEX.captures(line) {
        // 安全地提取函数声明
        let func_part = match line.find('(') {
            Some(pos) => &line[..pos],
            None => line,
        };

        // 移除 "func" 关键字并清理空白字符
        let name = func_part.replace("func", "").trim().to_string();

        if name.is_empty() || name.contains(' ') {
            // 如果名称为空或包含空格（可能是接收者方法），返回 anonymous
            Some("anonymous".to_string())
        } else {
            Some(name)
        }
    } else {
        None
    }
}
