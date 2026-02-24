use crate::diff_viewer::{DiffViewMode, DiffViewer};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

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
    pub(super) fn ensure_diff_cache(&mut self) {
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
    pub(super) fn clear_modal_background(&self, frame: &mut ratatui::Frame, area: ratatui::layout::Rect) {
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
    pub(super) fn render_diff_viewer_in_area(
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

    pub(super) fn render_diff_content_by_mode(
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
}
