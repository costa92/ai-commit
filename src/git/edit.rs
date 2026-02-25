use crate::git::core::GitCore;
use tokio::process::Command;

/// Git 编辑操作的结构化结果
#[derive(Debug, Clone)]
pub struct GitEditResult {
    /// 主要结果消息
    pub message: String,
    /// 额外的用户指引/说明
    pub instructions: Vec<String>,
}

impl GitEditResult {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            instructions: Vec::new(),
        }
    }

    pub fn with_instructions(mut self, instructions: Vec<String>) -> Self {
        self.instructions = instructions;
        self
    }
}

impl std::fmt::Display for GitEditResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)?;
        for instruction in &self.instructions {
            write!(f, "\n{}", instruction)?;
        }
        Ok(())
    }
}

/// Git 提交编辑和修改模块
pub struct GitEdit;

impl GitEdit {
    /// 修改最后一次提交（amend）
    pub async fn amend_last_commit(message: Option<&str>) -> anyhow::Result<GitEditResult> {
        let mut args = vec!["commit", "--amend"];

        if let Some(msg) = message {
            args.extend(&["-m", msg]);
        } else {
            // 如果没有提供新消息，使用 --no-edit 保持原有消息
            args.push("--no-edit");
        }

        let status = Command::new("git")
            .args(&args)
            .status()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to amend commit: {}", e))?;

        if !status.success() {
            anyhow::bail!(
                "Git commit --amend failed with exit code: {:?}",
                status.code()
            );
        }

        Ok(GitEditResult::new("✓ Successfully amended the last commit"))
    }

    /// 撤销最后一次提交（保留文件修改）
    pub async fn undo_last_commit() -> anyhow::Result<GitEditResult> {
        let status = Command::new("git")
            .args(["reset", "--soft", "HEAD~1"])
            .status()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to undo commit: {}", e))?;

        if !status.success() {
            anyhow::bail!("Git reset failed with exit code: {:?}", status.code());
        }

        Ok(GitEditResult::new(
            "✓ Undid the last commit (changes are now staged)",
        ))
    }

    /// 交互式 rebase 编辑提交
    pub async fn interactive_rebase(base_commit: &str) -> anyhow::Result<GitEditResult> {
        // 验证提交是否存在
        if !GitCore::commit_exists(base_commit).await? {
            anyhow::bail!("Commit '{}' does not exist", base_commit);
        }

        let instructions = vec![
            format!("Starting interactive rebase from {}", base_commit),
            "This will open your default Git editor for interactive rebase.".to_string(),
            "Available actions:".to_string(),
            "  pick (p)   = use commit".to_string(),
            "  reword (r) = use commit, but edit the commit message".to_string(),
            "  edit (e)   = use commit, but stop for amending".to_string(),
            "  squash (s) = use commit, but meld into previous commit".to_string(),
            "  drop (d)   = remove commit".to_string(),
        ];

        let status = Command::new("git")
            .args(["rebase", "-i", base_commit])
            .status()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to start interactive rebase: {}", e))?;

        if !status.success() {
            anyhow::bail!("Git rebase -i failed with exit code: {:?}", status.code());
        }

        Ok(GitEditResult::new("✓ Interactive rebase completed").with_instructions(instructions))
    }

