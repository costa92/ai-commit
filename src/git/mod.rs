pub mod commit;
pub mod core;
pub mod diff_viewer;
pub mod edit;
pub mod flow;
pub mod history;
pub mod hooks;
pub mod query;
pub mod tag;
pub mod watcher;
pub mod worktree;

// commit: 异步 git 操作函数
pub use commit::{
    get_all_changes_diff, get_git_diff, git_add_all, git_commit, git_commit_allow_empty,
    git_force_push, git_push, git_status_and_diff,
};

// core: 基础 Git 操作
pub use core::GitCore;

// diff_viewer: Diff 显示
pub use diff_viewer::{DiffStats, DiffViewer, FileStats, FileStatus};

// edit: 提交编辑
pub use edit::{GitEdit, GitEditResult, RebaseStatus};

// flow: Git Flow 工作流
pub use flow::{BranchType, GitFlow};

// history: 历史查看
pub use history::GitHistory;

// query: 查询
pub use query::{GitQuery, QueryFilter};

// watcher: 仓库监听
pub use watcher::{ChangeEvent, ChangeType, GitWatcher, RepoStatus};

// tag: 常用 tag 操作（完整 API 通过 git::tag:: 访问）
pub use tag::{create_tag_with_note, get_latest_tag, get_next_tag_name, push_tag};

// worktree: 工作树管理
pub use worktree::{
    clear_other_worktrees, create_worktree, create_worktree_with_new_branch, get_current_worktree,
    list_worktrees, list_worktrees_raw, list_worktrees_with_options, prune_worktrees,
    remove_worktree, switch_to_worktree, WorktreeInfo, WorktreeListOptions,
};
