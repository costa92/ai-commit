use std::collections::{HashMap, HashSet};
/// E2E 测试：多提供商切换系统
/// 测试在不同 AI 提供商之间的无缝切换功能
use std::env;

use ai_commit::config::providers::{ApiFormat, ProviderRegistry};
use ai_commit::config::Config;

/// 测试辅助函数：清理环境变量
fn clear_env_vars() {
    // 获取所有提供商的环境变量
    let providers = ProviderRegistry::list_providers();
    let mut vars_to_clear = vec!["AI_COMMIT_PROVIDER", "AI_COMMIT_MODEL", "AI_COMMIT_DEBUG"];

    for provider_name in providers {
        if let Some(provider_info) = ProviderRegistry::get_provider(provider_name) {
            vars_to_clear.push(provider_info.api_key_env_var().leak());
            vars_to_clear.push(provider_info.url_env_var().leak());
        }
    }

    for var in vars_to_clear {
        env::remove_var(var);
    }
}

/// 提供商切换测试场景
#[derive(Debug)]
struct ProviderSwitchScenario {
    name: &'static str,
    provider: &'static str,
    model: &'static str,
    api_key: Option<&'static str>,
    custom_url: Option<&'static str>,
    should_validate: bool,
    description: &'static str,
}

/// 获取所有提供商切换测试场景
fn get_provider_switch_scenarios() -> Vec<ProviderSwitchScenario> {
    vec![
        ProviderSwitchScenario {
            name: "ollama_local",
            provider: "ollama",
            model: "mistral",
            api_key: None,
            custom_url: Some("http://localhost:11434/api/generate"),
            should_validate: true,
            description: "本地 Ollama 服务，无需 API Key",
        },
        ProviderSwitchScenario {
            name: "ollama_custom",
            provider: "ollama",
            model: "llama3",
            api_key: None,
            custom_url: Some("http://custom.ollama:8080/api/generate"),
            should_validate: true,
            description: "自定义 Ollama 服务地址",
        },
        ProviderSwitchScenario {
            name: "deepseek_chat",
            provider: "deepseek",
            model: "deepseek-chat",
            api_key: Some("sk-deepseek-test-key"),
            custom_url: Some("https://api.deepseek.com/v1/chat/completions"),
            should_validate: true,
            description: "Deepseek 聊天模型",
        },
        ProviderSwitchScenario {
            name: "deepseek_coder",
            provider: "deepseek",
            model: "deepseek-coder",
            api_key: Some("sk-deepseek-coder-key"),
            custom_url: Some("https://custom.deepseek.com/v1/chat/completions"),
            should_validate: true,
            description: "Deepseek 代码模型，自定义 URL",
        },
        ProviderSwitchScenario {
            name: "siliconflow_7b",
            provider: "siliconflow",
            model: "qwen/Qwen2-7B-Instruct",
            api_key: Some("sk-siliconflow-7b-key"),
            custom_url: None, // 使用默认 URL
            should_validate: true,
            description: "SiliconFlow 7B 模型",
        },
        ProviderSwitchScenario {
            name: "siliconflow_72b",
            provider: "siliconflow",
            model: "qwen/Qwen2-72B-Instruct",
            api_key: Some("sk-siliconflow-72b-key"),
            custom_url: Some("https://custom.siliconflow.cn/v1/chat/completions"),
            should_validate: true,
            description: "SiliconFlow 72B 模型，自定义 URL",
        },
        ProviderSwitchScenario {
            name: "kimi_8k",
            provider: "kimi",
            model: "moonshot-v1-8k",
            api_key: Some("sk-kimi-8k-key"),
            custom_url: None,
            should_validate: true,
            description: "Kimi 8K 上下文模型",
        },
        ProviderSwitchScenario {
            name: "kimi_128k",
            provider: "kimi",
            model: "moonshot-v1-128k",
            api_key: Some("sk-kimi-128k-key"),
            custom_url: Some("https://custom.moonshot.cn/v1/chat/completions"),
            should_validate: true,
            description: "Kimi 128K 上下文模型，自定义 URL",
        },
        ProviderSwitchScenario {
            name: "deepseek_no_key",
            provider: "deepseek",
            model: "deepseek-chat",
            api_key: None, // 故意不提供 API Key
            custom_url: None,
            should_validate: false, // 应该验证失败
            description: "Deepseek 无 API Key（应该失败）",
        },
        ProviderSwitchScenario {
            name: "invalid_model",
            provider: "ollama",
            model: "nonexistent-model",
            api_key: None,
            custom_url: None,
            should_validate: false, // 应该验证失败
            description: "不存在的模型（应该失败）",
        },
    ]
}

