use ai_commit::analysis::duplication::{
    DuplicationDetector, DuplicationConfig, DuplicationResult, DuplicationType, RiskLevel,
    HotspotLevel, DuplicationGrade, RecommendationPriority, CodeDuplication, CodeBlock,
    RefactoringPriority
};
use ai_commit::languages::{Language, LanguageDetector};
use std::sync::Arc;
use tokio::sync::Mutex;
use tempfile::TempDir;
use std::fs;

/// 创建测试用的语言检测器
async fn create_test_language_detector() -> Arc<Mutex<LanguageDetector>> {
    let detector = LanguageDetector::new();
    Arc::new(Mutex::new(detector))
}

/// 创建包含重复代码的测试文件
async fn create_duplication_test_files(temp_dir: &TempDir) -> Vec<String> {
    let file1_path = temp_dir.path().join("service1.rs");
    let file2_path = temp_dir.path().join("service2.rs");
    let file3_path = temp_dir.path().join("utils.rs");
    let file4_path = temp_dir.path().join("handler.rs");

    // 文件1：包含大量重复代码（极端热点）
    let file1_content = r#"
fn process_user_data(user: &User) -> Result<String, Error> {
    if user.is_active {
        let result = validate_user(user);
        if result.is_ok() {
            let formatted = format_user_info(user);
            log::info!("User processed: {}", user.id);
            return Ok(formatted);
        } else {
            log::error!("User validation failed: {}", user.id);
            return Err(Error::ValidationFailed);
        }
    } else {
        log::warn!("Inactive user: {}", user.id);
        return Err(Error::InactiveUser);
    }
}

fn process_admin_data(admin: &Admin) -> Result<String, Error> {
    if admin.is_active {
        let result = validate_admin(admin);
        if result.is_ok() {
            let formatted = format_admin_info(admin);
            log::info!("Admin processed: {}", admin.id);
            return Ok(formatted);
        } else {
            log::error!("Admin validation failed: {}", admin.id);
            return Err(Error::ValidationFailed);
        }
    } else {
        log::warn!("Inactive admin: {}", admin.id);
        return Err(Error::InactiveUser);
    }
}

fn process_customer_data(customer: &Customer) -> Result<String, Error> {
    if customer.is_active {
        let result = validate_customer(customer);
        if result.is_ok() {
            let formatted = format_customer_info(customer);
            log::info!("Customer processed: {}", customer.id);
            return Ok(formatted);
        } else {
            log::error!("Customer validation failed: {}", customer.id);
            return Err(Error::ValidationFailed);
        }
    } else {
        log::warn!("Inactive customer: {}", customer.id);
        return Err(Error::InactiveUser);
    }
}
"#;

    // 文件2：包含中等重复代码
    let file2_content = r#"
fn handle_request(request: &Request) -> Response {
    let start_time = std::time::Instant::now();

    match request.method {
        Method::GET => {
            let result = process_get_request(request);
            log::info!("GET request processed in {:?}", start_time.elapsed());
            result
        }
        Method::POST => {
            let result = process_post_request(request);
            log::info!("POST request processed in {:?}", start_time.elapsed());
            result
        }
        Method::PUT => {
            let result = process_put_request(request);
            log::info!("PUT request processed in {:?}", start_time.elapsed());
            result
        }
        _ => {
            log::warn!("Unsupported method: {:?}", request.method);
            Response::error("Method not supported")
        }
    }
}

fn handle_api_call(api_call: &ApiCall) -> ApiResponse {
    let start_time = std::time::Instant::now();

    match api_call.endpoint {
        Endpoint::Users => {
            let result = process_users_call(api_call);
            log::info!("Users API call processed in {:?}", start_time.elapsed());
            result
        }
        Endpoint::Orders => {
            let result = process_orders_call(api_call);
            log::info!("Orders API call processed in {:?}", start_time.elapsed());
            result
        }
        _ => {
            log::warn!("Unsupported endpoint: {:?}", api_call.endpoint);
            ApiResponse::error("Endpoint not supported")
        }
    }
}
"#;

    // 文件3：包含少量重复代码
    let file3_content = r#"
fn calculate_total(items: &[Item]) -> f64 {
    let mut total = 0.0;
    for item in items {
        if item.is_valid {
            total += item.price * item.quantity;
        }
    }
    total
}

fn calculate_subtotal(products: &[Product]) -> f64 {
    let mut subtotal = 0.0;
    for product in products {
        if product.is_available {
            subtotal += product.cost * product.amount;
        }
    }
    subtotal
}

fn unique_utility_function() -> String {
    "This is a unique function".to_string()
}
"#;

    // 文件4：跨文件重复
    let file4_content = r#"
fn process_user_data(user: &User) -> Result<String, Error> {
    if user.is_active {
        let result = validate_user(user);
        if result.is_ok() {
            let formatted = format_user_info(user);
            log::info!("User processed: {}", user.id);
            return Ok(formatted);
        } else {
            log::error!("User validation failed: {}", user.id);
            return Err(Error::ValidationFailed);
        }
    } else {
        log::warn!("Inactive user: {}", user.id);
        return Err(Error::InactiveUser);
    }
}

fn different_function() -> bool {
    true
}
"#;

    fs::write(&file1_path, file1_content).expect("Failed to write test file 1");
    fs::write(&file2_path, file2_content).expect("Failed to write test file 2");
    fs::write(&file3_path, file3_content).expect("Failed to write test file 3");
    fs::write(&file4_path, file4_content).expect("Failed to write test file 4");

    vec![
        file1_path.to_string_lossy().to_string(),
        file2_path.to_string_lossy().to_string(),
        file3_path.to_string_lossy().to_string(),
        file4_path.to_string_lossy().to_string(),
    ]
}

