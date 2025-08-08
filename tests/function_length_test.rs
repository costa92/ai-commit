use ai_commit::analysis::complexity::{FunctionLengthAnalyzer, FunctionInfo, ComplexityConfig};

fn create_test_function(name: &str, content: &str) -> FunctionInfo {
    FunctionInfo {
        name: name.to_string(),
        line_start: 1,
        line_end: content.lines().count() as u32,
        content: content.to_string(),
    }
}

#[tokio::test]
async fn test_function_length_analyzer_creation() {
    let analyzer = FunctionLengthAnalyzer::new();
    // Should create without panicking
    assert!(true);
}

#[tokio::test]
async fn test_simple_function_length() {
    let analyzer = FunctionLengthAnalyzer::new();
    let config = ComplexityConfig::default();
    let function = create_test_function("simple", r#"
fn simple() {
    println!("Hello, world!");
}
"#);

    let result = analyzer.analyze(&function, &config);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 4); // 4 lines including function declaration and braces
}

#[tokio::test]
async fn test_function_length_with_comments_excluded() {
    let analyzer = FunctionLengthAnalyzer::new();
    let config = ComplexityConfig {
        include_comments_in_length: false,
        include_blank_lines_in_length: true,
        ..Default::default()
    };

    let function = create_test_function("with_comments", r#"
fn with_comments() {
    // This is a comment
    println!("Hello");

    /* Block comment */
    println!("World");
}
"#);

    let result = analyzer.analyze(&function, &config);
    assert!(result.is_ok());
    // Should exclude comment lines but include blank lines
    assert_eq!(result.unwrap(), 5); // function declaration, 2 println, 1 blank line, closing brace
}

#[tokio::test]
async fn test_function_length_with_comments_included() {
    let analyzer = FunctionLengthAnalyzer::new();
    let config = ComplexityConfig {
        include_comments_in_length: true,
        include_blank_lines_in_length: true,
        ..Default::default()
    };

    let function = create_test_function("with_comments", r#"
fn with_comments() {
    // This is a comment
    println!("Hello");

    /* Block comment */
    println!("World");
}
"#);

    let result = analyzer.analyze(&function, &config);
    assert!(result.is_ok());
    // Should include all lines
    assert_eq!(result.unwrap(), 7); // All 7 lines
}

#[tokio::test]
async fn test_function_length_with_blank_lines_excluded() {
    let analyzer = FunctionLengthAnalyzer::new();
    let config = ComplexityConfig {
        include_comments_in_length: false,
        include_blank_lines_in_length: false,
        ..Default::default()
    };

    let function = create_test_function("with_blank_lines", r#"
fn with_blank_lines() {
    println!("Line 1");

    println!("Line 2");

    println!("Line 3");
}
"#);

    let result = analyzer.analyze(&function, &config);
    assert!(result.is_ok());
    // Should exclude blank lines: function declaration + 3 println + closing brace = 5
    assert_eq!(result.unwrap(), 5);
}

#[tokio::test]
async fn test_function_length_with_blank_lines_included() {
    let analyzer = FunctionLengthAnalyzer::new();
    let config = ComplexityConfig {
        include_comments_in_length: false,
        include_blank_lines_in_length: true,
        ..Default::default()
    };

    let function = create_test_function("with_blank_lines", r#"
fn with_blank_lines() {
    println!("Line 1");

    println!("Line 2");

    println!("Line 3");
}
"#);

    let result = analyzer.analyze(&function, &config);
    assert!(result.is_ok());
    // Should include blank lines: function declaration + 3 println + 2 blank lines + closing brace = 7
    assert_eq!(result.unwrap(), 7);
}

#[tokio::test]
async fn test_long_function_length() {
    let analyzer = FunctionLengthAnalyzer::new();
    let config = ComplexityConfig::default();

    let mut function_content = String::from("fn long_function() {\n");
    for i in 1..=50 {
        function_content.push_str(&format!("    println!(\"Line {}\");\n", i));
    }
    function_content.push_str("}");

    let function = create_test_function("long_function", &function_content);

    let result = analyzer.analyze(&function, &config);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 52); // 1 declaration + 50 println + 1 closing brace
}

#[tokio::test]
async fn test_empty_function_length() {
    let analyzer = FunctionLengthAnalyzer::new();
    let config = ComplexityConfig::default();
    let function = create_test_function("empty", "");

    let result = analyzer.analyze(&function, &config);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 0); // Empty function
}

#[tokio::test]
async fn test_function_with_only_comments() {
    let analyzer = FunctionLengthAnalyzer::new();
    let config = ComplexityConfig {
        include_comments_in_length: false,
        include_blank_lines_in_length: false,
        ..Default::default()
    };

    let function = create_test_function("only_comments", r#"
// This is a comment
/* This is another comment */
// More comments
"#);

    let result = analyzer.analyze(&function, &config);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 0); // No non-comment lines
}

#[tokio::test]
async fn test_function_with_only_blank_lines() {
    let analyzer = FunctionLengthAnalyzer::new();
    let config = ComplexityConfig {
        include_comments_in_length: false,
        include_blank_lines_in_length: false,
        ..Default::default()
    };

    let function = create_test_function("only_blank_lines", r#"



"#);

    let result = analyzer.analyze(&function, &config);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 0); // No non-blank lines
}

#[tokio::test]
async fn test_mixed_comment_types() {
    let analyzer = FunctionLengthAnalyzer::new();
    let config = ComplexityConfig {
        include_comments_in_length: false,
        include_blank_lines_in_length: true,
        ..Default::default()
    };

    let function = create_test_function("mixed_comments", r#"
fn mixed_comments() {
    // Single line comment
    println!("Code line 1");
    /* Block comment */
    println!("Code line 2");
    /*
     * Multi-line
     * block comment
     */
    println!("Code line 3");

    // Another comment
}
"#);

    let result = analyzer.analyze(&function, &config);
    assert!(result.is_ok());
    // Should count: function declaration, 3 println statements, 1 blank line, closing brace = 6
    assert_eq!(result.unwrap(), 6);
}