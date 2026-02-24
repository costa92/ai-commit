use anyhow::Result;
use futures_util::StreamExt;
use tokio::io::{stdout, AsyncWriteExt};

/// 处理 SSE (Server-Sent Events) 流式响应
///
/// 通用 SSE 流解析器，适用于 Deepseek/SiliconFlow/Kimi 等 OpenAI 兼容 API。
/// `extract_content` 闭包从每行 JSON 中提取文本内容。
#[allow(dead_code)]
pub async fn process_sse_stream<F>(
    response: reqwest::Response,
    extract_content: F,
) -> Result<String>
where
    F: Fn(&str) -> Option<String>,
{
    let mut message = String::with_capacity(2048);
    let mut stdout_handle = stdout();
    let mut stream = response.bytes_stream();

    while let Some(item) = stream.next().await {
        let chunk = item?;
        let chunk_str = std::str::from_utf8(&chunk)?;

        for line in chunk_str.lines() {
            if line.starts_with("data:") {
                let json_str = line.strip_prefix("data:").unwrap().trim();
                if json_str == "[DONE]" {
                    break;
                }

                if let Some(content) = extract_content(json_str) {
                    stdout_handle.write_all(content.as_bytes()).await?;
                    stdout_handle.flush().await?;
                    message.push_str(&content);
                }
            }
        }
    }

    stdout_handle.write_all(b"\n").await?;
    Ok(message)
}

/// 处理 JSONL (JSON Lines) 流式响应
///
/// 通用 JSONL 流解析器，适用于 Ollama 等使用逐行 JSON 的 API。
/// `extract_content` 闭包从每行 JSON 中提取文本内容。
/// 返回 `(content, done)` 其中 `done` 表示是否结束。
#[allow(dead_code)]
pub async fn process_jsonl_stream<F>(
    response: reqwest::Response,
    extract_content: F,
) -> Result<String>
where
    F: Fn(&str) -> Option<(String, bool)>,
{
    let mut message = String::with_capacity(2048);
    let mut stdout_handle = stdout();
    let mut stream = response.bytes_stream();

    while let Some(item) = stream.next().await {
        let chunk = item?;
        let chunk_str = std::str::from_utf8(&chunk)?;

        for line in chunk_str.lines() {
            if let Some((content, done)) = extract_content(line) {
                stdout_handle.write_all(content.as_bytes()).await?;
                stdout_handle.flush().await?;
                message.push_str(&content);

                if done {
                    break;
                }
            }
        }
    }

    stdout_handle.write_all(b"\n").await?;
    Ok(message)
}

/// SSE 流映射：将 bytes_stream 转换为提取内容的 String stream
///
/// 用于 `stream_generate` 实现，将 SSE 格式的字节流映射为文本流。
pub fn map_sse_stream<F>(
    stream: impl futures_util::Stream<Item = Result<bytes::Bytes, reqwest::Error>> + Send + 'static,
    extract_content: F,
) -> impl futures_util::Stream<Item = Result<String>> + Send + 'static
where
    F: Fn(&str) -> Option<String> + Send + 'static,
{
    stream.map(move |item| match item {
        Ok(chunk) => {
            let chunk_str =
                std::str::from_utf8(&chunk).map_err(|e| anyhow::anyhow!("UTF-8 error: {}", e))?;

            let mut result = String::new();
            for line in chunk_str.lines() {
                if line.starts_with("data:") {
                    let json_str = line.strip_prefix("data:").unwrap().trim();
                    if json_str != "[DONE]" {
                        if let Some(content) = extract_content(json_str) {
                            result.push_str(&content);
                        }
                    }
                }
            }
            Ok(result)
        }
        Err(e) => Err(anyhow::anyhow!("Stream error: {}", e)),
    })
}

/// JSONL 流映射：将 bytes_stream 转换为提取内容的 String stream
///
/// 用于 `stream_generate` 实现，将 JSONL 格式的字节流映射为文本流。
pub fn map_jsonl_stream<F>(
    stream: impl futures_util::Stream<Item = Result<bytes::Bytes, reqwest::Error>> + Send + 'static,
    extract_content: F,
) -> impl futures_util::Stream<Item = Result<String>> + Send + 'static
where
    F: Fn(&str) -> Option<String> + Send + 'static,
{
    stream.map(move |item| match item {
        Ok(chunk) => {
            let chunk_str =
                std::str::from_utf8(&chunk).map_err(|e| anyhow::anyhow!("UTF-8 error: {}", e))?;

            let mut result = String::new();
            for line in chunk_str.lines() {
                if let Some(content) = extract_content(line) {
                    result.push_str(&content);
                }
            }
            Ok(result)
        }
        Err(e) => Err(anyhow::anyhow!("Stream error: {}", e)),
    })
}
