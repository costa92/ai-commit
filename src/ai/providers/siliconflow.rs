use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use anyhow::{Result, anyhow};

use crate::ai::manager::{AIProvider, AIConfig, AIRequest, AIResponse, TokenUsage};

/// SiliconFlow AI 提供商
pub struct SiliconFlowProvider {
    client: Arc<reqwest::Client>,
    config: SiliconFlowProviderConfig,
}

#[derive(Debug, Clone)]
struct SiliconFlowProviderConfig {
    api_key: String,
    base_url: String,
    default_model: String,
}

/// SiliconFlow API 请求结构（与 OpenAI 兼容）
#[derive(Debug, Serialize)]
struct SiliconFlowApiRequest {
    model: String,
    messages: Vec<SiliconFlowMessage>,
    temperature: Option<f32>,
    max_tokens: Option<u32>,
    stream: bool,
}

/// SiliconFlow 消息结构
#[derive(Debug, Serialize)]
struct SiliconFlowMessage {
    role: String,
    content: String,
}

/// SiliconFlow API 响应结构
#[derive(Debug, Deserialize)]
struct SiliconFlowApiResponse {
    id: String,
    object: String,
    created: u64,
    model: String,
    choices: Vec<SiliconFlowChoice>,
    usage: Option<SiliconFlowUsage>,
}

/// SiliconFlow 选择结构
#[derive(Debug, Deserialize)]
struct SiliconFlowChoice {
    index: u32,
    message: SiliconFlowResponseMessage,
    finish_reason: Option<String>,
}

/// SiliconFlow 响应消息结构
#[derive(Debug, Deserialize)]
struct SiliconFlowResponseMessage {
    role: String,
    content: String,
}

/// SiliconFlow 使用情况结构
#[derive(Debug, Deserialize)]
struct SiliconFlowUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

/// SiliconFlow 流式响应结构
#[derive(Debug, Deserialize)]
struct SiliconFlowStreamResponse {
    id: String,
    object: String,
    created: u64,
    model: String,
    choices: Vec<SiliconFlowStreamChoice>,
}

/// SiliconFlow 流式选择结构
#[derive(Debug, Deserialize)]
struct SiliconFlowStreamChoice {
    index: u32,
    delta: SiliconFlowDelta,
    finish_reason: Option<String>,
}

/// SiliconFlow 增量内容结构
#[derive(Debug, Deserialize)]
struct SiliconFlowDelta {
    role: Option<String>,
    content: Option<String>,
}

/// SiliconFlow 错误响应结构
#[derive(Debug, Deserialize)]
struct SiliconFlowErrorResponse {
    error: SiliconFlowError,
}

/// SiliconFlow 错误结构
#[derive(Debug, Deserialize)]
struct SiliconFlowError {
    message: String,
    #[serde(rename = "type")]
    error_type: Option<String>,
    code: Option<String>,
}

impl SiliconFlowProvider {
    /// 创建新的 SiliconFlow 提供商
    pub fn new(client: Arc<reqwest::Client>, api_key: String, base_url: Option<String>) -> Self {
        let config = SiliconFlowProviderConfig {
            api_key,
            base_url: base_url.unwrap_or_else(|| "https://api.siliconflow.cn/v1".to_string()),
            default_model: "deepseek-ai/DeepSeek-V2.5".to_string(),
        };

        Self { client, config }
    }

    /// 发送非流式请求
    async fn send_request(&self, request: &SiliconFlowApiRequest) -> Result<SiliconFlowApiResponse> {
        let url = format!("{}/chat/completions", self.config.base_url);

        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(request)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to send request to SiliconFlow: {}", e))?;

        let status = response.status();
        let response_text = response.text().await
            .map_err(|e| anyhow!("Failed to read SiliconFlow response: {}", e))?;

        if !status.is_success() {
            // 尝试解析错误响应
            if let Ok(error_response) = serde_json::from_str::<SiliconFlowErrorResponse>(&response_text) {
                return Err(anyhow!("SiliconFlow API error: {}", error_response.error.message));
            } else {
                return Err(anyhow!("SiliconFlow API error {}: {}", status, response_text));
            }
        }

        let api_response: SiliconFlowApiResponse = serde_json::from_str(&response_text)
            .map_err(|e| anyhow!("Failed to parse SiliconFlow response: {}", e))?;

        Ok(api_response)
    }

