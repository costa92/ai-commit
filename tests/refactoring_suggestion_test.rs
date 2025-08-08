use ai_commit::analysis::duplication::{
    DuplicationDetector, DuplicationConfig, RefactoringSuggestionGenerator,
    CodeDuplication, CodeBlock, DuplicationType, RiskLevel, RefactoringPriority,
    SuggestionType, ComplexityLevel
};
use ai_commit::languages::{Language, LanguageDetector};
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::test]
async fn test_refactoring_suggestion_generator_creation() {
    let generator = RefactoringSuggestionGenerator::new();

    // Test that the generator is created successfully
    // This is a basic test to ensure the struct can be instantiated
    assert!(true); // Generator created successfully
}

#[tokio::test]
async fn test_generate_suggestions_for_exact_duplication() {
    let generator = RefactoringSuggestionGenerator::new();

    // Create a sample exact duplication
    let duplication = CodeDuplication {
        id: "test-duplication-1".to_string(),
        duplication_type: DuplicationType::Exact,
        code_blocks: vec![
            CodeBlock {
                file_path: "src/main.rs".to_string(),
                start_line: 10,
                end_line: 15,
                start_column: None,
                end_column: None,
                content_hash: "hash1".to_string(),
            },
            CodeBlock {
                file_path: "src/lib.rs".to_string(),
                start_line: 20,
                end_line: 25,
                start_column: None,
                end_column: None,
                content_hash: "hash1".to_string(),
            },
        ],
        content: "const MAX_SIZE: usize = 1024;\nconst TIMEOUT: u64 = 30;\nconst RETRIES: i32 = 3;".to_string(),
        line_count: 6,
        similarity_score: 1.0,
        risk_level: RiskLevel::Medium,
        refactoring_priority: RefactoringPriority::Medium,
    };

    let duplications = vec![duplication];
    let suggestions = generator.generate_suggestions(&duplications).await.unwrap();

    // Verify that suggestions were generated
    assert!(!suggestions.is_empty(), "Should generate at least one suggestion");

    let suggestion = &suggestions[0];
    assert_eq!(suggestion.duplication_id, "test-duplication-1");
    assert_eq!(suggestion.suggestion_type, SuggestionType::ExtractConstant);
    assert!(suggestion.title.contains("常量"));
    assert!(!suggestion.expected_benefits.is_empty());
    assert!(suggestion.code_example.is_some());
    assert!(!suggestion.resources.is_empty());
}

#[tokio::test]
async fn test_generate_suggestions_for_method_extraction() {
    let generator = RefactoringSuggestionGenerator::new();

    // Create a sample duplication that should suggest method extraction
    let duplication = CodeDuplication {
        id: "test-duplication-2".to_string(),
        duplication_type: DuplicationType::Exact,
        code_blocks: vec![
            CodeBlock {
                file_path: "src/handler.rs".to_string(),
                start_line: 10,
                end_line: 25,
                start_column: None,
                end_column: None,
                content_hash: "hash2".to_string(),
            },
            CodeBlock {
                file_path: "src/processor.rs".to_string(),
                start_line: 30,
                end_line: 45,
                start_column: None,
                end_column: None,
                content_hash: "hash2".to_string(),
            },
        ],
        content: r#"if input.is_empty() {
    return Err(Error::EmptyInput);
}
let processed = input.trim().to_lowercase();
if processed.len() > MAX_LENGTH {
    return Err(Error::TooLong);
}
Ok(processed)"#.to_string(),
        line_count: 15,
        similarity_score: 1.0,
        risk_level: RiskLevel::Medium,
        refactoring_priority: RefactoringPriority::Medium,
    };

    let duplications = vec![duplication];
    let suggestions = generator.generate_suggestions(&duplications).await.unwrap();

    // Verify that suggestions were generated
    assert!(!suggestions.is_empty(), "Should generate at least one suggestion");

    let suggestion = &suggestions[0];
    assert_eq!(suggestion.duplication_id, "test-duplication-2");
    assert_eq!(suggestion.suggestion_type, SuggestionType::ExtractMethod);
    assert!(suggestion.title.contains("函数"));
    assert!(suggestion.implementation_complexity == ComplexityLevel::Simple ||
            suggestion.implementation_complexity == ComplexityLevel::Moderate);
    assert!(!suggestion.expected_benefits.is_empty());
    assert!(suggestion.code_example.is_some());
}

