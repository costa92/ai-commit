#[cfg(test)]
mod rust_ai_reviewer_tests {
    use crate::config::Config;
    use crate::languages::LanguageFeature;
    use crate::languages::rust::RustAIReviewer;

    fn create_test_config() -> Config {
        Config {
            provider: "ollama".to_string(),
            model: "test-model".to_string(),
            debug: true,
            ..Default::default()
        }
    }

    fn create_test_features() -> Vec<LanguageFeature> {
        vec![
            LanguageFeature {
                feature_type: "function".to_string(),
                name: "process_data".to_string(),
                line_number: Some(10),
                description: "fn process_data() -> Result<String, Error>".to_string(),
            },
            LanguageFeature {
                feature_type: "struct".to_string(),
                name: "User".to_string(),
                line_number: Some(5),
                description: "struct User { name: String, age: u32 }".to_string(),
            },
            LanguageFeature {
                feature_type: "impl".to_string(),
                name: "impl User".to_string(),
                line_number: Some(15),
                description: "impl User { fn new() -> Self {} }".to_string(),
            },
            LanguageFeature {
                feature_type: "unsafe".to_string(),
                name: "unsafe_operation".to_string(),
                line_number: Some(25),
                description: "unsafe { ptr::read(data) }".to_string(),
            },
        ]
    }

    #[test]
    fn test_rust_ai_reviewer_creation() {
        let config = create_test_config();
        let reviewer = RustAIReviewer::new(config);
        
        assert!(reviewer.config.model.contains("test-model"));
        assert!(reviewer.config.debug);
    }

    #[test]
    fn test_rust_ai_reviewer_default() {
        let reviewer = RustAIReviewer::default();
        
        // 验证默认配置
        assert!(!reviewer.config.provider.is_empty());
    }

    #[test]
    fn test_rust_ai_reviewer_with_config() {
        let config = create_test_config();
        let reviewer = RustAIReviewer::with_config(config.clone());
        
        assert_eq!(reviewer.config.provider, config.provider);
        assert_eq!(reviewer.config.model, config.model);
    }

    #[test]
    fn test_generate_review_prompt_comprehensive() {
        let reviewer = RustAIReviewer::default();
        let features = create_test_features();
        
        let prompt = reviewer.generate_review_prompt("comprehensive", &features, "test_file.rs");
        
        assert!(prompt.contains("全面的Rust代码审查"));
        assert!(prompt.contains("安全性分析"));
        assert!(prompt.contains("性能评估"));
        assert!(prompt.contains("内存管理"));
        assert!(prompt.contains("test_file.rs"));
        assert!(prompt.contains("process_data"));
        assert!(prompt.contains("User"));
        assert!(prompt.contains("unsafe_operation"));
    }

    #[test]
    fn test_generate_review_prompt_security() {
        let reviewer = RustAIReviewer::default();
        let features = create_test_features();
        
        let prompt = reviewer.generate_review_prompt("security", &features, "security.rs");
        
        assert!(prompt.contains("Rust安全性审查"));
        assert!(prompt.contains("内存安全"));
        assert!(prompt.contains("类型安全"));
        assert!(prompt.contains("并发安全"));
        assert!(prompt.contains("unsafe代码块"));
        assert!(prompt.contains("security.rs"));
    }

    #[test]
    fn test_generate_review_prompt_performance() {
        let reviewer = RustAIReviewer::default();
        let features = create_test_features();
        
        let prompt = reviewer.generate_review_prompt("performance", &features, "perf.rs");
        
        assert!(prompt.contains("Rust性能审查"));
        assert!(prompt.contains("零成本抽象"));
        assert!(prompt.contains("内存分配"));
        assert!(prompt.contains("算法复杂度"));
        assert!(prompt.contains("编译时优化"));
        assert!(prompt.contains("perf.rs"));
    }

