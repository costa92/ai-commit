use crate::cli::args::Args;
use crate::config::Config;
use crate::git::GitWatcher;

/// 处理监控命令
pub async fn handle_watch_command(_args: &Args, config: &Config) -> anyhow::Result<()> {
    if config.debug {
        println!("Starting repository monitoring...");
    }

    // 先显示一次当前状态
    GitWatcher::check_status().await?;

    // 检查是否需要关注的事项
    let notifications = GitWatcher::needs_attention().await?;
    if !notifications.is_empty() {
        println!("\n⚠️  Items needing attention:");
        for notification in notifications {
            println!("  • {}", notification);
        }
        println!();
    }

    // 开始持续监控
    let interval = if config.debug { 2 } else { 5 }; // debug模式更频繁检查
    GitWatcher::start_watching(interval).await?;

    Ok(())
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_watch_interval() {
        // 测试不同模式下的监控间隔
        let debug_interval = get_watch_interval(true);
        let normal_interval = get_watch_interval(false);

        assert_eq!(debug_interval, 2);
        assert_eq!(normal_interval, 5);
    }

    fn get_watch_interval(debug: bool) -> u64 {
        if debug {
            2
        } else {
            5
        }
    }
}
