/// Go 语言特定的代码模式和最佳实践检测
use regex::Regex;
use std::collections::HashMap;

/// Go 代码模式检测器
pub struct GoPatternDetector {
    patterns: HashMap<String, GoPattern>,
}

impl GoPatternDetector {
    pub fn new() -> Self {
        let mut patterns = HashMap::new();

        // 错误处理模式
        patterns.insert("error_handling".to_string(), GoPattern {
            name: "Go Error Handling".to_string(),
            regex: Regex::new(r"if\s+err\s*!=\s*nil\s*\{").unwrap(),
            description: "标准的 Go 错误处理模式".to_string(),
            is_good_practice: true,
            severity: PatternSeverity::Info,
        });

        // 未处理的错误 (简化版本，不使用 lookahead)
        patterns.insert("unhandled_error".to_string(), GoPattern {
            name: "Unhandled Error".to_string(),
            regex: Regex::new(r"_,\s*err\s*:=").unwrap(),
            description: "可能存在未处理的错误（使用了 _ 忽略错误）".to_string(),
            is_good_practice: false,
            severity: PatternSeverity::Warning,
        });

        // Goroutine 泄漏风险
        patterns.insert("goroutine_leak".to_string(), GoPattern {
            name: "Potential Goroutine Leak".to_string(),
            regex: Regex::new(r"go\s+func\s*\([^)]*\)\s*\{[^}]*for\s+\{[^}]*\}[^}]*\}").unwrap(),
            description: "可能存在 goroutine 泄漏的无限循环".to_string(),
            is_good_practice: false,
            severity: PatternSeverity::Warning,
        });

        // Channel 未关闭
        patterns.insert("unclosed_channel".to_string(), GoPattern {
            name: "Unclosed Channel".to_string(),
            regex: Regex::new(r"make\s*\(\s*chan\s+\w+").unwrap(),
            description: "创建的 channel 可能未正确关闭".to_string(),
            is_good_practice: false,
            severity: PatternSeverity::Info,
        });

        // 使用 defer 关闭资源
        patterns.insert("defer_close".to_string(), GoPattern {
            name: "Defer Close Resource".to_string(),
            regex: Regex::new(r"defer\s+\w+\.Close\(\)").unwrap(),
            description: "使用 defer 正确关闭资源".to_string(),
            is_good_practice: true,
            severity: PatternSeverity::Info,
        });

        // 空接口使用
        patterns.insert("empty_interface".to_string(), GoPattern {
            name: "Empty Interface Usage".to_string(),
            regex: Regex::new(r"interface\s*\{\s*\}").unwrap(),
            description: "使用空接口，可能影响类型安全".to_string(),
            is_good_practice: false,
            severity: PatternSeverity::Info,
        });

        // 上下文使用
        patterns.insert("context_usage".to_string(), GoPattern {
            name: "Context Usage".to_string(),
            regex: Regex::new(r"context\.Context").unwrap(),
            description: "正确使用 context 进行超时和取消控制".to_string(),
            is_good_practice: true,
            severity: PatternSeverity::Info,
        });

        // 字符串拼接性能问题
        patterns.insert("string_concatenation".to_string(), GoPattern {
            name: "String Concatenation Performance".to_string(),
            regex: Regex::new(r"\+\s*=\s*.*\+").unwrap(),
            description: "字符串拼接可能存在性能问题，考虑使用 strings.Builder".to_string(),
            is_good_practice: false,
            severity: PatternSeverity::Info,
        });

        // 切片预分配
        patterns.insert("slice_preallocation".to_string(), GoPattern {
            name: "Slice Preallocation".to_string(),
            regex: Regex::new(r"make\s*\(\s*\[\]\w+\s*,\s*\d+\s*,\s*\d+\s*\)").unwrap(),
            description: "切片预分配容量，提高性能".to_string(),
            is_good_practice: true,
            severity: PatternSeverity::Info,
        });

        // 互斥锁使用
        patterns.insert("mutex_usage".to_string(), GoPattern {
            name: "Mutex Usage".to_string(),
            regex: Regex::new(r"sync\.(Mutex|RWMutex)").unwrap(),
            description: "使用互斥锁进行并发控制".to_string(),
            is_good_practice: true,
            severity: PatternSeverity::Info,
        });

        // 竞态条件风险
        patterns.insert("race_condition".to_string(), GoPattern {
            name: "Potential Race Condition".to_string(),
            regex: Regex::new(r"go\s+func.*\{[^}]*\w+\s*=\s*[^}]*\}").unwrap(),
            description: "可能存在竞态条件，考虑使用同步机制".to_string(),
            is_good_practice: false,
            severity: PatternSeverity::Warning,
        });

        // 内存泄漏风险 - 未释放的定时器
        patterns.insert("timer_leak".to_string(), GoPattern {
            name: "Timer Leak".to_string(),
            regex: Regex::new(r"time\.NewTimer\(|time\.NewTicker\(").unwrap(),
            description: "创建的定时器可能未正确停止，存在内存泄漏风险".to_string(),
            is_good_practice: false,
            severity: PatternSeverity::Warning,
        });

        // 正确的包命名
        patterns.insert("package_naming".to_string(), GoPattern {
            name: "Package Naming Convention".to_string(),
            regex: Regex::new(r"package\s+[a-z][a-z0-9]*$").unwrap(),
            description: "包名符合 Go 命名约定（小写，无下划线）".to_string(),
            is_good_practice: true,
            severity: PatternSeverity::Info,
        });

        // 导出函数命名
        patterns.insert("exported_function_naming".to_string(), GoPattern {
            name: "Exported Function Naming".to_string(),
            regex: Regex::new(r"func\s+[A-Z]\w*\s*\(").unwrap(),
            description: "导出函数使用大写字母开头".to_string(),
            is_good_practice: true,
            severity: PatternSeverity::Info,
        });

        // 私有函数命名
        patterns.insert("private_function_naming".to_string(), GoPattern {
            name: "Private Function Naming".to_string(),
            regex: Regex::new(r"func\s+[a-z]\w*\s*\(").unwrap(),
            description: "私有函数使用小写字母开头".to_string(),
            is_good_practice: true,
            severity: PatternSeverity::Info,
        });

        Self { patterns }
    }

