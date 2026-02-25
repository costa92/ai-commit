use super::types::{
    get_file_icon, DiffDisplayMode, DiffFile, DiffLine, DiffLineType, FileTreeNode,
};
use super::DiffViewerComponent;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Scrollbar, ScrollbarOrientation, ScrollbarState},
    Frame,
};

impl DiffViewerComponent {
    /// 获取diff行的样式
    pub(super) fn get_line_style(&self, line: &DiffLine, is_selected: bool) -> Style {
        let base_style = match line.line_type {
            DiffLineType::Added => Style::default().fg(Color::Green),
            DiffLineType::Removed => Style::default().fg(Color::Red),
            DiffLineType::Header => Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
            DiffLineType::Hunk => Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
            DiffLineType::Context => {
                // 特殊处理 "No newline at end of file" 行
                if line.content.starts_with("\\")
                    && line.content.contains("No newline at end of file")
                {
                    Style::default()
                        .fg(Color::Gray)
                        .add_modifier(Modifier::ITALIC)
                } else {
                    Style::default().fg(Color::White)
                }
            }
            DiffLineType::FileTree => Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
            DiffLineType::Binary => Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::ITALIC),
        };

        if is_selected && self.focused {
            base_style.bg(Color::DarkGray)
        } else {
            base_style
        }
    }

    /// 格式化显示行
    pub(super) fn format_line(&self, line: &DiffLine) -> String {
        // 特殊处理 "No newline at end of file" 行
        if line.content.starts_with("\\") && line.content.contains("No newline at end of file") {
            return "⚠ No newline at end of file".to_string();
        }

        if self.show_line_numbers {
            let old_no = line
                .old_line_no
                .map_or("    ".to_string(), |n| format!("{:4}", n));
            let new_no = line
                .new_line_no
                .map_or("    ".to_string(), |n| format!("{:4}", n));
            format!("{} {} | {}", old_no, new_no, line.content)
        } else {
            line.content.clone()
        }
    }

    /// 生成统一diff视图
    pub(super) fn generate_unified_view(&self, visible_height: usize) -> Vec<ListItem<'_>> {
        // 检查是否正在显示二进制文件详情
        if let Some(file_index) = self.selected_file {
            if let Some(file) = self.diff_files.get(file_index) {
                if file.is_binary && self.display_mode == DiffDisplayMode::Unified {
                    // 显示二进制文件对比视图
                    return self.generate_binary_comparison_view(file);
                }
            }
        }

        // 常规的文本diff视图
        self.diff_lines
            .iter()
            .skip(self.scroll_position)
            .take(visible_height)
            .enumerate()
            .map(|(i, line)| {
                let is_selected = self.selected_line == Some(self.scroll_position + i);

                if self.word_level_diff {
                    // 使用单词级高亮
                    let line_content = self.format_line(line);
                    let spans = self.apply_word_level_highlighting(&line_content, &line.line_type);

                    // 为选中行添加背景色
                    let final_spans = if is_selected && self.focused {
                        spans
                            .into_iter()
                            .map(|span| {
                                let mut new_style = span.style;
                                new_style.bg = Some(Color::DarkGray);
                                Span::styled(span.content, new_style)
                            })
                            .collect()
                    } else {
                        spans
                    };

                    ListItem::new(Line::from(final_spans))
                } else {
                    // 使用传统行级高亮
                    let style = self.get_line_style(line, is_selected);
                    let content = self.format_line(line);
                    ListItem::new(Line::from(Span::styled(content, style)))
                }
            })
            .collect()
    }

    /// 生成并排对比视图
    #[allow(dead_code)]
    pub(super) fn generate_side_by_side_view(
        &self,
        area_width: u16,
        visible_height: usize,
    ) -> Vec<ListItem<'_>> {
        let mut result = Vec::new();
        let half_width = (area_width.saturating_sub(4)) / 2;

        // 存储左右两侧的行数据
        let mut _left_lines: Vec<String> = Vec::new();
        let mut _right_lines: Vec<String> = Vec::new();

        // 处理可见范围的diff行
        let visible_lines: Vec<&DiffLine> = self
            .diff_lines
            .iter()
            .skip(self.scroll_position)
            .take(visible_height)
            .collect();

        // 按行配对处理
        for (i, line) in visible_lines.iter().enumerate() {
            let is_selected = self.selected_line == Some(self.scroll_position + i);

            match line.line_type {
                DiffLineType::Header | DiffLineType::Hunk => {
                    let style = self.get_line_style(line, is_selected);
                    let content =
                        self.truncate_content(&line.content, area_width.saturating_sub(2) as usize);
                    result.push(ListItem::new(Line::from(Span::styled(content, style))));
                }
                DiffLineType::Context => {
                    if line.content.starts_with("\\")
                        && line.content.contains("No newline at end of file")
                    {
                        let notice_style = if is_selected {
                            Style::default().fg(Color::Gray).bg(Color::DarkGray)
                        } else {
                            Style::default().fg(Color::Gray)
                        };

                        let notice_text = "⚠ No newline at end of file";
                        let centered_content = format!(
                            "{:^width$}",
                            notice_text,
                            width = area_width.saturating_sub(2) as usize
                        );
                        result.push(ListItem::new(Line::from(Span::styled(
                            centered_content,
                            notice_style,
                        ))));
                    } else {
                        let left_content = self.format_side_content(
                            &line.content,
                            line.old_line_no,
                            half_width as usize,
                            true,
                        );
                        let right_content = self.format_side_content(
                            &line.content,
                            line.new_line_no,
                            half_width as usize,
                            false,
                        );

                        let left_style = if is_selected {
                            Style::default().fg(Color::White).bg(Color::DarkGray)
                        } else {
                            Style::default().fg(Color::White)
                        };

                        result.push(ListItem::new(Line::from(vec![
                            Span::styled(left_content, left_style),
                            Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
                            Span::styled(right_content, left_style),
                        ])));
                    }
                }
                DiffLineType::Added => {
                    let left_content = " ".repeat(half_width as usize);
                    let right_content = self.format_side_content(
                        &line.content,
                        line.new_line_no,
                        half_width as usize,
                        false,
                    );

                    if self.word_level_diff {
                        let right_spans =
                            self.apply_word_level_highlighting(&right_content, &line.line_type);
                        let mut spans = vec![
                            Span::styled(left_content, Style::default()),
                            Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
                        ];
                        spans.extend(right_spans);
                        result.push(ListItem::new(Line::from(spans)));
                    } else {
                        let right_style = if is_selected {
                            Style::default().fg(Color::Green).bg(Color::DarkGray)
                        } else {
                            Style::default().fg(Color::Green)
                        };

                        result.push(ListItem::new(Line::from(vec![
                            Span::styled(left_content, Style::default()),
                            Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
                            Span::styled(right_content, right_style),
                        ])));
                    }
                }
                DiffLineType::Removed => {
                    let left_content = self.format_side_content(
                        &line.content,
                        line.old_line_no,
                        half_width as usize,
                        true,
                    );
                    let right_content = " ".repeat(half_width as usize);

                    if self.word_level_diff {
                        let left_spans =
                            self.apply_word_level_highlighting(&left_content, &line.line_type);
                        let mut spans = vec![];
                        spans.extend(left_spans);
                        spans.extend(vec![
                            Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
                            Span::styled(right_content, Style::default()),
                        ]);
                        result.push(ListItem::new(Line::from(spans)));
                    } else {
                        let left_style = if is_selected {
                            Style::default().fg(Color::Red).bg(Color::DarkGray)
                        } else {
                            Style::default().fg(Color::Red)
                        };

                        result.push(ListItem::new(Line::from(vec![
                            Span::styled(left_content, left_style),
                            Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
                            Span::styled(right_content, Style::default()),
                        ])));
                    }
                }
                DiffLineType::Binary => {
                    let style = self.get_line_style(line, is_selected);
                    let content =
                        self.truncate_content(&line.content, area_width.saturating_sub(2) as usize);
                    result.push(ListItem::new(Line::from(Span::styled(content, style))));
                }
                DiffLineType::FileTree => {
                    // 文件树信息不应出现在并排模式中
                }
            }
        }

        result
    }

    /// 生成图片/二进制文件对比信息
    pub(super) fn generate_binary_comparison_view(&self, file: &DiffFile) -> Vec<ListItem<'_>> {
        let mut items = Vec::new();

        items.push(ListItem::new(Line::from(vec![
            Span::styled("📦 ", Style::default().fg(Color::Magenta)),
            Span::styled(
                format!("Binary File: {}", file.path),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
        ])));

        items.push(ListItem::new(Line::from(Span::raw(""))));

        if file.is_image {
            items.push(ListItem::new(Line::from(vec![
                Span::styled("🖼️  ", Style::default().fg(Color::Yellow)),
                Span::styled(
                    "Image File Detected",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
            ])));

            items.push(ListItem::new(Line::from(vec![
                Span::styled("   Type: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    self.get_file_extension(&file.path)
                        .unwrap_or_else(|| "Unknown".to_string()),
                    Style::default().fg(Color::White),
                ),
            ])));

            items.push(ListItem::new(Line::from(Span::raw(""))));

            items.push(ListItem::new(Line::from(vec![
                Span::styled("   📏 ", Style::default().fg(Color::Blue)),
                Span::styled(
                    "Image comparison not available in terminal",
                    Style::default().fg(Color::Gray),
                ),
            ])));

            items.push(ListItem::new(Line::from(vec![
                Span::styled("   💡 ", Style::default().fg(Color::Yellow)),
                Span::styled(
                    "Tip: Use external image diff tools for visual comparison",
                    Style::default().fg(Color::Gray),
                ),
            ])));
        } else {
            items.push(ListItem::new(Line::from(vec![
                Span::styled("📦  ", Style::default().fg(Color::Magenta)),
                Span::styled(
                    "Binary File",
                    Style::default()
                        .fg(Color::Magenta)
                        .add_modifier(Modifier::BOLD),
                ),
            ])));

            items.push(ListItem::new(Line::from(vec![
                Span::styled("   Extension: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    self.get_file_extension(&file.path)
                        .unwrap_or_else(|| "None".to_string()),
                    Style::default().fg(Color::White),
                ),
            ])));
        }

        items.push(ListItem::new(Line::from(Span::raw(""))));

        if file.additions > 0 || file.deletions > 0 {
            items.push(ListItem::new(Line::from(vec![Span::styled(
                "📊 Changes:",
                Style::default()
                    .fg(Color::Blue)
                    .add_modifier(Modifier::BOLD),
            )])));

            if file.additions > 0 {
                items.push(ListItem::new(Line::from(vec![
                    Span::styled("   +", Style::default().fg(Color::Green)),
                    Span::styled(
                        format!("{} additions", file.additions),
                        Style::default().fg(Color::Green),
                    ),
                ])));
            }

            if file.deletions > 0 {
                items.push(ListItem::new(Line::from(vec![
                    Span::styled("   -", Style::default().fg(Color::Red)),
                    Span::styled(
                        format!("{} deletions", file.deletions),
                        Style::default().fg(Color::Red),
                    ),
                ])));
            }
        } else {
            items.push(ListItem::new(Line::from(vec![
                Span::styled("ℹ️  ", Style::default().fg(Color::Blue)),
                Span::styled(
                    "File modified (binary diff cannot be displayed)",
                    Style::default().fg(Color::Gray),
                ),
            ])));
        }

        items.push(ListItem::new(Line::from(Span::raw(""))));

        items.push(ListItem::new(Line::from(vec![Span::styled(
            "⌨️  Controls:",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )])));

        items.push(ListItem::new(Line::from(vec![
            Span::styled("   • ", Style::default().fg(Color::Gray)),
            Span::styled(
                "ESC",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" - Return to file tree", Style::default().fg(Color::Gray)),
        ])));

        items.push(ListItem::new(Line::from(vec![
            Span::styled("   • ", Style::default().fg(Color::Gray)),
            Span::styled(
                "1/2/3",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" - Switch view modes", Style::default().fg(Color::Gray)),
        ])));

        items
    }

    /// 生成文件树显示内容
    pub(super) fn generate_file_tree_view(&self) -> Vec<ListItem<'_>> {
        let mut items = Vec::new();

        items.push(ListItem::new(Line::from(vec![
            Span::styled("📊 ", Style::default().fg(Color::Blue)),
            Span::styled(
                format!("Diff Summary: {} files", self.diff_files.len()),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
        ])));

        items.push(ListItem::new(Line::from(vec![
            Span::styled("  +", Style::default().fg(Color::Green)),
            Span::styled(
                format!("{} additions", self.total_additions),
                Style::default().fg(Color::Green),
            ),
            Span::styled("  -", Style::default().fg(Color::Red)),
            Span::styled(
                format!("{} deletions", self.total_deletions),
                Style::default().fg(Color::Red),
            ),
        ])));

        items.push(ListItem::new(Line::from(Span::raw(""))));

        let mut file_tree = std::collections::BTreeMap::new();
        for (i, file) in self.diff_files.iter().enumerate() {
            let path_parts: Vec<&str> = file.path.split('/').collect();
            self.insert_file_into_tree(&mut file_tree, &path_parts, i);
        }

        self.render_tree_node(&file_tree, 0, &mut items);

        items
    }

    /// 递归渲染文件树节点
    pub(super) fn render_tree_node(
        &self,
        tree: &std::collections::BTreeMap<String, FileTreeNode>,
        depth: usize,
        items: &mut Vec<ListItem>,
    ) {
        for (name, node) in tree {
            let indent = "  ".repeat(depth);

            match node {
                FileTreeNode::Directory(subtree) => {
                    let icon = if subtree.is_empty() { "📁 " } else { "📂 " };
                    items.push(ListItem::new(Line::from(vec![
                        Span::raw(indent),
                        Span::styled(icon, Style::default().fg(Color::Blue)),
                        Span::styled(
                            name.clone(),
                            Style::default()
                                .fg(Color::Blue)
                                .add_modifier(Modifier::BOLD),
                        ),
                    ])));

                    self.render_tree_node(subtree, depth + 1, items);
                }
                FileTreeNode::File(file_index) => {
                    if let Some(file) = self.diff_files.get(*file_index) {
                        let icon = if file.is_binary {
                            if file.is_image {
                                "🖼️ "
                            } else {
                                "📦 "
                            }
                        } else {
                            get_file_icon(&file.path).unwrap_or("📄 ")
                        };

                        let (status_color, status_text) = if file.is_binary {
                            (Color::Magenta, " (binary)".to_string())
                        } else {
                            match (file.additions, file.deletions) {
                                (0, 0) => (Color::Gray, "".to_string()),
                                (a, 0) => (Color::Green, format!(" (+{})", a)),
                                (0, d) => (Color::Red, format!(" (-{})", d)),
                                (a, d) => (Color::Yellow, format!(" (+{}, -{})", a, d)),
                            }
                        };

                        let is_selected = self.selected_file == Some(*file_index);
                        let file_style = if is_selected && self.focused {
                            Style::default().fg(Color::White).bg(Color::DarkGray)
                        } else {
                            Style::default().fg(Color::White)
                        };

                        items.push(ListItem::new(Line::from(vec![
                            Span::raw(indent),
                            Span::styled(icon, Style::default().fg(Color::Yellow)),
                            Span::styled(name.clone(), file_style),
                            Span::styled(status_text, Style::default().fg(status_color)),
                        ])));
                    }
                }
            }
        }
    }

    /// 将文件插入目录树
    #[allow(clippy::only_used_in_recursion)]
    pub(super) fn insert_file_into_tree(
        &self,
        tree: &mut std::collections::BTreeMap<String, FileTreeNode>,
        path_parts: &[&str],
        file_index: usize,
    ) {
        if path_parts.is_empty() {
            return;
        }

        let part = path_parts[0].to_string();
        let is_file = path_parts.len() == 1;

        if is_file {
            tree.insert(part, FileTreeNode::File(file_index));
        } else {
            let entry = tree
                .entry(part)
                .or_insert_with(|| FileTreeNode::Directory(std::collections::BTreeMap::new()));

            if let FileTreeNode::Directory(ref mut subtree) = entry {
                self.insert_file_into_tree(subtree, &path_parts[1..], file_index);
            }
        }
    }

    /// 渲染组件的主方法（由 Component::render 调用）
    pub(super) fn render_component(&mut self, frame: &mut Frame, area: Rect) {
        let border_style = if self.focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::White)
        };

        let mut title_parts = vec![];
        let mode_indicator = match self.display_mode {
            DiffDisplayMode::Unified => "📄 Unified Diff",
            DiffDisplayMode::SideBySide => "⚖️ Side-by-Side Diff",
            DiffDisplayMode::FileTree => "🌳 File Tree Diff",
        };
        title_parts.push(mode_indicator.to_string());

        let mut features = vec![];
        if self.word_level_diff {
            features.push("🔍Word");
        }
        if self.show_line_numbers {
            features.push("📊Line#");
        }
        if !features.is_empty() {
            title_parts.push(format!(" [{}]", features.join(" ")));
        }

        if let Some(ref file) = self.current_file {
            title_parts.push(format!(" - {}", file));
        }
        if let Some(ref commit) = self.current_commit {
            title_parts.push(format!(" ({})", &commit[..8.min(commit.len())]));
        }
        let title = title_parts.join("");

        let visible_lines = match self.display_mode {
            DiffDisplayMode::FileTree => self.generate_file_tree_view(),
            DiffDisplayMode::SideBySide => {
                self.render_three_column_layout(frame, area, &title);
                return;
            }
            DiffDisplayMode::Unified => {
                self.generate_unified_view(area.height.saturating_sub(2) as usize)
            }
        };

        let list = List::new(visible_lines).block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(border_style),
        );

        frame.render_widget(list, area);

        let content_len = match self.display_mode {
            DiffDisplayMode::FileTree => self.diff_files.len() + 3,
            _ => self.diff_lines.len(),
        };

        let visible_height = area.height.saturating_sub(2) as usize;
        if content_len > visible_height {
            let scrollbar_area = Rect {
                x: area.x + area.width - 1,
                y: area.y + 1,
                width: 1,
                height: area.height - 2,
            };

            let mut scrollbar_state = ScrollbarState::default()
                .content_length(content_len)
                .viewport_content_length(visible_height)
                .position(self.scroll_position);

            frame.render_stateful_widget(
                Scrollbar::default()
                    .orientation(ScrollbarOrientation::VerticalRight)
                    .begin_symbol(Some("↑"))
                    .end_symbol(Some("↓")),
                scrollbar_area,
                &mut scrollbar_state,
            );
        }
    }
}
