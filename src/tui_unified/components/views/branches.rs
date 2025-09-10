// 分支视图组件
use crossterm::event::KeyEvent;
use ratatui::{Frame, layout::Rect, style::{Color, Style}};
use crate::tui_unified::{
    state::{AppState, git_state::Branch},
    components::{
        base::{
            component::{Component, ViewComponent, ViewType},
            events::EventResult
        },
        widgets::list::ListWidget
    }
};

/// 分支视图 - 显示所有分支
pub struct BranchesView {
    list_widget: ListWidget<Branch>,
    show_remotes: bool,
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

        let style_fn = Box::new(|branch: &Branch, is_selected: bool, is_focused: bool| -> Style {
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
        });

        let search_fn = Box::new(|branch: &Branch, query: &str| -> bool {
            let query = query.to_lowercase();
            branch.name.to_lowercase().contains(&query) ||
            branch.full_name.to_lowercase().contains(&query) ||
            branch.upstream.as_ref().map_or(false, |u| u.to_lowercase().contains(&query))
        });

        let list_widget = ListWidget::new(
            "Branches".to_string(),
            format_fn,
            style_fn,
        ).with_search_fn(search_fn);

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
            state.repo_state.branches
                .iter()
                .filter(|branch| !branch.name.starts_with("remotes/"))
                .cloned()
                .collect()
        };
        self.list_widget.set_items(branches);
    }

    fn update_title(&mut self) {
        let title = if self.show_remotes {
            "Branches (All)".to_string()
        } else {
            "Branches (Local)".to_string()
        };

        // 重新创建 ListWidget 来更新标题
        let format_fn: Box<dyn Fn(&Branch) -> String + Send> = Box::new(|branch: &Branch| -> String {
            let indicator = if branch.is_current { "* " } else { "  " };
            let upstream_info = if let Some(ref upstream) = branch.upstream {
                format!(" -> {}", upstream)
            } else {
                String::new()
            };
            format!("{}{}{}", indicator, branch.name, upstream_info)
        });

        let style_fn = Box::new(|branch: &Branch, is_selected: bool, is_focused: bool| -> Style {
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
        });

        let current_items = self.list_widget.items().to_vec();
        let selected = self.list_widget.selected_index();

        self.list_widget = ListWidget::new(title, format_fn, style_fn).with_items(current_items);
        
        if let Some(_idx) = selected {
            self.list_widget.set_items(self.list_widget.items().to_vec());
        }
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
                            crate::tui_unified::state::app_state::NotificationLevel::Warning
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
                // TODO: 删除选中的分支
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
                self.list_widget.set_items(self.list_widget.items().to_vec());
            }
        }
    }
}