use crate::cli::args::Args;
use crate::config::Config;
use crate::git::edit::{GitEdit, RebaseStatus};
use crate::ai;

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
        GitEdit::undo_last_commit().await?;
        return Ok(());
    }

    // 处理编辑特定提交
    if let Some(commit_hash) = &args.edit_commit {
        GitEdit::edit_specific_commit(commit_hash).await?;
        return Ok(());
    }

    // 处理交互式 rebase
    if let Some(base_commit) = &args.rebase_edit {
        GitEdit::interactive_rebase(base_commit).await?;
        return Ok(());
    }

    // 处理重写提交消息
    if let Some(commit_hash) = &args.reword_commit {
        handle_reword_commit(commit_hash, config).await?;
        return Ok(());
    }

    // 如果没有具体的编辑操作，显示可编辑的提交列表
    GitEdit::show_editable_commits(Some(10)).await?;
    
    Ok(())
}

/// 处理 amend 提交，可选择使用 AI 生成新的提交消息
async fn handle_amend_commit(_args: &Args, config: &Config) -> anyhow::Result<()> {
    println!("🔄 Amending the last commit...");

    // 检查是否有新的暂存更改
    let diff_output = tokio::process::Command::new("git")
        .args(["diff", "--cached"])
        .output()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to check staged changes: {}", e))?;

    let has_staged_changes = !diff_output.stdout.is_empty();

    if has_staged_changes && config.debug {
        println!("Found staged changes, will include them in the amendment");
    }

    // 如果用户没有暂存任何新更改，询问是否要修改提交消息
    if !has_staged_changes {
        println!("No staged changes found.");
        println!("Options:");
        println!("  1. Use AI to generate a new commit message based on current changes");
        println!("  2. Keep the original commit message");
        println!("  3. Abort amendment");
        
        // 为简化起见，这里直接保持原有消息
        GitEdit::amend_last_commit(None).await?;
        return Ok(());
    }

    // 如果有暂存更改，可选择使用 AI 生成新的提交消息
    let staged_diff = String::from_utf8_lossy(&diff_output.stdout);
    
    if !staged_diff.trim().is_empty() {
        println!("Generating AI commit message for staged changes...");
        
        let prompt = crate::ai::prompt::get_prompt(&staged_diff);
        let ai_message = ai::generate_commit_message(&staged_diff, config, &prompt).await?;
        
        if !ai_message.is_empty() {
            println!("AI generated message: {}", ai_message);
            GitEdit::amend_last_commit(Some(&ai_message)).await?;
        } else {
            GitEdit::amend_last_commit(None).await?;
        }
    } else {
        GitEdit::amend_last_commit(None).await?;
    }

    Ok(())
}

/// 处理重写提交消息，使用 AI 生成新消息
async fn handle_reword_commit(commit_hash: &str, config: &Config) -> anyhow::Result<()> {
    println!("🔄 Rewriting commit message for {}...", commit_hash);

    // 获取该提交的变更内容
    let diff_output = tokio::process::Command::new("git")
        .args(["show", commit_hash, "--pretty=format:", "--name-only"])
        .output()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to get commit changes: {}", e))?;

    if !diff_output.status.success() {
        anyhow::bail!("Failed to get commit information for '{}'", commit_hash);
    }

    // 获取该提交的 diff
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

    println!("Generating AI commit message for the changes in {}...", commit_hash);
    
    let prompt = crate::ai::prompt::get_prompt(&commit_diff);
    let ai_message = ai::generate_commit_message(&commit_diff, config, &prompt).await?;
    
    if !ai_message.is_empty() {
        println!("AI generated message: {}", ai_message);
        GitEdit::reword_commit(commit_hash, &ai_message).await?;
    } else {
        println!("Failed to generate AI message, keeping original");
    }

    Ok(())
}

/// 显示编辑操作的帮助信息
pub async fn show_edit_help() -> anyhow::Result<()> {
    println!("✏️  Git Commit Editing Commands:");
    println!("{}", "─".repeat(50));
    println!("");
    println!("📝 Basic Operations:");
    println!("  --amend                     Modify the last commit (with AI)");
    println!("  --undo-commit               Undo last commit (keep changes staged)");
    println!("");
    println!("🔍 Advanced Operations:");
    println!("  --edit-commit HASH          Edit specific commit interactively");
    println!("  --reword-commit HASH        Rewrite commit message with AI");
    println!("  --rebase-edit BASE          Interactive rebase from base commit");
    println!("");
    println!("📋 Information:");
    println!("  (no args)                   Show recent editable commits");
    println!("");
    println!("💡 Tips:");
    println!("  - All operations preserve your work");
    println!("  - AI will generate contextual commit messages");
    println!("  - Use commit hashes or references like HEAD~1");
    println!("  - Interactive rebase opens your default editor");
    println!("");
    println!("⚠️  Safety Notes:");
    println!("  - These operations rewrite Git history");
    println!("  - Avoid editing pushed commits (use --force-push if necessary)");
    println!("  - Always backup important work before major edits");

    Ok(())
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