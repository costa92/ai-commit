use crate::languages::{LanguageAnalyzer, LanguageFeature};
use regex::Regex;
use std::collections::HashSet;

/// Go 语言分析器
pub struct GoAnalyzer {
    patterns: GoPatterns,
}

impl GoAnalyzer {
    pub fn new() -> Self {
        Self {
            patterns: GoPatterns::new(),
        }
    }

    /// 提取包声明
    fn extract_package_declaration(&self, content: &str) -> Option<LanguageFeature> {
        if let Some(captures) = self.patterns.package_regex.captures(content) {
            if let Some(package_name) = captures.get(1) {
                return Some(LanguageFeature::Package(package_name.as_str().to_string()));
            }
        }
        None
    }

    /// 提取函数定义
    fn extract_functions(&self, content: &str) -> Vec<LanguageFeature> {
        let mut functions = Vec::new();

        for captures in self.patterns.function_regex.captures_iter(content) {
            if let Some(func_name) = captures.get(1) {
                functions.push(LanguageFeature::Function(func_name.as_str().to_string()));
            }
        }

        functions
    }

    /// 提取结构体定义
    fn extract_structs(&self, content: &str) -> Vec<LanguageFeature> {
        let mut structs = Vec::new();

        for captures in self.patterns.struct_regex.captures_iter(content) {
            if let Some(struct_name) = captures.get(1) {
                structs.push(LanguageFeature::Struct(struct_name.as_str().to_string()));
            }
        }

        structs
    }

    /// 提取接口定义
    fn extract_interfaces(&self, content: &str) -> Vec<LanguageFeature> {
        let mut interfaces = Vec::new();

        for captures in self.patterns.interface_regex.captures_iter(content) {
            if let Some(interface_name) = captures.get(1) {
                interfaces.push(LanguageFeature::Interface(interface_name.as_str().to_string()));
            }
        }

        interfaces
    }

    /// 检测 Go 特定的代码模式
    pub fn detect_go_patterns(&self, content: &str) -> GoCodePatterns {
        let mut patterns = GoCodePatterns::default();

        // 检测 goroutine 使用
        patterns.has_goroutines = self.patterns.goroutine_regex.is_match(content);

        // 检测 channel 使用
        patterns.has_channels = self.patterns.channel_regex.is_match(content);

        // 检测 defer 语句
        patterns.has_defer = self.patterns.defer_regex.is_match(content);

        // 检测 select 语句
        patterns.has_select = self.patterns.select_regex.is_match(content);

        // 检测错误处理模式
        patterns.has_error_handling = self.patterns.error_handling_regex.is_match(content);

        // 检测接口实现
        patterns.has_interface_implementation = self.patterns.interface_impl_regex.is_match(content);

        // 检测嵌入结构体
        patterns.has_embedded_structs = self.patterns.embedded_struct_regex.is_match(content);

        // 检测方法定义
        patterns.has_methods = self.patterns.method_regex.is_match(content);

        // 检测类型断言
        patterns.has_type_assertions = self.patterns.type_assertion_regex.is_match(content);

        // 检测反射使用
        patterns.has_reflection = self.patterns.reflection_regex.is_match(content);

        patterns
    }

    /// 分析导入的包
    pub fn analyze_imports(&self, content: &str) -> Vec<String> {
        let mut imports = Vec::new();

        // 单行导入
        for captures in self.patterns.single_import_regex.captures_iter(content) {
            if let Some(import_path) = captures.get(1) {
                imports.push(import_path.as_str().trim_matches('"').to_string());
            }
        }

        // 多行导入块
        if let Some(captures) = self.patterns.multi_import_regex.captures(content) {
            if let Some(import_block) = captures.get(1) {
                for line in import_block.as_str().lines() {
                    let line = line.trim();
                    if !line.is_empty() && !line.starts_with("//") {
                        // 提取导入路径
                        if let Some(path_captures) = self.patterns.import_path_regex.captures(line) {
                            if let Some(path) = path_captures.get(1) {
                                imports.push(path.as_str().trim_matches('"').to_string());
                            }
                        }
                    }
                }
            }
        }

        // 去重并排序
        let mut unique_imports: Vec<String> = imports.into_iter().collect::<HashSet<_>>().into_iter().collect();
        unique_imports.sort();
        unique_imports
    }

