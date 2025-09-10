use crate::query_history::{QueryHistory, QueryHistoryEntry};
use crate::diff_viewer::DiffViewer;
use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Rect, Alignment},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame, Terminal,
};
use std::io;
use std::collections::HashSet;
use chrono::{DateTime, Local};
use tokio::process::Command;

/// 视图类型
#[derive(Clone, Debug, PartialEq)]
pub enum ViewType {
    MainMenu,           // 主菜单
    BranchList,         // 分支列表
    TagList,            // 标签列表  
    RemoteList,         // 远程仓库列表
    CommitList,         // 提交历史列表
    DiffView,           // Diff 查看
    QueryHistory,       // 查询历史
}

/// 视图上下文
#[derive(Clone, Debug)]
pub struct ViewContext {
    view_type: ViewType,
    title: String,
    #[allow(dead_code)]  // 保留用于未来功能
    context_data: Option<String>,
}

/// 视图栈
#[derive(Clone, Debug)]
pub struct ViewStack {
    stack: Vec<ViewContext>,
}

impl ViewStack {
    pub fn new() -> Self {
        Self {
            stack: vec![ViewContext {
                view_type: ViewType::MainMenu,
                title: "Repository Overview".to_string(),
                context_data: None,
            }],
        }
    }

    pub fn push(&mut self, view_type: ViewType, title: String, context: Option<String>) {
        self.stack.push(ViewContext {
            view_type,
            title,
            context_data: context,
        });
    }

    pub fn pop(&mut self) -> bool {
        if self.stack.len() > 1 {
            self.stack.pop();
            true
        } else {
            false
        }
    }

    pub fn current(&self) -> &ViewContext {
        self.stack.last().unwrap()
    }

    pub fn breadcrumb(&self) -> String {
        self.stack
            .iter()
            .map(|ctx| ctx.title.as_str())
            .collect::<Vec<_>>()
            .join(" > ")
    }

    pub fn can_go_back(&self) -> bool {
        self.stack.len() > 1
    }
}

/// Git 提交记录
#[derive(Clone, Debug)]
pub struct GitCommit {
    pub hash: String,
    pub message: String,
    pub author: String,
    pub timestamp: DateTime<Local>,
    pub refs: String,
}

/// 分支信息
#[derive(Clone, Debug)]
pub struct BranchInfo {
    pub name: String,
    pub is_current: bool,
    pub is_remote: bool,
}

/// 标签信息
#[derive(Clone, Debug)]
pub struct TagInfo {
    pub name: String,
    pub date: Option<DateTime<Local>>,
    pub message: String,
}

/// 远程仓库信息
#[derive(Clone, Debug)]
pub struct RemoteInfo {
    pub name: String,
    pub url: String,
}

/// 主菜单项
#[derive(Clone, Debug, PartialEq)]
pub enum MainMenuItem {
    Branches,
    Tags,
    Remotes,
    CurrentBranchLog,
    QueryHistory,
}

/// 应用状态
pub struct App {
    /// 视图栈
    view_stack: ViewStack,
    
    /// 主菜单
    main_menu_items: Vec<(MainMenuItem, String, String)>,
    main_menu_state: ListState,
    
    /// 分支列表
    branches: Vec<BranchInfo>,
    branch_list_state: ListState,
    
    /// 标签列表
    tags: Vec<TagInfo>,
    tag_list_state: ListState,
    
    /// 远程仓库列表
    remotes: Vec<RemoteInfo>,
    remote_list_state: ListState,
    
    /// 提交历史
    commits: Vec<GitCommit>,
    commit_list_state: ListState,
    
    /// 当前分支
    current_branch: String,
    
    /// Diff 查看器
    diff_viewer: Option<DiffViewer>,
    
    /// 查询历史
    #[allow(dead_code)]  // 保留用于未来功能
    query_history: QueryHistory,
    query_entries: Vec<QueryHistoryEntry>,
    query_list_state: ListState,
    
    /// 状态消息
    status_message: Option<String>,
    
    /// 退出标志
    should_quit: bool,
    
    /// 焦点面板 (0=左侧菜单, 1=中间提交列表, 2=右侧diff)
    focused_panel: usize,
    
    /// 当前选中的提交 diff 内容
    current_diff: Option<String>,
    
    /// Diff 滚动位置
    diff_scroll: u16,
}

impl App {
    pub async fn new() -> Result<Self> {
        // 加载 Git 数据
        let branches = Self::load_branches().await.unwrap_or_default();
        let tags = Self::load_tags().await.unwrap_or_default();
        let remotes = Self::load_remotes().await.unwrap_or_default();
        let current_branch = Self::get_current_branch().await.unwrap_or_else(|_| "main".to_string());
        
        // 查询历史
        let query_history = QueryHistory::new(1000)?;
        let query_entries: Vec<QueryHistoryEntry> = query_history.get_recent(100)
            .into_iter()
            .cloned()
            .collect();
        
        // 初始化主菜单
        let main_menu_items = vec![
            (MainMenuItem::Branches, "Branches".to_string(), format!("{} branches", branches.len())),
            (MainMenuItem::Tags, "Tags".to_string(), format!("{} tags", tags.len())),
            (MainMenuItem::Remotes, "Remotes".to_string(), format!("{} remotes", remotes.len())),
            (MainMenuItem::CurrentBranchLog, "Current Branch Log".to_string(), format!("Branch: {}", current_branch)),
            (MainMenuItem::QueryHistory, "Query History".to_string(), format!("{} queries", query_entries.len())),
        ];
        
        let mut main_menu_state = ListState::default();
        main_menu_state.select(Some(0));
        
        Ok(Self {
            view_stack: ViewStack::new(),
            main_menu_items,
            main_menu_state,
            branches,
            branch_list_state: ListState::default(),
            tags,
            tag_list_state: ListState::default(),
            remotes,
            remote_list_state: ListState::default(),
            commits: Vec::new(),
            commit_list_state: ListState::default(),
            current_branch,
            diff_viewer: None,
            query_history,
            query_entries,
            query_list_state: ListState::default(),
            status_message: None,
            should_quit: false,
            focused_panel: 0,  // 默认焦点在左侧菜单
            current_diff: None,
            diff_scroll: 0,    // 初始化滚动位置
        })
    }
    
