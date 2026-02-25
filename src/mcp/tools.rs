use serde_json::{json, Value};
use std::collections::HashMap;

use super::server::{ToolCallParams, ToolCallResult, ToolDefinition};

/// 列出所有可用的 MCP tools
pub fn list_tools() -> Vec<ToolDefinition> {
    vec![
        ToolDefinition {
            name: "generate_commit".to_string(),
            description:
                "Generate a conventional commit message using AI based on staged git changes"
                    .to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "emoji": {
                        "type": "boolean",
                        "description": "Add gitmoji prefix to the commit message",
                        "default": false
                    }
                }
            }),
        },
        ToolDefinition {
            name: "get_diff".to_string(),
            description: "Get the current git diff (staged changes)".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "all": {
                        "type": "boolean",
                        "description": "Include unstaged changes too",
                        "default": false
                    }
                }
            }),
        },
        ToolDefinition {
            name: "get_status".to_string(),
            description: "Get the current git repository status".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {}
            }),
        },
        ToolDefinition {
            name: "stage_files".to_string(),
            description: "Stage files for commit".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "files": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "File paths to stage. Empty or omitted means stage all."
                    }
                }
            }),
        },
        ToolDefinition {
            name: "commit".to_string(),
            description: "Create a git commit with the given message".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "message": {
                        "type": "string",
                        "description": "The commit message"
                    }
                },
                "required": ["message"]
            }),
        },
        ToolDefinition {
            name: "get_log".to_string(),
            description: "Get recent git commit log".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "limit": {
                        "type": "integer",
                        "description": "Number of commits to show",
                        "default": 10
                    }
                }
            }),
        },
    ]
}

/// 执行 tool 调用
pub async fn call_tool(params: ToolCallParams) -> ToolCallResult {
    match params.name.as_str() {
        "generate_commit" => tool_generate_commit(params.arguments).await,
        "get_diff" => tool_get_diff(params.arguments).await,
        "get_status" => tool_get_status().await,
        "stage_files" => tool_stage_files(params.arguments).await,
        "commit" => tool_commit(params.arguments).await,
        "get_log" => tool_get_log(params.arguments).await,
        _ => ToolCallResult::error(format!("Unknown tool: {}", params.name)),
    }
}

async fn tool_generate_commit(args: HashMap<String, Value>) -> ToolCallResult {
    let emoji = args.get("emoji").and_then(|v| v.as_bool()).unwrap_or(false);

    // 获取 diff
    let diff = match crate::git::get_git_diff().await {
        Ok(d) => d,
        Err(e) => return ToolCallResult::error(format!("Failed to get diff: {}", e)),
    };

    if diff.trim().is_empty() {
        // 尝试获取所有变更
        match crate::git::get_all_changes_diff().await {
            Ok(d) if !d.trim().is_empty() => {
                return ToolCallResult::text(format!(
                    "No staged changes found. There are unstaged changes. Run `git add` first, or use the stage_files tool.\n\nUnstaged diff preview:\n{}",
                    &d[..d.len().min(500)]
                ));
            }
            _ => return ToolCallResult::text("No changes to commit."),
        }
    }

    // 使用 Agent 生成
    let config = crate::config::Config::new();
    match generate_with_agent(&diff, &config).await {
        Ok(mut message) => {
            if emoji {
                message = crate::core::gitmoji::add_emoji(&message);
            }
            ToolCallResult::text(message)
        }
        Err(e) => ToolCallResult::error(format!("Failed to generate commit message: {}", e)),
    }
}

async fn tool_get_diff(args: HashMap<String, Value>) -> ToolCallResult {
    let all = args.get("all").and_then(|v| v.as_bool()).unwrap_or(false);

    let result = if all {
        crate::git::get_all_changes_diff().await
    } else {
        crate::git::get_git_diff().await
    };

    match result {
        Ok(diff) => {
            if diff.trim().is_empty() {
                ToolCallResult::text("No changes found.")
            } else {
                ToolCallResult::text(diff)
            }
        }
        Err(e) => ToolCallResult::error(format!("Failed to get diff: {}", e)),
    }
}

