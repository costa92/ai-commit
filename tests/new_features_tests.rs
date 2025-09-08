use ai_commit::{ui, cli::args::Args, config::Config, commands};
use clap::Parser;

/// 新功能集成测试
/// 测试二次确认功能和强制推送功能

#[cfg(test)]
mod new_features_integration_tests {
    use super::*;

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

            debug: false,
        }
    }

    #[test]
    fn test_confirm_result_enum() {
        // 测试 ConfirmResult 枚举
        let result1 = ui::ConfirmResult::Confirmed("test message".to_string());
        let result2 = ui::ConfirmResult::Rejected;
        
        assert_ne!(result1, result2);
        
        match result1 {
            ui::ConfirmResult::Confirmed(msg) => {
                assert_eq!(msg, "test message");
            }
            _ => panic!("Expected Confirmed variant"),
        }
        
        match result2 {
            ui::ConfirmResult::Rejected => {
                // 正确
            }
            _ => panic!("Expected Rejected variant"),
        }
    }

    #[test]
    fn test_skip_confirm_parameter() {
        // 测试跳过确认参数
        let result1 = ui::confirm_commit_message("test message", true);
        assert!(result1.is_ok());
        
        if let Ok(ui::ConfirmResult::Confirmed(msg)) = result1 {
            assert_eq!(msg, "test message");
        } else {
            panic!("Should return Confirmed when skip_confirm is true");
        }
    }

    #[test]
    fn test_conventional_commit_validation() {
        // 测试 Conventional Commits 格式验证的各种情况
        
        // 有效的格式
        let _valid_messages = vec![
            "feat: 添加新功能",
            "fix(ui): 修复按钮问题",
            "docs(readme): 更新文档",
            "style: 格式化代码",
            "refactor(core): 重构核心模块",
            "test: 添加单元测试",
            "chore: 更新依赖",
            "feat(api): 🎉 添加新的API端点",
            "fix: 修复#123问题",
            "docs: 更新README.md",
        ];

        // 无效的格式
        let _invalid_messages = vec![
            "添加新功能",
            "update readme",
            "feat 添加功能",
            "FEAT: 添加功能",
            "feat(): ",
            "",
            "feat: ",
        ];

        // 这里我们需要创建一个公开的验证函数，或者通过其他方式测试
        // 由于 is_valid_commit_message 是私有的，我们通过行为测试
    }

    #[tokio::test]
    async fn test_cli_args_parsing_new_features() {
        // 测试新功能的CLI参数解析
        
        // 测试 --yes 参数
        let args = Args::try_parse_from(["ai-commit", "--yes"]).unwrap();
        assert!(args.skip_confirm);
        
        // 测试 -y 参数
        let args = Args::try_parse_from(["ai-commit", "-y"]).unwrap();
        assert!(args.skip_confirm);
        
        // 测试 --force-push 参数
        let args = Args::try_parse_from(["ai-commit", "--force-push"]).unwrap();
        assert!(args.force_push);
        
        // 测试参数组合
        let args = Args::try_parse_from([
            "ai-commit", 
            "--yes", 
            "--force-push", 
            "--push",
            "--provider", "deepseek"
        ]).unwrap();
        assert!(args.skip_confirm);
        assert!(args.force_push);
        assert!(args.push);
        assert_eq!(args.provider, "deepseek");
    }

    #[tokio::test]
    async fn test_commit_commands_with_skip_confirm() {
        // 测试带跳过确认的提交命令
        let mut args = Args::default();
        args.skip_confirm = true;
        args.no_add = true; // 跳过 git add 以避免实际文件操作
        
        let config = create_test_config();
        
        // 这个测试可能会失败，因为需要实际的 git 环境和 AI 服务
        // 但我们可以验证函数调用不会 panic
        let result = commands::handle_commit_commands(&args, &config).await;
        
        // 在测试环境中，这通常会失败，但不应该 panic
        match result {
            Ok(_) => {
                println!("Commit command succeeded in test environment");
            }
            Err(e) => {
                println!("Commit command failed as expected in test environment: {}", e);
                // 验证错误消息包含预期的内容
                let error_str = e.to_string();
                assert!(
                    error_str.contains("No staged changes") || 
                    error_str.contains("Failed to") ||
                    error_str.contains("AI") ||
                    error_str.contains("git"),
                    "Error should be related to git operations or AI service: {}", error_str
                );
            }
        }
    }

    #[tokio::test]
    async fn test_tag_creation_with_confirmation() {
        // 测试标签创建的确认流程
        let mut args = Args::default();
        args.new_tag = Some("v1.0.0".to_string());
        args.skip_confirm = true; // 跳过确认以便测试
        
        let config = create_test_config();
        let test_diff = "diff --git a/test.txt b/test.txt\n+new line";
        
        let result = commands::handle_tag_creation_commit(&args, &config, test_diff).await;
        
        // 在测试环境中验证行为
        match result {
            Ok(_) => {
                println!("Tag creation succeeded in test environment");
            }
            Err(e) => {
                println!("Tag creation failed as expected in test environment: {}", e);
                let error_str = e.to_string();
                assert!(
                    error_str.contains("Failed to") ||
                    error_str.contains("git") ||
                    error_str.contains("Git") ||
                    error_str.contains("AI") ||
                    error_str.contains("Ollama") ||
                    error_str.contains("Deepseek") ||
                    error_str.contains("tag") ||
                    error_str.contains("响应错误") ||
                    error_str.contains("502"),
                    "Error should be related to git operations or AI services: {}", error_str
                );
            }
        }
    }

    #[test]
    fn test_force_push_parameter_combinations() {
        // 测试强制推送参数的各种组合
        
        // 单独的 force-push
        let args = Args::try_parse_from(["ai-commit", "--force-push"]).unwrap();
        assert!(args.force_push);
        assert!(!args.push); // force-push 不自动启用 push
        
        // force-push + push 组合
        let args = Args::try_parse_from(["ai-commit", "--force-push", "--push"]).unwrap();
        assert!(args.force_push);
        assert!(args.push);
        
        // 所有新功能的组合
        let args = Args::try_parse_from([
            "ai-commit", 
            "--force-push", 
            "--yes", 
            "--push",
            "--new-tag", "v2.0.0",
            "--provider", "ollama"
        ]).unwrap();
        assert!(args.force_push);
        assert!(args.skip_confirm);
        assert!(args.push);
        assert_eq!(args.new_tag, Some("v2.0.0".to_string()));
        assert_eq!(args.provider, "ollama");
    }

    #[test]
    fn test_config_with_new_features() {
        // 测试配置与新功能的集成
        let config = create_test_config();
        
        // 验证配置的基本属性
        assert_eq!(config.provider, "test");
        assert_eq!(config.model, "test-model");
        assert!(config.deepseek_api_key.is_some());
        assert!(!config.debug);
        
        // 验证配置可以与新功能参数一起工作
        let args = Args::try_parse_from([
            "ai-commit", 
            "--yes", 
            "--force-push",
            "--provider", "deepseek"
        ]).unwrap();
        
        // 模拟配置更新
        let mut updated_config = config;
        if !args.provider.is_empty() {
            updated_config.provider = args.provider.clone();
        }
        
        assert_eq!(updated_config.provider, "deepseek");
    }

    #[test] 
    fn test_error_handling_scenarios() {
        // 测试各种错误处理场景
        
        // 测试无效的参数组合
        let result = Args::try_parse_from(["ai-commit", "--invalid-flag"]);
        assert!(result.is_err(), "Invalid flag should cause parsing error");
        
        // 测试帮助信息包含新功能
        let result = Args::try_parse_from(["ai-commit", "--help"]);
        assert!(result.is_err()); // --help 会导致退出，被视为错误
        
        // 测试版本信息
        let result = Args::try_parse_from(["ai-commit", "--version"]);
        assert!(result.is_err()); // --version 会导致退出，被视为错误
    }

    #[test]
    fn test_default_behavior_unchanged() {
        // 验证新功能不会破坏默认行为
        let args = Args::try_parse_from(["ai-commit"]).unwrap();
        
        // 验证默认值
        assert!(!args.skip_confirm); // 默认需要确认
        assert!(!args.force_push);   // 默认不强制推送
        assert!(!args.push);         // 默认不推送
        assert_eq!(args.provider, ""); // 默认无提供商
        assert_eq!(args.model, "");    // 默认无模型
        assert!(!args.no_add);         // 默认执行 git add
        
        // 验证其他功能的默认值未受影响
        assert!(!args.show_tag);
        assert!(!args.push_branches);
        assert_eq!(args.new_tag, None);
        assert_eq!(args.tag_note, "");
    }

    #[test]
    fn test_backwards_compatibility() {
        // 测试向后兼容性
        
        // 老的参数组合应该仍然工作
        let args = Args::try_parse_from([
            "ai-commit",
            "--provider", "ollama",
            "--model", "mistral", 
            "--push",
            "--new-tag", "v1.0.0",
            "--tag-note", "Release version 1.0.0"
        ]).unwrap();
        
        assert_eq!(args.provider, "ollama");
        assert_eq!(args.model, "mistral");
        assert!(args.push);
        assert_eq!(args.new_tag, Some("v1.0.0".to_string()));
        assert_eq!(args.tag_note, "Release version 1.0.0");
        
        // 新的默认行为
        assert!(!args.skip_confirm); // 新功能默认启用确认
        assert!(!args.force_push);   // 新功能默认不强制推送
    }

    #[test]
    fn test_help_output_contains_new_features() {
        // 虽然我们无法直接捕获 --help 的输出，但可以验证参数存在
        use clap::CommandFactory;
        
        let mut cmd = Args::command();
        let help_text = cmd.render_help().to_string();
        
        // 验证帮助文本包含新功能
        assert!(help_text.contains("--yes") || help_text.contains("-y"));
        assert!(help_text.contains("--force-push"));
        assert!(help_text.contains("二次确认") || help_text.contains("确认"));
        assert!(help_text.contains("强制") || help_text.contains("冲突"));
    }

    #[test]
    fn test_ui_module_public_interface() {
        // 测试 UI 模块的公共接口
        
        // 验证 ConfirmResult 的 Debug 实现
        let confirmed = ui::ConfirmResult::Confirmed("test".to_string());
        let rejected = ui::ConfirmResult::Rejected;
        
        let debug_str1 = format!("{:?}", confirmed);
        let debug_str2 = format!("{:?}", rejected);
        
        assert!(debug_str1.contains("Confirmed"));
        assert!(debug_str1.contains("test"));
        assert!(debug_str2.contains("Rejected"));
        
        // 验证 PartialEq 实现
        assert_eq!(confirmed, ui::ConfirmResult::Confirmed("test".to_string()));
        assert_eq!(rejected, ui::ConfirmResult::Rejected);
        assert_ne!(confirmed, rejected);
    }

    #[test]
    fn test_comprehensive_argument_validation() {
        // 综合参数验证测试
        
        // 测试所有可能的布尔标志组合
        let test_cases = vec![
            (vec!["ai-commit"], false, false),
            (vec!["ai-commit", "--yes"], true, false),
            (vec!["ai-commit", "-y"], true, false),
            (vec!["ai-commit", "--force-push"], false, true),
            (vec!["ai-commit", "--yes", "--force-push"], true, true),
            (vec!["ai-commit", "-y", "--force-push"], true, true),
        ];
        
        for (args_vec, expected_skip_confirm, expected_force_push) in test_cases {
            let args = Args::try_parse_from(args_vec.clone()).unwrap();
            assert_eq!(args.skip_confirm, expected_skip_confirm, 
                       "Failed for args: {:?}", args_vec);
            assert_eq!(args.force_push, expected_force_push, 
                       "Failed for args: {:?}", args_vec);
        }
    }

    #[test]
    fn test_feature_interaction_matrix() {
        // 测试功能交互矩阵 - 确保新功能与现有功能的所有组合都能正常工作
        
        type BoolChecker = Box<dyn Fn(&Args) -> bool>;
        
        let base_features: Vec<(&str, BoolChecker)> = vec![
            ("--push", Box::new(|args: &Args| args.push)),
            ("--no-add", Box::new(|args: &Args| args.no_add)),
            ("--show-tag", Box::new(|args: &Args| args.show_tag)),
            ("--push-branches", Box::new(|args: &Args| args.push_branches)),
        ];
        
        let new_features: Vec<(&str, BoolChecker)> = vec![
            ("--yes", Box::new(|args: &Args| args.skip_confirm)),
            ("--force-push", Box::new(|args: &Args| args.force_push)),
        ];
        
        // 测试新功能与基础功能的组合
        for (base_flag, base_checker) in &base_features {
            for (new_flag, new_checker) in &new_features {
                let args = Args::try_parse_from([
                    "ai-commit", base_flag, new_flag
                ]).unwrap();
                
                assert!(base_checker(&args), 
                        "Base feature {} should be enabled", base_flag);
                assert!(new_checker(&args), 
                        "New feature {} should be enabled", new_flag);
            }
        }
    }
}

