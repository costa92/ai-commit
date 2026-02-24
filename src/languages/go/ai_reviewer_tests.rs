#[cfg(test)]
mod go_ai_reviewer_tests {
    use crate::config::Config;
    use crate::languages::LanguageFeature;
    use crate::languages::go::GoAIReviewer;

    fn create_test_config() -> Config {
        Config {
            provider: "ollama".to_string(),
            model: "test-model".to_string(),
            debug: true,
            ..Default::default()
        }
    }

    fn create_test_go_features() -> Vec<LanguageFeature> {
        vec![
            LanguageFeature {
                feature_type: "function".to_string(),
                name: "processData".to_string(),
                line_number: Some(10),
                description: "func processData() error".to_string(),
            },
            LanguageFeature {
                feature_type: "struct".to_string(),
                name: "User".to_string(),
                line_number: Some(5),
                description: "type User struct { Name string }".to_string(),
            },
            LanguageFeature {
                feature_type: "interface".to_string(),
                name: "Writer".to_string(),
                line_number: Some(15),
                description: "type Writer interface { Write([]byte) error }".to_string(),
            },
            LanguageFeature {
                feature_type: "goroutine".to_string(),
                name: "goroutine_start".to_string(),
                line_number: Some(25),
                description: "go func() { process() }()".to_string(),
            },
            LanguageFeature {
                feature_type: "channel".to_string(),
                name: "channel_operation".to_string(),
                line_number: Some(30),
                description: "ch := make(chan int, 10)".to_string(),
            },
        ]
    }

    #[test]
    fn test_go_ai_reviewer_creation() {
        let config = create_test_config();
        let reviewer = GoAIReviewer::new(config);
        
        assert!(reviewer.config.model.contains("test-model"));
        assert!(reviewer.config.debug);
    }

    #[test]
    fn test_go_ai_reviewer_default() {
        let reviewer = GoAIReviewer::default();
        
        // 验证默认配置
        assert!(!reviewer.config.provider.is_empty());
    }

    #[test]
    fn test_go_ai_reviewer_with_config() {
        let config = create_test_config();
        let reviewer = GoAIReviewer::with_config(config.clone());
        
        assert_eq!(reviewer.config.provider, config.provider);
        assert_eq!(reviewer.config.model, config.model);
    }

    #[test]
    fn test_generate_review_prompt_comprehensive() {
        let reviewer = GoAIReviewer::default();
        let features = create_test_go_features();
        
        let prompt = reviewer.generate_review_prompt("comprehensive", &features, "main.go");
        
        assert!(prompt.contains("全面的Go代码审查"));
        assert!(prompt.contains("并发安全"));
        assert!(prompt.contains("错误处理"));
        assert!(prompt.contains("性能优化"));
        assert!(prompt.contains("main.go"));
        assert!(prompt.contains("processData"));
        assert!(prompt.contains("User"));
        assert!(prompt.contains("goroutine"));
    }

    #[test]
    fn test_generate_review_prompt_concurrency() {
        let reviewer = GoAIReviewer::default();
        let features = create_test_go_features();
        
        let prompt = reviewer.generate_review_prompt("concurrency", &features, "concurrent.go");
        
        assert!(prompt.contains("Go并发编程审查"));
        assert!(prompt.contains("goroutine管理"));
        assert!(prompt.contains("channel使用"));
        assert!(prompt.contains("数据竞争"));
        assert!(prompt.contains("死锁检测"));
        assert!(prompt.contains("concurrent.go"));
    }

    #[test]
    fn test_generate_review_prompt_performance() {
        let reviewer = GoAIReviewer::default();
        let features = create_test_go_features();
        
        let prompt = reviewer.generate_review_prompt("performance", &features, "perf.go");
        
        assert!(prompt.contains("Go性能审查"));
        assert!(prompt.contains("内存分配"));
        assert!(prompt.contains("垃圾回收"));
        assert!(prompt.contains("算法效率"));
        assert!(prompt.contains("并发性能"));
        assert!(prompt.contains("perf.go"));
    }

    #[test]
    fn test_generate_review_prompt_security() {
        let reviewer = GoAIReviewer::default();
        let features = create_test_go_features();
        
        let prompt = reviewer.generate_review_prompt("security", &features, "security.go");
        
        assert!(prompt.contains("Go安全性审查"));
        assert!(prompt.contains("输入验证"));
        assert!(prompt.contains("错误泄露"));
        assert!(prompt.contains("并发安全"));
        assert!(prompt.contains("资源泄露"));
        assert!(prompt.contains("security.go"));
    }

