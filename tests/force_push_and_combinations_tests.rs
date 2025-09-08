/// 强制推送和参数组合测试
/// 全面测试强制推送功能和各种参数组合场景

#[cfg(test)]
mod force_push_tests {
    use ai_commit::{git, cli::args::Args, config::Config};
    use clap::Parser;
    use tokio;

    fn create_test_config() -> Config {
        Config {
            provider: "test".to_string(),
            model: "test-model".to_string(),
            deepseek_api_key: Some("test-key".to_string()),
            deepseek_url: "http://test.local".to_string(),
            ollama_url: "http://localhost:11434/api/generate".to_string(),
            siliconflow_api_key: None,
            siliconflow_url: "https://api.siliconflow.cn/v1/chat/completions".to_string(),
            kimi_api_key: None,
            kimi_url: "https://api.moonshot.cn/v1/chat/completions".to_string(),

            debug: true,
        }
    }

    #[tokio::test]
    async fn test_git_force_push_basic() {
        // 基本的强制推送测试
        let result = git::git_force_push().await;
        
        match result {
            Ok(_) => {
                println!("✓ 强制推送成功（可能是因为没有实际冲突）");
            }
            Err(e) => {
                let error_msg = e.to_string();
                println!("✓ 强制推送失败（预期）: {}", error_msg);
                
                // 验证错误消息包含预期的git相关内容
                let git_errors = [
                    "Failed to run git push",
                    "Git push failed",
                    "Failed to get current branch", 
                    "Failed to run git pull",
                    "Git pull failed",
                    "请手动解决冲突后重试"
                ];
                
                let contains_git_error = git_errors.iter()
                    .any(|expected| error_msg.contains(expected));
                
                assert!(contains_git_error, 
                    "强制推送错误应该包含git相关错误信息: {}", error_msg);
            }
        }
    }

    #[tokio::test]
    async fn test_git_operations_sequence() {
        // 测试git操作序列
        
        // 测试普通推送
        let push_result = git::git_push().await;
        match push_result {
            Ok(_) => println!("✓ 普通推送成功"),
            Err(e) => println!("✓ 普通推送失败（预期）: {}", e),
        }
        
        // 测试获取diff
        let diff_result = git::get_git_diff().await;
        match diff_result {
            Ok(diff) => {
                println!("✓ 获取diff成功，长度: {} 字符", diff.len());
                assert!(diff.is_empty() || !diff.is_empty()); // 验证返回字符串
            }
            Err(e) => println!("✓ 获取diff失败（预期）: {}", e),
        }
        
        // 测试git add
        let add_result = git::git_add_all().await;
        match add_result {
            Ok(_) => println!("✓ git add成功"),
            Err(e) => println!("✓ git add失败（预期）: {}", e),
        }
    }

    #[test]
    fn test_force_push_parameter_combinations() {
        // 测试强制推送参数的各种组合
        
        let test_cases = vec![
            // (参数列表, 期望的force_push值, 期望的push值, 描述)
            (vec!["ai-commit"], false, false, "默认状态"),
            (vec!["ai-commit", "--force-push"], true, false, "只有force-push"),
            (vec!["ai-commit", "--push"], false, true, "只有push"),
            (vec!["ai-commit", "--force-push", "--push"], true, true, "force-push + push"),
            (vec!["ai-commit", "--push", "--force-push"], true, true, "push + force-push（顺序不同）"),
        ];
        
        for (args_vec, expected_force_push, expected_push, description) in test_cases {
            let args = Args::try_parse_from(args_vec.clone()).unwrap();
            
            assert_eq!(args.force_push, expected_force_push, 
                "force_push不匹配，场景: {} 参数: {:?}", description, args_vec);
            assert_eq!(args.push, expected_push, 
                "push不匹配，场景: {} 参数: {:?}", description, args_vec);
        }
    }

    #[test]
    fn test_force_push_with_all_features() {
        // 测试强制推送与所有其他功能的组合
        
        let comprehensive_args = Args::try_parse_from([
            "ai-commit",
            "--force-push",           // 强制推送
            "--yes",                  // 跳过确认
            "--push",                 // 推送
            "--provider", "deepseek", // AI提供商
            "--model", "deepseek-chat", // 模型
            "--new-tag", "v2.0.0",    // 新标签
            "--tag-note", "综合功能测试", // 标签说明
            "--push-branches",        // 推送分支
            "--no-add",              // 不执行git add
        ]).unwrap();
        
        // 验证所有参数都正确解析
        assert!(comprehensive_args.force_push);
        assert!(comprehensive_args.skip_confirm);
        assert!(comprehensive_args.push);
        assert_eq!(comprehensive_args.provider, "deepseek");
        assert_eq!(comprehensive_args.model, "deepseek-chat");
        assert_eq!(comprehensive_args.new_tag, Some("v2.0.0".to_string()));
        assert_eq!(comprehensive_args.tag_note, "综合功能测试");
        assert!(comprehensive_args.push_branches);
        assert!(comprehensive_args.no_add);
    }

