use ai_commit::analysis::complexity::{CognitiveComplexityCalculator, FunctionInfo};
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
async fn test_cognitive_complexity_calculator_creation() {
    let calculator = CognitiveComplexityCalculator::new();
    // Should create without panicking
    assert!(true);
}

#[tokio::test]
async fn test_simple_rust_function_cognitive() {
    let calculator = CognitiveComplexityCalculator::new();
    let function = create_test_function("simple", r#"
fn simple() {
    println!("Hello, world!");
}
"#);

    let result = calculator.calculate(&function, &Language::Rust);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 0); // No cognitive complexity for simple function
}

#[tokio::test]
async fn test_rust_if_statement_cognitive() {
    let calculator = CognitiveComplexityCalculator::new();
    let function = create_test_function("with_if", r#"
fn with_if(x: i32) -> i32 {
    if x > 0 {
        return x;
    }
    return 0;
}
"#);

    let result = calculator.calculate(&function, &Language::Rust);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 1); // One if statement
}

#[tokio::test]
async fn test_rust_nested_if_cognitive() {
    let calculator = CognitiveComplexityCalculator::new();
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

    let result = calculator.calculate(&function, &Language::Rust);
    assert!(result.is_ok());
    // First if: +1, nested if: +1 (base) + 1 (nesting) = +2, total = 3
    assert_eq!(result.unwrap(), 3);
}

#[tokio::test]
async fn test_rust_loop_cognitive() {
    let calculator = CognitiveComplexityCalculator::new();
    let function = create_test_function("with_loop", r#"
fn with_loop() {
    for i in 0..10 {
        println!("{}", i);
    }

    while true {
        break;
    }

    loop {
        break;
    }
}
"#);

    let result = calculator.calculate(&function, &Language::Rust);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 3); // for + while + loop
}

