use crate::core::ai::http::shared_client;
use crate::core::ai::provider::{AIProvider, ProviderConfig, StreamResponse};
use crate::core::ai::stream::map_sse_stream;
use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Anthropic Messages API 请求
#[derive(Serialize)]
struct AnthropicRequest<'a> {
    model: &'a str,
    max_tokens: i32,
    messages: Vec<AnthropicMessage<'a>>,
    stream: bool,
}

/// Anthropic 消息
#[derive(Serialize)]
struct AnthropicMessage<'a> {
    role: &'a str,
    content: &'a str,
}

/// Anthropic 非流式响应
#[derive(Deserialize)]
struct AnthropicResponse {
    content: Vec<AnthropicContentBlock>,
}

/// Anthropic 内容块
#[derive(Deserialize)]
struct AnthropicContentBlock {
    #[serde(rename = "type")]
    _type: String,
    text: Option<String>,
}

/// Anthropic 流式事件
#[derive(Deserialize)]
struct AnthropicStreamEvent {
    #[serde(rename = "type")]
    event_type: String,
    delta: Option<AnthropicDelta>,
}

/// Anthropic 流式 Delta
#[derive(Deserialize)]
struct AnthropicDelta {
    #[serde(rename = "type")]
    _type: Option<String>,
    text: Option<String>,
}

/// 从 Anthropic SSE JSON 中提取内容
///
/// 仅处理 `content_block_delta` 事件中的 `text_delta`。
fn extract_anthropic_content(json_str: &str) -> Option<String> {
    serde_json::from_str::<AnthropicStreamEvent>(json_str)
        .ok()
        .filter(|e| e.event_type == "content_block_delta")
        .and_then(|e| e.delta)
        .and_then(|d| d.text)
}

/// Claude (Anthropic) AI 提供商
///
/// 使用 Anthropic Messages API，非 OpenAI 兼容格式。
/// 默认 URL: https://api.anthropic.com/v1/messages
/// 默认 model: claude-sonnet-4-20250514
/// 环境变量: AI_COMMIT_CLAUDE_API_KEY
pub struct ClaudeProvider {
    client: &'static reqwest::Client,
}

impl Default for ClaudeProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl ClaudeProvider {
    pub fn new() -> Self {
        Self {
            client: shared_client(),
        }
    }

    async fn send_request(
        &self,
        prompt: &str,
        config: &ProviderConfig,
        stream: bool,
    ) -> Result<reqwest::Response> {
        let api_key = config
            .api_key
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Claude API key is required"))?;

        let request = AnthropicRequest {
            model: &config.model,
            max_tokens: 500,
            messages: vec![AnthropicMessage {
                role: "user",
                content: prompt,
            }],
            stream,
        };

        let response = self
            .client
            .post(&config.api_url)
            .header("x-api-key", api_key.as_str())
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&request)
            .timeout(Duration::from_secs(config.timeout_secs))
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            anyhow::bail!("Claude request failed: {} - {}", status, text);
        }

        Ok(response)
    }
}

#[async_trait]
impl AIProvider for ClaudeProvider {
    async fn generate(&self, prompt: &str, config: &ProviderConfig) -> Result<String> {
        let response = self.send_request(prompt, config, false).await?;
        let api_response: AnthropicResponse = response.json().await?;

        let content = api_response
            .content
            .into_iter()
            .filter_map(|block| block.text)
            .collect::<Vec<_>>()
            .join("");

        Ok(content)
    }

    async fn stream_generate(
        &self,
        prompt: &str,
        config: &ProviderConfig,
    ) -> Result<StreamResponse> {
        let response = self.send_request(prompt, config, true).await?;
        let stream = response.bytes_stream();
        let mapped_stream = map_sse_stream(stream, extract_anthropic_content);

        Ok(Box::pin(mapped_stream))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_claude_provider_creation() {
        let _provider = ClaudeProvider::new();
    }

    #[test]
    fn test_claude_default() {
        let _provider = ClaudeProvider::default();
    }

    #[test]
    fn test_anthropic_request_serialization() {
        let request = AnthropicRequest {
            model: "claude-sonnet-4-20250514",
            max_tokens: 500,
            messages: vec![AnthropicMessage {
                role: "user",
                content: "test prompt",
            }],
            stream: false,
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("claude-sonnet-4-20250514"));
        assert!(json.contains("user"));
        assert!(json.contains("test prompt"));
        assert!(json.contains("500"));
    }

    #[test]
    fn test_anthropic_response_deserialization() {
        let json = r#"{"content": [{"type": "text", "text": "feat(api): 添加功能"}]}"#;
        let response: AnthropicResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.content.len(), 1);
        assert_eq!(
            response.content[0].text.as_ref().unwrap(),
            "feat(api): 添加功能"
        );
    }

    #[test]
    fn test_extract_anthropic_content() {
        let json =
            r#"{"type": "content_block_delta", "delta": {"type": "text_delta", "text": "hello"}}"#;
        assert_eq!(extract_anthropic_content(json), Some("hello".to_string()));

        // Non-delta events should return None
        let json = r#"{"type": "message_start", "delta": null}"#;
        assert_eq!(extract_anthropic_content(json), None);

        // Invalid JSON
        assert_eq!(extract_anthropic_content("invalid"), None);
    }

    #[test]
    fn test_anthropic_stream_event_deserialization() {
        let json =
            r#"{"type": "content_block_delta", "delta": {"type": "text_delta", "text": "world"}}"#;
        let event: AnthropicStreamEvent = serde_json::from_str(json).unwrap();
        assert_eq!(event.event_type, "content_block_delta");
        assert_eq!(event.delta.unwrap().text.unwrap(), "world");
    }
}
