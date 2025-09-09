use ratatui::{prelude::*, widgets::*};

pub struct StatusBar {
    pub current_branch: String,
    pub mode: String,
    pub message: String,
}

impl StatusBar {
    pub fn new() -> Self {
        Self {
            current_branch: "main".to_string(),
            mode: "Normal".to_string(),
            message: "Ready".to_string(),
        }
    }
    
    pub fn update_message(&mut self, message: String) {
        self.message = message;
    }
    
    pub fn update_branch(&mut self, branch: String) {
        self.current_branch = branch;
    }
    
    pub fn render(&self, area: Rect, buf: &mut Buffer) {
        let status_text = format!(
            "Branch: {} | Mode: {} | {}",
            self.current_branch, self.mode, self.message
        );
        
        let status = Paragraph::new(status_text)
            .style(Style::default().bg(Color::Blue).fg(Color::White))
            .alignment(Alignment::Left);
        status.render(area, buf);
    }
}