// 交互式暂存视图组件 - 支持文件级和 hunk 级暂存
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

/// 一个 diff hunk
#[derive(Debug, Clone)]
pub struct DiffHunk {
    /// hunk 头部 (e.g., "@@ -1,3 +1,4 @@")
    pub header: String,
    /// hunk 内容行 (包含 +/-/空格前缀)
    pub lines: Vec<String>,
    /// 添加行数
    pub additions: usize,
    /// 删除行数
    pub deletions: usize,
}

impl DiffHunk {
    /// 生成完整的 hunk 补丁文本 (用于 git apply --cached)
    pub fn to_patch(&self, file_path: &str) -> String {
        let mut patch = String::new();
        patch.push_str(&format!("--- a/{}\n", file_path));
        patch.push_str(&format!("+++ b/{}\n", file_path));
        patch.push_str(&self.header);
        patch.push('\n');
        for line in &self.lines {
            patch.push_str(line);
            patch.push('\n');
        }
        patch
    }

    /// 摘要描述
    pub fn summary(&self) -> String {
        format!("{} (+{} -{})", self.header, self.additions, self.deletions)
    }
}

/// 从 diff 文本解析 hunk 列表
pub fn parse_hunks(diff: &str) -> Vec<DiffHunk> {
    let mut hunks = Vec::new();
    let mut current_header = String::new();
    let mut current_lines: Vec<String> = Vec::new();
    let mut additions = 0usize;
    let mut deletions = 0usize;
    let mut in_hunk = false;

    for line in diff.lines() {
        if line.starts_with("@@") {
            // 保存前一个 hunk
            if in_hunk && !current_header.is_empty() {
                hunks.push(DiffHunk {
                    header: current_header.clone(),
                    lines: current_lines.clone(),
                    additions,
                    deletions,
                });
            }
            // 开始新 hunk
            current_header = line.to_string();
            current_lines.clear();
            additions = 0;
            deletions = 0;
            in_hunk = true;
        } else if in_hunk {
            // 跳过 diff 头部行
            if line.starts_with("diff ") || line.starts_with("index ") || line.starts_with("--- ") || line.starts_with("+++ ") {
                continue;
            }
            if line.starts_with('+') {
                additions += 1;
            } else if line.starts_with('-') {
                deletions += 1;
            }
            current_lines.push(line.to_string());
        }
    }

    // 保存最后一个 hunk
    if in_hunk && !current_header.is_empty() {
        hunks.push(DiffHunk {
            header: current_header,
            lines: current_lines,
            additions,
            deletions,
        });
    }

    hunks
}

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
    /// 文件是否展开显示 hunks
    expanded: bool,
    /// 解析后的 hunks
    hunks: Vec<DiffHunk>,
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

/// 列表中的可见行 (文件或 hunk)
#[derive(Debug, Clone)]
enum ListRow {
    /// 文件行 (index into files vec)
    File(usize),
    /// Hunk 行 (file_index, hunk_index)
    Hunk(usize, usize),
}

