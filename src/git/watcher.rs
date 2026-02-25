use std::time::Duration;
use tokio::time::sleep;
use crate::git::core::GitCore;

/// Git仓库监控器，类似GRV的实时更新功能
pub struct GitWatcher;

#[derive(Debug, Clone)]
pub struct RepoStatus {
    pub current_branch: String,
    pub staged_files: u32,
    pub unstaged_files: u32,
    pub untracked_files: u32,
    pub ahead_count: u32,
    pub behind_count: u32,
    pub latest_commit: String,
    pub is_clean: bool,
}

#[derive(Debug, Clone)]
pub struct ChangeEvent {
    pub event_type: ChangeType,
    pub description: String,
    pub timestamp: chrono::DateTime<chrono::Local>,
}

#[derive(Debug, Clone)]
pub enum ChangeType {
    NewCommit,
    BranchSwitch,
    FileStaged,
    FileUnstaged,
    FileAdded,
    FileDeleted,
    RemoteUpdate,
}

impl GitWatcher {
    /// 开始监控仓库变化
    pub async fn start_watching(interval_seconds: u64) -> anyhow::Result<()> {
        println!("👀 Starting repository monitoring (interval: {}s)", interval_seconds);
        println!("Press Ctrl+C to stop watching");
        println!("{}", "─".repeat(60));

        let mut last_status = Self::get_repo_status().await?;
        Self::display_status(&last_status);

        loop {
            sleep(Duration::from_secs(interval_seconds)).await;

            match Self::get_repo_status().await {
                Ok(current_status) => {
                    let changes = Self::detect_changes(&last_status, &current_status);
                    
                    if !changes.is_empty() {
                        println!("\n🔄 Changes detected:");
                        for change in changes {
                            Self::display_change(&change);
                        }
                        println!();
                        Self::display_status(&current_status);
                    }

                    last_status = current_status;
                }
                Err(e) => {
                    eprintln!("❌ Error checking repository status: {}", e);
                }
            }
        }
    }

    /// 获取仓库状态
    pub async fn get_repo_status() -> anyhow::Result<RepoStatus> {
        let current_branch = GitCore::get_current_branch().await.unwrap_or_else(|_| "unknown".to_string());
        let is_clean = GitCore::is_working_tree_clean().await.unwrap_or(false);
        let latest_commit = GitCore::get_latest_commit_hash().await.unwrap_or_else(|_| "unknown".to_string());

        // 获取文件状态统计
        let (staged_files, unstaged_files, untracked_files) = Self::get_file_counts().await?;

        // 获取远程跟踪信息
        let (ahead_count, behind_count) = Self::get_remote_tracking_info().await?;

        Ok(RepoStatus {
            current_branch,
            staged_files,
            unstaged_files,
            untracked_files,
            ahead_count,
            behind_count,
            latest_commit,
            is_clean,
        })
    }

    /// 获取文件状态统计
    async fn get_file_counts() -> anyhow::Result<(u32, u32, u32)> {
        let output = tokio::process::Command::new("git")
            .args(["status", "--porcelain"])
            .output()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get status: {}", e))?;

        if !output.status.success() {
            return Ok((0, 0, 0));
        }

        let status_output = String::from_utf8_lossy(&output.stdout);
        let mut staged = 0;
        let mut unstaged = 0;
        let mut untracked = 0;

        for line in status_output.lines() {
            if line.len() >= 2 {
                let index_status = line.chars().nth(0).unwrap_or(' ');
                let worktree_status = line.chars().nth(1).unwrap_or(' ');

                if index_status != ' ' && index_status != '?' {
                    staged += 1;
                }
                if worktree_status != ' ' && worktree_status != '?' {
                    unstaged += 1;
                }
                if line.starts_with("??") {
                    untracked += 1;
                }
            }
        }

        Ok((staged, unstaged, untracked))
    }

    /// 获取远程跟踪信息
    async fn get_remote_tracking_info() -> anyhow::Result<(u32, u32)> {
        let output = tokio::process::Command::new("git")
            .args(["rev-list", "--count", "--left-right", "HEAD...@{upstream}"])
            .output()
            .await;

        match output {
            Ok(output) if output.status.success() => {
                let count_output = String::from_utf8_lossy(&output.stdout);
                let parts: Vec<&str> = count_output.trim().split('\t').collect();
                
                if parts.len() >= 2 {
                    let ahead = parts[0].parse::<u32>().unwrap_or(0);
                    let behind = parts[1].parse::<u32>().unwrap_or(0);
                    return Ok((ahead, behind));
                }
            }
            _ => {
                // 没有上游分支或其他错误
            }
        }

        Ok((0, 0))
    }

