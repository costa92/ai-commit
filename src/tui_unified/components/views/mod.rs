pub mod git_log;
pub mod branches;
pub mod tags;
pub mod remotes;
pub mod stash;
pub mod query_history;

pub use git_log::GitLogView;
pub use branches::BranchesView;
pub use tags::TagsView;
pub use remotes::RemotesView;
pub use stash::StashView;
pub use query_history::QueryHistoryView;