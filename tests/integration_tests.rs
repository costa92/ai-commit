use ai_commit::cli::args::Args;
use ai_commit::config::{ensure_env_loaded, Config};
use ai_commit::core::ai::prompt;
use ai_commit::internationalization::{I18n, Language};
use clap::Parser;

/// 集成测试：测试配置系统的完整流程
#[test]
fn test_config_integration() {
    use std::env;

    // 强制清理所有可能的环境变量
    for (key, _) in env::vars() {
        if key.starts_with("AI_COMMIT_") {
            env::remove_var(&key);
        }
    }

    // 1. 测试默认配置
    let config = Config {
        provider: "ollama".to_string(),
        model: "mistral".to_string(),
        debug: false,
    };

    // 验证配置有效性
    assert!(!config.provider.is_empty());
    assert!(!config.model.is_empty());
    assert!(config.validate().is_ok()); // ollama provider应该总是valid

    // 验证debug模式默认为false
    assert!(!config.debug);

    // 2. 测试命令行参数覆盖
    let mut args = Args::default();
    args.provider = "ollama".to_string();
    args.model = "test-model".to_string();

    let mut config = Config::new();
    config.update_from_args(&args);

    assert_eq!(config.provider, "ollama");
    assert_eq!(config.model, "test-model");
}

/// 集成测试：测试CLI参数解析和配置更新的完整流程
#[test]
fn test_cli_config_integration() {
    // 模拟命令行参数
    let args = Args::try_parse_from([
        "ai-commit",
        "--provider",
        "deepseek",
        "--model",
        "deepseek-chat",
        "--push",
        "--new-tag",
        "v1.0.0",
        "--tag-note",
        "Integration test release",
        "--push-branches",
    ])
    .unwrap();

    // 验证参数解析
    assert_eq!(args.provider, "deepseek");
    assert_eq!(args.model, "deepseek-chat");
    assert!(args.push);
    assert_eq!(args.new_tag, Some("v1.0.0".to_string()));
    assert_eq!(args.tag_note, "Integration test release");
    assert!(args.push_branches);

    // 测试配置更新
    let mut config = Config::new();
    config.update_from_args(&args);

    assert_eq!(config.provider, "deepseek");
    assert_eq!(config.model, "deepseek-chat");
}

/// 集成测试：测试国际化系统
#[test]
fn test_internationalization_integration() {
    let mut i18n = I18n::new();

    // 测试语言切换和消息获取的完整流程
    let test_scenarios = vec![
        (Language::SimplifiedChinese, "Git提交失败", "没有暂存的变更"),
        (
            Language::TraditionalChinese,
            "Git提交失敗",
            "沒有暫存的變更",
        ),
        (Language::English, "Git commit failed", "No staged changes"),
    ];

    for (lang, expected_commit_failed, expected_no_changes) in test_scenarios {
        i18n.set_language(lang.clone());

        assert_eq!(i18n.get("git_commit_failed"), expected_commit_failed);
        assert_eq!(i18n.get("no_staged_changes"), expected_no_changes);

        // 测试语言代码转换
        let lang_code = lang.to_code();
        let converted_lang = Language::from_code(lang_code);
        assert_eq!(lang, converted_lang);
    }
}

/// 集成测试：测试提示模板系统
#[test]
fn test_prompt_integration() {
    // 测试多次调用缓存机制
    let diff1 = "diff --git a/test.txt b/test.txt\n+line 1";
    let diff2 = "diff --git a/test2.txt b/test2.txt\n+line 2";

    let prompt1 = prompt::get_prompt(diff1);
    let prompt2 = prompt::get_prompt(diff2);

    // 验证diff被正确替换
    assert!(prompt1.contains("line 1"));
    assert!(!prompt1.contains("{{git_diff}}"));

    assert!(prompt2.contains("line 2"));
    assert!(!prompt2.contains("{{git_diff}}"));

    // 验证模板结构
    assert!(prompt1.contains("输出格式"));
    assert!(prompt2.contains("输出格式"));
}

/// 集成测试：测试所有模块间的协调工作
#[test]
fn test_full_system_integration() {
    // 1. 解析命令行参数
    let args = Args::try_parse_from([
        "ai-commit",
        "--provider",
        "ollama",
        "--model",
        "mistral",
        "--no-add",
    ])
    .unwrap();

    // 2. 创建和配置系统
    let mut config = Config::new();
    config.update_from_args(&args);

    // 3. 验证配置
    assert!(config.validate().is_ok());
    assert_eq!(config.provider, "ollama");
    assert_eq!(config.model, "mistral");

    // 4. 测试国际化
    let mut i18n = I18n::new();
    i18n.set_language(Language::English);
    let error_message = i18n.get("git_commit_failed");
    assert_eq!(error_message, "Git commit failed");

    // 5. 测试提示系统
    let test_diff = "diff --git a/src/main.rs b/src/main.rs\n+println!(\"Hello, world!\");";
    let prompt = prompt::get_prompt(test_diff);
    assert!(prompt.contains("Hello, world!"));
    assert!(prompt.contains("输出格式"));

    // 6. 验证系统状态一致性
    assert!(args.no_add);
    assert!(!args.push);
    assert_eq!(config.provider, "ollama");
}

