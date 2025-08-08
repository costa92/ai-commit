use ai_commit::analysis::duplication::{
    DuplicationDetector, DuplicationConfig, DuplicationResult, DuplicationType, RiskLevel,
    ExactDuplicationDetector, ProcessedContent
};
use ai_commit::languages::{Language, LanguageDetector};
use std::sync::Arc;
use tokio::sync::Mutex;
use tempfile::TempDir;
use std::fs;
use std::collections::HashMap;

/// 创建测试用的语言检测器
async fn create_test_language_detector() -> Arc<Mutex<LanguageDetector>> {
    let detector = LanguageDetector::new();
    Arc::new(Mutex::new(detector))
}

/// 创建测试文件
async fn create_test_files(temp_dir: &TempDir) -> Vec<String> {
    let file1_path = temp_dir.path().join("test1.rs");
    let file2_path = temp_dir.path().join("test2.rs");
    let file3_path = temp_dir.path().join("test3.rs");

    // 创建包含重复代码的测试文件
    let duplicate_code = r#"
fn calculate_sum(numbers: &[i32]) -> i32 {
    let mut sum = 0;
    for num in numbers {
        sum += num;
    }
    sum
}

fn process_data(data: &[i32]) -> i32 {
    let mut result = 0;
    for item in data {
        result += item * 2;
    }
    result
}
"#;

    let duplicate_code2 = r#"
fn calculate_sum(numbers: &[i32]) -> i32 {
    let mut sum = 0;
    for num in numbers {
        sum += num;
    }
    sum
}

fn different_function() -> String {
    "hello world".to_string()
}
"#;

    let unique_code = r#"
fn unique_function() -> bool {
    true
}

fn another_unique_function(x: i32, y: i32) -> i32 {
    x * y + 10
}
"#;

    fs::write(&file1_path, duplicate_code).expect("Failed to write test file 1");
    fs::write(&file2_path, duplicate_code2).expect("Failed to write test file 2");
    fs::write(&file3_path, unique_code).expect("Failed to write test file 3");

    vec![
        file1_path.to_string_lossy().to_string(),
        file2_path.to_string_lossy().to_string(),
        file3_path.to_string_lossy().to_string(),
    ]
}

#[tokio::test]
async fn test_duplication_detector_creation() {
    let config = DuplicationConfig::default();
    let language_detector = create_test_language_detector().await;

    let detector = DuplicationDetector::new(config.clone(), language_detector);

    // 验证配置是否正确设置
    assert_eq!(config.min_duplicate_lines, 5);
    assert_eq!(config.min_duplicate_chars, 100);
    assert!(config.enable_exact_detection);
    assert!(config.enable_structural_detection);
    assert!(config.enable_cross_file_detection);
}

#[tokio::test]
async fn test_duplication_detection_with_exact_duplicates() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let files = create_test_files(&temp_dir).await;

    let config = DuplicationConfig {
        min_duplicate_lines: 3,
        min_duplicate_chars: 50,
        enable_exact_detection: true,
        enable_structural_detection: false,
        enable_cross_file_detection: false,
        ..Default::default()
    };

    let language_detector = create_test_language_detector().await;
    let mut detector = DuplicationDetector::new(config, language_detector);

    let result = detector.detect_duplications(
        temp_dir.path().to_string_lossy().as_ref(),
        &files
    ).await.expect("Failed to detect duplications");

    // 验证检测结果
    assert!(!result.duplications.is_empty(), "Should detect duplications");

    // 验证统计摘要
    assert!(result.summary.total_files > 0);
    assert!(result.summary.total_duplications > 0);
    assert!(result.summary.duplication_ratio > 0.0);

    // 验证重复类型
    let has_exact_duplicates = result.duplications.iter()
        .any(|d| d.duplication_type == DuplicationType::Exact);
    assert!(has_exact_duplicates, "Should detect exact duplicates");
}