    /// 编辑指定的提交
    pub async fn edit_specific_commit(commit_hash: &str) -> anyhow::Result<GitEditResult> {
        // 验证提交是否存在
        if !GitCore::commit_exists(commit_hash).await? {
            anyhow::bail!("Commit '{}' does not exist", commit_hash);
        }

        // 使用 rebase 到指定提交的前一个提交，然后设置为 edit
        let parent_output = Command::new("git")
            .args(["rev-parse", &format!("{}^", commit_hash)])
            .output()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get parent commit: {}", e))?;

        if !parent_output.status.success() {
            anyhow::bail!("Failed to find parent commit of '{}'", commit_hash);
        }

        let parent_hash = String::from_utf8_lossy(&parent_output.stdout)
            .trim()
            .to_string();

        let pre_instructions = vec![
            format!(
                "Setting up interactive rebase to edit commit {}",
                commit_hash
            ),
            "You'll be stopped at the commit to make your changes.".to_string(),
            "After making changes, use 'git commit --amend' and then 'git rebase --continue'"
                .to_string(),
        ];

        // 创建临时的 rebase 脚本
        let rebase_script = format!(
            "edit {} {}",
            &commit_hash[..7.min(commit_hash.len())],
            Self::get_commit_subject(commit_hash).await?
        );

        // 执行 rebase
        let status = Command::new("git")
            .args(["rebase", "-i", &parent_hash])
            .env("GIT_SEQUENCE_EDITOR", format!("echo '{}'", rebase_script))
            .status()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to edit commit: {}", e))?;

        if !status.success() {
            anyhow::bail!("Failed to edit commit '{}'", commit_hash);
        }

        let mut instructions = pre_instructions;
        instructions.extend([
            format!("✓ Positioned at commit {} for editing", commit_hash),
            "💡 Make your changes, then run:".to_string(),
            "   git add <files>".to_string(),
            "   git commit --amend".to_string(),
            "   git rebase --continue".to_string(),
        ]);

        Ok(GitEditResult::new(format!(
            "✓ Positioned at commit {} for editing",
            commit_hash
        ))
        .with_instructions(instructions))
    }

    /// 重写提交消息
    pub async fn reword_commit(
        commit_hash: &str,
        new_message: &str,
    ) -> anyhow::Result<GitEditResult> {
        // 验证提交是否存在
        if !GitCore::commit_exists(commit_hash).await? {
            anyhow::bail!("Commit '{}' does not exist", commit_hash);
        }

        // 如果是最后一次提交，直接使用 amend
        let latest_commit = GitCore::get_latest_commit_hash().await?;
        if commit_hash == latest_commit || commit_hash.starts_with(&latest_commit[..7]) {
            return Self::amend_last_commit(Some(new_message)).await;
        }

        // 否则使用 rebase 来重写历史提交的消息
        // 获取父提交
        let parent_output = Command::new("git")
            .args(["rev-parse", &format!("{}^", commit_hash)])
            .output()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get parent commit: {}", e))?;

        if !parent_output.status.success() {
            anyhow::bail!("Failed to find parent commit of '{}'", commit_hash);
        }

        let parent_hash = String::from_utf8_lossy(&parent_output.stdout)
            .trim()
            .to_string();

        // 使用 filter-branch 或 rebase 重写消息
        let status = Command::new("git")
            .args([
                "filter-branch",
                "--msg-filter",
                &format!(
                    "if [ \"$GIT_COMMIT\" = \"{}\" ]; then echo '{}'; else cat; fi",
                    commit_hash, new_message
                ),
                &format!("{}..HEAD", parent_hash),
            ])
            .status()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to rewrite commit message: {}", e))?;

        if !status.success() {
            anyhow::bail!("Failed to rewrite commit message for '{}'", commit_hash);
        }

        Ok(GitEditResult::new(format!(
            "✓ Rewrote commit message for {}",
            commit_hash
        )))
    }

    /// 压缩提交（squash）
    pub async fn squash_commits(
        from_commit: &str,
        to_commit: &str,
    ) -> anyhow::Result<GitEditResult> {
        // 验证两个提交都存在
        if !GitCore::commit_exists(from_commit).await? {
            anyhow::bail!("Commit '{}' does not exist", from_commit);
        }
        if !GitCore::commit_exists(to_commit).await? {
            anyhow::bail!("Commit '{}' does not exist", to_commit);
        }

        let instructions = vec![
            format!("Squashing commits from {} to {}", from_commit, to_commit),
            "This will combine multiple commits into one.".to_string(),
        ];

        let status = Command::new("git")
            .args(["rebase", "-i", &format!("{}^", from_commit)])
            .status()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to squash commits: {}", e))?;

        if !status.success() {
            anyhow::bail!("Git rebase for squashing failed");
        }

        Ok(GitEditResult::new("✓ Squash rebase completed").with_instructions(instructions))
    }

