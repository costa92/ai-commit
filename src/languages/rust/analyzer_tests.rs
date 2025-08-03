#[cfg(test)]
mod tests {
    use super::*;
    use crate::languages::{Language, LanguageFeature};

    #[test]
    fn test_rust_analyzer_creation() {
        let analyzer = RustAnalyzer::new();
        assert_eq!(analyzer.language(), Language::Rust);
    }

    #[test]
    fn test_rust_analyzer_default() {
        let analyzer = RustAnalyzer::default();
        assert_eq!(analyzer.language(), Language::Rust);
    }

    #[test]
    fn test_analyze_function_attributes() {
        let analyzer = RustAnalyzer::new();
        
        // 测试 public 函数
        let attrs = analyzer.analyze_function_attributes("pub fn test_function() {");
        assert!(attrs.contains(&"public".to_string()));
        
        // 测试 async 函数
        let attrs = analyzer.analyze_function_attributes("pub async fn async_function() {");
        assert!(attrs.contains(&"public".to_string()));
        assert!(attrs.contains(&"async".to_string()));
        
        // 测试 unsafe 函数
        let attrs = analyzer.analyze_function_attributes("pub unsafe fn unsafe_function() {");
        assert!(attrs.contains(&"public".to_string()));
        assert!(attrs.contains(&"unsafe".to_string()));
        
        // 测试组合属性
        let attrs = analyzer.analyze_function_attributes("pub async unsafe fn complex_function() {");
        assert!(attrs.contains(&"public".to_string()));
        assert!(attrs.contains(&"async".to_string()));
        assert!(attrs.contains(&"unsafe".to_string()));
        
        // 测试 const 函数
        let attrs = analyzer.analyze_function_attributes("const fn const_function() {");
        assert!(attrs.contains(&"const".to_string()));
    }

    #[test]
    fn test_extract_rust_scope_from_path() {
        let analyzer = RustAnalyzer::new();
        
        // 测试主程序文件
        let scopes = analyzer.extract_rust_scope_from_path("src/main.rs");
        assert!(scopes.contains(&"app".to_string()));
        
        // 测试库文件
        let scopes = analyzer.extract_rust_scope_from_path("src/lib.rs");
        assert!(scopes.contains(&"lib".to_string()));
        
        // 测试二进制文件
        let scopes = analyzer.extract_rust_scope_from_path("src/bin/server.rs");
        assert!(scopes.contains(&"bin".to_string()));
        
        // 测试测试文件
        let scopes = analyzer.extract_rust_scope_from_path("tests/integration_test.rs");
        assert!(scopes.contains(&"test".to_string()));
        
        // 测试示例文件
        let scopes = analyzer.extract_rust_scope_from_path("examples/example.rs");
        assert!(scopes.contains(&"example".to_string()));
        
        // 测试基准测试
        let scopes = analyzer.extract_rust_scope_from_path("benches/benchmark.rs");
        assert!(scopes.contains(&"bench".to_string()));
        
        // 测试模块文件
        let scopes = analyzer.extract_rust_scope_from_path("src/auth/mod.rs");
        assert!(scopes.contains(&"auth".to_string()));
        
        // 测试具体模块文件
        let scopes = analyzer.extract_rust_scope_from_path("src/database/connection.rs");
        assert!(scopes.contains(&"connection".to_string()));
        assert!(scopes.contains(&"database".to_string()));
    }

    #[test]
    fn test_analyze_rust_change_patterns() {
        let analyzer = RustAnalyzer::new();
        
        let features = vec![
            LanguageFeature {
                feature_type: "function".to_string(),
                name: "process_data".to_string(),
                line_number: Some(1),
                description: "fn process_data()".to_string(),
            },
            LanguageFeature {
                feature_type: "function".to_string(),
                name: "handle_request".to_string(),
                line_number: Some(2),
                description: "fn handle_request()".to_string(),
            },
            LanguageFeature {
                feature_type: "struct".to_string(),
                name: "User".to_string(),
                line_number: Some(3),
                description: "struct User".to_string(),
            },
            LanguageFeature {
                feature_type: "trait".to_string(),
                name: "Display".to_string(),
                line_number: Some(4),
                description: "trait Display".to_string(),
            },
            LanguageFeature {
                feature_type: "impl".to_string(),
                name: "impl_block".to_string(),
                line_number: Some(5),
                description: "impl User".to_string(),
            },
            LanguageFeature {
                feature_type: "use".to_string(),
                name: "std::collections::HashMap".to_string(),
                line_number: Some(6),
                description: "use std::collections::HashMap".to_string(),
            },
            LanguageFeature {
                feature_type: "use".to_string(),
                name: "serde::Serialize".to_string(),
                line_number: Some(7),
                description: "use serde::Serialize".to_string(),
            },
            LanguageFeature {
                feature_type: "use".to_string(),
                name: "tokio::fs".to_string(),
                line_number: Some(8),
                description: "use tokio::fs".to_string(),
            },
        ];

        let patterns = analyzer.analyze_rust_change_patterns(&features);
        
        assert!(patterns.iter().any(|p| p.contains("新增 2 个函数")));
        assert!(patterns.iter().any(|p| p.contains("新增 1 个结构体")));
        assert!(patterns.iter().any(|p| p.contains("新增 1 个 trait")));
        assert!(patterns.iter().any(|p| p.contains("新增 1 个 impl 块")));
        assert!(patterns.iter().any(|p| p.contains("大量依赖导入变更")));
    }