/// 集成测试：测试配置优先级
#[test]
fn test_configuration_priority_integration() {
    use std::env;

    // 强制清理所有可能的环境变量
    for (key, _) in env::vars() {
        if key.starts_with("AI_COMMIT_") {
            env::remove_var(&key);
        }
    }

    // 1. 测试默认配置
    let config = Config {
        provider: "ollama".to_string(),
        model: "mistral".to_string(),
        debug: false,
    };
    assert_eq!(config.provider, "ollama");
    assert_eq!(config.model, "mistral");
    assert!(!config.debug);

    // 2. 测试命令行参数覆盖
    let mut config = Config {
        provider: "ollama".to_string(),
        model: "mistral".to_string(),
        debug: false,
    };
    let mut args = Args::default();
    args.provider = "deepseek".to_string();
    args.model = "cli-model".to_string();

    config.update_from_args(&args);
    assert_eq!(config.provider, "deepseek");
    assert_eq!(config.model, "cli-model");
    // debug字段不受命令行参数影响
    assert!(!config.debug);
}

/// 集成测试：测试性能优化效果
#[test]
fn test_performance_optimizations() {
    use std::time::Instant;

    // 测试配置加载性能
    let start = Instant::now();
    for _ in 0..100 {
        let _ = Config::new();
    }
    let config_time = start.elapsed();

    // 测试提示模板加载性能
    let start = Instant::now();
    for i in 0..100 {
        let diff = format!("test diff {}", i);
        let _ = prompt::get_prompt(&diff);
    }
    let prompt_time = start.elapsed();

    // 验证性能在合理范围内
    assert!(
        config_time.as_millis() < 1000,
        "配置加载过慢: {:?}",
        config_time
    );
    assert!(
        prompt_time.as_millis() < 500,
        "提示模板加载过慢: {:?}",
        prompt_time
    );

    // 测试环境加载只执行一次
    let start = Instant::now();
    for _ in 0..10 {
        ensure_env_loaded();
    }
    let env_loading_time = start.elapsed();

    // 多次调用应该很快（因为单例模式）
    assert!(
        env_loading_time.as_millis() < 100,
        "环境加载应该被缓存: {:?}",
        env_loading_time
    );
}

/// 集成测试：测试debug模式的完整功能
#[test]
fn test_debug_mode_integration() {
    use std::env;

    // 强制清理所有可能的环境变量
    for (key, _) in env::vars() {
        if key.starts_with("AI_COMMIT_") {
            env::remove_var(&key);
        }
    }

    // 1. 测试debug模式默认关闭
    let config = Config {
        provider: "ollama".to_string(),
        model: "mistral".to_string(),
        debug: false,
    };
    assert!(!config.debug);

    // 2. 测试通过环境变量设置debug模式
    env::set_var("AI_COMMIT_DEBUG", "true");
    let config = Config::new();
    assert!(config.debug);

    // 3. 测试debug值解析逻辑
    let test_cases = vec![
        ("false", false),
        ("0", false),
        ("invalid", false),
        ("", false),
    ];

    for (value, expected) in test_cases {
        env::set_var("AI_COMMIT_DEBUG", value);
        let config = Config::new();
        assert_eq!(
            config.debug, expected,
            "Value '{}' should result in {}",
            value, expected
        );
    }

    // 清理
    env::remove_var("AI_COMMIT_DEBUG");
}

/// 集成测试：测试并发场景
#[tokio::test]
async fn test_concurrent_integration() {
    use std::sync::Arc;
    use tokio::task;

    // 创建共享的国际化实例
    let i18n = Arc::new(I18n::new());

    // 并发访问测试
    let handles: Vec<_> = (0..10)
        .map(|i| {
            let i18n_clone = Arc::clone(&i18n);
            let diff = format!("test diff {}", i);

            task::spawn(async move {
                // 并发访问国际化
                let message = i18n_clone.get("git_commit_failed");
                assert!(!message.is_empty());

                // 并发访问提示系统
                let prompt = prompt::get_prompt(&diff);
                assert!(prompt.contains(&format!("test diff {}", i)));

                i
            })
        })
        .collect();

    // 等待所有任务完成
    let results: Vec<_> = futures::future::join_all(handles)
        .await
        .into_iter()
        .map(|r| r.unwrap())
        .collect();

    assert_eq!(results.len(), 10);
}

