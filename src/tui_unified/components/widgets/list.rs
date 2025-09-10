// 通用列表组件
use crossterm::event::KeyEvent;
use ratatui::{Frame, layout::Rect, widgets::{Block, Borders, List, ListItem, ListState}, text::Text, style::{Color, Style}};
use crate::tui_unified::{
    state::AppState,
    components::base::{
        component::{Component, ViewComponent},
        events::EventResult
    }
};

/// 通用列表组件 - 可用于显示任意类型的列表数据
pub struct ListWidget<T> {
    focused: bool,
    items: Vec<T>,
    filtered_items: Vec<usize>, // 过滤后的项目索引
    selected_index: Option<usize>,
    scroll_offset: usize,
    list_state: ListState,
    title: String,
    format_fn: Box<dyn Fn(&T) -> String + Send>,
    style_fn: Box<dyn Fn(&T, bool, bool) -> Style + Send>,
    search_fn: Box<dyn Fn(&T, &str) -> bool + Send>, // 搜索函数
    current_search: Option<String>,
    show_search_results: bool,
}

impl<T> ListWidget<T> 
where 
    T: Clone + Send + 'static,
{
    pub fn new(
        title: String,
        format_fn: Box<dyn Fn(&T) -> String + Send>,
        style_fn: Box<dyn Fn(&T, bool, bool) -> Style + Send>,
    ) -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));

        // 默认搜索函数：总是匹配（子组件应该提供具体的搜索函数）
        let search_fn = Box::new(|_item: &T, _query: &str| {
            true // 默认总是匹配，具体组件应该重写搜索函数
        });

        Self {
            focused: false,
            items: Vec::new(),
            filtered_items: Vec::new(),
            selected_index: None,
            scroll_offset: 0,
            list_state,
            title,
            format_fn,
            style_fn,
            search_fn,
            current_search: None,
            show_search_results: false,
        }
    }

    pub fn with_search_fn(mut self, search_fn: Box<dyn Fn(&T, &str) -> bool + Send>) -> Self {
        self.search_fn = search_fn;
        self
    }

    pub fn with_items(mut self, items: Vec<T>) -> Self {
        self.items = items;
        if !self.items.is_empty() {
            self.selected_index = Some(0);
            self.list_state.select(Some(0));
        }
        self
    }

    pub fn set_items(&mut self, items: Vec<T>) {
        self.items = items;
        self.filtered_items = (0..self.items.len()).collect();
        self.update_selection_after_filter();
    }

    fn update_selection_after_filter(&mut self) {
        let effective_len = self.effective_len();
        if effective_len > 0 {
            if self.selected_index.is_none() || self.selected_index.unwrap_or(0) >= effective_len {
                self.selected_index = Some(0);
                self.list_state.select(Some(0));
            }
        } else {
            self.selected_index = None;
            self.list_state.select(None);
        }
    }

    fn effective_len(&self) -> usize {
        if self.show_search_results {
            self.filtered_items.len()
        } else {
            self.items.len()
        }
    }

    fn get_effective_item(&self, index: usize) -> Option<&T> {
        if self.show_search_results {
            self.filtered_items.get(index).and_then(|&i| self.items.get(i))
        } else {
            self.items.get(index)
        }
    }

    pub fn selected_item(&self) -> Option<&T> {
        self.selected_index.and_then(|idx| self.items.get(idx))
    }

    pub fn selected_index(&self) -> Option<usize> {
        self.selected_index
    }

    pub fn items(&self) -> &[T] {
        &self.items
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    fn move_selection(&mut self, direction: i32) {
        let effective_len = self.effective_len();
        if effective_len == 0 {
            return;
        }

        let current = self.selected_index.unwrap_or(0);
        let new_index = match direction {
            -1 => {
                if current > 0 {
                    current - 1
                } else {
                    effective_len - 1
                }
            }
            1 => {
                if current < effective_len - 1 {
                    current + 1
                } else {
                    0
                }
            }
            _ => current,
        };

        self.selected_index = Some(new_index);
        self.list_state.select(Some(new_index));
    }

    fn page_up(&mut self) {
        let effective_len = self.effective_len();
        if effective_len == 0 {
            return;
        }

        let current = self.selected_index.unwrap_or(0);
        let new_index = if current >= 10 { current - 10 } else { 0 };
        self.selected_index = Some(new_index);
        self.list_state.select(Some(new_index));
    }

    fn page_down(&mut self) {
        let effective_len = self.effective_len();
        if effective_len == 0 {
            return;
        }

        let current = self.selected_index.unwrap_or(0);
        let new_index = (current + 10).min(effective_len - 1);
        self.selected_index = Some(new_index);
        self.list_state.select(Some(new_index));
    }

    fn go_to_start(&mut self) {
        if self.effective_len() > 0 {
            self.selected_index = Some(0);
            self.list_state.select(Some(0));
        }
    }

    fn go_to_end(&mut self) {
        let effective_len = self.effective_len();
        if effective_len > 0 {
            let last_index = effective_len - 1;
            self.selected_index = Some(last_index);
            self.list_state.select(Some(last_index));
        }
    }
}

