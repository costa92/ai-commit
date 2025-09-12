use super::list::list_worktrees;
use std::path::PathBuf;

/// 切换到指定的worktree
pub async fn switch_to_worktree(path_or_name: &str) -> anyhow::Result<PathBuf> {
    let worktrees = list_worktrees().await?;

    let target_worktree = worktrees
        .iter()
        .find(|w| {
            w.path.to_string_lossy().contains(path_or_name)
                || w.branch.contains(path_or_name)
                || w.path
                    .file_name()
                    .is_some_and(|f| f.to_string_lossy().contains(path_or_name))
        })
        .ok_or_else(|| anyhow::anyhow!("找不到指定的 worktree: {}", path_or_name))?;

    std::env::set_current_dir(&target_worktree.path)
        .map_err(|e| anyhow::anyhow!("切换到 worktree 目录失败: {}", e))?;

    Ok(target_worktree.path.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // 需要Git仓库环境和实际的worktree
    async fn test_switch_to_worktree() {
        // 这个测试需要实际的Git环境，应该在集成测试中运行
        let _result = switch_to_worktree("main").await;
        // 在有worktree的环境中应该能找到并切换
    }

    #[test]
    fn test_worktree_matching_logic() {
        // 可以测试匹配逻辑，但需要模拟数据
        // 实际的测试应该在集成测试中进行
    }
}
