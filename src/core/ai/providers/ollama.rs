use crate::core::ai::http::shared_client;
use crate::core::ai::provider::{AIProvider, ProviderConfig, StreamResponse};
use crate::core::ai::stream::map_jsonl_stream;
use anyhow::Result;
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

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

/// 从 Ollama JSONL 中提取内容
fn extract_ollama_content(line: &str) -> Option<String> {
    serde_json::from_str::<OllamaResponse>(line)
        .ok()
        .map(|r| r.response)
}

/// Ollama AI 提供商
pub struct OllamaProvider {
    client: &'static Client,
}

impl Default for OllamaProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl OllamaProvider {
    /// 创建新的 Ollama 提供商
    pub fn new() -> Self {
        Self {
            client: shared_client(),
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

        let response = self
            .client
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
        let mapped_stream = map_jsonl_stream(stream, extract_ollama_content);

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

    #[test]
    fn test_extract_ollama_content() {
        let json = r#"{"response": "hello", "done": false}"#;
        assert_eq!(extract_ollama_content(json), Some("hello".to_string()));

        assert_eq!(extract_ollama_content("invalid"), None);
    }
}
