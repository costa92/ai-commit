use anyhow::Result;
use chrono::{DateTime, Local};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, Borders, List, ListItem, ListState, Paragraph, Scrollbar, ScrollbarOrientation,
    },
    Frame,
};
use tokio::process::Command;

/// Diff 文件信息
#[derive(Clone, Debug)]
pub struct DiffFile {
    pub path: String,
    pub additions: usize,
    pub deletions: usize,
    pub modified: bool,
}

/// Diff 查看器状态
#[derive(Clone)]
pub struct DiffViewer {
    /// 当前提交哈希
    pub commit_hash: String,
    /// 提交信息
    pub commit_info: CommitInfo,
    /// 文件列表
    pub files: Vec<DiffFile>,
    /// 当前选中的文件索引
    pub selected_file: usize,
    /// 文件列表状态
    pub file_list_state: ListState,
    /// 当前文件的 diff 内容
    pub current_diff: String,
    /// diff 滚动位置
    pub diff_scroll: u16,
    /// 显示模式
    pub view_mode: DiffViewMode,
    /// 语法高亮
    pub syntax_highlight: bool,
    /// 搜索模式
    pub search_mode: bool,
    /// 搜索关键词
    pub search_term: String,
    /// 是否显示文件列表
    pub show_file_list: bool,
    /// 当前文件的修改块列表
    pub hunks: Vec<DiffHunk>,
    /// 当前选中的修改块索引
    pub current_hunk: usize,
}

/// Diff 修改块（hunk）
#[derive(Clone, Debug)]
pub struct DiffHunk {
    /// 在diff内容中的起始行号（从0开始）
    pub start_line: usize,
    /// 在diff内容中的结束行号
    pub end_line: usize,
    /// 原文件起始行号
    pub old_start: u32,
    /// 原文件行数
    pub old_lines: u32,
    /// 新文件起始行号
    pub new_start: u32,
    /// 新文件行数
    pub new_lines: u32,
    /// 修改块的头部信息（如：@@ -1,4 +1,4 @@）
    pub header: String,
}

/// 提交信息
#[derive(Clone, Debug)]
pub struct CommitInfo {
    pub hash: String,
    pub author: String,
    pub date: DateTime<Local>,
    pub message: String,
}

/// 查看模式
#[derive(Clone, Debug, PartialEq)]
pub enum DiffViewMode {
    Split,      // 分屏查看
    Unified,    // 统一查看
    SideBySide, // 并排查看
}

/// Diff 行类型
#[derive(Clone, Debug, PartialEq)]
pub enum DiffLineType {
    Added,
    Removed,
    Context,
    Header,
}

/// Diff 行
#[derive(Clone, Debug)]
pub struct DiffLine {
    pub line_type: DiffLineType,
    pub content: String,
    pub old_line_num: Option<usize>,
    pub new_line_num: Option<usize>,
}

impl DiffViewer {
    /// 创建新的 Diff 查看器
    pub async fn new(commit_hash: &str) -> Result<Self> {
        // 首先验证提交是否存在
        let commit_exists = Command::new("git")
            .args(["rev-parse", "--verify", commit_hash])
            .output()
            .await?;

        if !commit_exists.status.success() {
            return Err(anyhow::anyhow!("Commit {} does not exist", commit_hash));
        }

        let commit_info = Self::load_commit_info(commit_hash).await.map_err(|e| {
            anyhow::anyhow!("Failed to load commit info for {}: {}", commit_hash, e)
        })?;
        let files = Self::load_diff_files(commit_hash)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to load diff files for {}: {}", commit_hash, e))?;

        let mut file_list_state = ListState::default();
        if !files.is_empty() {
            file_list_state.select(Some(0));
        }

        let current_diff = if !files.is_empty() {
            Self::load_file_diff(commit_hash, &files[0].path)
                .await
                .unwrap_or_else(|e| format!("Failed to load diff: {}", e))
        } else {
            // 如果没有文件，尝试获取完整的提交diff
            Self::load_commit_diff(commit_hash)
                .await
                .unwrap_or_else(|e| format!("No files changed in this commit. Error: {}", e))
        };

        let mut viewer = Self {
            commit_hash: commit_hash.to_string(),
            commit_info,
            files,
            selected_file: 0,
            file_list_state,
            current_diff,
            diff_scroll: 0,
            view_mode: DiffViewMode::SideBySide, // 默认使用左右对比视图
            syntax_highlight: true,
            search_mode: false,
            search_term: String::new(),
            show_file_list: true, // 默认显示文件列表
            hunks: Vec::new(),
            current_hunk: 0,
        };

        // 解析当前文件的修改块
        viewer.parse_hunks();

        Ok(viewer)
    }