    /// 检测状态变化
    fn detect_changes(old_status: &RepoStatus, new_status: &RepoStatus) -> Vec<ChangeEvent> {
        let mut changes = Vec::new();
        let now = chrono::Local::now();

        // 检测分支切换
        if old_status.current_branch != new_status.current_branch {
            changes.push(ChangeEvent {
                event_type: ChangeType::BranchSwitch,
                description: format!(
                    "Switched from '{}' to '{}'",
                    old_status.current_branch,
                    new_status.current_branch
                ),
                timestamp: now,
            });
        }

        // 检测新提交
        if old_status.latest_commit != new_status.latest_commit {
            changes.push(ChangeEvent {
                event_type: ChangeType::NewCommit,
                description: format!(
                    "New commit: {}",
                    &new_status.latest_commit[..8.min(new_status.latest_commit.len())]
                ),
                timestamp: now,
            });
        }

        // 检测暂存区变化
        if old_status.staged_files != new_status.staged_files {
            changes.push(ChangeEvent {
                event_type: ChangeType::FileStaged,
                description: format!(
                    "Staged files: {} -> {}",
                    old_status.staged_files,
                    new_status.staged_files
                ),
                timestamp: now,
            });
        }

        // 检测工作区变化
        if old_status.unstaged_files != new_status.unstaged_files {
            changes.push(ChangeEvent {
                event_type: ChangeType::FileUnstaged,
                description: format!(
                    "Unstaged files: {} -> {}",
                    old_status.unstaged_files,
                    new_status.unstaged_files
                ),
                timestamp: now,
            });
        }

        // 检测未跟踪文件变化
        if old_status.untracked_files != new_status.untracked_files {
            let change_type = if new_status.untracked_files > old_status.untracked_files {
                ChangeType::FileAdded
            } else {
                ChangeType::FileDeleted
            };

            changes.push(ChangeEvent {
                event_type: change_type,
                description: format!(
                    "Untracked files: {} -> {}",
                    old_status.untracked_files,
                    new_status.untracked_files
                ),
                timestamp: now,
            });
        }

        // 检测远程跟踪变化
        if old_status.ahead_count != new_status.ahead_count || old_status.behind_count != new_status.behind_count {
            changes.push(ChangeEvent {
                event_type: ChangeType::RemoteUpdate,
                description: format!(
                    "Remote tracking: ahead {} behind {} -> ahead {} behind {}",
                    old_status.ahead_count,
                    old_status.behind_count,
                    new_status.ahead_count,
                    new_status.behind_count
                ),
                timestamp: now,
            });
        }

        changes
    }

    /// 显示仓库状态
    pub fn display_status(status: &RepoStatus) {
        println!("📊 Repository Status:");
        println!("{}", "─".repeat(40));
        
        // 分支信息
        println!("🌿 Branch: {}", status.current_branch);
        
        // 提交信息
        let commit_short = if status.latest_commit.len() > 8 {
            &status.latest_commit[..8]
        } else {
            &status.latest_commit
        };
        println!("📝 Latest commit: {}", commit_short);
        
        // 远程跟踪信息
        if status.ahead_count > 0 || status.behind_count > 0 {
            println!("🔄 Remote: ahead {}, behind {}", status.ahead_count, status.behind_count);
        }

        // 文件状态
        if status.is_clean && status.staged_files == 0 && status.untracked_files == 0 {
            println!("✅ Working tree clean");
        } else {
            if status.staged_files > 0 {
                println!("📦 Staged files: {}", status.staged_files);
            }
            if status.unstaged_files > 0 {
                println!("📝 Unstaged files: {}", status.unstaged_files);
            }
            if status.untracked_files > 0 {
                println!("❓ Untracked files: {}", status.untracked_files);
            }
        }
    }

    /// 显示变化事件
    fn display_change(change: &ChangeEvent) {
        let icon = match change.event_type {
            ChangeType::NewCommit => "📝",
            ChangeType::BranchSwitch => "🌿",
            ChangeType::FileStaged => "📦",
            ChangeType::FileUnstaged => "📄",
            ChangeType::FileAdded => "➕",
            ChangeType::FileDeleted => "➖",
            ChangeType::RemoteUpdate => "🔄",
        };

        println!(
            "  {} {} ({})",
            icon,
            change.description,
            change.timestamp.format("%H:%M:%S")
        );
    }

    /// 一次性状态检查
    pub async fn check_status() -> anyhow::Result<()> {
        let status = Self::get_repo_status().await?;
        Self::display_status(&status);
        Ok(())
    }