    /// 加载分支列表
    async fn load_branches() -> Result<Vec<BranchInfo>> {
        let output = Command::new("git")
            .args(["branch", "-a", "-v"])
            .output()
            .await?;
        
        let mut branches = Vec::new();
        let branch_text = String::from_utf8_lossy(&output.stdout);
        
        for line in branch_text.lines() {
            let is_current = line.starts_with('*');
            let line = line.trim_start_matches('*').trim();
            
            if let Some(name) = line.split_whitespace().next() {
                let is_remote = name.starts_with("remotes/");
                let name = name.trim_start_matches("remotes/").to_string();
                
                branches.push(BranchInfo {
                    name,
                    is_current,
                    is_remote,
                });
            }
        }
        
        Ok(branches)
    }
    
    /// 加载标签列表
    async fn load_tags() -> Result<Vec<TagInfo>> {
        let output = Command::new("git")
            .args(["tag", "-l", "--sort=-version:refname", "--format=%(refname:short)╬%(creatordate:short)╬%(subject)"])
            .output()
            .await?;
        
        let mut tags = Vec::new();
        let tag_text = String::from_utf8_lossy(&output.stdout);
        
        for line in tag_text.lines() {
            let parts: Vec<&str> = line.split('╬').collect();
            if parts.len() >= 1 {
                let name = parts[0].to_string();
                let date = parts.get(1).and_then(|d| {
                    DateTime::parse_from_str(&format!("{} 00:00:00 +0000", d), "%Y-%m-%d %H:%M:%S %z")
                        .ok()
                        .map(|dt| dt.with_timezone(&Local))
                });
                let message = parts.get(2).unwrap_or(&"").to_string();
                
                tags.push(TagInfo {
                    name,
                    date,
                    message,
                });
            }
        }
        
        Ok(tags)
    }
    