#[tokio::test]
async fn test_duplication_analysis_comprehensive() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let files = create_duplication_test_files(&temp_dir).await;

    let config = DuplicationConfig {
        min_duplicate_lines: 3,
        min_duplicate_chars: 50,
        enable_exact_detection: true,
        enable_structural_detection: true,
        enable_cross_file_detection: true,
        structural_similarity_threshold: 0.7,
        ..Default::default()
    };

    let language_detector = create_test_language_detector().await;
    let mut detector = DuplicationDetector::new(config, language_detector);

    let result = detector.detect_duplications(
        temp_dir.path().to_string_lossy().as_ref(),
        &files
    ).await.expect("Failed to detect duplications");

    // 验证基本检测结果
    assert!(!result.duplications.is_empty(), "Should detect duplications");
    assert!(result.summary.total_files > 0, "Should have processed files");
    assert!(result.summary.duplication_ratio > 0.0, "Should have calculated duplication ratio");

    // 测试热点文件识别
    assert!(!result.summary.hotspot_files.is_empty(), "Should identify hotspot files");

    // 验证热点等级评估
    let has_high_hotspot = result.summary.hotspot_files
        .iter()
        .any(|h| matches!(h.hotspot_level, HotspotLevel::Severe | HotspotLevel::Extreme));
    assert!(has_high_hotspot, "Should identify high-level hotspots");

    println!("Duplication analysis results:");
    println!("Total files: {}", result.summary.total_files);
    println!("Files with duplications: {}", result.summary.files_with_duplications);
    println!("Total duplications: {}", result.summary.total_duplications);
    println!("Duplication ratio: {:.2}%", result.summary.duplication_ratio * 100.0);
    println!("Hotspot files: {}", result.summary.hotspot_files.len());
}

#[tokio::test]
async fn test_distribution_generation() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let files = create_duplication_test_files(&temp_dir).await;

    let config = DuplicationConfig {
        min_duplicate_lines: 3,
        min_duplicate_chars: 50,
        enable_exact_detection: true,
        enable_structural_detection: true,
        enable_cross_file_detection: true,
        ..Default::default()
    };

    let language_detector = create_test_language_detector().await;
    let mut detector = DuplicationDetector::new(config, language_detector);

    let result = detector.detect_duplications(
        temp_dir.path().to_string_lossy().as_ref(),
        &files
    ).await.expect("Failed to detect duplications");

    // 测试分布图生成
    let distribution = result.generate_distribution();

    // 验证文件大小分布
    assert!(!distribution.by_file_size.is_empty(), "Should generate file size distribution");

    // 验证类型分布
    assert!(!distribution.by_type_distribution.is_empty(), "Should generate type distribution");

    // 验证风险分布
    assert!(!distribution.by_risk_distribution.is_empty(), "Should generate risk distribution");

    // 验证密度分布
    assert!(!distribution.density_distribution.is_empty(), "Should generate density distribution");

    println!("Distribution analysis:");
    for file_size_dist in &distribution.by_file_size {
        println!("Size range {}: {} files, {:.1}% duplication",
                file_size_dist.size_range,
                file_size_dist.file_count,
                file_size_dist.duplication_ratio * 100.0);
    }

    for type_dist in &distribution.by_type_distribution {
        println!("Type {:?}: {} occurrences, {:.1}% of total, avg {:.1} lines",
                type_dist.duplication_type,
                type_dist.count,
                type_dist.percentage,
                type_dist.avg_lines);
    }
}

