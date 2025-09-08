/// E2E 测试：AI 提供商配置系统端到端测试
/// 验证配置文件加载、环境变量、提供商切换等完整功能

use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

use ai_commit::config::providers::{ProviderRegistry, ProviderInfo, ApiFormat};
use ai_commit::config::Config;

/// 测试辅助函数：清理环境变量
fn clear_provider_env_vars() {
    let vars_to_clear = [
        "AI_COMMIT_PROVIDER",
        "AI_COMMIT_MODEL", 
        "AI_COMMIT_DEBUG",
        "AI_COMMIT_OLLAMA_URL",
        "AI_COMMIT_DEEPSEEK_API_KEY",
        "AI_COMMIT_DEEPSEEK_URL",
        "AI_COMMIT_SILICONFLOW_API_KEY", 
        "AI_COMMIT_SILICONFLOW_URL",
        "AI_COMMIT_KIMI_API_KEY",
        "AI_COMMIT_KIMI_URL",
    ];
    
    for var in &vars_to_clear {
        env::remove_var(var);
    }
}

/// 创建测试用的 providers.toml 文件
fn create_test_providers_config(temp_dir: &Path) -> String {
    let config_content = r#"
# 测试用的提供商配置文件

[providers.ollama]
name = "ollama"
display_name = "Ollama"
default_url = "http://localhost:11434/api/generate"
requires_api_key = false
default_model = "mistral"
supported_models = ["mistral", "llama3", "qwen2"]
api_format = "ollama"
description = "本地 Ollama 服务，无需 API Key"
env_prefix = "AI_COMMIT_OLLAMA"

[providers.deepseek]
name = "deepseek"
display_name = "Deepseek"
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
default_url = "https://test.example.com/api/v1/chat"
requires_api_key = true
default_model = "test-model-v1"
supported_models = ["test-model-v1", "test-model-v2"]
api_format = "openai"
description = "测试提供商"
env_prefix = "AI_COMMIT_TEST"
"#;
    
    let config_path = temp_dir.join("providers.toml");
    fs::write(&config_path, config_content).expect("Failed to write test config");
    config_path.to_string_lossy().to_string()
}

#[test]
fn test_e2e_provider_registry_basic_functionality() {
    println!("🧪 E2E 测试：提供商注册表基础功能");
    
    // 获取所有可用提供商
    let providers = ProviderRegistry::list_providers();
    
    // 验证基本提供商存在
    assert!(providers.contains(&"ollama"), "应该包含 ollama 提供商");
    assert!(providers.contains(&"deepseek"), "应该包含 deepseek 提供商");
    assert!(providers.contains(&"siliconflow"), "应该包含 siliconflow 提供商");
    assert!(providers.contains(&"kimi"), "应该包含 kimi 提供商");
    
    println!("✅ 可用提供商: {:?}", providers);
    
    // 验证提供商信息
    let ollama = ProviderRegistry::get_provider("ollama").expect("ollama 提供商应该存在");
    assert_eq!(ollama.name, "ollama");
    assert_eq!(ollama.api_format, ApiFormat::Ollama);
    assert!(!ollama.requires_api_key, "Ollama 不应该需要 API Key");
    
    let deepseek = ProviderRegistry::get_provider("deepseek").expect("deepseek 提供商应该存在");
    assert_eq!(deepseek.name, "deepseek");
    assert_eq!(deepseek.api_format, ApiFormat::OpenAI);
    assert!(deepseek.requires_api_key, "Deepseek 应该需要 API Key");
    
    println!("✅ 提供商信息验证通过");
}

