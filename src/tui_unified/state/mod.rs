pub mod app_state;
pub mod git_state;
pub mod ui_state;
pub mod simple_persistence;
// pub mod persistence; // 暂时注释掉复杂版本
// pub mod validation;  // 暂时注释掉验证

#[cfg(test)]
mod simple_tests;

pub use app_state::{
    AppState, ViewType, SelectionState, SearchState, SelectionMode,
    ModalState, ModalType, ModalAction, LoadingTask, Notification, NotificationLevel
};
pub use git_state::{
    GitRepoState, RepoStatus, Branch, Commit, Tag, Remote, Stash,
    FileStatus, ChangeType, RepoSummary
};
pub use ui_state::{
    LayoutState, FocusState, ModalState as UIModalState,
    LayoutMode, PanelRatios, MinPanelSizes, FocusRing,
    ModalSize, ModalPosition, ModalButton
};
pub use simple_persistence::{
    SimpleStatePersistence, SimplePersistentState
};
// pub use persistence::{
//     StatePersistence, PersistentState, StateInfo,
//     LayoutPreferences, WindowState, UserPreferences, SessionData
// };
// pub use validation::{
//     StateValidator, StateRecovery, ValidationResult
// };