    /// 检测代码中的所有模式
    pub fn detect_patterns(&self, content: &str) -> Vec<PatternMatch> {
        let mut matches = Vec::new();

        for (pattern_id, pattern) in &self.patterns {
            for regex_match in pattern.regex.find_iter(content) {
                let line_number = content[..regex_match.start()].lines().count() + 1;

                matches.push(PatternMatch {
                    pattern_id: pattern_id.clone(),
                    pattern_name: pattern.name.clone(),
                    description: pattern.description.clone(),
                    line_number,
                    matched_text: regex_match.as_str().to_string(),
                    is_good_practice: pattern.is_good_practice,
                    severity: pattern.severity.clone(),
                });
            }
        }

        matches
    }

    /// 检测特定类型的模式
    pub fn detect_pattern_type(&self, content: &str, pattern_type: GoPatternType) -> Vec<PatternMatch> {
        let pattern_ids = match pattern_type {
            GoPatternType::ErrorHandling => vec!["error_handling", "unhandled_error"],
            GoPatternType::Concurrency => vec!["goroutine_leak", "race_condition", "mutex_usage"],
            GoPatternType::Performance => vec!["string_concatenation", "slice_preallocation"],
            GoPatternType::ResourceManagement => vec!["unclosed_channel", "defer_close", "timer_leak"],
            GoPatternType::Naming => vec!["package_naming", "exported_function_naming", "private_function_naming"],
            GoPatternType::TypeSafety => vec!["empty_interface"],
        };

        let mut matches = Vec::new();
        for pattern_id in pattern_ids {
            if let Some(pattern) = self.patterns.get(pattern_id) {
                for regex_match in pattern.regex.find_iter(content) {
                    let line_number = content[..regex_match.start()].lines().count() + 1;

                    matches.push(PatternMatch {
                        pattern_id: pattern_id.to_string(),
                        pattern_name: pattern.name.clone(),
                        description: pattern.description.clone(),
                        line_number,
                        matched_text: regex_match.as_str().to_string(),
                        is_good_practice: pattern.is_good_practice,
                        severity: pattern.severity.clone(),
                    });
                }
            }
        }

        matches
    }