#[tokio::test]
async fn test_detailed_report_generation() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let files = create_duplication_test_files(&temp_dir).await;

    let config = DuplicationConfig {
        min_duplicate_lines: 3,
        min_duplicate_chars: 50,
        enable_exact_detection: true,
        enable_structural_detection: true,
        enable_cross_file_detection: true,
        ..Default::default()
    };

    let language_detector = create_test_language_detector().await;
    let mut detector = DuplicationDetector::new(config, language_detector);

    let result = detector.detect_duplications(
        temp_dir.path().to_string_lossy().as_ref(),
        &files
    ).await.expect("Failed to detect duplications");

    // 测试详细报告生成
    let detailed_report = result.generate_detailed_report();

    // 验证项目概览
    assert!(!detailed_report.project_overview.project_path.is_empty(), "Should have project path");
    assert!(detailed_report.project_overview.health_score >= 0.0, "Should calculate health score");
    assert!(detailed_report.project_overview.health_score <= 100.0, "Health score should be valid");

    // 验证重复代码等级
    let grade = detailed_report.project_overview.duplication_grade;
    assert!(matches!(grade, DuplicationGrade::A | DuplicationGrade::B | DuplicationGrade::C | DuplicationGrade::D | DuplicationGrade::F),
           "Should assign valid duplication grade");

    // 验证关键指标
    let metrics = &detailed_report.project_overview.key_metrics;
    assert!(metrics.total_duplication_ratio >= 0.0, "Should calculate total duplication ratio");
    assert!(metrics.avg_duplication_size >= 0.0, "Should calculate average duplication size");
    assert!(metrics.duplicated_files_ratio >= 0.0, "Should calculate duplicated files ratio");

    // 验证热点分析
    assert!(!detailed_report.hotspot_analysis.hotspot_files.is_empty(), "Should identify hotspot files");
    assert!(!detailed_report.hotspot_analysis.hotspot_recommendations.is_empty(), "Should provide hotspot recommendations");

    // 验证改进建议
    assert!(!detailed_report.improvement_recommendations.is_empty(), "Should provide improvement recommendations");

    println!("Detailed report summary:");
    println!("Health score: {:.1}", detailed_report.project_overview.health_score);
    println!("Duplication grade: {:?}", detailed_report.project_overview.duplication_grade);
    println!("Total duplication ratio: {:.2}%", metrics.total_duplication_ratio * 100.0);
    println!("Average duplication size: {:.1} lines", metrics.avg_duplication_size);
    println!("High risk duplications: {}", metrics.high_risk_duplications);
    println!("Improvement recommendations: {}", detailed_report.improvement_recommendations.len());
}

#[tokio::test]
async fn test_hotspot_level_assessment() {
    // 测试热点等级评估
    assert_eq!(HotspotLevel::assess(0.05, 2), HotspotLevel::Minor);
    assert_eq!(HotspotLevel::assess(0.18, 6), HotspotLevel::Moderate);
    assert_eq!(HotspotLevel::assess(0.35, 12), HotspotLevel::Severe);
    assert_eq!(HotspotLevel::assess(0.6, 25), HotspotLevel::Extreme);

    // 边界情况测试
    assert_eq!(HotspotLevel::assess(0.15, 5), HotspotLevel::Moderate);
    assert_eq!(HotspotLevel::assess(0.3, 10), HotspotLevel::Severe);
    assert_eq!(HotspotLevel::assess(0.5, 20), HotspotLevel::Extreme);
}

#[tokio::test]
async fn test_duplication_grade_assessment() {
    // 测试重复代码等级评估
    assert_eq!(DuplicationGrade::from_ratio(0.03), DuplicationGrade::A);
    assert_eq!(DuplicationGrade::from_ratio(0.08), DuplicationGrade::B);
    assert_eq!(DuplicationGrade::from_ratio(0.15), DuplicationGrade::C);
    assert_eq!(DuplicationGrade::from_ratio(0.25), DuplicationGrade::D);
    assert_eq!(DuplicationGrade::from_ratio(0.35), DuplicationGrade::F);

    // 边界情况测试
    assert_eq!(DuplicationGrade::from_ratio(0.05), DuplicationGrade::B);
    assert_eq!(DuplicationGrade::from_ratio(0.10), DuplicationGrade::C);
    assert_eq!(DuplicationGrade::from_ratio(0.20), DuplicationGrade::D);
    assert_eq!(DuplicationGrade::from_ratio(0.30), DuplicationGrade::F);
}

