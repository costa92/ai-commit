use ai_commit::analysis::complexity::{CyclomaticComplexityCalculator, FunctionInfo};
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
async fn test_cyclomatic_complexity_calculator_creation() {
    let calculator = CyclomaticComplexityCalculator::new();
    // Should create without panicking
    assert!(true);
}

#[tokio::test]
async fn test_simple_rust_function_complexity() {
    let calculator = CyclomaticComplexityCalculator::new();
    let function = create_test_function("simple", r#"
fn simple() {
    println!("Hello, world!");
}
"#);

    let result = calculator.calculate(&function, &Language::Rust);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 1); // Base complexity
}

#[tokio::test]
async fn test_rust_if_statement_complexity() {
    let calculator = CyclomaticComplexityCalculator::new();
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
    assert_eq!(result.unwrap(), 2); // Base + if
}

#[tokio::test]
async fn test_rust_if_else_complexity() {
    let calculator = CyclomaticComplexityCalculator::new();
    let function = create_test_function("with_if_else", r#"
fn with_if_else(x: i32) -> i32 {
    if x > 0 {
        return x;
    } else if x < 0 {
        return -x;
    } else {
        return 0;
    }
}
"#);

    let result = calculator.calculate(&function, &Language::Rust);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 3); // Base + if + else if
}

#[tokio::test]
async fn test_rust_match_complexity() {
    let calculator = CyclomaticComplexityCalculator::new();
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
    // Base + match + 4 arms (=>)
    assert!(result.unwrap() >= 5);
}

