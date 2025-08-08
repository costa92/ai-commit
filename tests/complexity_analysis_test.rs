use ai_commit::analysis::complexity::{
    ComplexityAnalyzer, ComplexityConfig, RiskLevel, ComplexityIssueType
};
use ai_commit::languages::Language;

#[tokio::test]
async fn test_complexity_analyzer_creation() {
    let analyzer = ComplexityAnalyzer::new();
    // Should create without panicking
    assert!(true);
}

#[tokio::test]
async fn test_complexity_analyzer_with_custom_config() {
    let config = ComplexityConfig {
        cyclomatic_threshold_warning: 5,
        cyclomatic_threshold_error: 10,
        cognitive_threshold_warning: 8,
        cognitive_threshold_error: 15,
        function_length_threshold_warning: 30,
        function_length_threshold_error: 60,
        nesting_depth_threshold_warning: 3,
        nesting_depth_threshold_error: 5,
        include_comments_in_length: false,
        include_blank_lines_in_length: false,
    };

    let analyzer = ComplexityAnalyzer::with_config(config);
    // Should create without panicking
    assert!(true);
}

#[tokio::test]
async fn test_analyze_simple_rust_function() {
    let analyzer = ComplexityAnalyzer::new();
    let code = r#"
fn simple_function() {
    println!("Hello, world!");
}
"#;

    let result = analyzer.analyze_file("test.rs", code, &Language::Rust);
    assert!(result.is_ok());

    let complexity_result = result.unwrap();
    assert_eq!(complexity_result.file_path, "test.rs");
    assert_eq!(complexity_result.language, Language::Rust);
    assert!(!complexity_result.functions.is_empty());

    let function = &complexity_result.functions[0];
    assert_eq!(function.name, "simple_function");
    assert!(function.cyclomatic_complexity >= 1);
    assert_eq!(function.risk_level, RiskLevel::Low);
}

#[tokio::test]
async fn test_analyze_complex_rust_function() {
    let analyzer = ComplexityAnalyzer::new();
    let code = r#"
fn complex_function(x: i32, y: i32) -> i32 {
    if x > 0 {
        if y > 0 {
            if x > y {
                for i in 0..x {
                    if i % 2 == 0 {
                        println!("Even: {}", i);
                    } else {
                        println!("Odd: {}", i);
                    }
                }
                return x + y;
            } else {
                while y > 0 {
                    y -= 1;
                    if y == 5 {
                        break;
                    }
                }
                return y;
            }
        } else {
            match x {
                1 => return 1,
                2 => return 2,
                3 => return 3,
                _ => return 0,
            }
        }
    } else {
        return -1;
    }
}
"#;

    let result = analyzer.analyze_file("test.rs", code, &Language::Rust);
    assert!(result.is_ok());

    let complexity_result = result.unwrap();
    let function = &complexity_result.functions[0];
    assert_eq!(function.name, "complex_function");
    assert!(function.cyclomatic_complexity > 5);
    assert!(function.cognitive_complexity > 5);
    assert!(function.max_nesting_depth > 2);
    assert!(matches!(function.risk_level, RiskLevel::Medium | RiskLevel::High | RiskLevel::Critical));
    assert!(!function.issues.is_empty());
}

#[tokio::test]
async fn test_analyze_go_function() {
    let analyzer = ComplexityAnalyzer::new();
    let code = r#"
func processData(data []int) int {
    result := 0
    for _, value := range data {
        if value > 0 {
            if value%2 == 0 {
                result += value * 2
            } else {
                result += value
            }
        }
    }
    return result
}
"#;

    let result = analyzer.analyze_file("test.go", code, &Language::Go);
    assert!(result.is_ok());

    let complexity_result = result.unwrap();
    assert_eq!(complexity_result.language, Language::Go);
    assert!(!complexity_result.functions.is_empty());

    let function = &complexity_result.functions[0];
    assert_eq!(function.name, "processData");
    assert!(function.cyclomatic_complexity > 1);
}

#[tokio::test]
async fn test_analyze_typescript_function() {
    let analyzer = ComplexityAnalyzer::new();
    let code = r#"
function calculateScore(items: number[]): number {
    let score = 0;
    for (const item of items) {
        if (item > 0) {
            score += item > 10 ? item * 2 : item;
        } else if (item < 0) {
            score -= Math.abs(item);
        }
    }
    return score;
}
"#;

    let result = analyzer.analyze_file("test.ts", code, &Language::TypeScript);
    assert!(result.is_ok());

    let complexity_result = result.unwrap();
    assert_eq!(complexity_result.language, Language::TypeScript);
    assert!(!complexity_result.functions.is_empty());

    let function = &complexity_result.functions[0];
    assert_eq!(function.name, "calculateScore");
    assert!(function.cyclomatic_complexity > 1);
}

