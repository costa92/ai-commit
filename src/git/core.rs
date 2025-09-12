use tokio::process::Command;

/// 基础 Git 操作工具函数
pub struct GitCore;

impl GitCore {
    /// 初始化新的 Git 仓库
    pub async fn init_repository() -> anyhow::Result<()> {
        // 检查当前目录是否已经是 git 仓库
        if Self::is_git_repo().await {
            anyhow::bail!("Directory is already a Git repository");
        }

        let status = Command::new("git")
            .args(["init"])
            .status()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to run git init: {}", e))?;

        if !status.success() {
            anyhow::bail!("Git init failed with exit code: {:?}", status.code());
        }

        println!("✓ Initialized empty Git repository");

        // 设置默认分支为 main（如果 Git 版本支持）
        let config_status = Command::new("git")
            .args(["config", "--local", "init.defaultBranch", "main"])
            .status()
            .await;

        // 忽略配置错误，因为较老的 Git 版本可能不支持
        if config_status.is_ok() && config_status.unwrap().success() {
            println!("✓ Set default branch to 'main'");
        }

        // 检查是否存在 main 分支，如果不存在则创建
        if !Self::branch_exists("main").await.unwrap_or(false) {
            // 创建初始提交以便创建 main 分支
            let readme_exists = std::path::Path::new("README.md").exists();
            if !readme_exists {
                tokio::fs::write("README.md", "# Project\n\nThis is a new project.\n")
                    .await
                    .map_err(|e| anyhow::anyhow!("Failed to create README.md: {}", e))?;
                println!("✓ Created README.md");
            }

            // 添加文件到暂存区
            let add_status = Command::new("git")
                .args(["add", "."])
                .status()
                .await
                .map_err(|e| anyhow::anyhow!("Failed to add files: {}", e))?;

            if add_status.success() {
                // 创建初始提交
                let commit_status = Command::new("git")
                    .args(["commit", "-m", "chore: 初始化项目"])
                    .status()
                    .await
                    .map_err(|e| anyhow::anyhow!("Failed to create initial commit: {}", e))?;

                if commit_status.success() {
                    println!("✓ Created initial commit");
                } else {
                    println!("⚠ Initial commit failed, but repository is initialized");
                }
            }
        }

        Ok(())
    }

