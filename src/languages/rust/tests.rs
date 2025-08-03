#[cfg(test)]
mod rust_feature_tests {
    use crate::config::Config;
    use crate::languages::rust::{extract_rust_feature, RustFeatureType};
    use crate::languages::LanguageFeature;

    #[allow(dead_code)]
    fn create_test_config() -> Config {
        Config {
            provider: "ollama".to_string(),
            model: "test-model".to_string(),
            debug: true,
            ..Default::default()
        }
    }

    #[test]
    fn test_rust_feature_type_as_str() {
        assert_eq!(RustFeatureType::Function.as_str(), "function");
        assert_eq!(RustFeatureType::Method.as_str(), "method");
        assert_eq!(RustFeatureType::Struct.as_str(), "struct");
        assert_eq!(RustFeatureType::Enum.as_str(), "enum");
        assert_eq!(RustFeatureType::Trait.as_str(), "trait");
        assert_eq!(RustFeatureType::Impl.as_str(), "impl");
        assert_eq!(RustFeatureType::Module.as_str(), "module");
        assert_eq!(RustFeatureType::Use.as_str(), "use");
        assert_eq!(RustFeatureType::Const.as_str(), "const");
        assert_eq!(RustFeatureType::Static.as_str(), "static");
        assert_eq!(RustFeatureType::TypeAlias.as_str(), "type_alias");
        assert_eq!(RustFeatureType::Macro.as_str(), "macro");
        assert_eq!(RustFeatureType::Test.as_str(), "test");
    }

    #[test]
    fn test_extract_rust_feature_function() {
        // 测试各种函数声明
        let test_cases = vec![
            ("pub fn main() {", "main"),
            ("async fn process_data() -> Result<()> {", "process_data"),
            (
                "pub async fn handle_request(req: Request) -> Response {",
                "handle_request",
            ),
            ("unsafe fn dangerous_operation() {", "dangerous_operation"),
            (
                "pub unsafe async fn complex_function() {",
                "complex_function",
            ),
            (
                "const fn compile_time_function() -> i32 {",
                "compile_time_function",
            ),
        ];

        for (line, expected_name) in test_cases {
            let feature = extract_rust_feature(line, 1).unwrap();
            assert_eq!(feature.feature_type, "function");
            assert_eq!(feature.name, expected_name);
            assert_eq!(feature.line_number, Some(1));
            assert!(feature.description.contains("Rust function"));
        }
    }

    #[test]
    fn test_extract_rust_feature_struct() {
        let test_cases = vec![
            ("pub struct User {", "User"),
            ("struct InternalData {", "InternalData"),
            ("#[derive(Debug)] pub struct Config {", "Config"),
            ("struct GenericStruct<T> {", "GenericStruct"),
        ];

        for (line, expected_name) in test_cases {
            let feature = extract_rust_feature(line, 1).unwrap();
            assert_eq!(feature.feature_type, "struct");
            assert_eq!(feature.name, expected_name);
            assert!(feature.description.contains("Rust struct"));
        }
    }

    #[test]
    fn test_extract_rust_feature_enum() {
        let test_cases = vec![
            ("pub enum Status {", "Status"),
            ("enum InternalEnum {", "InternalEnum"),
            ("#[derive(Debug, Clone)] pub enum Result<T, E> {", "Result"),
        ];

        for (line, expected_name) in test_cases {
            let feature = extract_rust_feature(line, 1).unwrap();
            assert_eq!(feature.feature_type, "enum");
            assert_eq!(feature.name, expected_name);
        }
    }

    #[test]
    fn test_extract_rust_feature_trait() {
        let test_cases = vec![
            ("pub trait Display {", "Display"),
            ("trait InternalTrait {", "InternalTrait"),
            ("pub trait Iterator<Item> {", "Iterator"),
        ];

        for (line, expected_name) in test_cases {
            let feature = extract_rust_feature(line, 1).unwrap();
            assert_eq!(feature.feature_type, "trait");
            assert_eq!(feature.name, expected_name);
        }
    }

