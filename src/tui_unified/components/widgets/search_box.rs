use ratatui::{prelude::*, widgets::*};

pub struct SearchBox {
    pub input: String,
    pub cursor_position: usize,
    pub is_focused: bool,
}

impl SearchBox {
    pub fn new() -> Self {
        Self {
            input: String::new(),
            cursor_position: 0,
            is_focused: false,
        }
    }
    
    pub fn handle_input(&mut self, c: char) {
        self.input.insert(self.cursor_position, c);
        self.cursor_position += 1;
    }
    
    pub fn render(&self, area: Rect, buf: &mut Buffer) {
        let input_style = if self.is_focused {
            Style::default().bg(Color::Blue)
        } else {
            Style::default()
        };
        
        let input = Paragraph::new(self.input.as_str())
            .style(input_style)
            .block(Block::default().borders(Borders::ALL).title("Search"));
        input.render(area, buf);
    }
}