#[tokio::test]
async fn test_complexity_hotspots_identification() {
    let analyzer = ComplexityAnalyzer::new();
    let code = r#"
fn low_complexity() {
    println!("Simple");
}

fn high_complexity(x: i32) -> i32 {
    if x > 0 {
        if x > 10 {
            if x > 20 {
                if x > 30 {
                    if x > 40 {
                        return x * 5;
                    } else {
                        return x * 4;
                    }
                } else {
                    return x * 3;
                }
            } else {
                return x * 2;
            }
        } else {
            return x;
        }
    } else {
        return 0;
    }
}
"#;

    let result = analyzer.analyze_file("test.rs", code, &Language::Rust);
    assert!(result.is_ok());

    let complexity_result = result.unwrap();
    assert_eq!(complexity_result.functions.len(), 2);

    // Should identify the high complexity function as a hotspot
    assert!(!complexity_result.hotspots.is_empty());
    let hotspot = &complexity_result.hotspots[0];
    assert_eq!(hotspot.function_name, "high_complexity");
    assert!(hotspot.score > 50.0);
}

#[tokio::test]
async fn test_refactoring_recommendations() {
    let analyzer = ComplexityAnalyzer::new();
    let code = r#"
fn needs_refactoring(data: Vec<i32>) -> Vec<i32> {
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
"#;

    let result = analyzer.analyze_file("test.rs", code, &Language::Rust);
    assert!(result.is_ok());

    let complexity_result = result.unwrap();
    assert!(!complexity_result.recommendations.is_empty());

    let recommendation = &complexity_result.recommendations[0];
    assert_eq!(recommendation.function_name, "needs_refactoring");
    assert!(recommendation.priority > 0);
}

#[tokio::test]
async fn test_overall_metrics_calculation() {
    let analyzer = ComplexityAnalyzer::new();
    let code = r#"
fn simple1() {
    println!("1");
}

fn simple2() {
    println!("2");
}

fn complex() {
    if true {
        if true {
            if true {
                println!("nested");
            }
        }
    }
}
"#;

    let result = analyzer.analyze_file("test.rs", code, &Language::Rust);
    assert!(result.is_ok());

    let complexity_result = result.unwrap();
    let metrics = &complexity_result.overall_metrics;

    assert_eq!(metrics.total_functions, 3);
    assert!(metrics.average_cyclomatic_complexity > 0.0);
    assert!(metrics.complexity_score >= 0.0);
    assert!(metrics.max_cyclomatic_complexity >= metrics.average_cyclomatic_complexity as u32);
}

#[tokio::test]
async fn test_complexity_issues_detection() {
    let config = ComplexityConfig {
        cyclomatic_threshold_warning: 2,
        cyclomatic_threshold_error: 5,
        cognitive_threshold_warning: 3,
        cognitive_threshold_error: 8,
        function_length_threshold_warning: 10,
        function_length_threshold_error: 20,
        nesting_depth_threshold_warning: 2,
        nesting_depth_threshold_error: 4,
        include_comments_in_length: false,
        include_blank_lines_in_length: false,
    };

    let analyzer = ComplexityAnalyzer::with_config(config);
    let code = r#"
fn problematic_function() {
    if true {
        if true {
            if true {
                if true {
                    println!("Too deep");
                    println!("Line 1");
                    println!("Line 2");
                    println!("Line 3");
                    println!("Line 4");
                    println!("Line 5");
                    println!("Line 6");
                    println!("Line 7");
                    println!("Line 8");
                    println!("Line 9");
                    println!("Line 10");
                    println!("Line 11");
                    println!("Line 12");
                }
            }
        }
    }
}
"#;

    let result = analyzer.analyze_file("test.rs", code, &Language::Rust);
    assert!(result.is_ok());

    let complexity_result = result.unwrap();
    let function = &complexity_result.functions[0];

    // Should detect multiple issues
    assert!(!function.issues.is_empty());

    // Check for specific issue types
    let has_nesting_issue = function.issues.iter()
        .any(|issue| matches!(issue.issue_type, ComplexityIssueType::DeepNesting));
    let has_length_issue = function.issues.iter()
        .any(|issue| matches!(issue.issue_type, ComplexityIssueType::LongFunction));

    assert!(has_nesting_issue);
    assert!(has_length_issue);
}

#[tokio::test]
async fn test_empty_file_analysis() {
    let analyzer = ComplexityAnalyzer::new();
    let code = "";

    let result = analyzer.analyze_file("empty.rs", code, &Language::Rust);
    assert!(result.is_ok());

    let complexity_result = result.unwrap();
    assert!(complexity_result.functions.is_empty());
    assert_eq!(complexity_result.overall_metrics.total_functions, 0);
    assert!(complexity_result.hotspots.is_empty());
    assert!(complexity_result.recommendations.is_empty());
}

#[tokio::test]
async fn test_comments_and_blank_lines_handling() {
    let config = ComplexityConfig {
        include_comments_in_length: true,
        include_blank_lines_in_length: true,
        ..Default::default()
    };

    let analyzer = ComplexityAnalyzer::with_config(config);
    let code = r#"
fn test_function() {
    // This is a comment
    println!("Hello");

    // Another comment
    println!("World");

    /* Block comment */
    println!("End");
}
"#;

    let result = analyzer.analyze_file("test.rs", code, &Language::Rust);
    assert!(result.is_ok());

    let complexity_result = result.unwrap();
    let function = &complexity_result.functions[0];

    // Should include comments and blank lines in length calculation
    assert!(function.function_length > 3);
}