    /// 检查仓库是否需要关注
    pub async fn needs_attention() -> anyhow::Result<Vec<String>> {
        let status = Self::get_repo_status().await?;
        let mut notifications = Vec::new();

        // 检查是否有未提交的更改
        if status.staged_files > 0 {
            notifications.push(format!("You have {} staged files ready to commit", status.staged_files));
        }

        // 检查是否落后于远程
        if status.behind_count > 0 {
            notifications.push(format!("Your branch is {} commits behind upstream", status.behind_count));
        }

        // 检查是否领先于远程
        if status.ahead_count > 0 {
            notifications.push(format!("Your branch is {} commits ahead of upstream", status.ahead_count));
        }

        // 检查未跟踪文件
        if status.untracked_files > 5 {
            notifications.push(format!("You have {} untracked files", status.untracked_files));
        }

        Ok(notifications)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_repo_status() {
        let result = GitWatcher::get_repo_status().await;
        
        match result {
            Ok(status) => {
                assert!(!status.current_branch.is_empty());
                assert!(!status.latest_commit.is_empty());
                println!("Repository status retrieved successfully");
                println!("Branch: {}", status.current_branch);
                println!("Clean: {}", status.is_clean);
            }
            Err(e) => {
                println!("Repository status failed (expected in non-git environment): {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_get_file_counts() {
        let result = GitWatcher::get_file_counts().await;
        
        match result {
            Ok((staged, unstaged, untracked)) => {
                println!("File counts: {} staged, {} unstaged, {} untracked", staged, unstaged, untracked);
            }
            Err(e) => {
                println!("File counts failed: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_check_status() {
        let result = GitWatcher::check_status().await;
        
        match result {
            Ok(_) => {
                println!("Status check completed successfully");
            }
            Err(e) => {
                println!("Status check failed: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_needs_attention() {
        let result = GitWatcher::needs_attention().await;
        
        match result {
            Ok(notifications) => {
                println!("Attention check completed: {} notifications", notifications.len());
                for notification in notifications {
                    println!("  - {}", notification);
                }
            }
            Err(e) => {
                println!("Attention check failed: {}", e);
            }
        }
    }

    #[test]
    fn test_detect_changes() {
        let old_status = RepoStatus {
            current_branch: "main".to_string(),
            staged_files: 0,
            unstaged_files: 1,
            untracked_files: 0,
            ahead_count: 0,
            behind_count: 0,
            latest_commit: "abc123".to_string(),
            is_clean: false,
        };

        let new_status = RepoStatus {
            current_branch: "feature/test".to_string(),
            staged_files: 1,
            unstaged_files: 0,
            untracked_files: 1,
            ahead_count: 1,
            behind_count: 0,
            latest_commit: "def456".to_string(),
            is_clean: false,
        };

        let changes = GitWatcher::detect_changes(&old_status, &new_status);
        
        // 应该检测到多个变化
        assert!(!changes.is_empty());
        
        // 检查是否检测到分支切换
        assert!(changes.iter().any(|c| matches!(c.event_type, ChangeType::BranchSwitch)));
        
        // 检查是否检测到新提交
        assert!(changes.iter().any(|c| matches!(c.event_type, ChangeType::NewCommit)));

        println!("Detected {} changes", changes.len());
        for change in changes {
            println!("  {:?}: {}", change.event_type, change.description);
        }
    }

    #[test]
    fn test_display_functions() {
        let status = RepoStatus {
            current_branch: "test-branch".to_string(),
            staged_files: 2,
            unstaged_files: 1,
            untracked_files: 0,
            ahead_count: 0,
            behind_count: 1,
            latest_commit: "abcd1234efgh5678".to_string(),
            is_clean: false,
        };

        // Test display functions (should not panic)
        GitWatcher::display_status(&status);

        let change = ChangeEvent {
            event_type: ChangeType::NewCommit,
            description: "Test commit".to_string(),
            timestamp: chrono::Local::now(),
        };

        GitWatcher::display_change(&change);
    }

    #[test]
    fn test_change_detection_edge_cases() {
        // Test no changes
        let identical_status = RepoStatus {
            current_branch: "main".to_string(),
            staged_files: 1,
            unstaged_files: 1,
            untracked_files: 1,
            ahead_count: 0,
            behind_count: 0,
            latest_commit: "abc123".to_string(),
            is_clean: false,
        };

        let changes = GitWatcher::detect_changes(&identical_status, &identical_status);
        assert!(changes.is_empty(), "Identical status should produce no changes");

        // Test only clean status change
        let mut clean_status = identical_status.clone();
        clean_status.is_clean = true;

        let changes = GitWatcher::detect_changes(&identical_status, &clean_status);
        assert!(!changes.is_empty(), "Clean status change should be detected");

        // Test large file count changes
        let mut large_change_status = identical_status.clone();
        large_change_status.staged_files = 100;
        large_change_status.unstaged_files = 50;
        large_change_status.untracked_files = 25;

        let changes = GitWatcher::detect_changes(&identical_status, &large_change_status);
        assert!(!changes.is_empty(), "Large file changes should be detected");
    }

    #[test]
    fn test_change_type_detection() {
        let base_status = RepoStatus {
            current_branch: "main".to_string(),
            staged_files: 0,
            unstaged_files: 0,
            untracked_files: 0,
            ahead_count: 0,
            behind_count: 0,
            latest_commit: "base123".to_string(),
            is_clean: true,
        };

        // Test branch switch detection
        let mut branch_status = base_status.clone();
        branch_status.current_branch = "feature/new".to_string();
        let changes = GitWatcher::detect_changes(&base_status, &branch_status);
        assert!(changes.iter().any(|c| matches!(c.event_type, ChangeType::BranchSwitch)));

        // Test new commit detection
        let mut commit_status = base_status.clone();
        commit_status.latest_commit = "new456".to_string();
        let changes = GitWatcher::detect_changes(&base_status, &commit_status);
        assert!(changes.iter().any(|c| matches!(c.event_type, ChangeType::NewCommit)));

        // Test file changes detection
        let mut file_status = base_status.clone();
        file_status.staged_files = 5;
        file_status.unstaged_files = 3;
        file_status.untracked_files = 2;
        let changes = GitWatcher::detect_changes(&base_status, &file_status);
        assert!(changes.iter().any(|c| matches!(c.event_type, ChangeType::FileStaged)));

        // Test sync status detection
        let mut sync_status = base_status.clone();
        sync_status.ahead_count = 2;
        sync_status.behind_count = 1;
        let changes = GitWatcher::detect_changes(&base_status, &sync_status);
        assert!(changes.iter().any(|c| matches!(c.event_type, ChangeType::NewCommit)));
    }

    #[test]
    fn test_repo_status_equality() {
        let status1 = RepoStatus {
            current_branch: "main".to_string(),
            staged_files: 1,
            unstaged_files: 2,
            untracked_files: 3,
            ahead_count: 0,
            behind_count: 1,
            latest_commit: "abc123".to_string(),
            is_clean: false,
        };

        let status2 = status1.clone();
        assert_eq!(status1.current_branch, status2.current_branch);
        assert_eq!(status1.staged_files, status2.staged_files);
        assert_eq!(status1.latest_commit, status2.latest_commit);

        // Test with different values
        let mut status3 = status1.clone();
        status3.current_branch = "feature".to_string();
        assert_ne!(status1.current_branch, status3.current_branch);
    }

    #[tokio::test]
    async fn test_watcher_error_scenarios() {
        // 验证 git 命令在非 git 目录下会失败
        // 使用 Command::current_dir 代替 env::set_current_dir，避免干扰并行测试
        let output = tokio::process::Command::new("git")
            .args(["status", "--porcelain"])
            .current_dir("/tmp")
            .output()
            .await;

        match output {
            Ok(o) => assert!(
                !o.status.success(),
                "git status should fail in non-git dir"
            ),
            Err(e) => println!("Command failed as expected: {}", e),
        }
    }

    #[test]
    fn test_change_event_properties() {
        let now = chrono::Local::now();
        let change = ChangeEvent {
            event_type: ChangeType::FileStaged,
            description: "Test change description".to_string(),
            timestamp: now,
        };

        assert!(matches!(change.event_type, ChangeType::FileStaged));
        assert_eq!(change.description, "Test change description");
        assert_eq!(change.timestamp, now);
    }

    #[test]
    fn test_change_type_variants() {
        // Test all ChangeType variants exist
        let change_types = vec![
            ChangeType::BranchSwitch,
            ChangeType::NewCommit,
            ChangeType::FileStaged,
            ChangeType::FileUnstaged,
        ];

        for change_type in change_types {
            let change = ChangeEvent {
                event_type: change_type.clone(),
                description: format!("Test {:?} change", change_type),
                timestamp: chrono::Local::now(),
            };

            // Ensure the change can be displayed without panic
            GitWatcher::display_change(&change);
        }
    }

    #[tokio::test]
    async fn test_concurrent_watcher_operations() {
        // Test multiple concurrent watcher operations
        use tokio::task;

        let status_task = task::spawn(async { GitWatcher::get_repo_status().await });
        let counts_task = task::spawn(async { GitWatcher::get_file_counts().await });
        let check_task = task::spawn(async { GitWatcher::check_status().await });
        let attention_task = task::spawn(async { GitWatcher::needs_attention().await });

        // Handle each task separately due to different return types
        match status_task.await {
            Ok(result) => {
                match result {
                    Ok(_status) => println!("Concurrent repo status operation succeeded"),
                    Err(e) => println!("Concurrent repo status operation failed: {}", e),
                }
            }
            Err(e) => println!("Status task join error: {}", e),
        }

        match counts_task.await {
            Ok(result) => {
                match result {
                    Ok(_counts) => println!("Concurrent file counts operation succeeded"),
                    Err(e) => println!("Concurrent file counts operation failed: {}", e),
                }
            }
            Err(e) => println!("Counts task join error: {}", e),
        }

        match check_task.await {
            Ok(result) => {
                match result {
                    Ok(_) => println!("Concurrent check status operation succeeded"),
                    Err(e) => println!("Concurrent check status operation failed: {}", e),
                }
            }
            Err(e) => println!("Check task join error: {}", e),
        }

        match attention_task.await {
            Ok(result) => {
                match result {
                    Ok(_notifications) => println!("Concurrent needs attention operation succeeded"),
                    Err(e) => println!("Concurrent needs attention operation failed: {}", e),
                }
            }
            Err(e) => println!("Attention task join error: {}", e),
        }
    }

    #[test]
    fn test_repo_status_fields() {
        let status = RepoStatus {
            current_branch: "develop".to_string(),
            staged_files: 5,
            unstaged_files: 10,
            untracked_files: 3,
            ahead_count: 2,
            behind_count: 1,
            latest_commit: "1234567890abcdef".to_string(),
            is_clean: false,
        };

        // Test all fields are accessible and have expected values
        assert_eq!(status.current_branch, "develop");
        assert_eq!(status.staged_files, 5);
        assert_eq!(status.unstaged_files, 10);
        assert_eq!(status.untracked_files, 3);
        assert_eq!(status.ahead_count, 2);
        assert_eq!(status.behind_count, 1);
        assert_eq!(status.latest_commit, "1234567890abcdef");
        assert!(!status.is_clean);

        // Test clean repository
        let clean_status = RepoStatus {
            current_branch: "main".to_string(),
            staged_files: 0,
            unstaged_files: 0,
            untracked_files: 0,
            ahead_count: 0,
            behind_count: 0,
            latest_commit: "clean123".to_string(),
            is_clean: true,
        };

        assert!(clean_status.is_clean);
        assert_eq!(clean_status.staged_files + clean_status.unstaged_files + clean_status.untracked_files, 0);
    }

    #[test]
    fn test_comprehensive_change_scenarios() {
        // Test complex multi-change scenario
        let initial_status = RepoStatus {
            current_branch: "main".to_string(),
            staged_files: 1,
            unstaged_files: 2,
            untracked_files: 1,
            ahead_count: 0,
            behind_count: 0,
            latest_commit: "initial123".to_string(),
            is_clean: false,
        };

        let final_status = RepoStatus {
            current_branch: "feature/complex".to_string(),
            staged_files: 5,
            unstaged_files: 0,
            untracked_files: 3,
            ahead_count: 3,
            behind_count: 1,
            latest_commit: "final456".to_string(),
            is_clean: false,
        };

        let changes = GitWatcher::detect_changes(&initial_status, &final_status);
        
        // Should detect multiple types of changes
        assert!(!changes.is_empty());
        
        let change_types: std::collections::HashSet<_> = changes.iter()
            .map(|c| std::mem::discriminant(&c.event_type))
            .collect();
        
        // Should have detected multiple different types of changes
        assert!(change_types.len() >= 2, "Should detect multiple change types");

        println!("Complex scenario detected {} changes across {} types", 
                 changes.len(), change_types.len());
    }

    #[tokio::test]
    async fn test_start_watching_timeout() {
        // Test that start_watching can be interrupted
        use tokio::time::{timeout, Duration};

        let watching_future = GitWatcher::start_watching(1);
        
        // Set a short timeout to test interruption
        let result = timeout(Duration::from_millis(100), watching_future).await;
        
        match result {
            Ok(watch_result) => {
                match watch_result {
                    Ok(_) => println!("Watching completed unexpectedly quickly"),
                    Err(e) => println!("Watching failed: {}", e),
                }
            }
            Err(_) => {
                // Timeout occurred, which is expected for the watching loop
                println!("Watching timeout occurred as expected");
            }
        }
    }
}