    /// 发送流式请求
    async fn send_stream_request(&self, request: &SiliconFlowApiRequest) -> Result<String> {
        use futures_util::StreamExt;

        let url = format!("{}/chat/completions", self.config.base_url);

        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(request)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to send stream request to SiliconFlow: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!("SiliconFlow API error {}: {}", status, error_text));
        }

        let mut stream = response.bytes_stream();
        let mut content = String::new();
        let mut buffer = Vec::new();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| anyhow!("Stream error: {}", e))?;
            buffer.extend_from_slice(&chunk);

            // 处理缓冲区中的完整行
            let buffer_str = String::from_utf8_lossy(&buffer);
            let lines: Vec<&str> = buffer_str.lines().collect();

            // 保留最后一行（可能不完整）
            if lines.len() > 1 {
                for line in &lines[..lines.len()-1] {
                    if let Some(json_str) = line.strip_prefix("data: ") {
                        if json_str.trim() == "[DONE]" {
                            return Ok(content);
                        }

                        if let Ok(stream_response) = serde_json::from_str::<SiliconFlowStreamResponse>(json_str) {
                            if let Some(choice) = stream_response.choices.first() {
                                if let Some(delta_content) = &choice.delta.content {
                                    content.push_str(delta_content);
                                }
                            }
                        }
                    }
                }

                // 保留最后一行到缓冲区
                if let Some(last_line) = lines.last() {
                    buffer = last_line.as_bytes().to_vec();
                } else {
                    buffer.clear();
                }
            }
        }

        Ok(content)
    }

    /// 构建 API 请求
    fn build_request(&self, ai_request: &AIRequest) -> SiliconFlowApiRequest {
        let model = ai_request.model.as_ref()
            .unwrap_or(&self.config.default_model)
            .clone();

        let messages = vec![
            SiliconFlowMessage {
                role: "user".to_string(),
                content: ai_request.prompt.clone(),
            }
        ];

        SiliconFlowApiRequest {
            model,
            messages,
            temperature: ai_request.temperature,
            max_tokens: ai_request.max_tokens,
            stream: false, // 默认非流式
        }
    }

    /// 验证 API 密钥
    async fn validate_api_key(&self) -> Result<()> {
        let test_request = SiliconFlowApiRequest {
            model: self.config.default_model.clone(),
            messages: vec![
                SiliconFlowMessage {
                    role: "user".to_string(),
                    content: "Hello".to_string(),
                }
            ],
            temperature: Some(0.1),
            max_tokens: Some(10),
            stream: false,
        };

        match self.send_request(&test_request).await {
            Ok(_) => Ok(()),
            Err(e) => {
                if e.to_string().contains("401") || e.to_string().contains("403") {
                    Err(anyhow!("Invalid SiliconFlow API key"))
                } else {
                    Err(e)
                }
            }
        }
    }
}

#[async_trait]
impl AIProvider for SiliconFlowProvider {
    fn name(&self) -> &str {
        "siliconflow"
    }

    async fn analyze_code(&self, request: &AIRequest) -> Result<AIResponse> {
        let api_request = self.build_request(request);
        let start_time = std::time::Instant::now();

        let api_response = self.send_request(&api_request).await?;

        let content = api_response.choices
            .first()
            .ok_or_else(|| anyhow!("No choices in SiliconFlow response"))?
            .message
            .content
            .clone();

        let token_usage = api_response.usage.map(|usage| TokenUsage {
            prompt_tokens: usage.prompt_tokens,
            completion_tokens: usage.completion_tokens,
            total_tokens: usage.total_tokens,
        });

        let mut metadata = HashMap::new();
        metadata.insert("request_id".to_string(), api_response.id);
        metadata.insert("finish_reason".to_string(),
            api_response.choices.first()
                .and_then(|c| c.finish_reason.clone())
                .unwrap_or_else(|| "unknown".to_string())
        );

        Ok(AIResponse {
            content,
            model: api_response.model,
            provider: self.name().to_string(),
            response_time_ms: start_time.elapsed().as_millis() as u64,
            token_usage,
            metadata,
        })
    }

    fn is_available(&self) -> bool {
        !self.config.api_key.is_empty()
    }