    /// 加载远程仓库列表
    async fn load_remotes() -> Result<Vec<RemoteInfo>> {
        let output = Command::new("git")
            .args(["remote", "-v"])
            .output()
            .await?;
        
        let mut remotes = Vec::new();
        let remote_text = String::from_utf8_lossy(&output.stdout);
        let mut seen = HashSet::new();
        
        for line in remote_text.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let name = parts[0].to_string();
                let url = parts[1].to_string();
                
                if seen.insert(name.clone()) {
                    remotes.push(RemoteInfo { name, url });
                }
            }
        }
        
        Ok(remotes)
    }
    
    /// 获取当前分支
    async fn get_current_branch() -> Result<String> {
        let output = Command::new("git")
            .args(["branch", "--show-current"])
            .output()
            .await?;
        
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }
    
    /// 加载提交历史
    pub async fn load_commits(&mut self, ref_name: Option<&str>) -> Result<()> {
        let args = if let Some(ref_name) = ref_name {
            vec!["log", ref_name, "--pretty=format:%H╬%s╬%an╬%ai╬%D", "-n", "100"]
        } else {
            vec!["log", "--pretty=format:%H╬%s╬%an╬%ai╬%D", "-n", "100"]
        };
        
        let output = Command::new("git")
            .args(&args)
            .output()
            .await?;
        
        let mut commits = Vec::new();
        let log_text = String::from_utf8_lossy(&output.stdout);
        
        for line in log_text.lines() {
            let parts: Vec<&str> = line.split('╬').collect();
            if parts.len() >= 4 {
                let hash = parts[0].to_string();
                let message = parts[1].to_string();
                let author = parts[2].to_string();
                let timestamp_str = parts[3];
                let refs = parts.get(4).unwrap_or(&"").to_string();
                
                if let Ok(dt) = DateTime::parse_from_str(timestamp_str, "%Y-%m-%d %H:%M:%S %z") {
                    commits.push(GitCommit {
                        hash,
                        message,
                        author,
                        timestamp: dt.with_timezone(&Local),
                        refs,
                    });
                }
            }
        }
        
        self.commits = commits;
        if !self.commits.is_empty() {
            self.commit_list_state.select(Some(0));
        }
        
        Ok(())
    }
    
    /// 加载选中提交的 diff
    pub async fn load_commit_diff(&mut self) {
        if let Some(selected) = self.commit_list_state.selected() {
            if let Some(commit) = self.commits.get(selected) {
                let output = Command::new("git")
                    .args(["show", &commit.hash, "--color=never"])
                    .output()
                    .await;
                
                match output {
                    Ok(output) => {
                        self.current_diff = Some(String::from_utf8_lossy(&output.stdout).to_string());
                    }
                    Err(e) => {
                        self.current_diff = Some(format!("Error loading diff: {}", e));
                    }
                }
            }
        }
    }
    
    /// 处理主菜单选择
    async fn handle_main_menu_select(&mut self) {
        if let Some(selected) = self.main_menu_state.selected() {
            if let Some((item, _, _)) = self.main_menu_items.get(selected) {
                match item {
                    MainMenuItem::Branches => {
                        // 不需要切换视图，保持在主菜单
                        // 但需要确保有分支列表和提交历史
                        if !self.branches.is_empty() {
                            self.branch_list_state.select(Some(0));
                            // 加载第一个分支的提交历史
                            if let Some(branch) = self.branches.first() {
                                let branch_name = branch.name.clone();
                                self.load_commits(Some(&branch_name)).await.ok();
                                // 自动加载第一个提交的 diff
                                if !self.commits.is_empty() {
                                    self.commit_list_state.select(Some(0));
                                    self.load_commit_diff().await;
                                }
                            }
                        }
                        // 切换焦点到中间面板
                        self.focused_panel = 1;
                    }
                    MainMenuItem::Tags => {
                        // 不需要切换视图，保持在主菜单
                        if !self.tags.is_empty() {
                            self.tag_list_state.select(Some(0));
                            // 加载第一个标签的提交历史
                            if let Some(tag) = self.tags.first() {
                                let tag_name = tag.name.clone();
                                self.load_commits(Some(&tag_name)).await.ok();
                                // 自动加载第一个提交的 diff
                                if !self.commits.is_empty() {
                                    self.commit_list_state.select(Some(0));
                                    self.load_commit_diff().await;
                                }
                            }
                        }
                        // 切换焦点到中间面板
                        self.focused_panel = 1;
                    }
                    MainMenuItem::Remotes => {
                        // 远程仓库不需要显示提交，只切换焦点
                        if !self.remotes.is_empty() {
                            self.remote_list_state.select(Some(0));
                        }
                        self.focused_panel = 1;
                    }
                    MainMenuItem::CurrentBranchLog => {
                        // 加载当前分支的提交历史
                        self.load_commits(None).await.ok();
                        if !self.commits.is_empty() {
                            self.commit_list_state.select(Some(0));
                            self.load_commit_diff().await;
                        }
                        // 切换焦点到中间面板
                        self.focused_panel = 1;
                    }
                    MainMenuItem::QueryHistory => {
                        // 查询历史不需要显示提交
                        if !self.query_entries.is_empty() {
                            self.query_list_state.select(Some(0));
                        }
                        self.focused_panel = 1;
                    }
                }
            }
        }
    }
    
    
    /// 导航：向下
    async fn navigate_down(&mut self) {
        match self.focused_panel {
            0 => {
                // 左侧面板导航
                let selected_menu_item = self.main_menu_state.selected()
                    .and_then(|idx| self.main_menu_items.get(idx))
                    .map(|(item, _, _)| item.clone());
                
                match selected_menu_item {
                    Some(MainMenuItem::Branches) => {
                        // 在分支列表中导航
                        let len = self.branches.len();
                        if len > 0 {
                            let current = self.branch_list_state.selected().unwrap_or(0);
                            let new_index = (current + 1) % len;
                            self.branch_list_state.select(Some(new_index));
                            // 加载选中分支的提交历史
                            if let Some(branch) = self.branches.get(new_index) {
                                let branch_name = branch.name.clone();
                                self.load_commits(Some(&branch_name)).await.ok();
                                if !self.commits.is_empty() {
                                    self.commit_list_state.select(Some(0));
                                    self.load_commit_diff().await;
                                }
                            }
                        }
                    }
                    Some(MainMenuItem::Tags) => {
                        // 在标签列表中导航
                        let len = self.tags.len();
                        if len > 0 {
                            let current = self.tag_list_state.selected().unwrap_or(0);
                            let new_index = (current + 1) % len;
                            self.tag_list_state.select(Some(new_index));
                            // 加载选中标签的提交历史
                            if let Some(tag) = self.tags.get(new_index) {
                                let tag_name = tag.name.clone();
                                self.load_commits(Some(&tag_name)).await.ok();
                                if !self.commits.is_empty() {
                                    self.commit_list_state.select(Some(0));
                                    self.load_commit_diff().await;
                                }
                            }
                        }
                    }
                    Some(MainMenuItem::Remotes) => {
                        // 在远程仓库列表中导航
                        let len = self.remotes.len();
                        if len > 0 {
                            let current = self.remote_list_state.selected().unwrap_or(0);
                            let new_index = (current + 1) % len;
                            self.remote_list_state.select(Some(new_index));
                        }
                    }
                    Some(MainMenuItem::QueryHistory) => {
                        // 在查询历史中导航
                        let len = self.query_entries.len();
                        if len > 0 {
                            let current = self.query_list_state.selected().unwrap_or(0);
                            let new_index = (current + 1) % len;
                            self.query_list_state.select(Some(new_index));
                        }
                    }
                    _ => {
                        // 在主菜单中导航
                        let len = self.main_menu_items.len();
                        if len > 0 {
                            let current = self.main_menu_state.selected().unwrap_or(0);
                            self.main_menu_state.select(Some((current + 1) % len));
                        }
                    }
                }
            }
            1 => {
                // 中间面板导航（提交列表）
                let len = self.commits.len();
                if len > 0 {
                    let current = self.commit_list_state.selected().unwrap_or(0);
                    let new_index = (current + 1) % len;
                    self.commit_list_state.select(Some(new_index));
                    // 自动加载选中提交的 diff
                    self.load_commit_diff().await;
                }
            }
            _ => {}
        }
    }
    
    /// 导航：向上
    async fn navigate_up(&mut self) {
        match self.focused_panel {
            0 => {
                // 左侧面板导航
                let selected_menu_item = self.main_menu_state.selected()
                    .and_then(|idx| self.main_menu_items.get(idx))
                    .map(|(item, _, _)| item.clone());
                
                match selected_menu_item {
                    Some(MainMenuItem::Branches) => {
                        // 在分支列表中导航
                        let len = self.branches.len();
                        if len > 0 {
                            let current = self.branch_list_state.selected().unwrap_or(0);
                            let new_index = if current == 0 { len - 1 } else { current - 1 };
                            self.branch_list_state.select(Some(new_index));
                            // 加载选中分支的提交历史
                            if let Some(branch) = self.branches.get(new_index) {
                                let branch_name = branch.name.clone();
                                self.load_commits(Some(&branch_name)).await.ok();
                                if !self.commits.is_empty() {
                                    self.commit_list_state.select(Some(0));
                                    self.load_commit_diff().await;
                                }
                            }
                        }
                    }
                    Some(MainMenuItem::Tags) => {
                        // 在标签列表中导航
                        let len = self.tags.len();
                        if len > 0 {
                            let current = self.tag_list_state.selected().unwrap_or(0);
                            let new_index = if current == 0 { len - 1 } else { current - 1 };
                            self.tag_list_state.select(Some(new_index));
                            // 加载选中标签的提交历史
                            if let Some(tag) = self.tags.get(new_index) {
                                let tag_name = tag.name.clone();
                                self.load_commits(Some(&tag_name)).await.ok();
                                if !self.commits.is_empty() {
                                    self.commit_list_state.select(Some(0));
                                    self.load_commit_diff().await;
                                }
                            }
                        }
                    }
                    Some(MainMenuItem::Remotes) => {
                        // 在远程仓库列表中导航
                        let len = self.remotes.len();
                        if len > 0 {
                            let current = self.remote_list_state.selected().unwrap_or(0);
                            let new_index = if current == 0 { len - 1 } else { current - 1 };
                            self.remote_list_state.select(Some(new_index));
                        }
                    }
                    Some(MainMenuItem::QueryHistory) => {
                        // 在查询历史中导航
                        let len = self.query_entries.len();
                        if len > 0 {
                            let current = self.query_list_state.selected().unwrap_or(0);
                            let new_index = if current == 0 { len - 1 } else { current - 1 };
                            self.query_list_state.select(Some(new_index));
                        }
                    }
                    _ => {
                        // 在主菜单中导航
                        let len = self.main_menu_items.len();
                        if len > 0 {
                            let current = self.main_menu_state.selected().unwrap_or(0);
                            self.main_menu_state.select(Some(if current == 0 { len - 1 } else { current - 1 }));
                        }
                    }
                }
            }
            1 => {
                // 中间面板导航（提交列表）
                let len = self.commits.len();
                if len > 0 {
                    let current = self.commit_list_state.selected().unwrap_or(0);
                    let new_index = if current == 0 { len - 1 } else { current - 1 };
                    self.commit_list_state.select(Some(new_index));
                    // 自动加载选中提交的 diff
                    self.load_commit_diff().await;
                }
            }
            _ => {}
        }
    }
}

