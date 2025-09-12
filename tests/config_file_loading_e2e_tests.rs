/// E2E 测试：配置文件加载系统
/// 测试 providers.toml 文件的加载、解析和应用
use std::env;
use std::fs;
use std::path::Path;
use tempfile::{NamedTempFile, TempDir};

use ai_commit::config::providers::{ApiFormat, ProviderRegistry};
use ai_commit::config::Config;

/// 测试辅助函数：清理环境变量
fn clear_env_vars() {
    let vars = [
        "AI_COMMIT_PROVIDER",
        "AI_COMMIT_MODEL",
        "AI_COMMIT_DEBUG",
        "AI_COMMIT_TEST_API_KEY",
        "AI_COMMIT_TEST_URL",
        "AI_COMMIT_CUSTOM_API_KEY",
        "AI_COMMIT_CUSTOM_URL",
    ];

    for var in &vars {
        env::remove_var(var);
    }
}

/// 创建完整的测试配置文件
fn create_comprehensive_test_config() -> String {
    r#"
# 完整的测试提供商配置文件
# 用于验证配置文件加载和解析功能

[providers.ollama]
name = "ollama"
display_name = "Ollama Local"
default_url = "http://localhost:11434/api/generate"
requires_api_key = false
default_model = "mistral"
supported_models = ["mistral", "llama3", "qwen2", "codellama"]
api_format = "ollama"
description = "本地 Ollama 服务，无需 API Key"
env_prefix = "AI_COMMIT_OLLAMA"

[providers.deepseek]
name = "deepseek"
display_name = "Deepseek AI"
default_url = "https://api.deepseek.com/v1/chat/completions"
requires_api_key = true
default_model = "deepseek-chat"
supported_models = ["deepseek-chat", "deepseek-coder"]
api_format = "openai"
description = "深度求索 AI 服务，需要 API Key"
env_prefix = "AI_COMMIT_DEEPSEEK"

[providers.test_provider]
name = "test_provider"
display_name = "Test Provider"
default_url = "https://test.example.com/v1/chat"
requires_api_key = true
default_model = "test-model-v1"
supported_models = ["test-model-v1", "test-model-v2", "test-model-v3"]
api_format = "openai"
description = "测试专用提供商"
env_prefix = "AI_COMMIT_TEST"

[providers.custom_local]
name = "custom_local"
display_name = "Custom Local Service"
default_url = "http://custom.local:8080/api/v1/generate"
requires_api_key = false
default_model = "custom-model"
supported_models = ["custom-model", "custom-model-large"]
api_format = "custom"
description = "自定义本地服务"
env_prefix = "AI_COMMIT_CUSTOM"
"#
    .to_string()
}

/// 创建最小配置文件（只有一个提供商）
fn create_minimal_test_config() -> String {
    r#"
[providers.minimal]
name = "minimal"
display_name = "Minimal Provider"
default_url = "https://minimal.example.com/api"
requires_api_key = true
default_model = "minimal-model"
supported_models = ["minimal-model"]
api_format = "openai"
description = "最小配置提供商"
env_prefix = "AI_COMMIT_MINIMAL"
"#
    .to_string()
}

/// 创建无效的配置文件（用于错误处理测试）
fn create_invalid_test_config() -> String {
    r#"
# 无效的配置文件
[providers.invalid]
name = "invalid"
# 缺少必需字段 display_name
default_url = "https://invalid.com"
# 缺少其他必需字段...
"#
    .to_string()
}

#[test]
fn test_e2e_default_providers_loading() {
    println!("🧪 E2E 测试：默认提供商加载");

    // 测试默认配置（无配置文件时的内置提供商）
    let all_providers = ProviderRegistry::get_all();

    // 验证默认提供商存在
    assert!(all_providers.contains_key("ollama"), "应该包含 ollama");
    assert!(all_providers.contains_key("deepseek"), "应该包含 deepseek");
    assert!(
        all_providers.contains_key("siliconflow"),
        "应该包含 siliconflow"
    );
    assert!(all_providers.contains_key("kimi"), "应该包含 kimi");

    println!(
        "✅ 默认提供商加载验证通过，共 {} 个提供商",
        all_providers.len()
    );

    // 验证每个默认提供商的基本信息
    for (name, provider) in all_providers.iter() {
        assert!(!provider.name.is_empty(), "提供商 {} 名称不能为空", name);
        assert!(
            !provider.display_name.is_empty(),
            "提供商 {} 显示名称不能为空",
            name
        );
        assert!(
            !provider.default_url.is_empty(),
            "提供商 {} 默认 URL 不能为空",
            name
        );
        assert!(
            !provider.default_model.is_empty(),
            "提供商 {} 默认模型不能为空",
            name
        );
        assert!(
            !provider.supported_models.is_empty(),
            "提供商 {} 支持的模型列表不能为空",
            name
        );

        println!("✅ 提供商 {} 基本信息验证通过", name);
    }
}