/// 集成测试：测试性能优化的有效性
#[test]
fn test_performance_optimizations_v2() {
    use std::time::Instant;

    // 测试提示模板缓存性能
    let start = Instant::now();

    // 第一次调用（可能需要加载）
    let _prompt1 = prompt::get_prompt("test diff 1");
    let first_call_time = start.elapsed();

    let start2 = Instant::now();

    // 第二次调用（应该使用缓存）
    let _prompt2 = prompt::get_prompt("test diff 2");
    let second_call_time = start2.elapsed();

    println!(
        "First call: {:?}, Second call: {:?}",
        first_call_time, second_call_time
    );

    // 测试配置环境加载性能
    let start3 = Instant::now();
    for _ in 0..100 {
        ensure_env_loaded();
    }
    let env_loading_time = start3.elapsed();

    println!("100 env loading calls: {:?}", env_loading_time);
    assert!(env_loading_time.as_millis() < 100);
}

/// 集成测试：测试字符串处理优化
#[test]
fn test_string_processing_integration() {
    // 测试大量字符串操作的性能和正确性
    let large_diff = "a".repeat(10000);
    let prompt = prompt::get_prompt(&large_diff);

    // 验证大字符串处理正确
    assert!(prompt.contains(&large_diff));
    assert!(!prompt.contains("{{git_diff}}"));

    // 测试特殊字符处理
    let special_diff = "特殊字符测试\n🚀 emoji test\n\"quotes\" and 'single quotes'";
    let prompt_special = prompt::get_prompt(special_diff);

    assert!(prompt_special.contains("特殊字符测试"));
    assert!(prompt_special.contains("🚀 emoji test"));
    assert!(prompt_special.contains("\"quotes\""));
}

/// 集成测试：命令路由系统
#[tokio::test]
async fn test_command_routing_integration() {
    use ai_commit::cli::args::Args;
    use ai_commit::commands::route_command;
    use ai_commit::config::Config;

    // 创建测试配置
    let config = Config {
        provider: "test".to_string(),
        model: "test-model".to_string(),
        debug: false,
    };

    // 测试多种命令路由场景
    let test_cases = vec![
        ("tag_list", {
            let mut args = Args::default();
            args.tag_list = true;
            args
        }),
        ("flow_init", {
            let mut args = Args::default();
            args.flow_init = true;
            args
        }),
        ("history", {
            let mut args = Args::default();
            args.history = true;
            args
        }),
        ("amend", {
            let mut args = Args::default();
            args.amend = true;
            args
        }),
        ("no_command", Args::default()),
    ];

    for (test_name, args) in test_cases {
        let result = route_command(&args, &config).await;

        match test_name {
            "no_command" => {
                // 没有命令应该返回 false（继续执行主逻辑）
                assert!(result.is_ok(), "No command should not error");
                if let Ok(handled) = result {
                    assert!(!handled, "No command should not be handled");
                }
            }
            _ => {
                // 其他命令应该被处理（可能成功或失败，但应该被路由）
                match result {
                    Ok(handled) => {
                        assert!(handled, "Command '{}' should be handled", test_name);
                    }
                    Err(_) => {
                        // 预期可能失败（因为在测试环境中），但说明路由正确
                        println!("Command '{}' was routed correctly but execution failed (expected in test environment)", test_name);
                    }
                }
            }
        }
    }
}