async fn tool_get_status() -> ToolCallResult {
    let output = tokio::process::Command::new("git")
        .args(["status", "--porcelain=v1"])
        .output()
        .await;

    match output {
        Ok(output) if output.status.success() => {
            let status = String::from_utf8_lossy(&output.stdout).to_string();
            if status.trim().is_empty() {
                ToolCallResult::text("Working tree clean. No changes.")
            } else {
                // Also get branch info
                let branch = tokio::process::Command::new("git")
                    .args(["branch", "--show-current"])
                    .output()
                    .await
                    .ok()
                    .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
                    .unwrap_or_default();

                ToolCallResult::text(format!("Branch: {}\n\n{}", branch, status))
            }
        }
        Ok(output) => ToolCallResult::error(String::from_utf8_lossy(&output.stderr).to_string()),
        Err(e) => ToolCallResult::error(format!("Failed to run git status: {}", e)),
    }
}

async fn tool_stage_files(args: HashMap<String, Value>) -> ToolCallResult {
    let files: Vec<String> = args
        .get("files")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();

    let result = if files.is_empty() {
        // Stage all
        crate::git::git_add_all().await
    } else {
        // Stage specific files
        let output = match tokio::process::Command::new("git")
            .arg("add")
            .args(&files)
            .output()
            .await
        {
            Ok(o) => o,
            Err(e) => return ToolCallResult::error(format!("Failed to run git add: {}", e)),
        };

        if output.status.success() {
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "git add failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ))
        }
    };

    match result {
        Ok(()) => {
            if files.is_empty() {
                ToolCallResult::text("All changes staged successfully.")
            } else {
                ToolCallResult::text(format!("Staged {} file(s).", files.len()))
            }
        }
        Err(e) => ToolCallResult::error(format!("Failed to stage files: {}", e)),
    }
}

async fn tool_commit(args: HashMap<String, Value>) -> ToolCallResult {
    let message = match args.get("message").and_then(|v| v.as_str()) {
        Some(msg) => msg.to_string(),
        None => return ToolCallResult::error("Missing required parameter: message"),
    };

    if message.trim().is_empty() {
        return ToolCallResult::error("Commit message cannot be empty");
    }

    match crate::git::git_commit(&message).await {
        Ok(()) => ToolCallResult::text(format!("Committed: {}", message)),
        Err(e) => ToolCallResult::error(format!("Commit failed: {}", e)),
    }
}

async fn tool_get_log(args: HashMap<String, Value>) -> ToolCallResult {
    let limit = args
        .get("limit")
        .and_then(|v| v.as_u64())
        .unwrap_or(10)
        .min(50);

    let output = tokio::process::Command::new("git")
        .args([
            "log",
            &format!("-{}", limit),
            "--pretty=format:%h %s (%cr) <%an>",
        ])
        .output()
        .await;

    match output {
        Ok(output) if output.status.success() => {
            let log = String::from_utf8_lossy(&output.stdout).to_string();
            if log.trim().is_empty() {
                ToolCallResult::text("No commits found.")
            } else {
                ToolCallResult::text(log)
            }
        }
        Ok(output) => ToolCallResult::error(String::from_utf8_lossy(&output.stderr).to_string()),
        Err(e) => ToolCallResult::error(format!("Failed to run git log: {}", e)),
    }
}

