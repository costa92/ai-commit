use super::{Language, LanguageAnalyzer, LanguageFeature};
use once_cell::sync::Lazy;
use regex::Regex;

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

pub struct RustAnalyzer;

impl Default for RustAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl RustAnalyzer {
    pub fn new() -> Self {
        RustAnalyzer
    }

    /// 检测函数的可见性和特殊属性
    fn analyze_function_attributes(&self, line: &str) -> Vec<String> {
        let mut attributes = Vec::new();

        if line.contains("pub") {
            attributes.push("public".to_string());
        }
        if line.contains("async") {
            attributes.push("async".to_string());
        }
        if line.contains("unsafe") {
            attributes.push("unsafe".to_string());
        }
        if line.contains("const") {
            attributes.push("const".to_string());
        }

        attributes
    }

    /// 分析 Rust 项目结构
    fn analyze_project_structure(&self, file_path: &str) -> Vec<String> {
        let path_parts: Vec<&str> = file_path.split('/').collect();
        let mut suggestions = Vec::new();

        // 分析目录结构
        for (i, part) in path_parts.iter().enumerate() {
            match *part {
                "src" => {
                    if let Some(next_part) = path_parts.get(i + 1) {
                        match *next_part {
                            "bin" => suggestions.push("cli".to_string()),
                            "lib.rs" => suggestions.push("library".to_string()),
                            "main.rs" => suggestions.push("main".to_string()),
                            _ => suggestions.push(next_part.to_string()),
                        }
                    }
                }
                "tests" => suggestions.push("test".to_string()),
                "benches" => suggestions.push("bench".to_string()),
                "examples" => suggestions.push("example".to_string()),
                "docs" => suggestions.push("docs".to_string()),
                _ => {}
            }
        }

        // 从文件名推断
        if let Some(filename) = path_parts.last() {
            let name_without_ext = filename
                .split('.')
                .next()
                .unwrap_or(filename)
                .to_lowercase();

            match name_without_ext.as_str() {
                "lib" => suggestions.push("library".to_string()),
                "main" => suggestions.push("main".to_string()),
                name if name.ends_with("_test") => suggestions.push("test".to_string()),
                name if name.contains("test") => suggestions.push("test".to_string()),
                name if name.contains("bench") => suggestions.push("bench".to_string()),
                name if name.contains("error") => suggestions.push("error".to_string()),
                name if name.contains("config") => suggestions.push("config".to_string()),
                name if name.contains("util") => suggestions.push("utils".to_string()),
                _ => {
                    if !suggestions.contains(&name_without_ext) {
                        suggestions.push(name_without_ext);
                    }
                }
            }
        }

        // 去重并返回
        suggestions.sort();
        suggestions.dedup();
        suggestions
    }
}

impl LanguageAnalyzer for RustAnalyzer {
    fn language(&self) -> Language {
        Language::Rust
    }