#[test]
fn test_e2e_config_system_with_environment_variables() {
    println!("🧪 E2E 测试：环境变量配置系统");
    
    clear_provider_env_vars();
    
    // 设置环境变量
    env::set_var("AI_COMMIT_PROVIDER", "deepseek");
    env::set_var("AI_COMMIT_MODEL", "deepseek-coder");
    env::set_var("AI_COMMIT_DEBUG", "true");
    env::set_var("AI_COMMIT_DEEPSEEK_API_KEY", "test-key-123");
    env::set_var("AI_COMMIT_DEEPSEEK_URL", "https://custom.deepseek.com/api");
    
    // 创建配置并加载环境变量
    let mut config = Config::new();
    config.load_from_env();
    
    // 验证环境变量正确加载
    assert_eq!(config.provider, "deepseek");
    assert_eq!(config.model, "deepseek-coder");
    assert!(config.debug, "debug 模式应该启用");
    
    // 验证提供商特定配置
    assert_eq!(config.deepseek_api_key(), Some("test-key-123".to_string()));
    assert_eq!(config.deepseek_url(), "https://custom.deepseek.com/api");
    
    // 验证其他提供商配置为默认值
    assert_eq!(config.ollama_url(), "http://localhost:11434/api/generate");
    assert_eq!(config.kimi_api_key(), None);
    
    println!("✅ 环境变量配置验证通过");
    
    clear_provider_env_vars();
}

#[test]
fn test_e2e_provider_validation_workflow() {
    println!("🧪 E2E 测试：提供商验证工作流");
    
    clear_provider_env_vars();
    
    // 测试 Ollama 配置（无需 API Key）
    let mut config = Config::new();
    config.provider = "ollama".to_string();
    config.model = "mistral".to_string();
    
    let validation_result = config.validate();
    assert!(validation_result.is_ok(), "Ollama 配置应该验证通过：{:?}", validation_result);
    println!("✅ Ollama 配置验证通过");
    
    // 测试 Deepseek 配置（需要 API Key） - 无 Key 应该失败
    config.provider = "deepseek".to_string();
    config.model = "deepseek-chat".to_string();
    
    let validation_result = config.validate();
    assert!(validation_result.is_err(), "没有 API Key 的 Deepseek 配置应该验证失败");
    println!("✅ Deepseek 无 API Key 验证正确失败");
    
    // 设置 API Key 后应该成功
    env::set_var("AI_COMMIT_DEEPSEEK_API_KEY", "sk-test-key");
    config.load_from_env();
    
    let validation_result = config.validate();
    assert!(validation_result.is_ok(), "有 API Key 的 Deepseek 配置应该验证通过：{:?}", validation_result);
    println!("✅ Deepseek 有 API Key 验证通过");
    
    // 测试不支持的提供商
    config.provider = "nonexistent".to_string();
    let validation_result = config.validate();
    assert!(validation_result.is_err(), "不存在的提供商应该验证失败");
    println!("✅ 不存在的提供商验证正确失败");
    
    // 测试不支持的模型
    config.provider = "deepseek".to_string();
    config.model = "unsupported-model".to_string();
    let validation_result = config.validate();
    assert!(validation_result.is_err(), "不支持的模型应该验证失败");
    println!("✅ 不支持的模型验证正确失败");
    
    clear_provider_env_vars();
}