    /// 加载提交信息
    async fn load_commit_info(commit_hash: &str) -> Result<CommitInfo> {
        let output = Command::new("git")
            .args(["show", "--no-patch", "--format=%H╬%an╬%ai╬%s", commit_hash])
            .output()
            .await?;

        let info = String::from_utf8_lossy(&output.stdout);
        let parts: Vec<&str> = info.trim().split('╬').collect();

        if parts.len() >= 4 {
            let hash = parts[0].to_string();
            let author = parts[1].to_string();
            let date =
                DateTime::parse_from_str(parts[2], "%Y-%m-%d %H:%M:%S %z")?.with_timezone(&Local);
            let message = parts[3].to_string();

            Ok(CommitInfo {
                hash,
                author,
                date,
                message,
            })
        } else {
            anyhow::bail!("Failed to parse commit info")
        }
    }

    /// 加载 diff 文件列表
    async fn load_diff_files(commit_hash: &str) -> Result<Vec<DiffFile>> {
        // 使用更可靠的 git 命令来获取文件变更
        let output = Command::new("git")
            .args(["show", "--name-status", "--format=", commit_hash])
            .output()
            .await?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Git command failed: {}", error_msg));
        }

        let mut files = Vec::new();
        let file_list = String::from_utf8_lossy(&output.stdout);

        for line in file_list.lines() {
            if line.is_empty() {
                continue;
            }

            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() >= 2 {
                let status = parts[0];
                let path = parts[1].to_string();

                // 获取文件的增删行数
                let stats_output = Command::new("git")
                    .args([
                        "diff",
                        "--numstat",
                        &format!("{}^", commit_hash),
                        commit_hash,
                        "--",
                        &path,
                    ])
                    .output()
                    .await?;

                let stats = String::from_utf8_lossy(&stats_output.stdout);
                let (additions, deletions) = if let Some(line) = stats.lines().next() {
                    let parts: Vec<&str> = line.split('\t').collect();
                    if parts.len() >= 2 {
                        (parts[0].parse().unwrap_or(0), parts[1].parse().unwrap_or(0))
                    } else {
                        (0, 0)
                    }
                } else {
                    (0, 0)
                };

                files.push(DiffFile {
                    path,
                    additions,
                    deletions,
                    modified: status == "M",
                });
            }
        }

