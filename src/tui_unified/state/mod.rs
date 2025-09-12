pub mod app_state;
pub mod git_state;
pub mod simple_persistence;
pub mod ui_state;
// pub mod persistence; // 暂时注释掉复杂版本
// pub mod validation;  // 暂时注释掉验证

#[cfg(test)]
mod simple_tests;

pub use app_state::{
    AppState, LoadingTask, ModalAction, ModalState, ModalType, Notification, NotificationLevel,
    SearchState, SelectionMode, SelectionState, ViewType,
};
pub use git_state::{
    Branch, ChangeType, Commit, FileStatus, GitRepoState, Remote, RepoStatus, RepoSummary, Stash,
    Tag,
};
pub use simple_persistence::{SimplePersistentState, SimpleStatePersistence};
pub use ui_state::{
    FocusRing, FocusState, LayoutMode, LayoutState, MinPanelSizes, ModalButton, ModalPosition,
    ModalSize, ModalState as UIModalState, PanelRatios,
};
// pub use persistence::{
//     StatePersistence, PersistentState, StateInfo,
//     LayoutPreferences, WindowState, UserPreferences, SessionData
// };
// pub use validation::{
//     StateValidator, StateRecovery, ValidationResult
// };
