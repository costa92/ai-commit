use crossterm::event::KeyEvent;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use crate::diff_viewer::{DiffViewMode, DiffViewer};
use crate::tui_unified::components::base::component::Component;
use crate::tui_unified::Result;

/// 缓存 diff 解析结果，避免每帧重新计算
pub(crate) struct DiffRenderCache {
    content_hash: u64,
    view_mode: DiffViewMode,
    unified: Option<Vec<ratatui::text::Line<'static>>>,
    side_by_side: Option<(
        Vec<ratatui::text::Line<'static>>,
        Vec<ratatui::text::Line<'static>>,
    )>,
    split: Option<(
        Vec<ratatui::text::Line<'static>>,
        Vec<ratatui::text::Line<'static>>,
    )>,
}

impl DiffRenderCache {
    pub fn new() -> Self {
        Self {
            content_hash: 0,
            view_mode: DiffViewMode::Unified,
            unified: None,
            side_by_side: None,
            split: None,
        }
    }

    fn hash_content(content: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        hasher.finish()
    }
}

impl super::app::TuiUnifiedApp {
    /// 预填充 diff 渲染缓存（在渲染前调用，避免每帧重新解析）
    fn ensure_diff_cache(&mut self) {
        let (content_hash, view_mode) = match &self.diff_viewer {
            Some(v) if !v.current_diff.is_empty() => {
                let hash = DiffRenderCache::hash_content(&v.current_diff);
                let mode = v.view_mode.clone();
                (hash, mode)
            }
            _ => return,
        };

        // 内容变化时清除所有模式的缓存
        if content_hash != self.diff_render_cache.content_hash {
            self.diff_render_cache.content_hash = content_hash;
            self.diff_render_cache.unified = None;
            self.diff_render_cache.side_by_side = None;
            self.diff_render_cache.split = None;
        }

        self.diff_render_cache.view_mode = view_mode.clone();

        // 仅当前模式缓存缺失时才重新解析
        let need_parse = match view_mode {
            DiffViewMode::Unified => self.diff_render_cache.unified.is_none(),
            DiffViewMode::SideBySide => self.diff_render_cache.side_by_side.is_none(),
            DiffViewMode::Split => self.diff_render_cache.split.is_none(),
        };

        if !need_parse {
            return;
        }

        // Clone diff content for parsing (only on cache miss)
        let diff_content = self.diff_viewer.as_ref().unwrap().current_diff.clone();

        match view_mode {
            DiffViewMode::Unified => {
                let lines = self.parse_diff_for_unified(&diff_content);
                self.diff_render_cache.unified = Some(lines);
            }
            DiffViewMode::SideBySide => {
                let (left, right) = self.parse_diff_for_side_by_side(&diff_content);
                self.diff_render_cache.side_by_side = Some((left, right));
            }
            DiffViewMode::Split => {
                let (removed, added) = self.parse_diff_for_split(&diff_content);
                self.diff_render_cache.split = Some((removed, added));
            }
        }
    }

    /// 清除模态框背景，确保不会有底层内容泄露
    fn clear_modal_background(&self, frame: &mut ratatui::Frame, area: ratatui::layout::Rect) {
        use ratatui::style::{Color, Style};
        use ratatui::widgets::Clear;

        // 清除整个屏幕区域（重置所有 cell）
        frame.render_widget(Clear, area);

        // 逐行填充黑色背景，确保每个 cell 都有明确的 bg(Black)
        let bg_style = Style::default().bg(Color::Black).fg(Color::Black);
        let buf = frame.buffer_mut();
        for y in area.top()..area.bottom() {
            for x in area.left()..area.right() {
                let cell = buf.get_mut(x, y);
                cell.set_char(' ');
                cell.set_style(bg_style);
            }
        }
    }