        Ok(files)
    }

    /// 加载单个文件的 diff
    async fn load_file_diff(commit_hash: &str, file_path: &str) -> Result<String> {
        let output = Command::new("git")
            .args(["show", &format!("{}:{}", commit_hash, file_path)])
            .output()
            .await;

        match output {
            Ok(result) if result.status.success() => {
                // 如果可以显示文件内容，则获取完整的diff
                let diff_output = Command::new("git")
                    .args(["show", commit_hash, "--", file_path])
                    .output()
                    .await?;

                if diff_output.status.success() {
                    Ok(String::from_utf8_lossy(&diff_output.stdout).to_string())
                } else {
                    Ok(format!("Could not load diff for file: {}", file_path))
                }
            }
            _ => {
                // 如果文件不存在，可能是新增或删除的文件
                let diff_output = Command::new("git")
                    .args(["show", commit_hash, "--", file_path])
                    .output()
                    .await?;

                Ok(String::from_utf8_lossy(&diff_output.stdout).to_string())
            }
        }
    }

    /// 加载完整提交的 diff
    async fn load_commit_diff(commit_hash: &str) -> Result<String> {
        let output = Command::new("git")
            .args(["show", commit_hash])
            .output()
            .await?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            Err(anyhow::anyhow!("Failed to load commit diff: {}", error_msg))
        }
    }

    /// 选择下一个文件
    pub fn next_file(&mut self) {
        if self.files.is_empty() {
            return;
        }

        self.selected_file = (self.selected_file + 1) % self.files.len();
        self.file_list_state.select(Some(self.selected_file));
        self.diff_scroll = 0;
    }

    /// 选择上一个文件
    pub fn prev_file(&mut self) {
        if self.files.is_empty() {
            return;
        }

        if self.selected_file == 0 {
            self.selected_file = self.files.len() - 1;
        } else {
            self.selected_file -= 1;
        }
        self.file_list_state.select(Some(self.selected_file));
        self.diff_scroll = 0;
    }

    /// 加载当前选中文件的 diff
    pub async fn load_current_file_diff(&mut self) {
        if let Some(file) = self.files.get(self.selected_file) {
            match Self::load_file_diff(&self.commit_hash, &file.path).await {
                Ok(diff) => {
                    self.current_diff = diff;
                    self.parse_hunks();
                    self.current_hunk = 0;
                }
                Err(e) => self.current_diff = format!("Error loading diff: {}", e),
            }
        }
    }

    /// 切换视图模式
    pub fn toggle_view_mode(&mut self) {
        self.view_mode = match self.view_mode {
            DiffViewMode::Unified => DiffViewMode::SideBySide,
            DiffViewMode::SideBySide => DiffViewMode::Split,
            DiffViewMode::Split => DiffViewMode::Unified,
        };
    }

    /// 设置视图模式
    pub fn set_view_mode(&mut self, mode: DiffViewMode) {
        self.view_mode = mode;
    }

    /// 切换文件列表显示
    pub fn toggle_file_list(&mut self) {
        self.show_file_list = !self.show_file_list;
    }

    /// 解析 diff 内容为行列表
    pub fn parse_diff_lines(&self) -> (Vec<DiffLine>, Vec<DiffLine>) {
        let mut old_lines = Vec::new();
        let mut new_lines = Vec::new();
        let mut old_line_num = 0;
        let mut new_line_num = 0;
        let mut in_header = true;

        for line in self.current_diff.lines() {
            if line.starts_with("@@") {
                // 解析行号信息，例如 @@ -10,5 +10,8 @@
                if let Some(captures) = parse_hunk_header(line) {
                    old_line_num = captures.0;
                    new_line_num = captures.1;
                }
                in_header = false;
                // 添加 header 行到两边
                old_lines.push(DiffLine {
                    line_type: DiffLineType::Header,
                    content: line.to_string(),
                    old_line_num: None,
                    new_line_num: None,
                });
                new_lines.push(DiffLine {
                    line_type: DiffLineType::Header,
                    content: line.to_string(),
                    old_line_num: None,
                    new_line_num: None,
                });
            } else if line.starts_with("diff --git")
                || line.starts_with("index ")
                || line.starts_with("---")
                || line.starts_with("+++")
            {
                // 文件头信息，添加到两边
                old_lines.push(DiffLine {
                    line_type: DiffLineType::Header,
                    content: line.to_string(),
                    old_line_num: None,
                    new_line_num: None,
                });
                new_lines.push(DiffLine {
                    line_type: DiffLineType::Header,
                    content: line.to_string(),
                    old_line_num: None,
                    new_line_num: None,
                });
            } else if line.starts_with("-") && !line.starts_with("---") && !in_header {
                // 删除的行只显示在左边
                old_lines.push(DiffLine {
                    line_type: DiffLineType::Removed,
                    content: line[1..].to_string(),
                    old_line_num: Some(old_line_num),
                    new_line_num: None,
                });
                // 右边添加空行占位
                new_lines.push(DiffLine {
                    line_type: DiffLineType::Context,
                    content: String::new(),
                    old_line_num: None,
                    new_line_num: None,
                });
                old_line_num += 1;
            } else if line.starts_with("+") && !line.starts_with("+++") && !in_header {
                // 添加的行只显示在右边
                new_lines.push(DiffLine {
                    line_type: DiffLineType::Added,
                    content: line[1..].to_string(),
                    old_line_num: None,
                    new_line_num: Some(new_line_num),
                });
                // 左边添加空行占位
                old_lines.push(DiffLine {
                    line_type: DiffLineType::Context,
                    content: String::new(),
                    old_line_num: None,
                    new_line_num: None,
                });
                new_line_num += 1;
            } else if !in_header && !line.is_empty() {
                // 上下文行显示在两边
                let content = if line.starts_with(" ") {
                    line[1..].to_string()
                } else {
                    line.to_string()
                };
                old_lines.push(DiffLine {
                    line_type: DiffLineType::Context,
                    content: content.clone(),
                    old_line_num: Some(old_line_num),
                    new_line_num: None,
                });
                new_lines.push(DiffLine {
                    line_type: DiffLineType::Context,
                    content,
                    old_line_num: None,
                    new_line_num: Some(new_line_num),
                });
                old_line_num += 1;
                new_line_num += 1;
            }
        }

        (old_lines, new_lines)
    }

    /// 解析当前 diff 内容中的修改块 (hunks)
    fn parse_hunks(&mut self) {
        self.hunks.clear();
        let lines: Vec<&str> = self.current_diff.lines().collect();
        let mut current_line = 0;

        while current_line < lines.len() {
            let line = lines[current_line];

            // 找到 hunk header (以 @@ 开头)
            if line.starts_with("@@") {
                let start_line = current_line;

                // 解析 hunk header 获取行号信息
                let (old_start, new_start, old_lines, new_lines) =
                    if let Some((old_start, new_start)) = parse_hunk_header(line) {
                        // 尝试解析行数信息
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() >= 2 {
                            let old_info = &parts[1]; // -old_start,old_lines
                            let new_info = &parts[2]; // +new_start,new_lines

                            let old_count = if let Some(comma_pos) = old_info.find(',') {
                                old_info[comma_pos + 1..].parse().unwrap_or(1)
                            } else {
                                1
                            };

                            let new_count = if let Some(comma_pos) = new_info.find(',') {
                                new_info[comma_pos + 1..].parse().unwrap_or(1)
                            } else {
                                1
                            };

                            (old_start as u32, new_start as u32, old_count, new_count)
                        } else {
                            (0, 0, 1, 1)
                        }
                    } else {
                        (0, 0, 1, 1)
                    };

                // 寻找这个 hunk 的结束位置
                let mut end_line = current_line + 1;
                let mut processed_old = 0;
                let mut processed_new = 0;

                while end_line < lines.len()
                    && processed_old < old_lines
                    && processed_new < new_lines
                {
                    let content_line = lines[end_line];

                    // 如果遇到下一个 hunk header，停止
                    if content_line.starts_with("@@") {
                        break;
                    }

                    // 统计处理的行数
                    if content_line.starts_with("-") && !content_line.starts_with("---") {
                        processed_old += 1;
                    } else if content_line.starts_with("+") && !content_line.starts_with("+++") {
                        processed_new += 1;
                    } else if !content_line.starts_with("diff ")
                        && !content_line.starts_with("index ")
                        && !content_line.starts_with("---")
                        && !content_line.starts_with("+++")
                    {
                        // 上下文行，两边都计数
                        processed_old += 1;
                        processed_new += 1;
                    }

                    end_line += 1;
                }

                // 创建 hunk 对象
                let hunk = DiffHunk {
                    start_line,
                    end_line: end_line - 1,
                    old_start,
                    old_lines,
                    new_start,
                    new_lines,
                    header: line.to_string(),
                };

                self.hunks.push(hunk);
                current_line = end_line;
            } else {
                current_line += 1;
            }
        }

        // 如果没有找到任何 hunk，创建一个默认的包含整个 diff 的 hunk
        if self.hunks.is_empty() && !self.current_diff.is_empty() {
            self.hunks.push(DiffHunk {
                start_line: 0,
                end_line: lines.len().saturating_sub(1),
                old_start: 1,
                old_lines: lines.len() as u32,
                new_start: 1,
                new_lines: lines.len() as u32,
                header: "Complete diff".to_string(),
            });
        }
    }

    /// 跳转到下一个修改块 (hunk)
    pub fn next_hunk(&mut self) {
        if !self.hunks.is_empty() {
            self.current_hunk = (self.current_hunk + 1) % self.hunks.len();
            self.scroll_to_current_hunk();
        }
    }

    /// 跳转到上一个修改块 (hunk)  
    pub fn prev_hunk(&mut self) {
        if !self.hunks.is_empty() {
            if self.current_hunk == 0 {
                self.current_hunk = self.hunks.len() - 1;
            } else {
                self.current_hunk -= 1;
            }
            self.scroll_to_current_hunk();
        }
    }

    /// 滚动到当前选中的修改块
    fn scroll_to_current_hunk(&mut self) {
        if let Some(hunk) = self.hunks.get(self.current_hunk) {
            // 将当前 hunk 滚动到视图中央
            self.diff_scroll = hunk.start_line.saturating_sub(5) as u16;
        }
    }

    /// 获取当前修改块信息（用于状态栏显示）
    pub fn current_hunk_info(&self) -> String {
        if self.hunks.is_empty() {
            "No hunks".to_string()
        } else {
            format!("Hunk {}/{}", self.current_hunk + 1, self.hunks.len())
        }
    }
}

