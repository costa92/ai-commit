// Git stash视图组件
use crossterm::event::KeyEvent;
use ratatui::{Frame, layout::Rect};
use crate::tui_unified::{
    state::AppState,
    components::base::{
        component::{Component, ViewComponent, ViewType},
        events::EventResult
    },
    components::widgets::list::ListWidget,
    git::models::Stash,
};

/// Git stash视图组件 - 显示stash列表
pub struct StashView {
    list_widget: ListWidget<Stash>,
}

impl StashView {
    pub fn new() -> Self {
        // 格式化函数：显示stash索引、消息和分支
        let format_fn = Box::new(|stash: &Stash| -> String {
            format!("💾 stash@{{{}}}: On {} - {}", stash.index, stash.branch, stash.message)
        });

        // 样式函数：选中时高亮显示
        let style_fn = Box::new(|_stash: &Stash, is_selected: bool, is_focused: bool| -> ratatui::style::Style {
            use ratatui::style::{Color, Style};
            if is_selected && is_focused {
                Style::default().fg(Color::Yellow).bg(Color::DarkGray)
            } else if is_selected {
                Style::default().fg(Color::White).bg(Color::DarkGray)
            } else {
                Style::default().fg(Color::White)
            }
        });

        // 搜索函数：支持按消息和分支搜索
        let search_fn = Box::new(|stash: &Stash, query: &str| -> bool {
            let query = query.to_lowercase();
            stash.message.to_lowercase().contains(&query) ||
            stash.branch.to_lowercase().contains(&query) ||
            format!("stash@{{{}}}", stash.index).contains(&query)
        });

        let list_widget = ListWidget::new(
            "Git Stash".to_string(),
            format_fn,
            style_fn,
        ).with_search_fn(search_fn);

        Self {
            list_widget,
        }
    }

    pub async fn load_stashes(&mut self, app_state: &AppState) {
        // 从状态中获取stashes数据并转换为Stash模型
        let stashes: Vec<Stash> = app_state.repo_state.stashes
            .iter()
            .map(|s| Stash {
                index: s.index as u32,
                message: s.message.clone(),
                branch: s.branch.clone(),
            })
            .collect();
        
        self.list_widget.set_items(stashes);
    }

    pub fn selected_stash(&self) -> Option<&Stash> {
        self.list_widget.selected_item()
    }
}

impl Component for StashView {
    fn name(&self) -> &str {
        "StashView"
    }

    fn render(&mut self, frame: &mut Frame, area: Rect, state: &AppState) {
        self.list_widget.render(frame, area, state);
    }

    fn handle_key_event(&mut self, key: KeyEvent, state: &mut AppState) -> EventResult {
        self.list_widget.handle_key_event(key, state)
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
        self.list_widget.min_size()
    }
}

impl ViewComponent for StashView {
    fn view_type(&self) -> ViewType {
        ViewType::Stash
    }

    fn title(&self) -> String {
        "Git Stash".to_string()
    }

    fn supports_search(&self) -> bool {
        true
    }

    fn search(&mut self, query: &str) -> EventResult {
        self.list_widget.search(query)
    }

    fn clear_search(&mut self) -> EventResult {
        self.list_widget.clear_search()
    }

    fn selected_index(&self) -> Option<usize> {
        self.list_widget.selected_index()
    }

    fn set_selected_index(&mut self, index: Option<usize>) {
        self.list_widget.set_selected_index(index)
    }
}