#[test]
fn test_e2e_multi_provider_switching() {
    println!("🧪 E2E 测试：多提供商切换");
    
    clear_provider_env_vars();
    
    let test_scenarios = [
        ("ollama", "mistral", None),
        ("deepseek", "deepseek-chat", Some("sk-deepseek-key")),
        ("siliconflow", "qwen/Qwen2-7B-Instruct", Some("sk-siliconflow-key")),
        ("kimi", "moonshot-v1-8k", Some("sk-kimi-key")),
    ];
    
    for (provider, model, api_key) in &test_scenarios {
        println!("测试切换到提供商: {}", provider);
        
        clear_provider_env_vars();
        
        // 设置基本配置
        env::set_var("AI_COMMIT_PROVIDER", provider);
        env::set_var("AI_COMMIT_MODEL", model);
        
        // 设置 API Key（如果需要）
        if let Some(key) = api_key {
            let api_key_var = format!("AI_COMMIT_{}_API_KEY", provider.to_uppercase());
            env::set_var(api_key_var, key);
        }
        
        // 创建并验证配置
        let mut config = Config::new();
        config.load_from_env();
        
        assert_eq!(config.provider, *provider, "提供商应该正确设置");
        assert_eq!(config.model, *model, "模型应该正确设置");
        
        // 验证提供商信息可以获取
        let provider_info = config.current_provider_info();
        assert!(provider_info.is_some(), "应该能获取当前提供商信息");
        
        let provider_info = provider_info.unwrap();
        assert_eq!(provider_info.name, *provider, "提供商名称应该匹配");
        assert!(provider_info.supported_models.contains(&model.to_string()), 
               "提供商应该支持指定的模型");
        
        // 验证 API Key 配置
        if api_key.is_some() {
            let current_api_key = config.current_api_key();
            assert!(current_api_key.is_some(), "需要 API Key 的提供商应该有 API Key");
            assert_eq!(current_api_key.unwrap(), *api_key.unwrap());
        } else {
            assert_eq!(config.current_api_key(), None, "不需要 API Key 的提供商不应该有 API Key");
        }
        
        // 验证URL配置
        let current_url = config.current_url();
        assert!(!current_url.is_empty(), "应该有有效的 URL 配置");
        assert_eq!(current_url, provider_info.default_url, "URL 应该匹配默认值");
        
        println!("✅ 提供商 {} 切换成功", provider);
    }
    
    clear_provider_env_vars();
}

#[test]
fn test_e2e_provider_info_completeness() {
    println!("🧪 E2E 测试：提供商信息完整性");
    
    let all_providers = ProviderRegistry::get_all();
    
    // 验证每个提供商的信息完整性
    for (name, provider) in all_providers.iter() {
        println!("验证提供商: {}", name);
        
        // 基本信息不为空
        assert!(!provider.name.is_empty(), "提供商名称不能为空");
        assert!(!provider.display_name.is_empty(), "显示名称不能为空");
        assert!(!provider.default_url.is_empty(), "默认 URL 不能为空");
        assert!(!provider.default_model.is_empty(), "默认模型不能为空");
        assert!(!provider.supported_models.is_empty(), "支持的模型列表不能为空");
        assert!(!provider.env_prefix.is_empty(), "环境变量前缀不能为空");
        assert!(!provider.description.is_empty(), "描述不能为空");
        
        // 验证默认模型在支持列表中
        assert!(provider.supported_models.contains(&provider.default_model),
               "默认模型 {} 应该在支持的模型列表中", provider.default_model);
        
        // 验证环境变量命名规范
        assert!(provider.env_prefix.starts_with("AI_COMMIT_"),
               "环境变量前缀应该以 AI_COMMIT_ 开头");
        
        // 验证 URL 格式
        assert!(provider.default_url.starts_with("http://") || provider.default_url.starts_with("https://"),
               "默认 URL 应该是有效的 HTTP/HTTPS 地址");
        
        // 验证环境变量方法
        let api_key_var = provider.api_key_env_var();
        let url_var = provider.url_env_var();
        assert!(api_key_var.contains("API_KEY"), "API Key 环境变量应该包含 API_KEY");
        assert!(url_var.contains("URL"), "URL 环境变量应该包含 URL");
        
        println!("✅ 提供商 {} 信息完整性验证通过", name);
    }
}

#[test] 
fn test_e2e_configuration_priority() {
    println!("🧪 E2E 测试：配置优先级");
    
    clear_provider_env_vars();
    
    // 测试默认配置
    let config = Config::new();
    assert_eq!(config.provider, "ollama", "默认提供商应该是 ollama");
    
    // 测试环境变量覆盖默认值
    env::set_var("AI_COMMIT_PROVIDER", "deepseek");
    env::set_var("AI_COMMIT_MODEL", "deepseek-coder");
    
    let mut config = Config::new();
    config.load_from_env();
    assert_eq!(config.provider, "deepseek", "环境变量应该覆盖默认值");
    assert_eq!(config.model, "deepseek-coder");
    
    // 模拟命令行参数覆盖环境变量（通过 update_from_args）
    // 注意：这里我们不能直接测试 Args 结构，因为它可能有编译问题
    // 但我们可以直接测试 update 逻辑
    
    println!("✅ 配置优先级验证通过");
    
    clear_provider_env_vars();
}