    /// 分析函数复杂度（简单版本）
    pub fn analyze_function_complexity(&self, content: &str) -> Vec<FunctionComplexity> {
        let mut complexities = Vec::new();

        for captures in self.patterns.function_regex.captures_iter(content) {
            if let Some(func_name) = captures.get(1) {
                let func_name = func_name.as_str().to_string();
                let func_start = captures.get(0).unwrap().end();

                // 找到函数体（从函数签名后的第一个 { 开始）
                if let Some(function_body) = self.extract_function_body(content, func_start) {
                    let complexity = self.calculate_simple_complexity(&function_body);
                    let line_count = function_body.lines().count();

                    complexities.push(FunctionComplexity {
                        name: func_name,
                        cyclomatic_complexity: complexity,
                        line_count,
                    });
                }
            }
        }

        complexities
    }

    /// 提取函数体内容
    fn extract_function_body(&self, content: &str, start_pos: usize) -> Option<String> {
        let remaining_content = &content[start_pos..];

        // 找到第一个 {
        if let Some(brace_start) = remaining_content.find('{') {
            let mut brace_count = 0;
            let mut body_end = brace_start;
            let chars: Vec<char> = remaining_content.chars().collect();

            for (i, &ch) in chars.iter().enumerate().skip(brace_start) {
                match ch {
                    '{' => brace_count += 1,
                    '}' => {
                        brace_count -= 1;
                        if brace_count == 0 {
                            body_end = i + 1;
                            break;
                        }
                    }
                    _ => {}
                }
            }

            if brace_count == 0 {
                let body: String = chars[brace_start..body_end].iter().collect();
                return Some(body);
            }
        }

        None
    }

    fn calculate_simple_complexity(&self, function_body: &str) -> u32 {
        let mut complexity = 1; // 基础复杂度

        // 计算控制流语句
        complexity += self.patterns.if_regex.find_iter(function_body).count() as u32;
        complexity += self.patterns.for_regex.find_iter(function_body).count() as u32;
        complexity += self.patterns.switch_regex.find_iter(function_body).count() as u32;
        complexity += self.patterns.case_regex.find_iter(function_body).count() as u32;
        complexity += self.patterns.select_regex.find_iter(function_body).count() as u32;

        complexity
    }
}

impl LanguageAnalyzer for GoAnalyzer {
    fn analyze_features(&self, content: &str) -> Vec<LanguageFeature> {
        let mut features = Vec::new();

        // 提取包声明
        if let Some(package) = self.extract_package_declaration(content) {
            features.push(package);
        }

        // 提取函数
        features.extend(self.extract_functions(content));

        // 提取结构体
        features.extend(self.extract_structs(content));

        // 提取接口
        features.extend(self.extract_interfaces(content));

        features
    }
}

/// Go 语言正则表达式模式
struct GoPatterns {
    package_regex: Regex,
    function_regex: Regex,
    struct_regex: Regex,
    interface_regex: Regex,
    method_regex: Regex,
    goroutine_regex: Regex,
    channel_regex: Regex,
    defer_regex: Regex,
    select_regex: Regex,
    error_handling_regex: Regex,
    interface_impl_regex: Regex,
    embedded_struct_regex: Regex,
    type_assertion_regex: Regex,
    reflection_regex: Regex,
    single_import_regex: Regex,
    multi_import_regex: Regex,
    import_path_regex: Regex,
    if_regex: Regex,
    for_regex: Regex,
    switch_regex: Regex,
    case_regex: Regex,
}

