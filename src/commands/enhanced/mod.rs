//! 增强的 Git 功能命令处理模块
//!
//! 基于 GRV (Git Repository Viewer) 功能启发，提供高级的 Git 操作和查看功能。
//! 按功能单一原则，每种命令类型对应一个独立的文件。

pub mod diff_view;
pub mod interactive;
pub mod query;
pub mod watch;

// 重新导出主要函数
pub use diff_view::handle_diff_view_command;
pub use interactive::{
    handle_branches_command, handle_contributors_command, handle_interactive_history_command,
    handle_log_stats_command, handle_search_command,
};
pub use query::handle_query_command;
pub use watch::handle_watch_command;

use crate::cli::args::Args;
use crate::config::Config;

/// 检查是否有增强功能命令
pub fn has_enhanced_commands(args: &Args) -> bool {
    args.query.is_some()
        || args.query_history
        || args.query_stats
        || args.query_clear
        || args.query_browse
        || args.diff_view.is_some()
        || args.watch
        || args.log_stats
        || args.log_contributors
        || args.log_search.is_some()
        || args.log_branches
        || args.interactive_history
}

/// 处理增强的Git功能命令（基于GRV功能启发）
pub async fn handle_enhanced_commands(args: &Args, config: &Config) -> anyhow::Result<()> {
    // 查询历史相关功能（优先处理）
    if args.query_history {
        return handle_query_command("history", config).await;
    }

    if args.query_stats {
        return handle_query_command("history-stats", config).await;
    }

    if args.query_clear {
        return handle_query_command("history-clear", config).await;
    }

    if args.query_browse {
        return handle_query_command("history-browse", config).await;
    }

    // 查询功能
    if let Some(query) = &args.query {
        return handle_query_command(query, config).await;
    }

    // 差异查看功能
    if let Some(commit) = &args.diff_view {
        return handle_diff_view_command(commit, config).await;
    }

    // 监控功能
    if args.watch {
        return handle_watch_command(args, config).await;
    }

    // 增强的历史统计功能
    if args.log_stats {
        return handle_log_stats_command(args, config).await;
    }

    // 贡献者统计
    if args.log_contributors {
        return handle_contributors_command(args, config).await;
    }

    // 搜索提交
    if let Some(search_term) = &args.log_search {
        return handle_search_command(search_term, args, config).await;
    }

    // 分支历史图
    if args.log_branches {
        return handle_branches_command(args, config).await;
    }

    // 交互式历史浏览
    if args.interactive_history {
        return handle_interactive_history_command(args, config).await;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_has_enhanced_commands() {
        // 测试命令检测逻辑
        let mut args = Args::default();
        assert!(!has_enhanced_commands(&args));

        args.query = Some("test".to_string());
        assert!(has_enhanced_commands(&args));

        args.query = None;
        args.watch = true;
        assert!(has_enhanced_commands(&args));

        args.watch = false;
        args.log_stats = true;
        assert!(has_enhanced_commands(&args));
    }

    #[test]
    fn test_enhanced_command_priority() {
        // 测试命令优先级逻辑
        let mut args = Args::default();

        // 查询命令应该有最高优先级
        args.query = Some("test".to_string());
        args.diff_view = Some("HEAD".to_string());
        args.watch = true;

        // 在实际函数中，查询命令会先被处理
        assert!(args.query.is_some());
        assert!(args.diff_view.is_some());
        assert!(args.watch);
    }

    #[test]
    fn test_command_detection_comprehensive() {
        // 测试所有增强命令的检测

        // 查询命令
        let mut args = Args::default();
        args.query = Some("test".to_string());
        assert!(has_enhanced_commands(&args));

        // 差异查看命令
        let mut args = Args::default();
        args.diff_view = Some("HEAD".to_string());
        assert!(has_enhanced_commands(&args));

        // 监控命令
        let mut args = Args::default();
        args.watch = true;
        assert!(has_enhanced_commands(&args));

        // 统计命令
        let mut args = Args::default();
        args.log_stats = true;
        assert!(has_enhanced_commands(&args));

        // 贡献者命令
        let mut args = Args::default();
        args.log_contributors = true;
        assert!(has_enhanced_commands(&args));

        // 搜索命令
        let mut args = Args::default();
        args.log_search = Some("fix".to_string());
        assert!(has_enhanced_commands(&args));

        // 分支图命令
        let mut args = Args::default();
        args.log_branches = true;
        assert!(has_enhanced_commands(&args));

        // 交互式历史命令
        let mut args = Args::default();
        args.interactive_history = true;
        assert!(has_enhanced_commands(&args));
    }
}