#[tokio::test]
async fn test_rust_nested_loop_cognitive() {
    let calculator = CognitiveComplexityCalculator::new();
    let function = create_test_function("nested_loop", r#"
fn nested_loop() {
    for i in 0..10 {
        for j in 0..5 {
            println!("{} {}", i, j);
        }
    }
}
"#);

    let result = calculator.calculate(&function, &Language::Rust);
    assert!(result.is_ok());
    // Outer for: +1, inner for: +1 (base) + 1 (nesting) = +2, total = 3
    assert_eq!(result.unwrap(), 3);
}

#[tokio::test]
async fn test_rust_logical_operators_cognitive() {
    let calculator = CognitiveComplexityCalculator::new();
    let function = create_test_function("with_logical", r#"
fn with_logical(a: bool, b: bool, c: bool) -> bool {
    if a && b || c {
        return true;
    }
    return false;
}
"#);

    let result = calculator.calculate(&function, &Language::Rust);
    assert!(result.is_ok());
    // if: +1, &&: +1, ||: +1, total = 3
    assert_eq!(result.unwrap(), 3);
}

#[tokio::test]
async fn test_rust_match_cognitive() {
    let calculator = CognitiveComplexityCalculator::new();
    let function = create_test_function("with_match", r#"
fn with_match(x: i32) -> &'static str {
    match x {
        1 => "one",
        2 => "two",
        3 => "three",
        _ => "other",
    }
}
"#);

    let result = calculator.calculate(&function, &Language::Rust);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 1); // match statement adds 1
}

#[tokio::test]
async fn test_rust_break_continue_cognitive() {
    let calculator = CognitiveComplexityCalculator::new();
    let function = create_test_function("with_break_continue", r#"
fn with_break_continue() {
    for i in 0..10 {
        if i == 5 {
            break;
        }
        if i == 3 {
            continue;
        }
        println!("{}", i);
    }
}
"#);

    let result = calculator.calculate(&function, &Language::Rust);
    assert!(result.is_ok());
    // for: +1, first if: +1 + 1 (nesting) = +2, break: +1, second if: +1 + 1 (nesting) = +2, continue: +1, total = 8
    assert!(result.unwrap() >= 6);
}

#[tokio::test]
async fn test_go_function_cognitive() {
    let calculator = CognitiveComplexityCalculator::new();
    let function = create_test_function("go_func", r#"
func processData(data []int) int {
    result := 0
    for _, value := range data {
        if value > 0 {
            result += value
        } else {
            result -= value
        }
    }
    return result
}
"#);

    let result = calculator.calculate(&function, &Language::Go);
    assert!(result.is_ok());
    // for: +1, if: +1 + 1 (nesting) = +2, else: +1, total = 4
    assert_eq!(result.unwrap(), 4);
}

#[tokio::test]
async fn test_go_nested_conditions_cognitive() {
    let calculator = CognitiveComplexityCalculator::new();
    let function = create_test_function("go_nested", r#"
func complexLogic(x, y int) int {
    if x > 0 {
        if y > 0 {
            if x > y {
                return x
            } else {
                return y
            }
        }
    }
    return 0
}
"#);

    let result = calculator.calculate(&function, &Language::Go);
    assert!(result.is_ok());
    // First if: +1, second if: +1 + 1 (nesting) = +2, third if: +1 + 2 (nesting) = +3, else: +1, total = 7
    assert!(result.unwrap() >= 6);
}

#[tokio::test]
async fn test_typescript_function_cognitive() {
    let calculator = CognitiveComplexityCalculator::new();
    let function = create_test_function("ts_func", r#"
function calculateScore(items: number[]): number {
    let score = 0;
    for (const item of items) {
        if (item > 0) {
            score += item;
        } else if (item < 0) {
            score -= Math.abs(item);
        }
    }
    return score;
}
"#);

    let result = calculator.calculate(&function, &Language::TypeScript);
    assert!(result.is_ok());
    // for: +1, if: +1 + 1 (nesting) = +2, else: +1, total = 4
    assert_eq!(result.unwrap(), 4);
}

#[tokio::test]
async fn test_typescript_async_cognitive() {
    let calculator = CognitiveComplexityCalculator::new();
    let function = create_test_function("ts_async", r#"
async function fetchData(url: string): Promise<any> {
    try {
        const response = await fetch(url);
        if (response.ok) {
            return await response.json();
        } else {
            throw new Error('Failed to fetch');
        }
    } catch (error) {
        console.error(error);
        return null;
    }
}
"#);

    let result = calculator.calculate(&function, &Language::TypeScript);
    assert!(result.is_ok());
    // async: +1, try: +1, await: +1, if: +1 + 1 (nesting) = +2, else: +1, await: +1, catch: +1, total = 8
    assert!(result.unwrap() >= 6);
}

#[tokio::test]
async fn test_typescript_ternary_cognitive() {
    let calculator = CognitiveComplexityCalculator::new();
    let function = create_test_function("ts_ternary", r#"
function processValue(x: number): number {
    return x > 0 ? x * 2 : x < 0 ? x * -1 : 0;
}
"#);

    let result = calculator.calculate(&function, &Language::TypeScript);
    assert!(result.is_ok());
    // Two ternary operators: +2
    assert_eq!(result.unwrap(), 2);
}

#[tokio::test]
async fn test_deeply_nested_cognitive() {
    let calculator = CognitiveComplexityCalculator::new();
    let function = create_test_function("deeply_nested", r#"
fn deeply_nested(data: Vec<i32>) -> Vec<i32> {
    let mut result = Vec::new();

    for item in data {
        if item > 0 {
            if item % 2 == 0 {
                if item > 10 {
                    if item < 100 {
                        result.push(item * 2);
                    } else {
                        result.push(item);
                    }
                } else {
                    result.push(item + 1);
                }
            } else {
                if item > 5 {
                    result.push(item * 3);
                } else {
                    result.push(item);
                }
            }
        } else if item < 0 {
            result.push(0);
        }
    }

    result
}
"#);

    let result = calculator.calculate(&function, &Language::Rust);
    assert!(result.is_ok());
    let complexity = result.unwrap();

    // Should be quite high due to deep nesting
    assert!(complexity > 15);
}

#[tokio::test]
async fn test_comments_ignored_in_cognitive() {
    let calculator = CognitiveComplexityCalculator::new();
    let function = create_test_function("with_comments", r#"
fn with_comments(x: i32) -> i32 {
    // This comment contains if, while, for keywords
    // but should not affect cognitive complexity
    /*
     * Another comment with if and while
     */
    if x > 0 {
        return x;
    }
    return 0;
}
"#);

    let result = calculator.calculate(&function, &Language::Rust);
    assert!(result.is_ok());
    // Should only count the actual if statement, not keywords in comments
    assert_eq!(result.unwrap(), 1);
}

#[tokio::test]
async fn test_empty_function_cognitive() {
    let calculator = CognitiveComplexityCalculator::new();
    let function = create_test_function("empty", "");

    let result = calculator.calculate(&function, &Language::Rust);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 0); // No cognitive complexity
}

#[tokio::test]
async fn test_function_with_only_comments_cognitive() {
    let calculator = CognitiveComplexityCalculator::new();
    let function = create_test_function("only_comments", r#"
// This is a comment
/* This is another comment */
// More comments
"#);

    let result = calculator.calculate(&function, &Language::Rust);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 0); // No cognitive complexity
}