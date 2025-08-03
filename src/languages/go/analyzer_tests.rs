#[cfg(test)]
mod go_analyzer_tests {
    use crate::languages::{Language, LanguageFeature, LanguageAnalyzer};
    use crate::languages::go::GoAnalyzer;

    #[test]
    fn test_go_analyzer_creation() {
        let analyzer = GoAnalyzer::new();
        assert_eq!(analyzer.language(), Language::Go);
    }

    #[test]
    fn test_go_analyzer_default() {
        let analyzer = GoAnalyzer;
        assert_eq!(analyzer.language(), Language::Go);
    }

    #[test]
    fn test_extract_go_scope_from_path() {
        let analyzer = GoAnalyzer::new();
        
        // 测试主程序文件
        let scopes = analyzer.extract_go_scope_from_path("main.go");
        assert!(scopes.contains(&"main".to_string()));
        
        // 测试 cmd 目录
        let scopes = analyzer.extract_go_scope_from_path("cmd/server/main.go");
        assert!(scopes.contains(&"cmd".to_string()));
        
        // 测试 internal 目录
        let scopes = analyzer.extract_go_scope_from_path("internal/auth/service.go");
        assert!(scopes.contains(&"internal".to_string()));
        assert!(scopes.contains(&"auth".to_string()));
        
        // 测试 pkg 目录
        let scopes = analyzer.extract_go_scope_from_path("pkg/utils/helper.go");
        assert!(scopes.contains(&"pkg".to_string()));
        assert!(scopes.contains(&"utils".to_string()));
        
        // 测试测试文件
        let scopes = analyzer.extract_go_scope_from_path("test/integration_test.go");
        assert!(scopes.contains(&"test".to_string()));
        
        let scopes = analyzer.extract_go_scope_from_path("auth_test.go");
        assert!(scopes.contains(&"test".to_string()));
        
        // 测试示例文件
        let scopes = analyzer.extract_go_scope_from_path("examples/simple.go");
        assert!(scopes.contains(&"example".to_string()));
        
        // 测试包目录
        let scopes = analyzer.extract_go_scope_from_path("database/connection.go");
        assert!(scopes.contains(&"database".to_string()));
    }

    #[test]
    fn test_analyze_go_change_patterns() {
        let analyzer = GoAnalyzer::new();
        
        let features = vec![
            LanguageFeature {
                feature_type: "function".to_string(),
                name: "processData".to_string(),
                line_number: Some(1),
                description: "func processData()".to_string(),
            },
            LanguageFeature {
                feature_type: "function".to_string(),
                name: "handleRequest".to_string(),
                line_number: Some(2),
                description: "func handleRequest()".to_string(),
            },
            LanguageFeature {
                feature_type: "method".to_string(),
                name: "Save".to_string(),
                line_number: Some(3),
                description: "func (u *User) Save()".to_string(),
            },
            LanguageFeature {
                feature_type: "struct".to_string(),
                name: "User".to_string(),
                line_number: Some(4),
                description: "type User struct".to_string(),
            },
            LanguageFeature {
                feature_type: "interface".to_string(),
                name: "Writer".to_string(),
                line_number: Some(5),
                description: "type Writer interface".to_string(),
            },
            LanguageFeature {
                feature_type: "import".to_string(),
                name: "fmt".to_string(),
                line_number: Some(6),
                description: "import \"fmt\"".to_string(),
            },
            LanguageFeature {
                feature_type: "import".to_string(),
                name: "net/http".to_string(),
                line_number: Some(7),
                description: "import \"net/http\"".to_string(),
            },
            LanguageFeature {
                feature_type: "import".to_string(),
                name: "encoding/json".to_string(),
                line_number: Some(8),
                description: "import \"encoding/json\"".to_string(),
            },
            LanguageFeature {
                feature_type: "import".to_string(),
                name: "database/sql".to_string(),
                line_number: Some(9),
                description: "import \"database/sql\"".to_string(),
            },
            LanguageFeature {
                feature_type: "package".to_string(),
                name: "main".to_string(),
                line_number: Some(10),
                description: "package main".to_string(),
            },
        ];

        let patterns = analyzer.analyze_go_change_patterns(&features);
        
        assert!(patterns.iter().any(|p| p.contains("新增 2 个函数")));
        assert!(patterns.iter().any(|p| p.contains("新增 1 个方法")));
        assert!(patterns.iter().any(|p| p.contains("新增 1 个结构体")));
        assert!(patterns.iter().any(|p| p.contains("新增 1 个接口")));
        assert!(patterns.iter().any(|p| p.contains("大量依赖导入变更")));
        assert!(patterns.iter().any(|p| p.contains("包声明变更 (1 个)")));
    }

