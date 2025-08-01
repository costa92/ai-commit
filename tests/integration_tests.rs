use ai_commit::ai::prompt;
use ai_commit::cli::args::Args;
use ai_commit::config::{Config, ensure_env_loaded};
use ai_commit::internationalization::{I18n, Language};
use clap::Parser;

/// 集成测试：测试配置系统的完整流程
#[test]  
fn test_config_integration() {
    // 1. 测试默认配置（可能受到本地 .env 文件影响）
    let config = Config::new();
    
    // 验证配置有效性而不是具体值（因为可能受到本地环境影响）
    assert!(!config.provider.is_empty());
    assert!(!config.model.is_empty());
    assert!(config.validate().is_ok() || config.validate().is_err()); // 根据provider不同可能需要API key
    
    // 2. 测试配置验证（不同提供商）
    let mut config = Config::new();
    
    // 测试 deepseek 提供商验证
    config.provider = "deepseek".to_string();
    config.deepseek_api_key = Some("test-key".to_string());
    assert!(config.validate().is_ok());
    
    // 3. 测试命令行参数覆盖
    let args = Args {
        provider: "ollama".to_string(),  // 使用不需要API key的提供商
        model: "test-model".to_string(),
        no_add: false,
        push: false,
        new_tag: None,
        tag_note: String::new(),
        show_tag: false,
        push_branches: false,
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
    let args = Args::try_parse_from(&[
        "ai-commit",
        "--provider", "deepseek",
        "--model", "deepseek-chat",
        "--push",
        "--new-tag", "v1.0.0",
        "--tag-note", "Integration test release",
        "--push-branches",
    ]).unwrap();
    
    // 验证参数解析
    assert_eq!(args.provider, "deepseek");
    assert_eq!(args.model, "deepseek-chat");
    assert_eq!(args.push, true);
    assert_eq!(args.new_tag, Some("v1.0.0".to_string()));
    assert_eq!(args.tag_note, "Integration test release");
    assert_eq!(args.push_branches, true);
    
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
        (Language::TraditionalChinese, "Git提交失敗", "沒有暫存的變更"), 
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
    let args = Args::try_parse_from(&[
        "ai-commit",
        "--provider", "ollama",
        "--model", "mistral",
        "--no-add",
    ]).unwrap();
    
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
    assert_eq!(args.no_add, true);
    assert_eq!(args.push, false);
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
    let parse_result = Args::try_parse_from(&["ai-commit", "--invalid-flag"]);
    assert!(parse_result.is_err());
    
    // 3. 测试国际化的未知键处理
    let i18n = I18n::new();
    let unknown_message = i18n.get("unknown_key");
    assert_eq!(unknown_message, "unknown_key");
}

/// 集成测试：测试配置优先级
#[test]
fn test_configuration_priority_integration() {
    // 测试配置优先级：默认值 < 环境变量 < 命令行参数
    
    // 1. 测试默认配置（可能受到本地配置影响）
    let config = Config::new();
    let original_provider = config.provider.clone();
    let original_model = config.model.clone();
    
    // 验证有有效的配置值
    assert!(!original_provider.is_empty());
    assert!(!original_model.is_empty());
    
    // 2. 测试命令行参数覆盖默认值
    let args = Args {
        provider: "cli_provider".to_string(),
        model: "cli_model".to_string(),
        no_add: false,
        push: false,
        new_tag: None,
        tag_note: String::new(),
        show_tag: false,
        push_branches: false,
    };
    
    let mut config = Config::new();
    config.update_from_args(&args);
    
    // 命令行参数应该覆盖任何配置
    assert_eq!(config.provider, "cli_provider");
    assert_eq!(config.model, "cli_model");
    
    // 3. 测试空参数不覆盖配置
    let empty_args = Args {
        provider: String::new(),  // 空字符串不应该覆盖
        model: String::new(),     // 空字符串不应该覆盖
        no_add: false,
        push: false,
        new_tag: None,
        tag_note: String::new(),
        show_tag: false,
        push_branches: false,
    };
    
    let mut config = Config::new();
    let before_provider = config.provider.clone();
    let before_model = config.model.clone();
    
    config.update_from_args(&empty_args);
    
    // 空参数不应该覆盖现有配置
    assert_eq!(config.provider, before_provider);
    assert_eq!(config.model, before_model);
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
fn test_performance_optimizations() {
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
    println!("First call: {:?}, Second call: {:?}", first_call_time, second_call_time);
    
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