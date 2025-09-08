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

/// SiliconFlow 请求结构
#[derive(Serialize)]
struct SiliconFlowRequest<'a> {
    model: &'a str,
    messages: Vec<Message<'a>>,
    stream: bool,
    temperature: f32,
    max_tokens: i32,
    top_p: f32,
}

/// 消息结构
#[derive(Serialize)]
struct Message<'a> {
    role: &'a str,
    content: &'a str,
}

/// SiliconFlow 响应结构
#[derive(Deserialize)]
struct SiliconFlowResponse {
    choices: Vec<Choice>,
}

/// 选择结构
#[derive(Deserialize)]
struct Choice {
    delta: Delta,
}

/// Delta 结构
#[derive(Deserialize)]
struct Delta {
    content: String,
}

/// SiliconFlow AI 提供商
pub struct SiliconFlowProvider {
    client: &'static Client,
}

impl SiliconFlowProvider {
    /// 创建新的 SiliconFlow 提供商
    pub fn new() -> Self {
        Self {
            client: &HTTP_CLIENT,
        }
    }
    
    /// 发送请求到 SiliconFlow
    async fn send_request(
        &self,
        prompt: &str,
        config: &ProviderConfig,
    ) -> Result<reqwest::Response> {
        let api_key = config.api_key.as_ref()
            .ok_or_else(|| anyhow::anyhow!("SiliconFlow API key is required"))?;
        
        let request = SiliconFlowRequest {
            model: &config.model,
            messages: vec![Message {
                role: "user",
                content: prompt,
            }],
            stream: config.stream,
            temperature: 0.7,
            max_tokens: 500,
            top_p: 0.9,
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
            anyhow::bail!("SiliconFlow request failed: {} - {}", status, text);
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
                    
                    if let Ok(response) = serde_json::from_str::<SiliconFlowResponse>(json_str) {
                        if let Some(choice) = response.choices.first() {
                            let content = &choice.delta.content;
                            stdout_handle.write_all(content.as_bytes()).await?;
                            stdout_handle.flush().await?;
                            message.push_str(content);
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
impl AIProvider for SiliconFlowProvider {
    async fn generate(&self, prompt: &str, config: &ProviderConfig) -> Result<String> {
        let mut config = config.clone();
        config.stream = false;
        
        let response = self.send_request(prompt, &config).await?;
        let siliconflow_response: SiliconFlowResponse = response.json().await?;
        
        let content = siliconflow_response.choices
            .first()
            .map(|c| c.delta.content.clone())
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
                                if let Ok(response) = serde_json::from_str::<SiliconFlowResponse>(json_str) {
                                    if let Some(choice) = response.choices.first() {
                                        result.push_str(&choice.delta.content);
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
    fn test_siliconflow_provider_creation() {
        let provider = SiliconFlowProvider::new();
        assert!(!std::ptr::addr_of!(provider.client).is_null());
    }

    #[test]
    fn test_siliconflow_request_serialization() {
        let request = SiliconFlowRequest {
            model: "Qwen/Qwen2-7B-Instruct",
            messages: vec![Message {
                role: "user",
                content: "test",
            }],
            stream: true,
            temperature: 0.7,
            max_tokens: 500,
            top_p: 0.9,
        };
        
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("Qwen/Qwen2-7B-Instruct"));
        assert!(json.contains("user"));
        assert!(json.contains("test"));
        assert!(json.contains("0.7"));
        assert!(json.contains("0.9"));
    }

    #[test]
    fn test_siliconflow_response_deserialization() {
        let json = r#"{
            "choices": [{
                "delta": {
                    "content": "test response"
                }
            }]
        }"#;
        
        let response: SiliconFlowResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.choices.len(), 1);
        assert_eq!(response.choices[0].delta.content, "test response");
    }

    #[test]
    fn test_message_structure() {
        let message = Message {
            role: "assistant",
            content: "Response",
        };
        
        let json = serde_json::to_string(&message).unwrap();
        assert!(json.contains("assistant"));
        assert!(json.contains("Response"));
    }
}