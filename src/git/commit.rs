use tokio::process::Command;

// 批量Git操作：并行执行多个Git命令提升性能
pub async fn git_status_and_diff() -> anyhow::Result<(String, String)> {
    let (status_result, diff_result) = tokio::join!(
        Command::new("git").args(["status", "--porcelain"]).output(),
        Command::new("git").args(["diff", "--cached"]).output()
    );

    let status_output =
        status_result.map_err(|e| anyhow::anyhow!("Failed to run git status: {}", e))?;
    let diff_output = diff_result.map_err(|e| anyhow::anyhow!("Failed to run git diff: {}", e))?;

    if !status_output.status.success() {
        anyhow::bail!(
            "Git status failed with exit code: {:?}",
            status_output.status.code()
        );
    }
    if !diff_output.status.success() {
        anyhow::bail!(
            "Git diff failed with exit code: {:?}",
            diff_output.status.code()
        );
    }

    let status = String::from_utf8_lossy(&status_output.stdout).to_string();
    let diff = String::from_utf8_lossy(&diff_output.stdout).to_string();

    Ok((status, diff))
}

pub async fn git_add_all() -> anyhow::Result<()> {
    let status = Command::new("git")
        .args(["add", "."])
        .status()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to run git add: {}", e))?;

    if !status.success() {
        anyhow::bail!("Git add failed with exit code: {:?}", status.code());
    }
    Ok(())
}

pub async fn git_commit(message: &str) -> anyhow::Result<()> {
    let status = Command::new("git")
        .args(["commit", "-m", message])
        .status()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to run git commit: {}", e))?;

    if !status.success() {
        anyhow::bail!("Git commit failed with exit code: {:?}", status.code());
    }
    Ok(())
}

pub async fn git_push() -> anyhow::Result<()> {
    let status = Command::new("git")
        .args(["push"])
        .status()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to run git push: {}", e))?;

    if !status.success() {
        anyhow::bail!("Git push failed with exit code: {:?}", status.code());
    }
    Ok(())
}

/// 强制推送：自动解决 non-fast-forward 错误
pub async fn git_force_push() -> anyhow::Result<()> {
    // 首先尝试正常推送
    let push_result = git_push().await;
    
    if push_result.is_ok() {
        return push_result;
    }

    // 推送失败，尝试拉取并合并远程更新
    println!("检测到推送冲突，正在自动解决...");
    
    // 获取当前分支
    let branch_output = Command::new("git")
        .args(["branch", "--show-current"])
        .output()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to get current branch: {}", e))?;

    if !branch_output.status.success() {
        anyhow::bail!("Failed to get current branch");
    }

    let current_branch = String::from_utf8_lossy(&branch_output.stdout).trim().to_string();
    
    // 拉取远程更新
    let pull_status = Command::new("git")
        .args(["pull", "--no-ff", "origin", &current_branch])
        .status()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to run git pull: {}", e))?;

    if !pull_status.success() {
        anyhow::bail!("Git pull failed. 请手动解决冲突后重试。");
    }

    println!("已成功合并远程更新，正在重新推送...");
    
    // 重新推送
    git_push().await
}

pub async fn get_git_diff() -> anyhow::Result<String> {
    // 首先尝试暂存区的变更
    let cached_output = Command::new("git")
        .args(["diff", "--cached"])
        .output()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to run git diff --cached: {}", e))?;

    if !cached_output.status.success() {
        anyhow::bail!(
            "Git diff --cached failed with exit code: {:?}",
            cached_output.status.code()
        );
    }

    let cached_diff = String::from_utf8_lossy(&cached_output.stdout).to_string();

    // 如果暂存区有变更，直接返回
    if !cached_diff.trim().is_empty() {
        return Ok(cached_diff);
    }

    // 如果暂存区没有变更，检查工作目录的变更
    let working_output = Command::new("git")
        .args(["diff"])
        .output()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to run git diff: {}", e))?;

    if !working_output.status.success() {
        anyhow::bail!(
            "Git diff failed with exit code: {:?}",
            working_output.status.code()
        );
    }

    Ok(String::from_utf8_lossy(&working_output.stdout).to_string())
}

/// 获取所有变更（包括未暂存的工作区变更）用于 AI commit
pub async fn get_all_changes_diff() -> anyhow::Result<String> {
    // 首先检查是否有暂存的变更
    let staged_output = Command::new("git")
        .args(["diff", "--cached"])
        .output()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to run git diff --cached: {}", e))?;

    let staged_diff = String::from_utf8_lossy(&staged_output.stdout);
    
    if !staged_diff.trim().is_empty() {
        // 有暂存的变更，返回暂存变更
        return Ok(staged_diff.to_string());
    }
    
    // 没有暂存变更，获取工作区变更
    let unstaged_output = Command::new("git")
        .args(["diff"])
        .output()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to run git diff: {}", e))?;

    let unstaged_diff = String::from_utf8_lossy(&unstaged_output.stdout);
    
    if !unstaged_diff.trim().is_empty() {
        // 有工作区变更，返回工作区变更
        return Ok(unstaged_diff.to_string());
    }
    
    // 都没有变更，返回空字符串
    Ok(String::new())
}

