// Git 日志视图组件
use crossterm::event::KeyEvent;
use ratatui::{Frame, layout::Rect, style::{Color, Style}};
use crate::tui_unified::{
    state::{AppState, git_state::Commit},
    components::{
        base::{
            component::{Component, ViewComponent, ViewType},
            events::EventResult
        },
        widgets::list::ListWidget
    }
};

/// Git 日志视图 - 显示提交历史
pub struct GitLogView {
    list_widget: ListWidget<Commit>,
    show_details: bool,
}

impl GitLogView {
    pub fn new() -> Self {
        let format_fn = Box::new(|commit: &Commit| -> String {
            format!(
                "{} {} - {}",
                &commit.hash[..8.min(commit.hash.len())],
                commit.author,
                commit.message.lines().next().unwrap_or(&commit.message)
            )
        });

        let style_fn = Box::new(|_commit: &Commit, is_selected: bool, is_focused: bool| -> Style {
            if is_selected && is_focused {
                Style::default().fg(Color::Black).bg(Color::Yellow)
            } else if is_selected {
                Style::default().fg(Color::White).bg(Color::DarkGray)
            } else {
                Style::default().fg(Color::White)
            }
        });

        let search_fn = Box::new(|commit: &Commit, query: &str| -> bool {
            let query = query.to_lowercase();
            commit.message.to_lowercase().contains(&query) ||
            commit.author.to_lowercase().contains(&query) ||
            commit.author_email.to_lowercase().contains(&query) ||
            commit.hash.to_lowercase().contains(&query)
        });

        let list_widget = ListWidget::new(
            "Git Log".to_string(),
            format_fn,
            style_fn,
        ).with_search_fn(search_fn);

        Self {
            list_widget,
            show_details: false,
        }
    }

    pub fn selected_commit(&self) -> Option<&Commit> {
        self.list_widget.selected_item()
    }

    /// 更新commit列表数据
    pub fn update_commits(&mut self, commits: Vec<Commit>) {
        self.list_widget.set_items(commits);
    }

    pub fn toggle_details(&mut self) {
        self.show_details = !self.show_details;
        self.update_title();
    }

    pub fn refresh_commits(&mut self, state: &AppState) {
        let commits = state.repo_state.commits.clone();
        self.list_widget.set_items(commits);
    }

    fn update_title(&mut self) {
        let title = if self.show_details {
            "Git Log (Details)".to_string()
        } else {
            "Git Log".to_string()
        };

        // 需要重新创建 ListWidget 来更新标题
        let format_fn: Box<dyn Fn(&Commit) -> String + Send> = if self.show_details {
            Box::new(|commit: &Commit| -> String {
                format!(
                    "{}\n{} - {}\n{}\nFiles: {} | Date: {}",
                    &commit.hash[..8.min(commit.hash.len())],
                    commit.author,
                    commit.author_email,
                    commit.message,
                    commit.files_changed,
                    commit.date.format("%Y-%m-%d %H:%M")
                )
            })
        } else {
            Box::new(|commit: &Commit| -> String {
                format!(
                    "{} {} - {}",
                    &commit.hash[..8.min(commit.hash.len())],
                    commit.author,
                    commit.message.lines().next().unwrap_or(&commit.message)
                )
            })
        };

        let style_fn = Box::new(|_commit: &Commit, is_selected: bool, is_focused: bool| -> Style {
            if is_selected && is_focused {
                Style::default().fg(Color::Black).bg(Color::Yellow)
            } else if is_selected {
                Style::default().fg(Color::White).bg(Color::DarkGray)
            } else {
                Style::default().fg(Color::White)
            }
        });

        let current_items = self.list_widget.items().to_vec();
        let selected = self.list_widget.selected_index();

        self.list_widget = ListWidget::new(title, format_fn, style_fn).with_items(current_items);
        
        if let Some(_idx) = selected {
            self.list_widget.set_items(self.list_widget.items().to_vec());
        }
    }
}

impl Component for GitLogView {
    fn name(&self) -> &str {
        "GitLogView"
    }

    fn render(&mut self, frame: &mut Frame, area: Rect, state: &AppState) {
        // 确保提交列表是最新的
        if self.list_widget.len() != state.repo_state.commits.len() {
            self.refresh_commits(state);
        }

        self.list_widget.render(frame, area, state);
    }

    fn handle_key_event(&mut self, key: KeyEvent, state: &mut AppState) -> EventResult {
        use crossterm::event::KeyCode;

        // 处理视图特定的按键
        match key.code {
            KeyCode::Tab => {
                self.toggle_details();
                EventResult::Handled
            }
            KeyCode::Enter => {
                // 请求显示选中提交的diff
                if let Some(selected_commit) = self.selected_commit() {
                    state.request_diff(selected_commit.hash.clone());
                    EventResult::Handled
                } else {
                    EventResult::NotHandled
                }
            }
            KeyCode::Char('r') => {
                // 刷新提交历史
                self.refresh_commits(state);
                EventResult::Handled
            }
            _ => {
                // 委托给列表组件处理
                self.list_widget.handle_key_event(key, state)
            }
        }
    }

    fn is_focused(&self) -> bool {
        self.list_widget.is_focused()
    }

    fn set_focus(&mut self, focused: bool) {
        self.list_widget.set_focus(focused);
    }

    fn can_focus(&self) -> bool {
        self.list_widget.can_focus()
    }

    fn min_size(&self) -> (u16, u16) {
        (60, 20)
    }
}

impl ViewComponent for GitLogView {
    fn view_type(&self) -> ViewType {
        ViewType::GitLog
    }

    fn title(&self) -> String {
        if self.show_details {
            "Git Log (Details)".to_string()
        } else {
            "Git Log".to_string()
        }
    }

    fn supports_search(&self) -> bool {
        true
    }

    fn search(&mut self, query: &str) -> EventResult {
        // TODO: 实现按提交消息或作者搜索
        self.list_widget.search(query)
    }

    fn clear_search(&mut self) -> EventResult {
        self.list_widget.clear_search()
    }

    fn selected_index(&self) -> Option<usize> {
        self.list_widget.selected_index()
    }

    fn set_selected_index(&mut self, index: Option<usize>) {
        if let Some(idx) = index {
            if idx < self.list_widget.len() {
                self.list_widget.set_items(self.list_widget.items().to_vec());
            }
        }
    }
}

pub struct BranchesView; 
pub struct TagsView;
pub struct RemotesView;
pub struct StashView;
pub struct QueryHistoryView;