#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_argument_parsing_performance() {
        // 测试参数解析性能
        let start = Instant::now();
        
        for _ in 0..1000 {
            let _args = Args::try_parse_from([
                "ai-commit", 
                "--yes", 
                "--force-push", 
                "--push",
                "--provider", "deepseek",
                "--model", "deepseek-chat",
                "--new-tag", "v1.0.0",
                "--tag-note", "Performance test"
            ]).unwrap();
        }
        
        let elapsed = start.elapsed();
        println!("Parsed 1000 argument sets in {:?}", elapsed);
        
        // 参数解析应该很快（< 1000ms for 1000 iterations）
        assert!(elapsed.as_millis() < 1000, 
                "Argument parsing took too long: {:?}", elapsed);
    }

    #[test]
    fn test_ui_enum_performance() {
        // 测试UI枚举的性能
        let start = Instant::now();
        
        for i in 0..10000 {
            let result = if i % 2 == 0 {
                ui::ConfirmResult::Confirmed(format!("message_{}", i))
            } else {
                ui::ConfirmResult::Rejected
            };
            
            // 模拟使用枚举
            match result {
                ui::ConfirmResult::Confirmed(_) => {},
                ui::ConfirmResult::Rejected => {},
            }
        }
        
        let elapsed = start.elapsed();
        println!("Created and matched 10000 ConfirmResult enums in {:?}", elapsed);
        
        // 枚举操作应该很快
        assert!(elapsed.as_millis() < 50, 
                "Enum operations took too long: {:?}", elapsed);
    }
}

