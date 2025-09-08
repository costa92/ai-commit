use crate::core::ai::provider::{AIProvider, ProviderConfig, StreamResponse};
use anyhow::Result;
use async_trait::async_trait;
use futures_util::StreamExt;
use once_cell::sync::Lazy;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::io::{stdout, AsyncWriteExt};

/// 全局 HTTP 客户端
static HTTP_CLIENT: Lazy<Client> = Lazy::new(|| {
    Client::builder()
        .pool_max_idle_per_host(10)
        .pool_idle_timeout(Duration::from_secs(30))
        .timeout(Duration::from_secs(60))
        .build()
        .expect("Failed to create HTTP client")
});

/// Deepseek 请求结构
#[derive(Serialize)]
struct DeepseekRequest<'a> {
    model: &'a str,
    messages: Vec<Message<'a>>,
    stream: bool,
    temperature: f32,
    max_tokens: i32,
}

/// 消息结构
#[derive(Serialize)]
struct Message<'a> {
    role: &'a str,
    content: &'a str,
}

/// Deepseek 响应结构
#[derive(Deserialize)]
struct DeepseekResponse {
    choices: Vec<Choice>,
}

/// 选择结构
#[derive(Deserialize)]
struct Choice {
    delta: Option<Delta>,
    message: Option<MessageResponse>,
}

/// Delta 结构（用于流式响应）
#[derive(Deserialize)]
struct Delta {
    content: Option<String>,
}

/// 完整消息响应（用于非流式响应）
#[derive(Deserialize)]
struct MessageResponse {
    content: String,
}

/// Deepseek AI 提供商
pub struct DeepseekProvider {
    client: &'static Client,
}

impl DeepseekProvider {
    /// 创建新的 Deepseek 提供商
    pub fn new() -> Self {
        Self {
            client: &HTTP_CLIENT,
        }
    }
    
    /// 发送请求到 Deepseek
    async fn send_request(
        &self,
        prompt: &str,
        config: &ProviderConfig,
    ) -> Result<reqwest::Response> {
        let api_key = config.api_key.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Deepseek API key is required"))?;
        
        let request = DeepseekRequest {
            model: &config.model,
            messages: vec![Message {
                role: "user",
                content: prompt,
            }],
            stream: config.stream,
            temperature: 0.7,
            max_tokens: 500,
        };
        
        let response = self.client
            .post(&config.api_url)
            .bearer_auth(api_key)
            .json(&request)
            .timeout(Duration::from_secs(config.timeout_secs))
            .send()
            .await?;
        
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            anyhow::bail!("Deepseek request failed: {} - {}", status, text);
        }
        
        Ok(response)
    }
    
    /// 处理流式响应
    #[allow(dead_code)]
    async fn handle_stream_response(
        response: reqwest::Response,
    ) -> Result<String> {
        let mut message = String::with_capacity(2048);
        let mut stdout_handle = stdout();
        let mut stream = response.bytes_stream();
        
        while let Some(item) = stream.next().await {
            let chunk = item?;
            let chunk_str = std::str::from_utf8(&chunk)?;
            
            for line in chunk_str.lines() {
                if line.starts_with("data:") {
                    let json_str = line.strip_prefix("data:").unwrap().trim();
                    if json_str == "[DONE]" {
                        break;
                    }
                    
                    if let Ok(response) = serde_json::from_str::<DeepseekResponse>(json_str) {
                        if let Some(choice) = response.choices.first() {
                            if let Some(delta) = &choice.delta {
                                if let Some(content) = &delta.content {
                                    stdout_handle.write_all(content.as_bytes()).await?;
                                    stdout_handle.flush().await?;
                                    message.push_str(content);
                                }
                            }
                        }
                    }
                }
            }
        }
        
        stdout_handle.write_all(b"\n").await?;
        Ok(message)
    }
}

#[async_trait]
impl AIProvider for DeepseekProvider {
    async fn generate(&self, prompt: &str, config: &ProviderConfig) -> Result<String> {
        let mut config = config.clone();
        config.stream = false;
        
        let response = self.send_request(prompt, &config).await?;
        let deepseek_response: DeepseekResponse = response.json().await?;
        
        let content = deepseek_response.choices
            .first()
            .and_then(|c| {
                if let Some(delta) = &c.delta {
                    delta.content.clone()
                } else if let Some(message) = &c.message {
                    Some(message.content.clone())
                } else {
                    None
                }
            })
            .unwrap_or_default();
        
        Ok(content)
    }
    
    async fn stream_generate(
        &self,
        prompt: &str,
        config: &ProviderConfig,
    ) -> Result<StreamResponse> {
        let mut config = config.clone();
        config.stream = true;
        
        let response = self.send_request(prompt, &config).await?;
        let stream = response.bytes_stream();
        
        let mapped_stream = stream.map(move |item| {
            match item {
                Ok(chunk) => {
                    let chunk_str = std::str::from_utf8(&chunk)
                        .map_err(|e| anyhow::anyhow!("UTF-8 error: {}", e))?;
                    
                    let mut result = String::new();
                    for line in chunk_str.lines() {
                        if line.starts_with("data:") {
                            let json_str = line.strip_prefix("data:").unwrap().trim();
                            if json_str != "[DONE]" {
                                if let Ok(response) = serde_json::from_str::<DeepseekResponse>(json_str) {
                                    if let Some(choice) = response.choices.first() {
                                        if let Some(delta) = &choice.delta {
                                            if let Some(content) = &delta.content {
                                                result.push_str(content);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Ok(result)
                }
                Err(e) => Err(anyhow::anyhow!("Stream error: {}", e)),
            }
        });
        
        Ok(Box::pin(mapped_stream))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deepseek_provider_creation() {
        let provider = DeepseekProvider::new();
        assert!(!std::ptr::addr_of!(provider.client).is_null());
    }

    #[test]
    fn test_deepseek_request_serialization() {
        let request = DeepseekRequest {
            model: "deepseek-chat",
            messages: vec![Message {
                role: "user",
                content: "test",
            }],
            stream: true,
            temperature: 0.7,
            max_tokens: 500,
        };
        
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("deepseek-chat"));
        assert!(json.contains("user"));
        assert!(json.contains("test"));
        assert!(json.contains("0.7"));
    }

    #[test]
    fn test_deepseek_response_deserialization() {
        let json = r#"{
            "choices": [{
                "delta": {
                    "content": "test response"
                }
            }]
        }"#;
        
        let response: DeepseekResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.choices.len(), 1);
        assert!(response.choices[0].delta.is_some());
        assert_eq!(
            response.choices[0].delta.as_ref().unwrap().content.as_ref().unwrap(),
            "test response"
        );
    }

    #[test]
    fn test_deepseek_non_streaming_response() {
        let json = r#"{
            "choices": [{
                "message": {
                    "content": "non-streaming response"
                }
            }]
        }"#;
        
        let response: DeepseekResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.choices.len(), 1);
        assert!(response.choices[0].message.is_some());
        assert_eq!(response.choices[0].message.as_ref().unwrap().content, "non-streaming response");
    }

    #[test]
    fn test_message_structure() {
        let message = Message {
            role: "user",
            content: "Hello",
        };
        
        let json = serde_json::to_string(&message).unwrap();
        assert!(json.contains("user"));
        assert!(json.contains("Hello"));
    }
}