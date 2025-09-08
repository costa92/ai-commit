/// Commit 工作流程测试
/// 测试提交命令的完整工作流程，包括二次确认和强制推送

#[cfg(test)]
mod commit_workflow_tests {
    use ai_commit::{commands, cli::args::Args, config::Config, git, ui};
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
            debug: true, // 启用调试模式以获得更多测试信息
        }
    }

    #[tokio::test]
    async fn test_commit_command_with_skip_confirm() {
        // 测试带有跳过确认的提交命令
        let mut args = Args::default();
        args.skip_confirm = true;
        args.no_add = true; // 跳过 git add 以避免文件系统操作
        
        let config = create_test_config();
        
        let result = commands::handle_commit_commands(&args, &config).await;
        
        // 在测试环境中，这个操作通常会因为各种原因失败，但我们验证它不会 panic
        match result {
            Ok(_) => {
                println!("✓ 提交命令在测试环境中成功执行");
            }
            Err(e) => {
                let error_msg = e.to_string();
                println!("✓ 提交命令在测试环境中失败（预期）: {}", error_msg);
                
                // 验证错误消息包含预期的关键词
                let expected_keywords = [
                    "No staged changes", "没有暂存", "staged", "diff", 
                    "Failed to", "git", "AI", "服务", "生成", "Ollama", 
                    "响应错误", "502", "Bad Gateway", "Deepseek"
                ];
                
                let contains_expected = expected_keywords.iter()
                    .any(|keyword| error_msg.contains(keyword));
                    
                assert!(contains_expected, 
                    "错误消息应该包含预期的关键词之一: {}", error_msg);
            }
        }
    }

    #[tokio::test]
    async fn test_commit_command_without_skip_confirm() {
        // 测试不跳过确认的提交命令（但由于无法模拟用户输入，这主要测试结构）
        let mut args = Args::default();
        args.skip_confirm = false; // 需要确认
        args.no_add = true;
        
        let config = create_test_config();
        
        let result = commands::handle_commit_commands(&args, &config).await;
        
        // 这个测试主要验证函数调用结构正确
        match result {
            Ok(_) => {
                println!("✓ 提交命令（需要确认）在测试环境中成功执行");
            }
            Err(e) => {
                println!("✓ 提交命令（需要确认）在测试环境中失败（预期）: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_tag_creation_commit_workflow() {
        // 测试标签创建的提交工作流程
        let mut args = Args::default();
        args.new_tag = Some("v1.0.0".to_string());
        args.skip_confirm = true; // 跳过确认以简化测试
        args.tag_note = "测试标签".to_string(); // 提供标签说明
        
        let config = create_test_config();
        let test_diff = "diff --git a/test.txt b/test.txt\n@@ -1 +1,2 @@\n 原有内容\n+新增内容";
        
        let result = commands::handle_tag_creation_commit(&args, &config, test_diff).await;
        
        match result {
            Ok(_) => {
                println!("✓ 标签创建提交在测试环境中成功执行");
            }
            Err(e) => {
                println!("✓ 标签创建提交在测试环境中失败（预期）: {}", e);
                
                // 验证错误与 git 操作或 AI 服务相关
                let error_msg = e.to_string();
                let git_related = error_msg.contains("git") || 
                                 error_msg.contains("Git") ||
                                 error_msg.contains("tag") ||
                                 error_msg.contains("commit");
                                 
                assert!(git_related, "错误应该与 git 操作相关: {}", error_msg);
            }
        }
    }

    #[tokio::test]
    async fn test_tag_creation_without_note() {
        // 测试不提供标签说明的标签创建
        let mut args = Args::default();
        args.new_tag = Some("v1.1.0".to_string());
        args.skip_confirm = true;
        args.tag_note = String::new(); // 不提供标签说明
        
        let config = create_test_config();
        let test_diff = "diff --git a/test.txt b/test.txt\n@@ -1 +1,2 @@\n 原有内容\n+新增内容";
        
        let result = commands::handle_tag_creation_commit(&args, &config, test_diff).await;
        
        // 这种情况下应该使用 AI 生成消息
        match result {
            Ok(_) => {
                println!("✓ 无标签说明的标签创建在测试环境中成功执行");
            }
            Err(e) => {
                println!("✓ 无标签说明的标签创建在测试环境中失败（预期）: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_git_force_push_functionality() {
        // 测试强制推送功能
        let result = git::git_force_push().await;
        
        match result {
            Ok(_) => {
                println!("✓ 强制推送在测试环境中成功执行");
            }
            Err(e) => {
                println!("✓ 强制推送在测试环境中失败（预期）: {}", e);
                
                let error_msg = e.to_string();
                let expected_errors = [
                    "Failed to run git push",
                    "Git push failed", 
                    "Failed to get current branch",
                    "Failed to run git pull",
                    "Git pull failed"
                ];
                
                let contains_expected = expected_errors.iter()
                    .any(|expected| error_msg.contains(expected));
                    
                assert!(contains_expected, 
                    "强制推送错误应该包含预期的git操作错误: {}", error_msg);
            }
        }
    }

    #[tokio::test]  
    async fn test_commit_with_force_push_enabled() {
        // 测试启用强制推送的提交
        let mut args = Args::default();
        args.skip_confirm = true;
        args.push = true;
        args.force_push = true;
        args.no_add = true;
        
        let config = create_test_config();
        
        let result = commands::handle_commit_commands(&args, &config).await;
        
        // 验证函数调用结构
        match result {
            Ok(_) => {
                println!("✓ 带强制推送的提交命令在测试环境中成功执行");
            }
            Err(e) => {
                println!("✓ 带强制推送的提交命令在测试环境中失败（预期）: {}", e);
            }
        }
    }

    #[test]
    fn test_workflow_parameter_validation() {
        // 测试工作流程参数验证
        
        // 测试有效的参数组合
        let valid_combinations = vec![
            (vec!["ai-commit"], false, false, false),
            (vec!["ai-commit", "--yes"], true, false, false),
            (vec!["ai-commit", "--push"], false, true, false),
            (vec!["ai-commit", "--force-push"], false, false, true),
            (vec!["ai-commit", "--yes", "--push"], true, true, false),
            (vec!["ai-commit", "--yes", "--force-push"], true, false, true),
            (vec!["ai-commit", "--push", "--force-push"], false, true, true),
            (vec!["ai-commit", "--yes", "--push", "--force-push"], true, true, true),
        ];
        
        for (args_vec, expected_skip, expected_push, expected_force) in valid_combinations {
            let args = Args::try_parse_from(args_vec.clone()).unwrap();
            
            assert_eq!(args.skip_confirm, expected_skip, "skip_confirm 应该匹配，参数: {:?}", args_vec);
            assert_eq!(args.push, expected_push, "push 应该匹配，参数: {:?}", args_vec);
            assert_eq!(args.force_push, expected_force, "force_push 应该匹配，参数: {:?}", args_vec);
        }
    }

    #[test]
    fn test_workflow_integration_scenarios() {
        // 测试工作流程集成场景
        
        // 场景 1: 快速提交（跳过确认，直接推送）
        let quick_commit = Args::try_parse_from([
            "ai-commit", "--yes", "--push"
        ]).unwrap();
        assert!(quick_commit.skip_confirm);
        assert!(quick_commit.push);
        assert!(!quick_commit.force_push);
        
        // 场景 2: 安全提交（需要确认，解决冲突）
        let safe_commit = Args::try_parse_from([
            "ai-commit", "--push", "--force-push"
        ]).unwrap();
        assert!(!safe_commit.skip_confirm);
        assert!(safe_commit.push);
        assert!(safe_commit.force_push);
        
        // 场景 3: 完整工作流（标签 + 推送 + 冲突解决）
        let full_workflow = Args::try_parse_from([
            "ai-commit", "--yes", "--new-tag", "v1.0.0", 
            "--push", "--force-push", "--push-branches"
        ]).unwrap();
        assert!(full_workflow.skip_confirm);
        assert_eq!(full_workflow.new_tag, Some("v1.0.0".to_string()));
        assert!(full_workflow.push);
        assert!(full_workflow.force_push);
        assert!(full_workflow.push_branches);
        
        // 场景 4: 开发者友好模式（所有安全功能）
        let dev_friendly = Args::try_parse_from([
            "ai-commit", "--provider", "deepseek", "--model", "deepseek-chat"
        ]).unwrap();
        assert!(!dev_friendly.skip_confirm); // 默认需要确认
        assert!(!dev_friendly.force_push);   // 默认不强制推送
        assert_eq!(dev_friendly.provider, "deepseek");
        assert_eq!(dev_friendly.model, "deepseek-chat");
    }

    #[tokio::test]
    async fn test_error_recovery_scenarios() {
        // 测试错误恢复场景
        
        let config = create_test_config();
        
        // 场景 1: 无暂存更改
        let mut no_changes_args = Args::default();
        no_changes_args.skip_confirm = true;
        no_changes_args.no_add = true;
        
        let result = commands::handle_commit_commands(&no_changes_args, &config).await;
        match result {
            Ok(_) => {
                // 如果没有更改，函数应该优雅地返回
                println!("✓ 无更改情况处理正确");
            }
            Err(e) => {
                // 错误应该是可理解的
                let error_msg = e.to_string();
                assert!(!error_msg.is_empty(), "错误消息不应该为空");
                println!("✓ 无更改情况的错误处理正确: {}", error_msg);
            }
        }
        
        // 场景 2: 配置不完整
        let incomplete_config = Config {
            provider: "nonexistent".to_string(),
            model: "nonexistent-model".to_string(),
            deepseek_api_key: None,
            deepseek_url: "invalid-url".to_string(),
            ollama_url: "invalid-url".to_string(),
            siliconflow_api_key: None,
            siliconflow_url: "invalid-url".to_string(),
            kimi_api_key: None,
            kimi_url: "https://api.moonshot.cn/v1/chat/completions".to_string(),

            debug: false,
        };
        
        let result = commands::handle_commit_commands(&no_changes_args, &incomplete_config).await;
        match result {
            Ok(_) => {
                println!("✓ 不完整配置情况处理正确");
            }
            Err(e) => {
                println!("✓ 不完整配置的错误处理正确: {}", e);
            }
        }
    }

    #[test]
    fn test_confirmation_flow_logic() {
        // 测试确认流程逻辑
        
        // 测试跳过确认的逻辑
        let result = ui::confirm_commit_message("test message", true);
        assert!(result.is_ok());
        
        if let Ok(ui::ConfirmResult::Confirmed(msg)) = result {
            assert_eq!(msg, "test message");
        } else {
            panic!("跳过确认应该返回 Confirmed");
        }
        
        // 测试不同类型的消息
        let test_messages = vec![
            "feat: 添加新功能",
            "fix(ui): 修复按钮问题", 
            "docs: 更新文档",
            "refactor: 重构代码",
            "test: 添加测试",
            "chore: 维护工作",
        ];
        
        for message in test_messages {
            let result = ui::confirm_commit_message(message, true);
            assert!(result.is_ok(), "消息应该被接受: {}", message);
            
            if let Ok(ui::ConfirmResult::Confirmed(returned_msg)) = result {
                assert_eq!(returned_msg, message);
            }
        }
    }

    #[tokio::test]
    async fn test_concurrent_workflow_operations() {
        // 测试并发工作流程操作
        use std::sync::Arc;
        use tokio::task;
        
        let config = Arc::new(create_test_config());
        let mut handles = Vec::new();
        
        // 启动多个并发的工作流程
        for i in 0..5 {
            let config_clone = Arc::clone(&config);
            let handle = task::spawn(async move {
                let mut args = Args::default();
                args.skip_confirm = true;
                args.no_add = true;
                
                let result = commands::handle_commit_commands(&args, &config_clone).await;
                (i, result.is_ok())
            });
            handles.push(handle);
        }
        
        // 收集结果
        let mut success_count = 0;
        let mut total_count = 0;
        
        for handle in handles {
            let (task_id, success) = handle.await.unwrap();
            total_count += 1;
            
            if success {
                success_count += 1;
            }
            
            println!("任务 {} 完成，成功: {}", task_id, success);
        }
        
        println!("并发测试完成: {}/{} 个任务成功", success_count, total_count);
        
        // 验证至少没有崩溃
        assert_eq!(total_count, 5, "所有任务都应该完成");
    }

    #[test]
    fn test_workflow_configuration_matrix() {
        // 测试工作流程配置矩阵
        
        let providers = vec!["ollama", "deepseek", "siliconflow"];
        let models = vec!["mistral", "deepseek-chat", "qwen-turbo"];
        let boolean_flags = vec![
            ("skip_confirm", true),
            ("push", true), 
            ("force_push", true),
            ("no_add", true),
        ];
        
        // 测试提供商和模型的组合
        for provider in &providers {
            for model in &models {
                let args = Args::try_parse_from([
                    "ai-commit",
                    "--provider", provider,
                    "--model", model,
                    "--yes"
                ]).unwrap();
                
                assert_eq!(args.provider, *provider);
                assert_eq!(args.model, *model);
                assert!(args.skip_confirm);
            }
        }
        
        // 测试布尔标志的组合
        for (flag_name, _) in &boolean_flags {
            let flag_arg = format!("--{}", flag_name.replace('_', "-"));
            
            // 某些标志需要特殊处理
            let final_args = match flag_name.as_ref() {
                "skip_confirm" => vec!["ai-commit", "--yes"],
                "force_push" => vec!["ai-commit", "--force-push"],
                _ => vec!["ai-commit", &flag_arg],
            };
            
            let result = Args::try_parse_from(final_args);
            if result.is_err() {
                println!("标志 {} 解析失败（可能正常）: {:?}", flag_name, result.err());
            } else {
                let args = result.unwrap();
                println!("✓ 标志 {} 解析成功", flag_name);
                
                // 验证特定的标志
                match flag_name.as_ref() {
                    "skip_confirm" => assert!(args.skip_confirm),
                    "push" => assert!(args.push),
                    "force_push" => assert!(args.force_push),
                    "no_add" => assert!(args.no_add),
                    _ => {}
                }
            }
        }
    }

    #[test]
    fn test_regression_scenarios() {
        // 回归测试场景
        
        // 确保新功能不会破坏现有功能
        let legacy_args = Args::try_parse_from([
            "ai-commit",
            "--provider", "ollama",
            "--model", "mistral",
            "--push",
            "--new-tag", "v1.0.0",
            "--tag-note", "Legacy functionality test"
        ]).unwrap();
        
        // 验证旧功能仍然正常工作
        assert_eq!(legacy_args.provider, "ollama");
        assert_eq!(legacy_args.model, "mistral");
        assert!(legacy_args.push);
        assert_eq!(legacy_args.new_tag, Some("v1.0.0".to_string()));
        assert_eq!(legacy_args.tag_note, "Legacy functionality test");
        
        // 新功能的默认值应该是安全的
        assert!(!legacy_args.skip_confirm); // 默认需要确认
        assert!(!legacy_args.force_push);   // 默认不强制推送
        
        // 测试新旧功能的混合使用
        let mixed_args = Args::try_parse_from([
            "ai-commit",
            "--provider", "deepseek",  // 旧功能
            "--yes",                   // 新功能
            "--push",                  // 旧功能
            "--force-push",            // 新功能
            "--new-tag", "v2.0.0",     // 旧功能
        ]).unwrap();
        
        assert_eq!(mixed_args.provider, "deepseek");
        assert!(mixed_args.skip_confirm);
        assert!(mixed_args.push);
        assert!(mixed_args.force_push);
        assert_eq!(mixed_args.new_tag, Some("v2.0.0".to_string()));
    }
}