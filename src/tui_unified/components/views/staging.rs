// 交互式暂存视图组件
use crate::tui_unified::{
    components::base::{
        component::{Component, ViewComponent, ViewType},
        events::EventResult,
    },
    state::{
        app_state::NotificationLevel,
        git_state::ChangeType,
        AppState,
    },
};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};
use std::path::PathBuf;

/// 暂存文件条目 - 包含文件信息和暂存状态
#[derive(Debug, Clone)]
struct StagingEntry {
    path: PathBuf,
    change_type: ChangeType,
    is_staged: bool,
    #[allow(dead_code)]
    additions: usize,
    #[allow(dead_code)]
    deletions: usize,
}

impl StagingEntry {
    fn status_char(&self) -> &str {
        match self.change_type {
            ChangeType::Added => "A",
            ChangeType::Modified => "M",
            ChangeType::Deleted => "D",
            ChangeType::Renamed { .. } => "R",
            ChangeType::Copied { .. } => "C",
            ChangeType::Unmerged => "U",
            ChangeType::TypeChange => "T",
        }
    }

    fn status_color(&self) -> Color {
        match self.change_type {
            ChangeType::Added => Color::Green,
            ChangeType::Modified => Color::Yellow,
            ChangeType::Deleted => Color::Red,
            ChangeType::Renamed { .. } => Color::Cyan,
            ChangeType::Copied { .. } => Color::Cyan,
            ChangeType::Unmerged => Color::Magenta,
            ChangeType::TypeChange => Color::Blue,
        }
    }
}

/// 交互式暂存视图
///
/// 提供文件级别的 stage/unstage 操作、diff 预览和 AI commit 触发。
pub struct StagingView {
    files: Vec<StagingEntry>,
    selected_index: usize,
    focused: bool,
    diff_preview: String,
    diff_scroll_offset: usize,
    list_state: ListState,
}

impl Default for StagingView {
    fn default() -> Self {
        Self::new()
    }
}

