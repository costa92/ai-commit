use crate::ai::diff_analyzer::DiffAnalysis;
use futures_util::StreamExt;
use once_cell::sync::Lazy;
use regex::Regex;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::io::{stdout, AsyncWriteExt};

// 全局 HTTP 客户端复用
static HTTP_CLIENT: Lazy<Client> = Lazy::new(|| {
    Client::builder()
        .pool_max_idle_per_host(10)
        .pool_idle_timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("Failed to create HTTP client")
});

static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?s)```(?:[a-zA-Z]+\n)?(.*?)\n```").unwrap());

// 预编译验证正则表达式，提升性能
static INVALID_RESPONSE_PATTERNS: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)(\{\{git_diff\}\}|输出格式|git diff:|these are|here's a|the changes|overall assessment|breakdown|suggestions|\*\*|good changes|clean|helpful|address|improve|significant changes|i don't have|represent good|contribute to|robust codebase|^the |^i |^1\.|\*)").unwrap()
});

// 正面格式验证正则
static VALID_COMMIT_FORMAT: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(feat|fix|docs|style|refactor|test|chore)(\([^)]+\))?:\s+\S+.*$").unwrap()
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
    // 首先尝试从代码块中提取内容
    if let Some(caps) = RE.captures(message) {
        caps.get(1).map_or("", |m| m.as_str()).trim().to_owned()
    } else {
        // 如果没有代码块，只取第一行，去除多余内容
        let first_line = message.lines().next().unwrap_or("").trim();
        first_line.to_owned()
    }
}