    #[test]
    fn test_analyze_go_change_patterns_test_functions() {
        let analyzer = GoAnalyzer::new();
        
        let features = vec![
            LanguageFeature {
                feature_type: "function".to_string(),
                name: "TestUserCreation".to_string(),
                line_number: Some(1),
                description: "func TestUserCreation(t *testing.T)".to_string(),
            },
            LanguageFeature {
                feature_type: "function".to_string(),
                name: "BenchmarkProcessing".to_string(),
                line_number: Some(2),
                description: "func BenchmarkProcessing(b *testing.B)".to_string(),
            },
            LanguageFeature {
                feature_type: "function".to_string(),
                name: "ExampleUsage".to_string(),
                line_number: Some(3),
                description: "func ExampleUsage()".to_string(),
            },
        ];

        let patterns = analyzer.analyze_go_change_patterns(&features);
        
        assert!(patterns.iter().any(|p| p.contains("新增 3 个测试函数")));
    }

    #[test]
    fn test_analyze_go_change_patterns_concurrency() {
        let analyzer = GoAnalyzer::new();
        
        let features = vec![
            LanguageFeature {
                feature_type: "function".to_string(),
                name: "processAsync".to_string(),
                line_number: Some(1),
                description: "go func() { process() }()".to_string(),
            },
            LanguageFeature {
                feature_type: "function".to_string(),
                name: "handleChannel".to_string(),
                line_number: Some(2),
                description: "ch := make(chan int)".to_string(),
            },
        ];

        let patterns = analyzer.analyze_go_change_patterns(&features);
        
        assert!(patterns.iter().any(|p| p.contains("涉及并发编程变更")));
    }

    #[test]
    fn test_analyze_go_change_patterns_error_handling() {
        let analyzer = GoAnalyzer::new();
        
        let features = vec![
            LanguageFeature {
                feature_type: "function".to_string(),
                name: "processWithError".to_string(),
                line_number: Some(1),
                description: "func processWithError() error".to_string(),
            },
            LanguageFeature {
                feature_type: "struct".to_string(),
                name: "CustomError".to_string(),
                line_number: Some(2),
                description: "type CustomError struct".to_string(),
            },
        ];

        let patterns = analyzer.analyze_go_change_patterns(&features);
        
        assert!(patterns.iter().any(|p| p.contains("错误处理相关变更")));
    }

    #[test]
    fn test_generate_go_test_suggestions() {
        let analyzer = GoAnalyzer::new();
        
        let features = vec![
            LanguageFeature {
                feature_type: "function".to_string(),
                name: "calculate".to_string(),
                line_number: Some(1),
                description: "func calculate()".to_string(),
            },
            LanguageFeature {
                feature_type: "method".to_string(),
                name: "Save".to_string(),
                line_number: Some(2),
                description: "func (u *User) Save()".to_string(),
            },
            LanguageFeature {
                feature_type: "struct".to_string(),
                name: "User".to_string(),
                line_number: Some(3),
                description: "type User struct".to_string(),
            },
            LanguageFeature {
                feature_type: "interface".to_string(),
                name: "Processor".to_string(),
                line_number: Some(4),
                description: "type Processor interface".to_string(),
            },
            LanguageFeature {
                feature_type: "function".to_string(),
                name: "processAsync".to_string(),
                line_number: Some(5),
                description: "go func() { process() }()".to_string(),
            },
        ];

        let suggestions = analyzer.generate_go_test_suggestions(&features);
        
        assert!(suggestions.iter().any(|s| s.contains("为新增函数编写单元测试")));
        assert!(suggestions.iter().any(|s| s.contains("使用表驱动测试模式")));
        assert!(suggestions.iter().any(|s| s.contains("为结构体方法编写测试")));
        assert!(suggestions.iter().any(|s| s.contains("测试结构体的序列化和反序列化")));
        assert!(suggestions.iter().any(|s| s.contains("为接口实现编写集成测试")));
        assert!(suggestions.iter().any(|s| s.contains("编写并发安全测试")));
        assert!(suggestions.iter().any(|s| s.contains("使用 race detector 检测数据竞争")));
        assert!(suggestions.iter().any(|s| s.contains("运行 go test -v 执行详细测试")));
        assert!(suggestions.iter().any(|s| s.contains("使用 go test -race 检测竞态条件")));
        assert!(suggestions.iter().any(|s| s.contains("运行 go test -cover 检查测试覆盖率")));
        assert!(suggestions.iter().any(|s| s.contains("使用 go vet 进行静态分析")));
    }