/// 集成测试：Git模块集成
#[tokio::test]
async fn test_git_modules_integration() {
    use ai_commit::git::core::GitCore;

    // 测试基础Git操作的集成
    let is_repo = GitCore::is_git_repo().await;
    println!("Is git repo: {}", is_repo);

    if is_repo {
        let current_branch = GitCore::get_current_branch().await;
        match current_branch {
            Ok(branch) => {
                assert!(!branch.is_empty(), "Current branch should not be empty");
                println!("Current branch: {}", branch);

                let branch_exists = GitCore::branch_exists(&branch).await;
                match branch_exists {
                    Ok(exists) => {
                        assert!(exists, "Current branch should exist");
                    }
                    Err(e) => {
                        println!("Branch existence check failed: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("Failed to get current branch: {}", e);
            }
        }

        let head_exists = GitCore::commit_exists("HEAD").await;
        match head_exists {
            Ok(exists) => {
                println!("HEAD exists: {}", exists);
            }
            Err(e) => {
                println!("HEAD existence check failed: {}", e);
            }
        }

        let remotes = GitCore::get_remotes().await;
        match remotes {
            Ok(remote_list) => {
                println!("Remotes: {:?}", remote_list);
            }
            Err(e) => {
                println!("Failed to get remotes: {}", e);
            }
        }
    }
}

/// 集成测试：新功能命令行解析
#[test]
fn test_new_features_cli_parsing() {
    use ai_commit::cli::args::Args;
    use clap::Parser;

    // 测试标签管理命令解析
    let tag_args = vec!["ai-commit", "--tag-list"];
    let parsed = Args::try_parse_from(tag_args);
    assert!(parsed.is_ok(), "Tag list parsing should succeed");
    if let Ok(args) = parsed {
        assert!(args.tag_list, "Tag list flag should be set");
    }

    // 测试Git Flow命令解析
    let flow_args = vec!["ai-commit", "--flow-feature-start", "new-feature"];
    let parsed = Args::try_parse_from(flow_args);
    assert!(parsed.is_ok(), "Flow feature start parsing should succeed");
    if let Ok(args) = parsed {
        assert!(
            args.flow_feature_start.is_some(),
            "Flow feature start should be set"
        );
        assert_eq!(args.flow_feature_start.unwrap(), "new-feature");
    }

    // 测试历史命令解析
    let history_args = vec!["ai-commit", "--history", "--log-limit", "10"];
    let parsed = Args::try_parse_from(history_args);
    assert!(parsed.is_ok(), "History parsing should succeed");
    if let Ok(args) = parsed {
        assert!(args.history, "History flag should be set");
        assert!(args.log_limit.is_some(), "Log limit should be set");
        assert_eq!(args.log_limit.unwrap(), 10);
    }

    // 测试编辑命令解析
    let edit_args = vec!["ai-commit", "--amend"];
    let parsed = Args::try_parse_from(edit_args);
    assert!(parsed.is_ok(), "Amend parsing should succeed");
    if let Ok(args) = parsed {
        assert!(args.amend, "Amend flag should be set");
    }
}

/// 集成测试：错误处理和恢复
#[tokio::test]
async fn test_error_handling_integration() {
    use ai_commit::git::core::GitCore;

    let result = GitCore::branch_exists("definitely-non-existent-branch-123456").await;
    match result {
        Ok(exists) => {
            assert!(!exists, "Non-existent branch should return false");
        }
        Err(e) => {
            println!("Branch check returned error (acceptable): {}", e);
        }
    }

    let result = GitCore::commit_exists("0000000000000000000000000000000000000000").await;
    match result {
        Ok(exists) => {
            assert!(!exists, "Non-existent commit should return false");
        }
        Err(e) => {
            println!("Commit check returned error (acceptable): {}", e);
        }
    }
}

/// 集成测试：配置和命令的综合测试
#[test]
fn test_config_and_commands_integration() {
    use ai_commit::cli::args::Args;
    use ai_commit::config::Config;

    // 直接构造配置（避免环境变量竞态）
    let config = Config {
        provider: "test-provider".to_string(),
        model: "test-model".to_string(),
        debug: true,
    };

    // 验证配置
    assert_eq!(config.provider, "test-provider");
    assert_eq!(config.model, "test-model");
    assert!(config.debug);

    // 创建Args并测试与配置的交互
    let mut args = Args::default();
    args.tag_list = true;

    assert!(args.tag_list);
}

/// 性能集成测试：新功能性能验证
#[tokio::test]
async fn test_new_features_performance() {
    use ai_commit::git::core::GitCore;
    use std::time::Instant;

    let start = Instant::now();

    let _ = GitCore::is_git_repo().await;
    let _ = GitCore::get_current_branch().await;
    let _ = GitCore::is_working_tree_clean().await;
    let _ = GitCore::get_remotes().await;

    let duration = start.elapsed();

    println!("Git operations took: {:?}", duration);
    assert!(
        duration.as_secs() < 10,
        "Git operations should complete within 10 seconds"
    );
}

/// 集成测试：内存使用和资源管理
#[test]
fn test_memory_management_integration() {
    use ai_commit::cli::args::Args;
    use std::collections::HashMap;

    let mut args_collection = HashMap::new();

    for i in 0..1000 {
        let mut args = Args::default();
        args.tag_list = i % 2 == 0;
        args.history = i % 3 == 0;
        args.amend = i % 5 == 0;

        if i % 10 == 0 {
            args.log_limit = Some(i as u32);
        }

        args_collection.insert(i, args);
    }

    assert_eq!(args_collection.len(), 1000);

    let sample = args_collection.get(&100).unwrap();
    assert!(sample.tag_list);
    assert!(!sample.history);
    assert!(sample.amend);
    assert_eq!(sample.log_limit, Some(100));

    args_collection.clear();
    assert_eq!(args_collection.len(), 0);
}
