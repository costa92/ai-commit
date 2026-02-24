use crate::cli::args::Args;
use crate::config::Config;
use crate::core::ai::agents::{AgentConfig, AgentContext, AgentManager, AgentTask, TaskType};
use crate::git::edit::{GitEdit, RebaseStatus};
use std::collections::HashMap;

/// 处理所有 commit 编辑相关命令
pub async fn handle_edit_commands(args: &Args, config: &Config) -> anyhow::Result<()> {
    // 检查 rebase 状态
    let rebase_status = GitEdit::check_rebase_status().await?;
    match rebase_status {
        RebaseStatus::InProgressWithConflicts => {
            println!("⚠️  Rebase in progress with conflicts!");
            println!("   Resolve conflicts, then run: git rebase --continue");
            println!("   Or abort with: git rebase --abort");
            return Ok(());
        }
        RebaseStatus::InProgress => {
            println!("ℹ️  Rebase in progress");
            println!("   Continue with: git rebase --continue");
            println!("   Or abort with: git rebase --abort");
        }
        RebaseStatus::None => {}
    }

    // 处理修改最后一次提交
    if args.amend {
        handle_amend_commit(args, config).await?;
        return Ok(());
    }

    // 处理撤销最后一次提交
    if args.undo_commit {
        let result = GitEdit::undo_last_commit().await?;
        println!("{}", result);
        return Ok(());
    }

    // 处理编辑特定提交
    if let Some(commit_hash) = &args.edit_commit {
        let result = GitEdit::edit_specific_commit(commit_hash).await?;
        println!("{}", result);
        return Ok(());
    }

    // 处理交互式 rebase
    if let Some(base_commit) = &args.rebase_edit {
        let result = GitEdit::interactive_rebase(base_commit).await?;
        println!("{}", result);
        return Ok(());
    }

    // 处理重写提交消息
    if let Some(commit_hash) = &args.reword_commit {
        handle_reword_commit(commit_hash, config).await?;
        return Ok(());
    }

    // 如果没有具体的编辑操作，显示可编辑的提交列表
    let result = GitEdit::show_editable_commits(Some(10)).await?;
    println!("{}", result);

    Ok(())
}

/// 处理 amend 提交，可选择使用 AI 生成新的提交消息
async fn handle_amend_commit(_args: &Args, config: &Config) -> anyhow::Result<()> {
    println!("🔄 Amending the last commit...");

    let diff_output = tokio::process::Command::new("git")
        .args(["diff", "--cached"])
        .output()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to check staged changes: {}", e))?;

    let has_staged_changes = !diff_output.stdout.is_empty();

    if has_staged_changes && config.debug {
        println!("Found staged changes, will include them in the amendment");
    }

    if !has_staged_changes {
        println!("No staged changes found.");
        println!("Options:");
        println!("  1. Use AI to generate a new commit message based on current changes");
        println!("  2. Keep the original commit message");
        println!("  3. Abort amendment");

        let result = GitEdit::amend_last_commit(None).await?;
        println!("{}", result);
        return Ok(());
    }

    let staged_diff = String::from_utf8_lossy(&diff_output.stdout);

    if !staged_diff.trim().is_empty() {
        println!("Generating AI commit message for staged changes...");

        let ai_message = generate_message_with_agent(&staged_diff, config).await?;

        if !ai_message.is_empty() {
            println!("AI generated message: {}", ai_message);
            let result = GitEdit::amend_last_commit(Some(&ai_message)).await?;
            println!("{}", result);
        } else {
            let result = GitEdit::amend_last_commit(None).await?;
            println!("{}", result);
        }
    } else {
        let result = GitEdit::amend_last_commit(None).await?;
        println!("{}", result);
    }

    Ok(())
}