    #[test]
    fn test_generate_go_test_suggestions_performance_critical() {
        let analyzer = GoAnalyzer::new();
        
        let features = vec![
            LanguageFeature {
                feature_type: "function".to_string(),
                name: "processLargeData".to_string(),
                line_number: Some(1),
                description: "func processLargeData() benchmark".to_string(),
            },
        ];

        let suggestions = analyzer.generate_go_test_suggestions(&features);
        
        assert!(suggestions.iter().any(|s| s.contains("编写基准测试衡量性能")));
    }

    #[test]
    fn test_assess_go_risks() {
        let analyzer = GoAnalyzer::new();
        
        let features = vec![
            LanguageFeature {
                feature_type: "function".to_string(),
                name: "startGoroutine".to_string(),
                line_number: Some(1),
                description: "go func() { work() }()".to_string(),
            },
            LanguageFeature {
                feature_type: "function".to_string(),
                name: "useChannel".to_string(),
                line_number: Some(2),
                description: "ch := make(chan int)".to_string(),
            },
            LanguageFeature {
                feature_type: "interface".to_string(),
                name: "Interface1".to_string(),
                line_number: Some(3),
                description: "type Interface1 interface".to_string(),
            },
            LanguageFeature {
                feature_type: "interface".to_string(),
                name: "Interface2".to_string(),
                line_number: Some(4),
                description: "type Interface2 interface".to_string(),
            },
            LanguageFeature {
                feature_type: "interface".to_string(),
                name: "Interface3".to_string(),
                line_number: Some(5),
                description: "type Interface3 interface".to_string(),
            },
            LanguageFeature {
                feature_type: "struct".to_string(),
                name: "Struct1".to_string(),
                line_number: Some(6),
                description: "type Struct1 struct".to_string(),
            },
            LanguageFeature {
                feature_type: "struct".to_string(),
                name: "Struct2".to_string(),
                line_number: Some(7),
                description: "type Struct2 struct".to_string(),
            },
            LanguageFeature {
                feature_type: "struct".to_string(),
                name: "Struct3".to_string(),
                line_number: Some(8),
                description: "type Struct3 struct".to_string(),
            },
            LanguageFeature {
                feature_type: "struct".to_string(),
                name: "Struct4".to_string(),
                line_number: Some(9),
                description: "type Struct4 struct".to_string(),
            },
        ];

        let risks = analyzer.assess_go_risks(&features);
        
        assert!(risks.iter().any(|r| r.contains("使用 goroutine，需要检查是否存在泄漏风险")));
        assert!(risks.iter().any(|r| r.contains("使用 channel，需要验证是否存在死锁风险")));
        assert!(risks.iter().any(|r| r.contains("多个新接口可能影响 API 兼容性")));
        assert!(risks.iter().any(|r| r.contains("大量结构体变更可能增加内存使用")));
    }

    #[test]
    fn test_assess_go_risks_pointers() {
        let analyzer = GoAnalyzer::new();
        
        let features = vec![
            LanguageFeature {
                feature_type: "function".to_string(),
                name: "usePointer".to_string(),
                line_number: Some(1),
                description: "func usePointer(p *int)".to_string(),
            },
        ];

        let risks = analyzer.assess_go_risks(&features);
        
        assert!(risks.iter().any(|r| r.contains("使用指针，需要注意 nil 指针引用")));
    }

    #[test]
    fn test_assess_go_risks_reflection() {
        let analyzer = GoAnalyzer::new();
        
        let features = vec![
            LanguageFeature {
                feature_type: "function".to_string(),
                name: "useReflection".to_string(),
                line_number: Some(1),
                description: "reflect.ValueOf(obj)".to_string(),
            },
        ];

        let risks = analyzer.assess_go_risks(&features);
        
        assert!(risks.iter().any(|r| r.contains("使用反射，可能影响性能和类型安全")));
    }

    #[test]
    fn test_assess_go_risks_unsafe() {
        let analyzer = GoAnalyzer::new();
        
        let features = vec![
            LanguageFeature {
                feature_type: "function".to_string(),
                name: "useUnsafe".to_string(),
                line_number: Some(1),
                description: "unsafe.Pointer(ptr)".to_string(),
            },
        ];

        let risks = analyzer.assess_go_risks(&features);
        
        assert!(risks.iter().any(|r| r.contains("使用 unsafe 包，需要额外的安全性审查")));
    }

    #[test]
    fn test_analyze_line_function() {
        let analyzer = GoAnalyzer::new();
        
        let features = analyzer.analyze_line("func testFunction() {", 10);
        
        assert_eq!(features.len(), 1);
        assert_eq!(features[0].feature_type, "function");
        assert_eq!(features[0].name, "testFunction");
        assert_eq!(features[0].line_number, Some(10));
    }