#[tokio::test]
async fn test_recommendation_priority_assessment() {
    // 测试建议优先级评估
    assert_eq!(RecommendationPriority::Low.cmp(&RecommendationPriority::Medium), std::cmp::Ordering::Less);
    assert_eq!(RecommendationPriority::Medium.cmp(&RecommendationPriority::High), std::cmp::Ordering::Less);
    assert_eq!(RecommendationPriority::High.cmp(&RecommendationPriority::Critical), std::cmp::Ordering::Less);
}

#[tokio::test]
async fn test_health_score_calculation() {
    let mut result = DuplicationResult::new("/test/project".to_string());

    // 添加测试重复块
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
        risk_level: RiskLevel::High,
        refactoring_priority: RefactoringPriority::High,
    };

    result.add_duplication(duplication);
    result.calculate_summary(2, 100);

    let detailed_report = result.generate_detailed_report();
    let health_score = detailed_report.project_overview.health_score;

    // 健康评分应该在合理范围内
    assert!(health_score >= 0.0, "Health score should be non-negative");
    assert!(health_score <= 100.0, "Health score should not exceed 100");

    // 由于有高风险重复，健康评分应该受到影响
    assert!(health_score < 100.0, "Health score should be reduced due to high-risk duplications");

    println!("Health score with high-risk duplication: {:.1}", health_score);
}

#[tokio::test]
async fn test_empty_project_analysis() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let files: Vec<String> = vec![];

    let config = DuplicationConfig::default();
    let language_detector = create_test_language_detector().await;
    let mut detector = DuplicationDetector::new(config, language_detector);

    let result = detector.detect_duplications(
        temp_dir.path().to_string_lossy().as_ref(),
        &files
    ).await.expect("Failed to detect duplications");

    // 测试空项目的分析
    let distribution = result.generate_distribution();
    let detailed_report = result.generate_detailed_report();

    // 验证空项目的处理
    assert!(result.duplications.is_empty(), "Empty project should have no duplications");
    assert_eq!(result.summary.total_files, 0, "Empty project should have 0 files");
    assert_eq!(result.summary.duplication_ratio, 0.0, "Empty project should have 0% duplication");

    // 验证分布图处理
    assert!(distribution.by_file_size.is_empty(), "Empty project should have empty file size distribution");
    assert!(distribution.by_type_distribution.is_empty(), "Empty project should have empty type distribution");

    // 验证详细报告处理
    assert_eq!(detailed_report.project_overview.health_score, 100.0, "Empty project should have perfect health score");
    assert_eq!(detailed_report.project_overview.duplication_grade, DuplicationGrade::A, "Empty project should have grade A");
    assert!(detailed_report.hotspot_analysis.hotspot_files.is_empty(), "Empty project should have no hotspot files");
}

#[tokio::test]
async fn test_hotspot_pattern_identification() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let files = create_duplication_test_files(&temp_dir).await;

    let config = DuplicationConfig {
        min_duplicate_lines: 3,
        min_duplicate_chars: 50,
        enable_exact_detection: true,
        enable_structural_detection: true,
        enable_cross_file_detection: true,
        ..Default::default()
    };

    let language_detector = create_test_language_detector().await;
    let mut detector = DuplicationDetector::new(config, language_detector);

    let result = detector.detect_duplications(
        temp_dir.path().to_string_lossy().as_ref(),
        &files
    ).await.expect("Failed to detect duplications");

    let detailed_report = result.generate_detailed_report();
    let hotspot_patterns = &detailed_report.hotspot_analysis.hotspot_patterns;

    // 验证热点模式识别
    if !hotspot_patterns.is_empty() {
        for pattern in hotspot_patterns {
            assert!(!pattern.pattern_name.is_empty(), "Pattern should have a name");
            assert!(!pattern.description.is_empty(), "Pattern should have a description");
            assert!(pattern.occurrence_count > 0, "Pattern should have occurrences");
            assert!(!pattern.affected_files.is_empty(), "Pattern should affect files");
        }

        println!("Identified hotspot patterns:");
        for pattern in hotspot_patterns {
            println!("- {}: {} (affects {} files, {} occurrences)",
                    pattern.pattern_name,
                    pattern.description,
                    pattern.affected_files.len(),
                    pattern.occurrence_count);
        }
    }
}