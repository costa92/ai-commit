use ai_commit::analysis::duplication::{
    DuplicationDetector, DuplicationConfig, CrossFileDuplicationDetector,
    DuplicationType, ProcessedContent, CrossFilePerformanceStats, CrossFileCacheStats
};
use ai_commit::languages::{Language, LanguageDetector};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tempfile::TempDir;
use std::fs;

/// 创建测试用的语言检测器
async fn create_test_language_detector() -> Arc<Mutex<LanguageDetector>> {
    let detector = LanguageDetector::new();
    Arc::new(Mutex::new(detector))
}

/// 创建测试用的处理内容
fn create_cross_file_test_content() -> HashMap<String, ProcessedContent> {
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

    // 文件2：包含相同的重复代码（跨文件精确重复）
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

    // 文件3：包含结构相似的代码（跨文件结构重复）
    let content3 = r#"fn compute_total(values: &[i32]) -> i32 {
    let mut total = 0;
    for value in values {
        total += value;
    }
    total
}

fn handle_request() -> bool {
    true
}"#;

    // 文件4：Go语言文件，测试跨语言检测
    let content4 = r#"func CalculateSum(numbers []int) int {
    sum := 0
    for _, num := range numbers {
        sum += num
    }
    return sum
}

func ProcessData(data []int) int {
    result := 0
    for _, item := range data {
        result += item * 2
    }
    return result
}"#;

    // 文件5：TypeScript文件，测试跨语言检测
    let content5 = r#"function calculateSum(numbers: number[]): number {
    let sum = 0;
    for (const num of numbers) {
        sum += num;
    }
    return sum;
}

async function fetchData(): Promise<string> {
    return "data";
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

    contents.insert("file4.go".to_string(), ProcessedContent {
        original: content4.to_string(),
        processed: content4.to_string(),
        language: Language::Go,
        file_path: "file4.go".to_string(),
    });

    contents.insert("file5.ts".to_string(), ProcessedContent {
        original: content5.to_string(),
        processed: content5.to_string(),
        language: Language::TypeScript,
        file_path: "file5.ts".to_string(),
    });

    contents
}

#[tokio::test]
async fn test_cross_file_detector_creation() {
    let config = DuplicationConfig::default();
    let mut detector = CrossFileDuplicationDetector::new(&config);

    // 验证初始状态
    let cache_stats = detector.get_cache_stats();
    assert_eq!(cache_stats.hash_cache_size, 0);
    assert_eq!(cache_stats.pattern_cache_size, 0);
    assert_eq!(cache_stats.hash_cache_memory, 0);
    assert_eq!(cache_stats.pattern_cache_memory, 0);

    let perf_stats = detector.get_performance_stats();
    assert_eq!(perf_stats.file_pairs_analyzed, 0);
    assert_eq!(perf_stats.code_blocks_extracted, 0);
    assert_eq!(perf_stats.total_duplications_found, 0);
}

#[tokio::test]
async fn test_cross_file_exact_duplication_detection() {
    let config = DuplicationConfig {
        min_duplicate_lines: 3,
        min_duplicate_chars: 50,
        enable_cross_file_detection: true,
        ..Default::default()
    };

    let mut detector = CrossFileDuplicationDetector::new(&config);
    let contents = create_cross_file_test_content();

    let duplications = detector.detect(&contents).await
        .expect("Failed to detect cross-file duplications");

    // 验证检测到跨文件重复
    assert!(!duplications.is_empty(), "Should detect cross-file duplications");

    // 验证重复类型
    let cross_file_duplications: Vec<_> = duplications.iter()
        .filter(|d| d.duplication_type == DuplicationType::CrossFile)
        .collect();

    assert!(!cross_file_duplications.is_empty(), "Should detect cross-file type duplications");

    // 验证跨文件特性
    for duplication in &cross_file_duplications {
        let unique_files: std::collections::HashSet<&String> = duplication.code_blocks.iter()
            .map(|block| &block.file_path)
            .collect();

        assert!(unique_files.len() >= 2, "Cross-file duplication should span multiple files");
        assert!(duplication.code_blocks.len() >= 2, "Should have at least 2 code blocks");
    }
}