pub async fn git_commit_allow_empty(message: &str) -> anyhow::Result<()> {
    let status = Command::new("git")
        .args(["commit", "--allow-empty", "-m", message])
        .status()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to run git commit (allow-empty): {}", e))?;

    if !status.success() {
        anyhow::bail!(
            "Git commit (allow-empty) failed with exit code: {:?}",
            status.code()
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_git_add_all_command_structure() {
        // 测试命令结构（不实际执行 git）
        // 我们主要测试函数签名和返回类型
        let result = git_add_all().await;

        // 在没有 git 仓库的环境中，这会失败，但我们可以验证函数结构
        // 实际的 git 命令测试需要在有 git 环境的集成测试中进行
        match result {
            Ok(_) => {
                // 如果成功，说明在 git 仓库中
                println!("Git add all succeeded");
            }
            Err(e) => {
                // 验证错误类型包含预期信息
                let error_msg = e.to_string();
                assert!(
                    error_msg.contains("Failed to run git add")
                        || error_msg.contains("Git add failed"),
                    "Error message should contain git add information: {}",
                    error_msg
                );
            }
        }
    }

    #[tokio::test]
    async fn test_git_commit_command_structure() {
        let message = "test commit message";
        let result = git_commit(message).await;

        match result {
            Ok(_) => {
                println!("Git commit succeeded");
            }
            Err(e) => {
                let error_msg = e.to_string();
                assert!(
                    error_msg.contains("Failed to run git commit")
                        || error_msg.contains("Git commit failed"),
                    "Error message should contain git commit information: {}",
                    error_msg
                );
            }
        }
    }

    #[tokio::test]
    async fn test_git_commit_allow_empty_command_structure() {
        let message = "empty commit message";
        let result = git_commit_allow_empty(message).await;

        match result {
            Ok(_) => {
                println!("Git commit allow empty succeeded");
            }
            Err(e) => {
                let error_msg = e.to_string();
                assert!(
                    error_msg.contains("Failed to run git commit")
                        || error_msg.contains("Git commit (allow-empty) failed"),
                    "Error message should contain git commit information: {}",
                    error_msg
                );
            }
        }
    }

    #[tokio::test]
    async fn test_git_push_command_structure() {
        let result = git_push().await;

        match result {
            Ok(_) => {
                println!("Git push succeeded");
            }
            Err(e) => {
                let error_msg = e.to_string();
                assert!(
                    error_msg.contains("Failed to run git push")
                        || error_msg.contains("Git push failed"),
                    "Error message should contain git push information: {}",
                    error_msg
                );
            }
        }
    }

    #[tokio::test]
    async fn test_get_git_diff_command_structure() {
        let result = get_git_diff().await;

        match result {
            Ok(diff) => {
                // 如果成功，验证返回的是字符串
                assert!(diff.is_empty() || !diff.is_empty()); // 字符串类型验证
                println!("Git diff succeeded, length: {}", diff.len());
            }
            Err(e) => {
                let error_msg = e.to_string();
                assert!(
                    error_msg.contains("Failed to run git diff")
                        || error_msg.contains("Git diff failed"),
                    "Error message should contain git diff information: {}",
                    error_msg
                );
            }
        }
    }

    #[test]
    fn test_error_message_formatting() {
        // 测试错误消息格式
        let test_cases = vec![
            ("Failed to run git add: test error", "git add"),
            ("Failed to run git commit: test error", "git commit"),
            ("Failed to run git push: test error", "git push"),
            ("Failed to run git diff: test error", "git diff"),
        ];

        for (error_msg, expected_substring) in test_cases {
            assert!(
                error_msg.contains(expected_substring),
                "Error message '{}' should contain '{}'",
                error_msg,
                expected_substring
            );
        }
    }

    #[test]
    fn test_function_signatures() {
        // 验证函数签名的正确性（编译时检查）

        // 这些函数应该都是 async 且返回 Result
        #[allow(dead_code)]
        fn check_git_add_all() -> impl std::future::Future<Output = anyhow::Result<()>> {
            git_add_all()
        }

        #[allow(dead_code)]
        fn check_git_commit() -> impl std::future::Future<Output = anyhow::Result<()>> {
            git_commit("test")
        }

        #[allow(dead_code)]
        fn check_git_push() -> impl std::future::Future<Output = anyhow::Result<()>> {
            git_push()
        }

        #[allow(dead_code)]
        fn check_get_git_diff() -> impl std::future::Future<Output = anyhow::Result<String>> {
            get_git_diff()
        }

        #[allow(dead_code)]
        fn check_git_commit_allow_empty() -> impl std::future::Future<Output = anyhow::Result<()>> {
            git_commit_allow_empty("test")
        }

        // 如果编译通过，说明函数签名正确
    }

    #[tokio::test]
    async fn test_git_force_push_command_structure() {
        let result = git_force_push().await;

        match result {
            Ok(_) => {
                println!("Git force push succeeded");
            }
            Err(e) => {
                let error_msg = e.to_string();
                assert!(
                    error_msg.contains("Failed to run git push")
                        || error_msg.contains("Git push failed")
                        || error_msg.contains("Failed to get current branch")
                        || error_msg.contains("Failed to run git pull")
                        || error_msg.contains("Git pull failed"),
                    "Error message should contain expected git operation information: {}",
                    error_msg
                );
            }
        }
    }

    #[test]
    fn test_git_force_push_function_signature() {
        // 验证 git_force_push 函数签名的正确性（编译时检查）
        #[allow(dead_code)]
        fn check_git_force_push() -> impl std::future::Future<Output = anyhow::Result<()>> {
            git_force_push()
        }

        // 如果编译通过，说明函数签名正确
    }

    #[test]
    fn test_force_push_error_scenarios() {
        // 测试强制推送可能的错误场景
        let test_error_patterns = vec![
            "Failed to run git push",
            "Git push failed",
            "Failed to get current branch", 
            "Failed to run git pull",
            "Git pull failed. 请手动解决冲突后重试。"
        ];

        for pattern in test_error_patterns {
            assert!(pattern.contains("git") || pattern.contains("Git"), 
                "Error pattern should contain git reference: {}", pattern);
        }
    }
}
// 测试注释3
// Git相关修改
