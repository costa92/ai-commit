use std::path::PathBuf;
use tokio::process::Command;

/// 创建worktree（使用已存在的分支）
pub async fn create_worktree(branch: &str, custom_path: Option<&str>) -> anyhow::Result<PathBuf> {
    let path = generate_worktree_path(branch, custom_path)?;
    let path_str = path.to_string_lossy();

    let status = Command::new("git")
        .args(["worktree", "add", &path_str, branch])
        .status()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to run git worktree add: {}", e))?;

    if !status.success() {
        anyhow::bail!(
            "Git worktree add failed with exit code: {:?}",
            status.code()
        );
    }

    Ok(path)
}

/// 创建worktree（同时创建新分支）
pub async fn create_worktree_with_new_branch(
    branch: &str,
    custom_path: Option<&str>,
) -> anyhow::Result<PathBuf> {
    let path = generate_worktree_path(branch, custom_path)?;
    let path_str = path.to_string_lossy();

    let status = Command::new("git")
        .args(["worktree", "add", "-b", branch, &path_str])
        .status()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to run git worktree add with new branch: {}", e))?;

    if !status.success() {
        anyhow::bail!(
            "Git worktree add with new branch failed with exit code: {:?}",
            status.code()
        );
    }

    Ok(path)
}

/// 生成worktree路径
fn generate_worktree_path(branch: &str, custom_path: Option<&str>) -> anyhow::Result<PathBuf> {
    if let Some(custom) = custom_path {
        Ok(PathBuf::from(custom))
    } else {
        let current_dir = std::env::current_dir()?;
        let parent_dir = current_dir
            .parent()
            .ok_or_else(|| anyhow::anyhow!("无法确定父目录"))?;

        let branch_name = branch.replace('/', "-");
        Ok(parent_dir.join(format!("worktree-{}", branch_name)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_worktree_path_custom() {
        let result = generate_worktree_path("feature/test", Some("/custom/path"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), PathBuf::from("/custom/path"));
    }

    #[test]
    fn test_generate_worktree_path_default() {
        let result = generate_worktree_path("feature/test", None);
        assert!(result.is_ok());
        
        let path = result.unwrap();
        assert!(path.to_string_lossy().contains("worktree-feature-test"));
    }

    #[test]
    fn test_branch_name_sanitization() {
        let result = generate_worktree_path("feature/ui/new-design", None);
        assert!(result.is_ok());
        
        let path = result.unwrap();
        assert!(path.to_string_lossy().contains("worktree-feature-ui-new-design"));
    }

    #[test]
    fn test_generate_worktree_path_edge_cases() {
        // 测试空分支名
        let result = generate_worktree_path("", None);
        assert!(result.is_ok());
        
        // 测试特殊字符
        let result = generate_worktree_path("feature/test@123", None);
        assert!(result.is_ok());
        assert!(result.unwrap().to_string_lossy().contains("worktree-feature-test@123"));
        
        // 测试长分支名
        let long_branch = "feature/very-long-branch-name-that-might-cause-issues";
        let result = generate_worktree_path(long_branch, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_generate_worktree_path_custom_absolute() {
        let result = generate_worktree_path("main", Some("/tmp/test-worktree"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), PathBuf::from("/tmp/test-worktree"));
    }

    #[test]
    fn test_generate_worktree_path_custom_relative() {
        let result = generate_worktree_path("main", Some("../relative-path"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), PathBuf::from("../relative-path"));
    }

    #[test]
    fn test_branch_name_with_special_characters() {
        let test_cases = vec![
            ("feat/UI-123", "worktree-feat-UI-123"),
            ("bugfix/issue_456", "worktree-bugfix-issue_456"),
            ("release/v1.2.3", "worktree-release-v1.2.3"),
            ("hotfix/critical-fix", "worktree-hotfix-critical-fix"),
        ];
        
        for (branch, expected_contain) in test_cases {
            let result = generate_worktree_path(branch, None);
            assert!(result.is_ok());
            assert!(result.unwrap().to_string_lossy().contains(expected_contain));
        }
    }

    #[test]
    fn test_path_generation_consistency() {
        let branch = "feature/test";
        let result1 = generate_worktree_path(branch, None);
        let result2 = generate_worktree_path(branch, None);
        
        assert!(result1.is_ok());
        assert!(result2.is_ok());
        assert_eq!(result1.unwrap(), result2.unwrap());
    }

    #[test]
    fn test_custom_path_validation() {
        // 测试不同类型的自定义路径
        let test_paths = vec![
            "/tmp/worktree-test",
            "~/worktree-test", 
            "./local-worktree",
            "../parent-worktree",
            "relative/path/worktree",
        ];
        
        for path in test_paths {
            let result = generate_worktree_path("test", Some(path));
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), PathBuf::from(path));
        }
    }
}