#[test]
fn test_e2e_single_provider_switching() {
    println!("🧪 E2E 测试：单一提供商切换");

    let scenarios = get_provider_switch_scenarios();

    for scenario in &scenarios {
        clear_env_vars();

        println!("测试场景: {} - {}", scenario.name, scenario.description);

        // 设置环境变量
        env::set_var("AI_COMMIT_PROVIDER", scenario.provider);
        env::set_var("AI_COMMIT_MODEL", scenario.model);

        // 获取提供商信息以设置特定环境变量
        let provider_info = ProviderRegistry::get_provider(scenario.provider).unwrap();

        // 设置 API Key（如果提供）
        if let Some(api_key) = scenario.api_key {
            env::set_var(&provider_info.api_key_env_var(), api_key);
        }

        // 设置自定义 URL（如果提供）
        if let Some(custom_url) = scenario.custom_url {
            env::set_var(&provider_info.url_env_var(), custom_url);
        }

        // 创建和加载配置
        let mut config = Config::new();
        config.load_from_env();

        // 验证基础配置
        assert_eq!(
            config.provider, scenario.provider,
            "场景 {}: provider 不匹配",
            scenario.name
        );
        assert_eq!(
            config.model, scenario.model,
            "场景 {}: model 不匹配",
            scenario.name
        );

        // 验证当前提供商信息
        let current_provider = config.current_provider_info().unwrap();
        assert_eq!(
            current_provider.name, scenario.provider,
            "场景 {}: 当前提供商不匹配",
            scenario.name
        );

        // 验证 API Key
        if let Some(expected_key) = scenario.api_key {
            assert_eq!(
                config.current_api_key(),
                Some(expected_key.to_string()),
                "场景 {}: API Key 不匹配",
                scenario.name
            );
        } else if provider_info.requires_api_key {
            assert_eq!(
                config.current_api_key(),
                None,
                "场景 {}: 不应该有 API Key",
                scenario.name
            );
        }

        // 验证 URL
        let expected_url = scenario.custom_url.unwrap_or(&provider_info.default_url);
        assert_eq!(
            config.current_url(),
            expected_url,
            "场景 {}: URL 不匹配",
            scenario.name
        );

        // 验证配置有效性
        let validation_result = config.validate();
        if scenario.should_validate {
            assert!(
                validation_result.is_ok(),
                "场景 {}: 应该验证通过，但失败了: {:?}",
                scenario.name,
                validation_result
            );
        } else {
            assert!(
                validation_result.is_err(),
                "场景 {}: 应该验证失败，但通过了",
                scenario.name
            );
        }

        println!("✅ 场景 {} 验证通过", scenario.name);
    }

    clear_env_vars();
}

#[test]
fn test_e2e_rapid_provider_switching() {
    println!("🧪 E2E 测试：快速提供商切换");

    let valid_scenarios: Vec<_> = get_provider_switch_scenarios()
        .into_iter()
        .filter(|s| s.should_validate) // 只测试有效场景
        .collect();

    // 快速连续切换提供商
    for i in 0..3 {
        // 重复3次
        println!("快速切换轮次: {}", i + 1);

        for scenario in &valid_scenarios {
            clear_env_vars();

            // 快速设置环境变量
            env::set_var("AI_COMMIT_PROVIDER", scenario.provider);
            env::set_var("AI_COMMIT_MODEL", scenario.model);

            let provider_info = ProviderRegistry::get_provider(scenario.provider).unwrap();

            if let Some(api_key) = scenario.api_key {
                env::set_var(&provider_info.api_key_env_var(), api_key);
            }

            // 快速创建和验证配置
            let mut config = Config::new();
            config.load_from_env();

            assert_eq!(config.provider, scenario.provider);
            assert_eq!(config.model, scenario.model);

            let validation_result = config.validate();
            assert!(
                validation_result.is_ok(),
                "快速切换到 {} 应该成功: {:?}",
                scenario.provider,
                validation_result
            );
        }
    }

    println!("✅ 快速提供商切换验证通过");

    clear_env_vars();
}

