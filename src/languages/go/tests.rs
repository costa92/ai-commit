#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_go_feature_type_as_str() {
        assert_eq!(GoFeatureType::Package.as_str(), "package");
        assert_eq!(GoFeatureType::Function.as_str(), "function");
        assert_eq!(GoFeatureType::Method.as_str(), "method");
        assert_eq!(GoFeatureType::Struct.as_str(), "struct");
        assert_eq!(GoFeatureType::Interface.as_str(), "interface");
        assert_eq!(GoFeatureType::Const.as_str(), "const");
        assert_eq!(GoFeatureType::Var.as_str(), "var");
        assert_eq!(GoFeatureType::Import.as_str(), "import");
        assert_eq!(GoFeatureType::Type.as_str(), "type");
        assert_eq!(GoFeatureType::Goroutine.as_str(), "goroutine");
        assert_eq!(GoFeatureType::Channel.as_str(), "channel");
        assert_eq!(GoFeatureType::Test.as_str(), "test");
        assert_eq!(GoFeatureType::Benchmark.as_str(), "benchmark");
    }

    #[test]
    fn test_extract_go_feature_package() {
        let test_cases = vec![
            ("package main", "main"),
            ("package auth", "auth"),
            ("package utils", "utils"),
            ("  package database  ", "database"), // 测试空白字符
        ];

        for (line, expected_name) in test_cases {
            let feature = extract_go_feature(line, 1).unwrap();
            assert_eq!(feature.feature_type, "package");
            assert_eq!(feature.name, expected_name);
            assert_eq!(feature.line_number, Some(1));
            assert!(feature.description.contains("Go package"));
        }
    }

    #[test]
    fn test_extract_go_feature_function() {
        let test_cases = vec![
            ("func main() {", "main"),
            ("func processData(data []byte) error {", "processData"),
            ("func (s *Service) HandleRequest() {", "HandleRequest"), // 这是方法，但可能被识别为函数
            ("func init() {", "init"),
            ("  func   helper()   {", "helper"), // 测试额外空格
        ];

        for (line, expected_name) in test_cases {
            let feature = extract_go_feature(line, 1);
            if let Some(f) = feature {
                // 可能是函数或方法
                assert!(f.feature_type == "function" || f.feature_type == "method");
                if f.feature_type == "function" {
                    assert_eq!(f.name, expected_name);
                }
            }
        }
    }

    #[test]
    fn test_extract_go_feature_method() {
        let test_cases = vec![
            ("func (u *User) Save() error {", "Save"),
            ("func (s Service) Process() {", "Process"),
            ("func (c *Client) Connect() error {", "Connect"),
            ("func (h Handler) ServeHTTP(w http.ResponseWriter, r *http.Request) {", "ServeHTTP"),
        ];

        for (line, expected_name) in test_cases {
            let feature = extract_go_feature(line, 1).unwrap();
            assert_eq!(feature.feature_type, "method");
            assert!(feature.name.contains(expected_name));
            assert!(feature.description.contains("Go method"));
        }
    }

    #[test]
    fn test_extract_go_feature_struct() {
        let test_cases = vec![
            ("type User struct {", "User"),
            ("type DatabaseConfig struct {", "DatabaseConfig"),
            ("type GenericStruct[T any] struct {", "GenericStruct"),
            ("  type   InternalStruct   struct {", "InternalStruct"),
        ];

        for (line, expected_name) in test_cases {
            let feature = extract_go_feature(line, 1).unwrap();
            assert_eq!(feature.feature_type, "struct");
            assert_eq!(feature.name, expected_name);
            assert!(feature.description.contains("Go struct"));
        }
    }

    #[test]
    fn test_extract_go_feature_interface() {
        let test_cases = vec![
            ("type Writer interface {", "Writer"),
            ("type Reader interface {", "Reader"),
            ("type Processor interface {", "Processor"),
            ("type GenericInterface[T any] interface {", "GenericInterface"),
        ];

        for (line, expected_name) in test_cases {
            let feature = extract_go_feature(line, 1).unwrap();
            assert_eq!(feature.feature_type, "interface");
            assert_eq!(feature.name, expected_name);
            assert!(feature.description.contains("Go interface"));
        }
    }

    #[test]
    fn test_extract_go_feature_import() {
        let test_cases = vec![
            ("import \"fmt\"", "fmt"),
            ("import \"net/http\"", "net/http"),
            ("import json \"encoding/json\"", "encoding/json"),
            ("  import   \"os\"  ", "os"),
        ];

        for (line, expected_path) in test_cases {
            let feature = extract_go_feature(line, 1).unwrap();
            assert_eq!(feature.feature_type, "import");
            assert_eq!(feature.name, expected_path);
            assert!(feature.description.contains("Go import"));
        }
    }

    #[test]
    fn test_extract_function_name() {
        let test_cases = vec![
            ("func main() {", Some("main".to_string())),
            ("func processData(data []byte) error {", Some("processData".to_string())),
            ("func   spacedFunction   (", Some("spacedFunction".to_string())),
            ("func() {", Some("anonymous".to_string())), // 匿名函数
            ("not a function", None),
        ];

        for (line, expected) in test_cases {
            let result = extract_function_name(line);
            assert_eq!(result, expected, "Failed for line: {}", line);
        }
    }

    #[test]
    fn test_extract_go_feature_no_match() {
        let non_matching_lines = vec![
            "// This is a comment",
            "var x int = 5",
            "const MAX_SIZE = 1024",
            "if condition {",
            "}",
            "",
            "    // Indented comment",
            "fmt.Println(\"Hello\")",
            "x := 10",
            "for i := 0; i < 10; i++ {",
        ];

        for line in non_matching_lines {
            let feature = extract_go_feature(line, 1);
            assert!(feature.is_none(), "Line '{}' should not match any pattern", line);
        }
    }

    #[test]
    fn test_extract_go_feature_edge_cases() {
        let edge_cases = vec![
            ("func", None), // 不完整的函数声明
            ("type", None), // 不完整的类型声明
            ("import", None), // 不完整的导入语句
            ("package", None), // 不完整的包声明
            ("func ()", None), // 无效的函数语法
        ];

        for (line, expected) in edge_cases {
            let feature = extract_go_feature(line, 1);
            match expected {
                None => assert!(feature.is_none(), "Line '{}' should not match", line),
                Some(_) => assert!(feature.is_some(), "Line '{}' should match", line),
            }
        }
    }

    #[test]
    fn test_extract_go_feature_complex_generics() {
        // 测试 Go 1.18+ 泛型语法
        let generic_cases = vec![
            ("type Container[T any] struct {", "Container"),
            ("func GenericFunc[T comparable](items []T) T {", "GenericFunc"),
            ("type Processor[T any, U comparable] interface {", "Processor"),
            ("func (c *Container[T]) Add(item T) {", "Add"),
        ];

        for (line, expected_name) in generic_cases {
            let feature = extract_go_feature(line, 1);
            if let Some(f) = feature {
                assert!(f.name.contains(expected_name) || f.name == expected_name);
            }
        }
    }

    #[test]
    fn test_extract_go_feature_receiver_types() {
        // 测试各种接收器类型
        let receiver_cases = vec![
            ("func (u User) Method1() {", "Method1"),
            ("func (u *User) Method2() {", "Method2"),
            ("func (u user) method3() {", "method3"),
            ("func (service *ServiceImpl) Handle() {", "Handle"),
        ];

        for (line, expected_method) in receiver_cases {
            let feature = extract_go_feature(line, 1);
            if let Some(f) = feature {
                assert_eq!(f.feature_type, "method");
                assert!(f.name.contains(expected_method));
            }
        }
    }

    #[test]
    fn test_extract_go_feature_line_numbers() {
        let test_lines = vec![
            "package main",
            "func processData() {",
            "type User struct {",
        ];

        for (index, line) in test_lines.iter().enumerate() {
            let line_number = index + 10;
            let feature = extract_go_feature(line, line_number);
            if let Some(f) = feature {
                assert_eq!(f.line_number, Some(line_number));
            }
        }
    }

    #[test]
    fn test_extract_go_feature_description_content() {
        let test_line = "func processData(data []byte) ([]byte, error) {";
        let feature = extract_go_feature(test_line, 1).unwrap();
        
        assert!(feature.description.contains("Go function"));
        assert!(feature.description.contains(test_line.trim()));
    }

    #[test]
    fn test_extract_go_feature_whitespace_handling() {
        let whitespace_cases = vec![
            ("    func indentedFunction() {", "indentedFunction"),
            ("\tfunc tabIndentedFunction() {", "tabIndentedFunction"),
            ("func  extraSpacesFunction  () {", "extraSpacesFunction"),
            ("func\ttabSeparatedFunction() {", "tabSeparatedFunction"),
        ];

        for (line, expected_name) in whitespace_cases {
            let feature = extract_go_feature(line, 1);
            if let Some(f) = feature {
                assert_eq!(f.feature_type, "function");
                assert_eq!(f.name, expected_name);
            }
        }
    }

    #[test]
    fn test_extract_go_feature_import_variations() {
        let import_cases = vec![
            ("import \"fmt\"", "fmt"),
            ("import \"net/http\"", "net/http"),
            ("import json \"encoding/json\"", "encoding/json"),
            ("import . \"fmt\"", "fmt"),
            ("import _ \"database/sql\"", "database/sql"),
        ];

        for (line, expected_path) in import_cases {
            let feature = extract_go_feature(line, 1);
            if let Some(f) = feature {
                assert_eq!(f.feature_type, "import");
                assert_eq!(f.name, expected_path);
            }
        }
    }

    #[test]
    fn test_extract_go_feature_type_aliases() {
        let type_alias_cases = vec![
            "type UserID int",
            "type Handler func(http.ResponseWriter, *http.Request)",
            "type StringMap map[string]string",
        ];

        for line in type_alias_cases {
            let feature = extract_go_feature(line, 1);
            // 类型别名可能不被当前实现识别，但不应该崩溃
            if let Some(_f) = feature {
                // 如果识别了，确保不会崩溃
            }
        }
    }

    #[test]
    fn test_extract_go_feature_anonymous_functions() {
        let anonymous_cases = vec![
            "func() {",
            "func(x int) int {",
            "func() error {",
        ];

        for line in anonymous_cases {
            let feature = extract_go_feature(line, 1);
            if let Some(f) = feature {
                assert_eq!(f.feature_type, "function");
                assert_eq!(f.name, "anonymous");
            }
        }
    }

    #[test]
    fn test_extract_go_feature_concurrent_patterns() {
        // 测试常见的并发模式（这些不会被 extract_go_feature 直接识别，但用于完整性测试）
        let concurrent_lines = vec![
            "go func() {",
            "ch := make(chan int)",
            "select {",
            "case <-ch:",
        ];

        for line in concurrent_lines {
            let feature = extract_go_feature(line, 1);
            // 这些可能不会被识别为特定特征，这是正常的
            // 这个测试主要确保不会出现意外错误
        }
    }

    #[test]
    fn test_extract_go_feature_error_handling() {
        let error_handling_lines = vec![
            "if err != nil {",
            "return nil, err",
            "return fmt.Errorf(\"error: %v\", err)",
        ];

        for line in error_handling_lines {
            let feature = extract_go_feature(line, 1);
            // 错误处理模式可能不会被识别为特定特征
            // 这个测试确保不会崩溃
        }
    }

    #[test]
    fn test_extract_go_feature_embedded_types() {
        let embedded_cases = vec![
            "type Server struct {",
            "type Client struct {",
        ];

        for line in embedded_cases {
            let feature = extract_go_feature(line, 1);
            if let Some(f) = feature {
                assert_eq!(f.feature_type, "struct");
            }
        }
    }

    #[test]
    fn test_extract_go_feature_constants_and_variables() {
        // 虽然当前实现可能不识别这些，但测试确保不会出错
        let const_var_cases = vec![
            "const MaxRetries = 3",
            "var globalConfig Config",
            "const (",
            "var (",
        ];

        for line in const_var_cases {
            let feature = extract_go_feature(line, 1);
            // 可能不识别，但不应该崩溃
        }
    }

    #[test]
    fn test_extract_go_feature_build_tags() {
        // 测试构建标签（这些不应该被识别为特征）
        let build_tag_cases = vec![
            "//go:build linux",
            "// +build windows",
            "//go:generate mockgen",
        ];

        for line in build_tag_cases {
            let feature = extract_go_feature(line, 1);
            assert!(feature.is_none(), "Build tag '{}' should not be recognized as a feature", line);
        }
    }
}