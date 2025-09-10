use ratatui::{prelude::*, widgets::*};
use crossterm::event::{KeyEvent, KeyCode};
use crate::tui_unified::{
    state::AppState,
    components::base::{
        component::Component,
        events::EventResult
    }
};

/// Commit 消息编辑器组件 - 支持多行编辑
pub struct CommitEditor {
    lines: Vec<String>,
    cursor_line: usize,
    cursor_col: usize,
    focused: bool,
    scroll_offset: usize,
}

impl CommitEditor {
    pub fn new() -> Self {
        Self {
            lines: vec![String::new()],
            cursor_line: 0,
            cursor_col: 0,
            focused: false,
            scroll_offset: 0,
        }
    }

    pub fn set_content(&mut self, content: &str) {
        self.lines = content.lines().map(|s| s.to_string()).collect();
        if self.lines.is_empty() {
            self.lines.push(String::new());
        }
        self.cursor_line = 0;
        self.cursor_col = 0;
        self.scroll_offset = 0;
    }

    pub fn get_content(&self) -> String {
        self.lines.join("\n")
    }

    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    fn handle_input_char(&mut self, c: char) {
        if self.cursor_line < self.lines.len() {
            self.lines[self.cursor_line].insert(self.cursor_col, c);
            self.cursor_col += 1;
        }
    }

    fn handle_backspace(&mut self) {
        if self.cursor_col > 0 {
            if self.cursor_line < self.lines.len() {
                self.cursor_col -= 1;
                self.lines[self.cursor_line].remove(self.cursor_col);
            }
        } else if self.cursor_line > 0 {
            // 合并到上一行
            let current_line = self.lines.remove(self.cursor_line);
            self.cursor_line -= 1;
            self.cursor_col = self.lines[self.cursor_line].len();
            self.lines[self.cursor_line].push_str(&current_line);
        }
    }

    fn handle_enter(&mut self) {
        if self.cursor_line < self.lines.len() {
            let current_line = &self.lines[self.cursor_line];
            let right_part = current_line[self.cursor_col..].to_string();
            self.lines[self.cursor_line].truncate(self.cursor_col);
            self.lines.insert(self.cursor_line + 1, right_part);
            self.cursor_line += 1;
            self.cursor_col = 0;
        }
    }

    fn move_cursor_left(&mut self) {
        if self.cursor_col > 0 {
            self.cursor_col -= 1;
        } else if self.cursor_line > 0 {
            self.cursor_line -= 1;
            self.cursor_col = if self.cursor_line < self.lines.len() {
                self.lines[self.cursor_line].len()
            } else {
                0
            };
        }
    }

    fn move_cursor_right(&mut self) {
        if self.cursor_line < self.lines.len() {
            if self.cursor_col < self.lines[self.cursor_line].len() {
                self.cursor_col += 1;
            } else if self.cursor_line + 1 < self.lines.len() {
                self.cursor_line += 1;
                self.cursor_col = 0;
            }
        }
    }

    fn move_cursor_up(&mut self) {
        if self.cursor_line > 0 {
            self.cursor_line -= 1;
            if self.cursor_line < self.lines.len() {
                self.cursor_col = self.cursor_col.min(self.lines[self.cursor_line].len());
            }
        }
    }

    fn move_cursor_down(&mut self) {
        if self.cursor_line + 1 < self.lines.len() {
            self.cursor_line += 1;
            self.cursor_col = self.cursor_col.min(self.lines[self.cursor_line].len());
        }
    }
}

impl Component for CommitEditor {
    fn name(&self) -> &str {
        "CommitEditor"
    }

    fn render(&mut self, frame: &mut Frame, area: Rect, _state: &AppState) {
        let border_style = if self.focused {
            Style::default().fg(Color::Green)
        } else {
            Style::default().fg(Color::Gray)
        };

        let text_style = if self.focused {
            Style::default().fg(Color::White).bg(Color::DarkGray)
        } else {
            Style::default().fg(Color::Gray)
        };

        // 计算可见区域
        let inner_height = area.height.saturating_sub(2) as usize;
        let visible_lines = &self.lines[self.scroll_offset..
            (self.scroll_offset + inner_height).min(self.lines.len())];

        // 构建显示文本
        let mut text_lines = Vec::new();
        for line in visible_lines {
            text_lines.push(Line::from(Span::styled(line.clone(), text_style)));
        }

        let paragraph = Paragraph::new(text_lines)
            .block(Block::default()
                .borders(Borders::ALL)
                .title("Edit Commit Message")
                .border_style(border_style))
            .wrap(Wrap { trim: false });

        frame.render_widget(paragraph, area);

        // 渲染光标
        if self.focused && self.cursor_line >= self.scroll_offset && 
           self.cursor_line < self.scroll_offset + inner_height {
            let cursor_x = area.x + 1 + self.cursor_col as u16;
            let cursor_y = area.y + 1 + (self.cursor_line - self.scroll_offset) as u16;
            
            if cursor_x < area.x + area.width.saturating_sub(1) &&
               cursor_y < area.y + area.height.saturating_sub(1) {
                frame.set_cursor(cursor_x, cursor_y);
            }
        }
    }

    fn handle_key_event(&mut self, key: KeyEvent, _state: &mut AppState) -> EventResult {
        if !self.focused {
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
            KeyCode::Enter => {
                self.handle_enter();
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
            KeyCode::Up => {
                self.move_cursor_up();
                EventResult::Handled
            }
            KeyCode::Down => {
                self.move_cursor_down();
                EventResult::Handled
            }
            KeyCode::Home => {
                self.cursor_col = 0;
                EventResult::Handled
            }
            KeyCode::End => {
                if self.cursor_line < self.lines.len() {
                    self.cursor_col = self.lines[self.cursor_line].len();
                }
                EventResult::Handled
            }
            KeyCode::Tab => {
                // 退出编辑模式，由父组件处理
                EventResult::NotHandled
            }
            KeyCode::Esc => {
                // 取消编辑，由父组件处理
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
}