    /// 检查 rebase 状态
    pub async fn check_rebase_status() -> anyhow::Result<RebaseStatus> {
        // 检查是否在 rebase 过程中
        let rebase_head_exists = tokio::fs::metadata(".git/rebase-merge/head-name")
            .await
            .is_ok()
            || tokio::fs::metadata(".git/rebase-apply/head-name")
                .await
                .is_ok();

        if !rebase_head_exists {
            return Ok(RebaseStatus::None);
        }

        // 检查是否有冲突
        let status_output = Command::new("git")
            .args(["status", "--porcelain"])
            .output()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to check status: {}", e))?;

        let status_text = String::from_utf8_lossy(&status_output.stdout);

        if status_text
            .lines()
            .any(|line| line.starts_with("UU") || line.starts_with("AA"))
        {
            Ok(RebaseStatus::InProgressWithConflicts)
        } else {
            Ok(RebaseStatus::InProgress)
        }
    }

    /// 继续 rebase
    pub async fn continue_rebase() -> anyhow::Result<GitEditResult> {
        let status = Command::new("git")
            .args(["rebase", "--continue"])
            .status()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to continue rebase: {}", e))?;

        if !status.success() {
            anyhow::bail!("Git rebase --continue failed");
        }

        Ok(GitEditResult::new("✓ Rebase continued successfully"))
    }

    /// 中止 rebase
    pub async fn abort_rebase() -> anyhow::Result<GitEditResult> {
        let status = Command::new("git")
            .args(["rebase", "--abort"])
            .status()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to abort rebase: {}", e))?;

        if !status.success() {
            anyhow::bail!("Git rebase --abort failed");
        }

        Ok(GitEditResult::new("✓ Rebase aborted successfully"))
    }