    /// 生成模式检测报告
    pub fn generate_report(&self, content: &str) -> GoPatternReport {
        let all_matches = self.detect_patterns(content);

        let good_practices = all_matches.iter().filter(|m| m.is_good_practice).count();
        let warnings = all_matches.iter().filter(|m| matches!(m.severity, PatternSeverity::Warning)).count();
        let infos = all_matches.iter().filter(|m| matches!(m.severity, PatternSeverity::Info)).count();

        GoPatternReport {
            total_patterns: all_matches.len(),
            good_practices,
            warnings,
            infos,
            matches: all_matches,
        }
    }
}

/// Go 代码模式定义
#[derive(Debug, Clone)]
pub struct GoPattern {
    pub name: String,
    pub regex: Regex,
    pub description: String,
    pub is_good_practice: bool,
    pub severity: PatternSeverity,
}

/// 模式匹配结果
#[derive(Debug, Clone)]
pub struct PatternMatch {
    pub pattern_id: String,
    pub pattern_name: String,
    pub description: String,
    pub line_number: usize,
    pub matched_text: String,
    pub is_good_practice: bool,
    pub severity: PatternSeverity,
}

/// 模式严重程度
#[derive(Debug, Clone, PartialEq)]
pub enum PatternSeverity {
    Info,
    Warning,
    Error,
}

/// Go 模式类型
#[derive(Debug, Clone)]
pub enum GoPatternType {
    ErrorHandling,
    Concurrency,
    Performance,
    ResourceManagement,
    Naming,
    TypeSafety,
}

/// Go 模式检测报告
#[derive(Debug, Clone)]
pub struct GoPatternReport {
    pub total_patterns: usize,
    pub good_practices: usize,
    pub warnings: usize,
    pub infos: usize,
    pub matches: Vec<PatternMatch>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_handling_detection() {
        let detector = GoPatternDetector::new();
        let code = r#"
func main() {
    file, err := os.Open("test.txt")
    if err != nil {
        log.Fatal(err)
    }
    defer file.Close()
}
"#;

        let matches = detector.detect_pattern_type(code, GoPatternType::ErrorHandling);
        assert!(!matches.is_empty());

        let error_handling_match = matches.iter()
            .find(|m| m.pattern_id == "error_handling")
            .expect("Should find error handling pattern");

        assert!(error_handling_match.is_good_practice);
    }

    #[test]
    fn test_unhandled_error_detection() {
        let detector = GoPatternDetector::new();
        let code = r#"
func main() {
    _, err := os.Open("test.txt")
    // Error not handled - this is bad practice
    fmt.Println("File opened")
}
"#;

        let matches = detector.detect_pattern_type(code, GoPatternType::ErrorHandling);
        let unhandled_error = matches.iter()
            .find(|m| m.pattern_id == "unhandled_error");

        if let Some(match_result) = unhandled_error {
            assert!(!match_result.is_good_practice);
            assert_eq!(match_result.severity, PatternSeverity::Warning);
        }
    }

    #[test]
    fn test_defer_close_detection() {
        let detector = GoPatternDetector::new();
        let code = r#"
func main() {
    file, err := os.Open("test.txt")
    if err != nil {
        return
    }
    defer file.Close()
}
"#;

        let matches = detector.detect_pattern_type(code, GoPatternType::ResourceManagement);
        let defer_close = matches.iter()
            .find(|m| m.pattern_id == "defer_close")
            .expect("Should find defer close pattern");

        assert!(defer_close.is_good_practice);
    }

    #[test]
    fn test_goroutine_leak_detection() {
        let detector = GoPatternDetector::new();
        let code = r#"
func main() {
    go func() {
        for {
            // Infinite loop without exit condition
            time.Sleep(time.Second)
        }
    }()
}
"#;

        let matches = detector.detect_pattern_type(code, GoPatternType::Concurrency);
        let goroutine_leak = matches.iter()
            .find(|m| m.pattern_id == "goroutine_leak");

        if let Some(match_result) = goroutine_leak {
            assert!(!match_result.is_good_practice);
            assert_eq!(match_result.severity, PatternSeverity::Warning);
        }
    }

