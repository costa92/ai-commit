/// E2E 测试：Ollama 集成测试
/// 测试与真实 Ollama 服务的完整集成

use std::env;
use std::process::Command;
use std::time::Duration;

use reqwest::Client;
use serde_json::{json, Value};
use tokio::time::timeout;

use ai_commit::config::Config;
use ai_commit::config::providers::ProviderRegistry;

/// 测试辅助函数：清理环境变量
fn clear_env_vars() {
    let vars = [
        "AI_COMMIT_PROVIDER",
        "AI_COMMIT_MODEL", 
        "AI_COMMIT_DEBUG",
        "AI_COMMIT_OLLAMA_URL",
    ];
    
    for var in &vars {
        env::remove_var(var);
    }
}

/// 检查 Ollama 服务是否运行
async fn check_ollama_service() -> bool {
    let client = Client::new();
    
    match timeout(Duration::from_secs(5), client.get("http://localhost:11434/api/tags").send()).await {
        Ok(Ok(response)) => response.status().is_success(),
        _ => false,
    }
}

/// 获取可用的 Ollama 模型
async fn get_ollama_models() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let client = Client::new();
    
    let response = client
        .get("http://localhost:11434/api/tags")
        .send()
        .await?;
    
    let json: Value = response.json().await?;
    let models = json["models"]
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|model| {
            model["name"].as_str().map(|s| {
                // 提取模型名称（去掉 :latest 后缀）
                s.split(':').next().unwrap_or(s).to_string()
            })
        })
        .collect();
    
    Ok(models)
}

/// 测试与 Ollama 的基本连接
async fn test_ollama_connection(model: &str) -> Result<String, Box<dyn std::error::Error>> {
    let client = Client::new();
    
    let request_body = json!({
        "model": model,
        "prompt": "fix: 测试连接",
        "stream": false
    });
    
    let response = client
        .post("http://localhost:11434/api/generate")
        .json(&request_body)
        .send()
        .await?;
    
    if !response.status().is_success() {
        return Err(format!("Ollama API 错误: {}", response.status()).into());
    }
    
    let json: Value = response.json().await?;
    let generated_text = json["response"]
        .as_str()
        .unwrap_or("")
        .to_string();
    
    Ok(generated_text)
}

#[tokio::test]
async fn test_e2e_ollama_service_availability() {
    println!("🧪 E2E 测试：Ollama 服务可用性");
    
    let is_available = check_ollama_service().await;
    
    if !is_available {
        println!("⚠️  Ollama 服务未运行，跳过集成测试");
        println!("   启动 Ollama: ollama serve");
        return;
    }
    
    println!("✅ Ollama 服务运行正常");
    
    // 获取可用模型
    match get_ollama_models().await {
        Ok(models) => {
            println!("✅ 可用模型: {:?}", models);
            assert!(!models.is_empty(), "应该至少有一个模型可用");
        }
        Err(e) => {
            panic!("获取 Ollama 模型列表失败: {}", e);
        }
    }
}

#[tokio::test]
async fn test_e2e_ollama_config_integration() {
    println!("🧪 E2E 测试：Ollama 配置集成");
    
    if !check_ollama_service().await {
        println!("⚠️  Ollama 服务未运行，跳过集成测试");
        return;
    }
    
    clear_env_vars();
    
    // 设置 Ollama 配置
    env::set_var("AI_COMMIT_PROVIDER", "ollama");
    env::set_var("AI_COMMIT_MODEL", "mistral");
    env::set_var("AI_COMMIT_DEBUG", "true");
    
    // 创建并验证配置
    let mut config = Config::new();
    config.load_from_env();
    
    assert_eq!(config.provider, "ollama");
    assert_eq!(config.model, "mistral");
    assert!(config.debug);
    
    // 验证 Ollama 提供商信息
    let provider_info = config.current_provider_info().unwrap();
    assert_eq!(provider_info.name, "ollama");
    assert!(!provider_info.requires_api_key, "Ollama 不需要 API Key");
    
    // 验证配置有效性
    let validation_result = config.validate();
    assert!(validation_result.is_ok(), "Ollama 配置应该验证通过: {:?}", validation_result);
    
    // 验证 URL 配置
    assert_eq!(config.ollama_url(), "http://localhost:11434/api/generate");
    assert_eq!(config.current_url(), "http://localhost:11434/api/generate");
    assert_eq!(config.current_api_key(), None);
    
    println!("✅ Ollama 配置集成验证通过");
    
    clear_env_vars();
}