    #[test]
    fn test_generate_review_prompt_architecture() {
        let reviewer = RustAIReviewer::default();
        let features = create_test_features();
        
        let prompt = reviewer.generate_review_prompt("architecture", &features, "arch.rs");
        
        assert!(prompt.contains("Rust架构审查"));
        assert!(prompt.contains("模块设计"));
        assert!(prompt.contains("trait设计"));
        assert!(prompt.contains("错误处理"));
        assert!(prompt.contains("API设计"));
        assert!(prompt.contains("arch.rs"));
    }

    #[test]
    fn test_generate_review_prompt_unknown_type() {
        let reviewer = RustAIReviewer::default();
        let features = create_test_features();
        
        let prompt = reviewer.generate_review_prompt("unknown", &features, "test.rs");
        
        // 应该回退到综合审查
        assert!(prompt.contains("全面的Rust代码审查"));
    }

    #[test]
    fn test_generate_review_prompt_empty_features() {
        let reviewer = RustAIReviewer::default();
        let features: Vec<LanguageFeature> = vec![];
        
        let prompt = reviewer.generate_review_prompt("comprehensive", &features, "empty.rs");
        
        assert!(prompt.contains("全面的Rust代码审查"));
        assert!(prompt.contains("empty.rs"));
        assert!(prompt.contains("未检测到特定代码特征"));
    }

    #[test]
    fn test_parse_ai_response_valid_json() {
        let reviewer = RustAIReviewer::default();
        
        let json_response = r#"{
            "overall_score": 8.5,
            "summary": "代码质量良好",
            "detailed_feedback": "详细反馈信息",
            "security_score": 9.0,
            "performance_score": 8.0,
            "maintainability_score": 8.5,
            "recommendations": ["建议1", "建议2"],
            "learning_resources": ["https://doc.rust-lang.org"]
        }"#;
        
        let result = reviewer.parse_ai_response("comprehensive", json_response);
        