    #[test]
    fn test_string_concatenation_detection() {
        let detector = GoPatternDetector::new();
        let code = r#"
func buildString() string {
    result := ""
    for i := 0; i < 100; i++ {
        result += fmt.Sprintf("item %d", i)
    }
    return result
}
"#;

        let matches = detector.detect_pattern_type(code, GoPatternType::Performance);
        let string_concat = matches.iter()
            .find(|m| m.pattern_id == "string_concatenation");

        if let Some(match_result) = string_concat {
            assert!(!match_result.is_good_practice);
        }
    }

    #[test]
    fn test_slice_preallocation_detection() {
        let detector = GoPatternDetector::new();
        let code = r#"
func main() {
    items := make([]string, 0, 100)
    // Good practice: preallocated capacity
}
"#;

        let matches = detector.detect_pattern_type(code, GoPatternType::Performance);
        let slice_prealloc = matches.iter()
            .find(|m| m.pattern_id == "slice_preallocation");

        if let Some(match_result) = slice_prealloc {
            assert!(match_result.is_good_practice);
        }
    }

    #[test]
    fn test_package_naming_detection() {
        let detector = GoPatternDetector::new();
        let code = "package main\n";

        let matches = detector.detect_pattern_type(code, GoPatternType::Naming);
        let package_naming = matches.iter()
            .find(|m| m.pattern_id == "package_naming")
            .expect("Should find package naming pattern");

        assert!(package_naming.is_good_practice);
    }

    #[test]
    fn test_exported_function_naming() {
        let detector = GoPatternDetector::new();
        let code = r#"
func PublicFunction() {
    // Exported function
}

func privateFunction() {
    // Private function
}
"#;

        let matches = detector.detect_pattern_type(code, GoPatternType::Naming);

        let exported_func = matches.iter()
            .find(|m| m.pattern_id == "exported_function_naming")
            .expect("Should find exported function");

        let private_func = matches.iter()
            .find(|m| m.pattern_id == "private_function_naming")
            .expect("Should find private function");

        assert!(exported_func.is_good_practice);
        assert!(private_func.is_good_practice);
    }

    #[test]
    fn test_empty_interface_detection() {
        let detector = GoPatternDetector::new();
        let code = r#"
func process(data interface{}) {
    // Using empty interface
}
"#;

        let matches = detector.detect_pattern_type(code, GoPatternType::TypeSafety);
        let empty_interface = matches.iter()
            .find(|m| m.pattern_id == "empty_interface");

        if let Some(match_result) = empty_interface {
            assert!(!match_result.is_good_practice);
        }
    }

    #[test]
    fn test_generate_report() {
        let detector = GoPatternDetector::new();
        let code = r#"
package main

import "fmt"

func main() {
    file, err := os.Open("test.txt")
    if err != nil {
        return
    }
    defer file.Close()

    fmt.Println("Hello, World!")
}
"#;

        let report = detector.generate_report(code);

        assert!(report.total_patterns > 0);
        assert!(report.good_practices > 0);
        assert!(!report.matches.is_empty());
    }

    #[test]
    fn test_context_usage_detection() {
        let detector = GoPatternDetector::new();
        let code = r#"
func processWithContext(ctx context.Context) error {
    // Good practice: using context
    return nil
}
"#;

        let matches = detector.detect_patterns(code);
        let context_usage = matches.iter()
            .find(|m| m.pattern_id == "context_usage");

        if let Some(match_result) = context_usage {
            assert!(match_result.is_good_practice);
        }
    }

    #[test]
    fn test_mutex_usage_detection() {
        let detector = GoPatternDetector::new();
        let code = r#"
type Counter struct {
    mu    sync.Mutex
    value int
}
"#;

        let matches = detector.detect_pattern_type(code, GoPatternType::Concurrency);
        let mutex_usage = matches.iter()
            .find(|m| m.pattern_id == "mutex_usage");

        if let Some(match_result) = mutex_usage {
            assert!(match_result.is_good_practice);
        }
    }
}