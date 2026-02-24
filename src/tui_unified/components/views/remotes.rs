// Git 远程仓库视图组件
use crate::tui_unified::{
    components::base::{
        component::{Component, ViewComponent, ViewType},
        events::EventResult,
    },
    components::widgets::list::ListWidget,
    git::models::Remote,
    state::AppState,
};
use crossterm::event::KeyEvent;
use ratatui::{layout::Rect, Frame};

/// Git 远程仓库视图组件 - 显示远程仓库列表
pub struct RemotesView {
    list_widget: ListWidget<Remote>,
}

impl Default for RemotesView {
    fn default() -> Self {
        Self::new()
    }
}

impl RemotesView {
    pub fn new() -> Self {
        // 格式化函数：显示远程仓库名称和URL
        let format_fn = Box::new(|remote: &Remote| -> String {
            format!("📡 {} → {}", remote.name, remote.url)
        });

        // 样式函数：选中时高亮显示
        let style_fn = Box::new(super::shared::default_selection_style);

        // 搜索函数：支持按远程仓库名称和URL搜索
        let search_fn = Box::new(|remote: &Remote, query: &str| -> bool {
            let query = query.to_lowercase();
            remote.name.to_lowercase().contains(&query)
                || remote.url.to_lowercase().contains(&query)
        });

        let list_widget = ListWidget::new("Git Remotes".to_string(), format_fn, style_fn)
            .with_search_fn(search_fn);

        Self { list_widget }
    }

    pub async fn load_remotes(&mut self, app_state: &AppState) {
        // 从状态中获取remotes数据并转换为Remote模型
        let remotes: Vec<Remote> = app_state
            .repo_state
            .remotes
            .iter()
            .map(|r| Remote {
                name: r.name.clone(),
                url: r.url.clone(),
            })
            .collect();

        self.list_widget.set_items(remotes);
    }

    pub fn selected_remote(&self) -> Option<&Remote> {
        self.list_widget.selected_item()
    }
}

impl Component for RemotesView {
    fn name(&self) -> &str {
        "RemotesView"
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

impl ViewComponent for RemotesView {
    fn view_type(&self) -> ViewType {
        ViewType::Remotes
    }

    fn title(&self) -> String {
        "Git Remotes".to_string()
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
