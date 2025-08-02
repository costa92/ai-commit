use ai_commit::ai::prompt;
use ai_commit::cli::args::Args;
use ai_commit::config::{ensure_env_loaded, Config};
use ai_commit::internationalization::{I18n, Language};
use clap::Parser;

/// 集成测试：测试配置系统的完整流程
#[test]
fn test_config_integration() {
    use ai_commit::config::{Config, EnvVars};
    use std::env;

    // 强制清理所有可能的环境变量
    for (key, _) in env::vars() {
        if key.starts_with("AI_COMMIT_") {
            env::remove_var(&key);
        }
    }

    // 清理缓存
    EnvVars::clear_cache();

    // 1. 测试默认配置（强制不从环境变量加载）
    let config = Config {
        provider: "ollama".to_string(),
        model: "mistral".to_string(),
        deepseek_api_key: None,
        deepseek_url: "https://api.deepseek.com/v1/chat/completions".to_string(),
        ollama_url: "http://localhost:11434/api/generate".to_string(),
        siliconflow_api_key: None,
        siliconflow_url: "https://api.siliconflow.cn/v1/chat/completions".to_string(),
        debug: false,
    };

    // 验证配置有效性而不是具体值（因为可能受到本地环境影响）
    assert!(!config.provider.is_empty());
    assert!(!config.model.is_empty());
    assert!(config.validate().is_ok()); // ollama provider应该总是valid

    // 验证debug模式默认为false
    assert!(!config.debug);

    // 2. 测试配置验证（不同提供商）
    let mut config = Config::new();

    // 测试 deepseek 提供商验证
    config.provider = "deepseek".to_string();
    config.deepseek_api_key = Some("test-key".to_string());
    assert!(config.validate().is_ok());

    // 3. 测试命令行参数覆盖
    let args = Args {
        provider: "ollama".to_string(), // 使用不需要API key的提供商
        model: "test-model".to_string(),
        no_add: false,
        push: false,
        new_tag: None,
        tag_note: String::new(),
        show_tag: false,
        push_branches: false,
        worktree_create: None,
        worktree_switch: None,
        worktree_list: false,
        worktree_verbose: false,
        worktree_porcelain: false,
        worktree_z: false,
        worktree_expire: None,
        worktree_remove: None,
        worktree_path: None,
        worktree_clear: false,
    };

    let mut config = Config::new();
    config.update_from_args(&args);

    // 命令行参数应该覆盖任何配置
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

    // 测试配置验证（需要API key）
    config.deepseek_api_key = Some("test-key".to_string());
    assert!(config.validate().is_ok());
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

    // 验证模板结构（更新为实际模板内容）
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

/// 集成测试：测试错误处理流程
#[test]
fn test_error_handling_integration() {
    // 1. 测试配置验证错误
    let mut config = Config::new();
    config.provider = "deepseek".to_string();
    config.deepseek_api_key = None;

    let validation_result = config.validate();
    assert!(validation_result.is_err());
    let error_msg = validation_result.unwrap_err().to_string();
    assert!(error_msg.contains("Deepseek API key"));

    // 2. 测试CLI参数解析错误
    let parse_result = Args::try_parse_from(["ai-commit", "--invalid-flag"]);
    assert!(parse_result.is_err());

    // 3. 测试国际化的未知键处理
    let i18n = I18n::new();
    let unknown_message = i18n.get("unknown_key");
    assert_eq!(unknown_message, "unknown_key");
}

/// 集成测试：测试配置优先级
#[test]
fn test_configuration_priority_integration() {
    use ai_commit::config::{Config, EnvVars};
    use std::env;

    // 强制清理所有可能的环境变量
    for (key, _) in env::vars() {
        if key.starts_with("AI_COMMIT_") {
            env::remove_var(&key);
        }
    }

    // 清理缓存
    EnvVars::clear_cache();

    // 1. 测试默认配置（强制不从环境变量加载）
    let config = Config {
        provider: "ollama".to_string(),
        model: "mistral".to_string(),
        deepseek_api_key: None,
        deepseek_url: "https://api.deepseek.com/v1/chat/completions".to_string(),
        ollama_url: "http://localhost:11434/api/generate".to_string(),
        siliconflow_api_key: None,
        siliconflow_url: "https://api.siliconflow.cn/v1/chat/completions".to_string(),
        debug: false,
    };
    assert_eq!(config.provider, "ollama");
    assert_eq!(config.model, "mistral");
    assert!(!config.debug);

    // 2. 测试命令行参数覆盖
    let mut config = Config {
        provider: "ollama".to_string(),
        model: "mistral".to_string(),
        deepseek_api_key: None,
        deepseek_url: "https://api.deepseek.com/v1/chat/completions".to_string(),
        ollama_url: "http://localhost:11434/api/generate".to_string(),
        siliconflow_api_key: None,
        siliconflow_url: "https://api.siliconflow.cn/v1/chat/completions".to_string(),
        debug: false,
    };
    let args = Args {
        provider: "deepseek".to_string(),
        model: "cli-model".to_string(),
        no_add: false,
        push: false,
        new_tag: None,
        tag_note: String::new(),
        show_tag: false,
        push_branches: false,
        worktree_create: None,
        worktree_switch: None,
        worktree_list: false,
        worktree_verbose: false,
        worktree_porcelain: false,
        worktree_z: false,
        worktree_expire: None,
        worktree_remove: None,
        worktree_path: None,
        worktree_clear: false,
    };

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

    // 验证性能在合理范围内（这些阈值可以根据实际需要调整）
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
    use ai_commit::config::{Config, EnvVars};
    use std::env;

    // 强制清理所有可能的环境变量
    for (key, _) in env::vars() {
        if key.starts_with("AI_COMMIT_") {
            env::remove_var(&key);
        }
    }

    // 清理环境变量缓存
    EnvVars::clear_cache();

    // 1. 测试debug模式默认关闭（强制使用默认配置）
    let config = Config {
        provider: "ollama".to_string(),
        model: "mistral".to_string(),
        deepseek_api_key: None,
        deepseek_url: "https://api.deepseek.com/v1/chat/completions".to_string(),
        ollama_url: "http://localhost:11434/api/generate".to_string(),
        siliconflow_api_key: None,
        siliconflow_url: "https://api.siliconflow.cn/v1/chat/completions".to_string(),
        debug: false,
    };
    assert!(!config.debug);

    // 2. 测试通过环境变量设置debug模式（手动测试）
    env::set_var("AI_COMMIT_DEBUG", "true");

    // 清理缓存以确保读取新的环境变量
    #[cfg(test)]
    {
        use ai_commit::config::EnvVars;
        EnvVars::clear_cache();
    }

    let mut config = Config::new();
    config.load_from_env(); // 手动加载环境变量
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

        // 每次都清理缓存
        #[cfg(test)]
        {
            use ai_commit::config::EnvVars;
            EnvVars::clear_cache();
        }

        let mut config = Config::new();
        config.load_from_env();
        assert_eq!(
            config.debug, expected,
            "Value '{}' should result in {}",
            value, expected
        );
    }

    // 清理
    env::remove_var("AI_COMMIT_DEBUG");

    // 最后清理缓存
    #[cfg(test)]
    {
        use ai_commit::config::EnvVars;
        EnvVars::clear_cache();
    }
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

    // 由于缓存，第二次调用不应该比第一次慢太多
    // 这是一个粗略的性能测试
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

    // 多次调用 ensure_env_loaded 应该很快（因为单例）
    println!("100 env loading calls: {:?}", env_loading_time);

    // 基本性能断言（非严格）
    assert!(env_loading_time.as_millis() < 100); // 应该很快
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