#[tokio::test]
async fn test_duplication_config_filtering() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");

    // 创建测试文件，包括应该被忽略的文件
    let test_file = temp_dir.path().join("test.rs");
    let test_spec_file = temp_dir.path().join("test.spec.rs");

    fs::write(&test_file, "fn test() { println!(\"hello\"); }").expect("Failed to write test file");
    fs::write(&test_spec_file, "fn test_spec() { println!(\"hello\"); }").expect("Failed to write spec file");

    let files = vec![
        test_file.to_string_lossy().to_string(),
        test_spec_file.to_string_lossy().to_string(),
    ];

    let config = DuplicationConfig {
        ignore_patterns: vec!["*.spec.*".to_string()],
        ..Default::default()
    };

    let language_detector = create_test_language_detector().await;
    let mut detector = DuplicationDetector::new(config, language_detector);

    let result = detector.detect_duplications(
        temp_dir.path().to_string_lossy().as_ref(),
        &files
    ).await.expect("Failed to detect duplications");

    // 验证过滤是否生效（spec文件应该被忽略）
    assert_eq!(result.summary.total_files, 1, "Should filter out spec files");
}

#[tokio::test]
async fn test_risk_level_assessment() {
    // 测试风险等级评估
    assert_eq!(RiskLevel::assess(5, 0.6), RiskLevel::Low);
    assert_eq!(RiskLevel::assess(25, 0.8), RiskLevel::Medium);
    assert_eq!(RiskLevel::assess(60, 0.9), RiskLevel::High);
    assert_eq!(RiskLevel::assess(150, 0.98), RiskLevel::Critical);
}

#[tokio::test]
async fn test_duplication_result_summary_calculation() {
    let mut result = DuplicationResult::new("/test/project".to_string());

    // 添加测试重复块
    use ai_commit::analysis::duplication::{CodeDuplication, CodeBlock};

    let duplication = CodeDuplication {
        id: "test-1".to_string(),
        duplication_type: DuplicationType::Exact,
        code_blocks: vec![
            CodeBlock {
                file_path: "file1.rs".to_string(),
                start_line: 1,
                end_line: 10,
                start_column: None,
                end_column: None,
                content_hash: "hash1".to_string(),
            },
            CodeBlock {
                file_path: "file2.rs".to_string(),
                start_line: 5,
                end_line: 14,
                start_column: None,
                end_column: None,
                content_hash: "hash1".to_string(),
            },
        ],
        content: "test content".to_string(),
        line_count: 10,
        similarity_score: 1.0,
        risk_level: RiskLevel::Medium,
        refactoring_priority: ai_commit::analysis::duplication::RefactoringPriority::Medium,
    };

    result.add_duplication(duplication);
    result.calculate_summary(2, 100);

    // 验证统计摘要
    assert_eq!(result.summary.total_files, 2);
    assert_eq!(result.summary.files_with_duplications, 2);
    assert_eq!(result.summary.total_duplications, 1);
    assert_eq!(result.summary.duplicated_lines, 10);
    assert_eq!(result.summary.total_lines, 100);
    assert_eq!(result.summary.duplication_ratio, 0.1);

    // 验证按类型统计
    assert!(result.summary.by_type.contains_key(&DuplicationType::Exact));
    let exact_stats = &result.summary.by_type[&DuplicationType::Exact];
    assert_eq!(exact_stats.count, 1);
    assert_eq!(exact_stats.lines, 10);
    assert_eq!(exact_stats.ratio, 0.1);

    // 验证按风险等级统计
    assert!(result.summary.by_risk_level.contains_key(&RiskLevel::Medium));
    let medium_stats = &result.summary.by_risk_level[&RiskLevel::Medium];
    assert_eq!(medium_stats.count, 1);
    assert_eq!(medium_stats.lines, 10);
    assert_eq!(medium_stats.ratio, 0.1);
}

#[tokio::test]
async fn test_empty_project_detection() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let files: Vec<String> = vec![];

    let config = DuplicationConfig::default();
    let language_detector = create_test_language_detector().await;
    let mut detector = DuplicationDetector::new(config, language_detector);

    let result = detector.detect_duplications(
        temp_dir.path().to_string_lossy().as_ref(),
        &files
    ).await.expect("Failed to detect duplications");

    // 验证空项目的处理
    assert!(result.duplications.is_empty());
    assert_eq!(result.summary.total_files, 0);
    assert_eq!(result.summary.total_duplications, 0);
    assert_eq!(result.summary.duplication_ratio, 0.0);
}

