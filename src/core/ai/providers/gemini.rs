use crate::core::ai::http::shared_client;
use crate::core::ai::provider::{AIProvider, ProviderConfig, StreamResponse};
use crate::core::ai::stream::map_sse_stream;
use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Google Generative AI 请求
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GeminiRequest<'a> {
    contents: Vec<GeminiContent<'a>>,
    generation_config: GeminiGenerationConfig,
}

/// Gemini 内容
#[derive(Serialize)]
struct GeminiContent<'a> {
    parts: Vec<GeminiPart<'a>>,
}

/// Gemini 部分
#[derive(Serialize)]
struct GeminiPart<'a> {
    text: &'a str,
}

/// Gemini 生成配置
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GeminiGenerationConfig {
    temperature: f32,
    max_output_tokens: i32,
}

/// Gemini 响应
#[derive(Deserialize)]
struct GeminiResponse {
    candidates: Vec<GeminiCandidate>,
}

/// Gemini 候选
#[derive(Deserialize)]
struct GeminiCandidate {
    content: GeminiContentResponse,
}

/// Gemini 内容响应
#[derive(Deserialize)]
struct GeminiContentResponse {
    parts: Vec<GeminiPartResponse>,
}

/// Gemini 部分响应
#[derive(Deserialize)]
struct GeminiPartResponse {
    text: Option<String>,
}

/// 从 Gemini SSE JSON 中提取内容
fn extract_gemini_content(json_str: &str) -> Option<String> {
    serde_json::from_str::<GeminiResponse>(json_str)
        .ok()
        .and_then(|r| r.candidates.into_iter().next())
        .and_then(|c| c.content.parts.into_iter().next())
        .and_then(|p| p.text)
}

/// Gemini (Google) AI 提供商
///
/// 使用 Google Generative AI API，model 嵌入 URL 路径。
/// 默认 URL 前缀: https://generativelanguage.googleapis.com/v1beta
/// 默认 model: gemini-2.0-flash
/// 环境变量: AI_COMMIT_GEMINI_API_KEY
pub struct GeminiProvider {
    client: &'static reqwest::Client,
}

impl Default for GeminiProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl GeminiProvider {
    pub fn new() -> Self {
        Self {
            client: shared_client(),
        }
    }

    /// 构建 Gemini API URL
    ///
    /// URL 格式: {base_url}/models/{model}:{action}?key={api_key}
    fn build_url(&self, config: &ProviderConfig, stream: bool) -> Result<String> {
        let api_key = config
            .api_key
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Gemini API key is required"))?;

        let action = if stream {
            "streamGenerateContent?alt=sse"
        } else {
            "generateContent"
        };

        Ok(format!(
            "{}/models/{}:{}{}key={}",
            config.api_url.trim_end_matches('/'),
            config.model,
            action,
            if stream { "&" } else { "?" },
            api_key
        ))
    }

    async fn send_request(
        &self,
        prompt: &str,
        config: &ProviderConfig,
        stream: bool,
    ) -> Result<reqwest::Response> {
        let url = self.build_url(config, stream)?;

        let request = GeminiRequest {
            contents: vec![GeminiContent {
                parts: vec![GeminiPart { text: prompt }],
            }],
            generation_config: GeminiGenerationConfig {
                temperature: 0.7,
                max_output_tokens: 500,
            },
        };

        let response = self
            .client
            .post(&url)
            .header("content-type", "application/json")
            .json(&request)
            .timeout(Duration::from_secs(config.timeout_secs))
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            anyhow::bail!("Gemini request failed: {} - {}", status, text);
        }

        Ok(response)
    }
}

#[async_trait]
impl AIProvider for GeminiProvider {
    async fn generate(&self, prompt: &str, config: &ProviderConfig) -> Result<String> {
        let response = self.send_request(prompt, config, false).await?;
        let api_response: GeminiResponse = response.json().await?;

        let content = api_response
            .candidates
            .into_iter()
            .next()
            .and_then(|c| c.content.parts.into_iter().next())
            .and_then(|p| p.text)
            .unwrap_or_default();

        Ok(content)
    }

    async fn stream_generate(
        &self,
        prompt: &str,
        config: &ProviderConfig,
    ) -> Result<StreamResponse> {
        let response = self.send_request(prompt, config, true).await?;
        let stream = response.bytes_stream();
        let mapped_stream = map_sse_stream(stream, extract_gemini_content);

        Ok(Box::pin(mapped_stream))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gemini_provider_creation() {
        let _provider = GeminiProvider::new();
    }

    #[test]
    fn test_gemini_default() {
        let _provider = GeminiProvider::default();
    }

    #[test]
    fn test_gemini_request_serialization() {
        let request = GeminiRequest {
            contents: vec![GeminiContent {
                parts: vec![GeminiPart {
                    text: "test prompt",
                }],
            }],
            generation_config: GeminiGenerationConfig {
                temperature: 0.7,
                max_output_tokens: 500,
            },
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("test prompt"));
        assert!(json.contains("generationConfig"));
        assert!(json.contains("maxOutputTokens"));
    }

    #[test]
    fn test_gemini_response_deserialization() {
        let json =
            r#"{"candidates": [{"content": {"parts": [{"text": "feat(core): 添加功能"}]}}]}"#;
        let response: GeminiResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.candidates.len(), 1);
        assert_eq!(
            response.candidates[0].content.parts[0]
                .text
                .as_ref()
                .unwrap(),
            "feat(core): 添加功能"
        );
    }

    #[test]
    fn test_extract_gemini_content() {
        let json = r#"{"candidates": [{"content": {"parts": [{"text": "hello"}]}}]}"#;
        assert_eq!(extract_gemini_content(json), Some("hello".to_string()));
        assert_eq!(extract_gemini_content("invalid"), None);
    }

    #[test]
    fn test_build_url_non_stream() {
        let provider = GeminiProvider::new();
        let config = ProviderConfig {
            model: "gemini-2.0-flash".to_string(),
            api_key: Some("test-key".to_string()),
            api_url: "https://generativelanguage.googleapis.com/v1beta".to_string(),
            timeout_secs: 30,
            max_retries: 3,
            stream: false,
        };

        let url = provider.build_url(&config, false).unwrap();
        assert!(url.contains("/models/gemini-2.0-flash:generateContent"));
        assert!(url.contains("key=test-key"));
        assert!(!url.contains("alt=sse"));
    }

    #[test]
    fn test_build_url_stream() {
        let provider = GeminiProvider::new();
        let config = ProviderConfig {
            model: "gemini-2.0-flash".to_string(),
            api_key: Some("test-key".to_string()),
            api_url: "https://generativelanguage.googleapis.com/v1beta".to_string(),
            timeout_secs: 30,
            max_retries: 3,
            stream: true,
        };

        let url = provider.build_url(&config, true).unwrap();
        assert!(url.contains("/models/gemini-2.0-flash:streamGenerateContent"));
        assert!(url.contains("alt=sse"));
        assert!(url.contains("key=test-key"));
    }

    #[test]
    fn test_build_url_no_api_key() {
        let provider = GeminiProvider::new();
        let config = ProviderConfig {
            model: "gemini-2.0-flash".to_string(),
            api_key: None,
            api_url: "https://generativelanguage.googleapis.com/v1beta".to_string(),
            timeout_secs: 30,
            max_retries: 3,
            stream: false,
        };

        assert!(provider.build_url(&config, false).is_err());
    }
}