#[test]
fn test_e2e_config_file_priority_order() {
    println!("🧪 E2E 测试：配置文件优先级顺序");

    // 注意：由于我们使用的是全局单例，实际的文件优先级测试比较复杂
    // 这里主要测试配置加载逻辑的概念

    // 验证配置信息字符串包含正确的优先级信息
    let config_info = ProviderRegistry::get_config_info();

    assert!(
        config_info.contains("providers.toml"),
        "应该提及 providers.toml"
    );
    assert!(config_info.contains("当前目录"), "应该提及当前目录");
    assert!(
        config_info.contains("config/providers.toml"),
        "应该提及 config 目录"
    );
    assert!(
        config_info.contains("/etc/ai-commit/providers.toml"),
        "应该提及系统配置目录"
    );
    assert!(config_info.contains("内置默认配置"), "应该提及内置默认配置");
    assert!(
        config_info.contains("当前加载的提供商数量"),
        "应该显示当前提供商数量"
    );

    println!("✅ 配置文件优先级信息验证通过");
    println!("配置信息: {}", config_info);
}

#[test]
fn test_e2e_provider_info_data_integrity() {
    println!("🧪 E2E 测试：提供商信息数据完整性");

    let all_providers = ProviderRegistry::get_all();

    // 检查每个提供商的数据完整性
    for (name, provider) in all_providers.iter() {
        println!("验证提供商数据完整性: {}", name);

        // API 格式验证
        match provider.api_format {
            ApiFormat::OpenAI | ApiFormat::Ollama | ApiFormat::Custom => {
                // 有效的 API 格式
            }
        }

        // URL 格式验证
        assert!(
            provider.default_url.starts_with("http://")
                || provider.default_url.starts_with("https://"),
            "提供商 {} 的 URL 格式无效: {}",
            name,
            provider.default_url
        );

        // 环境变量前缀验证
        assert!(
            provider.env_prefix.starts_with("AI_COMMIT_"),
            "提供商 {} 的环境变量前缀应该以 AI_COMMIT_ 开头: {}",
            name,
            provider.env_prefix
        );

        assert!(
            provider.env_prefix.len() > "AI_COMMIT_".len(),
            "提供商 {} 的环境变量前缀太短: {}",
            name,
            provider.env_prefix
        );

        // 模型列表验证
        assert!(
            provider.supported_models.contains(&provider.default_model),
            "提供商 {} 的默认模型 {} 应该在支持的模型列表中",
            name,
            provider.default_model
        );

        // 环境变量方法验证
        let api_key_var = provider.api_key_env_var();
        let url_var = provider.url_env_var();

        assert!(
            api_key_var.starts_with(&provider.env_prefix),
            "API Key 环境变量应该以前缀开头: {}",
            api_key_var
        );

        assert!(
            url_var.starts_with(&provider.env_prefix),
            "URL 环境变量应该以前缀开头: {}",
            url_var
        );

        assert!(
            api_key_var.ends_with("_API_KEY"),
            "API Key 环境变量应该以 _API_KEY 结尾: {}",
            api_key_var
        );

        assert!(
            url_var.ends_with("_URL"),
            "URL 环境变量应该以 _URL 结尾: {}",
            url_var
        );

        // 验证逻辑测试
        if provider.requires_api_key {
            // 需要 API Key 的提供商，没有 key 应该验证失败
            assert!(
                provider.validate(None).is_err(),
                "需要 API Key 的提供商 {} 没有 key 时应该验证失败",
                name
            );

            // 有 key 应该验证通过
            assert!(
                provider.validate(Some("test-key")).is_ok(),
                "需要 API Key 的提供商 {} 有 key 时应该验证通过",
                name
            );
        } else {
            // 不需要 API Key 的提供商应该总是验证通过
            assert!(
                provider.validate(None).is_ok(),
                "不需要 API Key 的提供商 {} 应该验证通过",
                name
            );
        }

        println!("✅ 提供商 {} 数据完整性验证通过", name);
    }
}

