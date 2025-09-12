use ratatui::{prelude::*, widgets::*};

pub struct HelpPanel {
    pub shortcuts: Vec<(String, String)>,
    pub visible: bool,
}

impl Default for HelpPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl HelpPanel {
    pub fn new() -> Self {
        let shortcuts = vec![
            ("Tab".to_string(), "Switch panel".to_string()),
            ("q".to_string(), "Quit".to_string()),
            ("?".to_string(), "Toggle help".to_string()),
            ("Enter".to_string(), "Select item".to_string()),
        ];

        Self {
            shortcuts,
            visible: false,
        }
    }

    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }

    pub fn render(&self, area: Rect, buf: &mut Buffer) {
        if !self.visible {
            return;
        }

        let text: Vec<Line> = self
            .shortcuts
            .iter()
            .map(|(key, desc)| {
                Line::from(vec![
                    Span::styled(format!("{:<10}", key), Style::default().fg(Color::Yellow)),
                    Span::raw(" - "),
                    Span::raw(desc.clone()),
                ])
            })
            .collect();

        let help = Paragraph::new(text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Help (Press ? to toggle)"),
            )
            .wrap(Wrap { trim: true });
        help.render(area, buf);
    }
}
