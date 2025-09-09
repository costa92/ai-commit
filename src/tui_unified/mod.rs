//! TUI 统一界面模块
//! 
//! 这个模块实现了整合后的 TUI 界面，结合了 `--query-tui-pro` 和 `--tui` 的最佳功能。

pub mod app;
pub mod layout;
pub mod focus;
pub mod state;
pub mod components;
pub mod events;
pub mod git;
pub mod cache;
pub mod algorithms;
pub mod async_manager;
pub mod config;
pub mod utils;

pub use app::TuiUnifiedApp;

// 重要的公共类型和枚举
pub use state::{AppState, ViewType};
pub use layout::{LayoutMode, PanelType};
pub use focus::FocusPanel;
pub use events::{Event, EventResult};

// 错误类型
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;