    fn supported_models(&self) -> Vec<String> {
        vec![
            "deepseek-ai/DeepSeek-V2.5".to_string(),
            "deepseek-ai/DeepSeek-Coder-V2-Instruct".to_string(),
            "Qwen/Qwen2.5-Coder-32B-Instruct".to_string(),
            "Qwen/Qwen2.5-72B-Instruct".to_string(),
            "meta-llama/Meta-Llama-3.1-70B-Instruct".to_string(),
            "meta-llama/Meta-Llama-3.1-8B-Instruct".to_string(),
            "mistralai/Mistral-7B-Instruct-v0.3".to_string(),
            "01-ai/Yi-1.5-34B-Chat".to_string(),
        ]
    }

    fn validate_config(&self, config: &AIConfig) -> Result<()> {
        if let Some(siliconflow_config) = &config.siliconflow {
            if siliconflow_config.api_key.is_empty() {
                return Err(anyhow!("SiliconFlow API key is required"));
            }
            if siliconflow_config.base_url.is_empty() {
                return Err(anyhow!("SiliconFlow base URL is required"));
            }
            if siliconflow_config.model.is_empty() {
                return Err(anyhow!("SiliconFlow model is required"));
            }
        } else {
            return Err(anyhow!("SiliconFlow configuration is missing"));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    fn create_test_provider() -> SiliconFlowProvider {
        let client = Arc::new(reqwest::Client::new());
        SiliconFlowProvider::new(client, "test-key".to_string(), None)
    }

    #[test]
    fn test_provider_creation() {
        let provider = create_test_provider();
        assert_eq!(provider.name(), "siliconflow");
        assert!(provider.is_available());
        assert_eq!(provider.config.base_url, "https://api.siliconflow.cn/v1");
        assert_eq!(provider.config.default_model, "deepseek-ai/DeepSeek-V2.5");
    }

    #[test]
    fn test_provider_creation_with_custom_url() {
        let client = Arc::new(reqwest::Client::new());
        let provider = SiliconFlowProvider::new(
            client,
            "test-key".to_string(),
            Some("https://custom.api.com/v1".to_string())
        );
        assert_eq!(provider.config.base_url, "https://custom.api.com/v1");
    }

    #[test]
    fn test_supported_models() {
        let provider = create_test_provider();
        let models = provider.supported_models();
        assert!(models.contains(&"deepseek-ai/DeepSeek-V2.5".to_string()));
        assert!(models.contains(&"Qwen/Qwen2.5-Coder-32B-Instruct".to_string()));
        assert!(models.contains(&"meta-llama/Meta-Llama-3.1-70B-Instruct".to_string()));
        assert!(models.len() >= 8);
    }

    #[test]
    fn test_build_request() {
        let provider = create_test_provider();
        let ai_request = AIRequest::code_review("test.rs", "rust", "Review this code");

        let api_request = provider.build_request(&ai_request);
        assert_eq!(api_request.model, "deepseek-ai/DeepSeek-V2.5");
        assert_eq!(api_request.messages.len(), 1);
        assert_eq!(api_request.messages[0].role, "user");
        assert_eq!(api_request.messages[0].content, "Review this code");
        assert!(!api_request.stream);
    }

    #[test]
    fn test_build_request_with_custom_model() {
        let provider = create_test_provider();
        let mut ai_request = AIRequest::code_review("test.py", "python", "Review this Python code");
        ai_request.model = Some("Qwen/Qwen2.5-Coder-32B-Instruct".to_string());
        ai_request.temperature = Some(0.3);
        ai_request.max_tokens = Some(2000);

        let api_request = provider.build_request(&ai_request);
        assert_eq!(api_request.model, "Qwen/Qwen2.5-Coder-32B-Instruct");
        assert_eq!(api_request.temperature, Some(0.3));
        assert_eq!(api_request.max_tokens, Some(2000));
    }

    #[test]
    fn test_config_validation() {
        let provider = create_test_provider();

        // Valid config
        let valid_config = AIConfig {
            siliconflow: Some(crate::ai::manager::SiliconFlowConfig {
                api_key: "test-key".to_string(),
                base_url: "https://api.siliconflow.cn/v1".to_string(),
                model: "deepseek-ai/DeepSeek-V2.5".to_string(),
            }),
            ..Default::default()
        };
        assert!(provider.validate_config(&valid_config).is_ok());

        // Missing config
        let missing_config = AIConfig::default();
        assert!(provider.validate_config(&missing_config).is_err());

        // Empty API key
        let empty_key_config = AIConfig {
            siliconflow: Some(crate::ai::manager::SiliconFlowConfig {
                api_key: "".to_string(),
                base_url: "https://api.siliconflow.cn/v1".to_string(),
                model: "deepseek-ai/DeepSeek-V2.5".to_string(),
            }),
            ..Default::default()
        };
        assert!(provider.validate_config(&empty_key_config).is_err());

        // Empty base URL
        let empty_url_config = AIConfig {
            siliconflow: Some(crate::ai::manager::SiliconFlowConfig {
                api_key: "test-key".to_string(),
                base_url: "".to_string(),
                model: "deepseek-ai/DeepSeek-V2.5".to_string(),
            }),
            ..Default::default()
        };
        assert!(provider.validate_config(&empty_url_config).is_err());
    }

    #[test]
    fn test_provider_not_available_with_empty_key() {
        let client = Arc::new(reqwest::Client::new());
        let provider = SiliconFlowProvider::new(client, "".to_string(), None);
        assert!(!provider.is_available());
    }

    #[test]
    fn test_siliconflow_message_serialization() {
        let message = SiliconFlowMessage {
            role: "user".to_string(),
            content: "Hello, SiliconFlow!".to_string(),
        };

        let json = serde_json::to_string(&message).unwrap();
        assert!(json.contains("user"));
        assert!(json.contains("Hello, SiliconFlow!"));
    }

    #[test]
    fn test_siliconflow_api_request_serialization() {
        let request = SiliconFlowApiRequest {
            model: "deepseek-ai/DeepSeek-V2.5".to_string(),
            messages: vec![
                SiliconFlowMessage {
                    role: "user".to_string(),
                    content: "Test message".to_string(),
                }
            ],
            temperature: Some(0.8),
            max_tokens: Some(1500),
            stream: false,
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("deepseek-ai/DeepSeek-V2.5"));
        assert!(json.contains("Test message"));
        assert!(json.contains("0.8"));
        assert!(json.contains("1500"));
        assert!(json.contains("false"));
    }

    #[test]
    fn test_siliconflow_api_response_deserialization() {
        let json = r#"{
            "id": "chatcmpl-sf123",
            "object": "chat.completion",
            "created": 1677652288,
            "model": "deepseek-ai/DeepSeek-V2.5",
            "choices": [{
                "index": 0,
                "message": {
                    "role": "assistant",
                    "content": "Hello! I'm SiliconFlow AI assistant."
                },
                "finish_reason": "stop"
            }],
            "usage": {
                "prompt_tokens": 15,
                "completion_tokens": 8,
                "total_tokens": 23
            }
        }"#;

        let response: SiliconFlowApiResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.id, "chatcmpl-sf123");
        assert_eq!(response.model, "deepseek-ai/DeepSeek-V2.5");
        assert_eq!(response.choices.len(), 1);
        assert_eq!(response.choices[0].message.content, "Hello! I'm SiliconFlow AI assistant.");
        assert!(response.usage.is_some());
        assert_eq!(response.usage.unwrap().total_tokens, 23);
    }