#[tokio::test]
async fn test_e2e_ollama_api_call() {
    println!("🧪 E2E 测试：Ollama API 调用");
    
    if !check_ollama_service().await {
        println!("⚠️  Ollama 服务未运行，跳过集成测试");
        return;
    }
    
    // 获取可用模型
    let available_models = match get_ollama_models().await {
        Ok(models) => models,
        Err(e) => {
            println!("⚠️  无法获取 Ollama 模型: {}, 跳过 API 测试", e);
            return;
        }
    };
    
    if available_models.is_empty() {
        println!("⚠️  没有可用的 Ollama 模型，跳过 API 测试");
        println!("   安装模型: ollama pull mistral");
        return;
    }
    
    // 使用第一个可用模型进行测试
    let test_model = &available_models[0];
    println!("使用模型进行测试: {}", test_model);
    
    // 测试 API 调用
    match test_ollama_connection(test_model).await {
        Ok(response) => {
            println!("✅ Ollama API 调用成功");
            println!("响应内容: {}", response.chars().take(100).collect::<String>());
            assert!(!response.is_empty(), "响应不应该为空");
        }
        Err(e) => {
            panic!("Ollama API 调用失败: {}", e);
        }
    }
}

#[tokio::test]
async fn test_e2e_ollama_custom_url() {
    println!("🧪 E2E 测试：Ollama 自定义 URL");
    
    if !check_ollama_service().await {
        println!("⚠️  Ollama 服务未运行，跳过集成测试");
        return;
    }
    
    clear_env_vars();
    
    // 设置自定义 URL
    let custom_url = "http://localhost:11434/api/generate";
    env::set_var("AI_COMMIT_PROVIDER", "ollama");
    env::set_var("AI_COMMIT_OLLAMA_URL", custom_url);
    
    // 创建配置
    let mut config = Config::new();
    config.load_from_env();
    
    // 验证自定义 URL 生效
    assert_eq!(config.ollama_url(), custom_url);
    assert_eq!(config.current_url(), custom_url);
    
    println!("✅ Ollama 自定义 URL 配置验证通过");
    
    clear_env_vars();
}

#[tokio::test]
async fn test_e2e_ollama_model_validation() {
    println!("🧪 E2E 测试：Ollama 模型验证");
    
    clear_env_vars();
    
    let mut config = Config::new();
    config.provider = "ollama".to_string();
    
    // 测试支持的模型
    let supported_models = ["mistral", "llama3", "qwen2", "codellama", "gemma", "phi3"];
    
    for model in &supported_models {
        config.model = model.to_string();
        
        let validation_result = config.validate();
        assert!(validation_result.is_ok(), 
               "支持的模型 {} 应该验证通过: {:?}", model, validation_result);
    }
    
    println!("✅ 所有支持的模型验证通过");
    
    // 测试不支持的模型
    config.model = "unsupported-model".to_string();
    let validation_result = config.validate();
    assert!(validation_result.is_err(), "不支持的模型应该验证失败");
    
    println!("✅ 不支持的模型正确验证失败");
    
    clear_env_vars();
}

#[tokio::test]
async fn test_e2e_ollama_multiple_models() {
    println!("🧪 E2E 测试：Ollama 多模型切换");
    
    if !check_ollama_service().await {
        println!("⚠️  Ollama 服务未运行，跳过集成测试");
        return;
    }
    
    let available_models = match get_ollama_models().await {
        Ok(models) => models,
        Err(e) => {
            println!("⚠️  无法获取 Ollama 模型: {}, 跳过多模型测试", e);
            return;
        }
    };
    
    println!("可用模型: {:?}", available_models);
    
    // 如果有多个模型可用，测试切换
    for model in available_models.iter().take(3) { // 最多测试3个模型
        clear_env_vars();
        
        env::set_var("AI_COMMIT_PROVIDER", "ollama");
        env::set_var("AI_COMMIT_MODEL", model);
        
        let mut config = Config::new();
        config.load_from_env();
        
        assert_eq!(config.model, *model);
        
        // 如果是支持的模型，验证应该通过
        let ollama_info = ProviderRegistry::get_provider("ollama").unwrap();
        if ollama_info.supported_models.iter().any(|m| model.contains(m)) {
            let validation_result = config.validate();
            assert!(validation_result.is_ok(), 
                   "模型 {} 应该验证通过: {:?}", model, validation_result);
        }
        
        println!("✅ 模型 {} 配置测试通过", model);
    }
    
    clear_env_vars();
}

