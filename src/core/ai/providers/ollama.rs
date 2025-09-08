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

/// Ollama 请求结构
#[derive(Serialize)]
struct OllamaRequest<'a> {
    model: &'a str,
    prompt: &'a str,
    stream: bool,
    options: OllamaOptions,
}

/// Ollama 选项
#[derive(Serialize)]
struct OllamaOptions {
    temperature: f32,
    top_p: f32,
    num_predict: i32,
}

impl Default for OllamaOptions {
    fn default() -> Self {
        Self {
            temperature: 0.7,
            top_p: 0.9,
            num_predict: 500,
        }
    }
}

/// Ollama 响应结构
#[derive(Deserialize)]
#[allow(dead_code)]
struct OllamaResponse {
    response: String,
    done: bool,
}

/// Ollama AI 提供商
pub struct OllamaProvider {
    client: &'static Client,
}

impl OllamaProvider {
    /// 创建新的 Ollama 提供商
    pub fn new() -> Self {
        Self {
            client: &HTTP_CLIENT,
        }
    }
    
    /// 发送请求到 Ollama
    async fn send_request(
        &self,
        prompt: &str,
        config: &ProviderConfig,
    ) -> Result<reqwest::Response> {
        let request = OllamaRequest {
            model: &config.model,
            prompt,
            stream: config.stream,
            options: OllamaOptions::default(),
        };
        
        let response = self.client
            .post(&config.api_url)
            .json(&request)
            .timeout(Duration::from_secs(config.timeout_secs))
            .send()
            .await?;
        
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            anyhow::bail!("Ollama request failed: {} - {}", status, text);
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
                if let Ok(ollama_response) = serde_json::from_str::<OllamaResponse>(line) {
                    stdout_handle.write_all(ollama_response.response.as_bytes()).await?;
                    stdout_handle.flush().await?;
                    message.push_str(&ollama_response.response);
                    
                    if ollama_response.done {
                        break;
                    }
                }
            }
        }
        
        stdout_handle.write_all(b"\n").await?;
        Ok(message)
    }
}

#[async_trait]
impl AIProvider for OllamaProvider {
    async fn generate(&self, prompt: &str, config: &ProviderConfig) -> Result<String> {
        let mut config = config.clone();
        config.stream = false;
        
        let response = self.send_request(prompt, &config).await?;
        let ollama_response: OllamaResponse = response.json().await?;
        
        Ok(ollama_response.response)
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
                        if let Ok(response) = serde_json::from_str::<OllamaResponse>(line) {
                            result.push_str(&response.response);
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
    fn test_ollama_provider_creation() {
        let provider = OllamaProvider::new();
        assert!(!std::ptr::addr_of!(provider.client).is_null());
    }

    #[test]
    fn test_ollama_options_default() {
        let options = OllamaOptions::default();
        assert_eq!(options.temperature, 0.7);
        assert_eq!(options.top_p, 0.9);
        assert_eq!(options.num_predict, 500);
    }

    #[test]
    fn test_ollama_request_serialization() {
        let request = OllamaRequest {
            model: "mistral",
            prompt: "test prompt",
            stream: true,
            options: OllamaOptions::default(),
        };
        
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("mistral"));
        assert!(json.contains("test prompt"));
        assert!(json.contains("true"));
        assert!(json.contains("temperature"));
    }

    #[test]
    fn test_ollama_response_deserialization() {
        let json = r#"{"response": "test response", "done": true}"#;
        let response: OllamaResponse = serde_json::from_str(json).unwrap();
        
        assert_eq!(response.response, "test response");
        assert!(response.done);
    }
}