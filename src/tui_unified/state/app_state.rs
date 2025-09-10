use std::collections::HashMap;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use crate::tui_unified::{
    config::AppConfig,
    focus::FocusPanel,
    Result
};
use super::git_state::GitRepoState;
use super::ui_state::{LayoutState, FocusState};

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




#[derive(Debug, Clone, Default)]
pub struct SelectionState {
    pub selected_commit: Option<String>,
    pub selected_branch: Option<String>,
    pub selected_tag: Option<String>,
    pub selected_remote: Option<String>,
    pub selected_stash: Option<String>,
    pub multi_selection: Vec<String>,
    pub selection_mode: SelectionMode,
    pub pending_diff_commit: Option<String>, // 待显示diff的提交哈希
    pub pending_branch_switch: Option<String>, // 待切换的分支名
    pub direct_branch_switch: Option<String>, // 直接切换的分支名（不通过模态框）
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum SelectionMode {
    #[default]
    Single,
    Multiple,
}

#[derive(Debug, Clone, Default)]
pub struct SearchState {
    pub query: String,
    pub is_active: bool,
    pub history: Vec<String>,
    pub filters: SearchFilters,
    pub results_count: usize,
    pub current_match: usize,
}

#[derive(Debug, Clone, Default)]
pub struct SearchFilters {
    pub author: Option<String>,
    pub date_from: Option<DateTime<Utc>>,
    pub date_to: Option<DateTime<Utc>>,
    pub file_path: Option<String>,
    pub message_regex: bool,
}

#[derive(Debug, Clone)]
pub struct ModalState {
    pub modal_type: ModalType,
    pub title: String,
    pub content: String,
    pub buttons: Vec<ModalButton>,
    pub default_button: usize,
    pub can_cancel: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ModalType {
    Info,
    Warning,
    Error,
    Confirm,
    Input,
    Progress,
    DiffViewer,
    AICommit,
    GitPull,
    BranchSwitch,
}

#[derive(Debug, Clone)]
pub struct ModalButton {
    pub label: String,
    pub action: ModalAction,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ModalAction {
    Ok,
    Cancel,
    Yes,
    No,
    Retry,
    Custom(String),
}

#[derive(Debug, Clone)]
pub struct LoadingTask {
    pub id: Uuid,
    pub name: String,
    pub progress: Option<f64>,
    pub message: String,
    pub started_at: DateTime<Utc>,
    pub can_cancel: bool,
}

#[derive(Debug, Clone)]
pub struct Notification {
    pub id: Uuid,
    pub message: String,
    pub level: NotificationLevel,
    pub timestamp: DateTime<Utc>,
    pub auto_dismiss: Option<std::time::Duration>,
    pub dismissed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
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
            layout: LayoutState::default(),
            focus: FocusState::default(),
            current_view: ViewType::GitLog,
            modal: None,
            repo_state: GitRepoState::new(repo_path),
            selected_items: SelectionState::default(),
            search_state: SearchState::default(),
            config: config.clone(),
            loading_tasks: HashMap::new(),
            notifications: Vec::new(),
        })
    }
    
    // 视图管理
    pub fn set_current_view(&mut self, view: ViewType) {
        self.current_view = view;
    }
    
    pub fn get_current_view(&self) -> ViewType {
        self.current_view
    }
    
    // 选择状态管理
    pub fn select_commit(&mut self, commit_hash: String) {
        self.selected_items.selected_commit = Some(commit_hash);
    }
    
    pub fn select_branch(&mut self, branch_name: String) {
        self.selected_items.selected_branch = Some(branch_name);
    }
    
    pub fn select_tag(&mut self, tag_name: String) {
        self.selected_items.selected_tag = Some(tag_name);
    }
    
    pub fn get_current_selection(&self) -> Option<String> {
        match self.current_view {
            ViewType::GitLog => self.selected_items.selected_commit.clone(),
            ViewType::Branches => self.selected_items.selected_branch.clone(),
            ViewType::Tags => self.selected_items.selected_tag.clone(),
            ViewType::Remotes => self.selected_items.selected_remote.clone(),
            ViewType::Stash => self.selected_items.selected_stash.clone(),
            ViewType::QueryHistory => None,
        }
    }
    
    pub fn clear_selection(&mut self) {
        self.selected_items = SelectionState::default();
    }
    
    pub fn request_diff(&mut self, commit_hash: String) {
        self.selected_items.pending_diff_commit = Some(commit_hash);
    }
    
