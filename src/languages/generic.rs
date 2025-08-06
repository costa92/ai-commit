use super::{LanguageAnalyzer, LanguageFeature};

/// 通用语言分析器
/// 用于处理未知或不支持的语言
pub struct GenericAnalyzer;

impl GenericAnalyzer {
    pub fn new() -> Self {
        Self
    }
}

impl LanguageAnalyzer for GenericAnalyzer {
    fn analyze_features(&self, content: &str) -> Vec<LanguageFeature> {
        let mut features = Vec::new();

        // 基于通用模式的特征提取
        let lines: Vec<&str> = content.lines().collect();

        for (line_num, line) in lines.iter().enumerate() {
            let trimmed = line.trim();

            // 检测函数定义的通用模式
            if self.looks_like_function(trimmed) {
                if let Some(func_name) = self.extract_function_name(trimmed) {
                    features.push(LanguageFeature::Function(func_name));
                }
            }

            // 检测类定义的通用模式
            if self.looks_like_class(trimmed) {
                if let Some(class_name) = self.extract_class_name(trimmed) {
                    features.push(LanguageFeature::Class(class_name));
                }
            }

            // 检测模块/包的通用模式
            if self.looks_like_module(trimmed) {
                if let Some(module_name) = self.extract_module_name(trimmed) {
                    features.push(LanguageFeature::Module(module_name));
                }
            }
        }

        features
    }
}

impl GenericAnalyzer {
    fn looks_like_function(&self, line: &str) -> bool {
        // 检测常见的函数定义模式
        line.contains("function ") ||
        line.contains("def ") ||
        line.contains("fn ") ||
        line.contains("func ") ||
        (line.contains("(") && line.contains(")") && (line.contains("{") || line.ends_with(":")))
    }

    fn extract_function_name(&self, line: &str) -> Option<String> {
        // 尝试提取函数名
        if line.contains("function ") {
            return self.extract_name_after_keyword(line, "function ");
        }

        if line.contains("def ") {
            return self.extract_name_after_keyword(line, "def ");
        }

        if line.contains("fn ") {
            return self.extract_name_after_keyword(line, "fn ");
        }

        if line.contains("func ") {
            return self.extract_name_after_keyword(line, "func ");
        }

        None
    }

    fn looks_like_class(&self, line: &str) -> bool {
        line.contains("class ") ||
        line.contains("struct ") ||
        line.contains("interface ") ||
        line.contains("type ")
    }

    fn extract_class_name(&self, line: &str) -> Option<String> {
        if line.contains("class ") {
            return self.extract_name_after_keyword(line, "class ");
        }

        if line.contains("struct ") {
            return self.extract_name_after_keyword(line, "struct ");
        }

        if line.contains("interface ") {
            return self.extract_name_after_keyword(line, "interface ");
        }

        if line.contains("type ") {
            return self.extract_name_after_keyword(line, "type ");
        }

        None
    }

    fn looks_like_module(&self, line: &str) -> bool {
        line.contains("package ") ||
        line.contains("module ") ||
        line.contains("namespace ") ||
        line.starts_with("import ") ||
        line.starts_with("use ")
    }

    fn extract_module_name(&self, line: &str) -> Option<String> {
        if line.contains("package ") {
            return self.extract_name_after_keyword(line, "package ");
        }

        if line.contains("module ") {
            return self.extract_name_after_keyword(line, "module ");
        }

        if line.contains("namespace ") {
            return self.extract_name_after_keyword(line, "namespace ");
        }

        None
    }

    fn extract_name_after_keyword(&self, line: &str, keyword: &str) -> Option<String> {
        if let Some(start) = line.find(keyword) {
            let after_keyword = &line[start + keyword.len()..];
            let name = after_keyword
                .split_whitespace()
                .next()?
                .split('(')
                .next()?
                .split('{')
                .next()?
                .split(':')
                .next()?
                .trim();

            if !name.is_empty() && name.chars().all(|c| c.is_alphanumeric() || c == '_') {
                Some(name.to_string())
            } else {
                None
            }
        } else {
            None
        }
    }
}