    #[test]
    fn test_analyze_line_test_function() {
        let analyzer = GoAnalyzer::new();
        
        let features = analyzer.analyze_line("func TestSomething(t *testing.T) {", 5);
        
        assert_eq!(features.len(), 2); // function + test
        assert!(features.iter().any(|f| f.feature_type == "function"));
        assert!(features.iter().any(|f| f.feature_type == "test"));
    }

    #[test]
    fn test_analyze_line_benchmark_function() {
        let analyzer = GoAnalyzer::new();
        
        let features = analyzer.analyze_line("func BenchmarkProcess(b *testing.B) {", 3);
        
        assert_eq!(features.len(), 2); // function + test
        assert!(features.iter().any(|f| f.feature_type == "function"));
        assert!(features.iter().any(|f| f.feature_type == "test"));
    }

    #[test]
    fn test_analyze_line_example_function() {
        let analyzer = GoAnalyzer::new();
        
        let features = analyzer.analyze_line("func ExampleUsage() {", 2);
        
        assert_eq!(features.len(), 2); // function + test
        assert!(features.iter().any(|f| f.feature_type == "function"));
        assert!(features.iter().any(|f| f.feature_type == "test"));
    }

    #[test]
    fn test_analyze_line_goroutine() {
        let analyzer = GoAnalyzer::new();
        
        let features = analyzer.analyze_line("go func() { process() }()", 1);
        
        assert!(features.iter().any(|f| f.feature_type == "goroutine"));
        assert_eq!(features.iter().find(|f| f.feature_type == "goroutine").unwrap().name, "goroutine_start");
    }

    #[test]
    fn test_analyze_line_channel() {
        let analyzer = GoAnalyzer::new();
        
        // 测试 make(chan...)
        let features = analyzer.analyze_line("ch := make(chan int, 10)", 1);
        assert!(features.iter().any(|f| f.feature_type == "channel"));
        
        // 测试 chan 类型声明
        let features = analyzer.analyze_line("var ch chan string", 2);
        assert!(features.iter().any(|f| f.feature_type == "channel"));
    }

    #[test]
    fn test_analyze_line_no_features() {
        let analyzer = GoAnalyzer::new();
        
        let non_feature_lines = vec![
            "// This is a comment",
            "x := 5",
            "fmt.Println(\"Hello\")",
            "if err != nil {",
            "}",
            "",
            "return nil",
        ];

        for line in non_feature_lines {
            let features = analyzer.analyze_line(line, 1);
            assert!(features.is_empty(), "Line '{}' should not produce any features", line);
        }
    }

    #[test]
    fn test_extract_scope_suggestions() {
        let analyzer = GoAnalyzer::new();
        
        let test_cases = vec![
            ("main.go", vec!["main"]),
            ("cmd/server/main.go", vec!["cmd", "server"]),
            ("internal/auth/service.go", vec!["internal", "auth"]),
            ("pkg/utils/helper.go", vec!["pkg", "utils"]),
            ("database/connection.go", vec!["database"]),
            ("test/integration_test.go", vec!["test"]),
            ("examples/simple.go", vec!["example"]),
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
        let analyzer = GoAnalyzer::new();
        
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
        let analyzer = GoAnalyzer::new();
        
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
        let analyzer = GoAnalyzer::new();
        
        let features = vec![
            LanguageFeature {
                feature_type: "function".to_string(),
                name: "test".to_string(),
                line_number: Some(1),
                description: "go func() {}()".to_string(),
            },
        ];

        let risks = analyzer.assess_risks(&features);
        assert!(!risks.is_empty());
        assert!(risks.iter().any(|r| r.contains("goroutine")));
    }

    #[test]
    fn test_analyze_file_changes() {
        let analyzer = GoAnalyzer::new();
        
        let added_lines = vec![
            "package main".to_string(),
            "func newFunction() {".to_string(),
            "    fmt.Println(\"Hello\")".to_string(),
            "}".to_string(),
            "type NewStruct struct {".to_string(),
            "    Field int".to_string(),
            "}".to_string(),
        ];

        let result = analyzer.analyze_file_changes("main.go", &added_lines);
        
        assert_eq!(result.language, Language::Go);
        assert!(!result.features.is_empty());
        assert!(!result.scope_suggestions.is_empty());
        assert!(!result.change_patterns.is_empty());
        
        // 检查是否检测到了包、函数和结构体
        assert!(result.features.iter().any(|f| f.feature_type == "package"));
        assert!(result.features.iter().any(|f| f.feature_type == "function"));
        assert!(result.features.iter().any(|f| f.feature_type == "struct"));
    }
}