/// 解析 hunk header 获取行号
fn parse_hunk_header(line: &str) -> Option<(usize, usize)> {
    // 解析 @@ -10,5 +10,8 @@ 格式
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() >= 3 {
        let old_part = parts[1].trim_start_matches('-');
        let new_part = parts[2].trim_start_matches('+');

        let old_line = old_part.split(',').next()?.parse().ok()?;
        let new_line = new_part.split(',').next()?.parse().ok()?;

        Some((old_line, new_line))
    } else {
        None
    }
}

/// 渲染 Diff 查看器
pub fn render_diff_viewer(f: &mut Frame, viewer: &mut DiffViewer) {
    // 主布局：顶部信息栏 + 内容区 + 底部状态栏（3行）
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // 顶部信息
            Constraint::Min(0),    // 内容区
            Constraint::Length(3), // 状态栏（现在是3行）
        ])
        .split(f.size());

    // 渲染顶部信息
    render_commit_info(f, &viewer.commit_info, main_chunks[0]);

    // 内容区布局根据是否显示文件列表来决定
    if viewer.show_file_list {
        // 内容区分割：文件列表 + diff 内容
        let content_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(30), // 文件列表
                Constraint::Percentage(70), // diff 内容
            ])
            .split(main_chunks[1]);

        // 渲染文件列表
        render_file_list(f, viewer, content_chunks[0]);

        // 渲染 diff 内容
        render_diff_content(f, viewer, content_chunks[1]);
    } else {
        // 不显示文件列表，diff 内容占满整个区域
        render_diff_content(f, viewer, main_chunks[1]);
    }

    // 渲染状态栏
    render_status_bar(f, viewer, main_chunks[2]);
}