    /// 在指定区域内渲染diff viewer，而不是全屏渲染
    fn render_diff_viewer_in_area(
        &self,
        frame: &mut ratatui::Frame,
        viewer: &DiffViewer,
        area: ratatui::layout::Rect,
    ) {
        use ratatui::{
            layout::{Constraint, Direction, Layout},
            style::{Color, Style},
            text::Text,
            widgets::{Block, Borders, Paragraph},
        };

        // 主布局：顶部信息栏 + 内容区 + 底部状态栏
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // 顶部信息
                Constraint::Min(0),    // 内容区
                Constraint::Length(4), // 状态栏 (增加高度以显示更多信息)
            ])
            .split(area);

        // 渲染顶部信息
        let commit_info_text = format!(
            "Commit: {} | Files: {} | Mode: {}",
            viewer.commit_info.hash.get(0..8).unwrap_or("unknown"),
            viewer.files.len(),
            match viewer.view_mode {
                crate::diff_viewer::DiffViewMode::Unified => "Unified (1)",
                crate::diff_viewer::DiffViewMode::SideBySide => "Side-by-Side (2)",
                crate::diff_viewer::DiffViewMode::Split => "Split (3)",
            }
        );
        let info_paragraph = Paragraph::new(Text::from(commit_info_text))
            .block(Block::default().borders(Borders::ALL).title("Commit Info"))
            .style(Style::default().fg(Color::White).bg(Color::Black));
        frame.render_widget(info_paragraph, main_chunks[0]);

        // 内容区：根据视图模式渲染不同的diff显示
        self.render_diff_content_by_mode(frame, viewer, main_chunks[1]);

        // 状态栏 - 添加视图切换说明
        let status_text = format!(
            "File {}/{} | Scroll: {} | View Mode: {} | Keys: 1-Unified 2-Side-by-Side 3-Split q-Close",
            viewer.selected_file + 1,
            viewer.files.len().max(1),
            viewer.diff_scroll,
            match viewer.view_mode {
                crate::diff_viewer::DiffViewMode::Unified => "Unified",
                crate::diff_viewer::DiffViewMode::SideBySide => "Side-by-Side",
                crate::diff_viewer::DiffViewMode::Split => "Split",
            }
        );
        let status_paragraph = Paragraph::new(Text::from(status_text))
            .block(Block::default().borders(Borders::ALL).title("Controls"))
            .style(Style::default().fg(Color::Yellow).bg(Color::Black));
        frame.render_widget(status_paragraph, main_chunks[2]);
    }

    fn render_diff_content_by_mode(
        &self,
        frame: &mut ratatui::Frame,
        viewer: &DiffViewer,
        area: ratatui::layout::Rect,
    ) {
        use ratatui::{
            layout::{Constraint, Direction, Layout},
            style::{Color, Style},
            widgets::{Block, Borders, Paragraph},
        };

        // 获取当前文件名，用于显示在标题中
        let current_file_name = if !viewer.files.is_empty() {
            let file = &viewer.files[viewer.selected_file];
            let char_count = file.path.chars().count();
            if char_count > 35 {
                let suffix: String = file.path.chars().skip(char_count - 32).collect();
                format!("...{}", suffix)
            } else {
                file.path.clone()
            }
        } else {
            "Unknown".to_string()
        };

        match viewer.view_mode {
            crate::diff_viewer::DiffViewMode::Unified => {
                // 优先使用缓存，否则重新解析（仅在缓存未命中时 clone diff_content）
                let lines = if let Some(ref cached) = self.diff_render_cache.unified {
                    cached.clone()
                } else {
                    let diff_content = if !viewer.current_diff.is_empty() {
                        viewer.current_diff.clone()
                    } else {
                        "No diff content available".to_string()
                    };
                    self.parse_diff_for_unified(&diff_content)
                };

                let diff_paragraph = Paragraph::new(lines)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title(format!("📄 Unified Diff: {}", current_file_name)),
                    )
                    .style(Style::default().fg(Color::White).bg(Color::Black))
                    .scroll((viewer.diff_scroll, 0))
                    .wrap(ratatui::widgets::Wrap { trim: false });
                frame.render_widget(diff_paragraph, area);
            }
            crate::diff_viewer::DiffViewMode::SideBySide => {
                let horizontal_chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                    .split(area);

                // 优先使用缓存，否则重新解析（仅在缓存未命中时 clone diff_content）
                let (left_lines, right_lines) =
                    if let Some(ref cached) = self.diff_render_cache.side_by_side {
                        cached.clone()
                    } else {
                        let diff_content = if !viewer.current_diff.is_empty() {
                            viewer.current_diff.clone()
                        } else {
                            "No diff content available".to_string()
                        };
                        self.parse_diff_for_side_by_side(&diff_content)
                    };

                let left_paragraph = Paragraph::new(left_lines)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title(format!("🔻 Original: {}", current_file_name)),
                    )
                    .style(Style::default().fg(Color::White).bg(Color::Black))
                    .scroll((viewer.diff_scroll, 0))
                    .wrap(ratatui::widgets::Wrap { trim: false });
                frame.render_widget(left_paragraph, horizontal_chunks[0]);

                let right_paragraph = Paragraph::new(right_lines)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title(format!("🔺 Modified: {}", current_file_name)),
                    )
                    .style(Style::default().fg(Color::White).bg(Color::Black))
                    .scroll((viewer.diff_scroll, 0))
                    .wrap(ratatui::widgets::Wrap { trim: false });
                frame.render_widget(right_paragraph, horizontal_chunks[1]);
            }
            crate::diff_viewer::DiffViewMode::Split => {
                let vertical_chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                    .split(area);

                // 优先使用缓存，否则重新解析（仅在缓存未命中时 clone diff_content）
                let (removed_lines, added_lines) =
                    if let Some(ref cached) = self.diff_render_cache.split {
                        cached.clone()
                    } else {
                        let diff_content = if !viewer.current_diff.is_empty() {
                            viewer.current_diff.clone()
                        } else {
                            "No diff content available".to_string()
                        };
                        self.parse_diff_for_split(&diff_content)
                    };

                let top_paragraph = Paragraph::new(removed_lines)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title(format!("🗑️ Removed (-): {}", current_file_name)),
                    )
                    .style(Style::default().fg(Color::White).bg(Color::Black))
                    .scroll((viewer.diff_scroll, 0))
                    .wrap(ratatui::widgets::Wrap { trim: false });
                frame.render_widget(top_paragraph, vertical_chunks[0]);

                let bottom_paragraph = Paragraph::new(added_lines)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title(format!("➕ Added (+): {}", current_file_name)),
                    )
                    .style(Style::default().fg(Color::White).bg(Color::Black))
                    .scroll((viewer.diff_scroll, 0))
                    .wrap(ratatui::widgets::Wrap { trim: false });
                frame.render_widget(bottom_paragraph, vertical_chunks[1]);
            }
        }
    }

    /// 解析 diff 内容用于并排显示
    fn parse_diff_for_side_by_side(
        &self,
        diff_content: &str,
    ) -> (
        Vec<ratatui::text::Line<'static>>,
        Vec<ratatui::text::Line<'static>>,
    ) {
        use ratatui::{
            style::{Color, Style},
            text::{Line, Span},
        };

        let mut left_lines = Vec::new();
        let mut right_lines = Vec::new();
        let mut old_line_num = 0u32;
        let mut new_line_num = 0u32;

        // 收集所有行并按块进行处理
        let lines: Vec<&str> = diff_content.lines().collect();
        let mut i = 0;
        let mut in_diff = false;

        while i < lines.len() {
            let line = lines[i];

            // 跳过 diff --git 之前的 commit metadata（Author, Date, message 等）
            if line.starts_with("diff --git") {
                in_diff = true;
            }
            if !in_diff {
                i += 1;
                continue;
            }

            if line.starts_with("@@") {
                // 解析行号信息：@@ -old_start,old_count +new_start,new_count @@ [optional context]
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 {
                    if let Some(old_part) = parts[1].strip_prefix('-') {
                        if let Some((start, _)) = old_part.split_once(',') {
                            old_line_num = start.parse().unwrap_or(0);
                        } else {
                            old_line_num = old_part.parse().unwrap_or(0);
                        }
                    }
                    if let Some(new_part) = parts[2].strip_prefix('+') {
                        if let Some((start, _)) = new_part.split_once(',') {
                            new_line_num = start.parse().unwrap_or(0);
                        } else {
                            new_line_num = new_part.parse().unwrap_or(0);
                        }
                    }
                }

                let header_line = Line::from(Span::styled(
                    line.to_string(),
                    Style::default().fg(Color::Cyan),
                ));
                left_lines.push(header_line.clone());
                right_lines.push(header_line);
                i += 1;
                continue;
            }

            if line.starts_with("diff --git")
                || line.starts_with("index")
                || line.starts_with("---")
                || line.starts_with("+++")
            {
                i += 1;
                continue;
            }

            if line.starts_with('-') {
                // 收集连续的删除行
                let mut removed_lines = Vec::new();
                while i < lines.len() && lines[i].starts_with('-') {
                    removed_lines.push(lines[i]);
                    i += 1;
                }

                // 收集后续的添加行
                let mut added_lines = Vec::new();
                while i < lines.len() && lines[i].starts_with('+') {
                    added_lines.push(lines[i]);
                    i += 1;
                }

                // 处理删除和添加行的对齐
                let max_lines = removed_lines.len().max(added_lines.len());

                for j in 0..max_lines {
                    if j < removed_lines.len() {
                        // 有删除行，在左侧显示
                        let line_content = removed_lines[j]
                            .strip_prefix('-')
                            .unwrap_or(removed_lines[j]);
                        let formatted_line =
                            format!("{:4} │ {}", old_line_num + j as u32, line_content);
                        left_lines.push(Line::from(Span::styled(
                            formatted_line.to_string(),
                            Style::default().fg(Color::Red),
                        )));
                    } else {
                        // 没有删除行，左侧显示空行
                        left_lines.push(Line::from(Span::styled(
                            "     │".to_string(),
                            Style::default().fg(Color::DarkGray),
                        )));
                    }

                    if j < added_lines.len() {
                        // 有添加行，在右侧显示
                        let line_content =
                            added_lines[j].strip_prefix('+').unwrap_or(added_lines[j]);
                        let formatted_line =
                            format!("{:4} │ {}", new_line_num + j as u32, line_content);
                        right_lines.push(Line::from(Span::styled(
                            formatted_line.to_string(),
                            Style::default().fg(Color::Green),
                        )));
                    } else {
                        // 没有添加行，右侧显示空行
                        right_lines.push(Line::from(Span::styled(
                            "     │".to_string(),
                            Style::default().fg(Color::DarkGray),
                        )));
                    }
                }

                old_line_num += removed_lines.len() as u32;
                new_line_num += added_lines.len() as u32;
            } else if let Some(line_content) = line.strip_prefix('+') {
                // 只有添加行（没有前面的删除行）
                let formatted_line = format!("{:4} │ {}", new_line_num, line_content);
                right_lines.push(Line::from(Span::styled(
                    formatted_line.to_string(),
                    Style::default().fg(Color::Green),
                )));

                // 左边显示空行
                left_lines.push(Line::from(Span::styled(
                    "     │".to_string(),
                    Style::default().fg(Color::DarkGray),
                )));

                new_line_num += 1;
                i += 1;
            } else if let Some(line_content) = line.strip_prefix(' ') {
                // 上下文行：两边都显示
                let left_formatted = format!("{:4} │ {}", old_line_num, line_content);
                let right_formatted = format!("{:4} │ {}", new_line_num, line_content);

                left_lines.push(Line::from(Span::styled(
                    left_formatted.to_string(),
                    Style::default().fg(Color::White),
                )));
                right_lines.push(Line::from(Span::styled(
                    right_formatted.to_string(),
                    Style::default().fg(Color::White),
                )));

                old_line_num += 1;
                new_line_num += 1;
                i += 1;
            } else if !line.is_empty() {
                // 其他内容行（如文件名等）：两边都显示
                let header_line = Line::from(Span::styled(
                    line.to_string(),
                    Style::default().fg(Color::Yellow),
                ));
                left_lines.push(header_line.clone());
                right_lines.push(header_line);
                i += 1;
            } else {
                i += 1;
            }
        }

        (left_lines, right_lines)
    }

    /// 解析 diff 内容用于分割显示
    fn parse_diff_for_split(
        &self,
        diff_content: &str,
    ) -> (
        Vec<ratatui::text::Line<'static>>,
        Vec<ratatui::text::Line<'static>>,
    ) {
        use ratatui::{
            style::{Color, Style},
            text::{Line, Span},
        };

        let mut removed_lines = Vec::new();
        let mut added_lines = Vec::new();
        let mut old_line_num = 0u32;
        let mut new_line_num = 0u32;
        let mut in_diff = false;

        for line in diff_content.lines() {
            // 跳过 diff --git 之前的 commit metadata
            if line.starts_with("diff --git") {
                in_diff = true;
            }
            if !in_diff {
                continue;
            }

            if line.starts_with("@@") {
                // 解析行号信息：@@ -old_start,old_count +new_start,new_count @@ [optional context]
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 {
                    if let Some(old_part) = parts[1].strip_prefix('-') {
                        if let Some((start, _)) = old_part.split_once(',') {
                            old_line_num = start.parse().unwrap_or(0);
                        } else {
                            old_line_num = old_part.parse().unwrap_or(0);
                        }
                    }
                    if let Some(new_part) = parts[2].strip_prefix('+') {
                        if let Some((start, _)) = new_part.split_once(',') {
                            new_line_num = start.parse().unwrap_or(0);
                        } else {
                            new_line_num = new_part.parse().unwrap_or(0);
                        }
                    }
                }

                let header_line = Line::from(Span::styled(
                    line.to_string(),
                    Style::default().fg(Color::Cyan),
                ));
                removed_lines.push(header_line.clone());
                added_lines.push(header_line);
                continue;
            }

            if line.starts_with("diff --git")
                || line.starts_with("index")
                || line.starts_with("---")
                || line.starts_with("+++")
            {
                continue;
            }

            if let Some(line_content) = line.strip_prefix('-') {
                // 删除的行
                let formatted_line = format!("{:4} │ {}", old_line_num, line_content);
                removed_lines.push(Line::from(Span::styled(
                    formatted_line.to_string(),
                    Style::default().fg(Color::Red),
                )));
                old_line_num += 1;
            } else if let Some(line_content) = line.strip_prefix('+') {
                // 添加的行
                let formatted_line = format!("{:4} │ {}", new_line_num, line_content);
                added_lines.push(Line::from(Span::styled(
                    formatted_line.to_string(),
                    Style::default().fg(Color::Green),
                )));
                new_line_num += 1;
            } else if let Some(line_content) = line.strip_prefix(' ') {
                // 上下文行：两边都显示
                let old_formatted = format!("{:4} │ {}", old_line_num, line_content);
                let new_formatted = format!("{:4} │ {}", new_line_num, line_content);

                removed_lines.push(Line::from(Span::styled(
                    old_formatted.to_string(),
                    Style::default().fg(Color::White),
                )));
                added_lines.push(Line::from(Span::styled(
                    new_formatted.to_string(),
                    Style::default().fg(Color::White),
                )));

                old_line_num += 1;
                new_line_num += 1;
            } else if !line.is_empty() {
                // 其他内容行
                let header_line = Line::from(Span::styled(
                    line.to_string(),
                    Style::default().fg(Color::Yellow),
                ));
                removed_lines.push(header_line.clone());
                added_lines.push(header_line);
            }
        }

        (removed_lines, added_lines)
    }

    /// 解析 diff 内容用于统一显示
    fn parse_diff_for_unified(&self, diff_content: &str) -> Vec<ratatui::text::Line<'static>> {
        use ratatui::{
            style::{Color, Style},
            text::{Line, Span},
        };

        let mut lines = Vec::new();
        let mut old_line_num = 0u32;
        let mut new_line_num = 0u32;
        let mut in_diff = false;

        for line in diff_content.lines() {
            // 跳过 diff --git 之前的 commit metadata
            if line.starts_with("diff --git") {
                in_diff = true;
            }
            if !in_diff {
                continue;
            }

            if line.starts_with("@@") {
                // 解析行号信息：@@ -old_start,old_count +new_start,new_count @@ [optional context]
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 {
                    if let Some(old_part) = parts[1].strip_prefix('-') {
                        if let Some((start, _)) = old_part.split_once(',') {
                            old_line_num = start.parse().unwrap_or(0);
                        } else {
                            old_line_num = old_part.parse().unwrap_or(0);
                        }
                    }
                    if let Some(new_part) = parts[2].strip_prefix('+') {
                        if let Some((start, _)) = new_part.split_once(',') {
                            new_line_num = start.parse().unwrap_or(0);
                        } else {
                            new_line_num = new_part.parse().unwrap_or(0);
                        }
                    }
                }
                lines.push(Line::from(Span::styled(
                    line.to_string(),
                    Style::default().fg(Color::Cyan),
                )));
                continue;
            }

            if line.starts_with("diff --git") {
                lines.push(Line::from(Span::styled(
                    line.to_string(),
                    Style::default().fg(Color::Yellow),
                )));
                continue;
            }

            if line.starts_with("index") || line.starts_with("---") || line.starts_with("+++") {
                continue;
            }

            if let Some(line_content) = line.strip_prefix('-') {
                // 删除的行
                let formatted_line = format!("{:4}   │ -{}", old_line_num, line_content);
                lines.push(Line::from(Span::styled(
                    formatted_line.to_string(),
                    Style::default().fg(Color::Red),
                )));
                old_line_num += 1;
            } else if let Some(line_content) = line.strip_prefix('+') {
                // 添加的行
                let formatted_line = format!("   {:4} │ +{}", new_line_num, line_content);
                lines.push(Line::from(Span::styled(
                    formatted_line.to_string(),
                    Style::default().fg(Color::Green),
                )));
                new_line_num += 1;
            } else if let Some(line_content) = line.strip_prefix(' ') {
                // 上下文行
                let formatted_line =
                    format!("{:4}:{:4} │  {}", old_line_num, new_line_num, line_content);
                lines.push(Line::from(Span::styled(
                    formatted_line.to_string(),
                    Style::default().fg(Color::White),
                )));
                old_line_num += 1;
                new_line_num += 1;
            } else if !line.is_empty() {
                // 其他内容行
                lines.push(Line::from(Span::styled(
                    line.to_string(),
                    Style::default().fg(Color::White),
                )));
            }
        }

        lines
    }

    /// 渲染模态框
    pub(crate) fn render_modal(
        &mut self,
        frame: &mut ratatui::Frame,
        modal: &crate::tui_unified::state::app_state::ModalState,
        area: ratatui::layout::Rect,
    ) {
        use ratatui::{
            layout::{Alignment, Constraint, Direction, Layout},
            style::{Color, Style},
            text::Text,
            widgets::Paragraph,
        };

        match modal.modal_type {
            crate::tui_unified::state::app_state::ModalType::DiffViewer => {
                // 计算弹窗尺寸（占据大部分屏幕）
                let popup_area = {
                    let vertical = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([
                            Constraint::Length(2),
                            Constraint::Min(10),
                            Constraint::Length(2),
                        ])
                        .split(area);

                    Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints([
                            Constraint::Length(2),
                            Constraint::Min(60),
                            Constraint::Length(2),
                        ])
                        .split(vertical[1])[1]
                };

                // 使用专门的背景清除方法
                self.clear_modal_background(frame, area);

                // 更新视口高度（popup_area 减去 info(3) + status(4) + borders(4)）
                if let Some(viewer) = &mut self.diff_viewer {
                    viewer.viewport_height = popup_area.height.saturating_sub(11);
                }

                // 预填充渲染缓存（避免每帧重新解析 diff）
                self.ensure_diff_cache();

                // 使用自定义的DiffViewer渲染，限制在popup区域内
                if let Some(viewer) = &self.diff_viewer {
                    self.render_diff_viewer_in_area(frame, viewer, popup_area);
                } else {
                    // 如果diff_viewer没有初始化，显示loading
                    let loading_paragraph = ratatui::widgets::Paragraph::new("Loading diff...")
                        .block(
                            ratatui::widgets::Block::default()
                                .borders(ratatui::widgets::Borders::ALL)
                                .title("Diff Viewer"),
                        );
                    frame.render_widget(loading_paragraph, popup_area);
                }

                // 渲染关闭提示
                let help_area = ratatui::layout::Rect {
                    x: popup_area.x,
                    y: popup_area.y + popup_area.height,
                    width: popup_area.width,
                    height: 1,
                };

                let help_text = "Press [Esc] or [q] to close | [↑↓/jk] scroll | [PgUp/PgDn/ud] page | [g/G] start/end | [←→] files (side-by-side) | [1] unified | [2] side-by-side | [3/t] file list | [w] word-level | [n] line numbers | [h] syntax";
                let help = Paragraph::new(Text::from(help_text))
                    .style(Style::default().fg(Color::Gray).bg(Color::Black))
                    .alignment(Alignment::Center);
                frame.render_widget(help, help_area);
            }
            crate::tui_unified::state::app_state::ModalType::AICommit => {
                // AI Commit 模态框
                let popup_area = {
                    let vertical = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([
                            Constraint::Percentage(25),
                            Constraint::Min(15),
                            Constraint::Percentage(25),
                        ])
                        .split(area);

                    Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints([
                            Constraint::Percentage(20),
                            Constraint::Min(60),
                            Constraint::Percentage(20),
                        ])
                        .split(vertical[1])[1]
                };

                // 使用专门的背景清除方法
                self.clear_modal_background(frame, area);

                // AI Commit 对话框
                use ratatui::widgets::{Block, Borders};

                if self.ai_commit_editing {
                    // 编辑模式：显示编辑器
                    match self.state.try_read() {
                        Ok(state) => {
                            self.commit_editor.render(frame, popup_area, &state);
                        }
                        Err(_) => {
                            // 如果无法获取状态，使用一个静态的虚拟状态
                            static DUMMY_STATE: std::sync::LazyLock<
                                crate::tui_unified::state::AppState,
                            > = std::sync::LazyLock::new(|| crate::tui_unified::state::AppState {
                                layout: Default::default(),
                                focus: Default::default(),
                                current_view:
                                    crate::tui_unified::state::app_state::ViewType::GitLog,
                                modal: None,
                                repo_state: Default::default(),
                                selected_items: Default::default(),
                                search_state: Default::default(),
                                config: crate::tui_unified::config::AppConfig::default(),
                                loading_tasks: HashMap::new(),
                                notifications: Vec::new(),
                                new_layout: Default::default(),
                            });
                            self.commit_editor.render(frame, popup_area, &DUMMY_STATE);
                        }
                    }
                } else {
                    // 非编辑模式：显示生成的消息
                    let ai_commit_content = if let Some(ref message) = self.ai_commit_message {
                        format!(
                            "Status: {}\n\n📝 Generated Commit Message:\n\n{}",
                            self.ai_commit_status
                                .as_ref()
                                .unwrap_or(&"Ready".to_string()),
                            message.trim()
                        )
                    } else {
                        format!(
                            "🤖 {}",
                            self.ai_commit_status
                                .as_ref()
                                .unwrap_or(&"Generating commit message...".to_string())
                        )
                    };

                    let ai_commit_block = Paragraph::new(Text::from(ai_commit_content))
                        .block(
                            Block::default()
                                .borders(Borders::ALL)
                                .title("AI Commit")
                                .border_style(Style::default().fg(Color::Green)),
                        )
                        .style(Style::default().fg(Color::White))
                        .wrap(ratatui::widgets::Wrap { trim: true });

                    frame.render_widget(ai_commit_block, popup_area);
                }

                // 帮助文本
                let help_area = ratatui::layout::Rect {
                    x: popup_area.x,
                    y: popup_area.y + popup_area.height,
                    width: popup_area.width,
                    height: 1,
                };

                let help_text = if self.ai_commit_editing {
                    "[Tab] Save & Exit Edit | [Esc] Cancel Edit"
                } else if self.ai_commit_push_prompt {
                    "[y/Enter] Push | [n/Esc] Skip Push"
                } else if self.ai_commit_message.is_some() {
                    "[Enter] Commit | [e] Edit | [Esc] Cancel"
                } else {
                    "🤖 Generating commit message... | [Esc] Cancel"
                };
                let help = Paragraph::new(Text::from(help_text))
                    .style(Style::default().fg(Color::Gray))
                    .alignment(Alignment::Center);
                frame.render_widget(help, help_area);
            }
            crate::tui_unified::state::app_state::ModalType::AIReview
            | crate::tui_unified::state::app_state::ModalType::AIRefactor => {
                // AI Review / Refactor 结果模态框（大面积，可滚动）
                let popup_area = {
                    let vertical = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([
                            Constraint::Length(2),
                            Constraint::Min(10),
                            Constraint::Length(2),
                        ])
                        .split(area);

                    Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints([
                            Constraint::Length(4),
                            Constraint::Min(60),
                            Constraint::Length(4),
                        ])
                        .split(vertical[1])[1]
                };

                // 使用专门的背景清除方法
                self.clear_modal_background(frame, area);

                use ratatui::widgets::{Block, Borders, Wrap};

                let (title, border_color) = match modal.modal_type {
                    crate::tui_unified::state::app_state::ModalType::AIReview => {
                        ("AI Code Review", Color::Cyan)
                    }
                    crate::tui_unified::state::app_state::ModalType::AIRefactor => {
                        ("AI Refactor Suggestions", Color::Magenta)
                    }
                    _ => unreachable!(),
                };

                let content_block = Paragraph::new(Text::from(modal.content.clone()))
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title(title)
                            .border_style(Style::default().fg(border_color)),
                    )
                    .style(Style::default().fg(Color::White))
                    .wrap(Wrap { trim: false });

                frame.render_widget(content_block, popup_area);

                // 帮助文本
                let help_area = ratatui::layout::Rect {
                    x: popup_area.x,
                    y: popup_area.y + popup_area.height,
                    width: popup_area.width,
                    height: 1,
                };

                let help_text = "[Esc] or [q] Close";
                let help = Paragraph::new(Text::from(help_text))
                    .style(Style::default().fg(Color::Gray))
                    .alignment(Alignment::Center);
                frame.render_widget(help, help_area);
            }
            _ => {
                // 对于其他类型的模态框，使用简单的消息框
                let popup_area = {
                    let vertical = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([
                            Constraint::Percentage(30),
                            Constraint::Min(10),
                            Constraint::Percentage(30),
                        ])
                        .split(area);

                    Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints([
                            Constraint::Percentage(25),
                            Constraint::Min(50),
                            Constraint::Percentage(25),
                        ])
                        .split(vertical[1])[1]
                };

                // 使用专门的背景清除方法
                self.clear_modal_background(frame, area);

                // 渲染通用模态框
                use ratatui::widgets::{Block, Borders};
                let modal_block = Paragraph::new(Text::from(modal.content.clone()))
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title(modal.title.clone())
                            .border_style(Style::default().fg(Color::Yellow)),
                    )
                    .style(Style::default().fg(Color::White))
                    .wrap(ratatui::widgets::Wrap { trim: true });

                frame.render_widget(modal_block, popup_area);

                // 帮助文本
                let help_area = ratatui::layout::Rect {
                    x: popup_area.x,
                    y: popup_area.y + popup_area.height,
                    width: popup_area.width,
                    height: 1,
                };

                let help_text = "[Enter] OK | [Esc] Cancel";
                let help = Paragraph::new(Text::from(help_text))
                    .style(Style::default().fg(Color::Gray))
                    .alignment(Alignment::Center);
                frame.render_widget(help, help_area);
            }
        }
    }

    /// 处理模态框按键事件
    pub(crate) async fn handle_modal_key(&mut self, key: KeyEvent) -> Result<()> {
        use crossterm::event::KeyCode;

        // 先检查是否为DiffViewer模态框，如果是就转发键盘事件
        let state = self.state.read().await;
        if let Some(modal) = &state.modal {
            match modal.modal_type {
                crate::tui_unified::state::app_state::ModalType::DiffViewer => {
                    // 优先检查退出键，避免被DiffViewerComponent消费
                    match key.code {
                        KeyCode::Esc | KeyCode::Char('q') => {
                            drop(state);
                            self.diff_viewer = None;
                            let mut state = self.state.write().await;
                            state.hide_modal();
                            return Ok(());
                        }
                        _ => {}
                    }

                    // 其他键转发到DiffViewer，使用和--query-tui-pro相同的逻辑
                    drop(state);
                    if let Some(viewer) = &mut self.diff_viewer {
                        match key.code {
                            KeyCode::Char('j') | KeyCode::Tab | KeyCode::Down => {
                                viewer.next_file();
                                viewer.load_current_file_diff().await;
                            }
                            KeyCode::Char('k') | KeyCode::BackTab | KeyCode::Up => {
                                viewer.prev_file();
                                viewer.load_current_file_diff().await;
                            }
                            KeyCode::Char('J') => {
                                viewer.diff_scroll = viewer.diff_scroll.saturating_add(1);
                                viewer.clamp_scroll();
                            }
                            KeyCode::Char('K') => {
                                viewer.diff_scroll = viewer.diff_scroll.saturating_sub(1);
                            }
                            KeyCode::PageDown => {
                                viewer.diff_scroll = viewer.diff_scroll.saturating_add(10);
                                viewer.clamp_scroll();
                            }
                            KeyCode::PageUp => {
                                viewer.diff_scroll = viewer.diff_scroll.saturating_sub(10);
                            }
                            KeyCode::Char('1') => {
                                viewer.set_view_mode(crate::diff_viewer::DiffViewMode::Unified);
                            }
                            KeyCode::Char('2') => {
                                viewer.set_view_mode(crate::diff_viewer::DiffViewMode::SideBySide);
                            }
                            KeyCode::Char('3') => {
                                viewer.show_file_list = !viewer.show_file_list;
                            }
                            KeyCode::Char('t') => {
                                viewer.show_file_list = !viewer.show_file_list;
                            }
                            KeyCode::Char('h') => {
                                viewer.syntax_highlight = !viewer.syntax_highlight;
                            }
                            KeyCode::Left | KeyCode::Char('H') => {
                                viewer.prev_hunk();
                            }
                            KeyCode::Right | KeyCode::Char('L') => {
                                viewer.next_hunk();
                            }
                            _ => {}
                        }
                    }
                }
                crate::tui_unified::state::app_state::ModalType::AIReview
                | crate::tui_unified::state::app_state::ModalType::AIRefactor => {
                    // AI Review/Refactor 模态框：只处理关闭键
                    match key.code {
                        KeyCode::Esc | KeyCode::Char('q') => {
                            drop(state);
                            let mut state = self.state.write().await;
                            state.hide_modal();
                            return Ok(());
                        }
                        _ => {}
                    }
                }
                _ => {
                    // 对于其他模态框类型，只处理关闭快捷键
                    match key.code {
                        KeyCode::Esc | KeyCode::Char('q') => {
                            // 如果是AI commit推送提示模式，跳过推送
                            if self.ai_commit_mode && self.ai_commit_push_prompt {
                                drop(state); // 显式释放读锁
                                self.skip_push();
                                let mut state = self.state.write().await;
                                state.hide_modal();
                                return Ok(());
                            }
                            // 如果是AI commit编辑模式，退出编辑但保持AI commit模式
                            else if self.ai_commit_mode && self.ai_commit_editing {
                                drop(state); // 显式释放读锁
                                self.ai_commit_editing = false;
                                self.commit_editor.set_focused(false);
                                // 恢复到非编辑模式，用户仍可以提交或再次编辑
                                return Ok(());
                            }
                            // 如果是AI commit非编辑模式，完全退出AI commit模式
                            else if self.ai_commit_mode {
                                drop(state); // 显式释放读锁
                                self.exit_ai_commit_mode();
                            } else {
                                drop(state); // 显式释放读锁
                            }
                            let mut state = self.state.write().await;
                            state.hide_modal();
                            return Ok(());
                        }
                        KeyCode::Enter => {
                            // 在Git Pull模式下，Enter确认拉取
                            if modal.modal_type
                                == crate::tui_unified::state::app_state::ModalType::GitPull
                            {
                                drop(state); // 显式释放读锁
                                return self.confirm_git_pull().await;
                            }
                            // 在分支切换模式下，Enter确认切换
                            else if modal.modal_type
                                == crate::tui_unified::state::app_state::ModalType::BranchSwitch
                            {
                                drop(state); // 显式释放读锁
                                return self.confirm_branch_switch().await;
                            }
                            // 在AI commit推送提示模式下，Enter等于确认推送
                            else if self.ai_commit_mode && self.ai_commit_push_prompt {
                                drop(state); // 显式释放读锁
                                return self.confirm_push().await;
                            }
                            // 在AI commit模式下按Enter确认提交
                            else if self.ai_commit_mode
                                && !self.ai_commit_editing
                                && self.ai_commit_message.is_some()
                            {
                                drop(state); // 显式释放读锁
                                return self.confirm_ai_commit().await;
                            }
                        }
                        KeyCode::Char('e') => {
                            // 在AI commit模式下按e编辑commit message
                            if self.ai_commit_mode && !self.ai_commit_editing {
                                self.ai_commit_editing = true;
                                // 将当前消息加载到编辑器中
                                if let Some(ref message) = self.ai_commit_message {
                                    self.commit_editor.set_content(message);
                                }
                                self.commit_editor.set_focused(true);
                            }
                        }
                        KeyCode::Char('y') | KeyCode::Char('Y') => {
                            // 在AI commit推送提示模式下，'y'键确认推送
                            if self.ai_commit_mode && self.ai_commit_push_prompt {
                                drop(state); // 显式释放读锁
                                return self.confirm_push().await;
                            }
                        }
                        KeyCode::Char('n') | KeyCode::Char('N') => {
                            // 在AI commit推送提示模式下，'n'键跳过推送
                            if self.ai_commit_mode && self.ai_commit_push_prompt {
                                drop(state); // 显式释放读锁
                                self.skip_push();
                                return Ok(());
                            }
                        }
                        KeyCode::Tab => {
                            // 在AI commit编辑模式下，Tab键退出编辑并保存
                            if self.ai_commit_mode && self.ai_commit_editing {
                                self.ai_commit_editing = false;
                                self.commit_editor.set_focused(false);
                                // 保存编辑的内容
                                let edited_content = self.commit_editor.get_content();
                                self.ai_commit_message = Some(edited_content);
                                self.ai_commit_status = Some("Message edited".to_string());

                                // 不需要重新显示模态框，因为渲染逻辑会自动切换到非编辑模式显示
                                // 现在用户可以按 Enter 提交或 Esc 取消
                            }
                        }
                        _ => {
                            // 在AI commit编辑模式下，将键盘事件转发给编辑器
                            if self.ai_commit_mode && self.ai_commit_editing {
                                let mut dummy_state = crate::tui_unified::state::AppState::new(
                                    &crate::tui_unified::config::AppConfig::default(),
                                )
                                .await
                                .unwrap_or_else(|_| {
                                    // 如果创建失败，创建一个基本的虚拟状态
                                    crate::tui_unified::state::AppState {
                                        layout: Default::default(),
                                        focus: Default::default(),
                                        current_view:
                                            crate::tui_unified::state::app_state::ViewType::GitLog,
                                        modal: None,
                                        repo_state: Default::default(),
                                        selected_items: Default::default(),
                                        search_state: Default::default(),
                                        config: crate::tui_unified::config::AppConfig::default(),
                                        loading_tasks: HashMap::new(),
                                        notifications: Vec::new(),
                                        new_layout: Default::default(),
                                    }
                                });
                                let _result =
                                    self.commit_editor.handle_key_event(key, &mut dummy_state);
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
