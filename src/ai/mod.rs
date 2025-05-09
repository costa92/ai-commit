use reqwest::Client;
use serde::{Deserialize, Serialize};

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
    #[allow(dead_code)]
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
    _diff: &str,
    config: &crate::config::Config,
    prompt: &str,
) -> anyhow::Result<String> {
    let client = Client::new();
    match config.provider.as_str() {
        "deepseek" => {
            let request = DeepseekRequest {
                model: &config.model,
                messages: vec![DeepseekMessage {
                    role: "user",
                    content: prompt,
                }],
                stream: false,
            };
            let res = client
                .post(&config.deepseek_url)
                .bearer_auth(config.deepseek_api_key.as_ref().unwrap()) // 已经通过 validate 验证
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
            let request = OllamaRequest {
                model: &config.model,
                prompt,
                stream: false,
            };
            let res = client
                .post(&config.ollama_url)
                .json(&request)
                .send()
                .await?;
            let body: OllamaResponse = res.json().await?;
            Ok(body.response.trim().to_string())
        }
    }
}
pub mod prompt;
