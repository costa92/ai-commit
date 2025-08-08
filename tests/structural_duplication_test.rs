use ai_commit::analysis::duplication::{
    DuplicationDetector, DuplicationConfig, StructuralDuplicationDetector,
    DuplicationType, ProcessedContent
};
use ai_commit::languages::{Language, LanguageDetector};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

/// 测试结构相似性检测的基本功能
#[tokio::test]
async fn test_structural_similarity_detection_basic() {
    let config = DuplicationConfig {
        min_duplicate_lines: 3,
        min_duplicate_chars: 50,
        structural_similarity_threshold: 0.7,
        enable_structural_detection: true,
        ..Default::default()
    };

    let mut detector = StructuralDuplicationDetector::new(&config);

    // 创建结构相似但内容不同的代码示例
    let mut contents = HashMap::new();

    // Rust代码示例1
    contents.insert("file1.rs".to_string(), ProcessedContent {
        original: r#"
fn process_user_data(user: &User) -> Result<String, Error> {
    if user.is_active {
        let result = validate_user(user);
        if result.is_ok() {
            return format_user_info(user);
        }
    }
    Err(Error::InvalidUser)
}
"#.to_string(),
        processed: r#"
fn process_user_data(user: &User) -> Result<String, Error> {
    if user.is_active {
        let result = validate_user(user);
        if result.is_ok() {
            return format_user_info(user);
        }
    }
    Err(Error::InvalidUser)
}
"#.to_string(),
        language: Language::Rust,
        file_path: "file1.rs".to_string(),
    });

    // Rust代码示例2 - 结构相似但变量名不同
    contents.insert("file2.rs".to_string(), ProcessedContent {
        original: r#"
fn handle_customer_request(customer: &Customer) -> Result<String, Error> {
    if customer.is_verified {
        let validation = check_customer(customer);
        if validation.is_ok() {
            return generate_response(customer);
        }
    }
    Err(Error::InvalidCustomer)
}
"#.to_string(),
        processed: r#"
fn handle_customer_request(customer: &Customer) -> Result<String, Error> {
    if customer.is_verified {
        let validation = check_customer(customer);
        if validation.is_ok() {
            return generate_response(customer);
        }
    }
    Err(Error::InvalidCustomer)
}
"#.to_string(),
        language: Language::Rust,
        file_path: "file2.rs".to_string(),
    });

    let duplications = detector.detect(&contents).await.unwrap();

    assert!(!duplications.is_empty(), "应该检测到结构相似的重复代码");

    let structural_duplications: Vec<_> = duplications.iter()
        .filter(|d| d.duplication_type == DuplicationType::Structural)
        .collect();

    assert!(!structural_duplications.is_empty(), "应该检测到结构相似性重复");

    let duplication = &structural_duplications[0];
    assert!(duplication.code_blocks.len() >= 2, "应该有至少两个重复的代码块");
    assert!(duplication.similarity_score >= 0.7, "相似度应该大于阈值");
}

/// 测试Go语言的结构相似性检测
#[tokio::test]
async fn test_go_structural_similarity() {
    let config = DuplicationConfig {
        min_duplicate_lines: 4,
        structural_similarity_threshold: 0.75,
        enable_structural_detection: true,
        ..Default::default()
    };

    let mut detector = StructuralDuplicationDetector::new(&config);

    let mut contents = HashMap::new();

    // Go代码示例1
    contents.insert("handler1.go".to_string(), ProcessedContent {
        original: r#"
func ProcessOrder(order *Order) error {
    if order.Status == "pending" {
        err := ValidateOrder(order)
        if err == nil {
            return SaveOrder(order)
        }
    }
    return errors.New("invalid order")
}
"#.to_string(),
        processed: r#"
func ProcessOrder(order *Order) error {
    if order.Status == "pending" {
        err := ValidateOrder(order)
        if err == nil {
            return SaveOrder(order)
        }
    }
    return errors.New("invalid order")
}
"#.to_string(),
        language: Language::Go,
        file_path: "handler1.go".to_string(),
    });

    // Go代码示例2 - 结构相似
    contents.insert("handler2.go".to_string(), ProcessedContent {
        original: r#"
func HandlePayment(payment *Payment) error {
    if payment.Type == "credit" {
        result := CheckPayment(payment)
        if result == nil {
            return ProcessPayment(payment)
        }
    }
    return errors.New("payment failed")
}
"#.to_string(),
        processed: r#"
func HandlePayment(payment *Payment) error {
    if payment.Type == "credit" {
        result := CheckPayment(payment)
        if result == nil {
            return ProcessPayment(payment)
        }
    }
    return errors.New("payment failed")
}
"#.to_string(),
        language: Language::Go,
        file_path: "handler2.go".to_string(),
    });

    let duplications = detector.detect(&contents).await.unwrap();

    let structural_duplications: Vec<_> = duplications.iter()
        .filter(|d| d.duplication_type == DuplicationType::Structural)
        .collect();

    assert!(!structural_duplications.is_empty(), "应该检测到Go语言的结构相似性");

    let duplication = &structural_duplications[0];
    assert!(duplication.similarity_score >= 0.75, "Go代码相似度应该满足阈值");
}

