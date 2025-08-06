use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use anyhow::{Result, anyhow};

use crate::ai::manager::{AIProvider, AIConfig, AIRequest, AIResponse, TokenUsage};

/// Ollama AI 提供商
pub struct OllamaProvider {
    client: Arc<reqwest::Client>,
    config: OllamaProviderConfig,
}

#[derive(Debug, Clone)]
struct OllamaProviderConfig {
    base_url: String,
    default_model: String,
}

/// Ollama API 请求结构
#[derive(Debug, Serialize)]
struct OllamaApiRequest {
    model: String,
    prompt: String,
    stream: bool,
    options: Option<OllamaOptions>,
}

/// Ollama 选项结构
#[derive(Debug, Serialize)]
struct OllamaOptions {
    temperature: Option<f32>,
    num_predict: Option<u32>, // Ollama 使用 num_predict 而不是 max_tokens
}

/// Ollama API 响应结构
#[derive(Debug, Deserialize)]
struct OllamaApiResponse {
    model: String,
    created_at: String,
    response: String,
    done: bool,
    context: Option<Vec<i32>>,
    total_duration: Option<u64>,
    load_duration: Option<u64>,
    prompt_eval_count: Option<u32>,
    prompt_eval_duration: Option<u64>,
    eval_count: Option<u32>,
    eval_duration: Option<u64>,
}

/// Ollama 模型列表响应
#[derive(Debug, Deserialize)]
struct OllamaModelsResponse {
    models: Vec<OllamaModel>,
}

/// Ollama 模型信息
#[derive(Debug, Deserialize)]
struct OllamaModel {
    name: String,
    modified_at: String,
    size: u64,
    digest: String,
    details: Option<OllamaModelDetails>,
}

/// Ollama 模型详情
#[derive(Debug, Deserialize)]
struct OllamaModelDetails {
    format: String,
    family: String,
    families: Option<Vec<String>>,
    parameter_size: String,
    quantization_level: String,
}

/// Ollama 错误响应
#[derive(Debug, Deserialize)]
struct OllamaErrorResponse {
    error: String,
}

impl OllamaProvider {
    /// 创建新的 Ollama 提供商
    pub fn new(client: Arc<reqwest::Client>, base_url: Option<String>, default_model: Option<String>) -> Self {
        let config = OllamaProviderConfig {
            base_url: base_url.unwrap_or_else(|| "http://localhost:11434".to_string()),
            default_model: default_model.unwrap_or_else(|| "qwen2.5-coder:7b".to_string()),
        };

        Self { client, config }
    }

    /// 发送非流式请求
    async fn send_request(&self, request: &OllamaApiRequest) -> Result<OllamaApiResponse> {
        let url = format!("{}/api/generate", self.config.base_url);

        let response = self.client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(request)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to send request to Ollama: {}", e))?;

        let status = response.status();
        let response_text = response.text().await
            .map_err(|e| anyhow!("Failed to read Ollama response: {}", e))?;

        if !status.is_success() {
            // 尝试解析错误响应
            if let Ok(error_response) = serde_json::from_str::<OllamaErrorResponse>(&response_text) {
                return Err(anyhow!("Ollama API error: {}", error_response.error));
            } else {
                return Err(anyhow!("Ollama API error {}: {}", status, response_text));
            }
        }

        let api_response: OllamaApiResponse = serde_json::from_str(&response_text)
            .map_err(|e| anyhow!("Failed to parse Ollama response: {}", e))?;

        Ok(api_response)
    }

