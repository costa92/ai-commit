use super::list::list_worktrees;
use tokio::process::Command;

/// 删除指定的worktree
pub async fn remove_worktree(path_or_name: &str) -> anyhow::Result<()> {
    let worktrees = list_worktrees().await?;

    let target_path = if let Some(worktree) = worktrees.iter().find(|w| {
        w.path.to_string_lossy().contains(path_or_name) || w.branch.contains(path_or_name)
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
        anyhow::bail!(
            "Git worktree remove failed with exit code: {:?}",
            status.code()
        );
    }

    Ok(())
}

/// 清理worktree引用
pub async fn prune_worktrees() -> anyhow::Result<()> {
    let status = Command::new("git")
        .args(["worktree", "prune"])
        .status()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to run git worktree prune: {}", e))?;

    if !status.success() {
        anyhow::bail!(
            "Git worktree prune failed with exit code: {:?}",
            status.code()
        );
    }

    Ok(())
}

/// 清空除当前外的所有其他worktrees
pub async fn clear_other_worktrees() -> anyhow::Result<usize> {
    let current_dir = std::env::current_dir()?;
    let worktrees = list_worktrees().await?;

    let mut removed_count = 0;

    for worktree in worktrees {
        // 跳过当前工作目录
        if worktree.path == current_dir {
            continue;
        }

        // 跳过bare worktree（通常是主仓库）
        if worktree.is_bare {
            continue;
        }

        match remove_worktree(&worktree.path.to_string_lossy()).await {
            Ok(_) => removed_count += 1,
            Err(e) => {
                eprintln!(
                    "Failed to remove worktree {}: {}",
                    worktree.path.display(),
                    e
                );
            }
        }
    }

    // 最后清理无效的引用
    prune_worktrees().await?;

    Ok(removed_count)
}

#[cfg(test)]
mod tests {
    use super::*;

    // 注意：这些测试需要实际的Git仓库环境，所以可能需要集成测试

    #[tokio::test]
    #[ignore] // 需要Git仓库环境
    async fn test_prune_worktrees() {
        let _result = prune_worktrees().await;
        // 在有Git仓库的环境中应该成功
        // assert!(result.is_ok());
    }

    #[test]
    fn test_remove_logic() {
        // 这里可以测试路径匹配逻辑等非Git依赖的部分
        // 实际的Git命令测试应该在集成测试中进行
    }

    #[test]
    fn test_worktree_path_matching() {
        // 测试路径匹配逻辑（不依赖Git命令）
        use super::super::info::WorktreeInfo;
        use std::path::PathBuf;

        let worktrees = vec![
            WorktreeInfo::new(
                PathBuf::from("/repo/main"),
                "main".to_string(),
                "abc123".to_string(),
                false,
                false,
            ),
            WorktreeInfo::new(
                PathBuf::from("/repo/worktree-feature-test"),
                "feature/test".to_string(),
                "def456".to_string(),
                false,
                false,
            ),
        ];

        // 测试通过路径匹配
        let match_by_path = worktrees
            .iter()
            .find(|w| w.path.to_string_lossy().contains("feature-test"));
        assert!(match_by_path.is_some());
        assert_eq!(match_by_path.unwrap().branch, "feature/test");

        // 测试通过分支名匹配
        let match_by_branch = worktrees.iter().find(|w| w.branch.contains("main"));
        assert!(match_by_branch.is_some());
        assert_eq!(match_by_branch.unwrap().branch, "main");
    }

    #[test]
    fn test_worktree_filtering_for_clear() {
        use super::super::info::WorktreeInfo;
        use std::path::PathBuf;

        let current_dir = PathBuf::from("/repo/main");
        let worktrees = vec![
            WorktreeInfo::new(
                current_dir.clone(),
                "main".to_string(),
                "abc123".to_string(),
                false,
                false,
            ),
            WorktreeInfo::new(
                PathBuf::from("/repo/bare"),
                "bare".to_string(),
                "def456".to_string(),
                true, // bare worktree
                false,
            ),
            WorktreeInfo::new(
                PathBuf::from("/repo/feature"),
                "feature/test".to_string(),
                "ghi789".to_string(),
                false,
                false,
            ),
        ];

        // 过滤掉当前目录和bare worktree
        let to_remove: Vec<_> = worktrees
            .iter()
            .filter(|w| w.path != current_dir && !w.is_bare)
            .collect();

        assert_eq!(to_remove.len(), 1);
        assert_eq!(to_remove[0].branch, "feature/test");
    }

    #[test]
    fn test_worktree_name_patterns() {
        // 测试不同的worktree名称模式匹配
        let test_cases = vec![
            ("feature-test", "feature/test", true), // 路径中包含
            ("feature", "feature/test", true),      // 部分匹配
            ("test", "feature/test", true),         // 分支名部分匹配
            ("main", "feature/test", false),        // 不匹配
            ("nonexistent", "feature/test", false), // 不存在
        ];

        for (search_term, branch_name, should_match) in test_cases {
            let path_contains = format!("/repo/worktree-{}", search_term).contains(search_term);
            let branch_contains = branch_name.contains(search_term);
            let matches = path_contains || branch_contains;

            assert_eq!(
                matches, should_match,
                "Failed for search_term: {}, branch: {}",
                search_term, branch_name
            );
        }
    }

    #[test]
    fn test_clear_worktree_count_calculation() {
        use super::super::info::WorktreeInfo;
        use std::path::PathBuf;

        let current_dir = PathBuf::from("/repo/current");
        let test_scenarios = vec![
            // 场景1: 只有当前worktree
            (
                vec![WorktreeInfo::new(
                    current_dir.clone(),
                    "main".to_string(),
                    "abc".to_string(),
                    false,
                    false,
                )],
                0,
            ),
            // 场景2: 当前 + 1个其他
            (
                vec![
                    WorktreeInfo::new(
                        current_dir.clone(),
                        "main".to_string(),
                        "abc".to_string(),
                        false,
                        false,
                    ),
                    WorktreeInfo::new(
                        PathBuf::from("/repo/other"),
                        "feature".to_string(),
                        "def".to_string(),
                        false,
                        false,
                    ),
                ],
                1,
            ),
            // 场景3: 当前 + 1个bare + 2个其他
            (
                vec![
                    WorktreeInfo::new(
                        current_dir.clone(),
                        "main".to_string(),
                        "abc".to_string(),
                        false,
                        false,
                    ),
                    WorktreeInfo::new(
                        PathBuf::from("/repo/bare"),
                        "bare".to_string(),
                        "def".to_string(),
                        true,
                        false,
                    ),
                    WorktreeInfo::new(
                        PathBuf::from("/repo/other1"),
                        "f1".to_string(),
                        "ghi".to_string(),
                        false,
                        false,
                    ),
                    WorktreeInfo::new(
                        PathBuf::from("/repo/other2"),
                        "f2".to_string(),
                        "jkl".to_string(),
                        false,
                        false,
                    ),
                ],
                2,
            ),
        ];

        for (worktrees, expected_count) in test_scenarios {
            let removable_count = worktrees
                .iter()
                .filter(|w| w.path != current_dir && !w.is_bare)
                .count();

            assert_eq!(removable_count, expected_count);
        }
    }

    #[test]
    fn test_path_matching_edge_cases() {
        use super::super::info::WorktreeInfo;
        use std::path::PathBuf;

        let worktrees = vec![WorktreeInfo::new(
            PathBuf::from("/repo/worktree-feature-ui-test"),
            "feature/ui/test".to_string(),
            "abc123".to_string(),
            false,
            false,
        )];

        // 测试各种可能的匹配方式
        let search_terms = vec![
            "feature-ui-test", // 完整路径匹配
            "ui-test",         // 部分路径匹配
            "feature/ui",      // 分支名前缀匹配
            "ui/test",         // 分支名后缀匹配
            "test",            // 通用匹配
        ];

        for term in search_terms {
            let found = worktrees
                .iter()
                .find(|w| w.path.to_string_lossy().contains(term) || w.branch.contains(term));
            assert!(found.is_some(), "Should find worktree with term: {}", term);
        }
    }
}
