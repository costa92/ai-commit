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
                .bearer_auth(config.deepseek_api_key.as_ref().unwrap())
                .json(&request)
                .send()
                .await;
            let res = match res {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("[ai-commit] Deepseek 请求失败: {e:?}");
                    anyhow::bail!("Deepseek 请求失败: {e}");
                }
            };
            if !res.status().is_success() {
                let status = res.status();
                let text = res.text().await.unwrap_or_default();
                eprintln!("[ai-commit] Deepseek 响应错误: 状态码 {status}, 响应体: {text}");
                anyhow::bail!("Deepseek 响应错误: 状态码 {status}, 响应体: {text}");
            }
            let body: DeepseekResponse = match res.json().await {
                Ok(b) => b,
                Err(e) => {
                    eprintln!("[ai-commit] Deepseek 响应解析失败: {e:?}");
                    anyhow::bail!("Deepseek 响应解析失败: {e}");
                }
            };
            let content = body
                .choices
                .get(0)
                .map(|c| c.message.content.trim())
                .unwrap_or("");
            if content.contains("{{git_diff}}") || content.contains("Conventional Commits") {
                anyhow::bail!("AI 服务未返回有效 commit message，请检查 AI 服务配置或网络连接。");
            }
            Ok(content.to_string())
        }
        _ => {
            let request = OllamaRequest {
                model: &config.model,
                prompt,
                stream: false,
            };
            let res = client.post(&config.ollama_url).json(&request).send().await;
            let res = match res {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("[ai-commit] Ollama 请求失败: {e:?}");
                    anyhow::bail!("Ollama 请求失败: {e}");
                }
            };
            if !res.status().is_success() {
                let status = res.status();
                let text = res.text().await.unwrap_or_default();
                eprintln!("[ai-commit] Ollama 响应错误: 状态码 {status}, 响应体: {text}");
                anyhow::bail!("Ollama 响应错误: 状态码 {status}, 响应体: {text}");
            }
            let body: OllamaResponse = match res.json().await {
                Ok(b) => b,
                Err(e) => {
                    eprintln!("[ai-commit] Ollama 响应解析失败: {e:?}");
                    anyhow::bail!("Ollama 响应解析失败: {e}");
                }
            };
            let response = body.response.trim();
            if response.contains("{{git_diff}}") || response.contains("Conventional Commits") {
                anyhow::bail!("AI 服务未返回有效 commit message，请检查 AI 服务配置或网络连接。");
            }
            Ok(response.to_string())
        }
    }
}
pub mod prompt;