/// 渲染提交信息
fn render_commit_info(f: &mut Frame, info: &CommitInfo, area: Rect) {
    let text = vec![
        Line::from(vec![
            Span::raw("Commit: "),
            Span::styled(&info.hash[..8], Style::default().fg(Color::Yellow)),
            Span::raw(" | Author: "),
            Span::styled(&info.author, Style::default().fg(Color::Green)),
            Span::raw(" | Date: "),
            Span::styled(
                info.date.format("%Y-%m-%d %H:%M").to_string(),
                Style::default().fg(Color::Blue),
            ),
        ]),
        Line::from(vec![
            Span::raw("Message: "),
            Span::styled(&info.message, Style::default().fg(Color::White)),
        ]),
    ];

    let paragraph = Paragraph::new(text)
        .block(Block::default().borders(Borders::BOTTOM))
        .alignment(Alignment::Left);

    f.render_widget(paragraph, area);
}

/// 渲染文件列表
fn render_file_list(f: &mut Frame, viewer: &mut DiffViewer, area: Rect) {
    let items: Vec<ListItem> = viewer
        .files
        .iter()
        .enumerate()
        .map(|(i, file)| {
            let style = if i == viewer.selected_file {
                Style::default().bg(Color::DarkGray).fg(Color::Yellow)
            } else if file.modified {
                Style::default().fg(Color::Yellow)
            } else if file.additions > 0 && file.deletions == 0 {
                Style::default().fg(Color::Green)
            } else if file.deletions > 0 && file.additions == 0 {
                Style::default().fg(Color::Red)
            } else {
                Style::default()
            };

            let stats = format!("+{} -{}", file.additions, file.deletions);
            let content = format!("{:<40} {:>10}", file.path, stats);

            ListItem::new(content).style(style)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Files ")
            .border_style(Style::default().fg(Color::White)),
    );

    f.render_stateful_widget(list, area, &mut viewer.file_list_state);
}

/// 渲染 diff 内容
fn render_diff_content(f: &mut Frame, viewer: &DiffViewer, area: Rect) {
    match viewer.view_mode {
        DiffViewMode::Unified => render_unified_diff(f, viewer, area),
        DiffViewMode::SideBySide => render_side_by_side_diff(f, viewer, area),
        DiffViewMode::Split => render_split_diff(f, viewer, area),
    }
}

/// 渲染统一格式的 diff
fn render_unified_diff(f: &mut Frame, viewer: &DiffViewer, area: Rect) {
    let lines: Vec<Line> = viewer
        .current_diff
        .lines()
        .map(|line| {
            if line.starts_with('+') && !line.starts_with("+++") {
                Line::from(Span::styled(line, Style::default().fg(Color::Green)))
            } else if line.starts_with('-') && !line.starts_with("---") {
                Line::from(Span::styled(line, Style::default().fg(Color::Red)))
            } else if line.starts_with("@@") {
                Line::from(Span::styled(line, Style::default().fg(Color::Cyan)))
            } else if line.starts_with("diff --git") {
                Line::from(Span::styled(
                    line,
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ))
            } else {
                Line::from(line.to_string())
            }
        })
        .collect();

    // 显示当前文件名和统计信息
    let current_file = viewer
        .files
        .get(viewer.selected_file)
        .map(|f| {
            format!(
                " Diff: {} (+{} -{}) [{}/{}] ",
                f.path,
                f.additions,
                f.deletions,
                viewer.selected_file + 1,
                viewer.files.len()
            )
        })
        .unwrap_or_else(|| " Diff: No file selected ".to_string());

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(current_file)
                .border_style(Style::default().fg(Color::Yellow)),
        )
        .scroll((viewer.diff_scroll, 0));

    f.render_widget(paragraph, area);

    // 添加滚动条
    let scrollbar = Scrollbar::default()
        .orientation(ScrollbarOrientation::VerticalRight)
        .begin_symbol(Some("↑"))
        .end_symbol(Some("↓"));

    let max_scroll = viewer
        .current_diff
        .lines()
        .count()
        .saturating_sub(area.height as usize);
    if max_scroll > 0 {
        let mut scrollbar_state = ratatui::widgets::ScrollbarState::default()
            .content_length(viewer.current_diff.lines().count())
            .position(viewer.diff_scroll as usize);

        f.render_stateful_widget(
            scrollbar,
            area.inner(&ratatui::layout::Margin {
                vertical: 1,
                horizontal: 0,
            }),
            &mut scrollbar_state,
        );
    }
}

