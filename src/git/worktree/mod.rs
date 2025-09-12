//! Git Worktree 管理模块
//!
//! 提供完整的 Git worktree 管理功能，包括创建、列出、切换、删除等操作。
//! 按功能单一原则，每个操作类型对应一个独立的文件。

pub mod create;
pub mod info;
pub mod list;
pub mod remove;
pub mod switch;

// 重新导出主要的类型和函数
pub use create::{create_worktree, create_worktree_with_new_branch};
pub use info::{get_current_worktree, WorktreeInfo, WorktreeListOptions};
pub use list::{list_worktrees, list_worktrees_raw, list_worktrees_with_options};
pub use remove::{clear_other_worktrees, prune_worktrees, remove_worktree};
pub use switch::switch_to_worktree;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_exports() {
        // 验证所有公共接口都能正确导出
        // 这个测试主要是编译时检查

        // WorktreeInfo should be accessible
        let _info = WorktreeInfo::new(
            std::path::PathBuf::from("/test"),
            "main".to_string(),
            "abc123".to_string(),
            false,
            false,
        );

        // WorktreeListOptions should be accessible
        let _options = WorktreeListOptions::default();
    }
}
