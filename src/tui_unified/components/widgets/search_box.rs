use ratatui::{prelude::*, widgets::*};
use crossterm::event::{KeyEvent, KeyCode};
use crate::tui_unified::{
    state::AppState,
    components::base::{
        component::Component,
        events::EventResult
    }
};

/// 搜索框组件 - 提供搜索输入界面
pub struct SearchBox {
    input: String,
    cursor_position: usize,
    focused: bool,
    placeholder: String,
    search_active: bool,
}

impl SearchBox {
    pub fn new() -> Self {
        Self {
            input: String::new(),
            cursor_position: 0,
            focused: false,
            placeholder: "Type to search...".to_string(),
            search_active: false,
        }
    }

    pub fn with_placeholder(mut self, placeholder: String) -> Self {
        self.placeholder = placeholder;
        self
    }

    pub fn get_input(&self) -> &str {
        &self.input
    }

    pub fn set_input(&mut self, input: String) {
        self.input = input;
        self.cursor_position = self.input.len();
    }

    pub fn clear(&mut self) {
        self.input.clear();
        self.cursor_position = 0;
    }

    pub fn is_search_active(&self) -> bool {
        self.search_active
    }

    pub fn set_search_active(&mut self, active: bool) {
        self.search_active = active;
        if !active {
            self.clear();
        }
    }

    fn handle_input_char(&mut self, c: char) {
        self.input.insert(self.cursor_position, c);
        self.cursor_position += 1;
    }

    fn handle_backspace(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
            self.input.remove(self.cursor_position);
        }
    }

    fn move_cursor_left(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
        }
    }

    fn move_cursor_right(&mut self) {
        if self.cursor_position < self.input.len() {
            self.cursor_position += 1;
        }
    }
}

impl Component for SearchBox {
    fn name(&self) -> &str {
        "SearchBox"
    }

    fn render(&mut self, frame: &mut Frame, area: Rect, _state: &AppState) {
        let border_style = if self.focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::Gray)
        };

        let input_style = if self.focused {
            Style::default().fg(Color::White).bg(Color::DarkGray)
        } else {
            Style::default().fg(Color::Gray)
        };

        let display_text = if self.input.is_empty() && !self.focused {
            self.placeholder.as_str()
        } else {
            self.input.as_str()
        };

        let title = if self.search_active {
            "🔍 Search (ESC to cancel)"
        } else {
            "🔍 Search (/ to start)"
        };

        let paragraph = Paragraph::new(display_text)
            .style(input_style)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(title)
                    .border_style(border_style)
            );

        frame.render_widget(paragraph, area);

        // 显示光标位置（如果获得焦点）
        if self.focused && self.search_active {
            let cursor_x = area.x + 1 + self.cursor_position as u16;
            let cursor_y = area.y + 1;
            if cursor_x < area.x + area.width.saturating_sub(1) {
                frame.set_cursor(cursor_x, cursor_y);
            }
        }
    }

    fn handle_key_event(&mut self, key: KeyEvent, _state: &mut AppState) -> EventResult {
        if !self.search_active {
            return EventResult::NotHandled;
        }

        match key.code {
            KeyCode::Char(c) => {
                self.handle_input_char(c);
                EventResult::Handled
            }
            KeyCode::Backspace => {
                self.handle_backspace();
                EventResult::Handled
            }
            KeyCode::Left => {
                self.move_cursor_left();
                EventResult::Handled
            }
            KeyCode::Right => {
                self.move_cursor_right();
                EventResult::Handled
            }
            KeyCode::Home => {
                self.cursor_position = 0;
                EventResult::Handled
            }
            KeyCode::End => {
                self.cursor_position = self.input.len();
                EventResult::Handled
            }
            KeyCode::Enter => {
                // 搜索确认，由父组件处理
                EventResult::NotHandled
            }
            KeyCode::Esc => {
                // 取消搜索，由父组件处理
                EventResult::NotHandled
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
        (20, 3)
    }
}

impl Default for SearchBox {
    fn default() -> Self {
        Self::new()
    }
}