#[tokio::test]
async fn test_custom_config_thresholds() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");

    // 创建小的重复代码块
    let file1 = temp_dir.path().join("small1.rs");
    let file2 = temp_dir.path().join("small2.rs");

    let small_duplicate = "fn small() { 1 + 1 }";
    fs::write(&file1, small_duplicate).expect("Failed to write small file 1");
    fs::write(&file2, small_duplicate).expect("Failed to write small file 2");

    let files = vec![
        file1.to_string_lossy().to_string(),
        file2.to_string_lossy().to_string(),
    ];

    // 使用严格的阈值配置
    let strict_config = DuplicationConfig {
        min_duplicate_lines: 10,
        min_duplicate_chars: 200,
        ..Default::default()
    };

    let language_detector = create_test_language_detector().await;
    let mut detector = DuplicationDetector::new(strict_config, language_detector);

    let result = detector.detect_duplications(
        temp_dir.path().to_string_lossy().as_ref(),
        &files
    ).await.expect("Failed to detect duplications");

    // 验证严格阈值下不会检测到小的重复
    assert!(result.duplications.is_empty(), "Should not detect small duplicates with strict thresholds");
}

// ===== ExactDuplicationDetector 专门测试 =====

/// 创建测试用的处理内容
fn create_test_processed_content() -> HashMap<String, ProcessedContent> {
    let mut contents = HashMap::new();

    // 文件1：包含重复代码
    let content1 = r#"fn calculate_sum(numbers: &[i32]) -> i32 {
    let mut sum = 0;
    for num in numbers {
        sum += num;
    }
    sum
}

fn process_data(data: &[i32]) -> i32 {
    let mut result = 0;
    for item in data {
        result += item * 2;
    }
    result
}"#;

    // 文件2：包含相同的重复代码
    let content2 = r#"fn calculate_sum(numbers: &[i32]) -> i32 {
    let mut sum = 0;
    for num in numbers {
        sum += num;
    }
    sum
}

fn different_function() -> String {
    "hello world".to_string()
}"#;

    // 文件3：唯一代码
    let content3 = r#"fn unique_function() -> bool {
    true
}

fn another_unique_function(x: i32, y: i32) -> i32 {
    x * y + 10
}"#;

    contents.insert("file1.rs".to_string(), ProcessedContent {
        original: content1.to_string(),
        processed: content1.to_string(),
        language: Language::Rust,
        file_path: "file1.rs".to_string(),
    });

    contents.insert("file2.rs".to_string(), ProcessedContent {
        original: content2.to_string(),
        processed: content2.to_string(),
        language: Language::Rust,
        file_path: "file2.rs".to_string(),
    });

    contents.insert("file3.rs".to_string(), ProcessedContent {
        original: content3.to_string(),
        processed: content3.to_string(),
        language: Language::Rust,
        file_path: "file3.rs".to_string(),
    });

    contents
}

#[tokio::test]
async fn test_exact_duplication_detector_creation() {
    let config = DuplicationConfig::default();
    let mut detector = ExactDuplicationDetector::new(&config);

    // 验证初始状态
    let cache_stats = detector.get_cache_stats();
    assert_eq!(cache_stats.cache_size, 0);
    assert_eq!(cache_stats.memory_usage_bytes, 0);
}

#[tokio::test]
async fn test_exact_duplication_detection_basic() {
    let config = DuplicationConfig {
        min_duplicate_lines: 3,
        min_duplicate_chars: 50,
        ..Default::default()
    };

    let mut detector = ExactDuplicationDetector::new(&config);
    let contents = create_test_processed_content();

    let duplications = detector.detect(&contents).await
        .expect("Failed to detect duplications");

    // 验证检测到重复代码
    assert!(!duplications.is_empty(), "Should detect exact duplications");

    // 验证重复类型
    for duplication in &duplications {
        assert_eq!(duplication.duplication_type, DuplicationType::Exact);
        assert_eq!(duplication.similarity_score, 1.0);
        assert!(duplication.code_blocks.len() >= 2);
    }
}

#[tokio::test]
async fn test_exact_duplication_hash_calculation() {
    let config = DuplicationConfig::default();
    let mut detector = ExactDuplicationDetector::new(&config);

    // 测试相同内容产生相同哈希
    let content1 = "fn test() { println!(\"hello\"); }";
    let content2 = "fn test() { println!(\"hello\"); }";
    let content3 = "fn test() { println!(\"world\"); }";

    let hash1 = detector.calculate_hash_cached(content1);
    let hash2 = detector.calculate_hash_cached(content2);
    let hash3 = detector.calculate_hash_cached(content3);

    assert_eq!(hash1, hash2, "Same content should produce same hash");
    assert_ne!(hash1, hash3, "Different content should produce different hash");

    // 验证缓存工作
    let cache_stats = detector.get_cache_stats();
    assert_eq!(cache_stats.cache_size, 2); // content1 和 content3
}

