use crate::cli::args::Args;
use crate::config::Config;
use crate::git::GitHistory;

/// 处理统计命令
pub async fn handle_log_stats_command(args: &Args, config: &Config) -> anyhow::Result<()> {
    if config.debug {
        println!("Generating commit statistics...");
    }

    GitHistory::show_commit_stats(
        args.log_author.as_deref(),
        args.log_since.as_deref(),
        args.log_until.as_deref(),
    ).await?;

    Ok(())
}

/// 处理贡献者命令
pub async fn handle_contributors_command(_args: &Args, config: &Config) -> anyhow::Result<()> {
    if config.debug {
        println!("Generating contributors statistics...");
    }

    GitHistory::show_contributors().await?;

    Ok(())
}

/// 处理搜索命令
pub async fn handle_search_command(search_term: &str, args: &Args, config: &Config) -> anyhow::Result<()> {
    if config.debug {
        println!("Searching commits for: {}", search_term);
    }

    GitHistory::search_commits(search_term, args.log_limit).await?;

    Ok(())
}

/// 处理分支历史图命令
pub async fn handle_branches_command(args: &Args, config: &Config) -> anyhow::Result<()> {
    if config.debug {
        println!("Generating branch graph...");
    }

    GitHistory::show_branch_graph(args.log_limit).await?;

    Ok(())
}

/// 处理交互式历史浏览命令
pub async fn handle_interactive_history_command(args: &Args, config: &Config) -> anyhow::Result<()> {
    println!("🎯 Interactive History Browser");
    println!("{}", "─".repeat(60));
    
    if config.debug {
        println!("Starting interactive history mode...");
    }

    // 显示可用命令
    print_available_commands();

    // 首先显示基本历史
    GitHistory::show_history(
        args.log_author.as_deref(),
        args.log_since.as_deref(),
        args.log_until.as_deref(),
        args.log_graph,
        args.log_limit,
        args.log_file.as_deref(),
    ).await?;

    // 进入交互模式
    start_interactive_loop(args, config).await?;

    Ok(())
}

/// 显示可用命令
fn print_available_commands() {
    println!("Available commands:");
    println!("  h, help     - Show this help");
    println!("  q, quit     - Quit interactive mode");
    println!("  s <term>    - Search commits");
    println!("  a <author>  - Filter by author");
    println!("  d <date>    - Show commits since date");
    println!("  f <file>    - Show commits for file");
    println!("  stat        - Show statistics");
    println!("  graph       - Show branch graph");
    println!();
}

/// 启动交互循环
async fn start_interactive_loop(_args: &Args, config: &Config) -> anyhow::Result<()> {
    // 这里应该实现交互式的输入处理循环
    // 由于需要处理用户输入，这里简化实现
    if config.debug {
        println!("Interactive mode would start here...");
        println!("Note: Full interactive mode requires stdin handling implementation");
    }

    // TODO: 实现完整的交互式输入处理
    // 需要使用 std::io::stdin() 来读取用户输入
    // 并根据命令执行相应的Git历史操作

    Ok(())
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_command_parsing() {
        // 测试命令解析逻辑
        assert_eq!(parse_interactive_command("h"), InteractiveCommand::Help);
        assert_eq!(parse_interactive_command("help"), InteractiveCommand::Help);
        assert_eq!(parse_interactive_command("q"), InteractiveCommand::Quit);
        assert_eq!(parse_interactive_command("quit"), InteractiveCommand::Quit);
        assert_eq!(parse_interactive_command("s test"), InteractiveCommand::Search("test".to_string()));
    }

    #[derive(Debug, PartialEq)]
    enum InteractiveCommand {
        Help,
        Quit,
        Search(String),
        Author(String),
        Date(String),
        File(String),
        Stats,
        Graph,
    }

    fn parse_interactive_command(input: &str) -> InteractiveCommand {
        let parts: Vec<&str> = input.split_whitespace().collect();
        match parts.get(0) {
            Some(&"h") | Some(&"help") => InteractiveCommand::Help,
            Some(&"q") | Some(&"quit") => InteractiveCommand::Quit,
            Some(&"s") if parts.len() > 1 => InteractiveCommand::Search(parts[1..].join(" ")),
            Some(&"a") if parts.len() > 1 => InteractiveCommand::Author(parts[1..].join(" ")),
            Some(&"d") if parts.len() > 1 => InteractiveCommand::Date(parts[1].to_string()),
            Some(&"f") if parts.len() > 1 => InteractiveCommand::File(parts[1].to_string()),
            Some(&"stat") => InteractiveCommand::Stats,
            Some(&"graph") => InteractiveCommand::Graph,
            _ => InteractiveCommand::Help,
        }
    }
}