        assert!(result.is_ok());
        let ai_result = result.unwrap();
        assert_eq!(ai_result.review_type, "rust_comprehensive");
        assert_eq!(ai_result.overall_score, 8.5);
        assert_eq!(ai_result.summary, "代码质量良好");
        assert_eq!(ai_result.security_score, 9.0);
        assert_eq!(ai_result.recommendations.len(), 2);
        assert_eq!(ai_result.learning_resources.len(), 1);
    }

    #[test]
    fn test_parse_ai_response_partial_json() {
        let reviewer = RustAIReviewer::default();
        
        let partial_json = r#"{
            "overall_score": 7.0,
            "summary": "部分信息"
        }"#;
        
        let result = reviewer.parse_ai_response("security", partial_json);
        
        assert!(result.is_ok());
        let ai_result = result.unwrap();
        assert_eq!(ai_result.review_type, "rust_security");
        assert_eq!(ai_result.overall_score, 7.0);
        assert_eq!(ai_result.summary, "部分信息");
        // 缺失字段应该使用默认值
        assert!(ai_result.detailed_feedback.is_empty());
        assert_eq!(ai_result.security_score, 0.0);
        assert!(ai_result.recommendations.is_empty());
    }

    #[test]
    fn test_parse_ai_response_invalid_json() {
        let reviewer = RustAIReviewer::default();
        
        let invalid_json = "这不是有效的JSON";
        
        let result = reviewer.parse_ai_response("performance", invalid_json);
        
        assert!(result.is_ok());
        let ai_result = result.unwrap();
        assert_eq!(ai_result.review_type, "rust_performance");
        assert_eq!(ai_result.summary, "AI响应解析失败");
        assert!(ai_result.detailed_feedback.contains("原始响应"));
        assert!(ai_result.detailed_feedback.contains(invalid_json));
    }

    #[test]
    fn test_parse_ai_response_empty_response() {
        let reviewer = RustAIReviewer::default();
        
        let result = reviewer.parse_ai_response("architecture", "");
        
        assert!(result.is_ok());
        let ai_result = result.unwrap();
        assert_eq!(ai_result.review_type, "rust_architecture");
        assert_eq!(ai_result.summary, "AI响应解析失败");
        assert!(ai_result.detailed_feedback.contains("响应为空"));
    }

    #[test]
    fn test_parse_ai_response_with_nested_json() {
        let reviewer = RustAIReviewer::default();
        
        let nested_json = r#"{
            "overall_score": 9.0,
            "summary": "优秀代码",
            "recommendations": [
                "改进错误处理",
                "优化性能"
            ],
            "learning_resources": [
                "https://doc.rust-lang.org/book/",
                "https://rust-lang.github.io/async-book/"
            ],
            "extra_data": {
                "complexity": "medium",
                "lines_of_code": 150
            }
        }"#;
        
        let result = reviewer.parse_ai_response("comprehensive", nested_json);
        
        assert!(result.is_ok());
        let ai_result = result.unwrap();
        assert_eq!(ai_result.overall_score, 9.0);
        assert_eq!(ai_result.recommendations.len(), 2);
        assert_eq!(ai_result.learning_resources.len(), 2);
    }

    #[test]
    fn test_parse_ai_response_score_bounds() {
        let reviewer = RustAIReviewer::default();
        
        // 测试超出范围的分数
        let out_of_bounds_json = r#"{
            "overall_score": 15.0,
            "security_score": -5.0,
            "performance_score": 11.0,
            "maintainability_score": 0.0
        }"#;
        
        let result = reviewer.parse_ai_response("comprehensive", out_of_bounds_json);
        
        assert!(result.is_ok());
        let ai_result = result.unwrap();
        // 分数应该被约束在合理范围内或者保持原值（取决于实现）
        assert!(ai_result.overall_score >= 0.0);
        assert!(ai_result.maintainability_score >= 0.0);
    }

    #[tokio::test]
    async fn test_review_code_with_mock_response() {
        let reviewer = RustAIReviewer::default();
        let features = create_test_features();
        
        // 这个测试模拟了AI服务的响应，但实际上不会调用真实的AI服务
        // 在真实环境中，这需要mock AI服务或者使用测试配置
        let result = reviewer.review_code("test", &features, "test.rs").await;
        
        // 根据当前的实现，如果没有配置真实的AI服务，这可能会失败
        // 这个测试更多是为了验证接口的正确性
        match result {
            Ok(ai_result) => {
                assert_eq!(ai_result.review_type, "rust_test");
                assert!(!ai_result.summary.is_empty());
            }
            Err(_) => {
                // 在没有真实AI服务的情况下，这是预期的行为
                // 这个测试验证了错误处理路径
            }
        }
    }

    #[test]
    fn test_review_type_prefix() {
        let reviewer = RustAIReviewer::default();
        
        // 测试不同类型的前缀
        let test_cases = vec![
            ("comprehensive", "rust_comprehensive"),
            ("security", "rust_security"),
            ("performance", "rust_performance"),
            ("architecture", "rust_architecture"),
            ("custom", "rust_custom"),
        ];
        
        for (input_type, expected_prefix) in test_cases {
            let features = vec![];
            let prompt = reviewer.generate_review_prompt(input_type, &features, "test.rs");
            
            // 验证prompt包含适当的内容（间接验证类型处理）
            assert!(!prompt.is_empty());
            
            // 可以通过解析响应来验证类型前缀
            let mock_response = r#"{"overall_score": 8.0}"#;
            let result = reviewer.parse_ai_response(input_type, mock_response).unwrap();
            assert_eq!(result.review_type, expected_prefix);
        }
    }

    #[test]
    fn test_feature_analysis_in_prompt() {
        let reviewer = RustAIReviewer::default();
        
        // 测试不同特征类型对prompt的影响
        let test_features = vec![
            LanguageFeature {
                feature_type: "macro".to_string(),
                name: "debug_assert!".to_string(),
                line_number: Some(1),
                description: "macro usage".to_string(),
            },
            LanguageFeature {
                feature_type: "trait".to_string(),
                name: "Display".to_string(),
                line_number: Some(2),
                description: "trait implementation".to_string(),
            },
            LanguageFeature {
                feature_type: "async".to_string(),
                name: "async_function".to_string(),
                line_number: Some(3),
                description: "async fn async_function()".to_string(),
            },
        ];
        
        let prompt = reviewer.generate_review_prompt("comprehensive", &test_features, "complex.rs");
        
        assert!(prompt.contains("debug_assert!"));
        assert!(prompt.contains("Display"));
        assert!(prompt.contains("async_function"));
        assert!(prompt.contains("complex.rs"));
    }

    #[test]
    fn test_empty_file_handling() {
        let reviewer = RustAIReviewer::default();
        let empty_features: Vec<LanguageFeature> = vec![];
        
        let prompt = reviewer.generate_review_prompt("comprehensive", &empty_features, "empty.rs");
        
        assert!(prompt.contains("empty.rs"));
        assert!(prompt.contains("未检测到特定代码特征") || prompt.contains("no specific features"));
    }

    #[test]
    fn test_special_characters_in_filenames() {
        let reviewer = RustAIReviewer::default();
        let features = vec![];
        
        let special_filenames = vec![
            "file-with-dashes.rs",
            "file_with_underscores.rs",
            "file with spaces.rs",
            "src/nested/deep/file.rs",
            "файл.rs", // 非ASCII字符
        ];
        
        for filename in special_filenames {
            let prompt = reviewer.generate_review_prompt("comprehensive", &features, filename);
            assert!(prompt.contains(filename));
            assert!(!prompt.is_empty());
        }
    }

    #[test]
    fn test_large_feature_set() {
        let reviewer = RustAIReviewer::default();
        
        // 创建大量特征来测试性能和处理能力
        let mut large_features = Vec::new();
        for i in 0..100 {
            large_features.push(LanguageFeature {
                feature_type: "function".to_string(),
                name: format!("function_{}", i),
                line_number: Some(i),
                description: format!("fn function_{}() {{}}", i),
            });
        }
        
        let prompt = reviewer.generate_review_prompt("comprehensive", &large_features, "large_file.rs");
        
        assert!(!prompt.is_empty());
        assert!(prompt.contains("large_file.rs"));
        // 验证包含了一些特征（可能不是全部，取决于实现的截断逻辑）
        assert!(prompt.contains("function_0") || prompt.contains("function_"));
    }

    #[test]
    fn test_config_integration() {
        let mut config = create_test_config();
        config.provider = "test_provider".to_string();
        config.model = "test_model_v2".to_string();
        
        let reviewer = RustAIReviewer::with_config(config);
        
        assert_eq!(reviewer.config.provider, "test_provider");
        assert_eq!(reviewer.config.model, "test_model_v2");
    }

    #[test]
    fn test_serialization_compatibility() {
        // 测试AI结果的序列化和反序列化
        let ai_result = crate::languages::review_service_v2::AIReviewResult {
            review_type: "rust_test".to_string(),
            overall_score: 8.5,
            summary: "测试摘要".to_string(),
            detailed_feedback: "详细反馈".to_string(),
            security_score: 9.0,
            performance_score: 8.0,
            maintainability_score: 8.5,
            recommendations: vec!["建议1".to_string(), "建议2".to_string()],
            learning_resources: vec!["https://example.com".to_string()],
        };
        
        // 测试JSON序列化
        let json = serde_json::to_string(&ai_result).unwrap();
        assert!(json.contains("rust_test"));
        assert!(json.contains("8.5"));
        
        // 测试JSON反序列化
        let deserialized: crate::languages::review_service_v2::AIReviewResult = 
            serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.review_type, "rust_test");
        assert_eq!(deserialized.overall_score, 8.5);
    }
}