    /// 发送流式请求
    async fn send_stream_request(&self, request: &OllamaApiRequest) -> Result<String> {
        use futures_util::StreamExt;

        let url = format!("{}/api/generate", self.config.base_url);

        let response = self.client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(request)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to send stream request to Ollama: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!("Ollama API error {}: {}", status, error_text));
        }

        let mut stream = response.bytes_stream();
        let mut content = String::new();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| anyhow!("Stream error: {}", e))?;
            let chunk_str = String::from_utf8_lossy(&chunk);

            for line in chunk_str.lines() {
                if !line.trim().is_empty() {
                    if let Ok(stream_response) = serde_json::from_str::<OllamaApiResponse>(line) {
                        content.push_str(&stream_response.response);

                        if stream_response.done {
                            return Ok(content);
                        }
                    }
                }
            }
        }

        Ok(content)
    }

    /// 构建 API 请求
    fn build_request(&self, ai_request: &AIRequest) -> OllamaApiRequest {
        let model = ai_request.model.as_ref()
            .unwrap_or(&self.config.default_model)
            .clone();

        let options = if ai_request.temperature.is_some() || ai_request.max_tokens.is_some() {
            Some(OllamaOptions {
                temperature: ai_request.temperature,
                num_predict: ai_request.max_tokens,
            })
        } else {
            None
        };

        OllamaApiRequest {
            model,
            prompt: ai_request.prompt.clone(),
            stream: false, // 默认非流式
            options,
        }
    }

    /// 检查 Ollama 服务是否可用
    async fn check_service_availability(&self) -> Result<bool> {
        let url = format!("{}/api/tags", self.config.base_url);

        match self.client.get(&url).send().await {
            Ok(response) => Ok(response.status().is_success()),
            Err(_) => Ok(false),
        }
    }

    /// 获取可用模型列表
    pub async fn list_models(&self) -> Result<Vec<String>> {
        let url = format!("{}/api/tags", self.config.base_url);

        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to get models from Ollama: {}", e))?;

        if !response.status().is_success() {
            return Err(anyhow!("Failed to get models: {}", response.status()));
        }

        let models_response: OllamaModelsResponse = response.json().await
            .map_err(|e| anyhow!("Failed to parse models response: {}", e))?;

        Ok(models_response.models.into_iter().map(|m| m.name).collect())
    }

    /// 检查指定模型是否可用
    pub async fn is_model_available(&self, model_name: &str) -> Result<bool> {
        let models = self.list_models().await?;
        Ok(models.iter().any(|m| m == model_name || m.starts_with(&format!("{}:", model_name))))
    }

    /// 拉取模型（如果不存在）
    pub async fn pull_model(&self, model_name: &str) -> Result<()> {
        let url = format!("{}/api/pull", self.config.base_url);

        let request = serde_json::json!({
            "name": model_name
        });

        let response = self.client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to pull model from Ollama: {}", e))?;

        if !response.status().is_success() {
            return Err(anyhow!("Failed to pull model: {}", response.status()));
        }

        Ok(())
    }
}

#[async_trait]
impl AIProvider for OllamaProvider {
    fn name(&self) -> &str {
        "ollama"
    }

    async fn analyze_code(&self, request: &AIRequest) -> Result<AIResponse> {
        let api_request = self.build_request(request);
        let start_time = std::time::Instant::now();

        let api_response = self.send_request(&api_request).await?;

        // 计算 token 使用情况（基于 Ollama 的统计信息）
        let token_usage = if api_response.prompt_eval_count.is_some() || api_response.eval_count.is_some() {
            Some(TokenUsage {
                prompt_tokens: api_response.prompt_eval_count.unwrap_or(0),
                completion_tokens: api_response.eval_count.unwrap_or(0),
                total_tokens: api_response.prompt_eval_count.unwrap_or(0) + api_response.eval_count.unwrap_or(0),
            })
        } else {
            None
        };

        let mut metadata = HashMap::new();
        metadata.insert("created_at".to_string(), api_response.created_at);
        if let Some(total_duration) = api_response.total_duration {
            metadata.insert("total_duration_ns".to_string(), total_duration.to_string());
        }
        if let Some(load_duration) = api_response.load_duration {
            metadata.insert("load_duration_ns".to_string(), load_duration.to_string());
        }
        if let Some(eval_duration) = api_response.eval_duration {
            metadata.insert("eval_duration_ns".to_string(), eval_duration.to_string());
        }

        Ok(AIResponse {
            content: api_response.response,
            model: api_response.model,
            provider: self.name().to_string(),
            response_time_ms: start_time.elapsed().as_millis() as u64,
            token_usage,
            metadata,
        })
    }

    fn is_available(&self) -> bool {
        // 对于 Ollama，我们需要异步检查服务可用性
        // 这里返回基本的配置检查，实际可用性需要通过 check_service_availability 检查
        !self.config.base_url.is_empty() && !self.config.default_model.is_empty()
    }