    #[test]
    fn test_siliconflow_error_response_deserialization() {
        let json = r#"{
            "error": {
                "message": "Invalid API key provided",
                "type": "invalid_request_error",
                "code": "invalid_api_key"
            }
        }"#;

        let error_response: SiliconFlowErrorResponse = serde_json::from_str(json).unwrap();
        assert_eq!(error_response.error.message, "Invalid API key provided");
        assert_eq!(error_response.error.error_type, Some("invalid_request_error".to_string()));
        assert_eq!(error_response.error.code, Some("invalid_api_key".to_string()));
    }

    #[test]
    fn test_siliconflow_stream_response_deserialization() {
        let json = r#"{
            "id": "chatcmpl-sf123",
            "object": "chat.completion.chunk",
            "created": 1677652288,
            "model": "deepseek-ai/DeepSeek-V2.5",
            "choices": [{
                "index": 0,
                "delta": {
                    "content": "Hello from"
                },
                "finish_reason": null
            }]
        }"#;

        let response: SiliconFlowStreamResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.id, "chatcmpl-sf123");
        assert_eq!(response.model, "deepseek-ai/DeepSeek-V2.5");
        assert_eq!(response.choices.len(), 1);
        assert_eq!(response.choices[0].delta.content, Some("Hello from".to_string()));
    }
}