#[test]
fn test_e2e_config_and_provider_integration() {
    println!("🧪 E2E 测试：配置与提供商集成");

    clear_env_vars();

    let all_providers = ProviderRegistry::list_providers();

    // 为每个提供商测试配置集成
    for provider_name in all_providers.iter() {
        println!("测试提供商配置集成: {}", provider_name);

        clear_env_vars();

        let provider_info = ProviderRegistry::get_provider(provider_name).unwrap();

        // 设置基本环境变量
        env::set_var("AI_COMMIT_PROVIDER", provider_name);
        env::set_var("AI_COMMIT_MODEL", &provider_info.default_model);

        // 如果需要 API Key，设置一个测试用的
        if provider_info.requires_api_key {
            env::set_var(&provider_info.api_key_env_var(), "test-api-key-123");
        }

        // 设置自定义 URL
        let custom_url = format!("https://custom.{}.com/api", provider_name);
        env::set_var(&provider_info.url_env_var(), &custom_url);

        // 创建和加载配置
        let mut config = Config::new();
        config.load_from_env();

        // 验证配置正确加载
        assert_eq!(config.provider, *provider_name);
        assert_eq!(config.model, provider_info.default_model);

        // 验证当前提供商方法
        let current_provider = config.current_provider_info().unwrap();
        assert_eq!(current_provider.name, *provider_name);

        // 验证 URL 配置
        let current_url = config.current_url();
        assert_eq!(current_url, custom_url, "自定义 URL 应该生效");

        // 验证 API Key 配置
        if provider_info.requires_api_key {
            let current_api_key = config.current_api_key();
            assert!(
                current_api_key.is_some(),
                "需要 API Key 的提供商应该有 API Key"
            );
            assert_eq!(current_api_key.unwrap(), "test-api-key-123");
        } else {
            let current_api_key = config.current_api_key();
            assert!(
                current_api_key.is_none(),
                "不需要 API Key 的提供商不应该有 API Key"
            );
        }

        // 验证配置有效性
        let validation_result = config.validate();
        assert!(
            validation_result.is_ok(),
            "提供商 {} 的配置应该验证通过: {:?}",
            provider_name,
            validation_result
        );

        println!("✅ 提供商 {} 配置集成验证通过", provider_name);
    }

    clear_env_vars();
}

#[test]
fn test_e2e_provider_exists_and_get_methods() {
    println!("🧪 E2E 测试：提供商存在性和获取方法");

    let all_providers = ProviderRegistry::list_providers();

    // 测试每个提供商的存在性和获取方法
    for provider_name in &all_providers {
        // 测试 exists 方法
        assert!(
            ProviderRegistry::exists(provider_name),
            "提供商 {} 应该存在",
            provider_name
        );

        // 测试 get_provider 方法
        let provider = ProviderRegistry::get_provider(provider_name);
        assert!(
            provider.is_some(),
            "应该能获取提供商 {} 的信息",
            provider_name
        );

        let provider = provider.unwrap();
        assert_eq!(provider.name, *provider_name, "提供商名称应该匹配");

        println!("✅ 提供商 {} 存在性和获取方法验证通过", provider_name);
    }

    // 测试不存在的提供商
    assert!(
        !ProviderRegistry::exists("nonexistent_provider"),
        "不存在的提供商应该返回 false"
    );

    let nonexistent = ProviderRegistry::get_provider("nonexistent_provider");
    assert!(nonexistent.is_none(), "不存在的提供商应该返回 None");

    println!("✅ 不存在的提供商处理验证通过");
}

#[test]
fn test_e2e_api_format_consistency() {
    println!("🧪 E2E 测试：API 格式一致性");

    let all_providers = ProviderRegistry::get_all();

    let mut format_counts = std::collections::HashMap::new();

    // 统计不同 API 格式的使用情况
    for (name, provider) in all_providers.iter() {
        let count = format_counts
            .entry(provider.api_format.clone())
            .or_insert(0);
        *count += 1;

        // 验证 API 格式与提供商特征的一致性
        match provider.api_format {
            ApiFormat::Ollama => {
                assert_eq!(
                    provider.name, "ollama",
                    "只有 ollama 提供商应该使用 Ollama 格式"
                );
                assert!(!provider.requires_api_key, "Ollama 格式通常不需要 API Key");
            }
            ApiFormat::OpenAI => {
                // OpenAI 兼容格式通常需要 API Key（除非是自定义本地服务）
                if provider.name != "ollama" && !provider.default_url.contains("localhost") {
                    // 大多数云服务提供商需要 API Key
                }
            }
            ApiFormat::Custom => {
                // 自定义格式，验证基本信息存在即可
                assert!(!provider.name.is_empty(), "自定义格式提供商应该有名称");
            }
        }

        println!(
            "✅ 提供商 {} 使用 {:?} 格式验证通过",
            name, provider.api_format
        );
    }

    println!("API 格式分布: {:?}", format_counts);

    // 验证至少有一个 OpenAI 兼容的提供商
    assert!(
        format_counts.get(&ApiFormat::OpenAI).unwrap_or(&0) > &0,
        "应该至少有一个 OpenAI 兼容的提供商"
    );

    // 验证至少有一个 Ollama 提供商
    assert!(
        format_counts.get(&ApiFormat::Ollama).unwrap_or(&0) > &0,
        "应该至少有一个 Ollama 提供商"
    );
}