    pub fn get_pending_diff_commit(&mut self) -> Option<String> {
        self.selected_items.pending_diff_commit.take()
    }
    
    pub fn request_git_pull(&mut self) {
        let modal = ModalState {
            modal_type: ModalType::GitPull,
            title: "Git Pull".to_string(),
            content: "Pull latest changes from remote repository?".to_string(),
            buttons: vec![
                ModalButton {
                    label: "Pull".to_string(),
                    action: ModalAction::Yes,
                },
                ModalButton {
                    label: "Cancel".to_string(),
                    action: ModalAction::Cancel,
                },
            ],
            default_button: 0,
            can_cancel: true,
        };
        self.show_modal(modal);
    }
    
    pub fn request_branch_switch(&mut self, branch_name: String) {
        let modal = ModalState {
            modal_type: ModalType::BranchSwitch,
            title: "Switch Branch".to_string(),
            content: format!("Switch to branch '{}'?\n\nThis will change your current working branch.", branch_name),
            buttons: vec![
                ModalButton {
                    label: "Switch".to_string(),
                    action: ModalAction::Yes,
                },
                ModalButton {
                    label: "Cancel".to_string(),
                    action: ModalAction::Cancel,
                },
            ],
            default_button: 0,
            can_cancel: true,
        };
        self.show_modal(modal);
        
        // 存储要切换的分支名（我们需要在 SelectionState 中添加字段）
        self.selected_items.pending_branch_switch = Some(branch_name);
    }
    
    pub fn get_pending_branch_switch(&mut self) -> Option<String> {
        self.selected_items.pending_branch_switch.take()
    }
    
    pub fn request_direct_branch_switch(&mut self, branch_name: String) {
        self.selected_items.direct_branch_switch = Some(branch_name);
    }
    
    pub fn get_direct_branch_switch(&mut self) -> Option<String> {
        self.selected_items.direct_branch_switch.take()
    }
    
    // 搜索状态管理
    pub fn set_search_query(&mut self, query: String) {
        self.search_state.query = query;
        self.search_state.is_active = !self.search_state.query.is_empty();
    }
    
    pub fn add_search_to_history(&mut self, query: String) {
        if !query.is_empty() && !self.search_state.history.contains(&query) {
            self.search_state.history.insert(0, query);
            if self.search_state.history.len() > 50 {
                self.search_state.history.truncate(50);
            }
        }
    }
    
    pub fn clear_search(&mut self) {
        self.search_state.query.clear();
        self.search_state.is_active = false;
        self.search_state.results_count = 0;
        self.search_state.current_match = 0;
    }
    
    // 加载任务管理
    pub fn add_loading_task(&mut self, name: String, message: String) -> Uuid {
        let task = LoadingTask {
            id: Uuid::new_v4(),
            name: name.clone(),
            progress: None,
            message,
            started_at: Utc::now(),
            can_cancel: false,
        };
        let id = task.id;
        self.loading_tasks.insert(name, task);
        id
    }
    
    pub fn update_loading_progress(&mut self, name: &str, progress: f64, message: String) {
        if let Some(task) = self.loading_tasks.get_mut(name) {
            task.progress = Some(progress.clamp(0.0, 1.0));
            task.message = message;
        }
    }
    
    pub fn remove_loading_task(&mut self, name: &str) {
        self.loading_tasks.remove(name);
    }
    
    pub fn is_loading(&self) -> bool {
        !self.loading_tasks.is_empty()
    }
    
    pub fn get_loading_tasks(&self) -> Vec<&LoadingTask> {
        self.loading_tasks.values().collect()
    }
    
    // 通知管理
    pub fn add_notification(&mut self, message: String, level: NotificationLevel) -> Uuid {
        let notification = Notification {
            id: Uuid::new_v4(),
            message,
            level,
            timestamp: Utc::now(),
            auto_dismiss: match level {
                NotificationLevel::Info => Some(std::time::Duration::from_secs(3)),
                NotificationLevel::Success => Some(std::time::Duration::from_secs(3)),
                NotificationLevel::Warning => Some(std::time::Duration::from_secs(5)),
                NotificationLevel::Error => None, // 不自动消失
            },
            dismissed: false,
        };
        let id = notification.id;
        self.notifications.push(notification);
        
        // 限制通知数量
        if self.notifications.len() > 10 {
            self.notifications.remove(0);
        }
        
        id
    }
    
