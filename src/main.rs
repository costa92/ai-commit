use ai_commit::cli::args::Args;
use ai_commit::commands;
use ai_commit::config::Config;
use ai_commit::git;
use clap::Parser;

async fn handle_worktree_operations(args: &Args, config: &Config) -> anyhow::Result<bool> {
    // 返回 true 如果执行了 worktree 操作，false 如果应该继续执行正常流程

    // 列出所有 worktrees
    if args.worktree_list {
        // 构建worktree list选项
        let options = git::WorktreeListOptions {
            verbose: args.worktree_verbose,
            porcelain: args.worktree_porcelain,
            z: args.worktree_z,
            expire: args.worktree_expire.clone(),
        };

        // 如果用户指定了原生Git选项，直接输出原始结果
        if args.worktree_verbose
            || args.worktree_porcelain
            || args.worktree_z
            || args.worktree_expire.is_some()
        {
            let raw_output = git::list_worktrees_raw(&options).await?;
            print!("{}", raw_output);
        } else {
            // 使用我们的格式化输出
            let worktrees = git::list_worktrees_with_options(&options).await?;
            if worktrees.is_empty() {
                println!("No worktrees found in the repository");
            } else {
                println!("Available worktrees:");
                for worktree in &worktrees {
                    let status = if worktree.is_bare {
                        " (bare)"
                    } else if worktree.is_detached {
                        " (detached HEAD)"
                    } else {
                        ""
                    };
                    println!(
                        "  {} -> {} [{}]{}",
                        worktree.branch,
                        worktree.path.display(),
                        &worktree.commit[..8.min(worktree.commit.len())],
                        status
                    );
                }
            }
        }
        return Ok(true);
    }

    // 创建新的 worktree
    if let Some(branch) = &args.worktree_create {
        let custom_path = args.worktree_path.as_deref();

        // 尝试先创建已存在的分支的 worktree
        let path = match git::create_worktree(branch, custom_path).await {
            Ok(path) => {
                if config.debug {
                    println!(
                        "Created worktree for existing branch '{}' at: {}",
                        branch,
                        path.display()
                    );
                }
                path
            }
            Err(_) => {
                // 如果失败，尝试创建新分支的 worktree
                let path = git::create_worktree_with_new_branch(branch, custom_path).await?;
                if config.debug {
                    println!(
                        "Created worktree with new branch '{}' at: {}",
                        branch,
                        path.display()
                    );
                }
                path
            }
        };

        println!("✓ Worktree created at: {}", path.display());
        println!("  To switch to this worktree, run: cd {}", path.display());
        return Ok(true);
    }

    // 切换到指定的 worktree
    if let Some(name) = &args.worktree_switch {
        let path = git::switch_to_worktree(name).await?;
        println!("✓ Switched to worktree: {}", path.display());

        // 显示当前 worktree 信息
        if let Some(current) = git::get_current_worktree().await? {
            println!("  Current branch: {}", current.branch);
            println!("  Working directory: {}", current.path.display());
        }
        return Ok(true);
    }

    // 删除指定的 worktree
    if let Some(name) = &args.worktree_remove {
        git::remove_worktree(name).await?;
        println!("✓ Removed worktree: {}", name);

        // 清理无效的 worktree 引用
        if config.debug {
            println!("Pruning worktree references...");
        }
        git::prune_worktrees().await?;
        return Ok(true);
    }

    // 清空除当前外的所有其他 worktrees
    if args.worktree_clear {
        let removed_count = git::clear_other_worktrees().await?;

        if removed_count == 0 {
            println!("✓ No other worktrees to remove");
        } else {
            println!("✓ Cleared {} other worktree(s)", removed_count);
        }

        if config.debug {
            println!("Cleared all worktrees except current");
        }
        return Ok(true);
    }

    Ok(false)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let mut config = Config::new();

    config.update_from_args(&args);
    config.validate()?;

    // 处理 worktree 操作
    if handle_worktree_operations(&args, &config).await? {
        return Ok(());
    }

    // 路由到新的命令处理器
    if commands::route_command(&args, &config).await? {
        return Ok(());
    }

    // 显示最新 tag（保持向后兼容）
    if args.show_tag {
        if let Some((tag, note)) = git::get_latest_tag().await {
            println!("Latest tag: {}", tag);
            println!("Tag note: {}", note);
        } else {
            println!("No tags found in the repository");
        }
        return Ok(());
    }

    // 处理 tag 创建（保持向后兼容）
    if args.new_tag.is_some() || std::env::args().any(|arg| arg == "-t" || arg == "--new-tag") {
        let diff = git::get_git_diff().await?;
        commands::handle_tag_creation_commit(&args, &config, &diff).await?;
        return Ok(());
    }

    // 处理常规 commit（保持向后兼容）
    commands::handle_commit_commands(&args, &config).await?;

    Ok(())
}
// 测试大文件修改场景
// 验证逻辑测试
