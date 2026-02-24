pub mod branches;
pub mod git_log;
pub mod query_history;
pub mod remotes;
pub mod shared;
pub mod staging;
pub mod stash;
pub mod tags;

pub use branches::BranchesView;
pub use git_log::GitLogView;
pub use query_history::QueryHistoryView;
pub use remotes::RemotesView;
pub use staging::StagingView;
pub use stash::StashView;
pub use tags::TagsView;