impl GoPatterns {
    fn new() -> Self {
        Self {
            package_regex: Regex::new(r"(?m)^package\s+(\w+)").unwrap(),
            function_regex: Regex::new(r"(?m)func\s+(?:\([^)]*\)\s+)?(\w+)\s*\([^)]*\)").unwrap(),
            struct_regex: Regex::new(r"(?m)^type\s+(\w+)\s+struct\s*\{").unwrap(),
            interface_regex: Regex::new(r"(?m)^type\s+(\w+)\s+interface\s*\{").unwrap(),
            method_regex: Regex::new(r"(?m)^func\s+\([^)]+\)\s+(\w+)\s*\(").unwrap(),
            goroutine_regex: Regex::new(r"\bgo\s+\w+\s*\(").unwrap(),
            channel_regex: Regex::new(r"\bchan\s+\w+|\bmake\s*\(\s*chan\s+\w+").unwrap(),
            defer_regex: Regex::new(r"\bdefer\s+").unwrap(),
            select_regex: Regex::new(r"\bselect\s*\{").unwrap(),
            error_handling_regex: Regex::new(r"err\s*!=\s*nil|return\s+.*,\s*err").unwrap(),
            interface_impl_regex: Regex::new(r"func\s+\([^)]+\)\s+\w+\s*\([^)]*\)").unwrap(),
            embedded_struct_regex: Regex::new(r"(?m)^\s*\w+\s*$").unwrap(),
            type_assertion_regex: Regex::new(r"\.\([^)]+\)").unwrap(),
            reflection_regex: Regex::new(r"\breflect\.\w+").unwrap(),
            single_import_regex: Regex::new(r#"(?m)^import\s+"([^"]+)""#).unwrap(),
            multi_import_regex: Regex::new(r"(?s)import\s*\(\s*(.*?)\s*\)").unwrap(),
            import_path_regex: Regex::new(r#""([^"]+)""#).unwrap(),
            if_regex: Regex::new(r"\bif\s+").unwrap(),
            for_regex: Regex::new(r"\bfor\s+").unwrap(),
            switch_regex: Regex::new(r"\bswitch\s+").unwrap(),
            case_regex: Regex::new(r"\bcase\s+").unwrap(),
        }
    }
}

/// Go 代码模式检测结果
#[derive(Debug, Default, Clone)]
pub struct GoCodePatterns {
    pub has_goroutines: bool,
    pub has_channels: bool,
    pub has_defer: bool,
    pub has_select: bool,
    pub has_error_handling: bool,
    pub has_interface_implementation: bool,
    pub has_embedded_structs: bool,
    pub has_methods: bool,
    pub has_type_assertions: bool,
    pub has_reflection: bool,
}

/// 函数复杂度信息
#[derive(Debug, Clone)]
pub struct FunctionComplexity {
    pub name: String,
    pub cyclomatic_complexity: u32,
    pub line_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_go_analyzer_creation() {
        let analyzer = GoAnalyzer::new();
        // 验证分析器创建成功
        assert!(analyzer.patterns.package_regex.is_match("package main"));
    }

    #[test]
    fn test_package_extraction() {
        let analyzer = GoAnalyzer::new();
        let code = "package main\n\nfunc main() {}";

        if let Some(LanguageFeature::Package(name)) = analyzer.extract_package_declaration(code) {
            assert_eq!(name, "main");
        } else {
            panic!("Expected package declaration");
        }
    }

    #[test]
    fn test_function_extraction() {
        let analyzer = GoAnalyzer::new();
        let code = r#"
package main

func main() {
    fmt.Println("Hello, World!")
}

func helper(x int) string {
    return fmt.Sprintf("%d", x)
}
"#;

        let functions = analyzer.extract_functions(code);
        assert_eq!(functions.len(), 2);

        if let LanguageFeature::Function(name) = &functions[0] {
            assert_eq!(name, "main");
        }

        if let LanguageFeature::Function(name) = &functions[1] {
            assert_eq!(name, "helper");
        }
    }

    #[test]
    fn test_struct_extraction() {
        let analyzer = GoAnalyzer::new();
        let code = r#"
type User struct {
    Name string
    Age  int
}

type Config struct {
    Host string
    Port int
}
"#;

        let structs = analyzer.extract_structs(code);
        assert_eq!(structs.len(), 2);

        if let LanguageFeature::Struct(name) = &structs[0] {
            assert_eq!(name, "User");
        }

        if let LanguageFeature::Struct(name) = &structs[1] {
            assert_eq!(name, "Config");
        }
    }

    #[test]
    fn test_interface_extraction() {
        let analyzer = GoAnalyzer::new();
        let code = r#"
type Reader interface {
    Read([]byte) (int, error)
}

type Writer interface {
    Write([]byte) (int, error)
}
"#;

        let interfaces = analyzer.extract_interfaces(code);
        assert_eq!(interfaces.len(), 2);

        if let LanguageFeature::Interface(name) = &interfaces[0] {
            assert_eq!(name, "Reader");
        }

        if let LanguageFeature::Interface(name) = &interfaces[1] {
            assert_eq!(name, "Writer");
        }
    }

    #[test]
    fn test_go_pattern_detection() {
        let analyzer = GoAnalyzer::new();
        let code = r#"
package main

import (
    "fmt"
    "sync"
)

func main() {
    ch := make(chan int)

    go func() {
        defer close(ch)
        ch <- 42
    }()

    select {
    case value := <-ch:
        fmt.Println(value)
    default:
        fmt.Println("No value")
    }

    if err := doSomething(); err != nil {
        return
    }
}

func doSomething() error {
    return nil
}
"#;

        let patterns = analyzer.detect_go_patterns(code);

        assert!(patterns.has_goroutines);
        assert!(patterns.has_channels);
        assert!(patterns.has_defer);
        assert!(patterns.has_select);
        assert!(patterns.has_error_handling);
    }

    #[test]
    fn test_import_analysis() {
        let analyzer = GoAnalyzer::new();
        let code = r#"
package main

import (
    "fmt"
    "os"
    "github.com/user/repo"
)

import "strings"
"#;

        let imports = analyzer.analyze_imports(code);
        assert!(imports.contains(&"fmt".to_string()));
        assert!(imports.contains(&"os".to_string()));
        assert!(imports.contains(&"github.com/user/repo".to_string()));
        assert!(imports.contains(&"strings".to_string()));
    }

    #[test]
    fn test_function_complexity() {
        let analyzer = GoAnalyzer::new();
        let code = r#"
func complexFunction(x int) int {
    if x > 0 {
        for i := 0; i < x; i++ {
            switch i {
            case 1:
                return 1
            case 2:
                return 2
            default:
                continue
            }
        }
    }
    return 0
}
"#;

        let complexities = analyzer.analyze_function_complexity(code);
        assert_eq!(complexities.len(), 1);

        let complexity = &complexities[0];
        assert_eq!(complexity.name, "complexFunction");
        assert!(complexity.cyclomatic_complexity > 1);
    }

    #[test]
    fn test_language_analyzer_trait() {
        let analyzer = GoAnalyzer::new();
        let code = r#"
package main

type User struct {
    Name string
}

type Reader interface {
    Read() string
}

func main() {
    fmt.Println("Hello")
}

func helper() {
    // helper function
}
"#;

        let features = analyzer.analyze_features(code);

        // 应该包含：1个包，2个函数，1个结构体，1个接口
        assert!(features.len() >= 4);

        // 验证包含正确的特征类型
        let has_package = features.iter().any(|f| matches!(f, LanguageFeature::Package(_)));
        let has_function = features.iter().any(|f| matches!(f, LanguageFeature::Function(_)));
        let has_struct = features.iter().any(|f| matches!(f, LanguageFeature::Struct(_)));
        let has_interface = features.iter().any(|f| matches!(f, LanguageFeature::Interface(_)));

        assert!(has_package);
        assert!(has_function);
        assert!(has_struct);
        assert!(has_interface);
    }

    #[test]
    fn test_method_detection() {
        let analyzer = GoAnalyzer::new();
        let code = r#"
type User struct {
    Name string
}

func (u *User) GetName() string {
    return u.Name
}

func (u User) SetName(name string) {
    u.Name = name
}
"#;

        let patterns = analyzer.detect_go_patterns(code);
        assert!(patterns.has_methods);
    }

    #[test]
    fn test_type_assertion_detection() {
        let analyzer = GoAnalyzer::new();
        let code = r#"
func main() {
    var i interface{} = "hello"
    s := i.(string)
    fmt.Println(s)
}
"#;

        let patterns = analyzer.detect_go_patterns(code);
        assert!(patterns.has_type_assertions);
    }

    #[test]
    fn test_reflection_detection() {
        let analyzer = GoAnalyzer::new();
        let code = r#"
import "reflect"

func main() {
    t := reflect.TypeOf(42)
    fmt.Println(t)
}
"#;

        let patterns = analyzer.detect_go_patterns(code);
        assert!(patterns.has_reflection);
    }
}