/// 运行新的 TUI
pub async fn run_hierarchical_tui() -> Result<()> {
    // 设置终端
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    
    // 创建应用
    let mut app = App::new().await?;
    
    // 主循环
    let res = run_app(&mut terminal, &mut app).await;
    
    // 恢复终端
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    
    res
}

/// 主循环
async fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;
        
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                // 特殊处理 DiffView 的键盘事件
                if app.view_stack.current().view_type == ViewType::DiffView {
                    if let Some(viewer) = &mut app.diff_viewer {
                        match key.code {
                            KeyCode::Char('q') | KeyCode::Esc | KeyCode::Backspace => {
                                // 返回上一级
                                app.view_stack.pop();
                            }
                            KeyCode::Char('j') | KeyCode::Down | KeyCode::Tab => {
                                viewer.next_file();
                            }
                            KeyCode::Char('k') | KeyCode::Up | KeyCode::BackTab => {
                                viewer.prev_file();
                            }
                            KeyCode::Char('J') => {
                                // 大写 J 用于向下滚动 diff 内容
                                viewer.diff_scroll = viewer.diff_scroll.saturating_add(1);
                            }
                            KeyCode::Char('K') => {
                                // 大写 K 用于向上滚动 diff 内容
                                viewer.diff_scroll = viewer.diff_scroll.saturating_sub(1);
                            }
                            KeyCode::PageDown | KeyCode::Char('f') => {
                                viewer.diff_scroll = viewer.diff_scroll.saturating_add(10);
                            }
                            KeyCode::PageUp | KeyCode::Char('b') => {
                                viewer.diff_scroll = viewer.diff_scroll.saturating_sub(10);
                            }
                            KeyCode::Char('v') => {
                                // 切换视图模式（循环）
                                viewer.toggle_view_mode();
                            }
                            KeyCode::Char('1') => {
                                // 统一视图模式
                                viewer.set_view_mode(crate::diff_viewer::DiffViewMode::Unified);
                            }
                            KeyCode::Char('2') => {
                                // 左右对比视图模式
                                viewer.set_view_mode(crate::diff_viewer::DiffViewMode::SideBySide);
                            }
                            KeyCode::Char('3') => {
                                // 分屏视图模式
                                viewer.set_view_mode(crate::diff_viewer::DiffViewMode::Split);
                            }
                            KeyCode::Char('h') => {
                                viewer.syntax_highlight = !viewer.syntax_highlight;
                            }
                            KeyCode::Char('t') | KeyCode::Char('T') => {
                                // t 键切换文件列表（类似 tree）
                                viewer.toggle_file_list();
                            }
                            KeyCode::Char('g') => {
                                // 跳到第一个文件
                                viewer.selected_file = 0;
                                viewer.file_list_state.select(Some(0));
                                viewer.diff_scroll = 0;
                            }
                            KeyCode::Char('G') => {
                                // 跳到最后一个文件
                                if !viewer.files.is_empty() {
                                    viewer.selected_file = viewer.files.len() - 1;
                                    viewer.file_list_state.select(Some(viewer.selected_file));
                                    viewer.diff_scroll = 0;
                                }
                            }
                            _ => {}
                        }
                        // 加载当前文件的 diff（如果切换了文件）
                        if matches!(key.code, KeyCode::Char('j') | KeyCode::Down | KeyCode::Tab |
                                             KeyCode::Char('k') | KeyCode::Up | KeyCode::BackTab |
                                             KeyCode::Char('g') | KeyCode::Char('G')) {
                            viewer.load_current_file_diff().await;
                        }
                    }
                    continue; // 跳过通用键盘处理
                }
                
                // 通用键盘处理（非 DiffView）
                match key.code {
                    KeyCode::Char('q') => {
                        if app.view_stack.stack.len() == 1 {
                            // 在主菜单按 q 退出
                            app.should_quit = true;
                        } else {
                            // 在其他视图按 q 返回
                            app.view_stack.pop();
                        }
                    }
                    KeyCode::Esc | KeyCode::Backspace => {
                        // ESC 或 Backspace 返回上一级
                        if app.focused_panel != 0 {
                            // 如果不在左侧面板，返回左侧面板
                            app.focused_panel = 0;
                        } else {
                            // 如果在左侧面板，检查是否在子列表中
                            let current_selection = app.main_menu_state.selected();
                            if let Some(selected) = current_selection {
                                if let Some((item, _, _)) = app.main_menu_items.get(selected) {
                                    match item {
                                        MainMenuItem::Branches => {
                                            // 从分支列表返回主菜单，清除分支选择
                                            app.branch_list_state.select(None);
                                            app.commits.clear();
                                            app.current_diff = None;
                                        }
                                        MainMenuItem::Tags => {
                                            // 从标签列表返回主菜单
                                            app.tag_list_state.select(None);
                                            app.commits.clear();
                                            app.current_diff = None;
                                        }
                                        MainMenuItem::Remotes => {
                                            // 从远程列表返回主菜单
                                            app.remote_list_state.select(None);
                                        }
                                        MainMenuItem::QueryHistory => {
                                            // 从查询历史返回主菜单
                                            app.query_list_state.select(None);
                                        }
                                        MainMenuItem::CurrentBranchLog => {
                                            // 从当前分支日志返回主菜单
                                            app.commits.clear();
                                            app.current_diff = None;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    KeyCode::Tab => {
                        // Tab 键切换焦点面板
                        if app.view_stack.current().view_type == ViewType::MainMenu {
                            // 循环切换焦点: 0 -> 1 -> 2 -> 0
                            app.focused_panel = (app.focused_panel + 1) % 3;
                        }
                    }
                    KeyCode::BackTab => {
                        // Shift+Tab 反向切换焦点面板
                        if app.view_stack.current().view_type == ViewType::MainMenu {
                            app.focused_panel = if app.focused_panel == 0 { 2 } else { app.focused_panel - 1 };
                        }
                    }
                    KeyCode::Char('m') => {
                        // 'm' 键强制返回主菜单
                        app.main_menu_state.select(Some(0)); // 重置到第一个菜单项
                        app.branch_list_state.select(None);
                        app.tag_list_state.select(None);
                        app.remote_list_state.select(None);
                        app.query_list_state.select(None);
                        app.commits.clear();
                        app.current_diff = None;
                        app.focused_panel = 0;
                    }
                    KeyCode::Enter => {
                        // Enter 进入下一级或执行动作
                        if app.focused_panel == 0 {
                            // 在左侧主菜单，执行选择
                            app.handle_main_menu_select().await;
                        } else if app.focused_panel == 1 {
                            // 在中间面板，切换焦点到右侧查看 diff
                            app.focused_panel = 2;
                        }
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        app.navigate_down().await;
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        app.navigate_up().await;
                    }
                    KeyCode::PageDown | KeyCode::Char('f') => {
                        // 右侧 diff 面板向下滚动
                        if app.focused_panel == 2 && app.current_diff.is_some() {
                            app.diff_scroll = app.diff_scroll.saturating_add(10);
                        }
                    }
                    KeyCode::PageUp | KeyCode::Char('b') => {
                        // 右侧 diff 面板向上滚动
                        if app.focused_panel == 2 && app.current_diff.is_some() {
                            app.diff_scroll = app.diff_scroll.saturating_sub(10);
                        }
                    }
                    _ => {}
                }
            }
        }
        
        if app.should_quit {
            return Ok(());
        }
    }
}

/// 渲染 UI
fn ui(f: &mut Frame, app: &App) {
    // 三栏布局
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // 面包屑
            Constraint::Min(0),     // 内容区
            Constraint::Length(2),  // 状态栏
        ])
        .split(f.size());
    
    // 渲染面包屑
    render_breadcrumb(f, app, main_chunks[0]);
    
    // 将内容区分为三栏
    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25), // 左侧：主菜单
            Constraint::Percentage(35), // 中间：提交列表
            Constraint::Percentage(40), // 右侧：diff 显示
        ])
        .split(main_chunks[1]);
    
    // 渲染左侧菜单（主菜单或分支/标签列表）
    render_left_panel(f, app, content_chunks[0]);
    
    // 渲染中间提交列表
    render_middle_panel(f, app, content_chunks[1]);
    
    // 渲染右侧 diff
    render_right_panel(f, app, content_chunks[2]);
    
    // 渲染状态栏
    render_status_bar(f, app, main_chunks[2]);
}

