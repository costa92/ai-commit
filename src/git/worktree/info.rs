use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Worktree信息结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorktreeInfo {
    pub path: PathBuf,
    pub branch: String,
    pub commit: String,
    pub is_bare: bool,
    pub is_detached: bool,
}

impl WorktreeInfo {
    /// 创建新的WorktreeInfo实例
    pub fn new(
        path: PathBuf,
        branch: String,
        commit: String,
        is_bare: bool,
        is_detached: bool,
    ) -> Self {
        Self {
            path,
            branch,
            commit,
            is_bare,
            is_detached,
        }
    }
}

/// Worktree列表选项
#[derive(Debug, Clone, Default)]
pub struct WorktreeListOptions {
    pub verbose: bool,
    pub porcelain: bool,
    pub z: bool,
    pub expire: Option<String>,
}

/// 获取当前worktree信息
pub async fn get_current_worktree() -> anyhow::Result<Option<WorktreeInfo>> {
    let worktrees = super::list::list_worktrees().await?;
    let current_dir = std::env::current_dir()?;

    for worktree in worktrees {
        if worktree.path == current_dir {
            return Ok(Some(worktree));
        }
    }

    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_worktree_info_new() {
        let info = WorktreeInfo::new(
            PathBuf::from("/path/to/worktree"),
            "main".to_string(),
            "abc123".to_string(),
            false,
            false,
        );

        assert_eq!(info.path, PathBuf::from("/path/to/worktree"));
        assert_eq!(info.branch, "main");
        assert_eq!(info.commit, "abc123");
        assert!(!info.is_bare);
        assert!(!info.is_detached);
    }

    #[test]
    fn test_worktree_list_options_default() {
        let options = WorktreeListOptions::default();

        assert!(!options.verbose);
        assert!(!options.porcelain);
        assert!(!options.z);
        assert!(options.expire.is_none());
    }
}