use crate::config::Config;
use crate::git::DiffViewer;

/// 处理差异查看命令
pub async fn handle_diff_view_command(commit: &str, config: &Config) -> anyhow::Result<()> {
    if config.debug {
        println!("Viewing diff for commit: {}", commit);
    }

    if commit == "working" || commit == "unstaged" {
        DiffViewer::show_working_diff(false).await?;
    } else if commit == "staged" || commit == "cached" {
        DiffViewer::show_working_diff(true).await?;
    } else if commit.contains("..") {
        // 比较两个提交
        let parts: Vec<&str> = commit.split("..").collect();
        if parts.len() == 2 {
            DiffViewer::compare_commits(parts[0], parts[1]).await?;
        } else {
            anyhow::bail!("Invalid commit range format. Use: commit1..commit2");
        }
    } else {
        // 显示单个提交的差异
        DiffViewer::show_commit_diff(commit, Some(3)).await?;
    }

    // 显示差异统计
    if config.debug {
        let stats = DiffViewer::get_diff_stats(Some(commit), None).await?;
        DiffViewer::display_diff_summary(&stats);
    }

    Ok(())
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_commit_parsing() {
        // 测试不同的提交格式
        assert_eq!(parse_commit_type("working"), CommitType::Working);
        assert_eq!(parse_commit_type("unstaged"), CommitType::Working);
        assert_eq!(parse_commit_type("staged"), CommitType::Staged);
        assert_eq!(parse_commit_type("cached"), CommitType::Staged);
        assert_eq!(parse_commit_type("abc123..def456"), CommitType::Range);
        assert_eq!(parse_commit_type("abc123"), CommitType::Single);
    }

    #[derive(Debug, PartialEq)]
    enum CommitType {
        Working,
        Staged,
        Range,
        Single,
    }

    fn parse_commit_type(commit: &str) -> CommitType {
        if commit == "working" || commit == "unstaged" {
            CommitType::Working
        } else if commit == "staged" || commit == "cached" {
            CommitType::Staged
        } else if commit.contains("..") {
            CommitType::Range
        } else {
            CommitType::Single
        }
    }

    #[test]
    fn test_commit_range_parsing() {
        let commit = "abc123..def456";
        let parts: Vec<&str> = commit.split("..").collect();
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0], "abc123");
        assert_eq!(parts[1], "def456");
    }
}