/// 测试TypeScript的结构相似性检测
#[tokio::test]
async fn test_typescript_structural_similarity() {
    let config = DuplicationConfig {
        min_duplicate_lines: 5,
        structural_similarity_threshold: 0.8,
        enable_structural_detection: true,
        ..Default::default()
    };

    let mut detector = StructuralDuplicationDetector::new(&config);

    let mut contents = HashMap::new();

    // TypeScript代码示例1
    contents.insert("service1.ts".to_string(), ProcessedContent {
        original: r#"
async function fetchUserData(userId: string): Promise<UserData> {
    try {
        const response = await apiClient.get(`/users/${userId}`);
        if (response.status === 200) {
            return response.data;
        }
        throw new Error('User not found');
    } catch (error) {
        console.error('Failed to fetch user:', error);
        throw error;
    }
}
"#.to_string(),
        processed: r#"
async function fetchUserData(userId: string): Promise<UserData> {
    try {
        const response = await apiClient.get(`/users/${userId}`);
        if (response.status === 200) {
            return response.data;
        }
        throw new Error('User not found');
    } catch (error) {
        console.error('Failed to fetch user:', error);
        throw error;
    }
}
"#.to_string(),
        language: Language::TypeScript,
        file_path: "service1.ts".to_string(),
    });

    // TypeScript代码示例2 - 结构相似
    contents.insert("service2.ts".to_string(), ProcessedContent {
        original: r#"
async function loadProductInfo(productId: string): Promise<ProductInfo> {
    try {
        const result = await httpClient.get(`/products/${productId}`);
        if (result.status === 200) {
            return result.data;
        }
        throw new Error('Product not available');
    } catch (err) {
        console.error('Failed to load product:', err);
        throw err;
    }
}
"#.to_string(),
        processed: r#"
async function loadProductInfo(productId: string): Promise<ProductInfo> {
    try {
        const result = await httpClient.get(`/products/${productId}`);
        if (result.status === 200) {
            return result.data;
        }
        throw new Error('Product not available');
    } catch (err) {
        console.error('Failed to load product:', err);
        throw err;
    }
}
"#.to_string(),
        language: Language::TypeScript,
        file_path: "service2.ts".to_string(),
    });

    let duplications = detector.detect(&contents).await.unwrap();

    let structural_duplications: Vec<_> = duplications.iter()
        .filter(|d| d.duplication_type == DuplicationType::Structural)
        .collect();

    assert!(!structural_duplications.is_empty(), "应该检测到TypeScript的结构相似性");

    let duplication = &structural_duplications[0];
    assert!(duplication.similarity_score >= 0.8, "TypeScript代码相似度应该满足阈值");
}

