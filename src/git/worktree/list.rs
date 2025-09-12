use super::info::{WorktreeInfo, WorktreeListOptions};
use std::path::PathBuf;
use tokio::process::Command;

/// 列出所有worktree（使用默认选项）
pub async fn list_worktrees() -> anyhow::Result<Vec<WorktreeInfo>> {
    list_worktrees_with_options(&WorktreeListOptions::default()).await
}

/// 使用指定选项列出worktree
pub async fn list_worktrees_with_options(
    options: &WorktreeListOptions,
) -> anyhow::Result<Vec<WorktreeInfo>> {
    let mut args = vec!["worktree", "list"];

    // 构建Git命令参数
    if options.verbose {
        args.push("-v");
    } else if options.porcelain {
        args.push("--porcelain");
    }

    if options.z {
        args.push("-z");
    }

    if let Some(expire) = &options.expire {
        args.push("--expire");
        args.push(expire);
    }

    let output = Command::new("git")
        .args(&args)
        .output()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to run git worktree list: {}", e))?;

    if !output.status.success() {
        anyhow::bail!(
            "Git worktree list failed with exit code: {:?}",
            output.status.code()
        );
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    if options.porcelain {
        parse_worktree_list(&stdout)
    } else {
        parse_worktree_list_verbose(&stdout, options.verbose)
    }
}

/// 获取原始worktree list输出
pub async fn list_worktrees_raw(options: &WorktreeListOptions) -> anyhow::Result<String> {
    let mut args = vec!["worktree", "list"];

    // 构建Git命令参数
    if options.verbose {
        args.push("-v");
    } else if options.porcelain {
        args.push("--porcelain");
    }

    if options.z {
        args.push("-z");
    }

    if let Some(expire) = &options.expire {
        args.push("--expire");
        args.push(expire);
    }

    let output = Command::new("git")
        .args(&args)
        .output()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to run git worktree list: {}", e))?;

    if !output.status.success() {
        anyhow::bail!(
            "Git worktree list failed with exit code: {:?}",
            output.status.code()
        );
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// 解析verbose模式的worktree list输出
fn parse_worktree_list_verbose(output: &str, _verbose: bool) -> anyhow::Result<Vec<WorktreeInfo>> {
    let mut worktrees = Vec::new();

    for line in output.lines() {
        if line.trim().is_empty() {
            continue;
        }

        // Verbose模式的格式示例：
        // /path/to/worktree  abc1234 [branch-name]
        // /path/to/worktree  abc1234 (bare)
        // /path/to/worktree  abc1234 (detached HEAD)

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 2 {
            continue;
        }

        let path = PathBuf::from(parts[0]);
        let commit = parts[1].to_string();

        let mut branch = String::new();
        let mut is_bare = false;
        let mut is_detached = false;

        if parts.len() > 2 {
            let remainder = parts[2..].join(" ");
            if remainder.contains("(bare)") {
                is_bare = true;
                branch = "bare".to_string();
            } else if remainder.contains("(detached HEAD)") {
                is_detached = true;
                branch = "detached".to_string();
            } else if remainder.starts_with('[') && remainder.ends_with(']') {
                branch = remainder
                    .trim_start_matches('[')
                    .trim_end_matches(']')
                    .to_string();
            } else {
                branch = remainder;
            }
        }

        worktrees.push(WorktreeInfo::new(
            path,
            branch,
            commit,
            is_bare,
            is_detached,
        ));
    }

    Ok(worktrees)
}

/// 解析porcelain模式的worktree list输出
fn parse_worktree_list(output: &str) -> anyhow::Result<Vec<WorktreeInfo>> {
    let mut worktrees = Vec::new();
    let mut current_path: Option<PathBuf> = None;
    let mut current_branch = String::new();
    let mut current_commit = String::new();
    let mut current_is_bare = false;
    let mut current_is_detached = false;

    for line in output.lines() {
        if line.starts_with("worktree ") {
            // Save previous worktree if exists
            if let Some(path) = current_path.take() {
                worktrees.push(WorktreeInfo::new(
                    path,
                    current_branch.clone(),
                    current_commit.clone(),
                    current_is_bare,
                    current_is_detached,
                ));
                // Reset for next worktree
                current_branch.clear();
                current_commit.clear();
                current_is_bare = false;
                current_is_detached = false;
            }

            let path_str = line.strip_prefix("worktree ").unwrap_or("");
            current_path = Some(PathBuf::from(path_str));
        } else if line.starts_with("HEAD ") {
            current_commit = line.strip_prefix("HEAD ").unwrap_or("").to_string();
        } else if line.starts_with("branch ") {
            current_branch = line.strip_prefix("branch ").unwrap_or("").to_string();
        } else if line == "bare" {
            current_is_bare = true;
        } else if line == "detached" {
            current_is_detached = true;
        }
    }

    // Add the last worktree if exists
    if let Some(path) = current_path {
        worktrees.push(WorktreeInfo::new(
            path,
            current_branch,
            current_commit,
            current_is_bare,
            current_is_detached,
        ));
    }

    Ok(worktrees)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_worktree_list_verbose() {
        let output = "/path/to/main  abc123 [main]\n/path/to/feature  def456 [feature/test]\n/path/to/bare  ghi789 (bare)\n/path/to/detached  jkl012 (detached HEAD)";

        let result = parse_worktree_list_verbose(output, true);
        assert!(result.is_ok());

        let worktrees = result.unwrap();
        assert_eq!(worktrees.len(), 4);

        assert_eq!(worktrees[0].branch, "main");
        assert_eq!(worktrees[1].branch, "feature/test");
        assert_eq!(worktrees[2].branch, "bare");
        assert!(worktrees[2].is_bare);
        assert_eq!(worktrees[3].branch, "detached");
        assert!(worktrees[3].is_detached);
    }

    #[test]
    fn test_parse_worktree_list_porcelain() {
        let output = "worktree /path/to/main\nHEAD abc123\nbranch refs/heads/main\n\nworktree /path/to/feature\nHEAD def456\nbranch refs/heads/feature\n\nworktree /path/to/bare\nHEAD ghi789\nbare\n";

        let result = parse_worktree_list(output);
        assert!(result.is_ok());

        let worktrees = result.unwrap();
        assert_eq!(worktrees.len(), 3);

        assert_eq!(worktrees[0].branch, "refs/heads/main");
        assert_eq!(worktrees[1].branch, "refs/heads/feature");
        assert!(worktrees[2].is_bare);
    }

    #[test]
    fn test_parse_worktree_list_verbose_empty() {
        let result = parse_worktree_list_verbose("", false);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[test]
    fn test_parse_worktree_list_verbose_malformed() {
        let output = "malformed line\n/path abc [branch";
        let result = parse_worktree_list_verbose(output, false);
        assert!(result.is_ok());
        // Should handle malformed lines gracefully
    }

    #[test]
    fn test_parse_worktree_list_porcelain_complex() {
        let output = "worktree /path/to/main\nHEAD abc123\nbranch refs/heads/main\n\nworktree /path/to/feature\nHEAD def456\nbranch refs/heads/feature/complex-name\n\nworktree /path/to/detached\nHEAD ghi789\ndetached\n";

        let result = parse_worktree_list(output);
        assert!(result.is_ok());

        let worktrees = result.unwrap();
        assert_eq!(worktrees.len(), 3);

        assert_eq!(worktrees[0].commit, "abc123");
        assert_eq!(worktrees[1].branch, "refs/heads/feature/complex-name");
        assert!(worktrees[2].is_detached);
    }

    #[test]
    fn test_parse_worktree_list_with_whitespace() {
        let output =
            "  /path/to/main  abc123  [main]  \n\n  /path/to/feature  def456  [feature/test]  \n";

        let result = parse_worktree_list_verbose(output, false);
        assert!(result.is_ok());

        let worktrees = result.unwrap();
        assert_eq!(worktrees.len(), 2);
        assert_eq!(worktrees[0].branch, "main");
        assert_eq!(worktrees[1].branch, "feature/test");
    }

    #[test]
    fn test_parse_worktree_list_special_paths() {
        let output =
            "/path with spaces/main  abc123 [main]\n/path/with-special@chars  def456 [feature]\n";

        let result = parse_worktree_list_verbose(output, false);
        assert!(result.is_ok());

        let worktrees = result.unwrap();
        assert_eq!(worktrees.len(), 2);
        assert_eq!(worktrees[0].path, PathBuf::from("/path with spaces/main"));
        assert_eq!(worktrees[1].path, PathBuf::from("/path/with-special@chars"));
    }

    #[test]
    fn test_worktree_list_options_combinations() {
        // 测试不同选项组合
        let mut options = WorktreeListOptions::default();
        assert!(!options.verbose);
        assert!(!options.porcelain);
        assert!(!options.z);
        assert!(options.expire.is_none());

        options.verbose = true;
        assert!(options.verbose);

        options.porcelain = true;
        options.z = true;
        options.expire = Some("2weeks".to_string());
        assert!(options.porcelain);
        assert!(options.z);
        assert_eq!(options.expire, Some("2weeks".to_string()));
    }

    #[test]
    fn test_parse_worktree_list_edge_cases() {
        // 测试只有路径没有其他信息
        let output = "/path/to/worktree\n";
        let result = parse_worktree_list_verbose(output, false);
        assert!(result.is_ok());

        // 测试只有HEAD没有分支
        let output = "/path/to/worktree abc123\n";
        let result = parse_worktree_list_verbose(output, false);
        assert!(result.is_ok());
        let worktrees = result.unwrap();
        assert_eq!(worktrees.len(), 1);
        assert_eq!(worktrees[0].commit, "abc123");
    }

    #[test]
    fn test_parse_worktree_list_porcelain_incomplete() {
        // 测试不完整的porcelain输出
        let output = "worktree /path/to/main\nHEAD abc123\n";
        let result = parse_worktree_list(output);
        assert!(result.is_ok());

        let worktrees = result.unwrap();
        assert_eq!(worktrees.len(), 1);
        assert_eq!(worktrees[0].path, PathBuf::from("/path/to/main"));
        assert_eq!(worktrees[0].commit, "abc123");
        assert_eq!(worktrees[0].branch, ""); // No branch specified
    }
}
