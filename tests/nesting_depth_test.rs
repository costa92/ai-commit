use ai_commit::analysis::complexity::{NestingDepthAnalyzer, FunctionInfo};
use ai_commit::languages::Language;

fn create_test_function(name: &str, content: &str) -> FunctionInfo {
    FunctionInfo {
        name: name.to_string(),
        line_start: 1,
        line_end: content.lines().count() as u32,
        content: content.to_string(),
    }
}

#[tokio::test]
async fn test_nesting_depth_analyzer_creation() {
    let analyzer = NestingDepthAnalyzer::new();
    // Should create without panicking
    assert!(true);
}

#[tokio::test]
async fn test_simple_rust_function_nesting() {
    let analyzer = NestingDepthAnalyzer::new();
    let function = create_test_function("simple", r#"
fn simple() {
    println!("Hello, world!");
}
"#);

    let result = analyzer.analyze(&function, &Language::Rust);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 1); // One level of nesting (function body)
}

#[tokio::test]
async fn test_rust_if_statement_nesting() {
    let analyzer = NestingDepthAnalyzer::new();
    let function = create_test_function("with_if", r#"
fn with_if(x: i32) -> i32 {
    if x > 0 {
        return x;
    }
    return 0;
}
"#);

    let result = analyzer.analyze(&function, &Language::Rust);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 2); // Function body + if block
}

#[tokio::test]
async fn test_rust_nested_if_nesting() {
    let analyzer = NestingDepthAnalyzer::new();
    let function = create_test_function("nested_if", r#"
fn nested_if(x: i32, y: i32) -> i32 {
    if x > 0 {
        if y > 0 {
            return x + y;
        }
        return x;
    }
    return 0;
}
"#);

    let result = analyzer.analyze(&function, &Language::Rust);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 3); // Function body + outer if + inner if
}

#[tokio::test]
async fn test_rust_deeply_nested_nesting() {
    let analyzer = NestingDepthAnalyzer::new();
    let function = create_test_function("deeply_nested", r#"
fn deeply_nested(a: i32, b: i32, c: i32, d: i32) -> i32 {
    if a > 0 {
        if b > 0 {
            if c > 0 {
                if d > 0 {
                    return a + b + c + d;
                }
                return a + b + c;
            }
            return a + b;
        }
        return a;
    }
    return 0;
}
"#);

    let result = analyzer.analyze(&function, &Language::Rust);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 5); // Function body + 4 nested if statements
}

#[tokio::test]
async fn test_rust_loop_nesting() {
    let analyzer = NestingDepthAnalyzer::new();
    let function = create_test_function("with_loop", r#"
fn with_loop() {
    for i in 0..10 {
        println!("{}", i);
    }
}
"#);

    let result = analyzer.analyze(&function, &Language::Rust);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 2); // Function body + for loop
}

#[tokio::test]
async fn test_rust_nested_loops_nesting() {
    let analyzer = NestingDepthAnalyzer::new();
    let function = create_test_function("nested_loops", r#"
fn nested_loops() {
    for i in 0..10 {
        for j in 0..5 {
            for k in 0..3 {
                println!("{} {} {}", i, j, k);
            }
        }
    }
}
"#);

    let result = analyzer.analyze(&function, &Language::Rust);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 4); // Function body + 3 nested for loops
}

#[tokio::test]
async fn test_rust_match_nesting() {
    let analyzer = NestingDepthAnalyzer::new();
    let function = create_test_function("with_match", r#"
fn with_match(x: i32) -> &'static str {
    match x {
        1 => "one",
        2 => "two",
        _ => "other",
    }
}
"#);

    let result = analyzer.analyze(&function, &Language::Rust);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 2); // Function body + match block
}

#[tokio::test]
async fn test_rust_mixed_nesting() {
    let analyzer = NestingDepthAnalyzer::new();
    let function = create_test_function("mixed_nesting", r#"
fn mixed_nesting(data: Vec<i32>) -> Vec<i32> {
    let mut result = Vec::new();

    for item in data {
        if item > 0 {
            match item % 3 {
                0 => result.push(item * 2),
                1 => result.push(item * 3),
                _ => result.push(item),
            }
        }
    }

    result
}
"#);

    let result = analyzer.analyze(&function, &Language::Rust);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 4); // Function body + for + if + match
}