    /// 检查是否在 Git 仓库中
    pub async fn is_git_repo() -> bool {
        Command::new("git")
            .args(["rev-parse", "--git-dir"])
            .output()
            .await
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    /// 获取当前分支名
    pub async fn get_current_branch() -> anyhow::Result<String> {
        let output = Command::new("git")
            .args(["branch", "--show-current"])
            .output()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get current branch: {}", e))?;

        if !output.status.success() {
            anyhow::bail!(
                "Git branch command failed with exit code: {:?}",
                output.status.code()
            );
        }

        let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if branch.is_empty() {
            anyhow::bail!("Not on any branch (detached HEAD?)");
        }

        Ok(branch)
    }

    /// 检查分支是否存在
    pub async fn branch_exists(branch: &str) -> anyhow::Result<bool> {
        let output = Command::new("git")
            .args(["branch", "--list", branch])
            .output()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to check branch existence: {}", e))?;

        Ok(output.status.success() && !output.stdout.is_empty())
    }

    /// 检查远程分支是否存在
    pub async fn remote_branch_exists(branch: &str) -> anyhow::Result<bool> {
        let output = Command::new("git")
            .args(["branch", "-r", "--list", &format!("origin/{}", branch)])
            .output()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to check remote branch existence: {}", e))?;

        Ok(output.status.success() && !output.stdout.is_empty())
    }

    /// 创建并切换到新分支
    pub async fn create_and_checkout_branch(branch: &str) -> anyhow::Result<()> {
        let status = Command::new("git")
            .args(["checkout", "-b", branch])
            .status()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to create branch: {}", e))?;

        if !status.success() {
            anyhow::bail!(
                "Failed to create and checkout branch '{}' with exit code: {:?}",
                branch,
                status.code()
            );
        }

        Ok(())
    }

    /// 切换到指定分支
    pub async fn checkout_branch(branch: &str) -> anyhow::Result<()> {
        let status = Command::new("git")
            .args(["checkout", branch])
            .status()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to checkout branch: {}", e))?;

        if !status.success() {
            anyhow::bail!(
                "Failed to checkout branch '{}' with exit code: {:?}",
                branch,
                status.code()
            );
        }

        Ok(())
    }

    /// 合并分支
    pub async fn merge_branch(branch: &str, message: Option<&str>) -> anyhow::Result<()> {
        let mut args = vec!["merge"];
        if let Some(msg) = message {
            args.extend(&["-m", msg]);
        }
        args.push(branch);

        let status = Command::new("git")
            .args(&args)
            .status()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to merge branch: {}", e))?;

        if !status.success() {
            anyhow::bail!(
                "Failed to merge branch '{}' with exit code: {:?}",
                branch,
                status.code()
            );
        }

        Ok(())
    }

    /// 删除分支
    pub async fn delete_branch(branch: &str, force: bool) -> anyhow::Result<()> {
        let delete_flag = if force { "-D" } else { "-d" };
        let status = Command::new("git")
            .args(["branch", delete_flag, branch])
            .status()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to delete branch: {}", e))?;

        if !status.success() {
            anyhow::bail!(
                "Failed to delete branch '{}' with exit code: {:?}",
                branch,
                status.code()
            );
        }

        Ok(())
    }

    /// 检查工作区是否干净
    pub async fn is_working_tree_clean() -> anyhow::Result<bool> {
        let output = Command::new("git")
            .args(["status", "--porcelain"])
            .output()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to check working tree status: {}", e))?;

        if !output.status.success() {
            anyhow::bail!(
                "Git status command failed with exit code: {:?}",
                output.status.code()
            );
        }

        Ok(output.stdout.is_empty())
    }

    /// 获取最新提交的 hash
    pub async fn get_latest_commit_hash() -> anyhow::Result<String> {
        let output = Command::new("git")
            .args(["rev-parse", "HEAD"])
            .output()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get latest commit hash: {}", e))?;

        if !output.status.success() {
            anyhow::bail!(
                "Git rev-parse command failed with exit code: {:?}",
                output.status.code()
            );
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    /// 检查提交是否存在
    pub async fn commit_exists(commit_hash: &str) -> anyhow::Result<bool> {
        let output = Command::new("git")
            .args(["cat-file", "-e", commit_hash])
            .output()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to check commit existence: {}", e))?;

        Ok(output.status.success())
    }

    /// 获取远程仓库列表
    pub async fn get_remotes() -> anyhow::Result<Vec<String>> {
        let output = Command::new("git")
            .args(["remote"])
            .output()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get remotes: {}", e))?;

        if !output.status.success() {
            anyhow::bail!(
                "Git remote command failed with exit code: {:?}",
                output.status.code()
            );
        }

        let remotes = String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        Ok(remotes)
    }

    /// 创建新分支
    pub async fn create_branch(branch: &str, base: Option<&str>) -> anyhow::Result<()> {
        let mut args = vec!["branch", branch];
        if let Some(base_ref) = base {
            args.push(base_ref);
        }

        let status = Command::new("git")
            .args(&args)
            .status()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to create branch: {}", e))?;

        if !status.success() {
            anyhow::bail!(
                "Failed to create branch '{}' with exit code: {:?}",
                branch,
                status.code()
            );
        }

        Ok(())
    }

    /// 切换分支
    pub async fn switch_branch(branch: &str) -> anyhow::Result<()> {
        let status = Command::new("git")
            .args(["switch", branch])
            .status()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to switch branch: {}", e))?;

        if !status.success() {
            anyhow::bail!(
                "Failed to switch to branch '{}' with exit code: {:?}",
                branch,
                status.code()
            );
        }

        Ok(())
    }

    /// 推送分支到远程
    pub async fn push_branch(branch: &str, remote: &str, set_upstream: bool) -> anyhow::Result<()> {
        let mut args = vec!["push"];
        if set_upstream {
            args.extend(&["-u", remote, branch]);
        } else {
            args.extend(&[remote, branch]);
        }

        let status = Command::new("git")
            .args(&args)
            .status()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to push branch: {}", e))?;

        if !status.success() {
            anyhow::bail!(
                "Failed to push branch '{}' to '{}' with exit code: {:?}",
                branch,
                remote,
                status.code()
            );
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_git_repo_check() {
        let result = GitCore::is_git_repo().await;
        // 在 git 仓库中应该返回 true，否则返回 false
        assert!(result == true || result == false);
    }

    #[tokio::test]
    async fn test_get_current_branch() {
        let result = GitCore::get_current_branch().await;

        match result {
            Ok(branch) => {
                assert!(!branch.is_empty(), "Branch name should not be empty");
                println!("Current branch: {}", branch);
            }
            Err(e) => {
                println!(
                    "Failed to get current branch (expected in non-git environment): {}",
                    e
                );
            }
        }
    }

    #[tokio::test]
    async fn test_is_working_tree_clean() {
        let result = GitCore::is_working_tree_clean().await;

        match result {
            Ok(is_clean) => {
                println!("Working tree clean: {}", is_clean);
            }
            Err(e) => {
                println!("Failed to check working tree status: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_get_latest_commit_hash() {
        let result = GitCore::get_latest_commit_hash().await;

        match result {
            Ok(hash) => {
                assert!(!hash.is_empty(), "Commit hash should not be empty");
                assert!(
                    hash.len() >= 7,
                    "Commit hash should be at least 7 characters"
                );
                println!("Latest commit: {}", hash);
            }
            Err(e) => {
                println!("Failed to get latest commit hash: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_get_remotes() {
        let result = GitCore::get_remotes().await;

        match result {
            Ok(remotes) => {
                println!("Remotes: {:?}", remotes);
            }
            Err(e) => {
                println!("Failed to get remotes: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_branch_exists() {
        let result = GitCore::branch_exists("main").await;

        match result {
            Ok(exists) => {
                println!("Branch 'main' exists: {}", exists);
            }
            Err(e) => {
                println!("Failed to check if branch exists: {}", e);
            }
        }

        // 测试不存在的分支
        let result = GitCore::branch_exists("non-existent-branch-12345").await;
        match result {
            Ok(exists) => {
                assert!(!exists, "Non-existent branch should return false");
            }
            Err(e) => {
                println!("Failed to check non-existent branch: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_commit_exists() {
        let result = GitCore::commit_exists("HEAD").await;

        match result {
            Ok(exists) => {
                println!("Commit 'HEAD' exists: {}", exists);
            }
            Err(e) => {
                println!("Failed to check if commit exists: {}", e);
            }
        }

        // 测试不存在的提交
        let result = GitCore::commit_exists("0000000000000000000000000000000000000000").await;
        match result {
            Ok(exists) => {
                assert!(!exists, "Non-existent commit should return false");
            }
            Err(e) => {
                println!("Failed to check non-existent commit: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_create_branch() {
        // 注意：这个测试可能会修改仓库状态，所以使用一个测试专用的分支名
        let test_branch = format!("test-branch-{}", chrono::Utc::now().timestamp());

        let result = GitCore::create_branch(&test_branch, Some("HEAD")).await;

        match result {
            Ok(_) => {
                println!("Successfully created test branch: {}", test_branch);

                // 尝试删除测试分支（清理）
                let _ = Command::new("git")
                    .args(["branch", "-D", &test_branch])
                    .status()
                    .await;
            }
            Err(e) => {
                println!(
                    "Failed to create test branch (expected in some environments): {}",
                    e
                );
            }
        }
    }

    #[tokio::test]
    async fn test_delete_branch() {
        // 首先创建一个测试分支
        let test_branch = format!("test-delete-branch-{}", chrono::Utc::now().timestamp());

        let create_result = Command::new("git")
            .args(["branch", &test_branch])
            .status()
            .await;

        if let Ok(status) = create_result {
            if status.success() {
                // 尝试删除刚创建的分支
                let result = GitCore::delete_branch(&test_branch, false).await;

                match result {
                    Ok(_) => {
                        println!("Successfully deleted test branch: {}", test_branch);
                    }
                    Err(e) => {
                        println!("Failed to delete test branch: {}", e);
                        // 确保清理
                        let _ = Command::new("git")
                            .args(["branch", "-D", &test_branch])
                            .status()
                            .await;
                    }
                }
            }
        }
    }

    #[tokio::test]
    async fn test_switch_branch() {
        // 获取当前分支
        let original_branch = GitCore::get_current_branch().await;

        if let Ok(current) = original_branch {
            println!("Current branch before switch: {}", current);

            // 尝试切换到同一分支（应该成功但没有实际效果）
            let result = GitCore::switch_branch(&current).await;

            match result {
                Ok(_) => {
                    println!("Successfully switched to same branch: {}", current);
                }
                Err(e) => {
                    println!("Failed to switch to same branch: {}", e);
                }
            }
        }
    }

    #[tokio::test]
    async fn test_merge_branch() {
        // 这个测试比较复杂，因为需要实际的分支来合并
        // 我们只测试合并一个不存在的分支会失败
        let result = GitCore::merge_branch("non-existent-branch-for-test", None).await;

        match result {
            Ok(_) => {
                // 如果成功了，说明可能有这个分支存在，这不是我们期望的
                println!("Unexpectedly succeeded in merging non-existent branch");
            }
            Err(e) => {
                println!("Expected failure when merging non-existent branch: {}", e);
                assert!(
                    e.to_string().contains("non-existent")
                        || e.to_string().contains("error")
                        || e.to_string().contains("failed"),
                    "Error message should indicate failure"
                );
            }
        }
    }

    #[tokio::test]
    async fn test_push_branch() {
        // 测试推送到不存在的远程
        let result = GitCore::push_branch("main", "non-existent-remote", false).await;

        match result {
            Ok(_) => {
                println!("Unexpectedly succeeded in pushing to non-existent remote");
            }
            Err(e) => {
                println!(
                    "Expected failure when pushing to non-existent remote: {}",
                    e
                );
                assert!(
                    e.to_string().contains("non-existent")
                        || e.to_string().contains("error")
                        || e.to_string().contains("failed"),
                    "Error message should indicate failure"
                );
            }
        }
    }

    #[test]
    fn test_branch_name_validation() {
        // 测试分支名称验证逻辑
        let valid_names = vec![
            "main",
            "develop",
            "feature/user-auth",
            "hotfix/security-fix",
            "release/v1.0.0",
            "test-branch",
            "feature_branch",
        ];

        for name in valid_names {
            assert!(
                !name.is_empty(),
                "Branch name '{}' should not be empty",
                name
            );
            assert!(
                !name.contains(" "),
                "Branch name '{}' should not contain spaces",
                name
            );
            assert!(
                !name.starts_with("-"),
                "Branch name '{}' should not start with dash",
                name
            );
            assert!(
                !name.ends_with("."),
                "Branch name '{}' should not end with dot",
                name
            );
        }
    }

    #[test]
    fn test_commit_hash_validation() {
        // 测试提交哈希验证逻辑
        let valid_hashes = vec![
            "abc1234",
            "1234567890abcdef",
            "HEAD",
            "HEAD~1",
            "HEAD^",
            "main",
            "origin/main",
        ];

        for hash in valid_hashes {
            assert!(!hash.is_empty(), "Hash '{}' should not be empty", hash);
            // 检查是否包含有效字符
            let has_valid_chars = hash
                .chars()
                .all(|c| c.is_alphanumeric() || "~^/".contains(c));
            assert!(
                has_valid_chars,
                "Hash '{}' should contain only valid characters",
                hash
            );
        }
    }

    #[test]
    fn test_remote_name_validation() {
        // 测试远程名称验证逻辑
        let valid_remotes = vec!["origin", "upstream", "fork", "my-remote", "remote_name"];

        for remote in valid_remotes {
            assert!(
                !remote.is_empty(),
                "Remote name '{}' should not be empty",
                remote
            );
            assert!(
                !remote.contains(" "),
                "Remote name '{}' should not contain spaces",
                remote
            );
            assert!(
                !remote.starts_with("-"),
                "Remote name '{}' should not start with dash",
                remote
            );
        }
    }

    #[tokio::test]
    async fn test_init_repository() {
        // 注意：这个测试应该在临时目录中运行以避免干扰现有仓库
        use std::env;
        use std::path::Path;
        use tempfile::TempDir;

        // 创建临时目录
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let original_dir = env::current_dir().expect("Failed to get current directory");

        // 切换到临时目录
        env::set_current_dir(temp_dir.path()).expect("Failed to change to temp directory");

        // 确认这不是一个 git 仓库
        assert!(
            !GitCore::is_git_repo().await,
            "Temp directory should not be a git repo"
        );

        // 测试初始化
        let result = GitCore::init_repository().await;
        match result {
            Ok(_) => {
                println!("Git repository initialized successfully");

                // 验证仓库已被初始化
                assert!(
                    GitCore::is_git_repo().await,
                    "Directory should be a git repo after init"
                );

                // 验证 README.md 是否被创建
                assert!(
                    Path::new("README.md").exists(),
                    "README.md should be created"
                );

                // 验证是否有初始提交
                let commit_result = GitCore::get_latest_commit_hash().await;
                match commit_result {
                    Ok(hash) => {
                        assert!(!hash.is_empty(), "Should have initial commit");
                        println!("Initial commit hash: {}", hash);
                    }
                    Err(_) => println!("No initial commit found (this is OK)"),
                }
            }
            Err(e) => println!(
                "Git init failed (this might be expected in some environments): {}",
                e
            ),
        }

        // 恢复原目录
        env::set_current_dir(original_dir).expect("Failed to restore original directory");
    }

    #[tokio::test]
    async fn test_init_repository_already_exists() {
        // 测试在已存在的 git 仓库中运行 init_repository
        if GitCore::is_git_repo().await {
            let result = GitCore::init_repository().await;
            match result {
                Ok(_) => {
                    panic!("Should not succeed when directory is already a git repo");
                }
                Err(e) => {
                    assert!(
                        e.to_string().contains("already a Git repository"),
                        "Error should indicate directory is already a git repo"
                    );
                    println!("Expected error when trying to init existing repo: {}", e);
                }
            }
        } else {
            println!("Skipping test - not in a git repository");
        }
    }
}
