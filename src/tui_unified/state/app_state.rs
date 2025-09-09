use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use crate::tui_unified::{
    config::AppConfig,
    focus::FocusPanel,
    Result
};

#[derive(Debug, Clone)]
pub struct AppState {
    // UI状态
    pub layout: LayoutState,
    pub focus: FocusState,
    pub current_view: ViewType,
    pub modal: Option<ModalState>,
    
    // Git数据状态  
    pub repo_state: GitRepoState,
    pub selected_items: SelectionState,
    pub search_state: SearchState,
    
    // 配置状态
    pub config: AppConfig,
    
    // 运行时状态
    pub loading_tasks: HashMap<String, LoadingTask>,
    pub notifications: Vec<Notification>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewType {
    GitLog,
    Branches,
    Tags, 
    Remotes,
    Stash,
    QueryHistory,
}

#[derive(Debug, Clone)]
pub struct LayoutState {
    pub sidebar_width: u16,
    pub content_width: u16,
    pub detail_width: u16,
}

#[derive(Debug, Clone)]
pub struct FocusState {
    pub current_panel: FocusPanel,
    pub panel_history: Vec<FocusPanel>,
    pub can_navigate: bool,
}

#[derive(Debug, Clone)]
pub struct GitRepoState {
    pub current_branch: String,
    pub repo_path: PathBuf,
    // TODO: 添加更多Git状态
}

#[derive(Debug, Clone)]
pub struct SelectionState {
    // TODO: 实现选择状态
}

#[derive(Debug, Clone)]
pub struct SearchState {
    pub query: String,
    pub is_active: bool,
    // TODO: 添加更多搜索状态
}

#[derive(Debug, Clone)]
pub struct ModalState {
    // TODO: 实现模态框状态
}

#[derive(Debug, Clone)]
pub struct LoadingTask {
    pub name: String,
    pub progress: Option<f64>,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct Notification {
    pub message: String,
    pub level: NotificationLevel,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NotificationLevel {
    Info,
    Warning,
    Error,
    Success,
}

impl AppState {
    pub async fn new(config: &AppConfig) -> Result<Self> {
        let repo_path = std::env::current_dir()?;
        
        Ok(Self {
            layout: LayoutState {
                sidebar_width: 20,
                content_width: 50,
                detail_width: 30,
            },
            focus: FocusState {
                current_panel: FocusPanel::Sidebar,
                panel_history: Vec::new(),
                can_navigate: true,
            },
            current_view: ViewType::GitLog,
            modal: None,
            repo_state: GitRepoState {
                current_branch: "main".to_string(), // TODO: 从git获取实际分支
                repo_path,
            },
            selected_items: SelectionState {},
            search_state: SearchState {
                query: String::new(),
                is_active: false,
            },
            config: config.clone(),
            loading_tasks: HashMap::new(),
            notifications: Vec::new(),
        })
    }
    
    pub fn set_current_view(&mut self, view: ViewType) {
        self.current_view = view;
    }
    
    pub fn add_notification(&mut self, message: String, level: NotificationLevel) {
        self.notifications.push(Notification { message, level });
        
        // 限制通知数量
        if self.notifications.len() > 10 {
            self.notifications.remove(0);
        }
    }
    
    pub fn is_loading(&self) -> bool {
        !self.loading_tasks.is_empty()
    }
}