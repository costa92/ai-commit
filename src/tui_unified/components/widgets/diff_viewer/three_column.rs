use super::types::{DiffFile, DiffLineType};
use super::DiffViewerComponent;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Text,
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

impl DiffViewerComponent {
    /// 渲染三列布局：文件列表、旧内容、新内容
    pub(super) fn render_three_column_layout(
        &mut self,
        frame: &mut Frame,
        area: Rect,
        _title: &str,
    ) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(30),
                Constraint::Percentage(35),
                Constraint::Percentage(35),
            ])
            .split(area);

        self.render_file_list(frame, chunks[0]);
        self.render_old_file_content(frame, chunks[1]);
        self.render_new_file_content(frame, chunks[2]);
    }

    /// 渲染文件列表
    fn render_file_list(&mut self, frame: &mut Frame, area: Rect) {
        let border_style = if self.focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::White)
        };

        self.file_list_state.select(self.selected_file);

        let file_items: Vec<ListItem> = self
            .diff_files
            .iter()
            .map(|file| {
                let status_icon = if file.additions > 0 && file.deletions > 0 {
                    "📝"
                } else if file.additions > 0 {
                    "📄"
                } else if file.deletions > 0 {
                    "🗑️"
                } else {
                    "📄"
                };

                let display_name = Self::safe_truncate_path(&file.path, 25);
                let content = format!("{} {}", status_icon, display_name);
                ListItem::new(Text::raw(content))
            })
            .collect();

        let title = if let Some(selected) = self.selected_file {
            format!("📁 Files ({}/{})", selected + 1, self.diff_files.len())
        } else {
            "📁 Files".to_string()
        };

        let file_list = List::new(file_items)
            .block(
                Block::default()
                    .title(title)
                    .borders(Borders::ALL)
                    .border_style(border_style),
            )
            .highlight_style(
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            );

        frame.render_stateful_widget(file_list, area, &mut self.file_list_state);
    }

    /// 渲染旧文件内容
    fn render_old_file_content(&self, frame: &mut Frame, area: Rect) {
        let border_style = Style::default().fg(Color::Red);

        let old_content = self.get_old_file_content();
        let old_lines: Vec<ListItem> = old_content
            .into_iter()
            .map(|line| ListItem::new(Text::raw(line)))
            .collect();

        let title = if let Some(file_index) = self.selected_file {
            if let Some(file) = self.diff_files.get(file_index) {
                let display_path = Self::safe_truncate_path(&file.path, 40);
                format!("🔻 Old (-): {}", display_path)
            } else {
                "🔻 Old (-)".to_string()
            }
        } else {
            "🔻 Old (-)".to_string()
        };

        let old_list = List::new(old_lines).block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(border_style),
        );

        frame.render_widget(old_list, area);
    }

    /// 渲染新文件内容
    fn render_new_file_content(&self, frame: &mut Frame, area: Rect) {
        let border_style = Style::default().fg(Color::Green);

        let new_content = self.get_new_file_content();
        let new_lines: Vec<ListItem> = new_content
            .into_iter()
            .map(|line| ListItem::new(Text::raw(line)))
            .collect();

        let title = if let Some(file_index) = self.selected_file {
            if let Some(file) = self.diff_files.get(file_index) {
                let display_path = Self::safe_truncate_path(&file.path, 40);
                format!("🔺 New (+): {}", display_path)
            } else {
                "🔺 New (+)".to_string()
            }
        } else {
            "🔺 New (+)".to_string()
        };

        let new_list = List::new(new_lines).block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(border_style),
        );

        frame.render_widget(new_list, area);
    }

    /// 获取当前选中文件的旧内容
    fn get_old_file_content(&self) -> Vec<String> {
        if let Some(file_index) = self.selected_file {
            if let Some(file) = self.diff_files.get(file_index) {
                return self.extract_file_old_lines(file);
            }
        }

        vec![
            "".to_string(),
            "  Select a file from the".to_string(),
            "  file list to view its".to_string(),
            "  old content here.".to_string(),
            "".to_string(),
        ]
    }

    /// 获取当前选中文件的新内容
    fn get_new_file_content(&self) -> Vec<String> {
        if let Some(file_index) = self.selected_file {
            if let Some(file) = self.diff_files.get(file_index) {
                return self.extract_file_new_lines(file);
            }
        }

        vec![
            "".to_string(),
            "  Select a file from the".to_string(),
            "  file list to view its".to_string(),
            "  new content here.".to_string(),
            "".to_string(),
        ]
    }

    /// 从diff文件中提取旧内容行
    fn extract_file_old_lines(&self, file: &DiffFile) -> Vec<String> {
        let mut old_lines = Vec::new();

        for line in &file.lines {
            match line.line_type {
                DiffLineType::Context | DiffLineType::Removed => {
                    let content = if let Some(s) = line
                        .content
                        .strip_prefix(' ')
                        .or_else(|| line.content.strip_prefix('-'))
                    {
                        s.to_string()
                    } else {
                        line.content.clone()
                    };
                    old_lines.push(content);
                }
                _ => {}
            }
        }

        if old_lines.is_empty() {
            old_lines.push(format!("DEBUG - File: {}", file.path));
            old_lines.push(format!("Total lines in file: {}", file.lines.len()));
            old_lines.push(format!("Selected file index: {:?}", self.selected_file));
            old_lines.push("Line types and content:".to_string());
            for (i, line) in file.lines.iter().enumerate() {
                if i < 5 {
                    old_lines.push(format!(
                        "  {}: {:?} - {}",
                        i,
                        line.line_type,
                        Self::safe_truncate_content(&line.content, 50)
                    ));
                }
            }
            if file.lines.len() > 5 {
                old_lines.push(format!("  ... and {} more", file.lines.len() - 5));
            }

            let mut type_counts = std::collections::HashMap::new();
            for line in &file.lines {
                *type_counts
                    .entry(format!("{:?}", line.line_type))
                    .or_insert(0) += 1;
            }
            old_lines.push("Type counts:".to_string());
            for (line_type, count) in type_counts {
                old_lines.push(format!("  {}: {}", line_type, count));
            }
        }

        old_lines
    }

    /// 从diff文件中提取新内容行
    fn extract_file_new_lines(&self, file: &DiffFile) -> Vec<String> {
        let mut new_lines = Vec::new();

        for line in &file.lines {
            match line.line_type {
                DiffLineType::Context | DiffLineType::Added => {
                    let content = if let Some(s) = line
                        .content
                        .strip_prefix(' ')
                        .or_else(|| line.content.strip_prefix('+'))
                    {
                        s.to_string()
                    } else {
                        line.content.clone()
                    };
                    new_lines.push(content);
                }
                _ => {}
            }
        }

        if new_lines.is_empty() {
            new_lines.push(format!("DEBUG - File: {}", file.path));
            new_lines.push(format!("Total lines in file: {}", file.lines.len()));
            new_lines.push(format!("Selected file index: {:?}", self.selected_file));
            new_lines.push("Line types and content:".to_string());
            for (i, line) in file.lines.iter().enumerate() {
                if i < 5 {
                    new_lines.push(format!(
                        "  {}: {:?} - {}",
                        i,
                        line.line_type,
                        Self::safe_truncate_content(&line.content, 50)
                    ));
                }
            }
            if file.lines.len() > 5 {
                new_lines.push(format!("  ... and {} more", file.lines.len() - 5));
            }

            let mut type_counts = std::collections::HashMap::new();
            for line in &file.lines {
                *type_counts
                    .entry(format!("{:?}", line.line_type))
                    .or_insert(0) += 1;
            }
            new_lines.push("Type counts:".to_string());
            for (line_type, count) in type_counts {
                new_lines.push(format!("  {}: {}", line_type, count));
            }
        }

        new_lines
    }
}