#[tokio::test]
async fn test_rust_loop_complexity() {
    let calculator = CyclomaticComplexityCalculator::new();
    let function = create_test_function("with_loops", r#"
fn with_loops() {
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
    assert_eq!(result.unwrap(), 4); // Base + for + while + loop
}

#[tokio::test]
async fn test_rust_logical_operators_complexity() {
    let calculator = CyclomaticComplexityCalculator::new();
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
    // Base + if + && + ||
    assert_eq!(result.unwrap(), 4);
}

#[tokio::test]
async fn test_rust_question_mark_operator() {
    let calculator = CyclomaticComplexityCalculator::new();
    let function = create_test_function("with_question_mark", r#"
fn with_question_mark() -> Result<i32, String> {
    let x = some_function()?;
    let y = another_function()?;
    Ok(x + y)
}
"#);

    let result = calculator.calculate(&function, &Language::Rust);
    assert!(result.is_ok());
    // Base + two ? operators
    assert_eq!(result.unwrap(), 3);
}

#[tokio::test]
async fn test_go_function_complexity() {
    let calculator = CyclomaticComplexityCalculator::new();
    let function = create_test_function("go_func", r#"
func processData(data []int) int {
    result := 0
    for _, value := range data {
        if value > 0 {
            result += value
        } else if value < 0 {
            result -= value
        }
    }
    return result
}
"#);

    let result = calculator.calculate(&function, &Language::Go);
    assert!(result.is_ok());
    // Base + for + if + else if
    assert_eq!(result.unwrap(), 4);
}

#[tokio::test]
async fn test_go_switch_complexity() {
    let calculator = CyclomaticComplexityCalculator::new();
    let function = create_test_function("go_switch", r#"
func handleValue(x int) string {
    switch x {
    case 1:
        return "one"
    case 2:
        return "two"
    case 3:
        return "three"
    default:
        return "other"
    }
}
"#);

    let result = calculator.calculate(&function, &Language::Go);
    assert!(result.is_ok());
    // Base + switch + 4 cases
    assert!(result.unwrap() >= 5);
}

#[tokio::test]
async fn test_go_logical_operators() {
    let calculator = CyclomaticComplexityCalculator::new();
    let function = create_test_function("go_logical", r#"
func checkConditions(a, b, c bool) bool {
    if a && b || c {
        return true
    }
    return false
}
"#);

    let result = calculator.calculate(&function, &Language::Go);
    assert!(result.is_ok());
    // Base + if + && + ||
    assert_eq!(result.unwrap(), 4);
}

#[tokio::test]
async fn test_typescript_function_complexity() {
    let calculator = CyclomaticComplexityCalculator::new();
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
    // Base + for + if + else if
    assert_eq!(result.unwrap(), 4);
}

#[tokio::test]
async fn test_typescript_switch_complexity() {
    let calculator = CyclomaticComplexityCalculator::new();
    let function = create_test_function("ts_switch", r#"
function getGrade(score: number): string {
    switch (true) {
        case score >= 90:
            return 'A';
        case score >= 80:
            return 'B';
        case score >= 70:
            return 'C';
        default:
            return 'F';
    }
}
"#);

    let result = calculator.calculate(&function, &Language::TypeScript);
    assert!(result.is_ok());
    // Base + switch + 4 cases
    assert!(result.unwrap() >= 5);
}

#[tokio::test]
async fn test_typescript_ternary_operator() {
    let calculator = CyclomaticComplexityCalculator::new();
    let function = create_test_function("ts_ternary", r#"
function processValue(x: number): number {
    return x > 0 ? x * 2 : x < 0 ? x * -1 : 0;
}
"#);

    let result = calculator.calculate(&function, &Language::TypeScript);
    assert!(result.is_ok());
    // Base + two ternary operators
    assert_eq!(result.unwrap(), 3);
}

#[tokio::test]
async fn test_typescript_try_catch_complexity() {
    let calculator = CyclomaticComplexityCalculator::new();
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

    let result = calculator.calculate(&function, &Language::TypeScript);
    assert!(result.is_ok());
    // Base + try + catch
    assert_eq!(result.unwrap(), 3);
}

#[tokio::test]
async fn test_complex_nested_function() {
    let calculator = CyclomaticComplexityCalculator::new();
    let function = create_test_function("complex_nested", r#"
fn complex_nested(data: Vec<i32>) -> Vec<i32> {
    let mut result = Vec::new();

    for item in data {
        if item > 0 {
            if item % 2 == 0 {
                if item > 10 {
                    result.push(item * 2);
                } else {
                    result.push(item);
                }
            } else {
                match item {
                    1 => result.push(1),
                    3 => result.push(9),
                    5 => result.push(25),
                    _ => result.push(item * item),
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

    // Should be quite high due to nested conditions and match
    assert!(complexity > 10);
}

#[tokio::test]
async fn test_generic_language_complexity() {
    let calculator = CyclomaticComplexityCalculator::new();
    let function = create_test_function("generic_func", r#"
function unknown_language() {
    if (condition) {
        while (loop_condition) {
            for (item in items) {
                switch (item) {
                    case 1:
                        break;
                    case 2:
                        break;
                }
            }
        }
    }
}
"#);

    let result = calculator.calculate(&function, &Language::Unknown);
    assert!(result.is_ok());
    let complexity = result.unwrap();

    // Should detect basic control structures
    assert!(complexity > 5);
}

#[tokio::test]
async fn test_comments_ignored_in_complexity() {
    let calculator = CyclomaticComplexityCalculator::new();
    let function = create_test_function("with_comments", r#"
fn with_comments(x: i32) -> i32 {
    // This comment contains if, while, for keywords
    // but should not affect complexity
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
    assert_eq!(result.unwrap(), 2);
}

#[tokio::test]
async fn test_empty_function_complexity() {
    let calculator = CyclomaticComplexityCalculator::new();
    let function = create_test_function("empty", "");

    let result = calculator.calculate(&function, &Language::Rust);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 1); // Base complexity
}

#[tokio::test]
async fn test_function_with_only_comments() {
    let calculator = CyclomaticComplexityCalculator::new();
    let function = create_test_function("only_comments", r#"
// This is a comment
/* This is another comment */
// More comments
"#);

    let result = calculator.calculate(&function, &Language::Rust);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 1); // Base complexity only
}