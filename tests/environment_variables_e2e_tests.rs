use std::collections::HashMap;
/// E2E 测试：环境变量配置系统
/// 测试环境变量的设置、优先级、覆盖等完整功能
use std::env;

use ai_commit::config::providers::ProviderRegistry;
use ai_commit::config::Config;

/// 测试辅助函数：清理所有相关环境变量
fn clear_all_env_vars() {
    let vars = [
        "AI_COMMIT_PROVIDER",
        "AI_COMMIT_MODEL",
        "AI_COMMIT_DEBUG",
        "AI_COMMIT_PROVIDER_API_KEY",
        "AI_COMMIT_PROVIDER_URL",
    ];
    for var in &vars {
        env::remove_var(var);
    }
}

/// 获取当前所有 AI_COMMIT_* 环境变量
fn get_ai_commit_env_vars() -> HashMap<String, String> {
    env::vars()
        .filter(|(key, _)| key.starts_with("AI_COMMIT_"))
        .collect()
}

#[test]
fn test_e2e_environment_variable_detection() {
    println!("🧪 E2E 测试：环境变量检测");

    clear_all_env_vars();

    // 验证环境变量已清理
    let initial_vars = get_ai_commit_env_vars();
    assert!(initial_vars.is_empty(), "应该没有 AI_COMMIT_* 环境变量");

    // 设置一些环境变量
    env::set_var("AI_COMMIT_PROVIDER", "deepseek");
    env::set_var("AI_COMMIT_MODEL", "deepseek-coder");
    env::set_var("AI_COMMIT_DEBUG", "true");
    env::set_var("AI_COMMIT_PROVIDER_API_KEY", "test-key-123");

    // 验证环境变量设置成功
    let current_vars = get_ai_commit_env_vars();
    assert_eq!(current_vars.len(), 4, "应该有 4 个 AI_COMMIT_* 环境变量");

    assert_eq!(
        current_vars.get("AI_COMMIT_PROVIDER"),
        Some(&"deepseek".to_string())
    );
    assert_eq!(
        current_vars.get("AI_COMMIT_MODEL"),
        Some(&"deepseek-coder".to_string())
    );
    assert_eq!(
        current_vars.get("AI_COMMIT_DEBUG"),
        Some(&"true".to_string())
    );
    assert_eq!(
        current_vars.get("AI_COMMIT_PROVIDER_API_KEY"),
        Some(&"test-key-123".to_string())
    );

    println!("✅ 环境变量检测验证通过");

    clear_all_env_vars();
}

#[test]
fn test_e2e_basic_environment_variable_loading() {
    println!("🧪 E2E 测试：基础环境变量加载");

    clear_all_env_vars();

    // 设置基础配置环境变量
    env::set_var("AI_COMMIT_PROVIDER", "ollama");
    env::set_var("AI_COMMIT_MODEL", "llama3");
    env::set_var("AI_COMMIT_DEBUG", "false");

    // Config::new() 自动从环境变量加载
    let config = Config::new();

    // 验证环境变量正确加载
    assert_eq!(config.provider, "ollama", "provider 应该从环境变量加载");
    assert_eq!(config.model, "llama3", "model 应该从环境变量加载");
    assert!(!config.debug, "debug 应该从环境变量加载");

    println!("✅ 基础环境变量加载验证通过");

    clear_all_env_vars();
}

#[test]
fn test_e2e_unified_provider_api_key_and_url() {
    println!("🧪 E2E 测试：统一提供商 API Key 和 URL");

    let test_cases = [
        ("deepseek", "deepseek-chat"),
        ("siliconflow", "qwen/Qwen2-7B-Instruct"),
        ("kimi", "moonshot-v1-8k"),
    ];

    for (provider, model) in &test_cases {
        clear_all_env_vars();

        println!("测试提供商: {}", provider);

        // 设置统一的环境变量
        env::set_var("AI_COMMIT_PROVIDER", provider);
        env::set_var("AI_COMMIT_MODEL", model);
        env::set_var("AI_COMMIT_PROVIDER_API_KEY", "test-api-key-456");
        env::set_var("AI_COMMIT_PROVIDER_URL", "https://custom.example.com/api");

        let config = Config::new();

        // 验证基础配置
        assert_eq!(config.provider, *provider);
        assert_eq!(config.model, *model);

        // 验证统一 API Key 和 URL
        assert_eq!(
            config.get_api_key(),
            Some("test-api-key-456".to_string()),
            "API Key 应该从统一环境变量加载"
        );
        assert_eq!(
            config.get_url(),
            "https://custom.example.com/api",
            "URL 应该从统一环境变量加载"
        );

        println!("✅ 提供商 {} 统一环境变量验证通过", provider);
    }

    clear_all_env_vars();
}