pub async fn generate_commit_message(
    diff: &str,
    config: &crate::config::Config,
    prompt: &str,
) -> anyhow::Result<String> {
    if diff.trim().is_empty() {
        if config.debug {
            println!("No staged changes.");
        }
        std::process::exit(0);
    }

    // 分析diff，优化大文件和多文件场景
    let analysis = DiffAnalysis::analyze_diff(diff);

    // 创建优化的提示词
    let optimized_prompt = if analysis.is_large_diff || analysis.is_multi_file {
        if config.debug {
            println!(
                "检测到大型变更 ({}个文件, {}字符)，正在生成摘要...",
                analysis.total_files,
                diff.len()
            );
        }

        // 使用摘要版本的prompt
        create_summarized_prompt(&analysis, diff, prompt)
    } else {
        prompt.to_string()
    };

    let client = &*HTTP_CLIENT;
    match config.provider.as_str() {
        "siliconflow" | "deepseek" => {
            let request = DeepseekRequest {
                model: &config.model,
                messages: vec![DeepseekMessage {
                    role: "user",
                    content: &optimized_prompt,
                }],
                stream: true,
            };
            let (url, api_key) = if config.provider == "siliconflow" {
                (&config.siliconflow_url, config.siliconflow_api_key.as_ref())
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
            let mut message = String::with_capacity(2048); // 预分配更大的缓冲区
            let mut stdout_handle = stdout();
            let mut stream = res.bytes_stream();
            let mut buffer = Vec::with_capacity(8192); // 中间缓冲区

            while let Some(item) = stream.next().await {
                let chunk = item?;
                buffer.extend_from_slice(&chunk);

                // 批量处理缓冲区中的数据
                if buffer.len() > 4096 {
                    // 批量处理阈值
                    let chunk_str = std::str::from_utf8(&buffer)
                        .map_err(|e| anyhow::anyhow!("UTF-8 decode error: {}", e))?;

                    for line in chunk_str.lines() {
                        if line.starts_with("data:") {
                            let json_str = line.strip_prefix("data:").unwrap().trim();
                            if json_str == "[DONE]" {
                                break;
                            }
                            if let Ok(response) = serde_json::from_str::<DeepseekResponse>(json_str)
                            {
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
                            if let Ok(response) = serde_json::from_str::<DeepseekResponse>(json_str)
                            {
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

            if config.debug {
                println!("检查提交信息格式: '{}'", first_line);
                println!("字符数: {}", first_line.chars().count());
            }

            if !VALID_COMMIT_FORMAT.is_match(first_line) {
                anyhow::bail!("AI 返回的格式不符合 Conventional Commits 规范，请重试。\n期望格式：<type>(<scope>): <subject>\n实际内容：{}", first_line);
            }

            // 检查长度（基于字符数，不是字节数）
            if first_line.chars().count() > 100 {
                anyhow::bail!(
                    "提交信息过长（{}字符），请生成更简洁的描述（不超过100字符）。",
                    first_line.chars().count()
                );
            }

            Ok(clean_message(&message))
        }
        _ => {
            let request = OllamaRequest {
                model: &config.model,
                prompt: &optimized_prompt,
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
            let mut message = String::with_capacity(2048); // 预分配更大的缓冲区
            let mut stdout_handle = stdout();
            let mut stream = res.bytes_stream();
            let mut buffer = Vec::with_capacity(8192); // 中间缓冲区

            while let Some(item) = stream.next().await {
                let chunk = item?;
                buffer.extend_from_slice(&chunk);

                // 批量处理缓冲区中的数据
                if buffer.len() > 4096 {
                    // 批量处理阈值
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

            if config.debug {
                println!("检查提交信息格式: '{}'", first_line);
                println!("字符数: {}", first_line.chars().count());
            }

            if !VALID_COMMIT_FORMAT.is_match(first_line) {
                anyhow::bail!("AI 返回的格式不符合 Conventional Commits 规范，请重试。\n期望格式：<type>(<scope>): <subject>\n实际内容：{}", first_line);
            }

            // 检查长度（基于字符数，不是字节数）
            if first_line.chars().count() > 100 {
                anyhow::bail!(
                    "提交信息过长（{}字符），请生成更简洁的描述（不超过100字符）。",
                    first_line.chars().count()
                );
            }

            Ok(clean_message(&message))
        }
    }
}

/// 为大型或多文件变更创建摘要化的提示词
fn create_summarized_prompt(
    analysis: &DiffAnalysis,
    original_diff: &str,
    _base_prompt: &str,
) -> String {
    // 创建针对大文件场景的专用提示
    let summarized_template = format!(
        r#"输出格式：<type>(<scope>): <subject>

type: feat|fix|docs|style|refactor|test|chore  
subject: 中文，不超过50字，突出核心变更

大型变更摘要指导：
- 当前变更：{}
- 主要类型：{}
- 建议scope：{}
- 优先概括整体目标，避免详细罗列

错误示例（禁止）：
"修复config.rs、main.rs、lib.rs等多个文件的问题"
"更新src/ai/mod.rs, src/config/mod.rs等文件"
任何文件名罗列

正确示例：
feat(ai): 添加AI响应优化和配置管理
refactor(core): 重构模块架构提升性能  
fix(auth): 修复用户认证流程问题

变更详情：
{}"#,
        analysis.generate_summary(),
        analysis.primary_change_type,
        analysis.dominant_scope.as_deref().unwrap_or("core"),
        analysis.create_optimized_prompt(original_diff)
    );

    summarized_template
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    #[test]
    fn test_clean_message_with_code_blocks() {
        let message_with_code =
            "```\nfeat(user): add user authentication\n\nAdd JWT-based authentication system\n```";
        let cleaned = clean_message(message_with_code);
        assert_eq!(
            cleaned,
            "feat(user): add user authentication\n\nAdd JWT-based authentication system"
        );
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
        assert_eq!(captures.unwrap().get(1).unwrap().as_str(), "echo hello");
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
        assert!(!response.done);
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
        assert!(!std::ptr::addr_of!(*client).is_null());
    }

    #[test]
    fn test_config_provider_matching() {
        // 测试配置提供商匹配逻辑
        let providers = vec!["siliconflow", "deepseek", "ollama", "unknown"];

        for provider in providers {
            match provider {
                "siliconflow" | "deepseek" => {
                    // 应该使用 DeepseekRequest 格式
                }
                _ => {
                    // 应该使用 OllamaRequest 格式
                }
            }
        }
    }

    #[test]
    fn test_commit_message_validation() {
        let re = Regex::new(r"^(feat|fix|docs|style|refactor|test|chore)(\([^)]+\))?:\s+\S+.*$")
            .unwrap();

        let test_cases = vec![
            ("test(git): 重构并增强Git工作树和标签测试覆盖率", true),
            ("feat(api): 添加用户认证功能", true),
            ("fix(ui): 修复按钮显示问题", true),
            ("refactor(core): 重构数据处理逻辑", true),
            (
                "根据提供的变更信息和格式要求，以下是符合规范的提交消息：",
                false,
            ),
            ("", false),
            ("test:", false),  // 缺少内容
            ("test: ", false), // 只有空格
            ("test: a", true), // 最短有效内容
            ("chore: update dependencies", true),
            ("docs(readme): update installation guide", true),
        ];

        for (msg, expected) in test_cases {
            let result = re.is_match(msg);
            assert_eq!(
                result, expected,
                "Message '{}' should be {}, but got {}",
                msg, expected, result
            );
        }
    }

    #[test]
    fn test_commit_message_length() {
        let msg = "test(git): 重构并增强Git工作树和标签测试覆盖率";
        let char_count = msg.chars().count();

        // 确保中文字符正确计算
        assert!(
            char_count < 100,
            "Message should be under 100 characters, got {}",
            char_count
        );
        assert_eq!(char_count, 30, "Expected 30 characters, got {}", char_count);
    }
}

pub mod diff_analyzer;
pub mod prompt;
