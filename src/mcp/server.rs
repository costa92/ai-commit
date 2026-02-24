use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

use super::tools;

/// JSON-RPC 请求
#[derive(Debug, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub method: String,
    #[serde(default)]
    pub params: Value,
}

/// JSON-RPC 响应
#[derive(Debug, Serialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

/// MCP Server 信息
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ServerInfo {
    name: String,
    version: String,
}

/// MCP Initialize 响应
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct InitializeResult {
    protocol_version: String,
    capabilities: ServerCapabilities,
    server_info: ServerInfo,
}

#[derive(Debug, Serialize)]
struct ServerCapabilities {
    tools: ToolsCapability,
}

#[derive(Debug, Serialize)]
struct ToolsCapability {}

/// MCP Tool 定义
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

/// MCP tools/call 参数
#[derive(Debug, Deserialize)]
pub struct ToolCallParams {
    pub name: String,
    #[serde(default)]
    pub arguments: HashMap<String, Value>,
}

/// MCP tools/call 结果
#[derive(Debug, Serialize, Deserialize)]
pub struct ToolCallResult {
    pub content: Vec<ToolContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_error: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ToolContent {
    #[serde(rename = "type")]
    pub content_type: String,
    pub text: String,
}

impl ToolCallResult {
    pub fn text(text: impl Into<String>) -> Self {
        Self {
            content: vec![ToolContent {
                content_type: "text".to_string(),
                text: text.into(),
            }],
            is_error: None,
        }
    }

    pub fn error(text: impl Into<String>) -> Self {
        Self {
            content: vec![ToolContent {
                content_type: "text".to_string(),
                text: text.into(),
            }],
            is_error: Some(true),
        }
    }
}

/// 启动 MCP Server（JSON-RPC over stdio）
pub async fn run_server() -> anyhow::Result<()> {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

    let stdin = tokio::io::stdin();
    let mut stdout = tokio::io::stdout();
    let reader = BufReader::new(stdin);
    let mut lines = reader.lines();

    // MCP 使用行分隔的 JSON-RPC
    while let Some(line) = lines.next_line().await? {
        let line = line.trim().to_string();
        if line.is_empty() {
            continue;
        }

        let request: JsonRpcRequest = match serde_json::from_str(&line) {
            Ok(req) => req,
            Err(e) => {
                let response = JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: None,
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32700,
                        message: format!("Parse error: {}", e),
                        data: None,
                    }),
                };
                let output = serde_json::to_string(&response)?;
                stdout.write_all(output.as_bytes()).await?;
                stdout.write_all(b"\n").await?;
                stdout.flush().await?;
                continue;
            }
        };

        let response = handle_request(request).await;
        let output = serde_json::to_string(&response)?;
        stdout.write_all(output.as_bytes()).await?;
        stdout.write_all(b"\n").await?;
        stdout.flush().await?;
    }

    Ok(())
}

async fn handle_request(request: JsonRpcRequest) -> JsonRpcResponse {
    let id = request.id.clone();

    match request.method.as_str() {
        "initialize" => {
            let result = InitializeResult {
                protocol_version: "2024-11-05".to_string(),
                capabilities: ServerCapabilities {
                    tools: ToolsCapability {},
                },
                server_info: ServerInfo {
                    name: "ai-commit".to_string(),
                    version: env!("CARGO_PKG_VERSION").to_string(),
                },
            };
            JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: Some(serde_json::to_value(result).unwrap()),
                error: None,
            }
        }
        "notifications/initialized" => {
            // 通知，不需要响应（但如果有 id 则回复空）
            JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: Some(Value::Object(serde_json::Map::new())),
                error: None,
            }
        }
        "tools/list" => {
            let tool_list = tools::list_tools();
            let result = serde_json::json!({ "tools": tool_list });
            JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: Some(result),
                error: None,
            }
        }
        "tools/call" => {
            let params: ToolCallParams = match serde_json::from_value(request.params) {
                Ok(p) => p,
                Err(e) => {
                    return JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        id,
                        result: None,
                        error: Some(JsonRpcError {
                            code: -32602,
                            message: format!("Invalid params: {}", e),
                            data: None,
                        }),
                    };
                }
            };

            let result = tools::call_tool(params).await;
            JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: Some(serde_json::to_value(result).unwrap()),
                error: None,
            }
        }
        _ => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(JsonRpcError {
                code: -32601,
                message: format!("Method not found: {}", request.method),
                data: None,
            }),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_json_rpc_request() {
        let json = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#;
        let req: JsonRpcRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.method, "initialize");
        assert_eq!(req.id, Some(Value::Number(1.into())));
    }

    #[test]
    fn test_tool_call_result_text() {
        let result = ToolCallResult::text("hello");
        assert_eq!(result.content.len(), 1);
        assert_eq!(result.content[0].text, "hello");
        assert!(result.is_error.is_none());
    }

    #[test]
    fn test_tool_call_result_error() {
        let result = ToolCallResult::error("something failed");
        assert_eq!(result.content[0].text, "something failed");
        assert_eq!(result.is_error, Some(true));
    }

    #[tokio::test]
    async fn test_handle_initialize() {
        let req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(Value::Number(1.into())),
            method: "initialize".to_string(),
            params: Value::Object(serde_json::Map::new()),
        };

        let resp = handle_request(req).await;
        assert!(resp.result.is_some());
        assert!(resp.error.is_none());

        let result = resp.result.unwrap();
        assert_eq!(result["protocolVersion"], "2024-11-05");
        assert_eq!(result["serverInfo"]["name"], "ai-commit");
    }

    #[tokio::test]
    async fn test_handle_tools_list() {
        let req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(Value::Number(2.into())),
            method: "tools/list".to_string(),
            params: Value::Object(serde_json::Map::new()),
        };

        let resp = handle_request(req).await;
        assert!(resp.result.is_some());
        let result = resp.result.unwrap();
        let tools = result["tools"].as_array().unwrap();
        assert!(!tools.is_empty());
    }

    #[tokio::test]
    async fn test_handle_unknown_method() {
        let req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(Value::Number(99.into())),
            method: "unknown/method".to_string(),
            params: Value::Object(serde_json::Map::new()),
        };

        let resp = handle_request(req).await;
        assert!(resp.error.is_some());
        assert_eq!(resp.error.unwrap().code, -32601);
    }

    #[tokio::test]
    async fn test_handle_tool_call_get_status() {
        let req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(Value::Number(3.into())),
            method: "tools/call".to_string(),
            params: serde_json::json!({"name": "get_status", "arguments": {}}),
        };

        let resp = handle_request(req).await;
        assert!(resp.result.is_some());
    }

    #[tokio::test]
    async fn test_handle_tool_call_get_diff() {
        let req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(Value::Number(4.into())),
            method: "tools/call".to_string(),
            params: serde_json::json!({"name": "get_diff", "arguments": {}}),
        };

        let resp = handle_request(req).await;
        assert!(resp.result.is_some());
    }

    #[tokio::test]
    async fn test_handle_tool_call_unknown_tool() {
        let req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(Value::Number(5.into())),
            method: "tools/call".to_string(),
            params: serde_json::json!({"name": "nonexistent_tool", "arguments": {}}),
        };

        let resp = handle_request(req).await;
        assert!(resp.result.is_some());
        let result: ToolCallResult =
            serde_json::from_value(resp.result.unwrap()).unwrap();
        assert_eq!(result.is_error, Some(true));
    }
}
