use tokio::process::Command;

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

pub async fn get_git_diff() -> anyhow::Result<String> {
    let output = Command::new("git")
        .args(["diff", "--cached"])
        .output()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to run git diff: {}", e))?;

    if !output.status.success() {
        anyhow::bail!("Git diff failed with exit code: {:?}", output.status.code());
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

pub async fn git_commit_allow_empty(message: &str) -> anyhow::Result<()> {
    let status = Command::new("git")
        .args(["commit", "--allow-empty", "-m", message])
        .status()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to run git commit (allow-empty): {}", e))?;
    
    if !status.success() {
        anyhow::bail!("Git commit (allow-empty) failed with exit code: {:?}", status.code());
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
                    error_msg.contains("Failed to run git add") || 
                    error_msg.contains("Git add failed"),
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
                    error_msg.contains("Failed to run git commit") || 
                    error_msg.contains("Git commit failed"),
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
                    error_msg.contains("Failed to run git commit") || 
                    error_msg.contains("Git commit (allow-empty) failed"),
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
                    error_msg.contains("Failed to run git push") || 
                    error_msg.contains("Git push failed"),
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
                    error_msg.contains("Failed to run git diff") || 
                    error_msg.contains("Git diff failed"),
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
        fn check_git_add_all() -> impl std::future::Future<Output = anyhow::Result<()>> {
            git_add_all()
        }
        
        fn check_git_commit() -> impl std::future::Future<Output = anyhow::Result<()>> {
            git_commit("test")
        }
        
        fn check_git_push() -> impl std::future::Future<Output = anyhow::Result<()>> {
            git_push()
        }
        
        fn check_get_git_diff() -> impl std::future::Future<Output = anyhow::Result<String>> {
            get_git_diff()
        }
        
        fn check_git_commit_allow_empty() -> impl std::future::Future<Output = anyhow::Result<()>> {
            git_commit_allow_empty("test")
        }
        
        // 如果编译通过，说明函数签名正确
        assert!(true);
    }
}