#[test]
fn test_e2e_concurrent_provider_configurations() {
    println!("🧪 E2E 测试：并发提供商配置");

    clear_env_vars();

    // 同时设置所有提供商的环境变量
    let all_providers = ProviderRegistry::list_providers();
    let mut active_configs = HashMap::new();

    for provider_name in &all_providers {
        let provider_info = ProviderRegistry::get_provider(provider_name).unwrap();

        // 为每个提供商设置唯一的配置
        if provider_info.requires_api_key {
            let api_key = format!("sk-{}-concurrent-key", provider_name);
            env::set_var(&provider_info.api_key_env_var(), &api_key);
            active_configs.insert(provider_name.clone(), api_key);
        }

        let custom_url = format!("https://concurrent-{}.example.com/api", provider_name);
        env::set_var(&provider_info.url_env_var(), &custom_url);
    }

    // 测试每个提供商都能正确获取自己的配置
    for provider_name in &all_providers {
        env::set_var("AI_COMMIT_PROVIDER", provider_name);

        let provider_info = ProviderRegistry::get_provider(provider_name).unwrap();
        env::set_var("AI_COMMIT_MODEL", &provider_info.default_model);

        let mut config = Config::new();
        config.load_from_env();

        assert_eq!(config.provider, *provider_name);
        assert_eq!(config.model, provider_info.default_model);

        // 验证当前提供商获取正确的配置
        if let Some(expected_key) = active_configs.get(provider_name) {
            assert_eq!(config.current_api_key(), Some(expected_key.clone()));
        } else {
            assert_eq!(config.current_api_key(), None);
        }

        let expected_url = format!("https://concurrent-{}.example.com/api", provider_name);
        assert_eq!(config.current_url(), expected_url);

        // 验证配置有效性
        let validation_result = config.validate();
        assert!(
            validation_result.is_ok(),
            "并发配置中的提供商 {} 应该有效: {:?}",
            provider_name,
            validation_result
        );

        println!("✅ 并发配置中的提供商 {} 验证通过", provider_name);
    }

    clear_env_vars();
}

#[test]
fn test_e2e_provider_switching_with_model_validation() {
    println!("🧪 E2E 测试：带模型验证的提供商切换");

    // 为每个提供商测试其所有支持的模型
    let all_providers = ProviderRegistry::get_all();

    for (provider_name, provider_info) in all_providers.iter() {
        clear_env_vars();

        println!("测试提供商 {} 的所有模型", provider_name);

        // 设置基础配置
        env::set_var("AI_COMMIT_PROVIDER", provider_name);

        if provider_info.requires_api_key {
            env::set_var(
                &provider_info.api_key_env_var(),
                "test-key-for-model-validation",
            );
        }

        // 测试每个支持的模型
        for model in &provider_info.supported_models {
            env::set_var("AI_COMMIT_MODEL", model);

            let mut config = Config::new();
            config.load_from_env();

            assert_eq!(config.provider, *provider_name);
            assert_eq!(config.model, *model);

            // 验证模型配置有效
            let validation_result = config.validate();
            assert!(
                validation_result.is_ok(),
                "提供商 {} 的模型 {} 应该验证通过: {:?}",
                provider_name,
                model,
                validation_result
            );
        }

        println!(
            "✅ 提供商 {} 的 {} 个模型全部验证通过",
            provider_name,
            provider_info.supported_models.len()
        );
    }

    clear_env_vars();
}

#[test]
fn test_e2e_provider_api_format_consistency() {
    println!("🧪 E2E 测试：提供商 API 格式一致性");

    let all_providers = ProviderRegistry::get_all();
    let mut format_groups: HashMap<ApiFormat, Vec<String>> = HashMap::new();

    // 按 API 格式分组提供商
    for (name, provider) in all_providers.iter() {
        format_groups
            .entry(provider.api_format.clone())
            .or_default()
            .push(name.clone());
    }

    println!("API 格式分组: {:?}", format_groups);

    // 测试每个格式组内的提供商都能正常工作
    for (api_format, providers) in format_groups {
        println!("测试 {:?} 格式的提供商: {:?}", api_format, providers);

        for provider_name in &providers {
            clear_env_vars();

            let provider_info = ProviderRegistry::get_provider(provider_name).unwrap();

            env::set_var("AI_COMMIT_PROVIDER", provider_name);
            env::set_var("AI_COMMIT_MODEL", &provider_info.default_model);

            if provider_info.requires_api_key {
                env::set_var(&provider_info.api_key_env_var(), "test-format-key");
            }

            let mut config = Config::new();
            config.load_from_env();

            // 验证 API 格式一致性
            assert_eq!(
                config.current_provider_info().unwrap().api_format,
                api_format
            );

            // 验证配置有效性
            let validation_result = config.validate();
            assert!(
                validation_result.is_ok(),
                "{:?} 格式的提供商 {} 应该验证通过: {:?}",
                api_format,
                provider_name,
                validation_result
            );
        }

        println!("✅ {:?} 格式的提供商全部验证通过", api_format);
    }

    clear_env_vars();
}

