use crate::core::ai::http::shared_client;
use crate::core::ai::provider::{ProviderConfig, StreamResponse};
use crate::core::ai::stream::map_sse_stream;
use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// OpenAI 兼容 Chat Completion 请求
#[derive(Serialize)]
pub struct ChatCompletionRequest<'a> {
    pub model: &'a str,
    pub messages: Vec<ChatMessage<'a>>,
    pub stream: bool,
    pub temperature: f32,
    pub max_tokens: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
}

/// Chat 消息
#[derive(Serialize)]
pub struct ChatMessage<'a> {
    pub role: &'a str,
    pub content: &'a str,
}

/// OpenAI 兼容 Chat Completion 响应
#[derive(Deserialize)]
pub struct ChatCompletionResponse {
    pub choices: Vec<ChatChoice>,
}

/// 响应选择
#[derive(Deserialize)]
pub struct ChatChoice {
    pub delta: Option<ChatDelta>,
    pub message: Option<ChatMessageResponse>,
}

/// 流式响应 Delta
#[derive(Deserialize)]
pub struct ChatDelta {
    pub content: Option<String>,
}

/// 非流式完整消息响应
#[derive(Deserialize)]
pub struct ChatMessageResponse {
    pub content: String,
}

/// 从 OpenAI 兼容 SSE JSON 中提取内容
pub fn extract_chat_content(json_str: &str) -> Option<String> {
    serde_json::from_str::<ChatCompletionResponse>(json_str)
        .ok()
        .and_then(|r| r.choices.into_iter().next())
        .and_then(|c| c.delta)
        .and_then(|d| d.content)
}

/// OpenAI 兼容 Provider 基类
///
/// 提供共享的请求发送、生成和流式处理逻辑。
/// Deepseek/Kimi/SiliconFlow/OpenAI/Qwen 等 OpenAI 兼容 API 均可复用。
pub struct OpenAICompatibleBase {
    client: &'static Client,
}

impl Default for OpenAICompatibleBase {
    fn default() -> Self {
        Self::new()
    }
}

impl OpenAICompatibleBase {
    pub fn new() -> Self {
        Self {
            client: shared_client(),
        }
    }

    /// 发送 Chat Completion 请求
    pub async fn send_chat_request(
        &self,
        prompt: &str,
        config: &ProviderConfig,
        provider_name: &str,
        top_p: Option<f32>,
    ) -> Result<reqwest::Response> {
        let api_key = config
            .api_key
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("{} API key is required", provider_name))?;

        let request = ChatCompletionRequest {
            model: &config.model,
            messages: vec![ChatMessage {
                role: "user",
                content: prompt,
            }],
            stream: config.stream,
            temperature: 0.7,
            max_tokens: 500,
            top_p,
        };

        let response = self
            .client
            .post(&config.api_url)
            .bearer_auth(api_key)
            .json(&request)
            .timeout(Duration::from_secs(config.timeout_secs))
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            anyhow::bail!("{} request failed: {} - {}", provider_name, status, text);
        }

        Ok(response)
    }

    /// 非流式生成
    pub async fn generate_chat(
        &self,
        prompt: &str,
        config: &ProviderConfig,
        provider_name: &str,
        top_p: Option<f32>,
    ) -> Result<String> {
        let mut config = config.clone();
        config.stream = false;

        let response = self
            .send_chat_request(prompt, &config, provider_name, top_p)
            .await?;
        let chat_response: ChatCompletionResponse = response.json().await?;

        let content = chat_response
            .choices
            .first()
            .and_then(|c| {
                if let Some(delta) = &c.delta {
                    delta.content.clone()
                } else {
                    c.message.as_ref().map(|m| m.content.clone())
                }
            })
            .unwrap_or_default();

        Ok(content)
    }

    /// 流式生成
    pub async fn stream_chat(
        &self,
        prompt: &str,
        config: &ProviderConfig,
        provider_name: &str,
        top_p: Option<f32>,
    ) -> Result<StreamResponse> {
        let mut config = config.clone();
        config.stream = true;

        let response = self
            .send_chat_request(prompt, &config, provider_name, top_p)
            .await?;
        let stream = response.bytes_stream();
        let mapped_stream = map_sse_stream(stream, extract_chat_content);

        Ok(Box::pin(mapped_stream))
    }

    /// 发送无需 API key 的请求（如 Ollama-chat 兼容模式）
    pub async fn send_chat_request_no_auth(
        &self,
        prompt: &str,
        config: &ProviderConfig,
        provider_name: &str,
        top_p: Option<f32>,
    ) -> Result<reqwest::Response> {
        let request = ChatCompletionRequest {
            model: &config.model,
            messages: vec![ChatMessage {
                role: "user",
                content: prompt,
            }],
            stream: config.stream,
            temperature: 0.7,
            max_tokens: 500,
            top_p,
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
            anyhow::bail!("{} request failed: {} - {}", provider_name, status, text);
        }

        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chat_request_serialization() {
        let request = ChatCompletionRequest {
            model: "gpt-4o-mini",
            messages: vec![ChatMessage {
                role: "user",
                content: "test",
            }],
            stream: true,
            temperature: 0.7,
            max_tokens: 500,
            top_p: None,
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("gpt-4o-mini"));
        assert!(json.contains("user"));
        assert!(json.contains("test"));
        assert!(!json.contains("top_p")); // None should be skipped
    }

    #[test]
    fn test_chat_request_with_top_p() {
        let request = ChatCompletionRequest {
            model: "test-model",
            messages: vec![ChatMessage {
                role: "user",
                content: "hello",
            }],
            stream: false,
            temperature: 0.7,
            max_tokens: 500,
            top_p: Some(0.9),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("0.9"));
        assert!(json.contains("top_p"));
    }

    #[test]
    fn test_chat_response_deserialization_delta() {
        let json = r#"{"choices": [{"delta": {"content": "hello"}}]}"#;
        let response: ChatCompletionResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.choices.len(), 1);
        assert_eq!(
            response.choices[0]
                .delta
                .as_ref()
                .unwrap()
                .content
                .as_ref()
                .unwrap(),
            "hello"
        );
    }

    #[test]
    fn test_chat_response_deserialization_message() {
        let json = r#"{"choices": [{"message": {"content": "world"}}]}"#;
        let response: ChatCompletionResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.choices.len(), 1);
        assert_eq!(
            response.choices[0].message.as_ref().unwrap().content,
            "world"
        );
    }

    #[test]
    fn test_extract_chat_content() {
        let json = r#"{"choices": [{"delta": {"content": "hi"}}]}"#;
        assert_eq!(extract_chat_content(json), Some("hi".to_string()));
        assert_eq!(extract_chat_content("invalid"), None);
    }

    #[test]
    fn test_openai_compatible_base_creation() {
        let base = OpenAICompatibleBase::new();
        assert!(!std::ptr::addr_of!(base.client).is_null());
    }
}
