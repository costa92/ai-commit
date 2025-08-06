use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use anyhow::{Result, anyhow};

use crate::ai::manager::{AIProvider, AIConfig, AIRequest, AIResponse, TokenUsage};

/// DeepSeek AI 提供商
pub struct DeepSeekProvider {
    client: Arc<reqwest::Client>,
    config: DeepSeekProviderConfig,
}

#[derive(Debug, Clone)]
struct DeepSeekProviderConfig {
    api_key: String,
    base_url: String,
    default_model: String,
}

/// DeepSeek API 请求结构
#[derive(Debug, Serialize)]
struct DeepSeekApiRequest {
    model: String,
    messages: Vec<DeepSeekMessage>,
    temperature: Option<f32>,
    max_tokens: Option<u32>,
    stream: bool,
}

/// DeepSeek 消息结构
#[derive(Debug, Serialize)]
struct DeepSeekMessage {
    role: String,
    content: String,
}

/// DeepSeek API 响应结构
#[derive(Debug, Deserialize)]
struct DeepSeekApiResponse {
    id: String,
    object: String,
    created: u64,
    model: String,
    choices: Vec<DeepSeekChoice>,
    usage: Option<DeepSeekUsage>,
}

/// DeepSeek 选择结构
#[derive(Debug, Deserialize)]
struct DeepSeekChoice {
    index: u32,
    message: DeepSeekResponseMessage,
    finish_reason: Option<String>,
}

/// DeepSeek 响应消息结构
#[derive(Debug, Deserialize)]
struct DeepSeekResponseMessage {
    role: String,
    content: String,
}

/// DeepSeek 使用情况结构
#[derive(Debug, Deserialize)]
struct DeepSeekUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

/// DeepSeek 流式响应结构
#[derive(Debug, Deserialize)]
struct DeepSeekStreamResponse {
    id: String,
    object: String,
    created: u64,
    model: String,
    choices: Vec<DeepSeekStreamChoice>,
}

/// DeepSeek 流式选择结构
#[derive(Debug, Deserialize)]
struct DeepSeekStreamChoice {
    index: u32,
    delta: DeepSeekDelta,
    finish_reason: Option<String>,
}

/// DeepSeek 增量内容结构
#[derive(Debug, Deserialize)]
struct DeepSeekDelta {
    role: Option<String>,
    content: Option<String>,
}

impl DeepSeekProvider {
    /// 创建新的 DeepSeek 提供商
    pub fn new(client: Arc<reqwest::Client>, api_key: String, base_url: Option<String>) -> Self {
        let config = DeepSeekProviderConfig {
            api_key,
            base_url: base_url.unwrap_or_else(|| "https://api.deepseek.com/v1".to_string()),
            default_model: "deepseek-coder".to_string(),
        };

        Self { client, config }
    }

    /// 发送非流式请求
    async fn send_request(&self, request: &DeepSeekApiRequest) -> Result<DeepSeekApiResponse> {
        let url = format!("{}/chat/completions", self.config.base_url);

        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(request)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to send request to DeepSeek: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!("DeepSeek API error {}: {}", status, error_text));
        }

        let api_response: DeepSeekApiResponse = response.json().await
            .map_err(|e| anyhow!("Failed to parse DeepSeek response: {}", e))?;

        Ok(api_response)
    }

    /// 发送流式请求
    async fn send_stream_request(&self, request: &DeepSeekApiRequest) -> Result<String> {
        use futures_util::StreamExt;

        let url = format!("{}/chat/completions", self.config.base_url);

        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(request)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to send stream request to DeepSeek: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!("DeepSeek API error {}: {}", status, error_text));
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

                        if let Ok(stream_response) = serde_json::from_str::<DeepSeekStreamResponse>(json_str) {
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
    fn build_request(&self, ai_request: &AIRequest) -> DeepSeekApiRequest {
        let model = ai_request.model.as_ref()
            .unwrap_or(&self.config.default_model)
            .clone();

        let messages = vec![
            DeepSeekMessage {
                role: "user".to_string(),
                content: ai_request.prompt.clone(),
            }
        ];

        DeepSeekApiRequest {
            model,
            messages,
            temperature: ai_request.temperature,
            max_tokens: ai_request.max_tokens,
            stream: false, // 默认非流式
        }
    }

    /// 验证 API 密钥
    async fn validate_api_key(&self) -> Result<()> {
        let test_request = DeepSeekApiRequest {
            model: self.config.default_model.clone(),
            messages: vec![
                DeepSeekMessage {
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
                    Err(anyhow!("Invalid DeepSeek API key"))
                } else {
                    Err(e)
                }
            }
        }
    }
}

#[async_trait]
impl AIProvider for DeepSeekProvider {
    fn name(&self) -> &str {
        "deepseek"
    }

