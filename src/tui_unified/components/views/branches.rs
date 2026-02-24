// 分支视图组件
use crate::tui_unified::{
    components::{
        base::{
            component::{Component, ViewComponent, ViewType},
            events::EventResult,
        },
        widgets::list::ListWidget,
    },
    state::{git_state::Branch, AppState},
};
use crossterm::event::KeyEvent;
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    Frame,
};

/// 分支视图 - 显示所有分支
pub struct BranchesView {
    list_widget: ListWidget<Branch>,
    show_remotes: bool,
}

impl Default for BranchesView {
    fn default() -> Self {
        Self::new()
    }
}

impl BranchesView {
    pub fn new() -> Self {
        let format_fn = Box::new(|branch: &Branch| -> String {
            let indicator = if branch.is_current { "* " } else { "  " };
            let upstream_info = if let Some(ref upstream) = branch.upstream {
                format!(" -> {}", upstream)
            } else {
                String::new()
            };
            format!("{}{}{}", indicator, branch.name, upstream_info)
        });

        let style_fn = Box::new(
            |branch: &Branch, is_selected: bool, is_focused: bool| -> Style {
                let base_style = if branch.is_current {
                    Style::default().fg(Color::Green)
                } else {
                    Style::default().fg(Color::White)
                };

                if is_selected && is_focused {
                    base_style.bg(Color::Yellow).fg(Color::Black)
                } else if is_selected {
                    base_style.bg(Color::DarkGray)
                } else {
                    base_style
                }
            },
        );

        let search_fn = Box::new(|branch: &Branch, query: &str| -> bool {
            let query = query.to_lowercase();
            branch.name.to_lowercase().contains(&query)
                || branch.full_name.to_lowercase().contains(&query)
                || branch
                    .upstream
                    .as_ref()
                    .is_some_and(|u| u.to_lowercase().contains(&query))
        });

        let list_widget =
            ListWidget::new("Branches".to_string(), format_fn, style_fn).with_search_fn(search_fn);

        Self {
            list_widget,
            show_remotes: false,
        }
    }

    pub fn selected_branch(&self) -> Option<&Branch> {
        self.list_widget.selected_item()
    }

    pub fn toggle_remotes(&mut self) {
        self.show_remotes = !self.show_remotes;
        self.update_title();
    }

    pub fn refresh_branches(&mut self, state: &AppState) {
        let branches = if self.show_remotes {
            // TODO: 包含远程分支
            state.repo_state.branches.clone()
        } else {
            state
                .repo_state
                .branches
                .iter()
                .filter(|branch| !branch.name.starts_with("remotes/"))
                .cloned()
                .collect()
        };
        self.list_widget.set_items(branches);
    }

    /// 通知应用状态当前选中的分支
    pub fn update_selected_branch_in_state(&self, state: &mut AppState) {
        if let Some(selected_branch) = self.selected_branch() {
            state.select_branch(selected_branch.name.clone());
        }
    }

    fn update_title(&mut self) {
        let title = if self.show_remotes {
            "Branches (All)".to_string()
        } else {
            "Branches (Local)".to_string()
        };
        self.list_widget.set_title(title);
    }
}

impl Component for BranchesView {
    fn name(&self) -> &str {
        "BranchesView"
    }

    fn render(&mut self, frame: &mut Frame, area: Rect, state: &AppState) {
        // 确保分支列表是最新的
        if self.list_widget.len() != state.repo_state.branches.len() {
            self.refresh_branches(state);
        }

        self.list_widget.render(frame, area, state);
    }

    fn handle_key_event(&mut self, key: KeyEvent, state: &mut AppState) -> EventResult {
        use crossterm::event::KeyCode;

        // 处理视图特定的按键
        match key.code {
            KeyCode::Tab => {
                self.toggle_remotes();
                self.refresh_branches(state);
                EventResult::Handled
            }
            KeyCode::Enter | KeyCode::Char('c') => {
                // 切换到选中的分支 (Enter 或 c 键)
                if let Some(selected_branch) = self.selected_branch() {
                    // 不允许切换到当前分支
                    if selected_branch.is_current {
                        state.add_notification(
                            "Already on this branch".to_string(),
                            crate::tui_unified::state::app_state::NotificationLevel::Warning,
                        );
                    } else {
                        // 请求直接分支切换（不通过模态框）
                        state.request_direct_branch_switch(selected_branch.name.clone());
                    }
                }
                EventResult::Handled
            }
            KeyCode::Char('r') => {
                // 刷新分支列表
                self.refresh_branches(state);
                EventResult::Handled
            }
            // 移除了 'c' 键，因为现在它用于分支切换
            KeyCode::Char('d') => {
                // 显示选中分支的git diff
                if let Some(selected_branch) = self.selected_branch() {
                    if let Some(commit_hash) = &selected_branch.last_commit {
                        state.request_diff(commit_hash.clone());
                        EventResult::Handled
                    } else {
                        state.add_notification(
                            "No commit hash available for this branch".to_string(),
                            crate::tui_unified::state::app_state::NotificationLevel::Warning,
                        );
                        EventResult::Handled
                    }
                } else {
                    EventResult::NotHandled
                }
            }
            KeyCode::Up | KeyCode::Down | KeyCode::Char('k') | KeyCode::Char('j') => {
                // 处理方向键选择，并更新分支状态
                let old_selection = self.list_widget.selected_index();
                let result = self.list_widget.handle_key_event(key, state);
                let new_selection = self.list_widget.selected_index();

                // 如果选择发生变化，更新应用状态中的选中分支
                if old_selection != new_selection {
                    self.update_selected_branch_in_state(state);
                }

                result
            }
            _ => {
                // 委托给列表组件处理其他按键
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
        (40, 15)
    }
}

impl ViewComponent for BranchesView {
    fn view_type(&self) -> ViewType {
        ViewType::Branches
    }

    fn title(&self) -> String {
        if self.show_remotes {
            "Branches (All)".to_string()
        } else {
            "Branches (Local)".to_string()
        }
    }

    fn supports_search(&self) -> bool {
        true
    }

    fn search(&mut self, query: &str) -> EventResult {
        // TODO: 实现按分支名搜索
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
                self.list_widget
                    .set_items(self.list_widget.items().to_vec());
            }
        }
    }
}