/// 使用 Agent 生成 commit message
async fn generate_with_agent(diff: &str, config: &crate::config::Config) -> anyhow::Result<String> {
    use crate::core::ai::agents::{AgentConfig, AgentContext, AgentManager, AgentTask, TaskType};
    use crate::core::ai::memory::ProjectMemory;

    let mut agent_manager = AgentManager::with_default_context();

    let mut env_vars: HashMap<String, String> = std::env::vars().collect();
    if let Some(api_key) = config.get_api_key() {
        env_vars.insert("API_KEY".to_string(), api_key);
    }
    env_vars.insert("API_URL".to_string(), config.get_url());

    // 注入项目记忆
    let working_dir = std::env::current_dir().unwrap_or_default();
    let memory = ProjectMemory::load(&working_dir).unwrap_or_default();
    let memory_context = memory.to_prompt_context();
    if !memory_context.is_empty() {
        env_vars.insert("MEMORY_CONTEXT".to_string(), memory_context);
    }

    let agent_config = AgentConfig {
        provider: config.provider.clone(),
        model: config.model.clone(),
        temperature: 0.7,
        max_tokens: 2000,
        stream: false, // MCP server 不需要流式
        max_retries: 3,
        timeout_secs: 60,
    };

    let context = AgentContext {
        working_dir: std::env::current_dir()?,
        env_vars,
        config: agent_config,
        history: vec![],
    };

    agent_manager.update_context(context);
    let commit_agent = agent_manager.get_or_create_agent("commit").await?;
    let task = AgentTask::new(TaskType::GenerateCommit, diff);
    let result = commit_agent.execute(task, agent_manager.context()).await?;

    if !result.success {
        anyhow::bail!("Agent failed to generate commit message");
    }

    Ok(result.content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_tools_not_empty() {
        let tools = list_tools();
        assert!(!tools.is_empty());
    }

    #[test]
    fn test_list_tools_has_required_tools() {
        let tools = list_tools();
        let names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();

        assert!(names.contains(&"generate_commit"));
        assert!(names.contains(&"get_diff"));
        assert!(names.contains(&"get_status"));
        assert!(names.contains(&"stage_files"));
        assert!(names.contains(&"commit"));
        assert!(names.contains(&"get_log"));
    }

    #[test]
    fn test_tool_definitions_have_schemas() {
        let tools = list_tools();
        for tool in &tools {
            assert!(!tool.name.is_empty());
            assert!(!tool.description.is_empty());
            assert!(tool.input_schema.is_object());
        }
    }

    #[tokio::test]
    async fn test_call_unknown_tool() {
        let params = ToolCallParams {
            name: "nonexistent".to_string(),
            arguments: HashMap::new(),
        };
        let result = call_tool(params).await;
        assert_eq!(result.is_error, Some(true));
    }

    #[tokio::test]
    async fn test_call_get_status() {
        let params = ToolCallParams {
            name: "get_status".to_string(),
            arguments: HashMap::new(),
        };
        let result = call_tool(params).await;
        // Should succeed in a git repo (not an error)
        assert_ne!(result.is_error, Some(true));
        assert!(!result.content.is_empty());
    }

    #[tokio::test]
    async fn test_call_get_diff() {
        let params = ToolCallParams {
            name: "get_diff".to_string(),
            arguments: HashMap::new(),
        };
        let result = call_tool(params).await;
        assert_ne!(result.is_error, Some(true));
    }

    #[tokio::test]
    async fn test_call_get_log() {
        let mut args = HashMap::new();
        args.insert("limit".to_string(), json!(5));
        let params = ToolCallParams {
            name: "get_log".to_string(),
            arguments: args,
        };
        let result = call_tool(params).await;
        assert_ne!(result.is_error, Some(true));
    }

    #[tokio::test]
    async fn test_call_commit_missing_message() {
        let params = ToolCallParams {
            name: "commit".to_string(),
            arguments: HashMap::new(),
        };
        let result = call_tool(params).await;
        assert_eq!(result.is_error, Some(true));
        assert!(result.content[0].text.contains("Missing"));
    }

    #[tokio::test]
    async fn test_call_commit_empty_message() {
        let mut args = HashMap::new();
        args.insert("message".to_string(), json!(""));
        let params = ToolCallParams {
            name: "commit".to_string(),
            arguments: args,
        };
        let result = call_tool(params).await;
        assert_eq!(result.is_error, Some(true));
    }
}