/// 渲染面包屑
fn render_breadcrumb(f: &mut Frame, app: &App, area: Rect) {
    let breadcrumb = Paragraph::new(app.view_stack.breadcrumb())
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Blue)))
        .style(Style::default().fg(Color::White));
    
    f.render_widget(breadcrumb, area);
}

/// 渲染左侧面板（主菜单或分支/标签列表）
fn render_left_panel(f: &mut Frame, app: &App, area: Rect) {
    let border_color = if app.focused_panel == 0 {
        Color::Yellow
    } else {
        Color::DarkGray
    };
    
    // 根据当前选中的主菜单项决定显示内容
    let selected_menu_item = app.main_menu_state.selected()
        .and_then(|idx| app.main_menu_items.get(idx))
        .map(|(item, _, _)| item);
    
    match selected_menu_item {
        Some(MainMenuItem::Branches) => {
            // 只显示分支列表，最大化
            let items: Vec<ListItem> = app.branches
                .iter()
                .map(|branch| {
                    let style = if branch.is_current {
                        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
                    } else if branch.is_remote {
                        Style::default().fg(Color::Blue)
                    } else {
                        Style::default().fg(Color::White)
                    };
                    
                    let prefix = if branch.is_current { "* " } else { "  " };
                    let text = format!("{}{}", prefix, branch.name);
                    ListItem::new(text).style(style)
                })
                .collect();
            
            let list = List::new(items)
                .block(Block::default()
                    .borders(Borders::ALL)
                    .title(format!(" 📁 Branches ({}) - [m]Main [j/k]Navigate [Tab]Focus ", app.branches.len()))
                    .border_style(Style::default().fg(border_color)))
                .highlight_style(Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD))
                .highlight_symbol(">> ");
            
            f.render_stateful_widget(list, area, &mut app.branch_list_state.clone());
        }
        Some(MainMenuItem::Tags) => {
            // 只显示标签列表，最大化
            let items: Vec<ListItem> = app.tags
                .iter()
                .map(|tag| {
                    let text = if let Some(date) = &tag.date {
                        format!("🏷️  {} ({})", tag.name, date.format("%Y-%m-%d"))
                    } else {
                        format!("🏷️  {}", tag.name)
                    };
                    
                    ListItem::new(text).style(Style::default().fg(Color::Yellow))
                })
                .collect();
            
            let list = List::new(items)
                .block(Block::default()
                    .borders(Borders::ALL)
                    .title(format!(" 🏷️  Tags ({}) - [m]Main [j/k]Navigate [Tab]Focus ", app.tags.len()))
                    .border_style(Style::default().fg(border_color)))
                .highlight_style(Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD))
                .highlight_symbol(">> ");
            
            f.render_stateful_widget(list, area, &mut app.tag_list_state.clone());
        }
        Some(MainMenuItem::Remotes) => {
            // 只显示远程仓库列表，最大化
            let items: Vec<ListItem> = app.remotes
                .iter()
                .map(|remote| {
                    let text = format!("🌐 {}: {}", remote.name, remote.url);
                    ListItem::new(text).style(Style::default().fg(Color::Magenta))
                })
                .collect();
            
            let list = List::new(items)
                .block(Block::default()
                    .borders(Borders::ALL)
                    .title(format!(" 🌐 Remotes ({}) - [m]Main [j/k]Navigate [Tab]Focus ", app.remotes.len()))
                    .border_style(Style::default().fg(border_color)))
                .highlight_style(Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD))
                .highlight_symbol(">> ");
            
            f.render_stateful_widget(list, area, &mut app.remote_list_state.clone());
        }
        Some(MainMenuItem::QueryHistory) => {
            // 只显示查询历史列表，最大化
            let items: Vec<ListItem> = app.query_entries
                .iter()
                .map(|entry| {
                    let status_icon = if entry.success { "✅" } else { "❌" };
                    let timestamp = entry.timestamp.format("%H:%M:%S");
                    let text = format!("{} {} {}", status_icon, timestamp, entry.query);
                    
                    let style = if entry.success {
                        Style::default().fg(Color::Green)
                    } else {
                        Style::default().fg(Color::Red)
                    };
                    
                    ListItem::new(text).style(style)
                })
                .collect();
            
            let list = List::new(items)
                .block(Block::default()
                    .borders(Borders::ALL)
                    .title(format!(" 📝 Query History ({}) - [m]Main [j/k]Navigate [Tab]Focus ", app.query_entries.len()))
                    .border_style(Style::default().fg(border_color)))
                .highlight_style(Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD))
                .highlight_symbol(">> ");
            
            f.render_stateful_widget(list, area, &mut app.query_list_state.clone());
        }
        Some(MainMenuItem::CurrentBranchLog) => {
            // 显示当前分支信息
            let info_text = format!(
                "📋 Current Branch: {}\n\n🔄 Commits loaded in middle panel\n\n💡 Use [Tab] to navigate to commits\n    [j/k] to scroll this info\n    [m] to return to main menu",
                app.current_branch
            );
            
            let paragraph = Paragraph::new(info_text)
                .block(Block::default()
                    .borders(Borders::ALL)
                    .title(" 📋 Current Branch Log - [m]Main [Tab]Focus ")
                    .border_style(Style::default().fg(border_color)))
                .style(Style::default().fg(Color::Cyan))
                .wrap(ratatui::widgets::Wrap { trim: true });
            
            f.render_widget(paragraph, area);
        }
        _ => {
            // 显示主菜单
            let items: Vec<ListItem> = app.main_menu_items
                .iter()
                .map(|(_, name, desc)| {
                    let icon = match name.as_str() {
                        "Branches" => "📁",
                        "Tags" => "🏷️",
                        "Remotes" => "🌐",
                        "Current Branch Log" => "📋",
                        "Query History" => "📝",
                        _ => "•",
                    };
                    
                    ListItem::new(Line::from(vec![
                        Span::styled(format!("{} {:<15}", icon, name), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                        Span::raw("  "),
                        Span::styled(desc, Style::default().fg(Color::Gray)),
                    ]))
                })
                .collect();
            
            let list = List::new(items)
                .block(Block::default()
                    .borders(Borders::ALL)
                    .title(" 📋 Main Menu - [Enter]Select [j/k]Navigate [q]Quit ")
                    .border_style(Style::default().fg(border_color)))
                .highlight_style(Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD))
                .highlight_symbol("▶ ");
            
            f.render_stateful_widget(list, area, &mut app.main_menu_state.clone());
        }
    }
}