#[tokio::test]
async fn test_generate_suggestions_for_cross_file_duplication() {
    let generator = RefactoringSuggestionGenerator::new();

    // Create multiple cross-file duplications to trigger utility module suggestion
    let duplications = vec![
        CodeDuplication {
            id: "cross-file-1".to_string(),
            duplication_type: DuplicationType::CrossFile,
            code_blocks: vec![
                CodeBlock {
                    file_path: "src/module1.rs".to_string(),
                    start_line: 10,
                    end_line: 20,
                    start_column: None,
                    end_column: None,
                    content_hash: "cross-hash1".to_string(),
                },
                CodeBlock {
                    file_path: "src/module2.rs".to_string(),
                    start_line: 15,
                    end_line: 25,
                    start_column: None,
                    end_column: None,
                    content_hash: "cross-hash1".to_string(),
                },
            ],
            content: "fn validate_input(input: &str) -> bool { !input.is_empty() }".to_string(),
            line_count: 10,
            similarity_score: 1.0,
            risk_level: RiskLevel::Medium,
            refactoring_priority: RefactoringPriority::Medium,
        },
        CodeDuplication {
            id: "cross-file-2".to_string(),
            duplication_type: DuplicationType::CrossFile,
            code_blocks: vec![
                CodeBlock {
                    file_path: "src/module3.rs".to_string(),
                    start_line: 5,
                    end_line: 15,
                    start_column: None,
                    end_column: None,
                    content_hash: "cross-hash2".to_string(),
                },
                CodeBlock {
                    file_path: "src/module4.rs".to_string(),
                    start_line: 20,
                    end_line: 30,
                    start_column: None,
                    end_column: None,
                    content_hash: "cross-hash2".to_string(),
                },
            ],
            content: "fn format_output(data: &str) -> String { data.trim().to_string() }".to_string(),
            line_count: 10,
            similarity_score: 1.0,
            risk_level: RiskLevel::Medium,
            refactoring_priority: RefactoringPriority::Medium,
        },
        CodeDuplication {
            id: "cross-file-3".to_string(),
            duplication_type: DuplicationType::CrossFile,
            code_blocks: vec![
                CodeBlock {
                    file_path: "src/module5.rs".to_string(),
                    start_line: 8,
                    end_line: 18,
                    start_column: None,
                    end_column: None,
                    content_hash: "cross-hash3".to_string(),
                },
                CodeBlock {
                    file_path: "src/module6.rs".to_string(),
                    start_line: 12,
                    end_line: 22,
                    start_column: None,
                    end_column: None,
                    content_hash: "cross-hash3".to_string(),
                },
            ],
            content: "fn log_error(msg: &str) { eprintln!(\"Error: {}\", msg); }".to_string(),
            line_count: 10,
            similarity_score: 1.0,
            risk_level: RiskLevel::Medium,
            refactoring_priority: RefactoringPriority::Medium,
        },
    ];

    let suggestions = generator.generate_suggestions(&duplications).await.unwrap();

    // Should generate individual suggestions plus combined suggestions
    assert!(suggestions.len() >= 3, "Should generate suggestions for each duplication");

    // Check if there's a utility module suggestion
    let has_utility_suggestion = suggestions.iter().any(|s| {
        s.suggestion_type == SuggestionType::CreateUtilityClass &&
        s.title.contains("公共工具模块")
    });

    assert!(has_utility_suggestion, "Should generate utility module suggestion for multiple cross-file duplications");
}

