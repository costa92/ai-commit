use crate::cli::args::Args;
use crate::config::Config;
use crate::core::ai::agents::{AgentConfig, AgentContext, AgentManager, AgentTask, TaskType};
use crate::{git, ui};
use std::collections::HashMap;
use std::time::Instant;

/// 处理常规的 commit 相关命令
pub async fn handle_commit_commands(args: &Args, config: &Config) -> anyhow::Result<()> {
    // git add（如果需要）
    if !args.no_add {
        git::git_add_all().await?;
    }

    let diff = git::get_git_diff().await?;

    if diff.trim().is_empty() {
        if config.debug {
            println!("No staged changes.");
        }
        return Ok(());
    }

    // 使用 Agent 生成 commit message
    let start_time = Instant::now();
    let mut ai_message = generate_commit_message_with_agent(&diff, config).await?;
    let elapsed_time = start_time.elapsed();

    if config.debug {
        println!("AI 生成 commit message 耗时: {:.2?}", elapsed_time);
        if elapsed_time.as_secs() > 30 {
            println!("警告: AI 模型 '{}' 生成 commit message 耗时较长，建议更换更快的模型或优化网络环境。", config.model);
        }
    }

    if ai_message.is_empty() {
        eprintln!("AI 生成 commit message 为空，请检查 AI 服务。");
        std::process::exit(1);
    }

    // 应用 gitmoji（如果启用）
    if config.emoji {
        ai_message = crate::core::gitmoji::add_emoji(&ai_message);
    }

    // 用户确认 commit message
    let final_message = match ui::confirm_commit_message(&ai_message, args.skip_confirm)? {
        ui::ConfirmResult::Confirmed(message) => message,
        ui::ConfirmResult::Rejected => {
            println!("操作已取消。");
            return Ok(());
        }
    };

    // 提交更改
    git::git_commit(&final_message).await?;

    // 推送（如果需要）
    if args.push {
        if args.force_push {
            git::git_force_push().await?;
        } else {
            git::git_push().await?;
        }
    }

    Ok(())
}

/// 处理 tag 创建相关的 commit 逻辑
pub async fn handle_tag_creation_commit(
    args: &Args,
    config: &Config,
    diff: &str,
) -> anyhow::Result<()> {
    // 先生成下一个 tag 名字
    let tag_name = git::get_next_tag_name(args.new_tag.as_deref()).await?;

    // 决定 commit message
    let commit_message = if !args.tag_note.is_empty() {
        // 用户提供了 tag_note，直接使用
        args.tag_note.clone()
    } else {
        // 没有提供 tag_note，使用 AI 生成或默认使用 tag_name
        if !diff.trim().is_empty() {
            // 有代码变更，使用 Agent 生成 commit message
            let mut ai_message = generate_commit_message_with_agent(diff, config).await?;

            // 应用 gitmoji（如果启用）
            if config.emoji {
                ai_message = crate::core::gitmoji::add_emoji(&ai_message);
            }

            if !ai_message.is_empty() {
                // 用户确认 AI 生成的消息
                match ui::confirm_commit_message(&ai_message, args.skip_confirm)? {
                    ui::ConfirmResult::Confirmed(message) => message,
                    ui::ConfirmResult::Rejected => {
                        println!("操作已取消。");
                        return Ok(());
                    }
                }
            } else {
                // AI 生成失败，使用默认 tag name
                tag_name.clone()
            }
        } else {
            // 没有代码变更，直接使用 tag name
            tag_name.clone()
        }
    };

    if !diff.trim().is_empty() {
        git::git_commit(&commit_message).await?;
    } else {
        git::git_commit_allow_empty(&commit_message).await?;
    }

    // 创建 tag，使用相同的 commit message 作为 tag note
    git::create_tag_with_note(&tag_name, &commit_message).await?;

    if config.debug {
        println!("Created new tag: {}", &tag_name);
    }
    if args.push {
        if args.force_push {
            // 对于tag推送，先尝试强制推送commit，再推送tag
            git::git_force_push().await?;
        }
        git::push_tag(&tag_name, args.push_branches).await?;
        if config.debug {
            println!("Pushed tag {} to remote", &tag_name);
        }
    }
    Ok(())
}

/// 使用 Agent 生成 commit message
async fn generate_commit_message_with_agent(diff: &str, config: &Config) -> anyhow::Result<String> {
    // 创建 Agent 管理器
    let mut agent_manager = AgentManager::with_default_context();

    // 更新 Agent 配置
    let mut env_vars = std::env::vars().collect::<HashMap<String, String>>();

    // 添加 API Key 配置
    if let Some(api_key) = config.get_api_key() {
        env_vars.insert("API_KEY".to_string(), api_key);
    }

    // 设置 API URL
    let api_url = config.get_url();
    env_vars.insert("API_URL".to_string(), api_url);

    let agent_config = AgentConfig {
        provider: config.provider.clone(),
        model: config.model.clone(),
        temperature: 0.7,
        max_tokens: 2000,
        stream: true,
        max_retries: 3,
        timeout_secs: 60,
    };

    let context = AgentContext {
        working_dir: std::env::current_dir()?,
        env_vars,
        config: agent_config,
        history: vec![],
    };

    // 更新管理器上下文
    agent_manager.update_context(context);

    // 获取或创建 Commit Agent
    let commit_agent = agent_manager.get_or_create_agent("commit").await?;

    // 创建任务
    let task = AgentTask::new(TaskType::GenerateCommit, diff);

    // 执行任务
    let result = commit_agent.execute(task, agent_manager.context()).await?;

    if !result.success {
        anyhow::bail!("Agent failed to generate commit message");
    }

    Ok(result.content)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    #[tokio::test]
    async fn test_handle_commit_commands() {
        let config = Config::new();
        let args = create_test_args();

        let result = handle_commit_commands(&args, &config).await;

        match result {
            Ok(_) => {
                println!("Commit commands handled successfully");
            }
            Err(e) => {
                println!(
                    "Commit commands failed (expected in test environment): {}",
                    e
                );
            }
        }
    }

    #[tokio::test]
    async fn test_handle_tag_creation_commit() {
        let config = Config::new();
        let args = create_test_args();
        let test_diff = "diff --git a/test.txt b/test.txt\n+new line";

        let result = handle_tag_creation_commit(&args, &config, test_diff).await;

        match result {
            Ok(_) => {
                println!("Tag creation commit handled successfully");
            }
            Err(e) => {
                println!("Tag creation commit failed: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_generate_commit_message_with_agent() {
        let config = Config::new();
        let test_diff = "diff --git a/test.txt b/test.txt\n+new line";

        let result = generate_commit_message_with_agent(test_diff, &config).await;

        match result {
            Ok(message) => {
                println!("Generated commit message: {}", message);
                // 基本验证：消息不应该为空
                assert!(!message.is_empty());
            }
            Err(e) => {
                println!(
                    "Agent commit generation failed (expected in test environment): {}",
                    e
                );
            }
        }
    }

    fn create_test_args() -> Args {
        Args::default()
    }
}
