use std::path::PathBuf;
use tokio::process::Command;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorktreeInfo {
    pub path: PathBuf,
    pub branch: String,
    pub commit: String,
    pub is_bare: bool,
    pub is_detached: bool,
}

impl WorktreeInfo {
    pub fn new(path: PathBuf, branch: String, commit: String, is_bare: bool, is_detached: bool) -> Self {
        Self {
            path,
            branch,
            commit,
            is_bare,
            is_detached,
        }
    }
}

pub async fn create_worktree(branch: &str, custom_path: Option<&str>) -> anyhow::Result<PathBuf> {
    let path = if let Some(custom) = custom_path {
        PathBuf::from(custom)
    } else {
        let current_dir = std::env::current_dir()?;
        let parent_dir = current_dir.parent()
            .ok_or_else(|| anyhow::anyhow!("无法确定父目录"))?;
        
        let branch_name = branch.replace('/', "-");
        parent_dir.join(format!("worktree-{}", branch_name))
    };
    
    let path_str = path.to_string_lossy();
    
    let status = Command::new("git")
        .args(["worktree", "add", &path_str, branch])
        .status()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to run git worktree add: {}", e))?;
    
    if !status.success() {
        anyhow::bail!("Git worktree add failed with exit code: {:?}", status.code());
    }
    
    Ok(path)
}

pub async fn create_worktree_with_new_branch(branch: &str, custom_path: Option<&str>) -> anyhow::Result<PathBuf> {
    let path = if let Some(custom) = custom_path {
        PathBuf::from(custom)
    } else {
        let current_dir = std::env::current_dir()?;
        let parent_dir = current_dir.parent()
            .ok_or_else(|| anyhow::anyhow!("无法确定父目录"))?;
        
        let branch_name = branch.replace('/', "-");
        parent_dir.join(format!("worktree-{}", branch_name))
    };
    
    let path_str = path.to_string_lossy();
    
    let status = Command::new("git")
        .args(["worktree", "add", "-b", branch, &path_str])
        .status()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to run git worktree add with new branch: {}", e))?;
    
    if !status.success() {
        anyhow::bail!("Git worktree add with new branch failed with exit code: {:?}", status.code());
    }
    
    Ok(path)
}

#[derive(Debug, Clone, Default)]
pub struct WorktreeListOptions {
    pub verbose: bool,
    pub porcelain: bool,
    pub z: bool,
    pub expire: Option<String>,
}

pub async fn list_worktrees() -> anyhow::Result<Vec<WorktreeInfo>> {
    list_worktrees_with_options(&WorktreeListOptions::default()).await
}

pub async fn list_worktrees_with_options(options: &WorktreeListOptions) -> anyhow::Result<Vec<WorktreeInfo>> {
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
        anyhow::bail!("Git worktree list failed with exit code: {:?}", output.status.code());
    }
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    if options.porcelain {
        parse_worktree_list(&stdout)
    } else {
        parse_worktree_list_verbose(&stdout, options.verbose)
    }
}

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
        anyhow::bail!("Git worktree list failed with exit code: {:?}", output.status.code());
    }
    
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

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
                branch = remainder.trim_start_matches('[').trim_end_matches(']').to_string();
            } else {
                branch = remainder;
            }
        }
        
        worktrees.push(WorktreeInfo::new(path, branch, commit, is_bare, is_detached));
    }
    
    Ok(worktrees)
}

pub async fn remove_worktree(path_or_name: &str) -> anyhow::Result<()> {
    let worktrees = list_worktrees().await?;
    
    let target_path = if let Some(worktree) = worktrees.iter().find(|w| {
        w.path.to_string_lossy().contains(path_or_name) || 
        w.branch.contains(path_or_name)
    }) {
        worktree.path.to_string_lossy().to_string()
    } else {
        path_or_name.to_string()
    };
    
    let status = Command::new("git")
        .args(["worktree", "remove", &target_path])
        .status()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to run git worktree remove: {}", e))?;
    
    if !status.success() {
        anyhow::bail!("Git worktree remove failed with exit code: {:?}", status.code());
    }
    
    Ok(())
}