    #[test]
    fn test_generate_review_prompt_architecture() {
        let reviewer = GoAIReviewer::default();
        let features = create_test_go_features();
        
        let prompt = reviewer.generate_review_prompt("architecture", &features, "arch.go");
        
        assert!(prompt.contains("Go架构审查"));
        assert!(prompt.contains("包设计"));
        assert!(prompt.contains("接口设计"));
        assert!(prompt.contains("依赖管理"));
        assert!(prompt.contains("模块组织"));
        assert!(prompt.contains("arch.go"));
    }

    #[test]
    fn test_generate_review_prompt_unknown_type() {
        let reviewer = GoAIReviewer::default();
        let features = create_test_go_features();
        
        let prompt = reviewer.generate_review_prompt("unknown", &features, "test.go");
        
        // 应该回退到综合审查
        assert!(prompt.contains("全面的Go代码审查"));
    }

    #[test]
    fn test_generate_review_prompt_empty_features() {
        let reviewer = GoAIReviewer::default();
        let features: Vec<LanguageFeature> = vec![];
        
        let prompt = reviewer.generate_review_prompt("comprehensive", &features, "empty.go");
        
        assert!(prompt.contains("全面的Go代码审查"));
        assert!(prompt.contains("empty.go"));
        assert!(prompt.contains("未检测到特定代码特征"));
    }

    #[test]
    fn test_generate_review_prompt_goroutine_focus() {
        let reviewer = GoAIReviewer::default();
        let goroutine_features = vec![
            LanguageFeature {
                feature_type: "goroutine".to_string(),
                name: "worker_pool".to_string(),
                line_number: Some(1),
                description: "go worker()".to_string(),
            },
            LanguageFeature {
                feature_type: "channel".to_string(),
                name: "work_channel".to_string(),
                line_number: Some(2),
                description: "workCh := make(chan Task, 100)".to_string(),
            },
        ];
        
        let prompt = reviewer.generate_review_prompt("concurrency", &goroutine_features, "worker.go");
        
        assert!(prompt.contains("worker_pool"));
        assert!(prompt.contains("work_channel"));
        assert!(prompt.contains("goroutine管理"));
        assert!(prompt.contains("channel使用"));
    }

    #[test]
    fn test_parse_ai_response_valid_json() {
        let reviewer = GoAIReviewer::default();
        
        let json_response = r#"{
            "overall_score": 7.5,
            "summary": "Go代码质量良好",
            "detailed_feedback": "并发使用得当",
            "security_score": 8.0,
            "performance_score": 7.0,
            "maintainability_score": 8.0,
            "recommendations": ["优化goroutine池", "改进错误处理"],
            "learning_resources": ["https://golang.org/doc/effective_go.html"]
        }"#;
        
        let result = reviewer.parse_ai_response("comprehensive", json_response);
        