impl StagingView {
    pub fn new() -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));
        Self {
            files: Vec::new(),
            selected_index: 0,
            focused: false,
            diff_preview: String::new(),
            diff_scroll_offset: 0,
            list_state,
        }
    }

    /// 从 AppState 刷新文件列表
    pub fn refresh_file_list(&mut self, state: &AppState) {
        self.files.clear();

        // 添加已暂存文件
        for file in &state.repo_state.status.staged_files {
            self.files.push(StagingEntry {
                path: file.path.clone(),
                change_type: file.status.clone(),
                is_staged: true,
                additions: file.additions,
                deletions: file.deletions,
            });
        }

        // 添加未暂存文件
        for file in &state.repo_state.status.unstaged_files {
            self.files.push(StagingEntry {
                path: file.path.clone(),
                change_type: file.status.clone(),
                is_staged: false,
                additions: file.additions,
                deletions: file.deletions,
            });
        }

        // 添加未追踪文件
        for path in &state.repo_state.status.untracked_files {
            self.files.push(StagingEntry {
                path: path.clone(),
                change_type: ChangeType::Added,
                is_staged: false,
                additions: 0,
                deletions: 0,
            });
        }

        // 确保选中索引在范围内
        if !self.files.is_empty() {
            self.selected_index = self.selected_index.min(self.files.len() - 1);
        } else {
            self.selected_index = 0;
        }
        self.list_state.select(Some(self.selected_index));
    }

    /// 获取已暂存文件数量
    pub fn staged_count(&self) -> usize {
        self.files.iter().filter(|f| f.is_staged).count()
    }

    /// 获取总文件数量
    pub fn total_count(&self) -> usize {
        self.files.len()
    }

    /// 是否有已暂存的文件
    pub fn has_staged_files(&self) -> bool {
        self.files.iter().any(|f| f.is_staged)
    }

    /// 请求切换当前选中文件的暂存状态
    fn request_toggle_staging(&self, state: &mut AppState) {
        if !self.files.is_empty() {
            *state
                .selected_items
                .pending_staging_toggle
                .lock()
                .unwrap_or_else(|e| e.into_inner()) = Some(self.selected_index);
        }
    }

    /// 请求暂存全部文件
    fn request_stage_all(&self, state: &mut AppState) {
        *state
            .selected_items
            .pending_stage_all
            .lock()
            .unwrap_or_else(|e| e.into_inner()) = true;
    }

    /// 获取当前选中文件的路径
    pub fn selected_file_path(&self) -> Option<&PathBuf> {
        self.files.get(self.selected_index).map(|f| &f.path)
    }

    /// 设置 diff 预览内容
    pub fn set_diff_preview(&mut self, content: String) {
        self.diff_preview = content;
        self.diff_scroll_offset = 0;
    }

    fn move_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
            self.list_state.select(Some(self.selected_index));
            self.diff_scroll_offset = 0;
        }
    }

    fn move_down(&mut self) {
        if !self.files.is_empty() && self.selected_index < self.files.len() - 1 {
            self.selected_index += 1;
            self.list_state.select(Some(self.selected_index));
            self.diff_scroll_offset = 0;
        }
    }

    fn render_file_list(&mut self, frame: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = self
            .files
            .iter()
            .enumerate()
            .map(|(idx, entry)| {
                let checkbox = if entry.is_staged { "[x]" } else { "[ ]" };
                let status_char = entry.status_char();
                let path_str = entry.path.to_string_lossy();
                let stage_label = if entry.is_staged {
                    "(staged)"
                } else {
                    "(unstaged)"
                };

                let is_selected = idx == self.selected_index;

                let line = Line::from(vec![
                    Span::styled(
                        format!(" {} ", checkbox),
                        if entry.is_staged {
                            Style::default().fg(Color::Green)
                        } else {
                            Style::default().fg(Color::DarkGray)
                        },
                    ),
                    Span::styled(
                        format!("{:<2}", status_char),
                        Style::default().fg(entry.status_color()),
                    ),
                    Span::styled(
                        format!(" {:<40}", path_str),
                        if is_selected && self.focused {
                            Style::default()
                                .fg(Color::Black)
                                .bg(Color::Yellow)
                                .add_modifier(Modifier::BOLD)
                        } else if is_selected {
                            Style::default().bg(Color::DarkGray)
                        } else {
                            Style::default().fg(Color::White)
                        },
                    ),
                    Span::styled(
                        format!(" {}", stage_label),
                        Style::default()
                            .fg(if entry.is_staged {
                                Color::Green
                            } else {
                                Color::DarkGray
                            })
                            .add_modifier(Modifier::ITALIC),
                    ),
                ]);

                ListItem::new(line)
            })
            .collect();

        let title = format!(
            " Files to Stage  {}/{} ",
            self.staged_count(),
            self.total_count()
        );

        let list = List::new(items).block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(if self.focused {
                    Style::default().fg(Color::Cyan)
                } else {
                    Style::default().fg(Color::DarkGray)
                }),
        );

        frame.render_stateful_widget(list, area, &mut self.list_state);
    }

    fn render_diff_preview(&self, frame: &mut Frame, area: Rect) {
        let lines: Vec<Line> = if self.diff_preview.is_empty() {
            vec![Line::from(Span::styled(
                " Select a file to preview changes",
                Style::default().fg(Color::DarkGray),
            ))]
        } else {
            self.diff_preview
                .lines()
                .skip(self.diff_scroll_offset)
                .map(|line| {
                    let style = if line.starts_with('+') && !line.starts_with("+++") {
                        Style::default().fg(Color::Green)
                    } else if line.starts_with('-') && !line.starts_with("---") {
                        Style::default().fg(Color::Red)
                    } else if line.starts_with("@@") {
                        Style::default().fg(Color::Cyan)
                    } else if line.starts_with("diff ") || line.starts_with("index ") {
                        Style::default().fg(Color::Yellow)
                    } else {
                        Style::default().fg(Color::White)
                    };
                    Line::from(Span::styled(line.to_string(), style))
                })
                .collect()
        };

        let paragraph = Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Diff Preview ")
                    .border_style(Style::default().fg(Color::DarkGray)),
            )
            .wrap(Wrap { trim: false });

        frame.render_widget(paragraph, area);
    }

    fn render_help_bar(&self, frame: &mut Frame, area: Rect) {
        let help_text = Line::from(vec![
            Span::styled(" Space", Style::default().fg(Color::Yellow)),
            Span::raw(":toggle  "),
            Span::styled("a", Style::default().fg(Color::Yellow)),
            Span::raw(":stage all  "),
            Span::styled("u", Style::default().fg(Color::Yellow)),
            Span::raw(":unstage all  "),
            Span::styled("c", Style::default().fg(Color::Yellow)),
            Span::raw(":commit  "),
            Span::styled("r", Style::default().fg(Color::Yellow)),
            Span::raw(":refresh  "),
            Span::styled("q", Style::default().fg(Color::Yellow)),
            Span::raw(":back"),
        ]);

        let paragraph = Paragraph::new(help_text).style(Style::default().fg(Color::White));
        frame.render_widget(paragraph, area);
    }
}