/// 渲染左右对比的 diff
fn render_side_by_side_diff(f: &mut Frame, viewer: &DiffViewer, area: Rect) {
    // 解析 diff 为左右两列
    let (old_lines, new_lines) = viewer.parse_diff_lines();

    // 分割区域为左右两部分，中间留一个字符的分隔线
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(49), // 左边（旧代码）
            Constraint::Length(2),      // 分隔线
            Constraint::Percentage(49), // 右边（新代码）
        ])
        .split(area);

    // 渲染左边（旧代码）
    render_diff_column(f, &old_lines, chunks[0], "Old", viewer.diff_scroll, true);

    // 渲染中间分隔线
    let separator = Block::default()
        .borders(Borders::LEFT | Borders::RIGHT)
        .border_style(Style::default().fg(Color::DarkGray));
    f.render_widget(separator, chunks[1]);

    // 渲染右边（新代码）
    render_diff_column(f, &new_lines, chunks[2], "New", viewer.diff_scroll, false);
}

/// 渲染单列 diff
fn render_diff_column(
    f: &mut Frame,
    lines: &[DiffLine],
    area: Rect,
    title: &str,
    scroll: u16,
    is_old: bool,
) {
    let visible_lines: Vec<Line> = lines
        .iter()
        .skip(scroll as usize)
        .take(area.height.saturating_sub(2) as usize)
        .map(|diff_line| {
            // 格式化行号和内容
            let line_num_str = if is_old {
                diff_line.old_line_num.map(|n| format!("{:4} ", n))
            } else {
                diff_line.new_line_num.map(|n| format!("{:4} ", n))
            }
            .unwrap_or_else(|| "     ".to_string());

            let style = match diff_line.line_type {
                DiffLineType::Added => Style::default().fg(Color::Green).bg(Color::Rgb(0, 50, 0)),
                DiffLineType::Removed => Style::default().fg(Color::Red).bg(Color::Rgb(50, 0, 0)),
                DiffLineType::Header => Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
                DiffLineType::Context => Style::default().fg(Color::White),
            };

            // 如果是空行（占位符），显示灰色背景
            if diff_line.content.is_empty() && diff_line.line_type == DiffLineType::Context {
                Line::from(vec![
                    Span::styled(line_num_str, Style::default().fg(Color::DarkGray)),
                    Span::styled("", Style::default().bg(Color::Rgb(30, 30, 30))),
                ])
            } else {
                Line::from(vec![
                    Span::styled(line_num_str, Style::default().fg(Color::DarkGray)),
                    Span::styled(&diff_line.content, style),
                ])
            }
        })
        .collect();

    let paragraph = Paragraph::new(visible_lines).block(
        Block::default()
            .borders(Borders::TOP | Borders::BOTTOM)
            .title(format!(" {} ", title))
            .title_alignment(Alignment::Center)
            .border_style(Style::default().fg(if is_old { Color::Red } else { Color::Green })),
    );

    f.render_widget(paragraph, area);
}