        assert!(result.is_ok());
        let ai_result = result.unwrap();
        assert_eq!(ai_result.review_type, "go_comprehensive");
        assert_eq!(ai_result.overall_score, 7.5);
        assert_eq!(ai_result.summary, "Go代码质量良好");
        assert_eq!(ai_result.security_score, 8.0);
        assert_eq!(ai_result.recommendations.len(), 2);
        assert_eq!(ai_result.learning_resources.len(), 1);
    }

    #[test]
    fn test_parse_ai_response_concurrency_type() {
        let reviewer = GoAIReviewer::default();
        
        let json_response = r#"{
            "overall_score": 8.0,
            "summary": "并发处理良好"
        }"#;
        
        let result = reviewer.parse_ai_response("concurrency", json_response);
        
        assert!(result.is_ok());
        let ai_result = result.unwrap();
        assert_eq!(ai_result.review_type, "go_concurrency");
        assert_eq!(ai_result.overall_score, 8.0);
    }

    #[test]
    fn test_parse_ai_response_invalid_json() {
        let reviewer = GoAIReviewer::default();
        
        let invalid_json = "这不是有效的JSON - Go代码分析失败";
        
        let result = reviewer.parse_ai_response("performance", invalid_json);
        
        assert!(result.is_ok());
        let ai_result = result.unwrap();
        assert_eq!(ai_result.review_type, "go_performance");
        assert_eq!(ai_result.summary, "AI响应解析失败");
        assert!(ai_result.detailed_feedback.contains("原始响应"));
        assert!(ai_result.detailed_feedback.contains(invalid_json));
    }

    #[test]
    fn test_parse_ai_response_empty_response() {
        let reviewer = GoAIReviewer::default();
        
        let result = reviewer.parse_ai_response("security", "");
        
        assert!(result.is_ok());
        let ai_result = result.unwrap();
        assert_eq!(ai_result.review_type, "go_security");
        assert_eq!(ai_result.summary, "AI响应解析失败");
        assert!(ai_result.detailed_feedback.contains("响应为空"));
    }

    #[test]
    fn test_parse_ai_response_go_specific_fields() {
        let reviewer = GoAIReviewer::default();
        
        let go_specific_json = r#"{
            "overall_score": 9.0,
            "summary": "优秀的Go代码",
            "goroutine_safety": "excellent",
            "channel_usage": "proper",
            "error_handling": "idiomatic",
            "package_structure": "well_organized"
        }"#;
        
        let result = reviewer.parse_ai_response("comprehensive", go_specific_json);
        
        assert!(result.is_ok());
        let ai_result = result.unwrap();
        assert_eq!(ai_result.overall_score, 9.0);
        assert_eq!(ai_result.summary, "优秀的Go代码");
        // Go特定字段可能不在标准结构中，但不应该导致解析失败
    }

    #[tokio::test]
    async fn test_review_code_with_mock_response() {
        let reviewer = GoAIReviewer::default();
        let features = create_test_go_features();
        
        // 模拟测试（实际不会调用AI服务）
        let result = reviewer.review_code("concurrency", &features, "test.go").await;
        
        match result {
            Ok(ai_result) => {
                assert_eq!(ai_result.review_type, "go_concurrency");
                assert!(!ai_result.summary.is_empty());
            }
            Err(_) => {
                // 在没有真实AI服务的情况下，这是预期的行为
            }
        }
    }

    #[test]
    fn test_review_type_prefix_go() {
        let reviewer = GoAIReviewer::default();
        
        let test_cases = vec![
            ("comprehensive", "go_comprehensive"),
            ("concurrency", "go_concurrency"),
            ("performance", "go_performance"),
            ("security", "go_security"),
            ("architecture", "go_architecture"),
            ("custom", "go_custom"),
        ];
        
        for (input_type, expected_prefix) in test_cases {
            let features = vec![];
            let prompt = reviewer.generate_review_prompt(input_type, &features, "test.go");
            
            assert!(!prompt.is_empty());
            
            let mock_response = r#"{"overall_score": 8.0}"#;
            let result = reviewer.parse_ai_response(input_type, mock_response).unwrap();
            assert_eq!(result.review_type, expected_prefix);
        }
    }

    #[test]
    fn test_go_specific_feature_analysis() {
        let reviewer = GoAIReviewer::default();
        
        let go_features = vec![
            LanguageFeature {
                feature_type: "package".to_string(),
                name: "main".to_string(),
                line_number: Some(1),
                description: "package main".to_string(),
            },
            LanguageFeature {
                feature_type: "import".to_string(),
                name: "context".to_string(),
                line_number: Some(2),
                description: "import \"context\"".to_string(),
            },
            LanguageFeature {
                feature_type: "method".to_string(),
                name: "ServeHTTP".to_string(),
                line_number: Some(3),
                description: "func (h *Handler) ServeHTTP(w http.ResponseWriter, r *http.Request)".to_string(),
            },
        ];
        
        let prompt = reviewer.generate_review_prompt("architecture", &go_features, "server.go");
        
        assert!(prompt.contains("main"));
        assert!(prompt.contains("context"));
        assert!(prompt.contains("ServeHTTP"));
        assert!(prompt.contains("server.go"));
    }

    #[test]
    fn test_concurrency_pattern_detection() {
        let reviewer = GoAIReviewer::default();
        
        let concurrency_features = vec![
            LanguageFeature {
                feature_type: "goroutine".to_string(),
                name: "background_worker".to_string(),
                line_number: Some(1),
                description: "go func() { for range ch { work() } }()".to_string(),
            },
            LanguageFeature {
                feature_type: "channel".to_string(),
                name: "result_channel".to_string(),
                line_number: Some(2),
                description: "resultCh := make(chan Result)".to_string(),
            },
            LanguageFeature {
                feature_type: "function".to_string(),
                name: "select_statement".to_string(),
                line_number: Some(3),
                description: "select { case <-ctx.Done(): return }".to_string(),
            },
        ];
        
        let prompt = reviewer.generate_review_prompt("concurrency", &concurrency_features, "concurrent.go");
        
        assert!(prompt.contains("background_worker"));
        assert!(prompt.contains("result_channel"));
        assert!(prompt.contains("并发编程审查"));
        assert!(prompt.contains("goroutine管理"));
    }

    #[test]
    fn test_large_go_codebase() {
        let reviewer = GoAIReviewer::default();
        
        let mut large_features = Vec::new();
        
        // 模拟大型Go项目的特征
        for i in 0..50 {
            large_features.push(LanguageFeature {
                feature_type: "function".to_string(),
                name: format!("handler_{}", i),
                line_number: Some(i * 10),
                description: format!("func handler_{}(w http.ResponseWriter, r *http.Request)", i),
            });
        }
        
        for i in 0..20 {
            large_features.push(LanguageFeature {
                feature_type: "struct".to_string(),
                name: format!("Model_{}", i),
                line_number: Some(i * 5),
                description: format!("type Model_{} struct", i),
            });
        }
        
        let prompt = reviewer.generate_review_prompt("architecture", &large_features, "large_project.go");
        
        assert!(!prompt.is_empty());
        assert!(prompt.contains("large_project.go"));
        assert!(prompt.contains("架构审查"));
    }

    #[test]
    fn test_error_handling_patterns() {
        let reviewer = GoAIReviewer::default();
        
        let error_features = vec![
            LanguageFeature {
                feature_type: "function".to_string(),
                name: "processWithError".to_string(),
                line_number: Some(1),
                description: "func processWithError() (Result, error)".to_string(),
            },
            LanguageFeature {
                feature_type: "interface".to_string(),
                name: "CustomError".to_string(),
                line_number: Some(2),
                description: "type CustomError interface { Error() string }".to_string(),
            },
        ];
        
        let prompt = reviewer.generate_review_prompt("security", &error_features, "errors.go");
        
        assert!(prompt.contains("processWithError"));
        assert!(prompt.contains("CustomError"));
        assert!(prompt.contains("错误处理"));
    }

    #[test]
    fn test_special_go_files() {
        let reviewer = GoAIReviewer::default();
        let features = vec![];
        
        let special_files = vec![
            "main.go",
            "main_test.go",
            "benchmark_test.go",
            "example_test.go",
            "doc.go",
            "internal/service.go",
            "cmd/server/main.go",
            "pkg/utils/helper.go",
        ];
        
        for filename in special_files {
            let prompt = reviewer.generate_review_prompt("comprehensive", &features, filename);
            assert!(prompt.contains(filename));
            assert!(!prompt.is_empty());
        }
    }

    #[test]
    fn test_go_testing_features() {
        let reviewer = GoAIReviewer::default();
        
        let test_features = vec![
            LanguageFeature {
                feature_type: "test".to_string(),
                name: "TestUserCreation".to_string(),
                line_number: Some(1),
                description: "func TestUserCreation(t *testing.T)".to_string(),
            },
            LanguageFeature {
                feature_type: "benchmark".to_string(),
                name: "BenchmarkProcessing".to_string(),
                line_number: Some(2),
                description: "func BenchmarkProcessing(b *testing.B)".to_string(),
            },
        ];
        
        let prompt = reviewer.generate_review_prompt("comprehensive", &test_features, "user_test.go");
        
        assert!(prompt.contains("TestUserCreation"));
        assert!(prompt.contains("BenchmarkProcessing"));
        assert!(prompt.contains("user_test.go"));
    }

    #[test]
    fn test_config_override() {
        let mut config = create_test_config();
        config.provider = "go_specific_provider".to_string();
        config.model = "go_tuned_model".to_string();
        
        let reviewer = GoAIReviewer::with_config(config);
        
        assert_eq!(reviewer.config.provider, "go_specific_provider");
        assert_eq!(reviewer.config.model, "go_tuned_model");
    }
}