    #[test]
    fn test_analyze_rust_change_patterns_test_functions() {
        let analyzer = RustAnalyzer::new();
        
        let features = vec![
            LanguageFeature {
                feature_type: "function".to_string(),
                name: "test_user_creation".to_string(),
                line_number: Some(1),
                description: "#[test] fn test_user_creation()".to_string(),
            },
            LanguageFeature {
                feature_type: "function".to_string(),
                name: "test_validation".to_string(),
                line_number: Some(2),
                description: "fn test_validation()".to_string(),
            },
        ];

        let patterns = analyzer.analyze_rust_change_patterns(&features);
        
        assert!(patterns.iter().any(|p| p.contains("新增 2 个测试函数")));
    }

    #[test]
    fn test_analyze_rust_change_patterns_async_functions() {
        let analyzer = RustAnalyzer::new();
        
        let features = vec![
            LanguageFeature {
                feature_type: "function".to_string(),
                name: "async_process".to_string(),
                line_number: Some(1),
                description: "async fn async_process()".to_string(),
            },
            LanguageFeature {
                feature_type: "function".to_string(),
                name: "async_handler".to_string(),
                line_number: Some(2),
                description: "pub async fn async_handler()".to_string(),
            },
        ];

        let patterns = analyzer.analyze_rust_change_patterns(&features);
        
        assert!(patterns.iter().any(|p| p.contains("新增 2 个异步函数")));
    }

    #[test]
    fn test_generate_rust_test_suggestions() {
        let analyzer = RustAnalyzer::new();
        
        let features = vec![
            LanguageFeature {
                feature_type: "function".to_string(),
                name: "calculate".to_string(),
                line_number: Some(1),
                description: "fn calculate()".to_string(),
            },
            LanguageFeature {
                feature_type: "struct".to_string(),
                name: "Calculator".to_string(),
                line_number: Some(2),
                description: "struct Calculator".to_string(),
            },
            LanguageFeature {
                feature_type: "trait".to_string(),
                name: "Processor".to_string(),
                line_number: Some(3),
                description: "trait Processor".to_string(),
            },
            LanguageFeature {
                feature_type: "function".to_string(),
                name: "async_process".to_string(),
                line_number: Some(4),
                description: "async fn async_process()".to_string(),
            },
        ];

        let suggestions = analyzer.generate_rust_test_suggestions(&features);
        
        assert!(suggestions.iter().any(|s| s.contains("为新增函数编写单元测试")));
        assert!(suggestions.iter().any(|s| s.contains("为结构体实现 Debug、Clone 等常用 trait 的测试")));
        assert!(suggestions.iter().any(|s| s.contains("为 trait 实现编写集成测试")));
        assert!(suggestions.iter().any(|s| s.contains("使用 tokio::test 测试异步函数")));
        assert!(suggestions.iter().any(|s| s.contains("运行 cargo test 确保所有测试通过")));
        assert!(suggestions.iter().any(|s| s.contains("使用 cargo clippy 检查代码质量")));
        assert!(suggestions.iter().any(|s| s.contains("运行 cargo fmt 格式化代码")));
    }

    #[test]
    fn test_generate_rust_test_suggestions_unsafe_code() {
        let analyzer = RustAnalyzer::new();
        
        let features = vec![
            LanguageFeature {
                feature_type: "function".to_string(),
                name: "unsafe_operation".to_string(),
                line_number: Some(1),
                description: "unsafe fn unsafe_operation()".to_string(),
            },
        ];

        let suggestions = analyzer.generate_rust_test_suggestions(&features);
        
        assert!(suggestions.iter().any(|s| s.contains("为 unsafe 代码编写额外的安全性测试")));
    }