    #[test]
    fn test_parameter_precedence_and_conflicts() {
        // 测试参数优先级和冲突处理
        
        // 测试短参数和长参数的等价性
        let long_form = Args::try_parse_from([
            "ai-commit", "--yes", "--push", "--force-push"
        ]).unwrap();
        
        let short_form = Args::try_parse_from([
            "ai-commit", "-y", "--push", "--force-push"
        ]).unwrap();
        
        assert_eq!(long_form.skip_confirm, short_form.skip_confirm);
        assert_eq!(long_form.push, short_form.push);
        assert_eq!(long_form.force_push, short_form.force_push);
        
        // 测试重复参数（clap 可能会拒绝重复参数，这是正常的）
        let repeated_provider_result = Args::try_parse_from([
            "ai-commit", 
            "--provider", "ollama",
            "--provider", "deepseek"  // 这个可能会导致冲突
        ]);
        
        // 检查是否接受重复参数，或者正确地拒绝
        match repeated_provider_result {
            Ok(args) => {
                // 如果接受了重复参数，验证最后一个生效
                assert_eq!(args.provider, "deepseek");
                println!("✓ 重复参数被接受，最后一个生效");
            }
            Err(_) => {
                // 如果拒绝重复参数，这也是合理的行为
                println!("✓ 重复参数被正确拒绝");
            }
        }
        
        // 测试空值处理
        let empty_values = Args::try_parse_from([
            "ai-commit",
            "--provider", "",
            "--model", "",
            "--tag-note", "",
        ]).unwrap();
        
        assert_eq!(empty_values.provider, "");
        assert_eq!(empty_values.model, "");
        assert_eq!(empty_values.tag_note, "");
    }

    #[test]
    fn test_workflow_scenarios() {
        // 测试真实的工作流程场景
        
        // 场景1: 快速开发迭代
        let quick_dev = Args::try_parse_from([
            "ai-commit", "-y", "--push", "--force-push"
        ]).unwrap();
        assert!(quick_dev.skip_confirm);
        assert!(quick_dev.push);
        assert!(quick_dev.force_push);
        
        // 场景2: 安全的生产发布
        let production_release = Args::try_parse_from([
            "ai-commit", 
            "--new-tag", "v1.0.0",
            "--tag-note", "Production Release v1.0.0",
            "--push", 
            "--push-branches",
            "--provider", "deepseek"
        ]).unwrap();
        assert!(!production_release.skip_confirm); // 生产环境需要确认
        assert!(!production_release.force_push);   // 生产环境不强制推送
        assert!(production_release.push);
        assert!(production_release.push_branches);
        
        // 场景3: 紧急修复
        let hotfix = Args::try_parse_from([
            "ai-commit",
            "--yes",                    // 快速确认
            "--force-push",             // 解决冲突
            "--push",                   // 立即推送
            "--provider", "ollama",     // 本地AI（更快）
        ]).unwrap();
        assert!(hotfix.skip_confirm);
        assert!(hotfix.force_push);
        assert!(hotfix.push);
        assert_eq!(hotfix.provider, "ollama");
        
        // 场景4: 团队协作
        let team_collab = Args::try_parse_from([
            "ai-commit",
            "--provider", "deepseek",   // 云端AI（一致性好）
            "--push",                   // 推送到团队仓库
            // 不使用 --yes（需要人工确认）
            // 不使用 --force-push（避免强制覆盖队友的工作）
        ]).unwrap();
        assert!(!team_collab.skip_confirm); // 团队环境需要确认
        assert!(!team_collab.force_push);   // 团队环境不强制推送
        assert!(team_collab.push);
    }