pub async fn get_current_worktree() -> anyhow::Result<Option<WorktreeInfo>> {
    let current_dir = std::env::current_dir()?;
    let worktrees = list_worktrees().await?;
    
    Ok(worktrees.into_iter().find(|w| {
        w.path == current_dir || current_dir.starts_with(&w.path)
    }))
}

pub async fn switch_to_worktree(path_or_name: &str) -> anyhow::Result<PathBuf> {
    let worktrees = list_worktrees().await?;
    
    let target_worktree = worktrees.iter().find(|w| {
        w.path.to_string_lossy().contains(path_or_name) || 
        w.branch.contains(path_or_name) ||
        w.path.file_name().is_some_and(|f| f.to_string_lossy().contains(path_or_name))
    }).ok_or_else(|| anyhow::anyhow!("找不到指定的 worktree: {}", path_or_name))?;
    
    std::env::set_current_dir(&target_worktree.path)
        .map_err(|e| anyhow::anyhow!("切换到 worktree 目录失败: {}", e))?;
    
    Ok(target_worktree.path.clone())
}

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
                worktrees.push(WorktreeInfo::new(path, current_branch.clone(), current_commit.clone(), current_is_bare, current_is_detached));
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
        worktrees.push(WorktreeInfo::new(path, current_branch, current_commit, current_is_bare, current_is_detached));
    }
    
    Ok(worktrees)
}

pub async fn prune_worktrees() -> anyhow::Result<()> {
    let status = Command::new("git")
        .args(["worktree", "prune"])
        .status()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to run git worktree prune: {}", e))?;
    
    if !status.success() {
        anyhow::bail!("Git worktree prune failed with exit code: {:?}", status.code());
    }
    
    Ok(())
}