    #[test]
    fn test_assess_rust_risks() {
        let analyzer = RustAnalyzer::new();
        
        let features = vec![
            LanguageFeature {
                feature_type: "function".to_string(),
                name: "dangerous".to_string(),
                line_number: Some(1),
                description: "unsafe fn dangerous()".to_string(),
            },
            LanguageFeature {
                feature_type: "impl".to_string(),
                name: "impl1".to_string(),
                line_number: Some(2),
                description: "impl Trait1 for Struct1".to_string(),
            },
            LanguageFeature {
                feature_type: "impl".to_string(),
                name: "impl2".to_string(),
                line_number: Some(3),
                description: "impl Trait2 for Struct2".to_string(),
            },
            LanguageFeature {
                feature_type: "impl".to_string(),
                name: "impl3".to_string(),
                line_number: Some(4),
                description: "impl Trait3 for Struct3".to_string(),
            },
            LanguageFeature {
                feature_type: "impl".to_string(),
                name: "impl4".to_string(),
                line_number: Some(5),
                description: "impl Trait4 for Struct4".to_string(),
            },
            LanguageFeature {
                feature_type: "trait".to_string(),
                name: "trait1".to_string(),
                line_number: Some(6),
                description: "trait NewTrait1".to_string(),
            },
            LanguageFeature {
                feature_type: "trait".to_string(),
                name: "trait2".to_string(),
                line_number: Some(7),
                description: "trait NewTrait2".to_string(),
            },
            LanguageFeature {
                feature_type: "trait".to_string(),
                name: "trait3".to_string(),
                line_number: Some(8),
                description: "trait NewTrait3".to_string(),
            },
        ];

        let risks = analyzer.assess_rust_risks(&features);
        
        assert!(risks.iter().any(|r| r.contains("包含 unsafe 代码，需要额外的安全性审查")));
        assert!(risks.iter().any(|r| r.contains("大量 impl 块可能增加代码复杂性")));
        assert!(risks.iter().any(|r| r.contains("多个新 trait 可能影响 API 稳定性")));
    }

    #[test]
    fn test_assess_rust_risks_lifetime_params() {
        let analyzer = RustAnalyzer::new();
        
        let features = vec![
            LanguageFeature {
                feature_type: "function".to_string(),
                name: "with_lifetime".to_string(),
                line_number: Some(1),
                description: "fn with_lifetime<'a>(data: &'a str) -> &'a str".to_string(),
            },
        ];

        let risks = analyzer.assess_rust_risks(&features);
        
        assert!(risks.iter().any(|r| r.contains("涉及生命周期参数，需要仔细检查所有权管理")));
    }

    #[test]
    fn test_assess_rust_risks_generics() {
        let analyzer = RustAnalyzer::new();
        
        let features = vec![
            LanguageFeature {
                feature_type: "struct".to_string(),
                name: "GenericStruct".to_string(),
                line_number: Some(1),
                description: "struct GenericStruct<T: Clone + Debug>".to_string(),
            },
        ];

        let risks = analyzer.assess_rust_risks(&features);
        
        assert!(risks.iter().any(|r| r.contains("使用泛型，需要验证类型约束和编译时检查")));
    }

    #[test]
    fn test_analyze_line_function() {
        let analyzer = RustAnalyzer::new();
        
        let features = analyzer.analyze_line("pub fn test_function() {", 10);
        
        assert_eq!(features.len(), 1);
        assert_eq!(features[0].feature_type, "function");
        assert_eq!(features[0].name, "test_function");
        assert_eq!(features[0].line_number, Some(10));
    }

    #[test]
    fn test_analyze_line_macro() {
        let analyzer = RustAnalyzer::new();
        
        let features = analyzer.analyze_line("macro_rules! my_macro {", 5);
        
        assert_eq!(features.len(), 1);
        assert_eq!(features[0].feature_type, "macro");
        assert_eq!(features[0].name, "macro_definition");
        assert_eq!(features[0].line_number, Some(5));
    }