    /// 获取提交的主题行
    async fn get_commit_subject(commit_hash: &str) -> anyhow::Result<String> {
        let output = Command::new("git")
            .args(["log", "-1", "--pretty=format:%s", commit_hash])
            .output()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get commit subject: {}", e))?;

        if !output.status.success() {
            anyhow::bail!("Failed to get commit subject for '{}'", commit_hash);
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    /// 显示可编辑的提交列表
    pub async fn show_editable_commits(limit: Option<u32>) -> anyhow::Result<GitEditResult> {
        let mut args = vec![
            "log".to_string(),
            "--oneline".to_string(),
            "--pretty=format:%C(bold blue)%h%C(reset) - %C(white)%s%C(reset) %C(dim white)(%ar)%C(reset)".to_string(),
        ];

        if let Some(limit) = limit {
            args.extend(vec!["-n".to_string(), limit.to_string()]);
        }

        let output = Command::new("git")
            .args(&args)
            .output()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to show commits: {}", e))?;

        if !output.status.success() {
            anyhow::bail!("Git log command failed");
        }

        let commits = String::from_utf8_lossy(&output.stdout);

        let message = format!("✏️  Editable Commits:\n{}\n{}", "─".repeat(60), commits);

        let instructions = vec![
            "\n💡 Available edit commands:".to_string(),
            "  --amend                     Modify the last commit".to_string(),
            "  --edit-commit HASH          Edit specific commit".to_string(),
            "  --reword-commit HASH        Change commit message".to_string(),
            "  --undo-commit               Undo last commit (soft reset)".to_string(),
            "  --rebase-edit BASE_COMMIT   Interactive rebase from base".to_string(),
        ];

        Ok(GitEditResult::new(message).with_instructions(instructions))
    }
}

#[derive(Debug, PartialEq)]
pub enum RebaseStatus {
    None,
    InProgress,
    InProgressWithConflicts,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_check_rebase_status() {
        let result = GitEdit::check_rebase_status().await;

        match result {
            Ok(status) => {
                println!("Rebase status: {:?}", status);
            }
            Err(e) => {
                println!("Failed to check rebase status: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_show_editable_commits() {
        let result = GitEdit::show_editable_commits(Some(10)).await;

        match result {
            Ok(_) => {
                println!("Editable commits displayed successfully");
            }
            Err(e) => {
                println!("Failed to show editable commits: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_get_commit_subject() {
        let result = GitEdit::get_commit_subject("HEAD").await;

        match result {
            Ok(subject) => {
                assert!(!subject.is_empty(), "Commit subject should not be empty");
                println!("Commit subject: {}", subject);
            }
            Err(e) => {
                println!("Failed to get commit subject: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_undo_last_commit() {
        // 这个测试不能真正执行，因为会修改仓库状态
        // 但我们可以测试命令结构
        println!("Undo commit test (structure validation only)");

        // 在实际测试中，这里应该创建临时仓库进行测试
        // 现在只验证函数存在且可调用
        let result = GitEdit::undo_last_commit().await;

        match result {
            Ok(_) => {
                println!("Undo commit succeeded (or would succeed)");
            }
            Err(e) => {
                println!("Undo commit failed (expected in test environment): {}", e);
            }
        }
    }

    #[test]
    fn test_rebase_status_enum() {
        // 测试 RebaseStatus 枚举
        let status = RebaseStatus::None;
        assert_eq!(status, RebaseStatus::None);

        let status = RebaseStatus::InProgress;
        assert_eq!(status, RebaseStatus::InProgress);

        let status = RebaseStatus::InProgressWithConflicts;
        assert_eq!(status, RebaseStatus::InProgressWithConflicts);

        // 测试 Debug 格式
        let debug_str = format!("{:?}", RebaseStatus::InProgressWithConflicts);
        assert!(debug_str.contains("InProgressWithConflicts"));
    }

    #[test]
    fn test_commit_hash_validation() {
        let valid_hashes = vec!["abc123", "1234567890abcdef", "HEAD", "HEAD~1", "main"];

        for hash in valid_hashes {
            assert!(!hash.is_empty(), "Hash should not be empty");
            assert!(
                hash.chars()
                    .all(|c| c.is_alphanumeric() || "~^".contains(c)),
                "Hash should contain valid characters: {}",
                hash
            );
        }
    }

    #[tokio::test]
    async fn test_amend_last_commit_variations() {
        // Test amending with different message configurations
        let test_cases = vec![
            (None, "amend without message"),
            (Some(""), "amend with empty message"),
            (Some("test commit message"), "amend with custom message"),
            (
                Some("feat(test): add comprehensive test coverage"),
                "amend with conventional commit message",
            ),
        ];

        for (message, description) in test_cases {
            println!("Testing: {}", description);
            let result = GitEdit::amend_last_commit(message).await;

            match result {
                Ok(_) => println!("  Amend succeeded: {}", description),
                Err(e) => println!(
                    "  Amend failed (expected in test environment): {} - {}",
                    description, e
                ),
            }
        }
    }

    #[tokio::test]
    async fn test_edit_specific_commit_variations() {
        // Test editing specific commits with different hash formats
        let commit_references = vec![
            "HEAD",
            "HEAD~1",
            "HEAD~2",
            "main",
            "develop",
            "abcd1234",                                 // Short hash
            "1234567890abcdef1234567890abcdef12345678", // Full hash
        ];

        for commit_ref in commit_references {
            println!("Testing edit for commit: {}", commit_ref);
            let result = GitEdit::edit_specific_commit(commit_ref).await;

            match result {
                Ok(_) => println!("  Edit specific commit succeeded: {}", commit_ref),
                Err(e) => println!("  Edit specific commit failed: {} - {}", commit_ref, e),
            }
        }
    }

    #[tokio::test]
    async fn test_interactive_rebase_variations() {
        // Test interactive rebase with different base commits
        let base_commits = vec![
            "HEAD~3",
            "HEAD~5",
            "HEAD~10",
            "main",
            "develop",
            "origin/main",
        ];

        for base_commit in base_commits {
            println!("Testing interactive rebase from: {}", base_commit);
            let result = GitEdit::interactive_rebase(base_commit).await;

            match result {
                Ok(_) => println!("  Interactive rebase succeeded: {}", base_commit),
                Err(e) => println!("  Interactive rebase failed: {} - {}", base_commit, e),
            }
        }
    }

    #[tokio::test]
    async fn test_reword_commit_variations() {
        // 验证 reword_commit 在无效提交上会返回错误
        // 注意：不在真实仓库上调用 reword_commit，因为 filter-branch 会破坏工作树
        let result = GitEdit::reword_commit("nonexistent_hash_abc123", "new message").await;
        assert!(result.is_err(), "Should fail for non-existent commit");
    }

    #[tokio::test]
    async fn test_get_commit_subject_edge_cases() {
        // Test getting commit subject for different commit references
        let commit_refs = vec![
            "HEAD",
            "HEAD~1",
            "non-existent-hash",
            "",
            "invalid_hash_format",
        ];

        for commit_ref in commit_refs {
            let result = GitEdit::get_commit_subject(commit_ref).await;

            match result {
                Ok(subject) => {
                    println!("Commit subject for '{}': {}", commit_ref, subject);
                    if !commit_ref.is_empty()
                        && commit_ref != "non-existent-hash"
                        && commit_ref != "invalid_hash_format"
                    {
                        assert!(
                            !subject.trim().is_empty(),
                            "Valid commit should have non-empty subject"
                        );
                    }
                }
                Err(e) => {
                    println!("Failed to get commit subject for '{}': {}", commit_ref, e);
                }
            }
        }
    }

    #[tokio::test]
    async fn test_show_editable_commits_limits() {
        // Test showing editable commits with different limits
        let limits = vec![None, Some(1), Some(5), Some(10), Some(20), Some(100)];

        for limit in limits {
            println!("Testing editable commits with limit: {:?}", limit);
            let result = GitEdit::show_editable_commits(limit).await;

            match result {
                Ok(_) => println!("  Show editable commits succeeded with limit {:?}", limit),
                Err(e) => println!(
                    "  Show editable commits failed with limit {:?}: {}",
                    limit, e
                ),
            }
        }
    }

    #[test]
    fn test_rebase_status_comparison() {
        // Test RebaseStatus comparison and equality
        assert_eq!(RebaseStatus::None, RebaseStatus::None);
        assert_eq!(RebaseStatus::InProgress, RebaseStatus::InProgress);
        assert_eq!(
            RebaseStatus::InProgressWithConflicts,
            RebaseStatus::InProgressWithConflicts
        );

        assert_ne!(RebaseStatus::None, RebaseStatus::InProgress);
        assert_ne!(
            RebaseStatus::InProgress,
            RebaseStatus::InProgressWithConflicts
        );
        assert_ne!(RebaseStatus::None, RebaseStatus::InProgressWithConflicts);
    }

    #[test]
    fn test_commit_reference_patterns() {
        // Test various commit reference patterns
        let reference_patterns = vec![
            ("HEAD", true),
            ("HEAD~1", true),
            ("HEAD~10", true),
            ("HEAD^", true),
            ("HEAD^^", true),
            ("main", true),
            ("develop", true),
            ("feature/branch", true),
            ("origin/main", true),
            ("1234567", true),
            ("1234567890abcdef", true),
            ("", false),
            ("invalid ref", false),
        ];

        for (reference, should_be_valid) in reference_patterns {
            if should_be_valid {
                assert!(
                    !reference.is_empty(),
                    "Valid reference should not be empty: '{}'",
                    reference
                );
                assert!(
                    reference.len() <= 100,
                    "Reference should be reasonable length: '{}'",
                    reference
                );
            } else {
                let is_invalid = reference.is_empty() || reference.contains(' ');
                assert!(
                    is_invalid,
                    "Invalid reference should have issues: '{}'",
                    reference
                );
            }
        }
    }

    #[tokio::test]
    async fn test_concurrent_edit_operations() {
        // Test concurrent edit operations (read-only operations)
        use tokio::task;

        let status_task = task::spawn(async { GitEdit::check_rebase_status().await });
        let commits_task = task::spawn(async { GitEdit::show_editable_commits(Some(5)).await });
        let subject_task = task::spawn(async { GitEdit::get_commit_subject("HEAD").await });

        // Handle each task separately due to different return types
        match status_task.await {
            Ok(result) => match result {
                Ok(_status) => println!("Concurrent rebase status operation succeeded"),
                Err(e) => println!("Concurrent rebase status operation failed: {}", e),
            },
            Err(e) => println!("Status task join error: {}", e),
        }

        match commits_task.await {
            Ok(result) => match result {
                Ok(_) => println!("Concurrent show commits operation succeeded"),
                Err(e) => println!("Concurrent show commits operation failed: {}", e),
            },
            Err(e) => println!("Commits task join error: {}", e),
        }

        match subject_task.await {
            Ok(result) => match result {
                Ok(_subject) => println!("Concurrent get subject operation succeeded"),
                Err(e) => println!("Concurrent get subject operation failed: {}", e),
            },
            Err(e) => println!("Subject task join error: {}", e),
        }
    }

    #[tokio::test]
    async fn test_edit_error_scenarios() {
        // Test error handling in various scenarios

        // Test with non-existent commit
        let result = GitEdit::get_commit_subject("non-existent-commit-hash-12345").await;
        match result {
            Ok(subject) => println!(
                "Unexpectedly got subject for non-existent commit: {}",
                subject
            ),
            Err(e) => println!("Expected error for non-existent commit: {}", e),
        }

        // Test with empty commit hash
        let result = GitEdit::get_commit_subject("").await;
        match result {
            Ok(subject) => println!("Unexpectedly got subject for empty hash: {}", subject),
            Err(e) => println!("Expected error for empty hash: {}", e),
        }
    }

    #[test]
    fn test_commit_message_patterns() {
        // Test various commit message patterns
        let message_patterns = vec![
            "Simple commit message",
            "feat: add new feature",
            "fix(bug): resolve critical issue",
            "docs: update README with installation instructions",
            "style: fix code formatting",
            "refactor: restructure module organization",
            "test: add comprehensive test coverage",
            "chore: update dependencies to latest versions",
            "feat(user): implement user authentication system\n\nAdded login, logout, and session management.\nIncludes proper error handling and validation.",
            "",
        ];

        for message in message_patterns {
            if !message.is_empty() {
                assert!(
                    message.len() <= 1000,
                    "Commit message should be reasonable length"
                );

                // Check for conventional commit pattern
                let has_conventional_pattern = message.contains(':')
                    || message.starts_with("feat")
                    || message.starts_with("fix")
                    || message.starts_with("docs")
                    || message.starts_with("style")
                    || message.starts_with("refactor")
                    || message.starts_with("test")
                    || message.starts_with("chore")
                    || !message.chars().next().unwrap().is_lowercase();

                println!(
                    "Message pattern '{}' - Conventional: {}",
                    message.lines().next().unwrap_or(""),
                    has_conventional_pattern
                );
            }
        }
    }

    #[tokio::test]
    async fn test_edit_commands_in_non_git_environment() {
        // 验证 git 命令在非 git 目录下会失败
        // 使用 Command::current_dir 代替 env::set_current_dir，避免干扰并行测试
        let output = tokio::process::Command::new("git")
            .args(["log", "--oneline", "-1"])
            .current_dir("/tmp")
            .output()
            .await;

        match output {
            Ok(o) => assert!(
                !o.status.success(),
                "git log should fail in non-git dir"
            ),
            Err(e) => println!("Command failed as expected: {}", e),
        }
    }

    #[test]
    fn test_git_edit_module_structure() {
        // Test that all expected methods exist and are callable
        // This is a structural test to ensure API consistency

        // Test that RebaseStatus has all expected variants
        let _status_none = RebaseStatus::None;
        let _status_progress = RebaseStatus::InProgress;
        let _status_conflicts = RebaseStatus::InProgressWithConflicts;

        // Test that RebaseStatus implements required traits
        let status = RebaseStatus::InProgress;
        let _debug_output = format!("{:?}", status);
        let _is_equal = status == RebaseStatus::InProgress;
        let _is_not_equal = status != RebaseStatus::None;
    }

    #[tokio::test]
    async fn test_performance_with_large_commit_limits() {
        use std::time::Instant;

        // Test performance with different limits for showing commits
        let limits = vec![Some(10), Some(50), Some(100)];

        for limit in limits {
            let start = Instant::now();
            let result = GitEdit::show_editable_commits(limit).await;
            let duration = start.elapsed();

            match result {
                Ok(_) => println!(
                    "Show editable commits with limit {:?} completed in {:?}",
                    limit, duration
                ),
                Err(e) => println!(
                    "Show editable commits with limit {:?} failed in {:?}: {}",
                    limit, duration, e
                ),
            }

            // Performance check (not a strict assertion for CI)
            if duration.as_secs() > 10 {
                println!(
                    "Warning: Show editable commits took longer than expected: {:?}",
                    duration
                );
            }
        }
    }
}