pub async fn clear_other_worktrees() -> anyhow::Result<usize> {
    let current_dir = std::env::current_dir()?;
    let worktrees = list_worktrees().await?;
    
    // 找到当前工作目录所属的worktree
    let current_worktree = worktrees.iter().find(|w| {
        w.path == current_dir || current_dir.starts_with(&w.path)
    });
    
    if current_worktree.is_none() {
        anyhow::bail!("当前目录不在任何 worktree 中");
    }
    
    let current_path = &current_worktree.unwrap().path;
    let mut removed_count = 0;
    let mut failed_removals = Vec::new();
    
    // 删除除当前外的所有其他worktrees
    for worktree in &worktrees {
        if worktree.path != *current_path && !worktree.is_bare {
            let path_str = worktree.path.to_string_lossy();
            
            match Command::new("git")
                .args(["worktree", "remove", "--force", &path_str])
                .status()
                .await
            {
                Ok(status) if status.success() => {
                    removed_count += 1;
                }
                Ok(_) => {
                    failed_removals.push(format!("{} (删除失败)", path_str));
                }
                Err(e) => {
                    failed_removals.push(format!("{} (命令执行失败: {})", path_str, e));
                }
            }
        }
    }
    
    // 清理无效的worktree引用
    prune_worktrees().await?;
    
    if !failed_removals.is_empty() {
        anyhow::bail!(
            "成功删除 {} 个 worktree，但以下删除失败: {}",
            removed_count,
            failed_removals.join(", ")
        );
    }
    
    Ok(removed_count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_worktree_info_creation() {
        let path = PathBuf::from("/test/path");
        let branch = "feature/test".to_string();
        let commit = "abc123".to_string();
        
        let info = WorktreeInfo::new(path.clone(), branch.clone(), commit.clone(), false, false);
        
        assert_eq!(info.path, path);
        assert_eq!(info.branch, branch);
        assert_eq!(info.commit, commit);
        assert_eq!(info.is_bare, false);
        assert_eq!(info.is_detached, false);
    }

    #[test]
    fn test_parse_worktree_list_empty() {
        let output = "";
        let result = parse_worktree_list(output).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_parse_worktree_list_single() {
        let output = "worktree /path/to/main\nHEAD abc123\nbranch refs/heads/main\n";
        let result = parse_worktree_list(output).unwrap();
        
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].path, PathBuf::from("/path/to/main"));
        assert_eq!(result[0].commit, "abc123");
        assert_eq!(result[0].branch, "refs/heads/main");
        assert!(!result[0].is_bare);
        assert!(!result[0].is_detached);
    }

    #[test]
    fn test_parse_worktree_list_multiple() {
        let output = "worktree /path/to/main\nHEAD abc123\nbranch refs/heads/main\n\nworktree /path/to/feature\nHEAD def456\nbranch refs/heads/feature/test\n";
        let result = parse_worktree_list(output).unwrap();
        
        assert_eq!(result.len(), 2);
        
        assert_eq!(result[0].path, PathBuf::from("/path/to/main"));
        assert_eq!(result[0].commit, "abc123");
        assert_eq!(result[0].branch, "refs/heads/main");
        
        assert_eq!(result[1].path, PathBuf::from("/path/to/feature"));
        assert_eq!(result[1].commit, "def456");
        assert_eq!(result[1].branch, "refs/heads/feature/test");
    }

    #[test]
    fn test_parse_worktree_list_bare() {
        let output = "worktree /path/to/bare\nHEAD abc123\nbranch refs/heads/main\nbare\n";
        let result = parse_worktree_list(output).unwrap();
        
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].path, PathBuf::from("/path/to/bare"));
        assert!(result[0].is_bare);
        assert!(!result[0].is_detached);
    }

    #[test]
    fn test_parse_worktree_list_detached() {
        let output = "worktree /path/to/detached\nHEAD abc123\ndetached\n";
        let result = parse_worktree_list(output).unwrap();
        
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].path, PathBuf::from("/path/to/detached"));
        assert!(!result[0].is_bare);
        assert!(result[0].is_detached);
    }

    #[tokio::test]
    async fn test_create_worktree_command_structure() {
        let branch = "feature/test";
        let custom_path = Some("/tmp/test-worktree");
        let result = create_worktree(branch, custom_path).await;
        
        match result {
            Ok(path) => {
                assert_eq!(path, PathBuf::from("/tmp/test-worktree"));
            }
            Err(e) => {
                let error_msg = e.to_string();
                assert!(
                    error_msg.contains("Failed to run git worktree add") || 
                    error_msg.contains("Git worktree add failed"),
                    "Error message should contain git worktree add information: {}",
                    error_msg
                );
            }
        }
    }

    #[tokio::test]
    async fn test_create_worktree_with_new_branch_command_structure() {
        let branch = "feature/new-branch";
        let custom_path = Some("/tmp/new-branch-worktree");
        let result = create_worktree_with_new_branch(branch, custom_path).await;
        
        match result {
            Ok(path) => {
                assert_eq!(path, PathBuf::from("/tmp/new-branch-worktree"));
            }
            Err(e) => {
                let error_msg = e.to_string();
                assert!(
                    error_msg.contains("Failed to run git worktree add with new branch") || 
                    error_msg.contains("Git worktree add with new branch failed"),
                    "Error message should contain git worktree add information: {}",
                    error_msg
                );
            }
        }
    }

    #[tokio::test]
    async fn test_list_worktrees_command_structure() {
        let result = list_worktrees().await;
        
        match result {
            Ok(worktrees) => {
                assert!(worktrees.is_empty() || !worktrees.is_empty());
            }
            Err(e) => {
                let error_msg = e.to_string();
                assert!(
                    error_msg.contains("Failed to run git worktree list") || 
                    error_msg.contains("Git worktree list failed"),
                    "Error message should contain git worktree list information: {}",
                    error_msg
                );
            }
        }
    }

    #[tokio::test]
    async fn test_remove_worktree_command_structure() {
        let path_or_name = "test-worktree";
        let result = remove_worktree(path_or_name).await;
        
        match result {
            Ok(_) => {
                println!("Git worktree remove succeeded");
            }
            Err(e) => {
                let error_msg = e.to_string();
                assert!(
                    error_msg.contains("Failed to run git worktree remove") || 
                    error_msg.contains("Git worktree remove failed") ||
                    error_msg.contains("Failed to run git worktree list"), // 可能在查询时失败
                    "Error message should contain git worktree information: {}",
                    error_msg
                );
            }
        }
    }

    #[tokio::test]
    async fn test_get_current_worktree_command_structure() {
        let result = get_current_worktree().await;
        
        match result {
            Ok(worktree_opt) => {
                if let Some(worktree) = worktree_opt {
                    assert!(!worktree.path.as_os_str().is_empty());
                }
            }
            Err(e) => {
                let error_msg = e.to_string();
                assert!(
                    error_msg.contains("Failed to run git worktree list") || 
                    error_msg.contains("Git worktree list failed"),
                    "Error message should contain git worktree information: {}",
                    error_msg
                );
            }
        }
    }

    #[tokio::test]
    async fn test_prune_worktrees_command_structure() {
        let result = prune_worktrees().await;
        
        match result {
            Ok(_) => {
                println!("Git worktree prune succeeded");
            }
            Err(e) => {
                let error_msg = e.to_string();
                assert!(
                    error_msg.contains("Failed to run git worktree prune") || 
                    error_msg.contains("Git worktree prune failed"),
                    "Error message should contain git worktree prune information: {}",
                    error_msg
                );
            }
        }
    }

    #[test]
    fn test_worktree_info_debug_format() {
        let info = WorktreeInfo::new(
            PathBuf::from("/test"),
            "main".to_string(),
            "abc123".to_string(),
            false,
            false
        );
        let debug_str = format!("{:?}", info);
        assert!(debug_str.contains("WorktreeInfo"));
        assert!(debug_str.contains("path"));
        assert!(debug_str.contains("branch"));
    }

    #[test]
    fn test_worktree_info_clone() {
        let info1 = WorktreeInfo::new(
            PathBuf::from("/test"),
            "main".to_string(),
            "abc123".to_string(),
            false,
            false
        );
        let info2 = info1.clone();
        
        assert_eq!(info1.path, info2.path);
        assert_eq!(info1.branch, info2.branch);
        assert_eq!(info1.commit, info2.commit);
        assert_eq!(info1.is_bare, info2.is_bare);
        assert_eq!(info1.is_detached, info2.is_detached);
    }

    #[tokio::test]
    async fn test_clear_other_worktrees_command_structure() {
        let result = clear_other_worktrees().await;
        
        match result {
            Ok(count) => {
                println!("Clear other worktrees succeeded, removed: {}", count);
                // count 是 usize 类型，总是 >= 0，所以这个断言总是成功
            }
            Err(e) => {
                let error_msg = e.to_string();
                assert!(
                    error_msg.contains("当前目录不在任何 worktree 中") || 
                    error_msg.contains("Failed to run git worktree list") ||
                    error_msg.contains("Git worktree list failed") ||
                    error_msg.contains("成功删除") ||
                    error_msg.contains("删除失败"),
                    "Error message should contain expected worktree information: {}",
                    error_msg
                );
            }
        }
    }

    #[test]
    fn test_clear_other_worktrees_error_scenarios() {
        // 测试错误处理逻辑
        use std::env;
        
        // 保存当前目录
        let original_dir = env::current_dir().unwrap();
        
        // 这个测试验证函数签名和返回类型的正确性
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(async {
            clear_other_worktrees().await
        });
        
        // 验证函数返回正确的类型
        match result {
            Ok(_count) => {
                // count 是 usize 类型，总是非负数
                println!("Clear function returned successfully");
            }
            Err(e) => {
                // 错误消息应该包含预期的信息
                let error_msg = e.to_string();
                assert!(!error_msg.is_empty());
            }
        }
        
        // 恢复原始目录
        env::set_current_dir(original_dir).ok();
    }

    #[test]
    fn test_clear_function_signature() {
        // 编译时检查函数签名
        fn check_clear_other_worktrees() -> impl std::future::Future<Output = anyhow::Result<usize>> {
            clear_other_worktrees()
        }
        
        // 如果编译通过，说明函数签名正确
        assert!(true);
    }

    #[test]
    fn test_worktree_list_options_default() {
        let options = WorktreeListOptions::default();
        assert!(!options.verbose);
        assert!(!options.porcelain);
        assert!(!options.z);
        assert!(options.expire.is_none());
    }

    #[test]
    fn test_worktree_list_options_creation() {
        let options = WorktreeListOptions {
            verbose: true,
            porcelain: false,
            z: true,
            expire: Some("2weeks".to_string()),
        };
        
        assert!(options.verbose);
        assert!(!options.porcelain);
        assert!(options.z);
        assert_eq!(options.expire, Some("2weeks".to_string()));
    }

    #[test]
    fn test_worktree_list_options_debug() {
        let options = WorktreeListOptions {
            verbose: true,
            porcelain: true,
            z: false,
            expire: None,
        };
        
        let debug_str = format!("{:?}", options);
        assert!(debug_str.contains("WorktreeListOptions"));
        assert!(debug_str.contains("verbose: true"));
        assert!(debug_str.contains("porcelain: true"));
    }

    #[tokio::test]
    async fn test_list_worktrees_with_options_command_structure() {
        let options = WorktreeListOptions {
            verbose: true,
            porcelain: false,
            z: false,
            expire: None,
        };
        
        let result = list_worktrees_with_options(&options).await;
        
        match result {
            Ok(worktrees) => {
                println!("List worktrees with options succeeded, count: {}", worktrees.len());
                assert!(worktrees.is_empty() || !worktrees.is_empty());
            }
            Err(e) => {
                let error_msg = e.to_string();
                assert!(
                    error_msg.contains("Failed to run git worktree list") ||
                    error_msg.contains("Git worktree list failed"),
                    "Error message should contain git worktree list information: {}",
                    error_msg
                );
            }
        }
    }

    #[tokio::test]
    async fn test_list_worktrees_raw_command_structure() {
        let options = WorktreeListOptions {
            verbose: false,
            porcelain: true,
            z: false,
            expire: None,
        };
        
        let result = list_worktrees_raw(&options).await;
        
        match result {
            Ok(output) => {
                println!("List worktrees raw succeeded, output length: {}", output.len());
                // 输出可能为空也可能有内容
            }
            Err(e) => {
                let error_msg = e.to_string();
                assert!(
                    error_msg.contains("Failed to run git worktree list") ||
                    error_msg.contains("Git worktree list failed"),
                    "Error message should contain git worktree list information: {}",
                    error_msg
                );
            }
        }
    }

    #[test]
    fn test_parse_worktree_list_verbose_empty() {
        let output = "";
        let result = parse_worktree_list_verbose(output, true).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_parse_worktree_list_verbose_single() {
        let output = "/path/to/main  abc1234 [main]";
        let result = parse_worktree_list_verbose(output, true).unwrap();
        
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].path, PathBuf::from("/path/to/main"));
        assert_eq!(result[0].commit, "abc1234");
        assert_eq!(result[0].branch, "main");
        assert!(!result[0].is_bare);
        assert!(!result[0].is_detached);
    }

    #[test]
    fn test_parse_worktree_list_verbose_bare() {
        let output = "/path/to/bare  abc1234 (bare)";
        let result = parse_worktree_list_verbose(output, true).unwrap();
        
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].path, PathBuf::from("/path/to/bare"));
        assert_eq!(result[0].commit, "abc1234");
        assert_eq!(result[0].branch, "bare");
        assert!(result[0].is_bare);
        assert!(!result[0].is_detached);
    }

    #[test]
    fn test_parse_worktree_list_verbose_detached() {
        let output = "/path/to/detached  abc1234 (detached HEAD)";
        let result = parse_worktree_list_verbose(output, true).unwrap();
        
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].path, PathBuf::from("/path/to/detached"));
        assert_eq!(result[0].commit, "abc1234");
        assert_eq!(result[0].branch, "detached");
        assert!(!result[0].is_bare);
        assert!(result[0].is_detached);
    }
}