    fn analyze_line(&self, line: &str, line_number: usize) -> Vec<LanguageFeature> {
        let mut features = Vec::new();
        let trimmed_line = line.trim();

        // 跳过注释行
        if trimmed_line.starts_with("//")
            || trimmed_line.starts_with("/*")
            || trimmed_line.starts_with("*")
            || trimmed_line.starts_with("///")
        {
            return features;
        }

        // Use 声明
        if let Some(caps) = RUST_USE_REGEX.captures(trimmed_line) {
            let use_path = caps.get(1).map(|m| m.as_str()).unwrap_or("unknown");
            features.push(LanguageFeature {
                feature_type: "use".to_string(),
                name: use_path.to_string(),
                line_number: Some(line_number),
                description: "Rust use statement for importing modules and items".to_string(),
            });
        }

        // Mod 声明
        if let Some(caps) = RUST_MOD_REGEX.captures(trimmed_line) {
            features.push(LanguageFeature {
                feature_type: "module".to_string(),
                name: caps
                    .get(1)
                    .map(|m| m.as_str())
                    .unwrap_or("unknown")
                    .to_string(),
                line_number: Some(line_number),
                description: "Rust module declaration for code organization".to_string(),
            });
        }

        // Function 定义
        if let Some(caps) = RUST_FN_REGEX.captures(trimmed_line) {
            let func_name = caps.get(1).map(|m| m.as_str()).unwrap_or("unknown");
            let attributes = self.analyze_function_attributes(trimmed_line);
            let description = if attributes.is_empty() {
                "Rust function definition".to_string()
            } else {
                format!("Rust function definition ({})", attributes.join(", "))
            };

            features.push(LanguageFeature {
                feature_type: "function".to_string(),
                name: func_name.to_string(),
                line_number: Some(line_number),
                description,
            });
        }

        // Struct 定义
        if let Some(caps) = RUST_STRUCT_REGEX.captures(trimmed_line) {
            features.push(LanguageFeature {
                feature_type: "struct".to_string(),
                name: caps
                    .get(1)
                    .map(|m| m.as_str())
                    .unwrap_or("unknown")
                    .to_string(),
                line_number: Some(line_number),
                description: "Rust struct definition for data structure".to_string(),
            });
        }

        // Enum 定义
        if let Some(caps) = RUST_ENUM_REGEX.captures(trimmed_line) {
            features.push(LanguageFeature {
                feature_type: "enum".to_string(),
                name: caps
                    .get(1)
                    .map(|m| m.as_str())
                    .unwrap_or("unknown")
                    .to_string(),
                line_number: Some(line_number),
                description: "Rust enum definition for algebraic data types".to_string(),
            });
        }

        // Trait 定义
        if let Some(caps) = RUST_TRAIT_REGEX.captures(trimmed_line) {
            features.push(LanguageFeature {
                feature_type: "trait".to_string(),
                name: caps
                    .get(1)
                    .map(|m| m.as_str())
                    .unwrap_or("unknown")
                    .to_string(),
                line_number: Some(line_number),
                description: "Rust trait definition for shared behavior".to_string(),
            });
        }

        // Impl 块
        if let Some(caps) = RUST_IMPL_REGEX.captures(trimmed_line) {
            let impl_name = if let Some(trait_name) = caps.get(1) {
                format!(
                    "{} for {}",
                    trait_name.as_str(),
                    caps.get(2).map(|m| m.as_str()).unwrap_or("unknown")
                )
            } else {
                caps.get(2)
                    .map(|m| m.as_str())
                    .unwrap_or("unknown")
                    .to_string()
            };

            features.push(LanguageFeature {
                feature_type: "impl".to_string(),
                name: impl_name,
                line_number: Some(line_number),
                description: "Rust implementation block for methods and traits".to_string(),
            });
        }

        // Const 定义
        if let Some(caps) = RUST_CONST_REGEX.captures(trimmed_line) {
            features.push(LanguageFeature {
                feature_type: "const".to_string(),
                name: caps
                    .get(1)
                    .map(|m| m.as_str())
                    .unwrap_or("unknown")
                    .to_string(),
                line_number: Some(line_number),
                description: "Rust constant definition with compile-time evaluation".to_string(),
            });
        }

        // Static 定义
        if let Some(caps) = RUST_STATIC_REGEX.captures(trimmed_line) {
            features.push(LanguageFeature {
                feature_type: "static".to_string(),
                name: caps
                    .get(1)
                    .map(|m| m.as_str())
                    .unwrap_or("unknown")
                    .to_string(),
                line_number: Some(line_number),
                description: "Rust static variable with 'static lifetime".to_string(),
            });
        }

        // Type alias 定义
        if let Some(caps) = RUST_TYPE_ALIAS_REGEX.captures(trimmed_line) {
            features.push(LanguageFeature {
                feature_type: "type_alias".to_string(),
                name: caps
                    .get(1)
                    .map(|m| m.as_str())
                    .unwrap_or("unknown")
                    .to_string(),
                line_number: Some(line_number),
                description: "Rust type alias for complex type definitions".to_string(),
            });
        }

        features
    }

