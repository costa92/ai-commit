use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Serialize)]
pub struct OllamaRequest<'a> {
    pub model: &'a str,
    pub prompt: &'a str,
    pub stream: bool,
}

#[derive(Serialize)]
pub struct DeepseekRequest<'a> {
    pub model: &'a str,
    pub messages: Vec<DeepseekMessage<'a>>,
    pub stream: bool,
}

#[derive(Serialize)]
pub struct DeepseekMessage<'a> {
    pub role: &'a str,
    pub content: &'a str,
}

#[derive(Deserialize)]
pub struct OllamaResponse {
    pub response: String,
    pub done: bool,
}

#[derive(Deserialize)]
pub struct DeepseekResponse {
    pub choices: Vec<DeepseekChoice>,
}

#[derive(Deserialize)]
pub struct DeepseekChoice {
    pub message: DeepseekChoiceMessage,
}

#[derive(Deserialize)]
pub struct DeepseekChoiceMessage {
    pub content: String,
}

pub async fn generate_commit_message(
    diff: &str,
    provider: &str,
    model: &str,
    prompt: &str,
) -> anyhow::Result<String> {
    let client = Client::new();
    match provider {
        "deepseek" => {
            let url = env::var("AI_COMMIT_DEEPSEEK_URL")
                .unwrap_or_else(|_| "https://api.deepseek.com/v1/chat/completions".to_string());
            let api_key = env::var("AI_COMMIT_DEEPSEEK_API_KEY").unwrap_or_default();
            let request = DeepseekRequest {
                model,
                messages: vec![DeepseekMessage {
                    role: "user",
                    content: prompt,
                }],
                stream: false,
            };
            let res = client
                .post(url)
                .bearer_auth(api_key)
                .json(&request)
                .send()
                .await?;
            let body: DeepseekResponse = res.json().await?;
            let content = body
                .choices
                .get(0)
                .map(|c| c.message.content.trim())
                .unwrap_or("");
            Ok(content.to_string())
        }
        _ => {
            // 默认 ollama
            let url = env::var("AI_COMMIT_OLLAMA_URL")
                .unwrap_or_else(|_| "http://localhost:11434/api/generate".to_string());
            let request = OllamaRequest {
                model,
                prompt,
                stream: false,
            };
            let res = client.post(url).json(&request).send().await?;
            let body: OllamaResponse = res.json().await?;
            Ok(body.response.trim().to_string())
        }
    }
}