/// 渲染上下分屏的 diff
fn render_split_diff(f: &mut Frame, viewer: &DiffViewer, area: Rect) {
    // 解析 diff 为左右两列
    let (old_lines, new_lines) = viewer.parse_diff_lines();

    // 分割区域为上下两部分，中间留一行分隔线
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(49), // 上半部分（旧代码）
            Constraint::Length(1),      // 分隔线
            Constraint::Percentage(49), // 下半部分（新代码）
        ])
        .split(area);

    // 渲染上半部分（旧代码）
    render_diff_panel(
        f,
        &old_lines,
        chunks[0],
        "Old Version",
        viewer.diff_scroll,
        true,
    );

    // 渲染中间分隔线
    let separator = Block::default()
        .borders(Borders::TOP | Borders::BOTTOM)
        .border_style(Style::default().fg(Color::DarkGray));
    f.render_widget(separator, chunks[1]);

    // 渲染下半部分（新代码）
    render_diff_panel(
        f,
        &new_lines,
        chunks[2],
        "New Version",
        viewer.diff_scroll,
        false,
    );
}

/// 渲染单个 diff 面板（用于上下分屏）
fn render_diff_panel(
    f: &mut Frame,
    lines: &[DiffLine],
    area: Rect,
    title: &str,
    scroll: u16,
    is_old: bool,
) {
    let visible_lines: Vec<Line> = lines
        .iter()
        .skip(scroll as usize)
        .take(area.height.saturating_sub(2) as usize)
        .map(|diff_line| {
            // 格式化行号和内容
            let line_num_str = if is_old {
                diff_line.old_line_num.map(|n| format!("{:4} │ ", n))
            } else {
                diff_line.new_line_num.map(|n| format!("{:4} │ ", n))
            }
            .unwrap_or_else(|| "     │ ".to_string());

            let style = match diff_line.line_type {
                DiffLineType::Added => Style::default().fg(Color::Green).bg(Color::Rgb(0, 40, 0)),
                DiffLineType::Removed => Style::default().fg(Color::Red).bg(Color::Rgb(40, 0, 0)),
                DiffLineType::Header => Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
                DiffLineType::Context => Style::default().fg(Color::White),
            };

            // 如果是空行（占位符），显示虚线
            if diff_line.content.is_empty() && diff_line.line_type == DiffLineType::Context {
                Line::from(vec![
                    Span::styled(line_num_str, Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        "────────────────────────",
                        Style::default().fg(Color::DarkGray),
                    ),
                ])
            } else {
                Line::from(vec![
                    Span::styled(line_num_str, Style::default().fg(Color::DarkGray)),
                    Span::styled(&diff_line.content, style),
                ])
            }
        })
        .collect();

    let paragraph = Paragraph::new(visible_lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" {} ", title))
            .title_alignment(Alignment::Center)
            .border_style(Style::default().fg(if is_old { Color::Red } else { Color::Green })),
    );

    f.render_widget(paragraph, area);
}

/// 渲染状态栏和菜单
fn render_status_bar(f: &mut Frame, viewer: &DiffViewer, area: Rect) {
    // 分割状态栏为三行
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // 状态信息
            Constraint::Length(1), // 主菜单栏
            Constraint::Length(1), // 辅助菜单栏
        ])
        .split(area);

    // 渲染状态信息
    let mode = if viewer.search_mode {
        format!("SEARCH: {}", viewer.search_term)
    } else {
        "NORMAL".to_string()
    };

    let status = format!(
        " {} | File {}/{} | {} | {} View | Files: {} | Syntax: {} ",
        mode,
        viewer.selected_file + 1,
        viewer.files.len(),
        viewer.current_hunk_info(),
        match viewer.view_mode {
            DiffViewMode::Split => "Split",
            DiffViewMode::Unified => "Unified",
            DiffViewMode::SideBySide => "Side-by-Side",
        },
        if viewer.show_file_list { "ON" } else { "OFF" },
        if viewer.syntax_highlight { "ON" } else { "OFF" }
    );

    let status_bar =
        Paragraph::new(status).style(Style::default().fg(Color::White).bg(Color::DarkGray));

    f.render_widget(status_bar, chunks[0]);

    // 渲染主菜单栏
    render_main_menu_bar(f, chunks[1], viewer);

    // 渲染辅助菜单栏
    render_secondary_menu_bar(f, chunks[2], viewer);
}

