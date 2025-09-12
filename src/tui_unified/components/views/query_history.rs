// 查询历史视图组件
use crate::query_history::QueryHistory;
use crate::tui_unified::{
    components::base::{
        component::{Component, ViewComponent, ViewType},
        events::EventResult,
    },
    components::widgets::list::ListWidget,
    git::models::QueryHistoryEntry,
    state::AppState,
};
use crossterm::event::KeyEvent;
use ratatui::{layout::Rect, Frame};

/// 查询历史视图组件 - 显示查询历史列表
pub struct QueryHistoryView {
    list_widget: ListWidget<QueryHistoryEntry>,
}

impl Default for QueryHistoryView {
    fn default() -> Self {
        Self::new()
    }
}

impl QueryHistoryView {
    pub fn new() -> Self {
        // 格式化函数：显示查询内容、时间和结果
        let format_fn = Box::new(|entry: &QueryHistoryEntry| -> String {
            let status_icon = if entry.success { "✅" } else { "❌" };
            let result_info = if let Some(count) = entry.result_count {
                format!(" ({} results)", count)
            } else {
                String::new()
            };
            let time_str = entry.timestamp.format("%m-%d %H:%M").to_string();
            format!(
                "📜 {} {} - {}{}",
                status_icon, entry.query, time_str, result_info
            )
        });

        // 样式函数：选中时高亮显示，成功和失败用不同颜色
        let style_fn = Box::new(
            |entry: &QueryHistoryEntry,
             is_selected: bool,
             is_focused: bool|
             -> ratatui::style::Style {
                use ratatui::style::{Color, Style};
                let base_color = if entry.success {
                    Color::Green
                } else {
                    Color::Red
                };

                if is_selected && is_focused {
                    Style::default().fg(Color::Yellow).bg(Color::DarkGray)
                } else if is_selected {
                    Style::default().fg(Color::White).bg(Color::DarkGray)
                } else {
                    Style::default().fg(base_color)
                }
            },
        );

        // 搜索函数：支持按查询内容和类型搜索
        let search_fn = Box::new(|entry: &QueryHistoryEntry, query: &str| -> bool {
            let query = query.to_lowercase();
            entry.query.to_lowercase().contains(&query)
                || entry
                    .query_type
                    .as_ref()
                    .is_some_and(|t| t.to_lowercase().contains(&query))
        });

        let list_widget = ListWidget::new("Query History".to_string(), format_fn, style_fn)
            .with_search_fn(search_fn);

        Self { list_widget }
    }

    pub async fn load_history(&mut self) {
        match QueryHistory::new(1000) {
            Ok(history) => {
                let entries = history.get_recent(100);
                let entries_owned: Vec<QueryHistoryEntry> = entries.into_iter().cloned().collect();
                self.list_widget.set_items(entries_owned);
            }
            Err(_) => {
                // 如果加载失败，设置空列表
                self.list_widget.set_items(vec![]);
            }
        }
    }

    pub fn selected_query(&self) -> Option<&QueryHistoryEntry> {
        self.list_widget.selected_item()
    }
}

impl Component for QueryHistoryView {
    fn name(&self) -> &str {
        "QueryHistoryView"
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

impl ViewComponent for QueryHistoryView {
    fn view_type(&self) -> ViewType {
        ViewType::QueryHistory
    }

    fn title(&self) -> String {
        "Query History".to_string()
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
