use super::{extract_rust_feature, RustFeatureType};
use crate::languages::{Language, LanguageAnalyzer, LanguageFeature};

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
    #[allow(dead_code)]
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

    /// 从文件路径提取 Rust 特定的作用域建议
    fn extract_rust_scope_from_path(&self, file_path: &str) -> Vec<String> {
        let mut scopes = Vec::new();

        // 基于路径结构的作用域建议
        if file_path.contains("src/main.rs") {
            scopes.push("app".to_string());
        } else if file_path.contains("src/lib.rs") {
            scopes.push("lib".to_string());
        } else if file_path.contains("src/bin/") {
            scopes.push("bin".to_string());
        } else if file_path.contains("tests/") {
            scopes.push("test".to_string());
        } else if file_path.contains("examples/") {
            scopes.push("example".to_string());
        } else if file_path.contains("benches/") {
            scopes.push("bench".to_string());
        }

        // 基于模块名的作用域建议
        if let Some(file_name) = std::path::Path::new(file_path).file_stem() {
            if let Some(name) = file_name.to_str() {
                if name != "main" && name != "lib" && name != "mod" {
                    scopes.push(name.to_string());
                }
            }
        }

        // 如果是在子目录中，添加目录名作为作用域
        if let Some(parent) = std::path::Path::new(file_path).parent() {
            if let Some(dir_name) = parent.file_name() {
                if let Some(name) = dir_name.to_str() {
                    if name != "src" && name != "tests" && name != "examples" {
                        scopes.push(name.to_string());
                    }
                }
            }
        }

        scopes
    }

    /// 分析 Rust 特定的变更模式
    fn analyze_rust_change_patterns(&self, features: &[LanguageFeature]) -> Vec<String> {
        let mut patterns = Vec::new();

        let function_count = features
            .iter()
            .filter(|f| f.feature_type == "function")
            .count();
        let struct_count = features
            .iter()
            .filter(|f| f.feature_type == "struct")
            .count();
        let trait_count = features
            .iter()
            .filter(|f| f.feature_type == "trait")
            .count();
        let impl_count = features.iter().filter(|f| f.feature_type == "impl").count();
        let use_count = features.iter().filter(|f| f.feature_type == "use").count();

        if function_count > 0 {
            patterns.push(format!("新增 {} 个函数", function_count));
        }
        if struct_count > 0 {
            patterns.push(format!("新增 {} 个结构体", struct_count));
        }
        if trait_count > 0 {
            patterns.push(format!("新增 {} 个 trait", trait_count));
        }
        if impl_count > 0 {
            patterns.push(format!("新增 {} 个 impl 块", impl_count));
        }
        if use_count > 2 {
            patterns.push("大量依赖导入变更".to_string());
        }

        // 检测测试相关变更
        let test_functions = features
            .iter()
            .filter(|f| f.description.contains("#[test]") || f.name.starts_with("test_"))
            .count();

        if test_functions > 0 {
            patterns.push(format!("新增 {} 个测试函数", test_functions));
        }

        // 检测异步相关变更
        let async_functions = features
            .iter()
            .filter(|f| f.description.contains("async"))
            .count();

        if async_functions > 0 {
            patterns.push(format!("新增 {} 个异步函数", async_functions));
        }

        patterns
    }

    /// 生成 Rust 特定的测试建议
    fn generate_rust_test_suggestions(&self, features: &[LanguageFeature]) -> Vec<String> {
        let mut suggestions = Vec::new();

        let has_functions = features.iter().any(|f| f.feature_type == "function");
        let has_structs = features.iter().any(|f| f.feature_type == "struct");
        let has_traits = features.iter().any(|f| f.feature_type == "trait");
        let has_async = features.iter().any(|f| f.description.contains("async"));

        if has_functions {
            suggestions.push("为新增函数编写单元测试".to_string());
        }
        if has_structs {
            suggestions.push("为结构体实现 Debug、Clone 等常用 trait 的测试".to_string());
        }
        if has_traits {
            suggestions.push("为 trait 实现编写集成测试".to_string());
        }
        if has_async {
            suggestions.push("使用 tokio::test 测试异步函数".to_string());
        }

        // 基于 Rust 最佳实践的建议
        suggestions.push("运行 cargo test 确保所有测试通过".to_string());
        suggestions.push("使用 cargo clippy 检查代码质量".to_string());
        suggestions.push("运行 cargo fmt 格式化代码".to_string());

        if features.iter().any(|f| f.description.contains("unsafe")) {
            suggestions.push("为 unsafe 代码编写额外的安全性测试".to_string());
        }

        suggestions
    }

    /// 生成 Rust 特定的风险评估
    fn assess_rust_risks(&self, features: &[LanguageFeature]) -> Vec<String> {
        let mut risks = Vec::new();

        let has_unsafe = features.iter().any(|f| f.description.contains("unsafe"));
        let has_many_impl = features.iter().filter(|f| f.feature_type == "impl").count() > 3;
        let has_many_traits = features
            .iter()
            .filter(|f| f.feature_type == "trait")
            .count()
            > 2;

        if has_unsafe {
            risks.push("包含 unsafe 代码，需要额外的安全性审查".to_string());
        }
        if has_many_impl {
            risks.push("大量 impl 块可能增加代码复杂性".to_string());
        }
        if has_many_traits {
            risks.push("多个新 trait 可能影响 API 稳定性".to_string());
        }

        // 检查生命周期和所有权相关风险
        let has_lifetime_params = features
            .iter()
            .any(|f| f.description.contains("'") && f.description.contains("<"));
        if has_lifetime_params {
            risks.push("涉及生命周期参数，需要仔细检查所有权管理".to_string());
        }

        // 检查泛型相关风险
        let has_generics = features
            .iter()
            .any(|f| f.description.contains("<") && f.description.contains(">"));
        if has_generics {
            risks.push("使用泛型，需要验证类型约束和编译时检查".to_string());
        }

        risks
    }
}

impl LanguageAnalyzer for RustAnalyzer {
    fn language(&self) -> Language {
        Language::Rust
    }

    fn analyze_line(&self, line: &str, line_number: usize) -> Vec<LanguageFeature> {
        let mut features = Vec::new();

        if let Some(feature) = extract_rust_feature(line, line_number) {
            features.push(feature);
        }

        // 检测宏定义
        if line.trim().starts_with("macro_rules!") {
            features.push(LanguageFeature {
                feature_type: RustFeatureType::Macro.as_str().to_string(),
                name: "macro_definition".to_string(),
                line_number: Some(line_number),
                description: format!("Rust macro: {}", line.trim()),
            });
        }

        // 检测测试函数
        if line.contains("#[test]") || (line.contains("fn") && line.contains("test_")) {
            features.push(LanguageFeature {
                feature_type: RustFeatureType::Test.as_str().to_string(),
                name: "test_function".to_string(),
                line_number: Some(line_number),
                description: format!("Rust test: {}", line.trim()),
            });
        }

        features
    }

    fn extract_scope_suggestions(&self, file_path: &str) -> Vec<String> {
        self.extract_rust_scope_from_path(file_path)
    }

    fn analyze_change_patterns(&self, features: &[LanguageFeature]) -> Vec<String> {
        self.analyze_rust_change_patterns(features)
    }

    fn generate_test_suggestions(&self, features: &[LanguageFeature]) -> Vec<String> {
        self.generate_rust_test_suggestions(features)
    }

    fn assess_risks(&self, features: &[LanguageFeature]) -> Vec<String> {
        self.assess_rust_risks(features)
    }
}

#[cfg(test)]
mod tests {
    include!("analyzer_tests.rs");
}