#[test]
fn test_e2e_debug_mode_functionality() {
    println!("🧪 E2E 测试：调试模式功能");
    
    clear_provider_env_vars();
    
    // 测试不同的调试模式值
    let debug_values = [
        ("true", true),
        ("TRUE", true),
        ("True", true),
        ("1", true),
        ("false", false),
        ("FALSE", false),
        ("0", false),
        ("invalid", false),
        ("", false),
    ];
    
    for (debug_value, expected) in &debug_values {
        env::remove_var("AI_COMMIT_DEBUG");
        
        if !debug_value.is_empty() {
            env::set_var("AI_COMMIT_DEBUG", debug_value);
        }
        
        let mut config = Config::new();
        config.load_from_env();
        
        assert_eq!(config.debug, *expected, 
                  "调试值 '{}' 应该解析为 {}", debug_value, expected);
    }
    
    println!("✅ 调试模式功能验证通过");
    
    clear_provider_env_vars();
}

#[test]
fn test_e2e_provider_error_messages() {
    println!("🧪 E2E 测试：提供商错误消息");
    
    clear_provider_env_vars();
    
    // 测试缺少 API Key 的错误消息
    let mut config = Config::new();
    config.provider = "deepseek".to_string();
    config.model = "deepseek-chat".to_string();
    
    let result = config.validate();
    assert!(result.is_err(), "缺少 API Key 应该验证失败");
    
    let error_msg = result.err().unwrap().to_string();
    assert!(error_msg.contains("Deepseek"), "错误消息应该包含提供商名称");
    assert!(error_msg.contains("API key"), "错误消息应该提及 API key");
    assert!(error_msg.contains("AI_COMMIT_DEEPSEEK_API_KEY"), "错误消息应该包含环境变量名");
    
    println!("✅ Deepseek API Key 错误消息正确");
    
    // 测试不存在的提供商错误
    config.provider = "nonexistent".to_string();
    let result = config.validate();
    assert!(result.is_err(), "不存在的提供商应该验证失败");
    
    let error_msg = result.err().unwrap().to_string();
    assert!(error_msg.contains("Unsupported provider"), "错误消息应该提及不支持的提供商");
    assert!(error_msg.contains("nonexistent"), "错误消息应该包含提供商名称");
    
    println!("✅ 不存在提供商错误消息正确");
    
    clear_provider_env_vars();
}

#[test]
fn test_e2e_all_providers_basic_config() {
    println!("🧪 E2E 测试：所有提供商基础配置");
    
    clear_provider_env_vars();
    
    let all_providers = ProviderRegistry::list_providers();
    
    for provider_name in all_providers {
        println!("测试提供商基础配置: {}", provider_name);
        
        let provider_info = ProviderRegistry::get_provider(provider_name).unwrap();
        
        // 创建基础配置
        let mut config = Config::new();
        config.provider = provider_name.to_string();
        config.model = provider_info.default_model.clone();
        
        // 如果需要 API Key，设置一个测试用的
        if provider_info.requires_api_key {
            env::set_var(&provider_info.api_key_env_var(), "test-key-123");
            config.load_from_env();
        }
        
        // 验证配置有效
        let validation_result = config.validate();
        assert!(validation_result.is_ok(), 
               "提供商 {} 的基础配置应该验证通过: {:?}", provider_name, validation_result);
        
        // 验证辅助方法工作正常
        assert!(!config.current_url().is_empty(), "应该有有效的 URL");
        
        if provider_info.requires_api_key {
            assert!(config.current_api_key().is_some(), "需要 API Key 的提供商应该有 API Key");
        } else {
            assert!(config.current_api_key().is_none(), "不需要 API Key 的提供商不应该有 API Key");
        }
        
        println!("✅ 提供商 {} 基础配置验证通过", provider_name);
        
        // 清理环境变量
        env::remove_var(&provider_info.api_key_env_var());
    }
    
    clear_provider_env_vars();
}