#[test]
fn test_e2e_ollama_provider_info_completeness() {
    println!("🧪 E2E 测试：Ollama 提供商信息完整性");
    
    let ollama = ProviderRegistry::get_provider("ollama")
        .expect("应该能找到 ollama 提供商");
    
    // 验证基本信息
    assert_eq!(ollama.name, "ollama");
    assert_eq!(ollama.display_name, "Ollama");
    assert!(!ollama.requires_api_key, "Ollama 不需要 API Key");
    assert_eq!(ollama.default_url, "http://localhost:11434/api/generate");
    assert_eq!(ollama.api_format, ai_commit::config::providers::ApiFormat::Ollama);
    
    // 验证模型配置
    assert!(!ollama.default_model.is_empty(), "应该有默认模型");
    assert!(!ollama.supported_models.is_empty(), "应该有支持的模型列表");
    assert!(ollama.supported_models.contains(&ollama.default_model),
           "默认模型应该在支持的模型列表中");
    
    // 验证环境变量配置
    assert_eq!(ollama.env_prefix, "AI_COMMIT_OLLAMA");
    assert_eq!(ollama.api_key_env_var(), "AI_COMMIT_OLLAMA_API_KEY");
    assert_eq!(ollama.url_env_var(), "AI_COMMIT_OLLAMA_URL");
    
    // 验证验证逻辑
    let validation_result = ollama.validate(None);
    assert!(validation_result.is_ok(), "Ollama 不需要 API Key，验证应该通过");
    
    println!("✅ Ollama 提供商信息完整性验证通过");
}

#[tokio::test]
async fn test_e2e_ollama_error_handling() {
    println!("🧪 E2E 测试：Ollama 错误处理");
    
    // 测试无效 URL
    let client = Client::new();
    let result = client
        .post("http://localhost:99999/api/generate") // 无效端口
        .json(&json!({
            "model": "mistral",
            "prompt": "test",
            "stream": false
        }))
        .timeout(Duration::from_secs(2))
        .send()
        .await;
    
    assert!(result.is_err(), "连接无效 URL 应该失败");
    println!("✅ 无效 URL 错误处理正确");
    
    // 如果 Ollama 服务运行，测试无效模型
    if check_ollama_service().await {
        let result = client
            .post("http://localhost:11434/api/generate")
            .json(&json!({
                "model": "nonexistent-model-12345",
                "prompt": "test",
                "stream": false
            }))
            .timeout(Duration::from_secs(10))
            .send()
            .await;
        
        match result {
            Ok(response) => {
                // 某些情况下 Ollama 可能返回错误状态而不是连接错误
                if !response.status().is_success() {
                    println!("✅ 无效模型返回错误状态: {}", response.status());
                } else {
                    println!("ℹ️  Ollama 对无效模型的处理可能因版本而异");
                }
            }
            Err(_) => {
                println!("✅ 无效模型请求失败（预期行为）");
            }
        }
    }
}

#[test]
fn test_e2e_ollama_config_backwards_compatibility() {
    println!("🧪 E2E 测试：Ollama 配置向后兼容性");
    
    clear_env_vars();
    
    // 创建配置，测试向后兼容的方法调用
    let mut config = Config::new();
    config.provider = "ollama".to_string();
    config.load_from_env();
    
    // 测试所有向后兼容的方法
    assert!(!config.ollama_url().is_empty(), "ollama_url() 应该返回有效值");
    assert!(config.deepseek_api_key().is_none(), "deepseek_api_key() 应该返回 None");
    assert!(!config.deepseek_url().is_empty(), "deepseek_url() 应该返回有效值");
    
    // 测试通用方法
    assert_eq!(config.current_url(), config.ollama_url());
    assert_eq!(config.current_api_key(), None);
    
    let provider_info = config.current_provider_info();
    assert!(provider_info.is_some());
    assert_eq!(provider_info.unwrap().name, "ollama");
    
    println!("✅ Ollama 向后兼容性验证通过");
    
    clear_env_vars();
}