use crate::languages::{LanguageAnalyzer, LanguageFeature};
use regex::Regex;
use std::collections::HashSet;

/// Rust 语言分析器
pub struct RustAnalyzer {
    patterns: RustPatterns,
}

impl RustAnalyzer {
    pub fn new() -> Self {
        Self {
            patterns: RustPatterns::new(),
        }
    }

    /// 提取模块声明
    fn extract_module_declarations(&self, content: &str) -> Vec<LanguageFeature> {
        let mut modules = Vec::new();

        for captures in self.patterns.module_regex.captures_iter(content) {
            if let Some(module_name) = captures.get(1) {
                modules.push(LanguageFeature::Module(module_name.as_str().to_string()));
            }
        }

        modules
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

    /// 提取 trait 定义
    fn extract_traits(&self, content: &str) -> Vec<LanguageFeature> {
        let mut traits = Vec::new();

        for captures in self.patterns.trait_regex.captures_iter(content) {
            if let Some(trait_name) = captures.get(1) {
                traits.push(LanguageFeature::Interface(trait_name.as_str().to_string()));
            }
        }

        traits
    }

    /// 检测 Rust 特定的代码模式
    pub fn detect_rust_patterns(&self, content: &str) -> RustCodePatterns {
        let mut patterns = RustCodePatterns::default();

        // 检测内存安全模式
        patterns.has_ownership_patterns = self.patterns.ownership_regex.is_match(content);
        patterns.has_borrowing = self.patterns.borrowing_regex.is_match(content);
        patterns.has_lifetimes = self.patterns.lifetime_regex.is_match(content);

        // 检测错误处理
        patterns.has_result_type = self.patterns.result_regex.is_match(content);
        patterns.has_option_type = self.patterns.option_regex.is_match(content);
        patterns.has_question_mark_operator = self.patterns.question_mark_regex.is_match(content);

        // 检测并发模式
        patterns.has_async_await = self.patterns.async_await_regex.is_match(content);
        patterns.has_threads = self.patterns.thread_regex.is_match(content);
        patterns.has_channels = self.patterns.channel_regex.is_match(content);

        // 检测模式匹配
        patterns.has_match_expressions = self.patterns.match_regex.is_match(content);
        patterns.has_if_let = self.patterns.if_let_regex.is_match(content);

        // 检测宏使用
        patterns.has_macros = self.patterns.macro_regex.is_match(content);

        // 检测 unsafe 代码
        patterns.has_unsafe_code = self.patterns.unsafe_regex.is_match(content);

        // 检测泛型
        patterns.has_generics = self.patterns.generic_regex.is_match(content);

        // 检测智能指针
        patterns.has_smart_pointers = self.patterns.smart_pointer_regex.is_match(content);

        patterns
    }

    /// 分析导入的 crate 和模块
    pub fn analyze_imports(&self, content: &str) -> Vec<String> {
        let mut imports = Vec::new();

        // 外部 crate 导入
        for captures in self.patterns.extern_crate_regex.captures_iter(content) {
            if let Some(crate_name) = captures.get(1) {
                imports.push(format!("extern:{}", crate_name.as_str()));
            }
        }

        // use 语句导入
        for captures in self.patterns.use_regex.captures_iter(content) {
            if let Some(use_path) = captures.get(1) {
                imports.push(use_path.as_str().to_string());
            }
        }

        // 去重并排序
        let mut unique_imports: Vec<String> = imports.into_iter().collect::<HashSet<_>>().into_iter().collect();
        unique_imports.sort();
        unique_imports
    }

    /// 分析函数复杂度
    pub fn analyze_function_complexity(&self, content: &str) -> Vec<FunctionComplexity> {
        let mut complexities = Vec::new();

        for captures in self.patterns.function_regex.captures_iter(content) {
            if let Some(func_name) = captures.get(1) {
                let func_name = func_name.as_str().to_string();
                let func_start = captures.get(0).unwrap().end();

                // 找到函数体
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
        complexity += self.patterns.while_regex.find_iter(function_body).count() as u32;
        complexity += self.patterns.loop_regex.find_iter(function_body).count() as u32;
        complexity += self.patterns.match_regex.find_iter(function_body).count() as u32;

        complexity
    }

    /// 检测内存安全问题
    pub fn detect_memory_safety_issues(&self, content: &str) -> Vec<MemorySafetyIssue> {
        let mut issues = Vec::new();

        // 检测 unsafe 代码块
        for regex_match in self.patterns.unsafe_regex.find_iter(content) {
            let line_number = content[..regex_match.start()].lines().count() + 1;
            issues.push(MemorySafetyIssue {
                issue_type: MemorySafetyIssueType::UnsafeCode,
                line_number,
                description: "使用了 unsafe 代码块，需要仔细审查内存安全性".to_string(),
                severity: SafetySeverity::High,
            });
        }

        // 检测原始指针使用
        for regex_match in self.patterns.raw_pointer_regex.find_iter(content) {
            let line_number = content[..regex_match.start()].lines().count() + 1;
            issues.push(MemorySafetyIssue {
                issue_type: MemorySafetyIssueType::RawPointer,
                line_number,
                description: "使用了原始指针，可能存在内存安全风险".to_string(),
                severity: SafetySeverity::Medium,
            });
        }

        // 检测可能的内存泄漏
        for regex_match in self.patterns.memory_leak_regex.find_iter(content) {
            let line_number = content[..regex_match.start()].lines().count() + 1;
            issues.push(MemorySafetyIssue {
                issue_type: MemorySafetyIssueType::PotentialLeak,
                line_number,
                description: "可能存在内存泄漏，检查资源是否正确释放".to_string(),
                severity: SafetySeverity::Medium,
            });
        }

        issues
    }

    /// 检测所有权模式
    pub fn detect_ownership_patterns(&self, content: &str) -> Vec<OwnershipPattern> {
        let mut patterns = Vec::new();

        // 检测移动语义
        for regex_match in self.patterns.move_regex.find_iter(content) {
            let line_number = content[..regex_match.start()].lines().count() + 1;
            patterns.push(OwnershipPattern {
                pattern_type: OwnershipPatternType::Move,
                line_number,
                description: "使用了移动语义".to_string(),
            });
        }

        // 检测借用
        for regex_match in self.patterns.borrowing_regex.find_iter(content) {
            let line_number = content[..regex_match.start()].lines().count() + 1;
            patterns.push(OwnershipPattern {
                pattern_type: OwnershipPatternType::Borrow,
                line_number,
                description: "使用了借用".to_string(),
            });
        }

        // 检测可变借用
        for regex_match in self.patterns.mutable_borrow_regex.find_iter(content) {
            let line_number = content[..regex_match.start()].lines().count() + 1;
            patterns.push(OwnershipPattern {
                pattern_type: OwnershipPatternType::MutableBorrow,
                line_number,
                description: "使用了可变借用".to_string(),
            });
        }

        patterns
    }
}

impl LanguageAnalyzer for RustAnalyzer {
    fn analyze_features(&self, content: &str) -> Vec<LanguageFeature> {
        let mut features = Vec::new();

        // 提取模块
        features.extend(self.extract_module_declarations(content));

        // 提取函数
        features.extend(self.extract_functions(content));

        // 提取结构体
        features.extend(self.extract_structs(content));

        // 提取 trait
        features.extend(self.extract_traits(content));

        features
    }
}

/// Rust 语言正则表达式模式
struct RustPatterns {
    module_regex: Regex,
    function_regex: Regex,
    struct_regex: Regex,
    trait_regex: Regex,
    ownership_regex: Regex,
    borrowing_regex: Regex,
    mutable_borrow_regex: Regex,
    lifetime_regex: Regex,
    result_regex: Regex,
    option_regex: Regex,
    question_mark_regex: Regex,
    async_await_regex: Regex,
    thread_regex: Regex,
    channel_regex: Regex,
    match_regex: Regex,
    if_let_regex: Regex,
    macro_regex: Regex,
    unsafe_regex: Regex,
    generic_regex: Regex,
    smart_pointer_regex: Regex,
    extern_crate_regex: Regex,
    use_regex: Regex,
    if_regex: Regex,
    for_regex: Regex,
    while_regex: Regex,
    loop_regex: Regex,
    raw_pointer_regex: Regex,
    memory_leak_regex: Regex,
    move_regex: Regex,
}

impl RustPatterns {
    fn new() -> Self {
        Self {
            module_regex: Regex::new(r"(?m)^mod\s+(\w+)").unwrap(),
            function_regex: Regex::new(r"(?m)fn\s+(\w+)\s*\(").unwrap(),
            struct_regex: Regex::new(r"(?m)^struct\s+(\w+)").unwrap(),
            trait_regex: Regex::new(r"(?m)^trait\s+(\w+)").unwrap(),
            ownership_regex: Regex::new(r"\bmove\b|\bBox::|Rc::|Arc::").unwrap(),
            borrowing_regex: Regex::new(r"&\w+|&mut\s+\w+").unwrap(),
            mutable_borrow_regex: Regex::new(r"&mut\s+\w+").unwrap(),
            lifetime_regex: Regex::new(r"'[a-zA-Z_]\w*").unwrap(),
            result_regex: Regex::new(r"\bResult<").unwrap(),
            option_regex: Regex::new(r"\bOption<").unwrap(),
            question_mark_regex: Regex::new(r"\?\s*;").unwrap(),
            async_await_regex: Regex::new(r"\basync\s+fn|\bawait\b").unwrap(),
            thread_regex: Regex::new(r"std::thread::|thread::spawn").unwrap(),
            channel_regex: Regex::new(r"std::sync::mpsc::|mpsc::channel").unwrap(),
            match_regex: Regex::new(r"\bmatch\s+").unwrap(),
            if_let_regex: Regex::new(r"\bif\s+let\s+").unwrap(),
            macro_regex: Regex::new(r"\w+!").unwrap(),
            unsafe_regex: Regex::new(r"\bunsafe\s*\{").unwrap(),
            generic_regex: Regex::new(r"<[A-Z]\w*>|<\w+:\s*\w+>").unwrap(),
            smart_pointer_regex: Regex::new(r"\bBox<|\bRc<|\bArc<|\bRefCell<|\bBox::|\bRc::|\bArc::|\bRefCell::").unwrap(),
            extern_crate_regex: Regex::new(r"(?m)^extern\s+crate\s+(\w+)").unwrap(),
            use_regex: Regex::new(r"(?m)^use\s+([\w:{}_, ]+)").unwrap(),
            if_regex: Regex::new(r"\bif\s+").unwrap(),
            for_regex: Regex::new(r"\bfor\s+").unwrap(),
            while_regex: Regex::new(r"\bwhile\s+").unwrap(),
            loop_regex: Regex::new(r"\bloop\s*\{").unwrap(),
            raw_pointer_regex: Regex::new(r"\*const\s+\w+|\*mut\s+\w+").unwrap(),
            memory_leak_regex: Regex::new(r"std::mem::forget|Box::leak").unwrap(),
            move_regex: Regex::new(r"\bmove\s*\|").unwrap(),
        }
    }
}

/// Rust 代码模式检测结果
#[derive(Debug, Default, Clone)]
pub struct RustCodePatterns {
    pub has_ownership_patterns: bool,
    pub has_borrowing: bool,
    pub has_lifetimes: bool,
    pub has_result_type: bool,
    pub has_option_type: bool,
    pub has_question_mark_operator: bool,
    pub has_async_await: bool,
    pub has_threads: bool,
    pub has_channels: bool,
    pub has_match_expressions: bool,
    pub has_if_let: bool,
    pub has_macros: bool,
    pub has_unsafe_code: bool,
    pub has_generics: bool,
    pub has_smart_pointers: bool,
}

/// 函数复杂度信息
#[derive(Debug, Clone)]
pub struct FunctionComplexity {
    pub name: String,
    pub cyclomatic_complexity: u32,
    pub line_count: usize,
}

/// 内存安全问题
#[derive(Debug, Clone)]
pub struct MemorySafetyIssue {
    pub issue_type: MemorySafetyIssueType,
    pub line_number: usize,
    pub description: String,
    pub severity: SafetySeverity,
}

#[derive(Debug, Clone)]
pub enum MemorySafetyIssueType {
    UnsafeCode,
    RawPointer,
    PotentialLeak,
}

#[derive(Debug, Clone)]
pub enum SafetySeverity {
    Low,
    Medium,
    High,
}

/// 所有权模式
#[derive(Debug, Clone)]
pub struct OwnershipPattern {
    pub pattern_type: OwnershipPatternType,
    pub line_number: usize,
    pub description: String,
}

#[derive(Debug, Clone)]
pub enum OwnershipPatternType {
    Move,
    Borrow,
    MutableBorrow,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rust_analyzer_creation() {
        let analyzer = RustAnalyzer::new();
        // 验证分析器创建成功
        assert!(analyzer.patterns.function_regex.is_match("fn main() {"));
    }

    #[test]
    fn test_module_extraction() {
        let analyzer = RustAnalyzer::new();
        let code = r#"
mod utils;
mod config;

fn main() {}
"#;

        let modules = analyzer.extract_module_declarations(code);
        assert_eq!(modules.len(), 2);

        if let LanguageFeature::Module(name) = &modules[0] {
            assert_eq!(name, "utils");
        }

        if let LanguageFeature::Module(name) = &modules[1] {
            assert_eq!(name, "config");
        }
    }

    #[test]
    fn test_function_extraction() {
        let analyzer = RustAnalyzer::new();
        let code = r#"
fn main() {
    println!("Hello, World!");
}

fn helper(x: i32) -> String {
    format!("{}", x)
}

pub fn public_function() {
    // public function
}
"#;

        let functions = analyzer.extract_functions(code);
        assert_eq!(functions.len(), 3);

        if let LanguageFeature::Function(name) = &functions[0] {
            assert_eq!(name, "main");
        }

        if let LanguageFeature::Function(name) = &functions[1] {
            assert_eq!(name, "helper");
        }

        if let LanguageFeature::Function(name) = &functions[2] {
            assert_eq!(name, "public_function");
        }
    }

    #[test]
    fn test_struct_extraction() {
        let analyzer = RustAnalyzer::new();
        let code = r#"
struct User {
    name: String,
    age: u32,
}

struct Config {
    host: String,
    port: u16,
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
    fn test_trait_extraction() {
        let analyzer = RustAnalyzer::new();
        let code = r#"
trait Display {
    fn fmt(&self) -> String;
}

trait Clone {
    fn clone(&self) -> Self;
}
"#;

        let traits = analyzer.extract_traits(code);
        assert_eq!(traits.len(), 2);

        if let LanguageFeature::Interface(name) = &traits[0] {
            assert_eq!(name, "Display");
        }

        if let LanguageFeature::Interface(name) = &traits[1] {
            assert_eq!(name, "Clone");
        }
    }

    #[test]
    fn test_rust_pattern_detection() {
        let analyzer = RustAnalyzer::new();
        let code = r#"
use std::sync::mpsc;
use std::thread;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        tx.send(42).unwrap();
    });

    let value = rx.recv()?;

    match value {
        42 => println!("Found the answer!"),
        _ => println!("Something else"),
    }

    if let Some(data) = get_optional_data() {
        println!("Got data: {}", data);
    }

    Ok(())
}

fn get_optional_data() -> Option<String> {
    Some("test".to_string())
}

async fn async_function() {
    let result = some_async_operation().await;
}
"#;

        let patterns = analyzer.detect_rust_patterns(code);

        assert!(patterns.has_result_type);
        assert!(patterns.has_option_type);
        assert!(patterns.has_question_mark_operator);
        assert!(patterns.has_match_expressions);
        assert!(patterns.has_if_let);
        assert!(patterns.has_threads);
        assert!(patterns.has_channels);
        assert!(patterns.has_async_await);
        assert!(patterns.has_ownership_patterns);
    }

    #[test]
    fn test_import_analysis() {
        let analyzer = RustAnalyzer::new();
        let code = r#"
extern crate serde;
extern crate tokio;

use std::collections::HashMap;
use std::sync::Arc;
use serde::{Serialize, Deserialize};
"#;

        let imports = analyzer.analyze_imports(code);
        assert!(imports.contains(&"extern:serde".to_string()));
        assert!(imports.contains(&"extern:tokio".to_string()));
        assert!(imports.contains(&"std::collections::HashMap".to_string()));
        assert!(imports.contains(&"std::sync::Arc".to_string()));
        assert!(imports.contains(&"serde::{Serialize, Deserialize}".to_string()));
    }

    #[test]
    fn test_function_complexity() {
        let analyzer = RustAnalyzer::new();
        let code = r#"
fn complex_function(x: i32) -> i32 {
    if x > 0 {
        for i in 0..x {
            match i {
                1 => return 1,
                2 => return 2,
                _ => continue,
            }
        }
    }

    while x > 10 {
        loop {
            break;
        }
    }

    0
}
"#;

        let complexities = analyzer.analyze_function_complexity(code);
        assert_eq!(complexities.len(), 1);

        let complexity = &complexities[0];
        assert_eq!(complexity.name, "complex_function");
        assert!(complexity.cyclomatic_complexity > 1);
    }

    #[test]
    fn test_memory_safety_detection() {
        let analyzer = RustAnalyzer::new();
        let code = r#"
fn unsafe_function() {
    unsafe {
        let raw_ptr: *const i32 = std::ptr::null();
        let value = *raw_ptr;
    }
}

fn potential_leak() {
    let boxed = Box::new(42);
    std::mem::forget(boxed);
}
"#;

        let issues = analyzer.detect_memory_safety_issues(code);
        assert!(!issues.is_empty());

        let has_unsafe = issues.iter().any(|issue| matches!(issue.issue_type, MemorySafetyIssueType::UnsafeCode));
        let has_raw_pointer = issues.iter().any(|issue| matches!(issue.issue_type, MemorySafetyIssueType::RawPointer));
        let has_potential_leak = issues.iter().any(|issue| matches!(issue.issue_type, MemorySafetyIssueType::PotentialLeak));

        assert!(has_unsafe);
        assert!(has_raw_pointer);
        assert!(has_potential_leak);
    }

    #[test]
    fn test_ownership_pattern_detection() {
        let analyzer = RustAnalyzer::new();
        let code = r#"
fn ownership_examples() {
    let data = vec![1, 2, 3];

    // Move closure
    let closure = move |x| {
        println!("{:?}", data);
    };

    // Borrowing
    let borrowed = &data;

    // Mutable borrowing
    let mut mutable_data = vec![1, 2, 3];
    let mutable_borrowed = &mut mutable_data;
}
"#;

        let patterns = analyzer.detect_ownership_patterns(code);
        assert!(!patterns.is_empty());

        let has_move = patterns.iter().any(|p| matches!(p.pattern_type, OwnershipPatternType::Move));
        let has_borrow = patterns.iter().any(|p| matches!(p.pattern_type, OwnershipPatternType::Borrow));
        let has_mutable_borrow = patterns.iter().any(|p| matches!(p.pattern_type, OwnershipPatternType::MutableBorrow));

        assert!(has_move);
        assert!(has_borrow);
        assert!(has_mutable_borrow);
    }

    #[test]
    fn test_language_analyzer_trait() {
        let analyzer = RustAnalyzer::new();
        let code = r#"
mod utils;

struct User {
    name: String,
}

trait Display {
    fn fmt(&self) -> String;
}

fn main() {
    println!("Hello");
}

fn helper() {
    // helper function
}
"#;

        let features = analyzer.analyze_features(code);

        // 应该包含：1个模块，2个函数，1个结构体，1个trait
        assert!(features.len() >= 4);

        // 验证包含正确的特征类型
        let has_module = features.iter().any(|f| matches!(f, LanguageFeature::Module(_)));
        let has_function = features.iter().any(|f| matches!(f, LanguageFeature::Function(_)));
        let has_struct = features.iter().any(|f| matches!(f, LanguageFeature::Struct(_)));
        let has_trait = features.iter().any(|f| matches!(f, LanguageFeature::Interface(_)));

        assert!(has_module);
        assert!(has_function);
        assert!(has_struct);
        assert!(has_trait);
    }

    #[test]
    fn test_smart_pointer_detection() {
        let analyzer = RustAnalyzer::new();
        let code = r#"
use std::rc::Rc;
use std::sync::Arc;
use std::cell::RefCell;

fn smart_pointers() {
    let boxed = Box::new(42);
    let rc = Rc::new(42);
    let arc = Arc::new(42);
    let refcell = RefCell::new(42);
}
"#;

        let patterns = analyzer.detect_rust_patterns(code);
        assert!(patterns.has_smart_pointers);
    }

    #[test]
    fn test_generic_detection() {
        let analyzer = RustAnalyzer::new();
        let code = r#"
fn generic_function<T: Clone>(value: T) -> T {
    value.clone()
}

struct GenericStruct<T> {
    data: T,
}
"#;

        let patterns = analyzer.detect_rust_patterns(code);
        assert!(patterns.has_generics);
    }
}