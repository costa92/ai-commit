// 事件处理系统
use crossterm::event::{KeyEvent, KeyCode, KeyModifiers};
use crate::tui_unified::focus::FocusPanel;

#[derive(Debug, Clone, PartialEq)]
pub enum EventResult {
    Handled,
    NotHandled,
    Consumed(StateChange),
    Propagate(Vec<StateChange>),
}

#[derive(Debug, Clone)]
pub enum Navigation {
    NextPanel,
    PrevPanel,
    NextItem,
    PrevItem,
    FirstItem,
    LastItem,
    PageUp,
    PageDown,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StateChange {
    FocusChange(FocusPanel),
    ViewChange(crate::tui_unified::state::app_state::ViewType),
    SelectionChange { 
        component: String,
        index: Option<usize>
    },
    SearchModeToggle(bool),
    SearchQuery(String),
    RefreshData,
    ShowHelp(bool),
    ShowDetails(bool),
    ToggleMode(String),
}

#[derive(Debug, Clone)]
pub enum AsyncTask {
    GitRefresh,
    LoadCommitDiff(String),
    SearchCommits(String),
    FilterBranches(String),
}

#[derive(Debug, Clone)]
pub enum CustomEvent {
    KeyPress(KeyEvent),
    Resize(u16, u16),
    Tick,
    GitDataUpdate,
    SearchComplete(String, Vec<usize>),
    ErrorOccurred(String),
}

/// 事件分发器 - 处理全局按键绑定和事件路由
pub struct EventDispatcher {
    search_mode: bool,
    help_mode: bool,
    current_search_query: String,
}

impl EventDispatcher {
    pub fn new() -> Self {
        Self {
            search_mode: false,
            help_mode: false,
            current_search_query: String::new(),
        }
    }

    /// 处理全局按键事件
    pub fn handle_global_key(&mut self, key: KeyEvent) -> EventResult {
        // 处理全局按键绑定
        match key.code {
            KeyCode::Char('q') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                // Ctrl+Q 退出应用
                EventResult::Consumed(StateChange::ViewChange(
                    crate::tui_unified::state::app_state::ViewType::GitLog
                ))
            },
            KeyCode::Char('h') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                // Ctrl+H 显示/隐藏帮助
                self.help_mode = !self.help_mode;
                EventResult::Consumed(StateChange::ShowHelp(self.help_mode))
            },
            KeyCode::Char('/') => {
                // 开始搜索模式
                self.search_mode = true;
                self.current_search_query.clear();
                EventResult::Consumed(StateChange::SearchModeToggle(true))
            },
            KeyCode::Esc => {
                // 退出搜索模式或帮助模式
                if self.search_mode {
                    self.search_mode = false;
                    self.current_search_query.clear();
                    EventResult::Consumed(StateChange::SearchModeToggle(false))
                } else if self.help_mode {
                    self.help_mode = false;
                    EventResult::Consumed(StateChange::ShowHelp(false))
                } else {
                    EventResult::NotHandled
                }
            },
            KeyCode::Tab => {
                // 切换面板焦点
                EventResult::Consumed(StateChange::FocusChange(FocusPanel::Content))
            },
            KeyCode::F(5) => {
                // F5 刷新数据
                EventResult::Consumed(StateChange::RefreshData)
            },
            _ => {
                if self.search_mode {
                    self.handle_search_input(key)
                } else {
                    EventResult::NotHandled
                }
            }
        }
    }

    /// 处理搜索输入
    fn handle_search_input(&mut self, key: KeyEvent) -> EventResult {
        match key.code {
            KeyCode::Char(c) => {
                self.current_search_query.push(c);
                EventResult::Consumed(StateChange::SearchQuery(self.current_search_query.clone()))
            },
            KeyCode::Backspace => {
                self.current_search_query.pop();
                EventResult::Consumed(StateChange::SearchQuery(self.current_search_query.clone()))
            },
            KeyCode::Enter => {
                // 执行搜索
                let query = self.current_search_query.clone();
                self.search_mode = false;
                EventResult::Propagate(vec![
                    StateChange::SearchModeToggle(false),
                    StateChange::SearchQuery(query)
                ])
            },
            _ => EventResult::NotHandled
        }
    }

    pub fn is_search_mode(&self) -> bool {
        self.search_mode
    }

    pub fn is_help_mode(&self) -> bool {
        self.help_mode
    }

    pub fn current_search_query(&self) -> &str {
        &self.current_search_query
    }
}

impl Default for EventDispatcher {
    fn default() -> Self {
        Self::new()
    }
}