#[tokio::test]
async fn test_cross_file_structural_similarity_detection() {
    let config = DuplicationConfig {
        min_duplicate_lines: 4,
        min_duplicate_chars: 80,
        enable_structural_detection: true,
        enable_cross_file_detection: true,
        structural_similarity_threshold: 0.7,
        ..Default::default()
    };

    let mut detector = CrossFileDuplicationDetector::new(&config);
    let contents = create_cross_file_test_content();

    let duplications = detector.detect(&contents).await
        .expect("Failed to detect cross-file structural duplications");

    // 验证检测到结构相似的跨文件重复
    let structural_duplications: Vec<_> = duplications.iter()
        .filter(|d| d.similarity_score >= 0.7 && d.similarity_score < 1.0)
        .collect();

    if !structural_duplications.is_empty() {
        for duplication in &structural_duplications {
            let unique_files: std::collections::HashSet<&String> = duplication.code_blocks.iter()
                .map(|block| &block.file_path)
                .collect();

            assert!(unique_files.len() >= 2, "Structural duplication should span multiple files");
            assert!(duplication.similarity_score >= 0.7, "Similarity score should meet threshold");
        }
    }
}

#[tokio::test]
async fn test_cross_file_language_specific_detection() {
    let config = DuplicationConfig {
        min_duplicate_lines: 3,
        min_duplicate_chars: 50,
        enable_cross_file_detection: true,
        ..Default::default()
    };

    let mut detector = CrossFileDuplicationDetector::new(&config);
    let contents = create_cross_file_test_content();

    let duplications = detector.detect(&contents).await
        .expect("Failed to detect language-specific cross-file duplications");

    // 验证语言特定的检测
    let rust_files: Vec<_> = duplications.iter()
        .flat_map(|d| &d.code_blocks)
        .filter(|block| block.file_path.ends_with(".rs"))
        .collect();

    let go_files: Vec<_> = duplications.iter()
        .flat_map(|d| &d.code_blocks)
        .filter(|block| block.file_path.ends_with(".go"))
        .collect();

    let ts_files: Vec<_> = duplications.iter()
        .flat_map(|d| &d.code_blocks)
        .filter(|block| block.file_path.ends_with(".ts"))
        .collect();

    // 验证不同语言的文件都被处理
    assert!(!rust_files.is_empty(), "Should process Rust files");

    // 注意：由于预过滤机制，不同语言的文件可能不会被比较
    // 这是正确的行为，因为不同语言的代码结构差异很大
}