    fn extract_scope_suggestions(&self, file_path: &str) -> Vec<String> {
        self.analyze_project_structure(file_path)
    }

    fn analyze_change_patterns(&self, features: &[LanguageFeature]) -> Vec<String> {
        let mut patterns = Vec::new();

        let has_modules = features.iter().any(|f| f.feature_type == "module");
        let has_traits = features.iter().any(|f| f.feature_type == "trait");
        let has_structs = features.iter().any(|f| f.feature_type == "struct");
        let has_enums = features.iter().any(|f| f.feature_type == "enum");
        let has_impls = features.iter().any(|f| f.feature_type == "impl");
        let has_functions = features.iter().any(|f| f.feature_type == "function");
        let has_uses = features.iter().any(|f| f.feature_type == "use");

        if has_modules {
            patterns.push("模块结构变更，可能影响代码组织和可见性".to_string());
        }

        if has_traits {
            patterns.push("Trait定义变更，可能影响类型约束和泛型实现".to_string());
        }

        if has_structs {
            patterns.push("结构体定义变更，可能影响内存布局和序列化".to_string());
        }

        if has_enums {
            patterns.push("枚举定义变更，可能影响模式匹配和错误处理".to_string());
        }

        if has_impls {
            patterns.push("实现块变更，可能影响方法调用和trait实现".to_string());
        }

        if has_functions {
            patterns.push("函数实现变更，需要验证类型安全和借用检查".to_string());
        }

        if has_uses {
            patterns.push("依赖导入变更，需要检查crate版本和特性兼容性".to_string());
        }

        if patterns.is_empty() {
            patterns.push("代码细节调整".to_string());
        }

        patterns
    }

    fn generate_test_suggestions(&self, features: &[LanguageFeature]) -> Vec<String> {
        let mut suggestions = Vec::new();

        // 基础测试建议
        suggestions.push("创建对应的 #[cfg(test)] 模块或独立测试文件".to_string());
        suggestions.push("使用 cargo test 运行所有测试".to_string());

        // 基于特征的特定建议
        for feature in features {
            match feature.feature_type.as_str() {
                "function" => {
                    suggestions.push(format!(
                        "为 {} 函数添加单元测试，包括边界条件",
                        feature.name
                    ));
                    if feature.description.contains("unsafe") {
                        suggestions.push("为unsafe函数添加额外的安全性测试".to_string());
                    }
                    if feature.description.contains("async") {
                        suggestions.push("为异步函数添加async测试用例".to_string());
                    }
                }
                "struct" => {
                    suggestions.push(format!("测试 {} 结构体的创建、克隆和序列化", feature.name));
                    suggestions.push("验证结构体字段的默认值和约束".to_string());
                }
                "enum" => {
                    suggestions.push(format!("测试 {} 枚举的所有变体和模式匹配", feature.name));
                    suggestions.push("验证枚举的序列化和反序列化".to_string());
                }
                "trait" => {
                    suggestions.push(format!("为 {} trait 的所有实现创建测试", feature.name));
                    suggestions.push("测试trait的默认方法和关联类型".to_string());
                }
                "impl" => {
                    suggestions.push(format!("测试 {} 实现的所有方法", feature.name));
                    suggestions.push("验证方法的正确性和错误处理".to_string());
                }
                _ => {}
            }
        }

        // Rust 特定的测试建议
        suggestions.push("运行 cargo clippy 检查代码质量".to_string());
        suggestions.push("使用 cargo fmt 保持代码格式一致".to_string());
        suggestions.push("运行 cargo test --release 进行优化版本测试".to_string());
        suggestions.push("使用 cargo bench 进行性能基准测试".to_string());
        suggestions.push("运行 cargo miri 检查unsafe代码的内存安全".to_string());

        // 去重
        suggestions.sort();
        suggestions.dedup();
        suggestions
    }