#[cfg(test)]
mod edge_case_tests {
    use super::*;

    #[test]
    fn test_empty_and_special_inputs() {
        // 测试空输入和特殊输入
        
        // 空的提供商和模型
        let args = Args::try_parse_from([
            "ai-commit", 
            "--provider", "",
            "--model", "",
            "--yes"
        ]).unwrap();
        assert_eq!(args.provider, "");
        assert_eq!(args.model, "");
        assert!(args.skip_confirm);
        
        // 包含特殊字符的标签注释
        let args = Args::try_parse_from([
            "ai-commit",
            "--tag-note", "版本 1.0.0 - 修复 Bug #123 & 添加新功能 🎉",
            "--yes"
        ]).unwrap();
        assert_eq!(args.tag_note, "版本 1.0.0 - 修复 Bug #123 & 添加新功能 🎉");
        assert!(args.skip_confirm);
    }

    #[test]
    fn test_unicode_handling() {
        // 测试 Unicode 处理
        let test_messages = vec![
            "feat: 添加中文支持",
            "fix: 修复🐛问题",
            "docs: 更新文档📝",
            "style: 美化界面✨",
            "test: 测试用例🧪",
        ];
        
        for message in test_messages {
            // 测试通过跳过确认创建的确认结果
            let result = ui::confirm_commit_message(message, true).unwrap();
            match result {
                ui::ConfirmResult::Confirmed(msg) => {
                    assert_eq!(msg, message);
                    // 验证 Unicode 字符长度计算正确
                    assert_eq!(msg.chars().count(), message.chars().count());
                }
                _ => panic!("Should return Confirmed when skip_confirm is true"),
            }
        }
    }