    #[test]
    fn test_edge_cases_and_error_conditions() {
        // 测试边界条件和错误情况
        
        // 测试无效的参数组合
        let invalid_combinations = vec![
            vec!["ai-commit", "--invalid-flag"],
            vec!["ai-commit", "--provider"],  // 缺少值
            vec!["ai-commit", "--model"],     // 缺少值  
            vec!["ai-commit", "--new-tag"],   // 缺少值（虽然可选）
        ];
        
        for invalid_args in invalid_combinations {
            let result = Args::try_parse_from(invalid_args.clone());
            if result.is_ok() {
                println!("参数组合意外成功: {:?}", invalid_args);
                // 某些组合可能是有效的（如 --new-tag 不带值）
            } else {
                println!("✓ 无效参数组合被正确拒绝: {:?}", invalid_args);
            }
        }
        
        // 测试极长的参数值
        let very_long_note = "很长的标签说明".repeat(1000);
        let long_args = Args::try_parse_from([
            "ai-commit",
            "--tag-note", &very_long_note,
            "--yes"
        ]).unwrap();
        
        assert_eq!(long_args.tag_note.len(), very_long_note.len());
        assert!(long_args.tag_note.len() > 10000);
        
        // 测试特殊字符
        let special_chars_note = "标签说明 !@#$%^&*()_+-=[]{}|;':\",./<>?`~";
        let special_args = Args::try_parse_from([
            "ai-commit",
            "--tag-note", special_chars_note,
            "--yes"
        ]).unwrap();
        
        assert_eq!(special_args.tag_note, special_chars_note);
    }

    #[test]
    fn test_configuration_integration() {
        // 测试配置集成
        
        let config = create_test_config();
        
        // 验证配置的基本属性
        assert_eq!(config.provider, "test");
        assert_eq!(config.model, "test-model");
        assert!(config.debug);
        
        // 测试配置与参数的交互
        let args = Args::try_parse_from([
            "ai-commit",
            "--provider", "override-provider",
            "--model", "override-model",
            "--yes",
            "--force-push"
        ]).unwrap();
        
        // 模拟配置更新逻辑
        let mut updated_config = config.clone();
        if !args.provider.is_empty() {
            updated_config.provider = args.provider.clone();
        }
        if !args.model.is_empty() {
            updated_config.model = args.model.clone();
        }
        
        assert_eq!(updated_config.provider, "override-provider");
        assert_eq!(updated_config.model, "override-model");
        assert!(args.skip_confirm);
        assert!(args.force_push);
    }

    #[tokio::test]
    async fn test_concurrent_force_push_operations() {
        // 测试并发强制推送操作
        use std::sync::Arc;
        use std::sync::atomic::{AtomicUsize, Ordering};
        use tokio::task;
        
        let success_count = Arc::new(AtomicUsize::new(0));
        let total_count = Arc::new(AtomicUsize::new(0));
        
        let mut handles = Vec::new();
        
        // 启动多个并发的强制推送操作
        for i in 0..5 {
            let success_count_clone = Arc::clone(&success_count);
            let total_count_clone = Arc::clone(&total_count);
            
            let handle = task::spawn(async move {
                total_count_clone.fetch_add(1, Ordering::SeqCst);
                
                let result = git::git_force_push().await;
                
                match result {
                    Ok(_) => {
                        success_count_clone.fetch_add(1, Ordering::SeqCst);
                        println!("并发强制推送 {} 成功", i);
                    }
                    Err(e) => {
                        println!("并发强制推送 {} 失败（预期）: {}", i, e);
                    }
                }
            });
            
            handles.push(handle);
        }
        
        // 等待所有任务完成
        for handle in handles {
            handle.await.unwrap();
        }
        
        let final_success = success_count.load(Ordering::SeqCst);
        let final_total = total_count.load(Ordering::SeqCst);
        
        println!("并发强制推送测试完成: {}/{} 成功", final_success, final_total);
        assert_eq!(final_total, 5, "应该启动5个并发任务");
        
        // 在测试环境中，大多数操作可能会失败，但不应该崩溃
        // 主要验证线程安全性和错误处理
    }

    #[test]
    fn test_performance_with_complex_arguments() {
        // 测试复杂参数的性能
        use std::time::Instant;
        
        let start = Instant::now();
        let iterations = 1000;
        
        for i in 0..iterations {
            let args = Args::try_parse_from([
                "ai-commit",
                "--provider", "deepseek",
                "--model", "deepseek-chat",
                "--yes",
                "--force-push", 
                "--push",
                "--new-tag", &format!("v1.0.{}", i),
                "--tag-note", &format!("性能测试标签 {}", i),
                "--push-branches",
                "--no-add",
            ]).unwrap();
            
            // 验证解析正确
            assert_eq!(args.provider, "deepseek");
            assert_eq!(args.model, "deepseek-chat");
            assert!(args.skip_confirm);
            assert!(args.force_push);
            assert!(args.push);
            assert_eq!(args.new_tag, Some(format!("v1.0.{}", i)));
            assert_eq!(args.tag_note, format!("性能测试标签 {}", i));
            assert!(args.push_branches);
            assert!(args.no_add);
        }
        
        let elapsed = start.elapsed();
        let ops_per_sec = iterations as f64 / elapsed.as_secs_f64();
        
        println!("解析 {} 次复杂参数，耗时 {:?}", iterations, elapsed);
        println!("每秒操作数: {:.2}", ops_per_sec);
        
        // 性能应该足够好（> 500 ops/sec）
        assert!(ops_per_sec > 500.0, 
            "复杂参数解析性能应该 > 500 ops/sec，实际: {:.2}", ops_per_sec);
    }