impl<T> Component for ListWidget<T>
where 
    T: Clone + Send + 'static,
{
    fn name(&self) -> &str {
        "ListWidget"
    }

    fn render(&mut self, frame: &mut Frame, area: Rect, _state: &AppState) {
        let border_style = if self.focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::White)
        };

        let list_items: Vec<ListItem> = (0..self.effective_len())
            .filter_map(|i| {
                self.get_effective_item(i).map(|item| {
                    let is_selected = Some(i) == self.selected_index;
                    let content = (self.format_fn)(item);
                    let style = (self.style_fn)(item, is_selected, self.focused);
                    ListItem::new(Text::raw(content)).style(style)
                })
            })
            .collect();

        let list = List::new(list_items)
            .block(
                Block::default()
                    .title(self.title.as_str())
                    .borders(Borders::ALL)
                    .border_style(border_style)
            )
            .highlight_style(if self.focused {
                Style::default().fg(Color::Black).bg(Color::Yellow)
            } else {
                Style::default().fg(Color::White).bg(Color::DarkGray)
            });

        frame.render_stateful_widget(list, area, &mut self.list_state);
    }

    fn handle_key_event(&mut self, key: KeyEvent, _state: &mut AppState) -> EventResult {
        use crossterm::event::KeyCode;

        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.move_selection(-1);
                EventResult::Handled
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.move_selection(1);
                EventResult::Handled
            }
            KeyCode::PageUp | KeyCode::Char('u') => {
                self.page_up();
                EventResult::Handled
            }
            KeyCode::PageDown | KeyCode::Char('d') => {
                self.page_down();
                EventResult::Handled
            }
            KeyCode::Home | KeyCode::Char('g') => {
                self.go_to_start();
                EventResult::Handled
            }
            KeyCode::End | KeyCode::Char('G') => {
                self.go_to_end();
                EventResult::Handled
            }
            _ => EventResult::NotHandled
        }
    }

    fn is_focused(&self) -> bool {
        self.focused
    }

    fn set_focus(&mut self, focused: bool) {
        self.focused = focused;
    }

    fn can_focus(&self) -> bool {
        true
    }

    fn min_size(&self) -> (u16, u16) {
        (30, 10)
    }
}

impl<T> ViewComponent for ListWidget<T>
where 
    T: Clone + Send + 'static,
{
    fn view_type(&self) -> crate::tui_unified::components::base::component::ViewType {
        // 默认返回GitLog，具体组件应该重写这个方法
        crate::tui_unified::components::base::component::ViewType::GitLog
    }

    fn title(&self) -> String {
        self.title.clone()
    }

    fn supports_search(&self) -> bool {
        true
    }

    fn search(&mut self, query: &str) -> EventResult {
        if query.is_empty() {
            return self.clear_search();
        }

        let query = query.to_lowercase();
        self.filtered_items.clear();
        
        for (i, item) in self.items.iter().enumerate() {
            if (self.search_fn)(item, &query) {
                self.filtered_items.push(i);
            }
        }

        self.current_search = Some(query);
        self.show_search_results = true;
        self.update_selection_after_filter();
        
        EventResult::Handled
    }

    fn clear_search(&mut self) -> EventResult {
        self.current_search = None;
        self.show_search_results = false;
        self.filtered_items.clear();
        self.update_selection_after_filter();
        EventResult::Handled
    }

    fn selected_index(&self) -> Option<usize> {
        self.selected_index
    }

    fn set_selected_index(&mut self, index: Option<usize>) {
        if let Some(idx) = index {
            if idx < self.items.len() {
                self.selected_index = Some(idx);
                self.list_state.select(Some(idx));
            }
        } else {
            self.selected_index = None;
            self.list_state.select(None);
        }
    }
}