/// 渲染主菜单栏（视图切换）
fn render_main_menu_bar(f: &mut Frame, area: Rect, viewer: &DiffViewer) {
    let view_indicator = match viewer.view_mode {
        DiffViewMode::Unified => "[1]",
        DiffViewMode::SideBySide => "[2]",
        DiffViewMode::Split => "[3]",
    };

    let menu_items = vec![
        (view_indicator, "Current", Color::Yellow),
        (
            "1",
            "Unified",
            if viewer.view_mode == DiffViewMode::Unified {
                Color::Green
            } else {
                Color::White
            },
        ),
        (
            "2",
            "Side-by-Side",
            if viewer.view_mode == DiffViewMode::SideBySide {
                Color::Green
            } else {
                Color::White
            },
        ),
        (
            "3",
            "Split",
            if viewer.view_mode == DiffViewMode::Split {
                Color::Green
            } else {
                Color::White
            },
        ),
        ("v", "Cycle", Color::Magenta),
        ("│", "", Color::DarkGray),
        ("j/↓", "Next", Color::Cyan),
        ("k/↑", "Prev", Color::Cyan),
        ("g/G", "First/Last", Color::Blue),
    ];

    let mut spans = Vec::new();
    for (i, (key, desc, color)) in menu_items.iter().enumerate() {
        if i > 0 {
            spans.push(Span::raw(" "));
        }
        if desc.is_empty() {
            spans.push(Span::styled(*key, Style::default().fg(*color)));
        } else {
            spans.push(Span::styled(
                *key,
                Style::default().fg(*color).add_modifier(Modifier::BOLD),
            ));
            if !desc.is_empty() {
                spans.push(Span::raw(" "));
                spans.push(Span::styled(*desc, Style::default().fg(Color::Gray)));
            }
        }
    }

    let menu = Paragraph::new(Line::from(spans))
        .style(Style::default().bg(Color::Black))
        .alignment(Alignment::Center);

    f.render_widget(menu, area);
}

/// 渲染辅助菜单栏（导航和其他）
fn render_secondary_menu_bar(f: &mut Frame, area: Rect, viewer: &DiffViewer) {
    let file_list_key = if viewer.show_file_list {
        ("t", "Hide Files", Color::Yellow)
    } else {
        ("t", "Show Files", Color::Green)
    };

    let menu_items = vec![
        ("J/K", "Scroll", Color::Yellow),
        ("f/PgDn", "Page↓", Color::Blue),
        ("b/PgUp", "Page↑", Color::Blue),
        ("→/←", "File", Color::Green),
        ("↑/↓", "Hunk", Color::Green),
        file_list_key,
        ("h", "Syntax", Color::Cyan),
        ("│", "", Color::DarkGray),
        ("/", "Search", Color::Magenta),
        ("n/N", "Next/Prev", Color::Magenta),
        ("│", "", Color::DarkGray),
        ("q/Esc", "Exit", Color::Red),
    ];

    let mut spans = Vec::new();
    for (i, (key, desc, color)) in menu_items.iter().enumerate() {
        if i > 0 {
            spans.push(Span::raw(" "));
        }
        if desc.is_empty() {
            spans.push(Span::styled(*key, Style::default().fg(*color)));
        } else {
            spans.push(Span::styled(
                *key,
                Style::default().fg(*color).add_modifier(Modifier::BOLD),
            ));
            if !desc.is_empty() {
                spans.push(Span::raw(" "));
                spans.push(Span::styled(*desc, Style::default().fg(Color::Gray)));
            }
        }
    }

    let menu = Paragraph::new(Line::from(spans))
        .style(Style::default().bg(Color::Black))
        .alignment(Alignment::Center);

    f.render_widget(menu, area);
}