    fn supported_models(&self) -> Vec<String> {
        // 返回常见的 Ollama 模型，实际可用模型需要通过 list_models 获取
        vec![
            "qwen2.5-coder:7b".to_string(),
            "qwen2.5-coder:14b".to_string(),
            "qwen2.5-coder:32b".to_string(),
            "deepseek-coder:6.7b".to_string(),
            "deepseek-coder:33b".to_string(),
            "codellama:7b".to_string(),
            "codellama:13b".to_string(),
            "codellama:34b".to_string(),
            "llama3.1:8b".to_string(),
            "llama3.1:70b".to_string(),
            "mistral:7b".to_string(),
            "gemma2:9b".to_string(),
            "phi3:3.8b".to_string(),
        ]
    }

    fn validate_config(&self, config: &AIConfig) -> Result<()> {
        if let Some(ollama_config) = &config.ollama {
            if ollama_config.base_url.is_empty() {
                return Err(anyhow!("Ollama base URL is required"));
            }
            if ollama_config.model.is_empty() {
                return Err(anyhow!("Ollama model is required"));
            }

            // 验证 URL 格式
            if let Err(_) = url::Url::parse(&ollama_config.base_url) {
                return Err(anyhow!("Invalid Ollama base URL format"));
            }
        } else {
            return Err(anyhow!("Ollama configuration is missing"));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    fn create_test_provider() -> OllamaProvider {
        let client = Arc::new(reqwest::Client::new());
        OllamaProvider::new(client, None, None)
    }

    #[test]
    fn test_provider_creation() {
        let provider = create_test_provider();
        assert_eq!(provider.name(), "ollama");
        assert!(provider.is_available());
        assert_eq!(provider.config.base_url, "http://localhost:11434");
        assert_eq!(provider.config.default_model, "qwen2.5-coder:7b");
    }

    #[test]
    fn test_provider_creation_with_custom_config() {
        let client = Arc::new(reqwest::Client::new());
        let provider = OllamaProvider::new(
            client,
            Some("http://custom-ollama:11434".to_string()),
            Some("custom-model:7b".to_string())
        );
        assert_eq!(provider.config.base_url, "http://custom-ollama:11434");
        assert_eq!(provider.config.default_model, "custom-model:7b");
    }

    #[test]
    fn test_supported_models() {
        let provider = create_test_provider();
        let models = provider.supported_models();
        assert!(models.contains(&"qwen2.5-coder:7b".to_string()));
        assert!(models.contains(&"deepseek-coder:6.7b".to_string()));
        assert!(models.contains(&"codellama:7b".to_string()));
        assert!(models.contains(&"llama3.1:8b".to_string()));
        assert!(models.len() >= 10);
    }

    #[test]
    fn test_build_request() {
        let provider = create_test_provider();
        let ai_request = AIRequest::code_review("test.rs", "rust", "Review this Rust code");

        let api_request = provider.build_request(&ai_request);
        assert_eq!(api_request.model, "qwen2.5-coder:7b");
        assert_eq!(api_request.prompt, "Review this Rust code");
        assert!(!api_request.stream);
        assert!(api_request.options.is_none());
    }

    #[test]
    fn test_build_request_with_options() {
        let provider = create_test_provider();
        let mut ai_request = AIRequest::code_review("test.py", "python", "Review this Python code");
        ai_request.model = Some("deepseek-coder:6.7b".to_string());
        ai_request.temperature = Some(0.8);
        ai_request.max_tokens = Some(1000);

        let api_request = provider.build_request(&ai_request);
        assert_eq!(api_request.model, "deepseek-coder:6.7b");
        assert_eq!(api_request.prompt, "Review this Python code");
        assert!(api_request.options.is_some());

        let options = api_request.options.unwrap();
        assert_eq!(options.temperature, Some(0.8));
        assert_eq!(options.num_predict, Some(1000));
    }

    #[test]
    fn test_config_validation() {
        let provider = create_test_provider();

        // Valid config
        let valid_config = AIConfig {
            ollama: Some(crate::ai::manager::OllamaConfig {
                base_url: "http://localhost:11434".to_string(),
                model: "qwen2.5-coder:7b".to_string(),
            }),
            ..Default::default()
        };
        assert!(provider.validate_config(&valid_config).is_ok());

        // Missing config
        let missing_config = AIConfig::default();
        // Default config includes ollama config, so this should pass
        assert!(provider.validate_config(&missing_config).is_ok());

        // Empty base URL
        let empty_url_config = AIConfig {
            ollama: Some(crate::ai::manager::OllamaConfig {
                base_url: "".to_string(),
                model: "qwen2.5-coder:7b".to_string(),
            }),
            ..Default::default()
        };
        assert!(provider.validate_config(&empty_url_config).is_err());

        // Empty model
        let empty_model_config = AIConfig {
            ollama: Some(crate::ai::manager::OllamaConfig {
                base_url: "http://localhost:11434".to_string(),
                model: "".to_string(),
            }),
            ..Default::default()
        };
        assert!(provider.validate_config(&empty_model_config).is_err());

        // Invalid URL format
        let invalid_url_config = AIConfig {
            ollama: Some(crate::ai::manager::OllamaConfig {
                base_url: "not-a-valid-url".to_string(),
                model: "qwen2.5-coder:7b".to_string(),
            }),
            ..Default::default()
        };
        assert!(provider.validate_config(&invalid_url_config).is_err());
    }

    #[test]
    fn test_ollama_api_request_serialization() {
        let request = OllamaApiRequest {
            model: "qwen2.5-coder:7b".to_string(),
            prompt: "Review this code".to_string(),
            stream: false,
            options: Some(OllamaOptions {
                temperature: Some(0.7),
                num_predict: Some(500),
            }),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("qwen2.5-coder:7b"));
        assert!(json.contains("Review this code"));
        assert!(json.contains("false"));
        assert!(json.contains("0.7"));
        assert!(json.contains("500"));
    }

    #[test]
    fn test_ollama_api_response_deserialization() {
        let json = r#"{
            "model": "qwen2.5-coder:7b",
            "created_at": "2024-01-01T12:00:00Z",
            "response": "This code looks good!",
            "done": true,
            "total_duration": 1000000000,
            "load_duration": 100000000,
            "prompt_eval_count": 50,
            "prompt_eval_duration": 200000000,
            "eval_count": 25,
            "eval_duration": 700000000
        }"#;

        let response: OllamaApiResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.model, "qwen2.5-coder:7b");
        assert_eq!(response.response, "This code looks good!");
        assert!(response.done);
        assert_eq!(response.total_duration, Some(1000000000));
        assert_eq!(response.prompt_eval_count, Some(50));
        assert_eq!(response.eval_count, Some(25));
    }