    async fn analyze_code(&self, request: &AIRequest) -> Result<AIResponse> {
        let api_request = self.build_request(request);
        let start_time = std::time::Instant::now();

        let api_response = self.send_request(&api_request).await?;

        let content = api_response.choices
            .first()
            .ok_or_else(|| anyhow!("No choices in DeepSeek response"))?
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
            "deepseek-coder".to_string(),
            "deepseek-chat".to_string(),
            "deepseek-v2-coder".to_string(),
            "deepseek-v2-chat".to_string(),
        ]
    }

    fn validate_config(&self, config: &AIConfig) -> Result<()> {
        if let Some(deepseek_config) = &config.deepseek {
            if deepseek_config.api_key.is_empty() {
                return Err(anyhow!("DeepSeek API key is required"));
            }
            if deepseek_config.base_url.is_empty() {
                return Err(anyhow!("DeepSeek base URL is required"));
            }
            if deepseek_config.model.is_empty() {
                return Err(anyhow!("DeepSeek model is required"));
            }
        } else {
            return Err(anyhow!("DeepSeek configuration is missing"));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    fn create_test_provider() -> DeepSeekProvider {
        let client = Arc::new(reqwest::Client::new());
        DeepSeekProvider::new(client, "test-key".to_string(), None)
    }

    #[test]
    fn test_provider_creation() {
        let provider = create_test_provider();
        assert_eq!(provider.name(), "deepseek");
        assert!(provider.is_available());
    }

    #[test]
    fn test_supported_models() {
        let provider = create_test_provider();
        let models = provider.supported_models();
        assert!(models.contains(&"deepseek-coder".to_string()));
        assert!(models.contains(&"deepseek-chat".to_string()));
        assert!(models.len() >= 2);
    }

    #[test]
    fn test_build_request() {
        let provider = create_test_provider();
        let ai_request = AIRequest::code_review("test.rs", "rust", "Review this code");

        let api_request = provider.build_request(&ai_request);
        assert_eq!(api_request.model, "deepseek-coder");
        assert_eq!(api_request.messages.len(), 1);
        assert_eq!(api_request.messages[0].role, "user");
        assert_eq!(api_request.messages[0].content, "Review this code");
        assert!(!api_request.stream);
    }

    #[test]
    fn test_build_request_with_custom_model() {
        let provider = create_test_provider();
        let mut ai_request = AIRequest::code_review("test.rs", "rust", "Review this code");
        ai_request.model = Some("deepseek-chat".to_string());
        ai_request.temperature = Some(0.5);
        ai_request.max_tokens = Some(1000);

        let api_request = provider.build_request(&ai_request);
        assert_eq!(api_request.model, "deepseek-chat");
        assert_eq!(api_request.temperature, Some(0.5));
        assert_eq!(api_request.max_tokens, Some(1000));
    }

    #[test]
    fn test_config_validation() {
        let provider = create_test_provider();

        // Valid config
        let valid_config = AIConfig {
            deepseek: Some(crate::ai::manager::DeepSeekConfig {
                api_key: "test-key".to_string(),
                base_url: "https://api.deepseek.com/v1".to_string(),
                model: "deepseek-coder".to_string(),
            }),
            ..Default::default()
        };
        assert!(provider.validate_config(&valid_config).is_ok());

        // Missing config
        let missing_config = AIConfig::default();
        assert!(provider.validate_config(&missing_config).is_err());

        // Empty API key
        let empty_key_config = AIConfig {
            deepseek: Some(crate::ai::manager::DeepSeekConfig {
                api_key: "".to_string(),
                base_url: "https://api.deepseek.com/v1".to_string(),
                model: "deepseek-coder".to_string(),
            }),
            ..Default::default()
        };
        assert!(provider.validate_config(&empty_key_config).is_err());
    }

    #[test]
    fn test_provider_not_available_with_empty_key() {
        let client = Arc::new(reqwest::Client::new());
        let provider = DeepSeekProvider::new(client, "".to_string(), None);
        assert!(!provider.is_available());
    }

    #[test]
    fn test_deepseek_message_serialization() {
        let message = DeepSeekMessage {
            role: "user".to_string(),
            content: "Hello, world!".to_string(),
        };

        let json = serde_json::to_string(&message).unwrap();
        assert!(json.contains("user"));
        assert!(json.contains("Hello, world!"));
    }

    #[test]
    fn test_deepseek_api_request_serialization() {
        let request = DeepSeekApiRequest {
            model: "deepseek-coder".to_string(),
            messages: vec![
                DeepSeekMessage {
                    role: "user".to_string(),
                    content: "Test message".to_string(),
                }
            ],
            temperature: Some(0.7),
            max_tokens: Some(1000),
            stream: false,
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("deepseek-coder"));
        assert!(json.contains("Test message"));
        assert!(json.contains("0.7"));
        assert!(json.contains("1000"));
        assert!(json.contains("false"));
    }

    #[test]
    fn test_deepseek_api_response_deserialization() {
        let json = r#"{
            "id": "chatcmpl-123",
            "object": "chat.completion",
            "created": 1677652288,
            "model": "deepseek-coder",
            "choices": [{
                "index": 0,
                "message": {
                    "role": "assistant",
                    "content": "Hello! How can I help you today?"
                },
                "finish_reason": "stop"
            }],
            "usage": {
                "prompt_tokens": 9,
                "completion_tokens": 12,
                "total_tokens": 21
            }
        }"#;

        let response: DeepSeekApiResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.id, "chatcmpl-123");
        assert_eq!(response.model, "deepseek-coder");
        assert_eq!(response.choices.len(), 1);
        assert_eq!(response.choices[0].message.content, "Hello! How can I help you today?");
        assert!(response.usage.is_some());
        assert_eq!(response.usage.unwrap().total_tokens, 21);
    }

    #[test]
    fn test_deepseek_stream_response_deserialization() {
        let json = r#"{
            "id": "chatcmpl-123",
            "object": "chat.completion.chunk",
            "created": 1677652288,
            "model": "deepseek-coder",
            "choices": [{
                "index": 0,
                "delta": {
                    "content": "Hello"
                },
                "finish_reason": null
            }]
        }"#;

        let response: DeepSeekStreamResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.id, "chatcmpl-123");
        assert_eq!(response.model, "deepseek-coder");
        assert_eq!(response.choices.len(), 1);
        assert_eq!(response.choices[0].delta.content, Some("Hello".to_string()));
    }
}