#[tokio::test]
async fn test_priority_based_sorting() {
    let generator = RefactoringSuggestionGenerator::new();

    // Create duplications with different priorities
    let duplications = vec![
        CodeDuplication {
            id: "low-priority".to_string(),
            duplication_type: DuplicationType::Exact,
            code_blocks: vec![
                CodeBlock {
                    file_path: "src/test1.rs".to_string(),
                    start_line: 1,
                    end_line: 3,
                    start_column: None,
                    end_column: None,
                    content_hash: "low-hash".to_string(),
                },
                CodeBlock {
                    file_path: "src/test2.rs".to_string(),
                    start_line: 1,
                    end_line: 3,
                    start_column: None,
                    end_column: None,
                    content_hash: "low-hash".to_string(),
                },
            ],
            content: "let x = 1;".to_string(),
            line_count: 3,
            similarity_score: 1.0,
            risk_level: RiskLevel::Low,
            refactoring_priority: RefactoringPriority::Low,
        },
        CodeDuplication {
            id: "high-priority".to_string(),
            duplication_type: DuplicationType::CrossFile,
            code_blocks: vec![
                CodeBlock {
                    file_path: "src/critical1.rs".to_string(),
                    start_line: 1,
                    end_line: 50,
                    start_column: None,
                    end_column: None,
                    content_hash: "high-hash".to_string(),
                },
                CodeBlock {
                    file_path: "src/critical2.rs".to_string(),
                    start_line: 1,
                    end_line: 50,
                    start_column: None,
                    end_column: None,
                    content_hash: "high-hash".to_string(),
                },
                CodeBlock {
                    file_path: "src/critical3.rs".to_string(),
                    start_line: 1,
                    end_line: 50,
                    start_column: None,
                    end_column: None,
                    content_hash: "high-hash".to_string(),
                },
            ],
            content: "// Large duplicated code block".repeat(50),
            line_count: 50,
            similarity_score: 1.0,
            risk_level: RiskLevel::Critical,
            refactoring_priority: RefactoringPriority::Urgent,
        },
    ];

    let suggestions = generator.generate_suggestions(&duplications).await.unwrap();

    // Verify that high priority suggestions come first
    assert!(!suggestions.is_empty());

    // The first suggestion should be for the high priority duplication
    let first_suggestion = &suggestions[0];
    assert_eq!(first_suggestion.duplication_id, "high-priority");
}

#[tokio::test]
async fn test_language_specific_suggestions() {
    let generator = RefactoringSuggestionGenerator::new();

    // Create a Go language duplication
    let go_duplication = CodeDuplication {
        id: "go-duplication".to_string(),
        duplication_type: DuplicationType::Exact,
        code_blocks: vec![
            CodeBlock {
                file_path: "main.go".to_string(),
                start_line: 10,
                end_line: 15,
                start_column: None,
                end_column: None,
                content_hash: "go-hash".to_string(),
            },
            CodeBlock {
                file_path: "utils.go".to_string(),
                start_line: 20,
                end_line: 25,
                start_column: None,
                end_column: None,
                content_hash: "go-hash".to_string(),
            },
        ],
        content: "const MaxRetries = 3\nconst Timeout = 30".to_string(),
        line_count: 6,
        similarity_score: 1.0,
        risk_level: RiskLevel::Medium,
        refactoring_priority: RefactoringPriority::Medium,
    };

    let duplications = vec![go_duplication];
    let suggestions = generator.generate_suggestions(&duplications).await.unwrap();

    assert!(!suggestions.is_empty());

    let suggestion = &suggestions[0];
    assert_eq!(suggestion.suggestion_type, SuggestionType::ExtractConstant);

    // Check that the suggestion contains Go-specific guidance
    if let Some(ref example) = suggestion.code_example {
        assert!(example.contains("const"), "Should contain Go const syntax");
    }

    // Check that Go-specific resources are included
    let has_go_resources = suggestion.resources.iter().any(|resource| {
        resource.contains("golang.org") || resource.contains("effective_go")
    });
    assert!(has_go_resources, "Should include Go-specific resources");
}