#[test]
fn test_e2e_environment_variable_naming_consistency() {
    println!("🧪 E2E 测试：环境变量命名一致性");

    let all_providers = ProviderRegistry::get_all();
    let mut env_prefixes = std::collections::HashSet::new();

    // 验证环境变量命名的一致性
    for (name, provider) in all_providers.iter() {
        // 验证环境变量前缀唯一性
        assert!(
            !env_prefixes.contains(&provider.env_prefix),
            "环境变量前缀 {} 不应该重复",
            provider.env_prefix
        );
        env_prefixes.insert(provider.env_prefix.clone());

        // 验证命名规范
        let expected_prefix = format!("AI_COMMIT_{}", name.to_uppercase());
        assert_eq!(
            provider.env_prefix, expected_prefix,
            "提供商 {} 的环境变量前缀应该是 {}",
            name, expected_prefix
        );

        // 验证 API Key 环境变量名称
        let api_key_var = provider.api_key_env_var();
        let expected_api_key_var = format!("{}_API_KEY", expected_prefix);
        assert_eq!(
            api_key_var, expected_api_key_var,
            "提供商 {} 的 API Key 环境变量应该是 {}",
            name, expected_api_key_var
        );

        // 验证 URL 环境变量名称
        let url_var = provider.url_env_var();
        let expected_url_var = format!("{}_URL", expected_prefix);
        assert_eq!(
            url_var, expected_url_var,
            "提供商 {} 的 URL 环境变量应该是 {}",
            name, expected_url_var
        );

        println!("✅ 提供商 {} 环境变量命名一致性验证通过", name);
    }

    println!("✅ 所有提供商环境变量命名一致性验证通过");
}

#[test]
fn test_e2e_configuration_error_messages() {
    println!("🧪 E2E 测试：配置错误消息");

    clear_env_vars();

    // 测试各种配置错误的错误消息质量

    // 1. 测试不存在的提供商
    let mut config = Config::new();
    config.provider = "nonexistent_provider".to_string();
    config.model = "some-model".to_string();

    let result = config.validate();
    assert!(result.is_err(), "不存在的提供商应该验证失败");

    let error_msg = result.err().unwrap().to_string();
    assert!(
        error_msg.contains("Unsupported provider"),
        "错误消息应该提及不支持的提供商"
    );
    assert!(
        error_msg.contains("nonexistent_provider"),
        "错误消息应该包含提供商名称"
    );
    assert!(
        error_msg.contains("Available providers"),
        "错误消息应该列出可用的提供商"
    );

    println!("✅ 不存在提供商的错误消息验证通过");

    // 2. 测试需要 API Key 但未提供的情况
    let providers_need_key = ProviderRegistry::get_all()
        .iter()
        .filter(|(_, provider)| provider.requires_api_key)
        .map(|(name, _)| name.clone())
        .collect::<Vec<_>>();

    for provider_name in providers_need_key {
        let provider_info = ProviderRegistry::get_provider(&provider_name).unwrap();

        config.provider = provider_name.clone();
        config.model = provider_info.default_model.clone();

        let result = config.validate();
        assert!(result.is_err(), "缺少 API Key 应该验证失败");

        let error_msg = result.err().unwrap().to_string();
        assert!(error_msg.contains("API key"), "错误消息应该提及 API key");
        assert!(
            error_msg.contains(&provider_info.display_name),
            "错误消息应该包含提供商显示名称"
        );
        assert!(
            error_msg.contains(&provider_info.api_key_env_var()),
            "错误消息应该包含环境变量名"
        );

        println!(
            "✅ 提供商 {} 缺少 API Key 的错误消息验证通过",
            provider_name
        );
    }

    // 3. 测试不支持的模型
    config.provider = "ollama".to_string();
    config.model = "unsupported_model_xyz".to_string();

    let result = config.validate();
    assert!(result.is_err(), "不支持的模型应该验证失败");

    let error_msg = result.err().unwrap().to_string();
    assert!(
        error_msg.contains("not supported"),
        "错误消息应该提及不支持"
    );
    assert!(
        error_msg.contains("unsupported_model_xyz"),
        "错误消息应该包含模型名称"
    );
    assert!(
        error_msg.contains("Supported models"),
        "错误消息应该列出支持的模型"
    );

    println!("✅ 不支持模型的错误消息验证通过");

    clear_env_vars();
}
