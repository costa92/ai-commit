// Tags视图组件
use crossterm::event::KeyEvent;
use ratatui::{Frame, layout::Rect, style::{Color, Style}};
use crate::tui_unified::{
    state::{AppState, git_state::Tag},
    components::{
        base::{
            component::{Component, ViewComponent, ViewType},
            events::EventResult
        },
        widgets::list::ListWidget
    }
};

/// Tags视图 - 显示所有标签
pub struct TagsView {
    list_widget: ListWidget<Tag>,
}

impl TagsView {
    pub fn new() -> Self {
        let format_fn = Box::new(|tag: &Tag| -> String {
            if let Some(ref message) = tag.message {
                format!("🏷️  {} - {} ({})", tag.name, message, &tag.commit_hash[..8.min(tag.commit_hash.len())])
            } else {
                format!("🏷️  {} ({})", tag.name, &tag.commit_hash[..8.min(tag.commit_hash.len())])
            }
        });

        let style_fn = Box::new(|_tag: &Tag, is_selected: bool, is_focused: bool| -> Style {
            if is_selected && is_focused {
                Style::default().fg(Color::Black).bg(Color::Yellow)
            } else if is_selected {
                Style::default().fg(Color::White).bg(Color::DarkGray)
            } else {
                Style::default().fg(Color::White)
            }
        });

        let search_fn = Box::new(|tag: &Tag, query: &str| -> bool {
            let query = query.to_lowercase();
            tag.name.to_lowercase().contains(&query) ||
            tag.commit_hash.to_lowercase().contains(&query) ||
            tag.message.as_ref().map_or(false, |m| m.to_lowercase().contains(&query))
        });

        let list_widget = ListWidget::new(
            "Tags".to_string(),
            format_fn,
            style_fn,
        ).with_search_fn(search_fn);

        Self {
            list_widget,
        }
    }

    pub fn selected_tag(&self) -> Option<&Tag> {
        self.list_widget.selected_item()
    }

    pub fn refresh_tags(&mut self, state: &AppState) {
        let tags = state.repo_state.tags.clone();
        self.list_widget.set_items(tags);
    }
}

impl Component for TagsView {
    fn name(&self) -> &str {
        "TagsView"
    }

    fn render(&mut self, frame: &mut Frame, area: Rect, state: &AppState) {
        // 确保标签列表是最新的
        if self.list_widget.len() != state.repo_state.tags.len() {
            self.refresh_tags(state);
        }

        self.list_widget.render(frame, area, state);
    }

    fn handle_key_event(&mut self, key: KeyEvent, state: &mut AppState) -> EventResult {
        use crossterm::event::KeyCode;

        // 处理视图特定的按键
        match key.code {
            KeyCode::Enter => {
                // TODO: 显示选中标签的详细信息或差异
                EventResult::Handled
            }
            KeyCode::Char('r') => {
                // 刷新标签列表
                self.refresh_tags(state);
                EventResult::Handled
            }
            KeyCode::Char('d') => {
                // TODO: 删除选中的标签
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
        (50, 15)
    }
}

impl ViewComponent for TagsView {
    fn view_type(&self) -> ViewType {
        ViewType::Tags
    }

    fn title(&self) -> String {
        "Tags".to_string()
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
        if let Some(idx) = index {
            if idx < self.list_widget.len() {
                self.list_widget.set_items(self.list_widget.items().to_vec());
            }
        }
    }
}