#[test]
fn test_e2e_provider_switching_edge_cases() {
    println!("🧪 E2E 测试：提供商切换边界情况");

    // 边界情况测试
    let edge_cases = vec![
        ("empty_provider", "", "mistral", false, "空提供商名称"),
        (
            "nonexistent",
            "nonexistent_provider",
            "some-model",
            false,
            "不存在的提供商",
        ),
        (
            "case_sensitive",
            "OLLAMA",
            "mistral",
            false,
            "大写提供商名称",
        ),
        (
            "whitespace",
            " ollama ",
            "mistral",
            false,
            "包含空格的提供商名称",
        ),
    ];

    for (test_name, provider, model, should_validate, description) in edge_cases {
        clear_env_vars();

        println!("测试边界情况: {} - {}", test_name, description);

        if !provider.is_empty() {
            env::set_var("AI_COMMIT_PROVIDER", provider);
        }
        env::set_var("AI_COMMIT_MODEL", model);

        let mut config = Config::new();
        config.load_from_env();

        let validation_result = config.validate();

        if should_validate {
            assert!(
                validation_result.is_ok(),
                "边界情况 {} 应该验证通过: {:?}",
                test_name,
                validation_result
            );
        } else {
            assert!(
                validation_result.is_err(),
                "边界情况 {} 应该验证失败",
                test_name
            );
        }

        println!("✅ 边界情况 {} 验证通过", test_name);
    }

    clear_env_vars();
}

#[test]
fn test_e2e_provider_configuration_completeness_after_switching() {
    println!("🧪 E2E 测试：切换后提供商配置完整性");

    let all_providers = ProviderRegistry::list_providers();

    // 测试每次切换后配置的完整性
    for provider_name in &all_providers {
        clear_env_vars();

        println!("验证切换到 {} 后的配置完整性", provider_name);

        let provider_info = ProviderRegistry::get_provider(provider_name).unwrap();

        // 设置完整配置
        env::set_var("AI_COMMIT_PROVIDER", provider_name);
        env::set_var("AI_COMMIT_MODEL", &provider_info.default_model);
        env::set_var("AI_COMMIT_DEBUG", "true");

        if provider_info.requires_api_key {
            env::set_var(&provider_info.api_key_env_var(), "completeness-test-key");
        }

        let custom_url = format!("https://completeness-{}.test.com/api", provider_name);
        env::set_var(&provider_info.url_env_var(), &custom_url);

        let mut config = Config::new();
        config.load_from_env();

        // 验证所有配置项都正确设置
        assert_eq!(config.provider, *provider_name);
        assert_eq!(config.model, provider_info.default_model);
        assert!(config.debug);

        // 验证当前提供商方法的完整性
        let current_provider = config.current_provider_info().unwrap();
        assert_eq!(current_provider.name, *provider_name);
        assert_eq!(current_provider.api_format, provider_info.api_format);
        assert_eq!(
            current_provider.requires_api_key,
            provider_info.requires_api_key
        );

        assert_eq!(config.current_url(), custom_url);

        if provider_info.requires_api_key {
            assert_eq!(
                config.current_api_key(),
                Some("completeness-test-key".to_string())
            );
        } else {
            assert_eq!(config.current_api_key(), None);
        }

        // 验证向后兼容方法仍然工作
        match provider_name.as_str() {
            "ollama" => {
                assert_eq!(config.ollama_url(), custom_url);
                assert_eq!(config.ollama_api_key(), None);
            }
            "deepseek" => {
                assert_eq!(config.deepseek_url(), custom_url);
                if provider_info.requires_api_key {
                    assert_eq!(
                        config.deepseek_api_key(),
                        Some("completeness-test-key".to_string())
                    );
                }
            }
            "siliconflow" => {
                assert_eq!(config.siliconflow_url(), custom_url);
                if provider_info.requires_api_key {
                    assert_eq!(
                        config.siliconflow_api_key(),
                        Some("completeness-test-key".to_string())
                    );
                }
            }
            "kimi" => {
                assert_eq!(config.kimi_url(), custom_url);
                if provider_info.requires_api_key {
                    assert_eq!(
                        config.kimi_api_key(),
                        Some("completeness-test-key".to_string())
                    );
                }
            }
            _ => {
                // 新增的提供商，验证通用方法工作即可
            }
        }

        // 验证配置有效性
        let validation_result = config.validate();
        assert!(
            validation_result.is_ok(),
            "切换到 {} 后的完整配置应该有效: {:?}",
            provider_name,
            validation_result
        );

        println!("✅ 切换到 {} 后的配置完整性验证通过", provider_name);
    }

    clear_env_vars();
}