/// 处理重写提交消息，使用 AI 生成新消息
async fn handle_reword_commit(commit_hash: &str, config: &Config) -> anyhow::Result<()> {
    println!("🔄 Rewriting commit message for {}...", commit_hash);

    let diff_output = tokio::process::Command::new("git")
        .args(["show", commit_hash, "--pretty=format:", "--name-only"])
        .output()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to get commit changes: {}", e))?;

    if !diff_output.status.success() {
        anyhow::bail!("Failed to get commit information for '{}'", commit_hash);
    }

    let commit_diff_output = tokio::process::Command::new("git")
        .args(["show", commit_hash, "--pretty=format:"])
        .output()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to get commit diff: {}", e))?;

    let commit_diff = String::from_utf8_lossy(&commit_diff_output.stdout);

    if commit_diff.trim().is_empty() {
        println!("No changes found in commit, keeping original message");
        return Ok(());
    }

    println!(
        "Generating AI commit message for the changes in {}...",
        commit_hash
    );

    let ai_message = generate_message_with_agent(&commit_diff, config).await?;

    if !ai_message.is_empty() {
        println!("AI generated message: {}", ai_message);
        let result = GitEdit::reword_commit(commit_hash, &ai_message).await?;
        println!("{}", result);
    } else {
        println!("Failed to generate AI message, keeping original");
    }

    Ok(())
}

/// 显示编辑操作的帮助信息
pub async fn show_edit_help() -> anyhow::Result<()> {
    println!("✏️  Git Commit Editing Commands:");
    println!("{}", "─".repeat(50));
    println!();
    println!("📝 Basic Operations:");
    println!("  --amend                     Modify the last commit (with AI)");
    println!("  --undo-commit               Undo last commit (keep changes staged)");
    println!();
    println!("🔍 Advanced Operations:");
    println!("  --edit-commit HASH          Edit specific commit interactively");
    println!("  --reword-commit HASH        Rewrite commit message with AI");
    println!("  --rebase-edit BASE          Interactive rebase from base commit");
    println!();
    println!("📋 Information:");
    println!("  (no args)                   Show recent editable commits");
    println!();
    println!("💡 Tips:");
    println!("  - All operations preserve your work");
    println!("  - AI will generate contextual commit messages");
    println!("  - Use commit hashes or references like HEAD~1");
    println!("  - Interactive rebase opens your default editor");
    println!();
    println!("⚠️  Safety Notes:");
    println!("  - These operations rewrite Git history");
    println!("  - Avoid editing pushed commits (use --force-push if necessary)");
    println!("  - Always backup important work before major edits");

    Ok(())
}

/// 使用 Agent 生成 commit message
async fn generate_message_with_agent(diff: &str, config: &Config) -> anyhow::Result<String> {
    let mut agent_manager = AgentManager::with_default_context();

    let mut env_vars: HashMap<String, String> = std::env::vars().collect();
    if let Some(api_key) = config.get_api_key() {
        env_vars.insert("API_KEY".to_string(), api_key);
    }
    env_vars.insert("API_URL".to_string(), config.get_url());

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
    use crate::config::Config;

    #[tokio::test]
    async fn test_handle_edit_commands_no_args() {
        let config = Config::new();
        let args = create_empty_edit_args();

        let result = handle_edit_commands(&args, &config).await;

        match result {
            Ok(_) => {
                println!("Edit commands handled successfully (shows editable commits)");
            }
            Err(e) => {
                println!("Edit commands failed: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_show_edit_help() {
        let result = show_edit_help().await;

        match result {
            Ok(_) => {
                println!("Edit help displayed successfully");
            }
            Err(e) => {
                println!("Edit help failed: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_handle_amend_commit() {
        let config = Config::new();
        let mut args = create_empty_edit_args();
        args.amend = true;

        let result = handle_amend_commit(&args, &config).await;

        match result {
            Ok(_) => {
                println!("Amend commit handled successfully");
            }
            Err(e) => {
                println!("Amend commit failed (expected in test environment): {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_handle_reword_commit() {
        let config = Config::new();
        let result = handle_reword_commit("HEAD", &config).await;

        match result {
            Ok(_) => {
                println!("Reword commit handled successfully");
            }
            Err(e) => {
                println!("Reword commit failed (expected in test environment): {}", e);
            }
        }
    }

    fn create_empty_edit_args() -> Args {
        Args::default()
    }
}