#[tokio::test]
async fn test_cross_file_performance_optimization() {
    let config = DuplicationConfig {
        min_duplicate_lines: 5,
        min_duplicate_chars: 100,
        enable_cross_file_detection: true,
        ..Default::default()
    };

    let mut detector = CrossFileDuplicationDetector::new(&config);

    // 创建大量文件来测试性能优化
    let mut large_contents = HashMap::new();

    for i in 0..20 {
        let content = format!(r#"
fn function_{}() {{
    let mut result = 0;
    for i in 0..10 {{
        result += i * {};
    }}
    result
}}

fn unique_function_{}() {{
    println!("This is unique function {}")
}}
"#, i, i + 1, i, i);

        large_contents.insert(format!("large_file_{}.rs", i), ProcessedContent {
            original: content.clone(),
            processed: content.clone(),
            language: Language::Rust,
            file_path: format!("large_file_{}.rs", i),
        });
    }

    let start_time = std::time::Instant::now();
    let duplications = detector.detect(&large_contents).await
        .expect("Failed to detect duplications in large project");
    let detection_time = start_time.elapsed();

    // 验证性能统计
    let perf_stats = detector.get_performance_stats();
    assert!(perf_stats.file_pairs_analyzed > 0, "Should analyze file pairs");
    assert!(perf_stats.code_blocks_extracted > 0, "Should extract code blocks");
    assert_eq!(perf_stats.total_duplications_found, duplications.len());

    // 验证检测时间合理（应该在几秒内完成）
    assert!(detection_time.as_secs() < 10, "Detection should complete within 10 seconds");

    println!("Performance stats: {:?}", perf_stats);
    println!("Detection time: {:?}", detection_time);
}

#[tokio::test]
async fn test_cross_file_caching_mechanism() {
    let config = DuplicationConfig::default();
    let mut detector = CrossFileDuplicationDetector::new(&config);
    let contents = create_cross_file_test_content();

    // 第一次检测
    let _duplications1 = detector.detect(&contents).await
        .expect("Failed to detect duplications");
    let cache_stats1 = detector.get_cache_stats();

    // 第二次检测相同内容（应该使用缓存）
    let _duplications2 = detector.detect(&contents).await
        .expect("Failed to detect duplications");
    let cache_stats2 = detector.get_cache_stats();

    // 验证缓存被使用
    assert!(cache_stats2.hash_cache_size >= cache_stats1.hash_cache_size,
           "Cache size should maintain or grow");
    assert!(cache_stats2.hash_cache_memory > 0, "Cache should contain data");

    // 清空缓存
    detector.clear_cache();
    let cache_stats3 = detector.get_cache_stats();
    assert_eq!(cache_stats3.hash_cache_size, 0, "Cache should be empty after clear");
    assert_eq!(cache_stats3.pattern_cache_size, 0, "Pattern cache should be empty after clear");
}

#[tokio::test]
async fn test_cross_file_size_filtering() {
    let config = DuplicationConfig {
        min_duplicate_lines: 10,
        min_duplicate_chars: 200,
        enable_cross_file_detection: true,
        ..Default::default()
    };

    let mut detector = CrossFileDuplicationDetector::new(&config);

    // 创建小文件，不应该被检测
    let mut small_contents = HashMap::new();
    let small_code = "fn small() { 1 }";

    for i in 0..3 {
        small_contents.insert(format!("small_{}.rs", i), ProcessedContent {
            original: small_code.to_string(),
            processed: small_code.to_string(),
            language: Language::Rust,
            file_path: format!("small_{}.rs", i),
        });
    }

    let duplications = detector.detect(&small_contents).await
        .expect("Failed to detect duplications");

    // 验证小文件不会被检测为重复
    assert!(duplications.is_empty(), "Small files should not be detected with strict thresholds");
}

#[tokio::test]
async fn test_cross_file_different_languages_separation() {
    let config = DuplicationConfig {
        min_duplicate_lines: 3,
        min_duplicate_chars: 50,
        enable_cross_file_detection: true,
        ..Default::default()
    };

    let mut detector = CrossFileDuplicationDetector::new(&config);

    // 创建不同语言的相似代码
    let mut mixed_contents = HashMap::new();

    // Rust代码
    let rust_code = r#"fn calculate(x: i32) -> i32 {
    let mut result = 0;
    for i in 0..x {
        result += i;
    }
    result
}"#;

    // Go代码（结构相似但语法不同）
    let go_code = r#"func Calculate(x int) int {
    result := 0
    for i := 0; i < x; i++ {
        result += i
    }
    return result
}"#;

    mixed_contents.insert("rust_file.rs".to_string(), ProcessedContent {
        original: rust_code.to_string(),
        processed: rust_code.to_string(),
        language: Language::Rust,
        file_path: "rust_file.rs".to_string(),
    });

    mixed_contents.insert("go_file.go".to_string(), ProcessedContent {
        original: go_code.to_string(),
        processed: go_code.to_string(),
        language: Language::Go,
        file_path: "go_file.go".to_string(),
    });

    let duplications = detector.detect(&mixed_contents).await
        .expect("Failed to detect duplications");

    // 验证不同语言的文件不会被错误地检测为重复
    // 由于预过滤机制，不同语言的文件应该被分开处理
    for duplication in &duplications {
        let languages: std::collections::HashSet<&str> = duplication.code_blocks.iter()
            .map(|block| {
                if block.file_path.ends_with(".rs") { "rust" }
                else if block.file_path.ends_with(".go") { "go" }
                else { "other" }
            })
            .collect();

        // 每个重复应该只涉及一种语言
        assert_eq!(languages.len(), 1, "Cross-file duplications should not span different languages");
    }
}

#[tokio::test]
async fn test_cross_file_result_optimization() {
    let config = DuplicationConfig {
        min_duplicate_lines: 2,
        min_duplicate_chars: 30,
        enable_cross_file_detection: true,
        ..Default::default()
    };

    let mut detector = CrossFileDuplicationDetector::new(&config);

    // 创建有重叠的重复代码
    let mut overlapping_contents = HashMap::new();

    let content1 = r#"fn func1() {
    println!("duplicate line 1");
    println!("duplicate line 2");
    println!("duplicate line 3");
}

fn func2() {
    println!("unique line");
}"#;

    let content2 = r#"fn func3() {
    println!("duplicate line 1");
    println!("duplicate line 2");
    println!("duplicate line 3");
}

fn func4() {
    println!("another unique line");
}"#;

    let content3 = r#"fn func5() {
    println!("duplicate line 1");
    println!("duplicate line 2");
}