#[test]
fn test_e2e_environment_variable_override_defaults() {
    println!("🧪 E2E 测试：环境变量覆盖默认值");

    clear_all_env_vars();

    // 1. 测试默认配置
    let default_config = Config::new();
    assert_eq!(default_config.provider, "ollama", "默认提供商应该是 ollama");
    assert_eq!(default_config.model, "mistral", "默认模型应该是 mistral");

    // 2. 使用环境变量覆盖默认值
    env::set_var("AI_COMMIT_PROVIDER", "ollama");
    env::set_var("AI_COMMIT_MODEL", "qwen2"); // 不是默认的 mistral
    env::set_var(
        "AI_COMMIT_PROVIDER_URL",
        "http://custom.ollama:11434/api/generate",
    );

    let override_config = Config::new();

    // 验证环境变量覆盖了默认值
    assert_eq!(override_config.provider, "ollama");
    assert_eq!(override_config.model, "qwen2", "环境变量应该覆盖默认模型");
    assert_eq!(
        override_config.get_url(),
        "http://custom.ollama:11434/api/generate",
        "环境变量应该覆盖默认 URL"
    );

    // API Key 应该为 None（Ollama 不需要，且未设置统一 Key）
    assert_eq!(override_config.get_api_key(), None);

    println!("✅ 环境变量覆盖默认值验证通过");

    clear_all_env_vars();
}

#[test]
fn test_e2e_debug_mode_environment_variables() {
    println!("🧪 E2E 测试：调试模式环境变量");

    let debug_test_cases = [
        ("true", true),
        ("TRUE", true),
        ("True", true),
        ("1", true),
        ("yes", false), // 只接受 true/1
        ("false", false),
        ("FALSE", false),
        ("0", false),
        ("no", false),
        ("invalid", false),
        ("", false),
    ];

    for (debug_value, expected) in &debug_test_cases {
        clear_all_env_vars();

        println!("测试调试值: '{}' -> {}", debug_value, expected);

        env::set_var("AI_COMMIT_PROVIDER", "ollama");

        if !debug_value.is_empty() {
            env::set_var("AI_COMMIT_DEBUG", debug_value);
        }

        let config = Config::new();

        assert_eq!(
            config.debug, *expected,
            "调试值 '{}' 应该解析为 {}",
            debug_value, expected
        );
    }

    println!("✅ 调试模式环境变量验证通过");

    clear_all_env_vars();
}

#[test]
fn test_e2e_multiple_providers_environment_switching() {
    println!("🧪 E2E 测试：多提供商环境切换");

    let provider_configs = vec![
        ("ollama", "mistral", false),
        ("deepseek", "deepseek-coder", true),
        ("siliconflow", "qwen/Qwen2-72B-Instruct", true),
        ("kimi", "moonshot-v1-32k", true),
    ];

    for (provider, model, needs_key) in &provider_configs {
        clear_all_env_vars();

        println!("切换到提供商: {}", provider);

        env::set_var("AI_COMMIT_PROVIDER", provider);
        env::set_var("AI_COMMIT_MODEL", model);
        env::set_var("AI_COMMIT_DEBUG", "false");

        if *needs_key {
            env::set_var("AI_COMMIT_PROVIDER_API_KEY", "test-key-for-switch");
        }

        let config = Config::new();

        assert_eq!(config.provider, *provider);
        assert_eq!(config.model, *model);

        // 验证当前提供商信息
        let current_provider = config.get_current_provider_info().unwrap();
        assert_eq!(current_provider.name, *provider);

        // 验证 API Key
        if *needs_key {
            assert_eq!(
                config.get_api_key(),
                Some("test-key-for-switch".to_string())
            );
        } else {
            assert_eq!(config.get_api_key(), None);
        }

        // 验证配置有效性
        let validation_result = config.validate();
        assert!(
            validation_result.is_ok(),
            "提供商 {} 的环境配置应该有效: {:?}",
            provider,
            validation_result
        );

        println!("✅ 提供商 {} 环境切换验证通过", provider);
    }

    clear_all_env_vars();
}

#[test]
fn test_e2e_url_fallback_to_provider_default() {
    println!("🧪 E2E 测试：URL 回退到提供商默认值");

    clear_all_env_vars();

    // 只设置 provider，不设置 URL，应该使用提供商默认 URL
    env::set_var("AI_COMMIT_PROVIDER", "deepseek");
    env::set_var("AI_COMMIT_PROVIDER_API_KEY", "test-key");

    let config = Config::new();

    let provider_info = ProviderRegistry::get_provider("deepseek").unwrap();
    assert_eq!(
        config.get_url(),
        provider_info.default_url,
        "URL 应该使用提供商默认值"
    );

    // 设置自定义 URL 后应该覆盖默认值
    env::set_var("AI_COMMIT_PROVIDER_URL", "https://custom.deepseek.com/api");
    let config2 = Config::new();
    assert_eq!(
        config2.get_url(),
        "https://custom.deepseek.com/api",
        "自定义 URL 应该覆盖默认值"
    );

    println!("✅ URL 回退验证通过");

    clear_all_env_vars();
}