/// 交互式暂存视图
///
/// 提供文件级别和 hunk 级别的 stage/unstage 操作、diff 预览和 AI commit 触发。
pub struct StagingView {
    files: Vec<StagingEntry>,
    focused: bool,
    diff_preview: String,
    diff_scroll_offset: usize,
    list_state: ListState,
    /// 展开后的可见行列表
    visible_rows: Vec<ListRow>,
    /// 当前选中的可见行索引
    cursor: usize,
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
            focused: false,
            diff_preview: String::new(),
            diff_scroll_offset: 0,
            list_state,
            visible_rows: Vec::new(),
            cursor: 0,
        }
    }

    /// 重建可见行列表 (在文件列表或展开状态变更后调用)
    fn rebuild_visible_rows(&mut self) {
        self.visible_rows.clear();
        for (file_idx, entry) in self.files.iter().enumerate() {
            self.visible_rows.push(ListRow::File(file_idx));
            if entry.expanded {
                for hunk_idx in 0..entry.hunks.len() {
                    self.visible_rows.push(ListRow::Hunk(file_idx, hunk_idx));
                }
            }
        }
        // 确保 cursor 在范围内
        if !self.visible_rows.is_empty() {
            self.cursor = self.cursor.min(self.visible_rows.len() - 1);
        } else {
            self.cursor = 0;
        }
        self.list_state.select(Some(self.cursor));
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
                expanded: false,
                hunks: Vec::new(),
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
                expanded: false,
                hunks: Vec::new(),
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
                expanded: false,
                hunks: Vec::new(),
            });
        }

        self.rebuild_visible_rows();
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
        if let Some(ListRow::File(file_idx)) = self.visible_rows.get(self.cursor) {
            *state
                .selected_items
                .pending_staging_toggle
                .lock()
                .unwrap_or_else(|e| e.into_inner()) = Some(*file_idx);
        }
    }

    /// 请求暂存单个 hunk
    fn request_hunk_stage(&self, state: &mut AppState, file_idx: usize, hunk_idx: usize) {
        let entry = &self.files[file_idx];
        let hunk = &entry.hunks[hunk_idx];
        let file_path = entry.path.to_string_lossy().to_string();
        let patch = hunk.to_patch(&file_path);

        *state
            .selected_items
            .pending_hunk_stage
            .lock()
            .unwrap_or_else(|e| e.into_inner()) = Some((file_path, patch));
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
        match self.visible_rows.get(self.cursor) {
            Some(ListRow::File(idx)) => self.files.get(*idx).map(|f| &f.path),
            Some(ListRow::Hunk(file_idx, _)) => self.files.get(*file_idx).map(|f| &f.path),
            None => None,
        }
    }

    /// 设置 diff 预览内容，同时解析 hunks
    pub fn set_diff_preview(&mut self, content: String) {
        // 如果当前选中的是文件，解析 hunks
        if let Some(ListRow::File(file_idx)) = self.visible_rows.get(self.cursor) {
            let hunks = parse_hunks(&content);
            if let Some(entry) = self.files.get_mut(*file_idx) {
                entry.hunks = hunks;
            }
        }
        self.diff_preview = content;
        self.diff_scroll_offset = 0;
    }

    /// 展开/折叠当前选中的文件
    fn toggle_expand(&mut self) {
        if let Some(ListRow::File(file_idx)) = self.visible_rows.get(self.cursor) {
            let file_idx = *file_idx;
            if let Some(entry) = self.files.get_mut(file_idx) {
                if entry.hunks.is_empty() {
                    // 没有 hunks 数据，暂时无法展开（需要先加载 diff）
                    return;
                }
                entry.expanded = !entry.expanded;
                self.rebuild_visible_rows();
            }
        } else if let Some(ListRow::Hunk(file_idx, _)) = self.visible_rows.get(self.cursor) {
            // 在 hunk 行上按 Enter，折叠父文件
            let file_idx = *file_idx;
            if let Some(entry) = self.files.get_mut(file_idx) {
                entry.expanded = false;
                self.rebuild_visible_rows();
                // 跳回到文件行
                for (i, row) in self.visible_rows.iter().enumerate() {
                    if let ListRow::File(idx) = row {
                        if *idx == file_idx {
                            self.cursor = i;
                            self.list_state.select(Some(self.cursor));
                            break;
                        }
                    }
                }
            }
        }
    }

    fn move_up(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
            self.list_state.select(Some(self.cursor));
            self.diff_scroll_offset = 0;
        }
    }

    fn move_down(&mut self) {
        if !self.visible_rows.is_empty() && self.cursor < self.visible_rows.len() - 1 {
            self.cursor += 1;
            self.list_state.select(Some(self.cursor));
            self.diff_scroll_offset = 0;
        }
    }

    fn render_file_list(&mut self, frame: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = self
            .visible_rows
            .iter()
            .enumerate()
            .map(|(vis_idx, row)| {
                let is_selected = vis_idx == self.cursor;
                match row {
                    ListRow::File(file_idx) => {
                        let entry = &self.files[*file_idx];
                        let checkbox = if entry.is_staged { "[x]" } else { "[ ]" };
                        let status_char = entry.status_char();
                        let path_str = entry.path.to_string_lossy();
                        let expand_icon = if entry.expanded {
                            "▼"
                        } else if !entry.hunks.is_empty() {
                            "▶"
                        } else {
                            " "
                        };

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
                                format!("{} ", expand_icon),
                                Style::default().fg(Color::Cyan),
                            ),
                            Span::styled(
                                format!("{}", path_str),
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
                        ]);
                        ListItem::new(line)
                    }
                    ListRow::Hunk(file_idx, hunk_idx) => {
                        let entry = &self.files[*file_idx];
                        let hunk = &entry.hunks[*hunk_idx];

                        let line = Line::from(vec![
                            Span::raw("     "),
                            Span::styled(
                                format!("  {} ", hunk.header),
                                if is_selected && self.focused {
                                    Style::default()
                                        .fg(Color::Black)
                                        .bg(Color::Cyan)
                                        .add_modifier(Modifier::BOLD)
                                } else if is_selected {
                                    Style::default().bg(Color::DarkGray)
                                } else {
                                    Style::default().fg(Color::Cyan)
                                },
                            ),
                            Span::styled(
                                format!(" +{} -{}", hunk.additions, hunk.deletions),
                                Style::default().fg(Color::DarkGray),
                            ),
                        ]);
                        ListItem::new(line)
                    }
                }
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
        // 确定预览标题
        let title = match self.visible_rows.get(self.cursor) {
            Some(ListRow::Hunk(file_idx, hunk_idx)) => {
                let entry = &self.files[*file_idx];
                let hunk = &entry.hunks[*hunk_idx];
                format!(
                    " Hunk {}/{} - {} ",
                    hunk_idx + 1,
                    entry.hunks.len(),
                    hunk.header
                )
            }
            _ => " Diff Preview ".to_string(),
        };

        // 确定预览内容
        let preview_content = match self.visible_rows.get(self.cursor) {
            Some(ListRow::Hunk(file_idx, hunk_idx)) => {
                // 展示特定 hunk 的内容
                let entry = &self.files[*file_idx];
                if let Some(hunk) = entry.hunks.get(*hunk_idx) {
                    let mut content = hunk.header.clone();
                    content.push('\n');
                    for line in &hunk.lines {
                        content.push_str(line);
                        content.push('\n');
                    }
                    content
                } else {
                    self.diff_preview.clone()
                }
            }
            _ => self.diff_preview.clone(),
        };

        let lines: Vec<Line> = if preview_content.is_empty() {
            vec![Line::from(Span::styled(
                " Select a file to preview changes. Press Enter to expand hunks.",
                Style::default().fg(Color::DarkGray),
            ))]
        } else {
            preview_content
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
                    .title(title)
                    .border_style(Style::default().fg(Color::DarkGray)),
            )
            .wrap(Wrap { trim: false });

        frame.render_widget(paragraph, area);
    }

    fn render_help_bar(&self, frame: &mut Frame, area: Rect) {
        let help_text = Line::from(vec![
            Span::styled(" Space", Style::default().fg(Color::Yellow)),
            Span::raw(":toggle  "),
            Span::styled("Enter", Style::default().fg(Color::Yellow)),
            Span::raw(":expand  "),
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
            KeyCode::Enter => {
                // 展开/折叠文件的 hunks
                self.toggle_expand();
                EventResult::Handled
            }
            KeyCode::Char(' ') => {
                // 切换暂存状态 (文件级或 hunk 级)
                match self.visible_rows.get(self.cursor) {
                    Some(ListRow::File(_)) => {
                        self.request_toggle_staging(state);
                    }
                    Some(ListRow::Hunk(file_idx, hunk_idx)) => {
                        let file_idx = *file_idx;
                        let hunk_idx = *hunk_idx;
                        self.request_hunk_stage(state, file_idx, hunk_idx);
                        state.add_notification(
                            "Staging hunk...".to_string(),
                            NotificationLevel::Info,
                        );
                    }
                    None => {}
                }
                EventResult::Handled
            }
            KeyCode::Char('a') => {
                // 暂存全部
                self.request_stage_all(state);
                state.add_notification("Staging all files...".to_string(), NotificationLevel::Info);
                EventResult::Handled
            }
            KeyCode::Char('u') => {
                // 取消暂存全部
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
        Some(self.cursor)
    }

    fn set_selected_index(&mut self, index: Option<usize>) {
        if let Some(idx) = index {
            if idx < self.visible_rows.len() {
                self.cursor = idx;
                self.list_state.select(Some(idx));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hunks_empty() {
        let hunks = parse_hunks("");
        assert!(hunks.is_empty());
    }

    #[test]
    fn test_parse_hunks_single() {
        let diff = "@@ -1,3 +1,4 @@\n fn main() {\n+    println!(\"hello\");\n }";
        let hunks = parse_hunks(diff);
        assert_eq!(hunks.len(), 1);
        assert_eq!(hunks[0].additions, 1);
        assert_eq!(hunks[0].deletions, 0);
        assert_eq!(hunks[0].lines.len(), 3);
    }

    #[test]
    fn test_parse_hunks_multiple() {
        let diff = "\
diff --git a/foo.rs b/foo.rs
index abc..def 100644
--- a/foo.rs
+++ b/foo.rs
@@ -1,3 +1,4 @@
 fn main() {
+    println!(\"hello\");
 }
@@ -10,3 +11,3 @@
-fn old() {}
+fn new() {}
 // end";

        let hunks = parse_hunks(diff);
        assert_eq!(hunks.len(), 2);
        assert_eq!(hunks[0].additions, 1);
        assert_eq!(hunks[0].deletions, 0);
        assert_eq!(hunks[1].additions, 1);
        assert_eq!(hunks[1].deletions, 1);
    }

    #[test]
    fn test_hunk_to_patch() {
        let hunk = DiffHunk {
            header: "@@ -1,3 +1,4 @@".to_string(),
            lines: vec![
                " fn main() {".to_string(),
                "+    println!(\"hello\");".to_string(),
                " }".to_string(),
            ],
            additions: 1,
            deletions: 0,
        };

        let patch = hunk.to_patch("src/main.rs");
        assert!(patch.contains("--- a/src/main.rs"));
        assert!(patch.contains("+++ b/src/main.rs"));
        assert!(patch.contains("@@ -1,3 +1,4 @@"));
    }

    #[test]
    fn test_hunk_summary() {
        let hunk = DiffHunk {
            header: "@@ -1,3 +1,4 @@".to_string(),
            lines: vec![],
            additions: 3,
            deletions: 1,
        };
        let summary = hunk.summary();
        assert!(summary.contains("+3 -1"));
    }

    #[test]
    fn test_staging_view_new() {
        let view = StagingView::new();
        assert!(view.files.is_empty());
        assert!(view.visible_rows.is_empty());
        assert_eq!(view.cursor, 0);
    }

    #[test]
    fn test_rebuild_visible_rows() {
        let mut view = StagingView::new();
        view.files.push(StagingEntry {
            path: PathBuf::from("foo.rs"),
            change_type: ChangeType::Modified,
            is_staged: false,
            additions: 0,
            deletions: 0,
            expanded: false,
            hunks: vec![
                DiffHunk {
                    header: "@@ -1 +1 @@".to_string(),
                    lines: vec![],
                    additions: 1,
                    deletions: 0,
                },
            ],
        });
        view.files.push(StagingEntry {
            path: PathBuf::from("bar.rs"),
            change_type: ChangeType::Added,
            is_staged: true,
            additions: 0,
            deletions: 0,
            expanded: false,
            hunks: vec![],
        });

        view.rebuild_visible_rows();
        assert_eq!(view.visible_rows.len(), 2); // 2 files, none expanded

        // Expand first file
        view.files[0].expanded = true;
        view.rebuild_visible_rows();
        assert_eq!(view.visible_rows.len(), 3); // file + 1 hunk + file
    }

    #[test]
    fn test_staging_count() {
        let mut view = StagingView::new();
        view.files.push(StagingEntry {
            path: PathBuf::from("a.rs"),
            change_type: ChangeType::Modified,
            is_staged: true,
            additions: 0,
            deletions: 0,
            expanded: false,
            hunks: vec![],
        });
        view.files.push(StagingEntry {
            path: PathBuf::from("b.rs"),
            change_type: ChangeType::Modified,
            is_staged: false,
            additions: 0,
            deletions: 0,
            expanded: false,
            hunks: vec![],
        });

        assert_eq!(view.staged_count(), 1);
        assert_eq!(view.total_count(), 2);
        assert!(view.has_staged_files());
    }
}