#[tokio::test]
async fn test_exact_duplication_size_thresholds() {
    let config = DuplicationConfig {
        min_duplicate_lines: 5,
        min_duplicate_chars: 100,
        ..Default::default()
    };

    let mut detector = ExactDuplicationDetector::new(&config);

    // 创建小的重复代码（不满足阈值）
    let mut small_contents = HashMap::new();
    let small_code = "fn small() { 1 }";

    small_contents.insert("small1.rs".to_string(), ProcessedContent {
        original: small_code.to_string(),
        processed: small_code.to_string(),
        language: Language::Rust,
        file_path: "small1.rs".to_string(),
    });

    small_contents.insert("small2.rs".to_string(), ProcessedContent {
        original: small_code.to_string(),
        processed: small_code.to_string(),
        language: Language::Rust,
        file_path: "small2.rs".to_string(),
    });

    let duplications = detector.detect(&small_contents).await
        .expect("Failed to detect duplications");

    // 验证小的重复代码不会被检测到
    assert!(duplications.is_empty(), "Small duplicates should not be detected with strict thresholds");
}

#[tokio::test]
async fn test_exact_duplication_multiple_blocks() {
    let config = DuplicationConfig {
        min_duplicate_lines: 2,
        min_duplicate_chars: 20,
        ..Default::default()
    };

    let mut detector = ExactDuplicationDetector::new(&config);

    // 创建包含多个重复块的内容
    let mut contents = HashMap::new();
    let code_with_multiple_duplicates = r#"fn func1() {
    println!("duplicate");
    println!("block");
}

fn func2() {
    println!("unique");
}

fn func3() {
    println!("duplicate");
    println!("block");
}"#;

    contents.insert("multi1.rs".to_string(), ProcessedContent {
        original: code_with_multiple_duplicates.to_string(),
        processed: code_with_multiple_duplicates.to_string(),
        language: Language::Rust,
        file_path: "multi1.rs".to_string(),
    });

    contents.insert("multi2.rs".to_string(), ProcessedContent {
        original: code_with_multiple_duplicates.to_string(),
        processed: code_with_multiple_duplicates.to_string(),
        language: Language::Rust,
        file_path: "multi2.rs".to_string(),
    });

    let duplications = detector.detect(&contents).await
        .expect("Failed to detect duplications");

    // 验证检测到多个重复块
    assert!(!duplications.is_empty(), "Should detect multiple duplications");

    // 验证每个重复都有正确的代码块数量
    for duplication in &duplications {
        assert!(duplication.code_blocks.len() >= 2, "Each duplication should have at least 2 code blocks");
    }
}

#[tokio::test]
async fn test_exact_duplication_risk_assessment() {
    let config = DuplicationConfig {
        min_duplicate_lines: 1,
        min_duplicate_chars: 10,
        ..Default::default()
    };

    let mut detector = ExactDuplicationDetector::new(&config);

    // 创建不同大小的重复代码来测试风险评估
    let test_cases = vec![
        ("small", "fn small() { 1 }".to_string(), RiskLevel::Low),
        ("medium", "fn medium() {\n".to_string() + &"    println!(\"line\");\n".repeat(25) + "}", RiskLevel::Medium),
        ("large", "fn large() {\n".to_string() + &"    println!(\"line\");\n".repeat(60) + "}", RiskLevel::High),
        ("huge", "fn huge() {\n".to_string() + &"    println!(\"line\");\n".repeat(120) + "}", RiskLevel::Critical),
    ];

    for (name, code, expected_risk) in test_cases {
        let mut contents = HashMap::new();

        contents.insert(format!("{}1.rs", name), ProcessedContent {
            original: code.to_string(),
            processed: code.to_string(),
            language: Language::Rust,
            file_path: format!("{}1.rs", name),
        });

        contents.insert(format!("{}2.rs", name), ProcessedContent {
            original: code.to_string(),
            processed: code.to_string(),
            language: Language::Rust,
            file_path: format!("{}2.rs", name),
        });

        let duplications = detector.detect(&contents).await
            .expect("Failed to detect duplications");

        if !duplications.is_empty() {
            let duplication = &duplications[0];
            assert_eq!(duplication.risk_level, expected_risk,
                "Risk level mismatch for {} code", name);
        }
    }
}