/// 渲染中间面板（提交列表）
fn render_middle_panel(f: &mut Frame, app: &App, area: Rect) {
    let border_color = if app.focused_panel == 1 {
        Color::Yellow
    } else {
        Color::DarkGray
    };
    
    if app.commits.is_empty() {
        let paragraph = Paragraph::new("No commits to display\n\nSelect a branch or tag from the left panel")
            .block(Block::default()
                .borders(Borders::ALL)
                .title(" Commits ")
                .border_style(Style::default().fg(border_color)))
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);
        f.render_widget(paragraph, area);
    } else {
        let items: Vec<ListItem> = app.commits
            .iter()
            .map(|commit| {
                let short_hash = &commit.hash[..8.min(commit.hash.len())];
                let timestamp = commit.timestamp.format("%m-%d %H:%M");
                let text = Line::from(vec![
                    Span::styled(short_hash, Style::default().fg(Color::Blue)),
                    Span::raw(" "),
                    Span::styled(format!("[{}]", timestamp), Style::default().fg(Color::DarkGray)),
                    Span::raw(" "),
                    Span::styled(&commit.message, Style::default().fg(Color::White)),
                ]);
                ListItem::new(text)
            })
            .collect();
        
        let title = match app.view_stack.current().view_type {
            ViewType::BranchList => {
                if let Some(idx) = app.branch_list_state.selected() {
                    if let Some(branch) = app.branches.get(idx) {
                        format!(" Commits ({}) ", branch.name)
                    } else {
                        " Commits ".to_string()
                    }
                } else {
                    " Commits ".to_string()
                }
            }
            ViewType::TagList => {
                if let Some(idx) = app.tag_list_state.selected() {
                    if let Some(tag) = app.tags.get(idx) {
                        format!(" Commits ({}) ", tag.name)
                    } else {
                        " Commits ".to_string()
                    }
                } else {
                    " Commits ".to_string()
                }
            }
            _ => " Commits ".to_string(),
        };
        
        let list = List::new(items)
            .block(Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(Style::default().fg(border_color)))
            .highlight_style(Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD))
            .highlight_symbol("> ");
        
        f.render_stateful_widget(list, area, &mut app.commit_list_state.clone());
    }
}

