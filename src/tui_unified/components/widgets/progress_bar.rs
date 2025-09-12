use ratatui::{prelude::*, widgets::*};

pub struct ProgressBar {
    pub progress: f64,
    pub label: String,
}

impl ProgressBar {
    pub fn new(progress: f64, label: String) -> Self {
        Self { progress, label }
    }

    pub fn render(&self, area: Rect, buf: &mut Buffer) {
        let gauge = Gauge::default()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(self.label.as_str()),
            )
            .gauge_style(Style::default().fg(Color::Green))
            .ratio(self.progress);
        gauge.render(area, buf);
    }
}