#[tokio::test]
async fn test_exact_duplication_cache_management() {
    let config = DuplicationConfig::default();
    let mut detector = ExactDuplicationDetector::new(&config);

    // 添加一些内容到缓存
    let content1 = "fn test1() { println!(\"hello\"); }";
    let content2 = "fn test2() { println!(\"world\"); }";

    detector.calculate_hash_cached(content1);
    detector.calculate_hash_cached(content2);

    // 验证缓存状态
    let stats_before = detector.get_cache_stats();
    assert_eq!(stats_before.cache_size, 2);
    assert!(stats_before.memory_usage_bytes > 0);

    // 清空缓存
    detector.clear_cache();

    // 验证缓存已清空
    let stats_after = detector.get_cache_stats();
    assert_eq!(stats_after.cache_size, 0);
    assert_eq!(stats_after.memory_usage_bytes, 0);
}

#[tokio::test]
async fn test_exact_duplication_normalization() {
    let config = DuplicationConfig {
        min_duplicate_lines: 1,
        min_duplicate_chars: 10,
        ..Default::default()
    };

    let mut detector = ExactDuplicationDetector::new(&config);

    // 测试内容标准化（移除行尾空白）
    let mut contents = HashMap::new();
    let code1 = "fn test() {   \n    println!(\"hello\");  \n}";  // 有行尾空白
    let code2 = "fn test() {\n    println!(\"hello\");\n}";      // 无行尾空白

    contents.insert("norm1.rs".to_string(), ProcessedContent {
        original: code1.to_string(),
        processed: code1.to_string(),
        language: Language::Rust,
        file_path: "norm1.rs".to_string(),
    });

    contents.insert("norm2.rs".to_string(), ProcessedContent {
        original: code2.to_string(),
        processed: code2.to_string(),
        language: Language::Rust,
        file_path: "norm2.rs".to_string(),
    });

    let duplications = detector.detect(&contents).await
        .expect("Failed to detect duplications");

    // 验证标准化后的内容被识别为重复
    assert!(!duplications.is_empty(), "Normalized content should be detected as duplicates");
}

#[tokio::test]
async fn test_exact_duplication_empty_content() {
    let config = DuplicationConfig::default();
    let mut detector = ExactDuplicationDetector::new(&config);

    // 测试空内容
    let empty_contents = HashMap::new();
    let duplications = detector.detect(&empty_contents).await
        .expect("Failed to detect duplications");

    assert!(duplications.is_empty(), "Empty content should not produce duplications");
}

#[tokio::test]
async fn test_exact_duplication_sorting() {
    let config = DuplicationConfig {
        min_duplicate_lines: 1,
        min_duplicate_chars: 10,
        ..Default::default()
    };

    let mut detector = ExactDuplicationDetector::new(&config);

    // 创建不同风险等级的重复代码
    let mut contents = HashMap::new();

    // 小重复（低风险）
    let small_code = "fn small() { 1 }";
    contents.insert("small1.rs".to_string(), ProcessedContent {
        original: small_code.to_string(),
        processed: small_code.to_string(),
        language: Language::Rust,
        file_path: "small1.rs".to_string(),
    });
    contents.insert("small2.rs".to_string(), ProcessedContent {
        original: small_code.to_string(),
        processed: small_code.to_string(),
        language: Language::Rust,
        file_path: "small2.rs".to_string(),
    });

    // 大重复（高风险）
    let large_code = "fn large() {\n".to_string() + &"    println!(\"line\");\n".repeat(60) + "}";
    contents.insert("large1.rs".to_string(), ProcessedContent {
        original: large_code.to_string(),
        processed: large_code.to_string(),
        language: Language::Rust,
        file_path: "large1.rs".to_string(),
    });
    contents.insert("large2.rs".to_string(), ProcessedContent {
        original: large_code.to_string(),
        processed: large_code.to_string(),
        language: Language::Rust,
        file_path: "large2.rs".to_string(),
    });

    let duplications = detector.detect(&contents).await
        .expect("Failed to detect duplications");

    // 验证结果按风险等级排序（高风险在前）
    if duplications.len() > 1 {
        for i in 0..duplications.len()-1 {
            assert!(duplications[i].risk_level >= duplications[i+1].risk_level,
                "Duplications should be sorted by risk level (high to low)");
        }
    }
}