#[tokio::test]
async fn test_go_function_nesting() {
    let analyzer = NestingDepthAnalyzer::new();
    let function = create_test_function("go_func", r#"
func processData(data []int) int {
    result := 0
    for _, value := range data {
        if value > 0 {
            result += value
        }
    }
    return result
}
"#);

    let result = analyzer.analyze(&function, &Language::Go);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 3); // Function body + for + if
}

#[tokio::test]
async fn test_go_switch_nesting() {
    let analyzer = NestingDepthAnalyzer::new();
    let function = create_test_function("go_switch", r#"
func handleValue(x int) string {
    switch x {
    case 1:
        return "one"
    case 2:
        return "two"
    default:
        return "other"
    }
}
"#);

    let result = analyzer.analyze(&function, &Language::Go);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 2); // Function body + switch
}

#[tokio::test]
async fn test_typescript_function_nesting() {
    let analyzer = NestingDepthAnalyzer::new();
    let function = create_test_function("ts_func", r#"
function calculateScore(items: number[]): number {
    let score = 0;
    for (const item of items) {
        if (item > 0) {
            score += item;
        } else {
            score -= Math.abs(item);
        }
    }
    return score;
}
"#);

    let result = analyzer.analyze(&function, &Language::TypeScript);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 3); // Function body + for + if/else
}

#[tokio::test]
async fn test_typescript_try_catch_nesting() {
    let analyzer = NestingDepthAnalyzer::new();
    let function = create_test_function("ts_try_catch", r#"
function riskyOperation(): number {
    try {
        return performOperation();
    } catch (error) {
        console.error(error);
        return 0;
    }
}
"#);

    let result = analyzer.analyze(&function, &Language::TypeScript);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 2); // Function body + try/catch blocks
}

#[tokio::test]
async fn test_typescript_nested_try_catch() {
    let analyzer = NestingDepthAnalyzer::new();
    let function = create_test_function("nested_try_catch", r#"
function complexOperation(): number {
    try {
        const data = fetchData();
        try {
            return processData(data);
        } catch (processError) {
            console.error("Process error:", processError);
            return 0;
        }
    } catch (fetchError) {
        console.error("Fetch error:", fetchError);
        return -1;
    }
}
"#);

    let result = analyzer.analyze(&function, &Language::TypeScript);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 3); // Function body + outer try/catch + inner try/catch
}

#[tokio::test]
async fn test_comments_ignored_in_nesting() {
    let analyzer = NestingDepthAnalyzer::new();
    let function = create_test_function("with_comments", r#"
fn with_comments(x: i32) -> i32 {
    // This comment contains { and } braces
    // but should not affect nesting depth
    /*
     * Another comment with { braces }
     */
    if x > 0 {
        return x;
    }
    return 0;
}
"#);

    let result = analyzer.analyze(&function, &Language::Rust);
    assert!(result.is_ok());
    // Should only count actual code braces, not braces in comments
    assert_eq!(result.unwrap(), 2); // Function body + if block
}

#[tokio::test]
async fn test_empty_function_nesting() {
    let analyzer = NestingDepthAnalyzer::new();
    let function = create_test_function("empty", "");

    let result = analyzer.analyze(&function, &Language::Rust);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 0); // No nesting
}

#[tokio::test]
async fn test_function_with_only_comments_nesting() {
    let analyzer = NestingDepthAnalyzer::new();
    let function = create_test_function("only_comments", r#"
// This is a comment
/* This is another comment */
// More comments
"#);

    let result = analyzer.analyze(&function, &Language::Rust);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 0); // No nesting
}

#[tokio::test]
async fn test_unbalanced_braces_handling() {
    let analyzer = NestingDepthAnalyzer::new();
    let function = create_test_function("unbalanced", r#"
fn unbalanced() {
    if true {
        println!("Missing closing brace");
    // Missing }
}
"#);

    let result = analyzer.analyze(&function, &Language::Rust);
    assert!(result.is_ok());
    // Should handle unbalanced braces gracefully
    assert!(result.unwrap() >= 1);
}

#[tokio::test]
async fn test_generic_language_nesting() {
    let analyzer = NestingDepthAnalyzer::new();
    let function = create_test_function("generic_func", r#"
function unknown_language() {
    if (condition) {
        while (loop_condition) {
            for (item in items) {
                // Some code
            }
        }
    }
}
"#);

    let result = analyzer.analyze(&function, &Language::Unknown);
    assert!(result.is_ok());
    let depth = result.unwrap();

    // Should detect basic nesting structure
    assert!(depth >= 3);
}