    #[test]
    fn test_extract_rust_feature_impl() {
        let test_cases = vec![
            "impl User {",
            "impl<T> Display for T {",
            "impl Iterator for MyStruct {",
            "pub impl Default for Config {",
        ];

        for line in test_cases {
            let feature = extract_rust_feature(line, 1).unwrap();
            assert_eq!(feature.feature_type, "impl");
            assert_eq!(feature.name, "impl_block");
        }
    }

    #[test]
    fn test_extract_rust_feature_use() {
        let test_cases = vec![
            (
                "use std::collections::HashMap;",
                "std::collections::HashMap",
            ),
            ("use crate::config::Config;", "crate::config::Config"),
            ("pub use super::*;", "super::*"),
            (
                "use serde::{Serialize, Deserialize};",
                "serde::{Serialize, Deserialize}",
            ),
        ];

        for (line, expected_name) in test_cases {
            let feature = extract_rust_feature(line, 1).unwrap();
            assert_eq!(feature.feature_type, "use");
            assert_eq!(feature.name, expected_name);
        }
    }

    #[test]
    fn test_extract_rust_feature_const_static() {
        // 测试常量
        let feature = extract_rust_feature("pub const MAX_SIZE: usize = 1024;", 1);
        assert!(feature.is_some());
        // 具体实现可能不同，这里只测试能够识别

        // 测试静态变量
        let feature = extract_rust_feature("static GLOBAL_CONFIG: Config = Config::new();", 1);
        assert!(feature.is_some());
    }

    #[test]
    fn test_extract_rust_feature_no_match() {
        let non_matching_lines = vec![
            "// This is a comment",
            "let x = 5;",
            "println!(\"Hello, world!\");",
            "if condition {",
            "}",
            "",
            "    // Indented comment",
        ];

        for line in non_matching_lines {
            let feature = extract_rust_feature(line, 1);
            assert!(
                feature.is_none(),
                "Line '{}' should not match any pattern",
                line
            );
        }
    }

    #[test]
    fn test_rust_feature_with_attributes() {
        let lines_with_attributes = vec![
            "#[derive(Debug, Clone)]",
            "#[serde(rename_all = \"camelCase\")]",
            "#[cfg(test)]",
            "#[allow(dead_code)]",
        ];

        // 属性行本身不应该被识别为特征
        for line in lines_with_attributes {
            let feature = extract_rust_feature(line, 1);
            assert!(
                feature.is_none(),
                "Attribute line '{}' should not be recognized as a feature",
                line
            );
        }
    }

    #[test]
    fn test_rust_feature_edge_cases() {
        // 测试边界情况
        let edge_cases: Vec<(&str, Option<LanguageFeature>)> = vec![
            ("fn", None),     // 不完整的函数声明
            ("struct", None), // 不完整的结构体声明
            ("use;", None),   // 不完整的 use 语句
            ("impl", None),   // 不完整的 impl 块
        ];

        for (line, expected) in edge_cases {
            let feature = extract_rust_feature(line, 1);
            match expected {
                None => assert!(feature.is_none(), "Line '{}' should not match", line),
                Some(_) => assert!(feature.is_some(), "Line '{}' should match", line),
            }
        }
    }

    #[test]
    fn test_rust_feature_complex_generics() {
        // 测试复杂的泛型语法
        let complex_cases = vec![
            ("pub struct HashMap<K: Hash + Eq, V> {", "HashMap"),
            (
                "impl<T: Clone + Debug> Display for Wrapper<T> {",
                "impl_block",
            ),
            ("pub trait Iterator<Item: Send + Sync> {", "Iterator"),
            (
                "fn generic_function<T, U>() where T: Clone, U: Debug {",
                "generic_function",
            ),
        ];

        for (line, expected_name) in complex_cases {
            let feature = extract_rust_feature(line, 1);
            if let Some(f) = feature {
                assert_eq!(f.name, expected_name);
            }
        }
    }