    pub fn dismiss_notification(&mut self, id: Uuid) {
        if let Some(notification) = self.notifications.iter_mut().find(|n| n.id == id) {
            notification.dismissed = true;
        }
    }
    
    pub fn clean_dismissed_notifications(&mut self) {
        self.notifications.retain(|n| !n.dismissed);
    }
    
    pub fn get_active_notifications(&self) -> Vec<&Notification> {
        self.notifications.iter().filter(|n| !n.dismissed).collect()
    }
    
    // 模态框管理
    pub fn show_modal(&mut self, modal: ModalState) {
        self.modal = Some(modal);
    }
    
    pub fn hide_modal(&mut self) {
        self.modal = None;
    }
    
    pub fn is_modal_active(&self) -> bool {
        self.modal.is_some()
    }
    
    pub fn show_diff_modal(&mut self, commit_hash: String, diff_content: String) {
        let modal = ModalState {
            modal_type: ModalType::DiffViewer,
            title: format!("Git Diff - {}", &commit_hash[..8.min(commit_hash.len())]),
            content: diff_content,
            buttons: vec![
                ModalButton {
                    label: "Close".to_string(),
                    action: ModalAction::Cancel,
                }
            ],
            default_button: 0,
            can_cancel: true,
        };
        self.show_modal(modal);
    }
    
    pub fn show_ai_commit_modal(&mut self, message: String, status: String) {
        let modal = ModalState {
            modal_type: ModalType::AICommit,
            title: "AI Commit".to_string(),
            content: format!("Status: {}\n\nMessage: {}", status, message.trim()),
            buttons: vec![
                ModalButton {
                    label: "Commit (Enter)".to_string(),
                    action: ModalAction::Ok,
                },
                ModalButton {
                    label: "Edit (e)".to_string(),
                    action: ModalAction::Custom("edit".to_string()),
                },
                ModalButton {
                    label: "Cancel (Esc)".to_string(),
                    action: ModalAction::Cancel,
                }
            ],
            default_button: 0,
            can_cancel: true,
        };
        self.show_modal(modal);
    }

    pub fn show_ai_commit_push_modal(&mut self, status: String) {
        let modal = ModalState {
            modal_type: ModalType::AICommit,
            title: "AI Commit - Push".to_string(),
            content: format!("{}\n\nPush changes to remote repository?", status),
            buttons: vec![
                ModalButton {
                    label: "Push (y/Enter)".to_string(),
                    action: ModalAction::Yes,
                },
                ModalButton {
                    label: "Skip (n/Esc)".to_string(),
                    action: ModalAction::No,
                }
            ],
            default_button: 0,
            can_cancel: true,
        };
        self.show_modal(modal);
    }
    
    // 焦点管理
    pub fn set_focus(&mut self, panel: FocusPanel) {
        if self.focus.current_panel != panel {
            self.focus.panel_history.push(self.focus.current_panel.clone());
            self.focus.current_panel = panel;
            
            // 限制历史长度
            if self.focus.panel_history.len() > 10 {
                self.focus.panel_history.remove(0);
            }
        }
        
        // 同时更新焦点环
        self.focus.focus_ring.set_current(self.focus.current_panel.clone());
    }
    
    pub fn focus_previous(&mut self) {
        if let Some(previous_panel) = self.focus.panel_history.pop() {
            self.focus.current_panel = previous_panel;
            self.focus.focus_ring.set_current(self.focus.current_panel.clone());
        }
    }
    
    pub fn focus_next(&mut self) {
        let next_panel = self.focus.focus_ring.next();
        self.set_focus(next_panel);
    }
    
    pub fn get_current_focus(&self) -> &FocusPanel {
        &self.focus.current_panel
    }
    
    // 布局管理
    pub fn update_layout(&mut self, sidebar_width: u16, content_width: u16, detail_width: u16) {
        self.layout.sidebar_width = sidebar_width;
        self.layout.content_width = content_width;
        self.layout.detail_width = detail_width;
    }
    
    pub fn adjust_layout_ratios(&mut self, sidebar_delta: f32, content_delta: f32) {
        self.layout.adjust_panel_ratios(sidebar_delta, content_delta);
    }
    
    pub fn set_layout_mode(&mut self, mode: super::ui_state::LayoutMode) {
        self.layout.set_layout_mode(mode);
    }
}