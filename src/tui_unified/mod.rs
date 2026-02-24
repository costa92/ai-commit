//! TUI 统一界面模块
//!
//! 这个模块实现了整合后的 TUI 界面，结合了 `--query-tui-pro` 和 `--tui` 的最佳功能。

mod ai_agent_handler;
mod ai_commit_handler;
pub mod algorithms;
pub mod app;
pub mod async_manager;
pub mod cache;
pub mod components;
pub mod config;
mod diff_parsing;
mod diff_rendering;
pub mod events;
pub mod focus;
pub mod git;
mod git_operations;
mod input_handler;
pub mod layout;
mod modal_rendering;
mod rendering;
pub mod state;
pub mod utils;

pub use app::TuiUnifiedApp;

// 重要的公共类型和枚举
pub use events::{Event, EventResult};
pub use focus::FocusPanel;
pub use layout::{LayoutMode, PanelType};
pub use state::{AppState, ViewType};

// 错误类型
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