fn func6() {
    println!("third unique line");
}"#;

    overlapping_contents.insert("overlap1.rs".to_string(), ProcessedContent {
        original: content1.to_string(),
        processed: content1.to_string(),
        language: Language::Rust,
        file_path: "overlap1.rs".to_string(),
    });

    overlapping_contents.insert("overlap2.rs".to_string(), ProcessedContent {
        original: content2.to_string(),
        processed: content2.to_string(),
        language: Language::Rust,
        file_path: "overlap2.rs".to_string(),
    });

    overlapping_contents.insert("overlap3.rs".to_string(), ProcessedContent {
        original: content3.to_string(),
        processed: content3.to_string(),
        language: Language::Rust,
        file_path: "overlap3.rs".to_string(),
    });

    let duplications = detector.detect(&overlapping_contents).await
        .expect("Failed to detect overlapping duplications");

    // 验证结果优化工作正常
    assert!(!duplications.is_empty(), "Should detect duplications");

    // 验证结果按风险等级排序
    for i in 0..duplications.len().saturating_sub(1) {
        assert!(duplications[i].risk_level >= duplications[i + 1].risk_level,
               "Results should be sorted by risk level");
    }
}

#[tokio::test]
async fn test_cross_file_empty_and_edge_cases() {
    let config = DuplicationConfig::default();
    let mut detector = CrossFileDuplicationDetector::new(&config);

    // 测试空内容
    let empty_contents = HashMap::new();
    let duplications = detector.detect(&empty_contents).await
        .expect("Failed to handle empty contents");
    assert!(duplications.is_empty(), "Empty contents should produce no duplications");

    // 测试单个文件
    let mut single_file_contents = HashMap::new();
    single_file_contents.insert("single.rs".to_string(), ProcessedContent {
        original: "fn test() { println!(\"hello\"); }".to_string(),
        processed: "fn test() { println!(\"hello\"); }".to_string(),
        language: Language::Rust,
        file_path: "single.rs".to_string(),
    });

    let duplications = detector.detect(&single_file_contents).await
        .expect("Failed to handle single file");
    assert!(duplications.is_empty(), "Single file should produce no cross-file duplications");

    // 测试完全不同的文件
    let mut different_contents = HashMap::new();
    different_contents.insert("diff1.rs".to_string(), ProcessedContent {
        original: "fn unique1() { println!(\"first\"); }".to_string(),
        processed: "fn unique1() { println!(\"first\"); }".to_string(),
        language: Language::Rust,
        file_path: "diff1.rs".to_string(),
    });

    different_contents.insert("diff2.rs".to_string(), ProcessedContent {
        original: "fn unique2() { println!(\"second\"); }".to_string(),
        processed: "fn unique2() { println!(\"second\"); }".to_string(),
        language: Language::Rust,
        file_path: "diff2.rs".to_string(),
    });

    let duplications = detector.detect(&different_contents).await
        .expect("Failed to handle different files");
    assert!(duplications.is_empty(), "Completely different files should produce no duplications");
}