    #[test]
    fn test_ollama_models_response_deserialization() {
        let json = r#"{
            "models": [
                {
                    "name": "qwen2.5-coder:7b",
                    "modified_at": "2024-01-01T12:00:00Z",
                    "size": 4661224676,
                    "digest": "sha256:abc123",
                    "details": {
                        "format": "gguf",
                        "family": "qwen2",
                        "parameter_size": "7.6B",
                        "quantization_level": "Q4_0"
                    }
                }
            ]
        }"#;

        let response: OllamaModelsResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.models.len(), 1);
        assert_eq!(response.models[0].name, "qwen2.5-coder:7b");
        assert_eq!(response.models[0].size, 4661224676);
        assert!(response.models[0].details.is_some());

        let details = response.models[0].details.as_ref().unwrap();
        assert_eq!(details.format, "gguf");
        assert_eq!(details.family, "qwen2");
        assert_eq!(details.parameter_size, "7.6B");
    }

    #[test]
    fn test_ollama_error_response_deserialization() {
        let json = r#"{
            "error": "model not found"
        }"#;

        let error_response: OllamaErrorResponse = serde_json::from_str(json).unwrap();
        assert_eq!(error_response.error, "model not found");
    }

    #[test]
    fn test_ollama_options_serialization() {
        let options = OllamaOptions {
            temperature: Some(0.5),
            num_predict: Some(200),
        };

        let json = serde_json::to_string(&options).unwrap();
        assert!(json.contains("0.5"));
        assert!(json.contains("200"));
    }

    #[test]
    fn test_ollama_options_none_values() {
        let options = OllamaOptions {
            temperature: None,
            num_predict: None,
        };

        let json = serde_json::to_string(&options).unwrap();
        assert!(json.contains("null") || !json.contains("temperature"));
    }
}