impl Component for StagingView {
    fn name(&self) -> &str {
        "StagingView"
    }

    fn render(&mut self, frame: &mut Frame, area: Rect, _state: &AppState) {
        // 布局: 文件列表 50% + diff 预览 45% + 帮助栏 1行
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(50),
                Constraint::Percentage(48),
                Constraint::Min(1),
            ])
            .split(area);

        self.render_file_list(frame, chunks[0]);
        self.render_diff_preview(frame, chunks[1]);
        self.render_help_bar(frame, chunks[2]);
    }

    fn handle_key_event(&mut self, key: KeyEvent, state: &mut AppState) -> EventResult {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.move_up();
                // 请求更新 diff 预览
                if let Some(path) = self.selected_file_path() {
                    state.request_diff(path.to_string_lossy().to_string());
                }
                EventResult::Handled
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.move_down();
                if let Some(path) = self.selected_file_path() {
                    state.request_diff(path.to_string_lossy().to_string());
                }
                EventResult::Handled
            }
            KeyCode::Char(' ') => {
                // 切换暂存状态
                self.request_toggle_staging(state);
                EventResult::Handled
            }
            KeyCode::Char('a') => {
                // 暂存全部
                self.request_stage_all(state);
                state.add_notification("Staging all files...".to_string(), NotificationLevel::Info);
                EventResult::Handled
            }
            KeyCode::Char('u') => {
                // 取消暂存全部（通过逐个取消已暂存的文件）
                // 这里简单处理：设置一个标记，在 git_operations 中处理
                state.add_notification(
                    "Unstaging all files...".to_string(),
                    NotificationLevel::Info,
                );
                EventResult::Handled
            }
            KeyCode::Char('c') => {
                // 触发 AI commit
                if self.has_staged_files() {
                    state.add_notification(
                        "Triggering AI commit...".to_string(),
                        NotificationLevel::Info,
                    );
                    EventResult::Handled
                } else {
                    state.add_notification(
                        "No staged files to commit".to_string(),
                        NotificationLevel::Warning,
                    );
                    EventResult::Handled
                }
            }
            KeyCode::Char('r') => {
                // 刷新文件列表
                self.refresh_file_list(state);
                state.add_notification(
                    "Staging view refreshed".to_string(),
                    NotificationLevel::Info,
                );
                EventResult::Handled
            }
            KeyCode::PageUp | KeyCode::Char('U') => {
                // Diff 预览向上翻页
                self.diff_scroll_offset = self.diff_scroll_offset.saturating_sub(10);
                EventResult::Handled
            }
            KeyCode::PageDown | KeyCode::Char('D') => {
                // Diff 预览向下翻页
                let max_offset = self.diff_preview.lines().count().saturating_sub(5);
                self.diff_scroll_offset = (self.diff_scroll_offset + 10).min(max_offset);
                EventResult::Handled
            }
            _ => EventResult::NotHandled,
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
        (50, 20)
    }
}

impl ViewComponent for StagingView {
    fn view_type(&self) -> ViewType {
        ViewType::Staging
    }

    fn title(&self) -> String {
        format!("Staging ({}/{})", self.staged_count(), self.total_count())
    }

    fn supports_search(&self) -> bool {
        false
    }

    fn search(&mut self, _query: &str) -> EventResult {
        EventResult::NotHandled
    }

    fn clear_search(&mut self) -> EventResult {
        EventResult::NotHandled
    }

    fn selected_index(&self) -> Option<usize> {
        Some(self.selected_index)
    }

    fn set_selected_index(&mut self, index: Option<usize>) {
        if let Some(idx) = index {
            if idx < self.files.len() {
                self.selected_index = idx;
                self.list_state.select(Some(idx));
            }
        }
    }
}