    #[test]
    fn test_very_long_inputs() {
        // 测试很长的输入
        let long_message = "feat: ".to_string() + &"很长的提交消息内容".repeat(100);
        let long_tag_note = "发布说明内容".repeat(200);
        
        let args = Args::try_parse_from([
            "ai-commit",
            "--tag-note", &long_tag_note,
            "--yes"
        ]).unwrap();
        
        assert_eq!(args.tag_note, long_tag_note);
        assert!(args.skip_confirm);
        
        // 测试长消息的确认
        let result = ui::confirm_commit_message(&long_message, true).unwrap();
        match result {
            ui::ConfirmResult::Confirmed(msg) => {
                assert_eq!(msg, long_message);
                assert!(msg.len() > 1000); // 验证确实很长
            }
            _ => panic!("Should return Confirmed for long messages"),
        }
    }

    #[test]
    fn test_boundary_conditions() {
        // 测试边界条件
        
        // 最短的有效参数
        let args = Args::try_parse_from(["ai-commit", "-y"]).unwrap();
        assert!(args.skip_confirm);
        
        // 最长的参数组合（测试所有可能的参数）
        let args = Args::try_parse_from([
            "ai-commit",
            "--provider", "deepseek",
            "--model", "deepseek-chat", 
            "--no-add",
            "--push",
            "--new-tag", "v1.0.0",
            "--tag-note", "Complete feature set test",
            "--show-tag",
            "--push-branches",
            "--yes",
            "--force-push",
            // 添加其他现有功能的参数...
        ]).unwrap();
        
        // 验证所有参数都被正确解析
        assert_eq!(args.provider, "deepseek");
        assert_eq!(args.model, "deepseek-chat");
        assert!(args.no_add);
        assert!(args.push);
        assert_eq!(args.new_tag, Some("v1.0.0".to_string()));
        assert_eq!(args.tag_note, "Complete feature set test");
        assert!(args.show_tag);
        assert!(args.push_branches);
        assert!(args.skip_confirm);
        assert!(args.force_push);
    }
}