/// 渲染右侧面板（diff 显示）
fn render_right_panel(f: &mut Frame, app: &App, area: Rect) {
    let border_color = if app.focused_panel == 2 {
        Color::Yellow
    } else {
        Color::DarkGray
    };
    
    if let Some(diff) = &app.current_diff {
        // 处理 diff 内容，添加颜色
        let lines: Vec<Line> = diff
            .lines()
            .skip(app.diff_scroll as usize)
            .take(area.height.saturating_sub(2) as usize)
            .map(|line| {
                if line.starts_with('+') && !line.starts_with("+++") {
                    Line::from(Span::styled(line, Style::default().fg(Color::Green)))
                } else if line.starts_with('-') && !line.starts_with("---") {
                    Line::from(Span::styled(line, Style::default().fg(Color::Red)))
                } else if line.starts_with("@@") {
                    Line::from(Span::styled(line, Style::default().fg(Color::Cyan)))
                } else if line.starts_with("diff --git") {
                    Line::from(Span::styled(line, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)))
                } else {
                    Line::from(line.to_string())
                }
            })
            .collect();
        
        let title = if let Some(selected) = app.commit_list_state.selected() {
            if let Some(commit) = app.commits.get(selected) {
                format!(" Diff ({}) [Scroll: {}] ", &commit.hash[..8.min(commit.hash.len())], app.diff_scroll)
            } else {
                format!(" Diff [Scroll: {}] ", app.diff_scroll)
            }
        } else {
            format!(" Diff [Scroll: {}] ", app.diff_scroll)
        };
        
        let paragraph = Paragraph::new(lines)
            .block(Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(Style::default().fg(border_color)));
        
        f.render_widget(paragraph, area);
    } else {
        let paragraph = Paragraph::new("No diff to display\n\nSelect a commit from the middle panel")
            .block(Block::default()
                .borders(Borders::ALL)
                .title(" Diff ")
                .border_style(Style::default().fg(border_color)))
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);
        f.render_widget(paragraph, area);
    }
}

/// 渲染主菜单
#[allow(dead_code)]  // 保留用于未来功能
fn render_main_menu(f: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app.main_menu_items
        .iter()
        .map(|(_, name, desc)| {
            ListItem::new(Line::from(vec![
                Span::styled(format!("{:<20}", name), Style::default().fg(Color::Yellow)),
                Span::raw(" - "),
                Span::styled(desc, Style::default().fg(Color::Gray)),
            ]))
        })
        .collect();
    
    let list = List::new(items)
        .block(Block::default()
            .borders(Borders::ALL)
            .title(" Main Menu ")
            .border_style(Style::default().fg(Color::Green)))
        .highlight_style(Style::default()
            .bg(Color::DarkGray)
            .add_modifier(Modifier::BOLD))
        .highlight_symbol("> ");
    
    f.render_stateful_widget(list, area, &mut app.main_menu_state.clone());
}

/// 渲染分支列表
#[allow(dead_code)]  // 保留用于未来功能
fn render_branch_list(f: &mut Frame, app: &App, area: Rect) {
    // 分割区域：左侧分支列表，右侧提交历史
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(30), // 分支列表
            Constraint::Percentage(70), // 提交历史
        ])
        .split(area);
    
    // 渲染左侧分支列表
    let items: Vec<ListItem> = app.branches
        .iter()
        .map(|branch| {
            let style = if branch.is_current {
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
            } else if branch.is_remote {
                Style::default().fg(Color::Blue)
            } else {
                Style::default().fg(Color::White)
            };
            
            let prefix = if branch.is_current { "* " } else { "  " };
            ListItem::new(format!("{}{}", prefix, branch.name)).style(style)
        })
        .collect();
    
    let list = List::new(items)
        .block(Block::default()
            .borders(Borders::ALL)
            .title(format!(" Branches ({}) ", app.branches.len()))
            .border_style(Style::default().fg(Color::Yellow)))
        .highlight_style(Style::default()
            .bg(Color::DarkGray)
            .add_modifier(Modifier::BOLD))
        .highlight_symbol(">> ");
    
    f.render_stateful_widget(list, chunks[0], &mut app.branch_list_state.clone());
    
    // 渲染右侧提交历史
    render_commit_list_panel(f, app, chunks[1]);
}

/// 渲染标签列表
#[allow(dead_code)]  // 保留用于未来功能
fn render_tag_list(f: &mut Frame, app: &App, area: Rect) {
    // 分割区域：左侧标签列表，右侧提交历史
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(30), // 标签列表
            Constraint::Percentage(70), // 提交历史
        ])
        .split(area);
    
    // 渲染左侧标签列表
    let items: Vec<ListItem> = app.tags
        .iter()
        .map(|tag| {
            let text = if let Some(date) = &tag.date {
                format!("{:<20} {} ({})", 
                    tag.name, 
                    tag.message, 
                    date.format("%Y-%m-%d"))
            } else {
                format!("{:<20} {}", tag.name, tag.message)
            };
            
            ListItem::new(text).style(Style::default().fg(Color::Yellow))
        })
        .collect();
    
    let list = List::new(items)
        .block(Block::default()
            .borders(Borders::ALL)
            .title(format!(" Tags ({}) ", app.tags.len()))
            .border_style(Style::default().fg(Color::Yellow)))
        .highlight_style(Style::default()
            .bg(Color::DarkGray)
            .add_modifier(Modifier::BOLD))
        .highlight_symbol(">> ");
    
    f.render_stateful_widget(list, chunks[0], &mut app.tag_list_state.clone());
    
    // 渲染右侧提交历史
    render_commit_list_panel(f, app, chunks[1]);
}

/// 渲染远程仓库列表
#[allow(dead_code)]  // 保留用于未来功能
fn render_remote_list(f: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app.remotes
        .iter()
        .map(|remote| {
            ListItem::new(format!("{}: {}", remote.name, remote.url))
                .style(Style::default().fg(Color::Magenta))
        })
        .collect();
    
    let list = List::new(items)
        .block(Block::default()
            .borders(Borders::ALL)
            .title(format!(" Remotes ({}) ", app.remotes.len()))
            .border_style(Style::default().fg(Color::Yellow)))
        .highlight_style(Style::default()
            .bg(Color::DarkGray)
            .add_modifier(Modifier::BOLD))
        .highlight_symbol(">> ");
    
    f.render_stateful_widget(list, area, &mut app.remote_list_state.clone());
}

