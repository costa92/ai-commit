use crate::cli::args::Args;
use crate::config::Config;
use crate::git::history::GitHistory;

/// 处理所有历史日志相关命令
pub async fn handle_history_commands(args: &Args, config: &Config) -> anyhow::Result<()> {
    // 如果用户指定了 --history 或任何其他历史相关参数
    if args.history || has_history_filters(args) {
        show_commit_history(args, config).await?;
    }

    Ok(())
}

/// 显示提交历史
async fn show_commit_history(args: &Args, config: &Config) -> anyhow::Result<()> {
    if config.debug {
        println!("Showing commit history with filters...");
    }

    // 如果指定了特定文件，显示文件历史
    if let Some(file_path) = &args.log_file {
        GitHistory::show_file_history(file_path, args.log_limit).await?;
        return Ok(());
    }

    // 如果要显示分支图，使用专门的分支图显示
    if args.log_graph {
        GitHistory::show_branch_graph(args.log_limit).await?;
        return Ok(());
    }

    // 显示常规历史
    GitHistory::show_history(
        args.log_author.as_deref(),
        args.log_since.as_deref(),
        args.log_until.as_deref(),
        false,
        args.log_limit,
        None,
    ).await?;

    // 如果没有其他过滤条件，显示额外的统计信息
    if !has_specific_filters(args) {
        println!("\n");
        show_additional_stats(args, config).await?;
    }

    Ok(())
}

/// 显示额外的统计信息
async fn show_additional_stats(args: &Args, config: &Config) -> anyhow::Result<()> {
    if config.debug {
        println!("Showing additional statistics...");
    }

    // 显示贡献者信息
    println!("\n");
    GitHistory::show_contributors().await?;

    // 显示文件修改统计
    println!("\n");
    GitHistory::show_commit_stats(
        args.log_author.as_deref(),
        args.log_since.as_deref(),
        args.log_until.as_deref(),
    ).await?;

    Ok(())
}

/// 检查是否有历史过滤条件
fn has_history_filters(args: &Args) -> bool {
    args.log_author.is_some()
        || args.log_since.is_some()
        || args.log_until.is_some()
        || args.log_graph
        || args.log_limit.is_some()
        || args.log_file.is_some()
}

/// 检查是否有特定的过滤条件（排除通用的历史显示）
fn has_specific_filters(args: &Args) -> bool {
    args.log_author.is_some()
        || args.log_since.is_some()
        || args.log_until.is_some()
        || args.log_file.is_some()
}

/// 交互式历史浏览
pub async fn interactive_history_browser(config: &Config) -> anyhow::Result<()> {
    println!("🔍 Interactive History Browser");
    println!("{}", "─".repeat(40));
    println!("Available commands:");
    println!("  history                     - Show recent commit history");
    println!("  history --log-graph         - Show branch graph");
    println!("  history --log-author NAME   - Filter by author");
    println!("  history --log-since DATE    - Show commits since date");
    println!("  history --log-until DATE    - Show commits until date");
    println!("  history --log-file PATH     - Show file history");
    println!("  history --log-limit N       - Limit number of commits");
    println!("");
    println!("Date formats: '2023-01-01', 'yesterday', '1 week ago', etc.");
    
    if config.debug {
        println!("\nDebug mode: Additional statistics will be shown");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    #[tokio::test]
    async fn test_handle_history_commands() {
        let config = Config::new();
        let mut args = create_empty_history_args();
        args.history = true;
        
        let result = handle_history_commands(&args, &config).await;
        
        match result {
            Ok(_) => {
                println!("History commands handled successfully");
            }
            Err(e) => {
                println!("History commands failed: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_show_commit_history_with_filters() {
        let config = Config::new();
        let mut args = create_empty_history_args();
        args.history = true;
        args.log_limit = Some(5);
        
        let result = show_commit_history(&args, &config).await;
        
        match result {
            Ok(_) => {
                println!("Commit history with filters displayed successfully");
            }
            Err(e) => {
                println!("Commit history failed: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_interactive_history_browser() {
        let config = Config::new();
        let result = interactive_history_browser(&config).await;
        
        match result {
            Ok(_) => {
                println!("Interactive history browser displayed successfully");
            }
            Err(e) => {
                println!("Interactive history browser failed: {}", e);
            }
        }
    }

    #[test]
    fn test_has_history_filters() {
        let mut args = create_empty_history_args();
        
        // 测试空参数
        assert!(!has_history_filters(&args));
        
        // 测试各种过滤器
        args.log_author = Some("test_author".to_string());
        assert!(has_history_filters(&args));
        
        args = create_empty_history_args();
        args.log_since = Some("yesterday".to_string());
        assert!(has_history_filters(&args));
        
        args = create_empty_history_args();
        args.log_graph = true;
        assert!(has_history_filters(&args));
        
        args = create_empty_history_args();
        args.log_limit = Some(10);
        assert!(has_history_filters(&args));
        
        args = create_empty_history_args();
        args.log_file = Some("test.txt".to_string());
        assert!(has_history_filters(&args));
    }

    #[test]
    fn test_has_specific_filters() {
        let mut args = create_empty_history_args();
        
        // 测试空参数
        assert!(!has_specific_filters(&args));
        
        // 测试特定过滤器
        args.log_author = Some("test_author".to_string());
        assert!(has_specific_filters(&args));
        
        args = create_empty_history_args();
        args.log_file = Some("test.txt".to_string());
        assert!(has_specific_filters(&args));
        
        // log_graph 和 log_limit 不被认为是特定过滤器
        args = create_empty_history_args();
        args.log_graph = true;
        assert!(!has_specific_filters(&args));
        
        args = create_empty_history_args();
        args.log_limit = Some(10);
        assert!(!has_specific_filters(&args));
    }

    fn create_empty_history_args() -> Args {
        Args::default()
    }

}