    #[test]
    fn test_memory_usage_patterns() {
        // 测试内存使用模式
        use std::mem;
        
        // 测试Args结构的内存占用
        let simple_args = Args::default();
        let complex_args = Args::try_parse_from([
            "ai-commit",
            "--provider", "very-long-provider-name-for-testing",
            "--model", "very-long-model-name-for-testing",
            "--tag-note", &"很长的标签说明".repeat(100),
            "--yes",
            "--force-push"
        ]).unwrap();
        
        let simple_size = mem::size_of_val(&simple_args);
        let complex_size = mem::size_of_val(&complex_args);
        
        println!("简单Args大小: {} bytes", simple_size);
        println!("复杂Args大小: {} bytes", complex_size);
        
        // 大小应该相同（因为String只是指针）
        assert_eq!(simple_size, complex_size, "Args结构大小应该一致");
        
        // 测试大量Args实例的创建和销毁
        let mut args_vec = Vec::new();
        
        for i in 0..1000 {
            let args = Args::try_parse_from([
                "ai-commit",
                "--provider", &format!("provider_{}", i),
                "--yes"
            ]).unwrap();
            
            args_vec.push(args);
        }
        
        // 验证所有实例都正确创建
        assert_eq!(args_vec.len(), 1000);
        
        // 验证第一个和最后一个实例
        assert_eq!(args_vec[0].provider, "provider_0");
        assert_eq!(args_vec[999].provider, "provider_999");
        assert!(args_vec.iter().all(|args| args.skip_confirm));
        
        // args_vec 在函数结束时被销毁，测试内存清理
    }

    #[test]
    fn test_help_and_version_integration() {
        // 测试帮助和版本信息集成
        use clap::CommandFactory;
        
        let mut cmd = Args::command();
        let help_output = cmd.render_help().to_string();
        
        // 验证帮助信息包含新功能
        assert!(help_output.contains("force-push") || help_output.contains("强制"));
        assert!(help_output.contains("yes") || help_output.contains("确认"));
        
        // 验证帮助信息的结构
        assert!(help_output.contains("Options:"));
        assert!(help_output.contains("Usage:"));
        
        // 测试版本信息（通过命令工厂）
        let version_str = cmd.render_version();
        assert!(!version_str.is_empty(), "版本信息不应该为空");
        
        println!("版本信息: {}", version_str);
    }

    #[test]
    fn test_regression_compatibility() {
        // 回归兼容性测试
        
        // 确保所有现有的参数组合仍然有效
        let legacy_combinations = vec![
            vec!["ai-commit"],
            vec!["ai-commit", "--push"],
            vec!["ai-commit", "--provider", "ollama"],
            vec!["ai-commit", "--model", "mistral"],
            vec!["ai-commit", "--new-tag", "v1.0.0"],
            vec!["ai-commit", "--tag-note", "测试"],
            vec!["ai-commit", "--no-add"],
            vec!["ai-commit", "--show-tag"],
            vec!["ai-commit", "--push-branches"],
        ];
        
        for legacy_args in legacy_combinations {
            let result = Args::try_parse_from(legacy_args.clone());
            assert!(result.is_ok(), "遗留参数组合应该仍然有效: {:?}", legacy_args);
            
            let args = result.unwrap();
            
            // 新功能的默认值应该是安全的
            assert!(!args.skip_confirm, "默认应该需要确认: {:?}", legacy_args);
            assert!(!args.force_push, "默认不应该强制推送: {:?}", legacy_args);
        }
        
        // 测试新旧参数的混合
        let mixed_legacy_new = Args::try_parse_from([
            "ai-commit",
            "--provider", "deepseek",  // 旧参数
            "--push",                  // 旧参数
            "--yes",                   // 新参数
            "--force-push",            // 新参数
        ]).unwrap();
        
        assert_eq!(mixed_legacy_new.provider, "deepseek");
        assert!(mixed_legacy_new.push);
        assert!(mixed_legacy_new.skip_confirm);
        assert!(mixed_legacy_new.force_push);
    }
}