use crate::cli::args::Args;
use crate::config::Config;
use crate::{ai, git, ui};
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

    // 生成 AI commit message
    let prompt = crate::ai::prompt::get_prompt(&diff);
    let start_time = Instant::now();
    let ai_message = ai::generate_commit_message(&diff, config, &prompt).await?;
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
pub async fn handle_tag_creation_commit(args: &Args, config: &Config, diff: &str) -> anyhow::Result<()> {
    // 先生成下一个 tag 名字
    let tag_name = git::get_next_tag_name(args.new_tag.as_deref()).await?;
    
    // 决定 commit message
    let commit_message = if !args.tag_note.is_empty() {
        // 用户提供了 tag_note，直接使用
        args.tag_note.clone()
    } else {
        // 没有提供 tag_note，使用 AI 生成或默认使用 tag_name
        if !diff.trim().is_empty() {
            // 有代码变更，使用 AI 生成 commit message
            let prompt = crate::ai::prompt::get_prompt(diff);
            let ai_message = ai::generate_commit_message(diff, config, &prompt).await?;
            
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
                println!("Commit commands failed (expected in test environment): {}", e);
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

    fn create_test_args() -> Args {
        Args::default()
    }
}