/// 测试复杂控制流的结构相似性检测
#[tokio::test]
async fn test_complex_control_flow_similarity() {
    let config = DuplicationConfig {
        min_duplicate_lines: 8,
        structural_similarity_threshold: 0.7,
        enable_structural_detection: true,
        ..Default::default()
    };

    let mut detector = StructuralDuplicationDetector::new(&config);

    let mut contents = HashMap::new();

    // 复杂控制流示例1
    contents.insert("complex1.rs".to_string(), ProcessedContent {
        original: r#"
fn process_data(items: Vec<Item>) -> Result<Vec<ProcessedItem>, Error> {
    let mut results = Vec::new();

    for item in items {
        if item.is_valid() {
            match item.category {
                Category::A => {
                    let processed = transform_a(item);
                    results.push(processed);
                }
                Category::B => {
                    let processed = transform_b(item);
                    results.push(processed);
                }
                _ => continue,
            }
        } else {
            log::warn!("Invalid item: {:?}", item);
        }
    }

    Ok(results)
}
"#.to_string(),
        processed: r#"
fn process_data(items: Vec<Item>) -> Result<Vec<ProcessedItem>, Error> {
    let mut results = Vec::new();

    for item in items {
        if item.is_valid() {
            match item.category {
                Category::A => {
                    let processed = transform_a(item);
                    results.push(processed);
                }
                Category::B => {
                    let processed = transform_b(item);
                    results.push(processed);
                }
                _ => continue,
            }
        } else {
            log::warn!("Invalid item: {:?}", item);
        }
    }

    Ok(results)
}
"#.to_string(),
        language: Language::Rust,
        file_path: "complex1.rs".to_string(),
    });

    // 复杂控制流示例2 - 结构相似
    contents.insert("complex2.rs".to_string(), ProcessedContent {
        original: r#"
fn handle_requests(requests: Vec<Request>) -> Result<Vec<Response>, Error> {
    let mut responses = Vec::new();

    for request in requests {
        if request.is_authorized() {
            match request.method {
                Method::GET => {
                    let response = handle_get(request);
                    responses.push(response);
                }
                Method::POST => {
                    let response = handle_post(request);
                    responses.push(response);
                }
                _ => continue,
            }
        } else {
            log::error!("Unauthorized request: {:?}", request);
        }
    }

    Ok(responses)
}
"#.to_string(),
        processed: r#"
fn handle_requests(requests: Vec<Request>) -> Result<Vec<Response>, Error> {
    let mut responses = Vec::new();

    for request in requests {
        if request.is_authorized() {
            match request.method {
                Method::GET => {
                    let response = handle_get(request);
                    responses.push(response);
                }
                Method::POST => {
                    let response = handle_post(request);
                    responses.push(response);
                }
                _ => continue,
            }
        } else {
            log::error!("Unauthorized request: {:?}", request);
        }
    }

    Ok(responses)
}
"#.to_string(),
        language: Language::Rust,
        file_path: "complex2.rs".to_string(),
    });

    let duplications = detector.detect(&contents).await.unwrap();

    let structural_duplications: Vec<_> = duplications.iter()
        .filter(|d| d.duplication_type == DuplicationType::Structural)
        .collect();

    assert!(!structural_duplications.is_empty(), "应该检测到复杂控制流的结构相似性");

    let duplication = &structural_duplications[0];
    assert!(duplication.line_count >= 8, "复杂代码块应该有足够的行数");
    assert!(duplication.similarity_score >= 0.7, "复杂控制流相似度应该满足阈值");
}

/// 测试相似度阈值过滤
#[tokio::test]
async fn test_similarity_threshold_filtering() {
    let config = DuplicationConfig {
        min_duplicate_lines: 3,
        structural_similarity_threshold: 0.9, // 高阈值
        enable_structural_detection: true,
        ..Default::default()
    };

    let mut detector = StructuralDuplicationDetector::new(&config);

    let mut contents = HashMap::new();

    // 低相似度的代码示例
    contents.insert("low_sim1.rs".to_string(), ProcessedContent {
        original: r#"
fn simple_function() {
    println!("Hello");
    let x = 1;
    return x;
}
"#.to_string(),
        processed: r#"
fn simple_function() {
    println!("Hello");
    let x = 1;
    return x;
}
"#.to_string(),
        language: Language::Rust,
        file_path: "low_sim1.rs".to_string(),
    });

    contents.insert("low_sim2.rs".to_string(), ProcessedContent {
        original: r#"
fn different_function() {
    if true {
        for i in 0..10 {
            println!("{}", i);
        }
    }
}
"#.to_string(),
        processed: r#"
fn different_function() {
    if true {
        for i in 0..10 {
            println!("{}", i);
        }
    }
}
"#.to_string(),
        language: Language::Rust,
        file_path: "low_sim2.rs".to_string(),
    });

    let duplications = detector.detect(&contents).await.unwrap();

    // 由于相似度阈值很高，低相似度的代码不应该被检测为重复
    let structural_duplications: Vec<_> = duplications.iter()
        .filter(|d| d.duplication_type == DuplicationType::Structural)
        .collect();

    // 这些代码结构差异较大，不应该被高阈值检测为相似
    assert!(structural_duplications.is_empty() ||
           structural_duplications.iter().all(|d| d.similarity_score >= 0.9),
           "高阈值应该过滤掉低相似度的代码");
}