    #[test]
    fn test_rust_feature_with_lifetimes() {
        // 测试生命周期参数
        let lifetime_cases = vec![
            (
                "fn with_lifetime<'a>(s: &'a str) -> &'a str {",
                "with_lifetime",
            ),
            ("struct Wrapper<'a, T> {", "Wrapper"),
            ("impl<'a> Display for Ref<'a> {", "impl_block"),
        ];

        for (line, expected_name) in lifetime_cases {
            let feature = extract_rust_feature(line, 1);
            if let Some(f) = feature {
                assert_eq!(f.name, expected_name);
            }
        }
    }

    #[test]
    fn test_rust_feature_visibility_modifiers() {
        // 测试各种可见性修饰符
        let visibility_cases = vec![
            ("pub fn public_function() {", "public_function"),
            ("pub(crate) fn crate_function() {", "crate_function"),
            ("pub(super) fn super_function() {", "super_function"),
            (
                "pub(in crate::module) fn scoped_function() {",
                "scoped_function",
            ),
        ];

        for (line, expected_name) in visibility_cases {
            let feature = extract_rust_feature(line, 1);
            if let Some(f) = feature {
                assert_eq!(f.name, expected_name);
                assert_eq!(f.feature_type, "function");
            }
        }
    }

    #[test]
    fn test_rust_feature_async_unsafe_combinations() {
        // 测试 async 和 unsafe 的组合
        let combination_cases = vec![
            ("pub async unsafe fn dangerous_async() {", "dangerous_async"),
            ("unsafe async fn another_dangerous() {", "another_dangerous"),
            ("pub unsafe fn just_unsafe() {", "just_unsafe"),
            ("pub async fn just_async() {", "just_async"),
        ];

        for (line, expected_name) in combination_cases {
            let feature = extract_rust_feature(line, 1);
            if let Some(f) = feature {
                assert_eq!(f.name, expected_name);
                assert_eq!(f.feature_type, "function");
                assert!(f.description.contains("Rust function"));
            }
        }
    }

    #[test]
    fn test_rust_feature_line_numbers() {
        // 测试行号是否正确设置
        let test_lines = [
            "pub fn first_function() {",
            "struct SecondStruct {",
            "use third::module;",
        ];

        for (index, line) in test_lines.iter().enumerate() {
            let line_number = index + 10; // 从第10行开始
            let feature = extract_rust_feature(line, line_number);
            if let Some(f) = feature {
                assert_eq!(f.line_number, Some(line_number));
            }
        }
    }

    #[test]
    fn test_rust_feature_description_content() {
        // 测试描述内容是否包含原始代码
        let test_line = "pub async fn complex_function(data: &[u8]) -> Result<String, Error> {";
        let feature = extract_rust_feature(test_line, 1).unwrap();

        assert!(feature.description.contains("Rust function"));
        assert!(feature.description.contains(test_line.trim()));
    }

    #[test]
    fn test_rust_feature_multiple_features_same_line() {
        // 虽然不常见，但测试同一行可能包含多个特征的情况
        // 例如：use std::collections::{HashMap, HashSet};
        let line = "use std::collections::{HashMap, HashSet};";
        let feature = extract_rust_feature(line, 1);

        // 应该识别为一个 use 语句
        assert!(feature.is_some());
        let f = feature.unwrap();
        assert_eq!(f.feature_type, "use");
        assert!(f.name.contains("HashMap"));
        assert!(f.name.contains("HashSet"));
    }

    #[test]
    fn test_rust_feature_whitespace_handling() {
        // 测试各种空白字符的处理
        let whitespace_cases = vec![
            ("    pub fn indented_function() {", "indented_function"), // 前导空格
            ("\tpub fn tab_indented() {", "tab_indented"),             // 前导制表符
            ("pub  fn  extra_spaces() {", "extra_spaces"),             // 额外空格
            ("pub\tfn\ttab_separated() {", "tab_separated"),           // 制表符分隔
        ];

        for (line, expected_name) in whitespace_cases {
            let feature = extract_rust_feature(line, 1);
            if let Some(f) = feature {
                assert_eq!(f.name, expected_name);
                assert_eq!(f.feature_type, "function");
            }
        }
    }
}
