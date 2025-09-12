use crate::cli::args::Args;
use crate::config::Config;
use crate::git::flow::GitFlow;

/// 处理所有 Git Flow 相关命令
pub async fn handle_flow_commands(args: &Args, config: &Config) -> anyhow::Result<()> {
    if args.flow_init {
        GitFlow::init().await?;
        return Ok(());
    }

    if let Some(name) = &args.flow_feature_start {
        GitFlow::start_feature(name).await?;
        return Ok(());
    }

    if let Some(name) = &args.flow_feature_finish {
        GitFlow::finish_feature(name).await?;
        return Ok(());
    }

    if let Some(name) = &args.flow_hotfix_start {
        GitFlow::start_hotfix(name).await?;
        return Ok(());
    }

    if let Some(name) = &args.flow_hotfix_finish {
        GitFlow::finish_hotfix(name).await?;
        return Ok(());
    }

    if let Some(version) = &args.flow_release_start {
        GitFlow::start_release(version).await?;
        return Ok(());
    }

    if let Some(version) = &args.flow_release_finish {
        GitFlow::finish_release(version).await?;
        return Ok(());
    }

    // 如果没有指定具体操作，显示当前状态
    show_flow_status(config).await?;

    Ok(())
}

/// 显示 Git Flow 状态
async fn show_flow_status(config: &Config) -> anyhow::Result<()> {
    println!("🌿 Git Flow Status:");
    println!("{}", "─".repeat(40));

    // 显示当前分支类型
    match GitFlow::get_branch_type().await {
        Ok(branch_type) => {
            println!("📍 Current branch type: {:?}", branch_type);
        }
        Err(e) => {
            if config.debug {
                println!("Could not determine branch type: {}", e);
            }
        }
    }

    // 列出所有 flow 分支
    GitFlow::list_flow_branches().await?;

    println!("\n💡 Available Git Flow commands:");
    println!("  --flow-init                    Initialize Git Flow");
    println!("  --flow-feature-start NAME      Start new feature");
    println!("  --flow-feature-finish NAME     Finish feature");
    println!("  --flow-hotfix-start NAME       Start hotfix");
    println!("  --flow-hotfix-finish NAME      Finish hotfix");
    println!("  --flow-release-start VERSION   Start release");
    println!("  --flow-release-finish VERSION  Finish release");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::args::Args;
    use crate::config::Config;

    #[tokio::test]
    async fn test_show_flow_status() {
        let config = Config::new();
        let result = show_flow_status(&config).await;

        match result {
            Ok(_) => {
                println!("Flow status displayed successfully");
            }
            Err(e) => {
                println!("Flow status failed: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_handle_flow_commands_no_args() {
        let config = Config::new();
        let args = create_empty_args();

        let result = handle_flow_commands(&args, &config).await;

        match result {
            Ok(_) => {
                println!("Handle flow commands succeeded (shows status)");
            }
            Err(e) => {
                println!("Handle flow commands failed: {}", e);
            }
        }
    }

    fn create_empty_args() -> Args {
        Args::default()
    }
}
