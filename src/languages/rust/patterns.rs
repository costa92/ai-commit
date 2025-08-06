/// Rust 语言特定的代码模式和最佳实践检测
use regex::Regex;
use std::collections::HashMap;

/// Rust 代码模式检测器
pub struct RustPatternDetector {
    patterns: HashMap<String, RustPattern>,
}

impl RustPatternDetector {
    pub fn new() -> Self {
        let mut patterns = HashMap::new();

        // 错误处理模式
        patterns.insert("result_error_handling".to_string(), RustPattern {
            name: "Result Error Handling".to_string(),
            regex: Regex::new(r"Result<.*,.*>").unwrap(),
            description: "使用 Result 类型进行错误处理".to_string(),
            is_good_practice: true,
            severity: PatternSeverity::Info,
        });

        // Option 类型使用
        patterns.insert("option_usage".to_string(), RustPattern {
            name: "Option Type Usage".to_string(),
            regex: Regex::new(r"Option<.*>").unwrap(),
            description: "使用 Option 类型处理可能为空的值".to_string(),
            is_good_practice: true,
            severity: PatternSeverity::Info,
        });

        // 问号操作符
        patterns.insert("question_mark_operator".to_string(), RustPattern {
            name: "Question Mark Operator".to_string(),
            regex: Regex::new(r"\?\s*;").unwrap(),
            description: "使用 ? 操作符简化错误传播".to_string(),
            is_good_practice: true,
            severity: PatternSeverity::Info,
        });

        // unwrap() 使用（潜在问题）
        patterns.insert("unwrap_usage".to_string(), RustPattern {
            name: "Unwrap Usage".to_string(),
            regex: Regex::new(r"\.unwrap\(\)").unwrap(),
            description: "使用 unwrap() 可能导致 panic，考虑使用模式匹配或 ? 操作符".to_string(),
            is_good_practice: false,
            severity: PatternSeverity::Warning,
        });

        // expect() 使用（相对较好）
        patterns.insert("expect_usage".to_string(), RustPattern {
            name: "Expect Usage".to_string(),
            regex: Regex::new(r"\.expect\(").unwrap(),
            description: "使用 expect() 提供错误信息，比 unwrap() 更好".to_string(),
            is_good_practice: true,
            severity: PatternSeverity::Info,
        });

        // 生命周期参数
        patterns.insert("lifetime_parameters".to_string(), RustPattern {
            name: "Lifetime Parameters".to_string(),
            regex: Regex::new(r"<'[a-zA-Z_]\w*>").unwrap(),
            description: "使用生命周期参数确保内存安全".to_string(),
            is_good_practice: true,
            severity: PatternSeverity::Info,
        });

        // 借用检查器友好的模式
        patterns.insert("borrowing_patterns".to_string(), RustPattern {
            name: "Borrowing Patterns".to_string(),
            regex: Regex::new(r"&\w+|&mut\s+\w+").unwrap(),
            description: "使用借用避免不必要的所有权转移".to_string(),
            is_good_practice: true,
            severity: PatternSeverity::Info,
        });

        // Clone 过度使用
        patterns.insert("excessive_cloning".to_string(), RustPattern {
            name: "Excessive Cloning".to_string(),
            regex: Regex::new(r"\.clone\(\)").unwrap(),
            description: "频繁使用 clone() 可能影响性能，考虑使用借用".to_string(),
            is_good_practice: false,
            severity: PatternSeverity::Info,
        });

        // 智能指针使用
        patterns.insert("smart_pointers".to_string(), RustPattern {
            name: "Smart Pointers".to_string(),
            regex: Regex::new(r"\bBox<|\bRc<|\bArc<|\bRefCell<").unwrap(),
            description: "使用智能指针管理复杂的所有权场景".to_string(),
            is_good_practice: true,
            severity: PatternSeverity::Info,
        });

        // 异步编程模式
        patterns.insert("async_patterns".to_string(), RustPattern {
            name: "Async Programming".to_string(),
            regex: Regex::new(r"\basync\s+fn|\bawait\b").unwrap(),
            description: "使用异步编程提高并发性能".to_string(),
            is_good_practice: true,
            severity: PatternSeverity::Info,
        });

        // 线程安全模式
        patterns.insert("thread_safety".to_string(), RustPattern {
            name: "Thread Safety".to_string(),
            regex: Regex::new(r"std::sync::|Arc<Mutex<|Arc<RwLock<").unwrap(),
            description: "使用线程安全的同步原语".to_string(),
            is_good_practice: true,
            severity: PatternSeverity::Info,
        });

        // unsafe 代码块
        patterns.insert("unsafe_code".to_string(), RustPattern {
            name: "Unsafe Code".to_string(),
            regex: Regex::new(r"\bunsafe\s*\{").unwrap(),
            description: "使用 unsafe 代码，需要仔细审查内存安全性".to_string(),
            is_good_practice: false,
            severity: PatternSeverity::Warning,
        });

        // 原始指针使用
        patterns.insert("raw_pointers".to_string(), RustPattern {
            name: "Raw Pointers".to_string(),
            regex: Regex::new(r"\*const\s+\w+|\*mut\s+\w+").unwrap(),
            description: "使用原始指针，确保在 unsafe 块中正确处理".to_string(),
            is_good_practice: false,
            severity: PatternSeverity::Warning,
        });

        // 模式匹配
        patterns.insert("pattern_matching".to_string(), RustPattern {
            name: "Pattern Matching".to_string(),
            regex: Regex::new(r"\bmatch\s+|\bif\s+let\s+").unwrap(),
            description: "使用模式匹配进行安全的数据解构".to_string(),
            is_good_practice: true,
            severity: PatternSeverity::Info,
        });

        // 迭代器使用
        patterns.insert("iterator_usage".to_string(), RustPattern {
            name: "Iterator Usage".to_string(),
            regex: Regex::new(r"\.iter\(\)|\.into_iter\(\)|\.map\(|\.filter\(|\.collect\(\)").unwrap(),
            description: "使用迭代器进行函数式编程".to_string(),
            is_good_practice: true,
            severity: PatternSeverity::Info,
        });

        // 宏使用
        patterns.insert("macro_usage".to_string(), RustPattern {
            name: "Macro Usage".to_string(),
            regex: Regex::new(r"\w+!").unwrap(),
            description: "使用宏进行代码生成和抽象".to_string(),
            is_good_practice: true,
            severity: PatternSeverity::Info,
        });

        // 内存泄漏风险
        patterns.insert("memory_leak_risk".to_string(), RustPattern {
            name: "Memory Leak Risk".to_string(),
            regex: Regex::new(r"std::mem::forget|Box::leak|Rc::leak").unwrap(),
            description: "可能导致内存泄漏的操作".to_string(),
            is_good_practice: false,
            severity: PatternSeverity::Warning,
        });

        // 零成本抽象
        patterns.insert("zero_cost_abstractions".to_string(), RustPattern {
            name: "Zero Cost Abstractions".to_string(),
            regex: Regex::new(r"impl\s+\w+\s+for\s+\w+|#\[inline\]").unwrap(),
            description: "使用零成本抽象提高性能".to_string(),
            is_good_practice: true,
            severity: PatternSeverity::Info,
        });

        // 错误类型定义
        patterns.insert("custom_error_types".to_string(), RustPattern {
            name: "Custom Error Types".to_string(),
            regex: Regex::new(r"impl\s+std::error::Error|#\[derive\(.*Error.*\)\]").unwrap(),
            description: "定义自定义错误类型提高错误处理质量".to_string(),
            is_good_practice: true,
            severity: PatternSeverity::Info,
        });

        // 文档注释
        patterns.insert("documentation".to_string(), RustPattern {
            name: "Documentation".to_string(),
            regex: Regex::new(r"///|//!").unwrap(),
            description: "使用文档注释提高代码可读性".to_string(),
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
    pub fn detect_pattern_type(&self, content: &str, pattern_type: RustPatternType) -> Vec<PatternMatch> {
        let pattern_ids = match pattern_type {
            RustPatternType::ErrorHandling => vec![
                "result_error_handling", "option_usage", "question_mark_operator",
                "unwrap_usage", "expect_usage", "custom_error_types"
            ],
            RustPatternType::MemorySafety => vec![
                "borrowing_patterns", "lifetime_parameters", "smart_pointers",
                "unsafe_code", "raw_pointers", "memory_leak_risk"
            ],
            RustPatternType::Performance => vec![
                "excessive_cloning", "iterator_usage", "zero_cost_abstractions"
            ],
            RustPatternType::Concurrency => vec![
                "async_patterns", "thread_safety"
            ],
            RustPatternType::CodeQuality => vec![
                "pattern_matching", "macro_usage", "documentation"
            ],
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
    pub fn generate_report(&self, content: &str) -> RustPatternReport {
        let all_matches = self.detect_patterns(content);

        let good_practices = all_matches.iter().filter(|m| m.is_good_practice).count();
        let warnings = all_matches.iter().filter(|m| matches!(m.severity, PatternSeverity::Warning)).count();
        let infos = all_matches.iter().filter(|m| matches!(m.severity, PatternSeverity::Info)).count();

        RustPatternReport {
            total_patterns: all_matches.len(),
            good_practices,
            warnings,
            infos,
            matches: all_matches,
        }
    }
}

/// Rust 代码模式定义
#[derive(Debug, Clone)]
pub struct RustPattern {
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

/// Rust 模式类型
#[derive(Debug, Clone)]
pub enum RustPatternType {
    ErrorHandling,
    MemorySafety,
    Performance,
    Concurrency,
    CodeQuality,
}

/// Rust 模式检测报告
#[derive(Debug, Clone)]
pub struct RustPatternReport {
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
    fn test_result_error_handling_detection() {
        let detector = RustPatternDetector::new();
        let code = r#"
fn read_file() -> Result<String, std::io::Error> {
    std::fs::read_to_string("file.txt")
}
"#;

        let matches = detector.detect_pattern_type(code, RustPatternType::ErrorHandling);
        let result_handling = matches.iter()
            .find(|m| m.pattern_id == "result_error_handling")
            .expect("Should find Result error handling pattern");

        assert!(result_handling.is_good_practice);
    }

    #[test]
    fn test_unwrap_usage_detection() {
        let detector = RustPatternDetector::new();
        let code = r#"
fn main() {
    let value = Some(42).unwrap();
    println!("{}", value);
}
"#;

        let matches = detector.detect_pattern_type(code, RustPatternType::ErrorHandling);
        let unwrap_usage = matches.iter()
            .find(|m| m.pattern_id == "unwrap_usage")
            .expect("Should find unwrap usage");

        assert!(!unwrap_usage.is_good_practice);
        assert_eq!(unwrap_usage.severity, PatternSeverity::Warning);
    }

    #[test]
    fn test_borrowing_patterns_detection() {
        let detector = RustPatternDetector::new();
        let code = r#"
fn process_data(data: &Vec<i32>) {
    for item in data {
        println!("{}", item);
    }
}

fn modify_data(data: &mut Vec<i32>) {
    data.push(42);
}
"#;

        let matches = detector.detect_pattern_type(code, RustPatternType::MemorySafety);
        let borrowing = matches.iter()
            .find(|m| m.pattern_id == "borrowing_patterns")
            .expect("Should find borrowing patterns");

        assert!(borrowing.is_good_practice);
    }

    #[test]
    fn test_unsafe_code_detection() {
        let detector = RustPatternDetector::new();
        let code = r#"
fn dangerous_operation() {
    unsafe {
        let raw_ptr: *const i32 = std::ptr::null();
        let value = *raw_ptr;
    }
}
"#;

        let matches = detector.detect_pattern_type(code, RustPatternType::MemorySafety);
        let unsafe_code = matches.iter()
            .find(|m| m.pattern_id == "unsafe_code")
            .expect("Should find unsafe code");

        assert!(!unsafe_code.is_good_practice);
        assert_eq!(unsafe_code.severity, PatternSeverity::Warning);
    }

    #[test]
    fn test_async_patterns_detection() {
        let detector = RustPatternDetector::new();
        let code = r#"
async fn fetch_data() -> Result<String, reqwest::Error> {
    let response = reqwest::get("https://api.example.com/data").await?;
    response.text().await
}
"#;

        let matches = detector.detect_pattern_type(code, RustPatternType::Concurrency);
        let async_patterns = matches.iter()
            .find(|m| m.pattern_id == "async_patterns")
            .expect("Should find async patterns");

        assert!(async_patterns.is_good_practice);
    }

    #[test]
    fn test_iterator_usage_detection() {
        let detector = RustPatternDetector::new();
        let code = r#"
fn process_numbers(numbers: Vec<i32>) -> Vec<i32> {
    numbers
        .iter()
        .filter(|&&x| x > 0)
        .map(|&x| x * 2)
        .collect()
}
"#;

        let matches = detector.detect_pattern_type(code, RustPatternType::Performance);
        let iterator_usage = matches.iter()
            .find(|m| m.pattern_id == "iterator_usage")
            .expect("Should find iterator usage");

        assert!(iterator_usage.is_good_practice);
    }

    #[test]
    fn test_pattern_matching_detection() {
        let detector = RustPatternDetector::new();
        let code = r#"
fn handle_option(opt: Option<i32>) {
    match opt {
        Some(value) => println!("Got value: {}", value),
        None => println!("No value"),
    }

    if let Some(x) = opt {
        println!("Value is {}", x);
    }
}
"#;

        let matches = detector.detect_pattern_type(code, RustPatternType::CodeQuality);
        let pattern_matching = matches.iter()
            .find(|m| m.pattern_id == "pattern_matching")
            .expect("Should find pattern matching");

        assert!(pattern_matching.is_good_practice);
    }

    #[test]
    fn test_smart_pointers_detection() {
        let detector = RustPatternDetector::new();
        let code = r#"
use std::rc::Rc;
use std::sync::Arc;

fn use_smart_pointers() {
    let boxed = Box::new(42);
    let rc = Rc::new("hello");
    let arc = Arc::new(vec![1, 2, 3]);
}
"#;

        let matches = detector.detect_pattern_type(code, RustPatternType::MemorySafety);
        let smart_pointers = matches.iter()
            .find(|m| m.pattern_id == "smart_pointers")
            .expect("Should find smart pointers");

        assert!(smart_pointers.is_good_practice);
    }

    #[test]
    fn test_documentation_detection() {
        let detector = RustPatternDetector::new();
        let code = r#"
/// This function adds two numbers together
///
/// # Examples
///
/// ```
/// let result = add(2, 3);
/// assert_eq!(result, 5);
/// ```
fn add(a: i32, b: i32) -> i32 {
    a + b
}

//! This module provides mathematical operations
"#;

        let matches = detector.detect_pattern_type(code, RustPatternType::CodeQuality);
        let documentation = matches.iter()
            .find(|m| m.pattern_id == "documentation")
            .expect("Should find documentation");

        assert!(documentation.is_good_practice);
    }

    #[test]
    fn test_generate_report() {
        let detector = RustPatternDetector::new();
        let code = r#"
/// Well documented function
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let data = vec![1, 2, 3];
    let result = data.iter().map(|x| x * 2).collect::<Vec<_>>();

    match result.get(0) {
        Some(value) => println!("First value: {}", value),
        None => println!("No values"),
    }

    Ok(())
}
"#;

        let report = detector.generate_report(code);

        assert!(report.total_patterns > 0);
        assert!(report.good_practices > 0);
        assert!(!report.matches.is_empty());
    }

    #[test]
    fn test_memory_leak_risk_detection() {
        let detector = RustPatternDetector::new();
        let code = r#"
fn potential_leak() {
    let boxed = Box::new(42);
    std::mem::forget(boxed);

    let leaked = Box::leak(Box::new("leaked string"));
}
"#;

        let matches = detector.detect_pattern_type(code, RustPatternType::MemorySafety);
        let memory_leak = matches.iter()
            .find(|m| m.pattern_id == "memory_leak_risk")
            .expect("Should find memory leak risk");

        assert!(!memory_leak.is_good_practice);
        assert_eq!(memory_leak.severity, PatternSeverity::Warning);
    }

    #[test]
    fn test_question_mark_operator_detection() {
        let detector = RustPatternDetector::new();
        let code = r#"
fn read_and_parse() -> Result<i32, Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string("number.txt")?;
    let number = content.trim().parse::<i32>()?;
    Ok(number)
}
"#;

        let matches = detector.detect_pattern_type(code, RustPatternType::ErrorHandling);
        let question_mark = matches.iter()
            .find(|m| m.pattern_id == "question_mark_operator")
            .expect("Should find question mark operator");

        assert!(question_mark.is_good_practice);
    }
}