/// 测试缓存功能
#[tokio::test]
async fn test_structural_detector_caching() {
    let config = DuplicationConfig::default();
    let mut detector = StructuralDuplicationDetector::new(&config);

    let mut contents = HashMap::new();
    contents.insert("cache_test.rs".to_string(), ProcessedContent {
        original: r#"
fn test_function() {
    if true {
        let x = 1;
        println!("{}", x);
    }
}
"#.to_string(),
        processed: r#"
fn test_function() {
    if true {
        let x = 1;
        println!("{}", x);
    }
}
"#.to_string(),
        language: Language::Rust,
        file_path: "cache_test.rs".to_string(),
    });

    // 第一次检测
    let _duplications1 = detector.detect(&contents).await.unwrap();
    let stats1 = detector.get_cache_stats();

    // 第二次检测相同内容
    let _duplications2 = detector.detect(&contents).await.unwrap();
    let stats2 = detector.get_cache_stats();

    // 缓存应该被使用
    assert!(stats2.cache_size >= stats1.cache_size, "缓存大小应该保持或增长");

    // 清空缓存
    detector.clear_cache();
    let stats3 = detector.get_cache_stats();
    assert_eq!(stats3.cache_size, 0, "清空后缓存大小应该为0");
}

/// 测试最小要求过滤
#[tokio::test]
async fn test_minimum_requirements_filtering() {
    let config = DuplicationConfig {
        min_duplicate_lines: 10, // 高行数要求
        min_duplicate_chars: 500, // 高字符数要求
        enable_structural_detection: true,
        ..Default::default()
    };

    let mut detector = StructuralDuplicationDetector::new(&config);

    let mut contents = HashMap::new();

    // 小代码块，不满足最小要求
    contents.insert("small1.rs".to_string(), ProcessedContent {
        original: r#"
fn small() {
    let x = 1;
}
"#.to_string(),
        processed: r#"
fn small() {
    let x = 1;
}
"#.to_string(),
        language: Language::Rust,
        file_path: "small1.rs".to_string(),
    });

    contents.insert("small2.rs".to_string(), ProcessedContent {
        original: r#"
fn tiny() {
    let y = 2;
}
"#.to_string(),
        processed: r#"
fn tiny() {
    let y = 2;
}
"#.to_string(),
        language: Language::Rust,
        file_path: "small2.rs".to_string(),
    });

    let duplications = detector.detect(&contents).await.unwrap();

    // 小代码块不应该被检测为重复
    assert!(duplications.is_empty(), "不满足最小要求的代码块不应该被检测");
}

/// 集成测试：结构相似性检测与完整的重复检测器
#[tokio::test]
async fn test_structural_detection_integration() {
    let config = DuplicationConfig {
        enable_exact_detection: true,
        enable_structural_detection: true,
        enable_cross_file_detection: false,
        structural_similarity_threshold: 0.75,
        ..Default::default()
    };

    let language_detector = Arc::new(Mutex::new(LanguageDetector::new()));
    let mut detector = DuplicationDetector::new(config, language_detector);

    // 创建临时文件进行测试
    let temp_dir = tempfile::tempdir().unwrap();
    let file1_path = temp_dir.path().join("test1.rs");
    let file2_path = temp_dir.path().join("test2.rs");

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
"#).await.unwrap();

    tokio::fs::write(&file2_path, r#"
fn compute_sum(products: &[Product]) -> f64 {
    let mut sum = 0.0;
    for product in products {
        if product.is_active {
            sum += product.cost * product.amount;
        }
    }
    sum
}
"#).await.unwrap();

    let files = vec![
        file1_path.to_string_lossy().to_string(),
        file2_path.to_string_lossy().to_string(),
    ];

    let result = detector.detect_duplications(
        temp_dir.path().to_string_lossy().as_ref(),
        &files
    ).await.unwrap();

    // 应该检测到结构相似的重复
    assert!(!result.duplications.is_empty(), "应该检测到重复代码");

    let structural_duplications: Vec<_> = result.duplications.iter()
        .filter(|d| d.duplication_type == DuplicationType::Structural)
        .collect();

    assert!(!structural_duplications.is_empty(), "应该检测到结构相似性重复");

    // 验证统计摘要
    assert!(result.summary.total_duplications > 0, "统计摘要应该包含重复信息");
    assert!(result.summary.by_type.contains_key(&DuplicationType::Structural),
           "统计摘要应该包含结构相似性类型");
}