#[test]
fn test_e2e_environment_variable_validation_integration() {
    println!("🧪 E2E 测试：环境变量与验证集成");

    clear_all_env_vars();

    // 测试有效的环境变量配置
    env::set_var("AI_COMMIT_PROVIDER", "kimi");
    env::set_var("AI_COMMIT_MODEL", "moonshot-v1-128k");
    env::set_var("AI_COMMIT_PROVIDER_API_KEY", "valid-kimi-key");
    env::set_var("AI_COMMIT_DEBUG", "true");

    let config = Config::new();

    assert_eq!(config.provider, "kimi");
    assert_eq!(config.model, "moonshot-v1-128k");
    assert!(config.debug);
    assert_eq!(config.get_api_key(), Some("valid-kimi-key".to_string()));

    let validation_result = config.validate();
    assert!(validation_result.is_ok(), "有效的环境变量配置应该验证通过");

    println!("✅ 有效环境变量配置验证通过");

    // 测试无效的环境变量配置（缺少必需的 API Key）
    clear_all_env_vars();

    env::set_var("AI_COMMIT_PROVIDER", "deepseek");
    env::set_var("AI_COMMIT_MODEL", "deepseek-chat");
    // 故意不设置 API Key

    let config = Config::new();
    let validation_result = config.validate();
    assert!(
        validation_result.is_err(),
        "缺少 API Key 的配置应该验证失败"
    );

    let error_msg = validation_result.err().unwrap().to_string();
    assert!(
        error_msg.contains("API key"),
        "错误消息应该提及 API key, got: {}",
        error_msg
    );

    println!("✅ 无效环境变量配置验证通过");

    clear_all_env_vars();
}

#[test]
fn test_e2e_all_provider_environment_variables() {
    println!("🧪 E2E 测试：所有提供商环境变量");

    let all_providers = ProviderRegistry::list_providers();

    for provider_name in &all_providers {
        clear_all_env_vars();

        println!("测试提供商完整环境变量: {}", provider_name);

        let provider_info = ProviderRegistry::get_provider(provider_name).unwrap();

        // 设置完整的环境变量配置
        env::set_var("AI_COMMIT_PROVIDER", provider_name);
        env::set_var("AI_COMMIT_MODEL", &provider_info.default_model);
        env::set_var("AI_COMMIT_DEBUG", "false");

        if provider_info.requires_api_key {
            env::set_var("AI_COMMIT_PROVIDER_API_KEY", "test-key-for-validation");
        }

        let custom_url = format!("https://custom-{}.example.com/api", provider_name);
        env::set_var("AI_COMMIT_PROVIDER_URL", &custom_url);

        let config = Config::new();

        // 验证基础配置
        assert_eq!(config.provider, *provider_name);
        assert_eq!(config.model, provider_info.default_model);
        assert!(!config.debug);

        // 验证 API Key
        if provider_info.requires_api_key {
            assert_eq!(
                config.get_api_key(),
                Some("test-key-for-validation".to_string())
            );
        }

        // 验证 URL
        assert_eq!(config.get_url(), custom_url);

        // 验证当前提供商信息
        let current_provider = config.get_current_provider_info().unwrap();
        assert_eq!(current_provider.name, *provider_name);

        // 验证配置有效性
        let validation_result = config.validate();
        assert!(
            validation_result.is_ok(),
            "提供商 {} 的完整环境变量配置应该有效: {:?}",
            provider_name,
            validation_result
        );

        println!("✅ 提供商 {} 完整环境变量验证通过", provider_name);
    }

    clear_all_env_vars();
}

#[test]
fn test_e2e_config_default_values() {
    println!("🧪 E2E 测试：配置默认值");

    clear_all_env_vars();

    let config = Config::new();

    assert_eq!(config.provider, "ollama", "默认 provider 应该是 ollama");
    assert_eq!(config.model, "mistral", "默认 model 应该是 mistral");
    assert!(!config.debug, "默认 debug 应该是 false");
    assert_eq!(config.get_api_key(), None, "默认 API Key 应该是 None");

    // 默认 URL 应该来自 ollama provider info
    let ollama_info = ProviderRegistry::get_provider("ollama").unwrap();
    assert_eq!(
        config.get_url(),
        ollama_info.default_url,
        "默认 URL 应该是 ollama 的默认 URL"
    );

    println!("✅ 配置默认值验证通过");

    clear_all_env_vars();
}
