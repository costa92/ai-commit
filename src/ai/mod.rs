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

async fn make_request<T: Serialize>(
    client: &Client,
    url: &str,
    api_key: Option<&String>,
    request: &T,
) -> anyhow::Result<reqwest::Response> {
    let mut builder = client.post(url);
    if let Some(key) = api_key {
        builder = builder.bearer_auth(key);
    }
    let res = builder.json(request).send().await;
    match res {
        Ok(r) => Ok(r),
        Err(e) => {
            eprintln!("[ai-commit] 请求失败: {e:?}");
            anyhow::bail!("请求失败: {e}");
        }
    }
}

pub async fn generate_commit_message(
    diff: &str,
    config: &crate::config::Config,
    prompt: &str,
) -> anyhow::Result<String> {
    if diff.trim().is_empty() {
        println!("No staged changes.");
        std::process::exit(0);
    }
    let client = Client::new();
    match config.provider.as_str() {
        "siliconflow" | "deepseek" => {
            let request = DeepseekRequest {
                model: &config.model,
                messages: vec![DeepseekMessage {
                    role: "user",
                    content: prompt,
                }],
                stream: false,
            };
            let (url, api_key) = if config.provider == "siliconflow" {
                (
                    &config.siliconflow_url,
                    config.siliconflow_api_key.as_ref(),
                )
            } else {
                (&config.deepseek_url, config.deepseek_api_key.as_ref())
            };
            let res = make_request(&client, url, api_key, &request).await?;
            if !res.status().is_success() {
                let status = res.status();
                let text = res.text().await.unwrap_or_default();
                eprintln!("[ai-commit] 响应错误: 状态码 {status}, 响应体: {text}");
                anyhow::bail!("响应错误: 状态码 {status}, 响应体: {text}");
            }
            let body: DeepseekResponse = match res.json().await {
                Ok(b) => b,
                Err(e) => {
                    eprintln!("[ai-commit] 响应解析失败: {e:?}");
                    anyhow::bail!("响应解析失败: {e}");
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
            let res = make_request(&client, &config.ollama_url, None, &request).await?;
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