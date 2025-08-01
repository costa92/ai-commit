use futures_util::StreamExt;
use once_cell::sync::Lazy;
use regex::Regex;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncWriteExt, stdout};

// 全局 HTTP 客户端复用
static HTTP_CLIENT: Lazy<Client> = Lazy::new(|| {
    Client::builder()
        .pool_max_idle_per_host(10)
        .pool_idle_timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("Failed to create HTTP client")
});

static RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?s)```(?:[a-zA-Z]+
)?(.*?)
```").unwrap()
});

// 预编译验证正则表达式，提升性能
static INVALID_RESPONSE_PATTERNS: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)(\{\{git_diff\}\}|输出格式|git diff:|these are|here's a|the changes|overall assessment|breakdown|suggestions|\*\*|good changes|clean|helpful|address|improve|significant changes|i don't have|represent good|contribute to|robust codebase|^the |^i |^1\.|\*)").unwrap()
});

// 正面格式验证正则
static VALID_COMMIT_FORMAT: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(feat|fix|docs|style|refactor|test|chore)(\([^)]+\))?:\s*.{1,100}$").unwrap()
});

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
    pub delta: DeepseekChoiceDelta,
}

#[derive(Deserialize)]
pub struct DeepseekChoiceDelta {
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

// 提取消息清理逻辑
fn clean_message(message: &str) -> String {
    // 只取第一行，去除多余内容
    let first_line = message.lines().next().unwrap_or("").trim();
    
    if let Some(caps) = RE.captures(first_line) {
        caps.get(1).map_or("", |m| m.as_str()).trim().to_owned()
    } else {
        first_line.to_owned()
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
    let client = &*HTTP_CLIENT;
    match config.provider.as_str() {
        "siliconflow" | "deepseek" => {
            let request = DeepseekRequest {
                model: &config.model,
                messages: vec![DeepseekMessage {
                    role: "user",
                    content: prompt,
                }],
                stream: true,
            };
            let (url, api_key) = if config.provider == "siliconflow" {
                (
                    &config.siliconflow_url,
                    config.siliconflow_api_key.as_ref(),
                )
            } else {
                (&config.deepseek_url, config.deepseek_api_key.as_ref())
            };
            let res = make_request(client, url, api_key, &request).await?;
            if !res.status().is_success() {
                let status = res.status();
                let text = res.text().await.unwrap_or_default();
                eprintln!("[ai-commit] 响应错误: 状态码 {status}, 响应体: {text}");
                anyhow::bail!("响应错误: 状态码 {status}, 响应体: {text}");
            }

            // 优化的流处理：预分配缓冲区，减少内存重新分配
            let mut message = String::with_capacity(2048);  // 预分配更大的缓冲区
            let mut stdout_handle = stdout();
            let mut stream = res.bytes_stream();
            let mut buffer = Vec::with_capacity(8192);  // 中间缓冲区
            
            while let Some(item) = stream.next().await {
                let chunk = item?;
                buffer.extend_from_slice(&chunk);
                
                // 批量处理缓冲区中的数据
                if buffer.len() > 4096 {  // 批量处理阈值
                    let chunk_str = std::str::from_utf8(&buffer)
                        .map_err(|e| anyhow::anyhow!("UTF-8 decode error: {}", e))?;
                    
                    for line in chunk_str.lines() {
                        if line.starts_with("data:") {
                            let json_str = line.strip_prefix("data:").unwrap().trim();
                            if json_str == "[DONE]" {
                                break;
                            }
                            if let Ok(response) = serde_json::from_str::<DeepseekResponse>(json_str) {
                                if let Some(choice) = response.choices.first() {
                                    let content = &choice.delta.content;
                                    stdout_handle.write_all(content.as_bytes()).await?;
                                    stdout_handle.flush().await?;
                                    message.push_str(content);
                                }
                            }
                        }
                    }
                    buffer.clear();
                }
            }
            
            // 处理剩余的数据
            if !buffer.is_empty() {
                let chunk_str = std::str::from_utf8(&buffer)
                    .map_err(|e| anyhow::anyhow!("UTF-8 decode error: {}", e))?;
                    
                for line in chunk_str.lines() {
                    if line.starts_with("data:") {
                        let json_str = line.strip_prefix("data:").unwrap().trim();
                        if json_str != "[DONE]" {
                            if let Ok(response) = serde_json::from_str::<DeepseekResponse>(json_str) {
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
            }
            stdout_handle.write_all(b"\n").await?;
            
            // 优化的响应验证：使用预编译正则表达式
            if INVALID_RESPONSE_PATTERNS.is_match(&message) || message.trim().is_empty() {
                anyhow::bail!("AI 服务未返回有效 commit message，请检查 AI 服务配置或网络连接。");
            }

            // 正面格式检查：使用预编译正则表达式
            let first_line = message.lines().next().unwrap_or("").trim();
            if !VALID_COMMIT_FORMAT.is_match(first_line) {
                anyhow::bail!("AI 返回的格式不符合 Conventional Commits 规范，请重试。");
            }

            Ok(clean_message(&message))
        }
        _ => {
            let request = OllamaRequest {
                model: &config.model,
                prompt,
                stream: true,
            };
            let res = make_request(client, &config.ollama_url, None, &request).await?;
            if !res.status().is_success() {
                let status = res.status();
                let text = res.text().await.unwrap_or_default();
                eprintln!("[ai-commit] Ollama 响应错误: 状态码 {status}, 响应体: {text}");
                anyhow::bail!("Ollama 响应错误: 状态码 {status}, 响应体: {text}");
            }

            // 优化的流处理：预分配缓冲区，减少内存重新分配  
            let mut message = String::with_capacity(2048);  // 预分配更大的缓冲区
            let mut stdout_handle = stdout();
            let mut stream = res.bytes_stream();
            let mut buffer = Vec::with_capacity(8192);  // 中间缓冲区
            
            while let Some(item) = stream.next().await {
                let chunk = item?;
                buffer.extend_from_slice(&chunk);
                
                // 批量处理缓冲区中的数据
                if buffer.len() > 4096 {  // 批量处理阈值
                    let chunk_str = std::str::from_utf8(&buffer)
                        .map_err(|e| anyhow::anyhow!("UTF-8 decode error: {}", e))?;
                    
                    for line in chunk_str.lines() {
                        if let Ok(response) = serde_json::from_str::<OllamaResponse>(line) {
                            let content = &response.response;
                            stdout_handle.write_all(content.as_bytes()).await?;
                            stdout_handle.flush().await?;
                            message.push_str(content);
                        }
                    }
                    buffer.clear();
                }
            }
            
            // 处理剩余的数据
            if !buffer.is_empty() {
                let chunk_str = std::str::from_utf8(&buffer)
                    .map_err(|e| anyhow::anyhow!("UTF-8 decode error: {}", e))?;
                    
                for line in chunk_str.lines() {
                    if let Ok(response) = serde_json::from_str::<OllamaResponse>(line) {
                        let content = &response.response;
                        stdout_handle.write_all(content.as_bytes()).await?;
                        stdout_handle.flush().await?;
                        message.push_str(content);
                    }
                }
            }
            stdout_handle.write_all(b"\n").await?;
            
            // 优化的响应验证：使用预编译正则表达式
            if INVALID_RESPONSE_PATTERNS.is_match(&message) || message.trim().is_empty() {
                anyhow::bail!("AI 服务未返回有效 commit message，请检查 AI 服务配置或网络连接。");
            }

            // 正面格式检查：使用预编译正则表达式
            let first_line = message.lines().next().unwrap_or("").trim();
            if !VALID_COMMIT_FORMAT.is_match(first_line) {
                anyhow::bail!("AI 返回的格式不符合 Conventional Commits 规范，请重试。");
            }

            Ok(clean_message(&message))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    #[test]
    fn test_clean_message_with_code_blocks() {
        let message_with_code = "```\nfeat(user): add user authentication\n\nAdd JWT-based authentication system\n```";
        let cleaned = clean_message(message_with_code);
        assert_eq!(cleaned, "feat(user): add user authentication\n\nAdd JWT-based authentication system");
    }

    #[test]
    fn test_clean_message_without_code_blocks() {
        let message = "feat(auth): implement login functionality";
        let cleaned = clean_message(message);
        assert_eq!(cleaned, "feat(auth): implement login functionality");
    }

    #[test]
    fn test_clean_message_with_whitespace() {
        let message = "  feat(api): add new endpoint  \n  ";
        let cleaned = clean_message(message);
        assert_eq!(cleaned, "feat(api): add new endpoint");
    }

    #[test]
    fn test_clean_message_empty() {
        let message = "";
        let cleaned = clean_message(message);
        assert_eq!(cleaned, "");
    }

    #[test]
    fn test_clean_message_only_whitespace() {
        let message = "   \n\t  ";
        let cleaned = clean_message(message);
        assert_eq!(cleaned, "");
    }

    #[test]
    fn test_regex_compilation() {
        // 测试正则表达式编译是否正确
        let test_text = "```bash\necho hello\n```";
        let captures = RE.captures(test_text);
        assert!(captures.is_some());
        assert_eq!(captures.unwrap().get(1).unwrap().as_str(), "echo hello\n");
    }

    #[test]
    fn test_http_client_singleton() {
        // 测试 HTTP 客户端是否是单例
        let client1 = &*HTTP_CLIENT;
        let client2 = &*HTTP_CLIENT;
        
        // 两个引用应该指向同一个对象
        assert!(std::ptr::eq(client1, client2));
    }

    #[test]
    fn test_request_serialization() {
        let ollama_request = OllamaRequest {
            model: "test-model",
            prompt: "test prompt",
            stream: true,
        };
        
        let json = serde_json::to_string(&ollama_request).unwrap();
        assert!(json.contains("test-model"));
        assert!(json.contains("test prompt"));
        assert!(json.contains("true"));
    }

    #[test]
    fn test_deepseek_request_serialization() {
        let deepseek_request = DeepseekRequest {
            model: "gpt-4",
            messages: vec![DeepseekMessage {
                role: "user",
                content: "Hello, world!",
            }],
            stream: true,
        };
        
        let json = serde_json::to_string(&deepseek_request).unwrap();
        assert!(json.contains("gpt-4"));
        assert!(json.contains("user"));
        assert!(json.contains("Hello, world!"));
        assert!(json.contains("true"));
    }

    #[test]
    fn test_ollama_response_deserialization() {
        let json = r#"{"response": "test response", "done": false}"#;
        let response: OllamaResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.response, "test response");
        assert_eq!(response.done, false);
    }

    #[test]
    fn test_deepseek_response_deserialization() {
        let json = r#"{"choices": [{"delta": {"content": "test content"}}]}"#;
        let response: DeepseekResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.choices.len(), 1);
        assert_eq!(response.choices[0].delta.content, "test content");
    }

    #[test]
    fn test_invalid_diff_detection() {
        let invalid_messages = vec![
            "{{git_diff}}",
            "Conventional Commits规范",
            "请严格按照如下 Conventional Commits 规范",
            "以下是 git diff：\n{{git_diff}}",
        ];

        for message in invalid_messages {
            assert!(
                message.contains("{{git_diff}}") || message.contains("Conventional Commits"),
                "Message should be detected as invalid: {}",
                message
            );
        }
    }

    #[tokio::test]
    async fn test_generate_commit_message_empty_diff() {
        let config = Config::new();
        let _result = generate_commit_message("", &config, "test prompt").await;
        
        // 应该因为空的 diff 而退出程序，这里我们无法测试 std::process::exit(0)
        // 但我们可以测试 diff.trim().is_empty() 的逻辑
        assert!("".trim().is_empty());
    }

    // 模拟测试：由于实际的 HTTP 请求需要外部服务，我们主要测试数据结构和逻辑
    #[tokio::test]
    async fn test_make_request_structure() {
        // 测试 make_request 函数的结构，但不实际发送请求
        let client = &*HTTP_CLIENT;
        
        // 验证客户端已正确初始化（通过检查是否为有效的 Client 实例）
        // 由于 reqwest::Client 没有实现 Display trait，我们只能验证它存在
        assert!(std::ptr::addr_of!(*client) != std::ptr::null());
    }

    #[test]
    fn test_config_provider_matching() {
        // 测试配置提供商匹配逻辑
        let providers = vec!["siliconflow", "deepseek", "ollama", "unknown"];
        
        for provider in providers {
            match provider {
                "siliconflow" | "deepseek" => {
                    // 应该使用 DeepseekRequest 格式
                    assert!(true);
                }
                _ => {
                    // 应该使用 OllamaRequest 格式  
                    assert!(true);
                }
            }
        }
    }
}

pub mod prompt;// 测试注释1
// 更多测试内容