    fn assess_risks(&self, features: &[LanguageFeature]) -> Vec<String> {
        let mut risks = Vec::new();

        // Trait 变更风险
        if features.iter().any(|f| f.feature_type == "trait") {
            risks.push("Trait变更可能导致现有实现失效，影响下游crate".to_string());
        }

        // 公共API风险
        for feature in features {
            // 检查是否为公共API (通过描述或可能的pub关键字)
            if feature.description.contains("public") || feature.name.contains("pub") {
                match feature.feature_type.as_str() {
                    "function" | "struct" | "enum" | "trait" => {
                        risks.push(format!(
                            "公共 {} {} 的变更可能破坏API兼容性",
                            feature.feature_type, feature.name
                        ));
                    }
                    _ => {}
                }
            }
        }

        // Unsafe 代码风险
        if features.iter().any(|f| f.description.contains("unsafe")) {
            risks.push("Unsafe代码变更需要额外关注内存安全和未定义行为".to_string());
        }

        // 异步代码风险
        if features.iter().any(|f| f.description.contains("async")) {
            risks.push("异步代码变更需要关注Future和并发安全性".to_string());
        }

        // 模块结构风险
        if features.iter().any(|f| f.feature_type == "module") {
            risks.push("模块结构变更可能影响可见性和依赖关系".to_string());
        }

        // 依赖变更风险
        if features.iter().any(|f| f.feature_type == "use") {
            risks.push("依赖导入变更需要检查crate版本兼容性和特性标志".to_string());
        }

        // 枚举变更风险
        if features.iter().any(|f| f.feature_type == "enum") {
            risks.push("枚举变更可能导致现有模式匹配失效".to_string());
        }

        // 生命周期和借用检查风险
        let has_complex_types = features.iter().any(|f| {
            f.name.contains("&") || f.name.contains("'") || f.description.contains("lifetime")
        });
        if has_complex_types {
            risks.push("涉及生命周期的变更需要特别关注借用检查器的影响".to_string());
        }

        risks
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rust_analyzer_basic() {
        let analyzer = RustAnalyzer::new();
        assert_eq!(analyzer.language(), Language::Rust);
    }

    #[test]
    fn test_default_implementation() {
        // 测试 Default trait 实现
        let analyzer = RustAnalyzer;
        assert_eq!(analyzer.language(), Language::Rust);

        // 确保 Default 和 new() 创建的实例功能相同
        let new_analyzer = RustAnalyzer::new();
        assert_eq!(analyzer.language(), new_analyzer.language());

        // 测试默认实例能正常工作
        let line = "fn test() {}";
        let features_default = analyzer.analyze_line(line, 1);
        let features_new = new_analyzer.analyze_line(line, 1);
        assert_eq!(features_default.len(), features_new.len());
    }

    #[test]
    fn test_function_detection() {
        let analyzer = RustAnalyzer::new();
        let line = "pub async fn process_data(input: &str) -> Result<String, Error> {";
        let features = analyzer.analyze_line(line, 10);

        assert_eq!(features.len(), 1);
        assert_eq!(features[0].feature_type, "function");
        assert_eq!(features[0].name, "process_data");
        assert!(features[0].description.contains("public"));
        assert!(features[0].description.contains("async"));
    }

    #[test]
    fn test_struct_detection() {
        let analyzer = RustAnalyzer::new();
        let line = "pub struct User {";
        let features = analyzer.analyze_line(line, 15);

        assert_eq!(features.len(), 1);
        assert_eq!(features[0].feature_type, "struct");
        assert_eq!(features[0].name, "User");
    }

    #[test]
    fn test_enum_detection() {
        let analyzer = RustAnalyzer::new();
        let line = "enum Status {";
        let features = analyzer.analyze_line(line, 20);

        assert_eq!(features.len(), 1);
        assert_eq!(features[0].feature_type, "enum");
        assert_eq!(features[0].name, "Status");
    }

    #[test]
    fn test_trait_detection() {
        let analyzer = RustAnalyzer::new();
        let line = "pub trait Repository {";
        let features = analyzer.analyze_line(line, 25);

        assert_eq!(features.len(), 1);
        assert_eq!(features[0].feature_type, "trait");
        assert_eq!(features[0].name, "Repository");
    }

    #[test]
    fn test_impl_detection() {
        let analyzer = RustAnalyzer::new();
        let line = "impl Repository for UserRepository {";
        let features = analyzer.analyze_line(line, 30);

        assert_eq!(features.len(), 1);
        assert_eq!(features[0].feature_type, "impl");
        assert!(features[0].name.contains("Repository for UserRepository"));
    }

    #[test]
    fn test_use_detection() {
        let analyzer = RustAnalyzer::new();
        let line = "use std::collections::HashMap;";
        let features = analyzer.analyze_line(line, 1);

        assert_eq!(features.len(), 1);
        assert_eq!(features[0].feature_type, "use");
        assert_eq!(features[0].name, "std::collections::HashMap");
    }

    #[test]
    fn test_module_detection() {
        let analyzer = RustAnalyzer::new();
        let line = "pub mod auth;";
        let features = analyzer.analyze_line(line, 5);

        assert_eq!(features.len(), 1);
        assert_eq!(features[0].feature_type, "module");
        assert_eq!(features[0].name, "auth");
    }

    #[test]
    fn test_scope_suggestions() {
        let analyzer = RustAnalyzer::new();

        // lib.rs
        let suggestions = analyzer.extract_scope_suggestions("src/lib.rs");
        assert!(suggestions.contains(&"library".to_string()));

        // main.rs
        let suggestions = analyzer.extract_scope_suggestions("src/main.rs");
        assert!(suggestions.contains(&"main".to_string()));

        // 测试文件
        let suggestions = analyzer.extract_scope_suggestions("tests/integration_test.rs");
        assert!(suggestions.contains(&"test".to_string()));
    }

    #[test]
    fn test_change_patterns() {
        let analyzer = RustAnalyzer::new();
        let features = vec![
            LanguageFeature {
                feature_type: "trait".to_string(),
                name: "Service".to_string(),
                line_number: Some(1),
                description: "test".to_string(),
            },
            LanguageFeature {
                feature_type: "struct".to_string(),
                name: "User".to_string(),
                line_number: Some(2),
                description: "test".to_string(),
            },
        ];

        let patterns = analyzer.analyze_change_patterns(&features);
        assert!(patterns.iter().any(|p| p.contains("Trait定义变更")));
        assert!(patterns.iter().any(|p| p.contains("结构体定义变更")));
    }

    #[test]
    fn test_test_suggestions() {
        let analyzer = RustAnalyzer::new();
        let features = vec![LanguageFeature {
            feature_type: "function".to_string(),
            name: "process_data".to_string(),
            line_number: Some(1),
            description: "Rust function definition (unsafe)".to_string(),
        }];

        let suggestions = analyzer.generate_test_suggestions(&features);
        assert!(suggestions.iter().any(|s| s.contains("cargo test")));
        assert!(suggestions.iter().any(|s| s.contains("unsafe函数")));
    }

    #[test]
    fn test_risk_assessment() {
        let analyzer = RustAnalyzer::new();
        let features = vec![LanguageFeature {
            feature_type: "trait".to_string(),
            name: "PublicTrait".to_string(),
            line_number: Some(1),
            description: "Rust trait definition (public)".to_string(),
        }];

        let risks = analyzer.assess_risks(&features);
        assert!(risks.iter().any(|r| r.contains("Trait变更")));
    }
}