/// 渲染提交列表
#[allow(dead_code)]  // 保留用于未来功能
fn render_commit_list(f: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app.commits
        .iter()
        .map(|commit| {
            let short_hash = &commit.hash[..8.min(commit.hash.len())];
            let timestamp = commit.timestamp.format("%Y-%m-%d %H:%M");
            
            ListItem::new(format!("{} {} - {} ({})", 
                short_hash,
                commit.message,
                commit.author,
                timestamp))
                .style(Style::default().fg(Color::White))
        })
        .collect();
    
    let list = List::new(items)
        .block(Block::default()
            .borders(Borders::ALL)
            .title(format!(" Commits ({}) ", app.commits.len()))
            .border_style(Style::default().fg(Color::Yellow)))
        .highlight_style(Style::default()
            .bg(Color::DarkGray)
            .add_modifier(Modifier::BOLD))
        .highlight_symbol(">> ");
    
    f.render_stateful_widget(list, area, &mut app.commit_list_state.clone());
}

/// 渲染查询历史
#[allow(dead_code)]  // 保留用于未来功能
fn render_query_history(f: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app.query_entries
        .iter()
        .map(|entry| {
            let status_icon = if entry.success { "✓" } else { "✗" };
            let timestamp = entry.timestamp.format("%Y-%m-%d %H:%M");
            
            ListItem::new(format!("{} {} - {}", 
                status_icon,
                timestamp,
                entry.query))
                .style(if entry.success {
                    Style::default().fg(Color::Green)
                } else {
                    Style::default().fg(Color::Red)
                })
        })
        .collect();
    
    let list = List::new(items)
        .block(Block::default()
            .borders(Borders::ALL)
            .title(format!(" Query History ({}) ", app.query_entries.len()))
            .border_style(Style::default().fg(Color::Yellow)))
        .highlight_style(Style::default()
            .bg(Color::DarkGray)
            .add_modifier(Modifier::BOLD))
        .highlight_symbol(">> ");
    
    f.render_stateful_widget(list, area, &mut app.query_list_state.clone());
}

/// 渲染提交列表面板（用于分支和标签视图的右侧）
#[allow(dead_code)]  // 保留用于未来功能
fn render_commit_list_panel(f: &mut Frame, app: &App, area: Rect) {
    // 获取当前选中的分支或标签名称
    let title = match app.view_stack.current().view_type {
        ViewType::BranchList => {
            if let Some(idx) = app.branch_list_state.selected() {
                if let Some(branch) = app.branches.get(idx) {
                    format!(" Commits ({}) ", branch.name)
                } else {
                    " Commits ".to_string()
                }
            } else {
                " Commits ".to_string()
            }
        }
        ViewType::TagList => {
            if let Some(idx) = app.tag_list_state.selected() {
                if let Some(tag) = app.tags.get(idx) {
                    format!(" Commits ({}) ", tag.name)
                } else {
                    " Commits ".to_string()
                }
            } else {
                " Commits ".to_string()
            }
        }
        _ => " Commits ".to_string(),
    };
    
    if app.commits.is_empty() {
        // 如果没有提交，显示提示信息
        let paragraph = Paragraph::new("No commits to display\nPress ↑/↓ to select a branch/tag")
            .block(Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(Style::default().fg(Color::DarkGray)))
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);
        f.render_widget(paragraph, area);
    } else {
        // 渲染提交列表
        let items: Vec<ListItem> = app.commits
            .iter()
            .map(|commit| {
                let short_hash = &commit.hash[..8.min(commit.hash.len())];
                let timestamp = commit.timestamp.format("%m-%d %H:%M");
                let text = format!("{} {} - {} ({})", 
                    short_hash,
                    commit.message,
                    commit.author,
                    timestamp
                );
                ListItem::new(text).style(Style::default().fg(Color::White))
            })
            .collect();
        
        let list = List::new(items)
            .block(Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(Style::default().fg(Color::Cyan)));
        
        f.render_widget(list, area);
    }
}

/// 渲染状态栏
fn render_status_bar(f: &mut Frame, app: &App, area: Rect) {
    let focus_info = match app.focused_panel {
        0 => "Left Panel",
        1 => "Middle Panel", 
        2 => "Right Panel",
        _ => "Unknown",
    };
    
    let help_text = match app.view_stack.current().view_type {
        ViewType::MainMenu => {
            // 检查是否在子列表中
            let in_submenu = app.main_menu_state.selected()
                .and_then(|idx| app.main_menu_items.get(idx))
                .map(|(item, _, _)| match item {
                    MainMenuItem::Branches => app.branch_list_state.selected().is_some(),
                    MainMenuItem::Tags => app.tag_list_state.selected().is_some(),
                    MainMenuItem::Remotes => app.remote_list_state.selected().is_some(),
                    MainMenuItem::QueryHistory => app.query_list_state.selected().is_some(),
                    _ => false,
                })
                .unwrap_or(false);
            
            if in_submenu {
                format!("[Tab] Switch Focus ({})  [j/k] Navigate  [m] Main Menu  [ESC] Back  [q] Quit", focus_info)
            } else {
                format!("[Tab] Switch Focus ({})  [Enter] Select  [j/k] Navigate  [q] Quit", focus_info)
            }
        }
        ViewType::DiffView => "[j/k] Navigate Files  [ESC] Back  [q] Back".to_string(),
        _ => {
            format!("[Tab] Switch Focus ({})  [Enter] Select  [ESC] Back  [j/k] Navigate  [q] Back/Quit", focus_info)
        }
    };
    
    let status = if let Some(msg) = &app.status_message {
        format!("{} | {}", help_text, msg)
    } else {
        help_text.to_string()
    };
    
    let status_bar = Paragraph::new(status)
        .block(Block::default()
            .borders(Borders::TOP)
            .border_style(Style::default().fg(Color::Gray)))
        .style(Style::default().fg(Color::Cyan));
    
    f.render_widget(status_bar, area);
}