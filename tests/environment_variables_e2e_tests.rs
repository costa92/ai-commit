use std::collections::HashMap;
/// E2E 测试：环境变量配置系统
/// 测试环境变量的设置、优先级、覆盖等完整功能
use std::env;

use ai_commit::config::providers::ProviderRegistry;
use ai_commit::config::Config;

/// 测试辅助函数：清理所有相关环境变量
fn clear_all_env_vars() {
    // 基础环境变量
    let basic_vars = ["AI_COMMIT_PROVIDER", "AI_COMMIT_MODEL", "AI_COMMIT_DEBUG"];

    // 所有提供商的环境变量
    let providers = ProviderRegistry::list_providers();
    let mut provider_vars = Vec::new();

    for provider_name in providers {
        if let Some(provider_info) = ProviderRegistry::get_provider(provider_name) {
            provider_vars.push(provider_info.api_key_env_var());
            provider_vars.push(provider_info.url_env_var());
        }
    }

    // 清理所有变量
    for var in basic_vars.iter().chain(provider_vars.iter()) {
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
    env::set_var("AI_COMMIT_DEEPSEEK_API_KEY", "test-key-123");

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
        current_vars.get("AI_COMMIT_DEEPSEEK_API_KEY"),
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

    // 创建配置并加载环境变量
    let mut config = Config::new();
    config.load_from_env();

    // 验证环境变量正确加载
    assert_eq!(config.provider, "ollama", "provider 应该从环境变量加载");
    assert_eq!(config.model, "llama3", "model 应该从环境变量加载");
    assert!(!config.debug, "debug 应该从环境变量加载");

    println!("✅ 基础环境变量加载验证通过");

    clear_all_env_vars();
}

#[test]
fn test_e2e_provider_specific_environment_variables() {
    println!("🧪 E2E 测试：提供商特定环境变量");

    let test_cases = [
        (
            "deepseek",
            "AI_COMMIT_DEEPSEEK_API_KEY",
            "AI_COMMIT_DEEPSEEK_URL",
            "deepseek-chat",
        ),
        (
            "siliconflow",
            "AI_COMMIT_SILICONFLOW_API_KEY",
            "AI_COMMIT_SILICONFLOW_URL",
            "qwen/Qwen2-7B-Instruct",
        ),
        (
            "kimi",
            "AI_COMMIT_KIMI_API_KEY",
            "AI_COMMIT_KIMI_URL",
            "moonshot-v1-8k",
        ),
    ];

    for (provider, api_key_var, url_var, model) in &test_cases {
        clear_all_env_vars();

        println!("测试提供商环境变量: {}", provider);

        // 设置提供商特定的环境变量
        env::set_var("AI_COMMIT_PROVIDER", provider);
        env::set_var("AI_COMMIT_MODEL", model);
        env::set_var(api_key_var, "test-api-key-456");
        env::set_var(url_var, "https://custom.example.com/api");

        // 加载配置
        let mut config = Config::new();
        config.load_from_env();

        // 验证基础配置
        assert_eq!(config.provider, *provider);
        assert_eq!(config.model, *model);

        // 验证提供商特定配置
        match *provider {
            "deepseek" => {
                assert_eq!(
                    config.deepseek_api_key(),
                    Some("test-api-key-456".to_string())
                );
                assert_eq!(config.deepseek_url(), "https://custom.example.com/api");
            }
            "siliconflow" => {
                assert_eq!(
                    config.siliconflow_api_key(),
                    Some("test-api-key-456".to_string())
                );
                assert_eq!(config.siliconflow_url(), "https://custom.example.com/api");
            }
            "kimi" => {
                assert_eq!(config.kimi_api_key(), Some("test-api-key-456".to_string()));
                assert_eq!(config.kimi_url(), "https://custom.example.com/api");
            }
            _ => panic!("未知的测试提供商: {}", provider),
        }

        // 验证当前提供商方法
        assert_eq!(
            config.current_api_key(),
            Some("test-api-key-456".to_string())
        );
        assert_eq!(config.current_url(), "https://custom.example.com/api");

        println!("✅ 提供商 {} 环境变量验证通过", provider);
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

    // 获取默认的 ollama 配置
    let default_ollama_url = default_config.ollama_url();
    let default_ollama_api_key = default_config.ollama_api_key();

    // 2. 使用环境变量覆盖默认值
    env::set_var("AI_COMMIT_PROVIDER", "ollama");
    env::set_var("AI_COMMIT_MODEL", "qwen2"); // 不是默认的 mistral
    env::set_var(
        "AI_COMMIT_OLLAMA_URL",
        "http://custom.ollama:11434/api/generate",
    );

    let mut override_config = Config::new();
    override_config.load_from_env();

    // 验证环境变量覆盖了默认值
    assert_eq!(override_config.provider, "ollama");
    assert_eq!(override_config.model, "qwen2", "环境变量应该覆盖默认模型");
    assert_eq!(
        override_config.ollama_url(),
        "http://custom.ollama:11434/api/generate",
        "环境变量应该覆盖默认 URL"
    );

    // API Key 应该保持默认（None，因为 Ollama 不需要）
    assert_eq!(override_config.ollama_api_key(), default_ollama_api_key);

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
        // 如果是空字符串，则不设置环境变量

        let mut config = Config::new();
        config.load_from_env();

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

    // 准备多个提供商的完整配置
    let provider_configs = vec![
        (
            "ollama",
            "mistral",
            None,
            Some("http://localhost:11434/api/generate"),
        ),
        (
            "deepseek",
            "deepseek-coder",
            Some("sk-deepseek-test"),
            Some("https://api.deepseek.com/v1/chat/completions"),
        ),
        (
            "siliconflow",
            "qwen/Qwen2-72B-Instruct",
            Some("sk-siliconflow-test"),
            Some("https://api.siliconflow.cn/v1/chat/completions"),
        ),
        (
            "kimi",
            "moonshot-v1-32k",
            Some("sk-kimi-test"),
            Some("https://api.moonshot.cn/v1/chat/completions"),
        ),
    ];

    for (provider, model, api_key, url) in &provider_configs {
        clear_all_env_vars();

        println!("切换到提供商: {}", provider);

        // 设置基础配置
        env::set_var("AI_COMMIT_PROVIDER", provider);
        env::set_var("AI_COMMIT_MODEL", model);
        env::set_var("AI_COMMIT_DEBUG", "false");

        // 设置提供商特定配置
        let provider_info = ProviderRegistry::get_provider(provider).unwrap();

        if let Some(key) = api_key {
            env::set_var(&provider_info.api_key_env_var(), key);
        }

        if let Some(custom_url) = url {
            env::set_var(&provider_info.url_env_var(), custom_url);
        }

        // 加载并验证配置
        let mut config = Config::new();
        config.load_from_env();

        assert_eq!(config.provider, *provider);
        assert_eq!(config.model, *model);

        // 验证当前提供商信息
        let current_provider = config.current_provider_info().unwrap();
        assert_eq!(current_provider.name, *provider);

        // 验证 API Key
        if api_key.is_some() {
            assert_eq!(config.current_api_key(), api_key.map(|s| s.to_string()));
        } else {
            assert_eq!(config.current_api_key(), None);
        }

        // 验证 URL
        if let Some(expected_url) = url {
            assert_eq!(config.current_url(), *expected_url);
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
fn test_e2e_environment_variable_isolation() {
    println!("🧪 E2E 测试：环境变量隔离");

    clear_all_env_vars();

    // 设置多个提供商的环境变量，验证它们互不干扰
    env::set_var("AI_COMMIT_DEEPSEEK_API_KEY", "deepseek-key");
    env::set_var("AI_COMMIT_DEEPSEEK_URL", "https://deepseek.custom.com");
    env::set_var("AI_COMMIT_SILICONFLOW_API_KEY", "siliconflow-key");
    env::set_var(
        "AI_COMMIT_SILICONFLOW_URL",
        "https://siliconflow.custom.com",
    );
    env::set_var("AI_COMMIT_KIMI_API_KEY", "kimi-key");
    env::set_var("AI_COMMIT_KIMI_URL", "https://kimi.custom.com");
    env::set_var(
        "AI_COMMIT_OLLAMA_URL",
        "http://ollama.custom.com:11434/api/generate",
    );

    // 测试每个提供商都能正确获取自己的配置
    let test_providers = ["deepseek", "siliconflow", "kimi", "ollama"];

    for provider in &test_providers {
        env::set_var("AI_COMMIT_PROVIDER", provider);

        let mut config = Config::new();
        config.load_from_env();

        assert_eq!(config.provider, *provider);

        // 验证每个提供商只获取自己的配置
        match *provider {
            "deepseek" => {
                assert_eq!(config.deepseek_api_key(), Some("deepseek-key".to_string()));
                assert_eq!(config.deepseek_url(), "https://deepseek.custom.com");
                assert_eq!(config.current_api_key(), Some("deepseek-key".to_string()));
                assert_eq!(config.current_url(), "https://deepseek.custom.com");

                // 验证其他提供商的配置不受影响但可以访问
                assert_eq!(
                    config.siliconflow_api_key(),
                    Some("siliconflow-key".to_string())
                );
                assert_eq!(config.kimi_api_key(), Some("kimi-key".to_string()));
            }
            "siliconflow" => {
                assert_eq!(
                    config.siliconflow_api_key(),
                    Some("siliconflow-key".to_string())
                );
                assert_eq!(config.siliconflow_url(), "https://siliconflow.custom.com");
                assert_eq!(
                    config.current_api_key(),
                    Some("siliconflow-key".to_string())
                );
                assert_eq!(config.current_url(), "https://siliconflow.custom.com");
            }
            "kimi" => {
                assert_eq!(config.kimi_api_key(), Some("kimi-key".to_string()));
                assert_eq!(config.kimi_url(), "https://kimi.custom.com");
                assert_eq!(config.current_api_key(), Some("kimi-key".to_string()));
                assert_eq!(config.current_url(), "https://kimi.custom.com");
            }
            "ollama" => {
                assert_eq!(config.ollama_api_key(), None); // Ollama 不需要 API Key
                assert_eq!(
                    config.ollama_url(),
                    "http://ollama.custom.com:11434/api/generate"
                );
                assert_eq!(config.current_api_key(), None);
                assert_eq!(
                    config.current_url(),
                    "http://ollama.custom.com:11434/api/generate"
                );
            }
            _ => panic!("未知的测试提供商: {}", provider),
        }

        println!("✅ 提供商 {} 环境变量隔离验证通过", provider);
    }

    clear_all_env_vars();
}

#[test]
fn test_e2e_environment_variable_fallback_to_defaults() {
    println!("🧪 E2E 测试：环境变量回退到默认值");

    clear_all_env_vars();

    // 只设置部分环境变量，验证其他配置使用默认值
    env::set_var("AI_COMMIT_PROVIDER", "deepseek");
    env::set_var("AI_COMMIT_DEEPSEEK_API_KEY", "partial-test-key");
    // 故意不设置 MODEL, DEBUG, URL 等其他变量

    let mut config = Config::new();
    config.load_from_env();

    // 验证设置的环境变量生效
    assert_eq!(config.provider, "deepseek");
    assert_eq!(
        config.deepseek_api_key(),
        Some("partial-test-key".to_string())
    );

    // 验证未设置的环境变量使用默认值
    let provider_info = ProviderRegistry::get_provider("deepseek").unwrap();
    assert_eq!(
        config.model, provider_info.default_model,
        "模型应该使用默认值"
    );
    assert!(!config.debug, "debug 应该使用默认值 false");
    assert_eq!(
        config.deepseek_url(),
        provider_info.default_url,
        "URL 应该使用默认值"
    );

    // 验证其他提供商使用默认配置
    let ollama_info = ProviderRegistry::get_provider("ollama").unwrap();
    assert_eq!(config.ollama_url(), ollama_info.default_url);
    assert_eq!(config.ollama_api_key(), None);

    println!("✅ 环境变量回退到默认值验证通过");

    clear_all_env_vars();
}

#[test]
fn test_e2e_environment_variable_validation_integration() {
    println!("🧪 E2E 测试：环境变量与验证集成");

    clear_all_env_vars();

    // 测试有效的环境变量配置
    env::set_var("AI_COMMIT_PROVIDER", "kimi");
    env::set_var("AI_COMMIT_MODEL", "moonshot-v1-128k");
    env::set_var("AI_COMMIT_KIMI_API_KEY", "valid-kimi-key");
    env::set_var("AI_COMMIT_DEBUG", "true");

    let mut config = Config::new();
    config.load_from_env();

    // 验证配置加载
    assert_eq!(config.provider, "kimi");
    assert_eq!(config.model, "moonshot-v1-128k");
    assert!(config.debug);
    assert_eq!(config.kimi_api_key(), Some("valid-kimi-key".to_string()));

    // 验证配置有效性
    let validation_result = config.validate();
    assert!(validation_result.is_ok(), "有效的环境变量配置应该验证通过");

    println!("✅ 有效环境变量配置验证通过");

    // 测试无效的环境变量配置
    clear_all_env_vars();

    env::set_var("AI_COMMIT_PROVIDER", "deepseek");
    env::set_var("AI_COMMIT_MODEL", "deepseek-chat");
    // 故意不设置必需的 API Key

    config = Config::new();
    config.load_from_env();

    let validation_result = config.validate();
    assert!(
        validation_result.is_err(),
        "缺少 API Key 的配置应该验证失败"
    );

    let error_msg = validation_result.err().unwrap().to_string();
    assert!(
        error_msg.contains("Deepseek API key"),
        "错误消息应该提及 Deepseek API key"
    );

    println!("✅ 无效环境变量配置验证通过");

    clear_all_env_vars();
}

#[test]
fn test_e2e_all_provider_environment_variables() {
    println!("🧪 E2E 测试：所有提供商环境变量");

    let all_providers = ProviderRegistry::list_providers();

    // 为每个提供商测试完整的环境变量配置
    for provider_name in &all_providers {
        clear_all_env_vars();

        println!("测试提供商完整环境变量: {}", provider_name);

        let provider_info = ProviderRegistry::get_provider(provider_name).unwrap();

        // 设置完整的环境变量配置
        env::set_var("AI_COMMIT_PROVIDER", provider_name);
        env::set_var("AI_COMMIT_MODEL", &provider_info.default_model);
        env::set_var("AI_COMMIT_DEBUG", "false");

        // 设置提供商特定环境变量
        if provider_info.requires_api_key {
            env::set_var(&provider_info.api_key_env_var(), "test-key-for-validation");
        }

        let custom_url = format!("https://custom-{}.example.com/api", provider_name);
        env::set_var(&provider_info.url_env_var(), &custom_url);

        // 加载配置
        let mut config = Config::new();
        config.load_from_env();

        // 验证基础配置
        assert_eq!(config.provider, *provider_name);
        assert_eq!(config.model, provider_info.default_model);
        assert!(!config.debug);

        // 验证提供商特定配置
        if provider_info.requires_api_key {
            assert_eq!(
                config.current_api_key(),
                Some("test-key-for-validation".to_string())
            );
        } else {
            assert_eq!(config.current_api_key(), None);
        }

        assert_eq!(config.current_url(), custom_url);

        // 验证当前提供商信息
        let current_provider = config.current_provider_info().unwrap();
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