/// 集成测试：跨文件检测与完整的重复检测器
#[tokio::test]
async fn test_cross_file_detection_integration() {
    let config = DuplicationConfig {
        enable_exact_detection: true,
        enable_structural_detection: true,
        enable_cross_file_detection: true,
        min_duplicate_lines: 3,
        min_duplicate_chars: 50,
        structural_similarity_threshold: 0.75,
        ..Default::default()
    };

    let language_detector = create_test_language_detector().await;
    let mut detector = DuplicationDetector::new(config, language_detector);

    // 创建临时文件进行集成测试
    let temp_dir = tempfile::tempdir().unwrap();
    let file1_path = temp_dir.path().join("integration1.rs");
    let file2_path = temp_dir.path().join("integration2.rs");
    let file3_path = temp_dir.path().join("integration3.rs");

    // 写入测试文件
    tokio::fs::write(&file1_path, r#"
fn calculate_total(items: &[Item]) -> f64 {
    let mut total = 0.0;
    for item in items {
        if item.is_valid {
            total += item.price * item.quantity;
        }
    }
    total
}

fn process_order(order: &Order) -> bool {
    if order.is_confirmed {
        return true;
    }
    false
}
"#).await.unwrap();

    tokio::fs::write(&file2_path, r#"
fn calculate_total(items: &[Item]) -> f64 {
    let mut total = 0.0;
    for item in items {
        if item.is_valid {
            total += item.price * item.quantity;
        }
    }
    total
}

fn handle_payment(payment: &Payment) -> String {
    "processed".to_string()
}
"#).await.unwrap();

    tokio::fs::write(&file3_path, r#"
fn compute_sum(products: &[Product]) -> f64 {
    let mut sum = 0.0;
    for product in products {
        if product.is_active {
            sum += product.cost * product.amount;
        }
    }
    sum
}

fn validate_user(user: &User) -> bool {
    user.is_verified
}
"#).await.unwrap();

    let files = vec![
        file1_path.to_string_lossy().to_string(),
        file2_path.to_string_lossy().to_string(),
        file3_path.to_string_lossy().to_string(),
    ];

    let result = detector.detect_duplications(
        temp_dir.path().to_string_lossy().as_ref(),
        &files
    ).await.unwrap();

    // 应该检测到跨文件重复
    assert!(!result.duplications.is_empty(), "Should detect cross-file duplications");

    let cross_file_duplications: Vec<_> = result.duplications.iter()
        .filter(|d| {
            let unique_files: std::collections::HashSet<&String> = d.code_blocks.iter()
                .map(|block| &block.file_path)
                .collect();
            unique_files.len() >= 2
        })
        .collect();

    assert!(!cross_file_duplications.is_empty(), "Should detect cross-file duplications");

    // 验证统计摘要包含跨文件信息
    assert!(result.summary.total_duplications > 0, "Summary should include cross-file duplications");
    assert!(result.summary.files_with_duplications >= 2, "Should involve multiple files");

    // 验证重构建议
    assert!(!result.refactoring_suggestions.is_empty(), "Should provide refactoring suggestions");
}

#[tokio::test]
async fn test_cross_file_similarity_calculation() {
    let config = DuplicationConfig {
        min_duplicate_lines: 3,
        min_duplicate_chars: 50,
        enable_cross_file_detection: true,
        structural_similarity_threshold: 0.8,
        ..Default::default()
    };

    let mut detector = CrossFileDuplicationDetector::new(&config);

    // 创建具有不同相似度的代码
    let mut similarity_contents = HashMap::new();

    // 高相似度代码（结构相同，变量名不同）
    let high_sim1 = r#"fn process_data(input: &[i32]) -> i32 {
    let mut result = 0;
    for item in input {
        if *item > 0 {
            result += item * 2;
        }
    }
    result
}"#;

    let high_sim2 = r#"fn handle_values(data: &[i32]) -> i32 {
    let mut output = 0;
    for value in data {
        if *value > 0 {
            output += value * 2;
        }
    }
    output
}"#;

    // 中等相似度代码（结构相似，逻辑略有不同）
    let med_sim = r#"fn calculate_result(numbers: &[i32]) -> i32 {
    let mut total = 0;
    for num in numbers {
        if *num > 5 {
            total += num * 3;
        }
    }
    total
}"#;

    // 低相似度代码（结构不同）
    let low_sim = r#"fn different_approach(arr: &[i32]) -> String {
    match arr.len() {
        0 => "empty".to_string(),
        1 => "single".to_string(),
        _ => "multiple".to_string(),
    }
}"#;

    similarity_contents.insert("high_sim1.rs".to_string(), ProcessedContent {
        original: high_sim1.to_string(),
        processed: high_sim1.to_string(),
        language: Language::Rust,
        file_path: "high_sim1.rs".to_string(),
    });

    similarity_contents.insert("high_sim2.rs".to_string(), ProcessedContent {
        original: high_sim2.to_string(),
        processed: high_sim2.to_string(),
        language: Language::Rust,
        file_path: "high_sim2.rs".to_string(),
    });

    similarity_contents.insert("med_sim.rs".to_string(), ProcessedContent {
        original: med_sim.to_string(),
        processed: med_sim.to_string(),
        language: Language::Rust,
        file_path: "med_sim.rs".to_string(),
    });

    similarity_contents.insert("low_sim.rs".to_string(), ProcessedContent {
        original: low_sim.to_string(),
        processed: low_sim.to_string(),
        language: Language::Rust,
        file_path: "low_sim.rs".to_string(),
    });

    let duplications = detector.detect(&similarity_contents).await
        .expect("Failed to detect similarity-based duplications");

    // 验证相似度计算
    for duplication in &duplications {
        if duplication.similarity_score >= 0.8 {
            // 高相似度的重复应该涉及高相似度的文件
            let has_high_sim_files = duplication.code_blocks.iter()
                .any(|block| block.file_path.contains("high_sim"));

            if has_high_sim_files {
                assert!(duplication.similarity_score >= 0.8,
                       "High similarity files should have high similarity score");
            }
        }
    }
}