    #[test]
    fn test_analyze_line_test_function() {
        let analyzer = RustAnalyzer::new();
        
        // 测试 #[test] 属性
        let features = analyzer.analyze_line("#[test]", 1);
        assert_eq!(features.len(), 1);
        assert_eq!(features[0].feature_type, "test");
        
        // 测试 test_ 开头的函数
        let features = analyzer.analyze_line("fn test_something() {", 2);
        assert_eq!(features.len(), 2); // 一个函数特征 + 一个测试特征
        assert!(features.iter().any(|f| f.feature_type == "function"));
        assert!(features.iter().any(|f| f.feature_type == "test"));
    }

    #[test]
    fn test_analyze_line_multiple_features() {
        let analyzer = RustAnalyzer::new();
        
        // 测试可能包含多个特征的行
        let features = analyzer.analyze_line("fn test_function() {", 1);
        
        // 如果函数名包含 test_，应该检测到两个特征
        assert!(features.len() >= 1);
        assert!(features.iter().any(|f| f.feature_type == "function"));
        assert!(features.iter().any(|f| f.feature_type == "test"));
    }

    #[test]
    fn test_analyze_line_no_features() {
        let analyzer = RustAnalyzer::new();
        
        let non_feature_lines = vec![
            "// This is a comment",
            "let x = 5;",
            "println!(\"Hello\");",
            "if condition {",
            "}",
            "",
        ];

        for line in non_feature_lines {
            let features = analyzer.analyze_line(line, 1);
            assert!(features.is_empty(), "Line '{}' should not produce any features", line);
        }
    }

    #[test]
    fn test_extract_scope_suggestions() {
        let analyzer = RustAnalyzer::new();
        
        let test_cases = vec![
            ("src/main.rs", vec!["app"]),
            ("src/lib.rs", vec!["lib"]),
            ("src/auth/mod.rs", vec!["auth"]),
            ("src/database/connection.rs", vec!["connection", "database"]),
            ("tests/integration.rs", vec!["test"]),
            ("examples/simple.rs", vec!["example"]),
            ("benches/performance.rs", vec!["bench"]),
        ];

        for (path, expected_scopes) in test_cases {
            let scopes = analyzer.extract_scope_suggestions(path);
            for expected in expected_scopes {
                assert!(
                    scopes.contains(&expected.to_string()),
                    "Path '{}' should contain scope '{}', got: {:?}",
                    path,
                    expected,
                    scopes
                );
            }
        }
    }

    #[test]
    fn test_analyze_change_patterns() {
        let analyzer = RustAnalyzer::new();
        
        let features = vec![
            LanguageFeature {
                feature_type: "function".to_string(),
                name: "test".to_string(),
                line_number: Some(1),
                description: "test".to_string(),
            },
        ];

        let patterns = analyzer.analyze_change_patterns(&features);
        assert!(!patterns.is_empty());
        assert!(patterns.iter().any(|p| p.contains("新增 1 个函数")));
    }

    #[test]
    fn test_generate_test_suggestions() {
        let analyzer = RustAnalyzer::new();
        
        let features = vec![
            LanguageFeature {
                feature_type: "function".to_string(),
                name: "test".to_string(),
                line_number: Some(1),
                description: "test".to_string(),
            },
        ];

        let suggestions = analyzer.generate_test_suggestions(&features);
        assert!(!suggestions.is_empty());
        assert!(suggestions.iter().any(|s| s.contains("为新增函数编写单元测试")));
    }

    #[test]
    fn test_assess_risks() {
        let analyzer = RustAnalyzer::new();
        
        let features = vec![
            LanguageFeature {
                feature_type: "function".to_string(),
                name: "test".to_string(),
                line_number: Some(1),
                description: "unsafe fn test()".to_string(),
            },
        ];

        let risks = analyzer.assess_risks(&features);
        assert!(!risks.is_empty());
        assert!(risks.iter().any(|r| r.contains("unsafe")));
    }

    #[test]
    fn test_analyze_file_changes() {
        let analyzer = RustAnalyzer::new();
        
        let added_lines = vec![
            "pub fn new_function() {".to_string(),
            "    println!(\"Hello\");".to_string(),
            "}".to_string(),
            "struct NewStruct {".to_string(),
            "    field: i32,".to_string(),
            "}".to_string(),
        ];

        let result = analyzer.analyze_file_changes("src/test.rs", &added_lines);
        
        assert_eq!(result.language, Language::Rust);
        assert!(!result.features.is_empty());
        assert!(!result.scope_suggestions.is_empty());
        assert!(!result.change_patterns.is_empty());
        
        // 检查是否检测到了函数和结构体
        assert!(result.features.iter().any(|f| f.feature_type == "function"));
        assert!(result.features.iter().any(|f| f.feature_type == "struct"));
    }
}