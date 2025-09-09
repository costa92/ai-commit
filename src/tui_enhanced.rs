use crate::query_history::{QueryHistory, QueryHistoryEntry};
use crate::diff_viewer::{DiffViewer, render_diff_viewer};
use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap, Clear, Tabs, Scrollbar, ScrollbarOrientation},
    Frame, Terminal,
};
use std::io;
use std::collections::HashSet;
use chrono::{DateTime, Local};
use tokio::process::Command;

/// Git 提交记录
#[derive(Clone, Debug)]
pub struct GitCommit {
    pub hash: String,
    pub message: String,
    pub author: String,
    pub timestamp: DateTime<Local>,
    pub refs: String, // 分支和标签信息
}

impl GitCommit {
    /// 获取提交的 diff 内容
    pub async fn get_diff(&self) -> Result<String> {
        let output = Command::new("git")
            .args([
                "show", 
                &self.hash, 
                "--color=never",
                "--stat",           // 显示文件统计
                "--patch",          // 显示完整的差异内容
                "--abbrev-commit"   // 使用短哈希
            ])
            .output()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get git diff: {}", e))?;

        if !output.status.success() {
            anyhow::bail!("Git show command failed with exit code: {:?}", output.status.code());
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}

/// 分支信息
#[derive(Clone, Debug)]
pub struct BranchInfo {
    pub name: String,
    pub is_current: bool,
    pub is_remote: bool,
    pub commit_count: Option<usize>,
}

/// Tag 信息
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

/// 视图类型
#[derive(Clone, Debug, PartialEq)]
pub enum ViewType {
    History,            // Git 历史
    Branches,           // 分支列表
    Tags,               // 标签列表
    Remotes,            // 远程仓库列表
    Results,            // 查询结果
    QueryHistory,       // 查询历史
}


/// 标签页
pub struct Tab {
    pub name: String,
    pub view_type: ViewType,
    pub content: String,
}

/// TUI 应用状态
pub struct App {
    /// 查询历史
    history: QueryHistory,
    /// 历史记录列表
    entries: Vec<QueryHistoryEntry>,
    /// Git 提交记录列表
    pub git_commits: Vec<GitCommit>,
    /// 列表状态
    list_state: ListState,
    /// Git 提交列表状态
    pub git_list_state: ListState,
    /// 当前选中的条目索引
    selected_index: usize,
    /// 是否显示详情
    show_details: bool,
    /// 搜索过滤器
    search_filter: String,
    /// 是否在搜索模式
    search_mode: bool,
    /// 退出标志
    should_quit: bool,
    /// 要执行的查询
    execute_query: Option<String>,
    /// 显示执行结果
    execution_result: Option<String>,
    /// 显示帮助
    show_help: bool,
    /// 标签页列表
    pub tabs: Vec<Tab>,
    /// 当前标签页索引
    pub current_tab: usize,
    /// 分屏模式
    split_mode: SplitMode,
    /// 当前焦点窗口
    focused_pane: FocusedPane,
    /// 命令行模式
    command_mode: bool,
    /// 命令行输入
    command_input: String,
    /// 结果滚动位置
    result_scroll: u16,
    /// 高亮的查询语法
    syntax_highlight: bool,
    /// 当前选中提交的 diff 内容
    current_diff: Option<String>,
    /// 当前已加载 diff 的提交哈希
    diff_commit_hash: Option<String>,
    /// Diff 查看器（用于专业的 diff 显示）
    diff_viewer: Option<DiffViewer>,
    /// 是否在 diff 查看模式
    diff_view_mode: bool,
    
    // 主界面功能字段
    /// 分支列表
    pub branches: Vec<BranchInfo>,
    /// 分支列表状态
    pub branch_list_state: ListState,
    /// Tag 列表
    pub tags: Vec<TagInfo>,
    /// Tag 列表状态
    pub tag_list_state: ListState,
    /// 远程仓库列表
    pub remotes: Vec<RemoteInfo>,
    /// 远程仓库列表状态
    pub remote_list_state: ListState,
    /// 当前分支
    pub current_branch: String,
    /// 状态信息
    pub status_message: Option<String>,
    /// 是否显示左侧面板
    pub show_left_panel: bool,
}

/// 分屏模式
#[derive(Clone, Debug, PartialEq)]
pub enum SplitMode {
    None,
    Horizontal,
    Vertical,
}

/// 焦点窗口
#[derive(Clone, Debug, PartialEq)]
pub enum FocusedPane {
    Left,
    Right,
    Top,
    Bottom,
}

impl App {
    /// 创建新的应用实例
    pub async fn new() -> Result<Self> {
        let history = QueryHistory::new(1000)?;
        let entries = history.get_recent(1000)
            .into_iter()
            .map(|e| e.clone())
            .collect::<Vec<_>>();
        
        // 加载 Git 提交记录，如果失败则使用空列表
        let git_commits = match Self::load_git_commits().await {
            Ok(commits) => commits,
            Err(e) => {
                eprintln!("Warning: Failed to load git commits: {}", e);
                eprintln!("The TUI will start with an empty git log.");
                Vec::new()
            }
        };
        
        // 加载分支、标签和远程仓库信息
        let branches = Self::load_branches().await.unwrap_or_default();
        let tags = Self::load_tags().await.unwrap_or_default();
        let remotes = Self::load_remotes().await.unwrap_or_default();
        let current_branch = Self::get_current_branch().await.unwrap_or_else(|_| "main".to_string());
        
        let mut list_state = ListState::default();
        if !entries.is_empty() {
            list_state.select(Some(0));
        }

        let mut git_list_state = ListState::default();
        if !git_commits.is_empty() {
            git_list_state.select(Some(0));
        }
        
        let mut branch_list_state = ListState::default();
        if !branches.is_empty() {
            branch_list_state.select(Some(0));
        }
        
        let mut tag_list_state = ListState::default();
        if !tags.is_empty() {
            tag_list_state.select(Some(0));
        }
        
        let mut remote_list_state = ListState::default();
        if !remotes.is_empty() {
            remote_list_state.select(Some(0));
        }

        let tabs = vec![
            Tab {
                name: "Git Log".to_string(),
                view_type: ViewType::History,
                content: String::new(),
            },
            Tab {
                name: "Branches".to_string(),
                view_type: ViewType::Branches,
                content: String::new(),
            },
            Tab {
                name: "Tags".to_string(),
                view_type: ViewType::Tags,
                content: String::new(),
            },
            Tab {
                name: "Remotes".to_string(),
                view_type: ViewType::Remotes,
                content: String::new(),
            },
        ];

        Ok(Self {
            history,
            entries,
            git_commits,
            list_state,
            git_list_state,
            selected_index: 0,
            show_details: true,
            search_filter: String::new(),
            search_mode: false,
            should_quit: false,
            execute_query: None,
            execution_result: None,
            show_help: false,
            tabs,
            current_tab: 0,
            split_mode: SplitMode::None,
            focused_pane: FocusedPane::Left,
            command_mode: false,
            command_input: String::new(),
            result_scroll: 0,
            syntax_highlight: true,
            current_diff: None,
            diff_commit_hash: None,
            diff_viewer: None,
            diff_view_mode: false,
            branches,
            branch_list_state,
            tags,
            tag_list_state,
            remotes,
            remote_list_state,
            current_branch,
            status_message: None,
            show_left_panel: true,
        })
    }

    /// 加载 Git 提交记录
    async fn load_git_commits() -> Result<Vec<GitCommit>> {
        // 使用更安全的分隔符，避免与提交消息中的内容冲突
        let output = Command::new("git")
            .args([
                "log",
                "--pretty=format:%H╬%s╬%an╬%ai╬%D",  // 使用 ╬ 作为分隔符
                "-n", "100", // 限制100条记录
                "--no-merges", // 可选：排除合并提交
            ])
            .env("LANG", "C.UTF-8")  // 强制使用 UTF-8
            .env("LC_ALL", "C.UTF-8")
            .output()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get git log: {}", e))?;

        if !output.status.success() {
            anyhow::bail!("Git log command failed with exit code: {:?}", output.status.code());
        }

        let log_output = String::from_utf8_lossy(&output.stdout);
        let mut commits = Vec::new();

        // Debug: 输出原始 git log 数据
        if std::env::var("AI_COMMIT_DEBUG").is_ok() {
            eprintln!("Git log output ({} lines):", log_output.lines().count());
            for (i, line) in log_output.lines().enumerate().take(3) {
                eprintln!("  Line {}: {}", i + 1, line);
            }
        }

        for line in log_output.lines() {
            if line.trim().is_empty() {
                continue;
            }

            let parts: Vec<&str> = line.splitn(5, '╬').collect();  // 使用 ╬ 分隔符
            if parts.len() >= 4 {
                let hash = parts[0].trim().to_string();
                let message = parts[1].trim().to_string();
                let author = parts[2].trim().to_string();
                let timestamp_str = parts[3].trim();
                let refs = parts.get(4).map(|s| s.trim().to_string()).unwrap_or_default();

                // 解析时间戳 - Git %ai 格式: "2025-09-08 19:45:55 +0800"
                match DateTime::parse_from_str(timestamp_str, "%Y-%m-%d %H:%M:%S %z") {
                    Ok(dt) => {
                        let timestamp = dt.with_timezone(&Local);
                        commits.push(GitCommit {
                            hash: hash.clone(),
                            message: message.clone(),
                            author: author.clone(),
                            timestamp,
                            refs: refs.clone(),
                        });
                    }
                    Err(e) => {
                        // 如果解析失败，尝试使用当前时间作为备用
                        if std::env::var("AI_COMMIT_DEBUG").is_ok() {
                            eprintln!("Warning: Failed to parse timestamp '{}': {}", timestamp_str, e);
                            eprintln!("  Hash: {}, Message: {}", &hash[..8.min(hash.len())], message);
                        }
                        commits.push(GitCommit {
                            hash,
                            message,
                            author,
                            timestamp: Local::now(),
                            refs,
                        });
                    }
                }
            } else if std::env::var("AI_COMMIT_DEBUG").is_ok() {
                eprintln!("Warning: Skipping malformed line with {} parts: {}", parts.len(), line);
            }
        }

        // Debug: 输出最终加载的提交数量
        if std::env::var("AI_COMMIT_DEBUG").is_ok() {
            eprintln!("Successfully loaded {} git commits", commits.len());
            for (i, commit) in commits.iter().enumerate().take(3) {
                eprintln!("  Commit {}: {} - {}", i + 1, &commit.hash[..8], commit.message);
            }
        }

        Ok(commits)
    }

    /// 刷新 git 提交记录
    async fn refresh_git_commits(&mut self) {
        match Self::load_git_commits().await {
            Ok(commits) => {
                self.git_commits = commits;
                if !self.git_commits.is_empty() {
                    self.git_list_state.select(Some(0));
                    self.selected_index = 0;
                    self.load_selected_diff().await;
                }
            }
            Err(e) => {
                self.execution_result = Some(format!("Failed to refresh commits: {}", e));
            }
        }
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
                    commit_count: None,
                });
            }
        }
        
        Ok(branches)
    }
    
    /// 加载 Tag 列表（倒序）
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
                    DateTime::parse_from_str(d, "%Y-%m-%d")
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
                
                // 避免重复（fetch 和 push）
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
    
    /// 切换分支
    pub async fn checkout_branch(&mut self, branch_name: &str) -> Result<()> {
        let output = Command::new("git")
            .args(["checkout", branch_name])
            .output()
            .await?;
        
        if output.status.success() {
            self.current_branch = branch_name.to_string();
            self.status_message = Some(format!("Switched to branch '{}'", branch_name));
            // 重新加载分支列表和提交记录
            self.branches = Self::load_branches().await?;
            self.refresh_git_commits().await;
        } else {
            let error = String::from_utf8_lossy(&output.stderr);
            self.status_message = Some(format!("Failed to switch branch: {}", error));
        }
        
        Ok(())
    }
    
    /// 切换到 Tag
    pub async fn checkout_tag(&mut self, tag_name: &str) -> Result<()> {
        let output = Command::new("git")
            .args(["checkout", tag_name])
            .output()
            .await?;
        
        if output.status.success() {
            self.status_message = Some(format!("Switched to tag '{}'", tag_name));
            self.refresh_git_commits().await;
        } else {
            let error = String::from_utf8_lossy(&output.stderr);
            self.status_message = Some(format!("Failed to checkout tag: {}", error));
        }
        
        Ok(())
    }
    
    /// 拉取最新代码
    pub async fn pull(&mut self) -> Result<()> {
        let output = Command::new("git")
            .args(["pull"])
            .output()
            .await?;
        
        if output.status.success() {
            self.status_message = Some("Successfully pulled latest changes".to_string());
            // 重新加载提交历史
            self.refresh_git_commits().await;
        } else {
            let error = String::from_utf8_lossy(&output.stderr);
            self.status_message = Some(format!("Failed to pull: {}", error));
        }
        
        Ok(())
    }
    
    pub async fn load_branch_commits(&mut self) {
        if let Some(idx) = self.branch_list_state.selected() {
            if let Some(branch) = self.branches.get(idx) {
                // 加载指定分支的提交历史
                match Self::load_commits_for_ref(&branch.name).await {
                    Ok(commits) => {
                        self.git_commits = commits;
                        if !self.git_commits.is_empty() {
                            self.git_list_state.select(Some(0));
                        }
                        self.status_message = Some(format!("Loaded commits for branch: {}", branch.name));
                    }
                    Err(e) => {
                        self.status_message = Some(format!("Failed to load commits: {}", e));
                    }
                }
            }
        }
    }
    
    /// 加载选中标签的提交历史
    pub async fn load_tag_commits(&mut self) {
        if let Some(idx) = self.tag_list_state.selected() {
            if let Some(tag) = self.tags.get(idx) {
                // 加载指定标签的提交历史
                match Self::load_commits_for_ref(&tag.name).await {
                    Ok(commits) => {
                        self.git_commits = commits;
                        if !self.git_commits.is_empty() {
                            self.git_list_state.select(Some(0));
                        }
                        self.status_message = Some(format!("Loaded commits for tag: {}", tag.name));
                    }
                    Err(e) => {
                        self.status_message = Some(format!("Failed to load commits: {}", e));
                    }
                }
            }
        }
    }
    
    /// 加载指定引用（分支/标签）的提交历史
    async fn load_commits_for_ref(ref_name: &str) -> Result<Vec<GitCommit>> {
        let output = Command::new("git")
            .args([
                "log",
                ref_name,
                "--pretty=format:%H╬%s╬%an╬%ai╬%D",
                "-n", "100",
                "--no-merges",
            ])
            .env("LANG", "C.UTF-8")
            .env("LC_ALL", "C.UTF-8")
            .output()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get git log for {}: {}", ref_name, e))?;

        if !output.status.success() {
            anyhow::bail!("Git log command failed for {}", ref_name);
        }

        let log_output = String::from_utf8_lossy(&output.stdout);
        let mut commits = Vec::new();

        for line in log_output.lines() {
            if line.trim().is_empty() {
                continue;
            }

            let parts: Vec<&str> = line.split('╬').collect();
            if parts.len() >= 4 {
                let hash = parts[0].to_string();
                let message = parts[1].to_string();
                let author = parts[2].to_string();
                let timestamp_str = parts[3];
                let refs = parts.get(4).unwrap_or(&"").to_string();

                match DateTime::parse_from_str(timestamp_str, "%Y-%m-%d %H:%M:%S %z") {
                    Ok(dt) => {
                        let timestamp = dt.with_timezone(&Local);
                        commits.push(GitCommit {
                            hash: hash.clone(),
                            message: message.clone(),
                            author: author.clone(),
                            timestamp,
                            refs: refs.clone(),
                        });
                    }
                    Err(_) => {
                        commits.push(GitCommit {
                            hash,
                            message,
                            author,
                            timestamp: Local::now(),
                            refs,
                        });
                    }
                }
            }
        }

        Ok(commits)
    }

    /// 进入 diff 查看模式
    async fn enter_diff_view_mode(&mut self) {
        if self.tabs[self.current_tab].view_type == ViewType::History {
            if let Some(selected) = self.git_list_state.selected() {
                if let Some(commit) = self.git_commits.get(selected) {
                    // 创建 diff 查看器
                    match DiffViewer::new(&commit.hash).await {
                        Ok(viewer) => {
                            self.diff_viewer = Some(viewer);
                            self.diff_view_mode = true;
                        }
                        Err(e) => {
                            self.execution_result = Some(format!("Failed to create diff viewer: {}", e));
                        }
                    }
                }
            }
        }
    }
    
    /// 退出 diff 查看模式
    fn exit_diff_view_mode(&mut self) {
        self.diff_view_mode = false;
        self.diff_viewer = None;
    }
    
    /// 加载选中提交的 diff 内容
    async fn load_selected_diff(&mut self) {
        if self.tabs[self.current_tab].view_type == ViewType::History {
            if let Some(selected) = self.git_list_state.selected() {
                if let Some(commit) = self.git_commits.get(selected) {
                    // 检查是否已经加载了这个提交的 diff
                    if self.diff_commit_hash.as_ref() != Some(&commit.hash) {
                        match commit.get_diff().await {
                            Ok(diff) => {
                                // 限制 diff 的最大长度，避免内存问题
                                let max_diff_size = 50000; // 50KB
                                let truncated_diff = if diff.len() > max_diff_size {
                                    let truncated = safe_truncate(&diff, max_diff_size);
                                    format!("{}\n\n[Diff truncated, showing first ~50KB]", truncated)
                                } else {
                                    diff
                                };
                                self.current_diff = Some(truncated_diff);
                                self.diff_commit_hash = Some(commit.hash.clone());
                            }
                            Err(e) => {
                                self.current_diff = Some(format!("Failed to load diff: {}", e));
                                self.diff_commit_hash = Some(commit.hash.clone());
                            }
                        }
                    }
                }
            }
        }
    }

    /// 移动到下一个条目
    async fn next(&mut self) {
        // 根据当前标签页决定使用哪个列表
        match self.tabs[self.current_tab].view_type {
            ViewType::History => {
                if self.git_commits.is_empty() {
                    return;
                }
                let i = match self.git_list_state.selected() {
                    Some(i) => {
                        if i >= self.git_commits.len() - 1 {
                            0
                        } else {
                            i + 1
                        }
                    }
                    None => 0,
                };
                self.git_list_state.select(Some(i));
                self.selected_index = i;
            }
            ViewType::Branches => {
                if self.branches.is_empty() {
                    return;
                }
                let i = match self.branch_list_state.selected() {
                    Some(i) => {
                        if i >= self.branches.len() - 1 {
                            0
                        } else {
                            i + 1
                        }
                    }
                    None => 0,
                };
                self.branch_list_state.select(Some(i));
                // 加载选中分支的提交历史
                self.load_branch_commits().await;
            }
            ViewType::Tags => {
                if self.tags.is_empty() {
                    return;
                }
                let i = match self.tag_list_state.selected() {
                    Some(i) => {
                        if i >= self.tags.len() - 1 {
                            0
                        } else {
                            i + 1
                        }
                    }
                    None => 0,
                };
                self.tag_list_state.select(Some(i));
                // 加载选中标签的提交历史
                self.load_tag_commits().await;
            }
            ViewType::Remotes => {
                if self.remotes.is_empty() {
                    return;
                }
                let i = match self.remote_list_state.selected() {
                    Some(i) => {
                        if i >= self.remotes.len() - 1 {
                            0
                        } else {
                            i + 1
                        }
                    }
                    None => 0,
                };
                self.remote_list_state.select(Some(i));
            }
            _ => {
                if self.entries.is_empty() {
                    return;
                }
                let i = match self.list_state.selected() {
                    Some(i) => {
                        if i >= self.entries.len() - 1 {
                            0
                        } else {
                            i + 1
                        }
                    }
                    None => 0,
                };
                self.list_state.select(Some(i));
                self.selected_index = i;
            }
        }
    }

    /// 移动到上一个条目
    async fn previous(&mut self) {
        // 根据当前标签页决定使用哪个列表
        match self.tabs[self.current_tab].view_type {
            ViewType::History => {
                if self.git_commits.is_empty() {
                    return;
                }
                let i = match self.git_list_state.selected() {
                    Some(i) => {
                        if i == 0 {
                            self.git_commits.len() - 1
                        } else {
                            i - 1
                        }
                    }
                    None => 0,
                };
                self.git_list_state.select(Some(i));
                self.selected_index = i;
            }
            ViewType::Branches => {
                if self.branches.is_empty() {
                    return;
                }
                let i = match self.branch_list_state.selected() {
                    Some(i) => {
                        if i == 0 {
                            self.branches.len() - 1
                        } else {
                            i - 1
                        }
                    }
                    None => 0,
                };
                self.branch_list_state.select(Some(i));
                // 加载选中分支的提交历史
                self.load_branch_commits().await;
            }
            ViewType::Tags => {
                if self.tags.is_empty() {
                    return;
                }
                let i = match self.tag_list_state.selected() {
                    Some(i) => {
                        if i == 0 {
                            self.tags.len() - 1
                        } else {
                            i - 1
                        }
                    }
                    None => 0,
                };
                self.tag_list_state.select(Some(i));
                // 加载选中标签的提交历史
                self.load_tag_commits().await;
            }
            ViewType::Remotes => {
                if self.remotes.is_empty() {
                    return;
                }
                let i = match self.remote_list_state.selected() {
                    Some(i) => {
                        if i == 0 {
                            self.remotes.len() - 1
                        } else {
                            i - 1
                        }
                    }
                    None => 0,
                };
                self.remote_list_state.select(Some(i));
            }
            _ => {
                if self.entries.is_empty() {
                    return;
                }
                let i = match self.list_state.selected() {
                    Some(i) => {
                        if i == 0 {
                            self.entries.len() - 1
                        } else {
                            i - 1
                        }
                    }
                    None => 0,
                };
                self.list_state.select(Some(i));
                self.selected_index = i;
            }
        }
    }

    /// 跳转到第一个条目
    fn first(&mut self) {
        if self.tabs[self.current_tab].view_type == ViewType::History {
            if !self.git_commits.is_empty() {
                self.git_list_state.select(Some(0));
                self.selected_index = 0;
            }
        } else {
            if !self.entries.is_empty() {
                self.list_state.select(Some(0));
                self.selected_index = 0;
            }
        }
    }

    /// 跳转到最后一个条目
    fn last(&mut self) {
        if self.tabs[self.current_tab].view_type == ViewType::History {
            if !self.git_commits.is_empty() {
                let last_index = self.git_commits.len() - 1;
                self.git_list_state.select(Some(last_index));
                self.selected_index = last_index;
            }
        } else {
            if !self.entries.is_empty() {
                let last_index = self.entries.len() - 1;
                self.list_state.select(Some(last_index));
                self.selected_index = last_index;
            }
        }
    }

    /// 向下翻页
    fn page_down(&mut self) {
        let page_size = 10;
        if self.tabs[self.current_tab].view_type == ViewType::History {
            if !self.git_commits.is_empty() {
                let current = self.git_list_state.selected().unwrap_or(0);
                let new_index = std::cmp::min(current + page_size, self.git_commits.len() - 1);
                self.git_list_state.select(Some(new_index));
                self.selected_index = new_index;
            }
        } else {
            if !self.entries.is_empty() {
                let current = self.list_state.selected().unwrap_or(0);
                let new_index = std::cmp::min(current + page_size, self.entries.len() - 1);
                self.list_state.select(Some(new_index));
                self.selected_index = new_index;
            }
        }
    }

    /// 向上翻页
    fn page_up(&mut self) {
        let page_size = 10;
        if self.tabs[self.current_tab].view_type == ViewType::History {
            if !self.git_commits.is_empty() {
                let current = self.git_list_state.selected().unwrap_or(0);
                let new_index = current.saturating_sub(page_size);
                self.git_list_state.select(Some(new_index));
                self.selected_index = new_index;
            }
        } else {
            if !self.entries.is_empty() {
                let current = self.list_state.selected().unwrap_or(0);
                let new_index = current.saturating_sub(page_size);
                self.list_state.select(Some(new_index));
                self.selected_index = new_index;
            }
        }
    }

    /// 切换到下一个标签页
    fn next_tab(&mut self) {
        if !self.tabs.is_empty() {
            self.current_tab = (self.current_tab + 1) % self.tabs.len();
        }
    }

    /// 切换到上一个标签页
    fn prev_tab(&mut self) {
        if !self.tabs.is_empty() {
            if self.current_tab == 0 {
                self.current_tab = self.tabs.len() - 1;
            } else {
                self.current_tab -= 1;
            }
        }
    }

    /// 切换分屏模式
    fn toggle_split(&mut self) {
        self.split_mode = match self.split_mode {
            SplitMode::None => SplitMode::Horizontal,
            SplitMode::Horizontal => SplitMode::Vertical,
            SplitMode::Vertical => SplitMode::None,
        };
    }

    /// 切换焦点窗口
    fn toggle_focus(&mut self) {
        self.focused_pane = match (&self.split_mode, &self.focused_pane) {
            (SplitMode::Horizontal, FocusedPane::Top) => FocusedPane::Bottom,
            (SplitMode::Horizontal, _) => FocusedPane::Top,
            (SplitMode::Vertical, FocusedPane::Left) => FocusedPane::Right,
            (SplitMode::Vertical, _) => FocusedPane::Left,
            _ => FocusedPane::Left,
        };
    }

    /// 执行选中的查询
    async fn execute_selected_query(&mut self) {
        if let Some(query) = self.get_selected_query() {
            self.execute_query = Some(query.clone());
            
            // 执行查询并获取结果
            use crate::config::Config;
            use crate::git::GitQuery;
            
            let _config = Config::new();
            match GitQuery::parse_query(&query) {
                Ok(filters) => {
                    match GitQuery::execute_query(&filters).await {
                        Ok(results) => {
                            let result_count = results.lines().count();
                            
                            // 创建新的结果标签页
                            let tab = Tab {
                                name: format!("Results: {}", query.chars().take(20).collect::<String>()),
                                view_type: ViewType::Results,
                                content: results.clone(),
                            };
                            
                            // 查找是否已存在相同的标签页
                            let existing = self.tabs.iter().position(|t| t.view_type == ViewType::Results);
                            if let Some(idx) = existing {
                                self.tabs[idx] = tab;
                                self.current_tab = idx;
                            } else {
                                self.tabs.push(tab);
                                self.current_tab = self.tabs.len() - 1;
                            }
                            
                            // 如果没有分屏，自动启用
                            if self.split_mode == SplitMode::None {
                                self.split_mode = SplitMode::Vertical;
                            }
                            
                            self.execution_result = Some(format!(
                                "Query executed: {} results found",
                                result_count
                            ));
                            
                            // 更新历史记录
                            let _ = self.history.add_entry(
                                query,
                                Some("execute".to_string()),
                                Some(result_count),
                                true
                            );
                        }
                        Err(e) => {
                            self.execution_result = Some(format!("Error executing query: {}", e));
                        }
                    }
                }
                Err(e) => {
                    self.execution_result = Some(format!("Error parsing query: {}", e));
                }
            }
        }
    }

    /// 获取选中的查询
    pub fn get_selected_query(&self) -> Option<String> {
        self.list_state.selected()
            .and_then(|i| self.entries.get(i))
            .map(|entry| entry.query.clone())
    }

    /// 执行命令
    fn execute_command(&mut self) {
        let parts: Vec<&str> = self.command_input.split_whitespace().collect();
        if parts.is_empty() {
            return;
        }

        match parts[0] {
            "q" | "quit" => self.should_quit = true,
            "split" => self.toggle_split(),
            "vsplit" => self.split_mode = SplitMode::Vertical,
            "hsplit" => self.split_mode = SplitMode::Horizontal,
            "tab" => {
                if parts.len() > 1 {
                    let tab = Tab {
                        name: parts[1].to_string(),
                        view_type: ViewType::History,
                        content: String::new(),
                    };
                    self.tabs.push(tab);
                }
            }
            "help" => self.show_help = true,
            _ => {}
        }

        self.command_input.clear();
        self.command_mode = false;
    }

    /// 应用搜索过滤器
    fn apply_filter(&mut self) {
        if self.search_filter.is_empty() {
            self.entries = self.history.get_recent(1000)
                .into_iter()
                .map(|e| e.clone())
                .collect();
        } else {
            self.entries = self.history.search(&self.search_filter)
                .into_iter()
                .map(|e| e.clone())
                .collect();
        }

        // 重置选择
        if !self.entries.is_empty() {
            self.list_state.select(Some(0));
            self.selected_index = 0;
        } else {
            self.list_state.select(None);
        }
    }
}

/// 运行TUI应用
pub async fn run_tui() -> Result<Option<String>> {
    // 设置 panic hook，确保终端恢复
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        // 尝试恢复终端
        let _ = disable_raw_mode();
        let _ = execute!(
            io::stdout(),
            LeaveAlternateScreen,
            DisableMouseCapture
        );
        
        // 调用原始的 panic hook
        original_hook(panic_info);
    }));
    
    // 设置终端
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // 创建应用并运行
    let mut app = App::new().await?;
    
    // Debug: 输出应用状态
    if std::env::var("AI_COMMIT_DEBUG").is_ok() {
        eprintln!("TUI: Created app with {} commits and {} tabs", 
            app.git_commits.len(), app.tabs.len());
        if !app.tabs.is_empty() {
            eprintln!("TUI: Current tab '{}' (type: {:?})", 
                app.tabs[app.current_tab].name, 
                app.tabs[app.current_tab].view_type);
        }
    }

    // 初始加载第一个提交的 diff
    app.load_selected_diff().await;
    
    let res = run_app(&mut terminal, &mut app).await;

    // 恢复终端
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    
    // 恢复原始的 panic hook
    let _ = std::panic::take_hook();

    // 返回结果
    if let Ok(()) = res {
        Ok(app.get_selected_query())
    } else {
        res.map(|_| None)
    }
}

/// 主循环
async fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                // 命令模式
                if app.command_mode {
                    match key.code {
                        KeyCode::Esc => {
                            app.command_mode = false;
                            app.command_input.clear();
                        }
                        KeyCode::Enter => {
                            app.execute_command();
                        }
                        KeyCode::Backspace => {
                            app.command_input.pop();
                        }
                        KeyCode::Char(c) => {
                            app.command_input.push(c);
                        }
                        _ => {}
                    }
                    continue;
                }

                // Diff 查看模式
                if app.diff_view_mode {
                    if let Some(viewer) = &mut app.diff_viewer {
                        match key.code {
                            KeyCode::Esc | KeyCode::Char('q') => {
                                app.exit_diff_view_mode();
                            }
                            KeyCode::Char('j') | KeyCode::Tab | KeyCode::Down => {
                                viewer.next_file();
                                viewer.load_current_file_diff().await;
                            }
                            KeyCode::Char('k') | KeyCode::BackTab | KeyCode::Up => {
                                viewer.prev_file();
                                viewer.load_current_file_diff().await;
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
                                // 分屏视图模式（预留）
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
                                viewer.load_current_file_diff().await;
                            }
                            KeyCode::Char('G') => {
                                // 跳到最后一个文件
                                if !viewer.files.is_empty() {
                                    viewer.selected_file = viewer.files.len() - 1;
                                    viewer.file_list_state.select(Some(viewer.selected_file));
                                    viewer.diff_scroll = 0;
                                    viewer.load_current_file_diff().await;
                                }
                            }
                            _ => {}
                        }
                    }
                    continue;
                }

                // 搜索模式
                if app.search_mode {
                    match key.code {
                        KeyCode::Esc => {
                            app.search_mode = false;
                            app.search_filter.clear();
                            app.apply_filter();
                        }
                        KeyCode::Enter => {
                            app.search_mode = false;
                            app.apply_filter();
                        }
                        KeyCode::Backspace => {
                            app.search_filter.pop();
                            app.apply_filter();
                        }
                        KeyCode::Char(c) => {
                            app.search_filter.push(c);
                            app.apply_filter();
                        }
                        _ => {}
                    }
                    continue;
                }

                // 帮助模式
                if app.show_help {
                    match key.code {
                        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('?') => {
                            app.show_help = false;
                        }
                        _ => {}
                    }
                    continue;
                }

                // 正常模式快捷键
                match (key.modifiers, key.code) {
                    // Ctrl组合键
                    (KeyModifiers::CONTROL, KeyCode::Char('c')) => {
                        app.should_quit = true;
                        return Ok(());
                    }
                    (KeyModifiers::CONTROL, KeyCode::Char('w')) => {
                        app.toggle_focus();
                    }
                    (KeyModifiers::CONTROL, KeyCode::Char('s')) => {
                        app.toggle_split();
                    }
                    // ESC 键返回左侧焦点或关闭 Results 视图
                    (_, KeyCode::Esc) => {
                        match app.tabs[app.current_tab].view_type {
                            ViewType::Results => {
                                // 在 Results 视图中，ESC 键关闭该标签页
                                if app.tabs.len() > 1 {
                                    // 移除当前的 Results 标签页
                                    app.tabs.retain(|t| t.view_type != ViewType::Results);
                                    // 切换到第一个标签页
                                    app.current_tab = 0;
                                }
                            }
                            ViewType::Branches | ViewType::Tags | ViewType::Remotes => {
                                if app.focused_pane == FocusedPane::Right {
                                    app.focused_pane = FocusedPane::Left;
                                }
                            }
                            _ => {}
                        }
                    }
                    // 普通按键
                    (_, KeyCode::Char('q')) => {
                        app.should_quit = true;
                        return Ok(());
                    }
                    (_, KeyCode::Char(':')) => {
                        app.command_mode = true;
                        app.command_input.clear();
                    }
                    // 数字键快速切换标签页
                    (_, KeyCode::Char('1')) => {
                        if app.tabs.len() > 0 {
                            app.current_tab = 0;
                        }
                    }
                    (_, KeyCode::Char('2')) => {
                        if app.tabs.len() > 1 {
                            app.current_tab = 1;
                        }
                    }
                    (_, KeyCode::Char('3')) => {
                        if app.tabs.len() > 2 {
                            app.current_tab = 2;
                        }
                    }
                    (_, KeyCode::Char('4')) => {
                        if app.tabs.len() > 3 {
                            app.current_tab = 3;
                        }
                    }
                    (_, KeyCode::Tab) => {
                        if key.modifiers.contains(KeyModifiers::SHIFT) {
                            app.prev_tab();
                        } else {
                            app.next_tab();
                        }
                    }
                    (_, KeyCode::Enter) | (_, KeyCode::Char('x')) => {
                        // 根据当前标签页类型执行不同的操作
                        match app.tabs[app.current_tab].view_type {
                            ViewType::History => {
                                // 在 Git 日志视图中，Enter 键进入 diff 查看模式
                                app.enter_diff_view_mode().await;
                            }
                            ViewType::Branches | ViewType::Tags | ViewType::Remotes => {
                                // 在分支/标签/远程视图中，根据焦点位置决定行为
                                if app.focused_pane == FocusedPane::Left {
                                    // 如果焦点在左侧，切换到右侧提交列表
                                    if app.split_mode == SplitMode::None {
                                        // 如果没有分屏，先启用分屏
                                        app.split_mode = SplitMode::Vertical;
                                    }
                                    app.focused_pane = FocusedPane::Right;
                                    
                                    // 确保右侧有选中的提交
                                    if !app.git_commits.is_empty() && app.git_list_state.selected().is_none() {
                                        app.git_list_state.select(Some(0));
                                    }
                                } else {
                                    // 如果焦点在右侧（提交列表），进入 diff 查看模式
                                    app.enter_diff_view_mode().await;
                                }
                            }
                            _ => {
                                // 在查询历史视图中，执行查询
                                app.execute_selected_query().await;
                            }
                        }
                    }
                    (_, KeyCode::Char('c')) => {
                        // c 键用于 checkout（仅在分支/标签视图）
                        match app.tabs[app.current_tab].view_type {
                            ViewType::Branches => {
                                if let Some(selected) = app.branch_list_state.selected() {
                                    if let Some(branch) = app.branches.get(selected) {
                                        let branch_name = branch.name.clone();
                                        if !branch.is_current {
                                            app.checkout_branch(&branch_name).await.ok();
                                        }
                                    }
                                }
                            }
                            ViewType::Tags => {
                                if let Some(selected) = app.tag_list_state.selected() {
                                    if let Some(tag) = app.tags.get(selected) {
                                        let tag_name = tag.name.clone();
                                        app.checkout_tag(&tag_name).await.ok();
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    (_, KeyCode::Char('p')) => {
                        // p 键拉取最新代码
                        app.pull().await.ok();
                    }
                    (_, KeyCode::Char('l')) => {
                        // l 键切换左侧面板显示
                        app.show_left_panel = !app.show_left_panel;
                    }
                    (_, KeyCode::Down) | (_, KeyCode::Char('j')) => {
                        // 根据当前视图和焦点位置决定导航行为
                        match app.tabs[app.current_tab].view_type {
                            ViewType::Results => {
                                // Results 视图中，上下键用于滚动内容
                                app.result_scroll = app.result_scroll.saturating_add(1);
                            }
                            ViewType::Branches | ViewType::Tags | ViewType::Remotes => {
                                if app.focused_pane == FocusedPane::Right && app.split_mode != SplitMode::None {
                                    // 焦点在右侧，在提交列表中导航
                                    if !app.git_commits.is_empty() {
                                        let i = match app.git_list_state.selected() {
                                            Some(i) => {
                                                if i >= app.git_commits.len() - 1 {
                                                    0
                                                } else {
                                                    i + 1
                                                }
                                            }
                                            None => 0,
                                        };
                                        app.git_list_state.select(Some(i));
                                        app.load_selected_diff().await;
                                    }
                                } else {
                                    // 焦点在左侧，在分支/标签列表中导航
                                    app.next().await;
                                    app.load_selected_diff().await;
                                }
                            }
                            _ => {
                                // 其他视图的原有逻辑
                                if app.focused_pane == FocusedPane::Left || app.split_mode == SplitMode::None {
                                    app.next().await;
                                    app.load_selected_diff().await;
                                } else {
                                    app.result_scroll = app.result_scroll.saturating_add(1);
                                }
                            }
                        }
                    }
                    (_, KeyCode::Up) | (_, KeyCode::Char('k')) => {
                        // 根据当前视图和焦点位置决定导航行为
                        match app.tabs[app.current_tab].view_type {
                            ViewType::Results => {
                                // Results 视图中，上下键用于滚动内容
                                app.result_scroll = app.result_scroll.saturating_sub(1);
                            }
                            ViewType::Branches | ViewType::Tags | ViewType::Remotes => {
                                if app.focused_pane == FocusedPane::Right && app.split_mode != SplitMode::None {
                                    // 焦点在右侧，在提交列表中导航
                                    if !app.git_commits.is_empty() {
                                        let i = match app.git_list_state.selected() {
                                            Some(i) => {
                                                if i == 0 {
                                                    app.git_commits.len() - 1
                                                } else {
                                                    i - 1
                                                }
                                            }
                                            None => 0,
                                        };
                                        app.git_list_state.select(Some(i));
                                        app.load_selected_diff().await;
                                    }
                                } else {
                                    // 焦点在左侧，在分支/标签列表中导航
                                    app.previous().await;
                                    app.load_selected_diff().await;
                                }
                            }
                            _ => {
                                // 其他视图的原有逻辑
                                if app.focused_pane == FocusedPane::Left || app.split_mode == SplitMode::None {
                                    app.previous().await;
                                    app.load_selected_diff().await;
                                } else {
                                    app.result_scroll = app.result_scroll.saturating_sub(1);
                                }
                            }
                        }
                    }
                    (_, KeyCode::Char('d')) => app.show_details = !app.show_details,
                    (_, KeyCode::Char('/')) => {
                        app.search_mode = true;
                        app.search_filter.clear();
                    }
                    (_, KeyCode::Char('?')) => {
                        app.show_help = true;
                    }
                    (_, KeyCode::Char('h')) => app.syntax_highlight = !app.syntax_highlight,
                    (_, KeyCode::Char('r')) => {
                        // 如果在历史视图中，刷新历史记录；否则重置滚动位置
                        if app.tabs[app.current_tab].view_type == ViewType::History {
                            // 刷新 git 提交记录
                            app.refresh_git_commits().await;
                        } else {
                            app.result_scroll = 0;
                        }
                    }
                    (_, KeyCode::Char('g')) => {
                        // 跳转到开头
                        if app.focused_pane == FocusedPane::Left || app.split_mode == SplitMode::None {
                            app.first();
                            app.load_selected_diff().await;
                        }
                    }
                    (_, KeyCode::Char('G')) => {
                        // 跳转到结尾
                        if app.focused_pane == FocusedPane::Left || app.split_mode == SplitMode::None {
                            app.last();
                            app.load_selected_diff().await;
                        }
                    }
                    (_, KeyCode::Char('f')) => {
                        // 向下翻页
                        if app.focused_pane == FocusedPane::Left || app.split_mode == SplitMode::None {
                            app.page_down();
                            app.load_selected_diff().await;
                        } else {
                            app.result_scroll = app.result_scroll.saturating_add(10);
                        }
                    }
                    (_, KeyCode::Char('b')) => {
                        // 向上翻页
                        if app.focused_pane == FocusedPane::Left || app.split_mode == SplitMode::None {
                            app.page_up();
                            app.load_selected_diff().await;
                        } else {
                            app.result_scroll = app.result_scroll.saturating_sub(10);
                        }
                    }
                    (_, KeyCode::PageUp) => {
                        // 快速向上滚动
                        match app.tabs[app.current_tab].view_type {
                            ViewType::Results => {
                                app.result_scroll = app.result_scroll.saturating_sub(10);
                            }
                            _ => {
                                app.result_scroll = app.result_scroll.saturating_sub(10);
                            }
                        }
                    }
                    (_, KeyCode::PageDown) => {
                        // 快速向下滚动  
                        match app.tabs[app.current_tab].view_type {
                            ViewType::Results => {
                                app.result_scroll = app.result_scroll.saturating_add(10);
                            }
                            _ => {
                                app.result_scroll = app.result_scroll.saturating_add(10);
                            }
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

/// 绘制UI
fn ui(f: &mut Frame, app: &mut App) {
    // 如果在 diff 查看模式，显示专业的 diff 查看器
    if app.diff_view_mode {
        if let Some(viewer) = &mut app.diff_viewer {
            render_diff_viewer(f, viewer);
            return;
        }
    }
    
    // 显示帮助弹窗
    if app.show_help {
        render_help(f, f.size());
        return;
    }

    // 主布局
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),    // 标签栏 - 增加高度以确保可见
            Constraint::Min(0),       // 内容区
            Constraint::Length(3),    // 命令行/状态栏 + 菜单栏
        ])
        .split(f.size());

    // 渲染标签栏
    render_tabs(f, app, main_chunks[0]);

    // 根据分屏模式渲染内容
    match app.split_mode {
        SplitMode::None => {
            render_single_view(f, app, main_chunks[1]);
        }
        SplitMode::Horizontal => {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(main_chunks[1]);
            
            render_history_view(f, app, chunks[0], app.focused_pane == FocusedPane::Top);
            render_result_view(f, app, chunks[1], app.focused_pane == FocusedPane::Bottom);
        }
        SplitMode::Vertical => {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(main_chunks[1]);
            
            render_history_view(f, app, chunks[0], app.focused_pane == FocusedPane::Left);
            render_result_view(f, app, chunks[1], app.focused_pane == FocusedPane::Right);
        }
    }

    // 渲染命令行或状态栏
    if app.command_mode {
        render_command_line(f, app, main_chunks[2]);
    } else {
        render_status_bar(f, app, main_chunks[2]);
    }
}

/// 渲染标签栏
fn render_tabs(f: &mut Frame, app: &App, area: Rect) {
    // 使用更紧凑的标签名，并添加索引提示
    let titles: Vec<String> = app.tabs.iter().enumerate().map(|(i, t)| {
        let name = match t.view_type {
            ViewType::History => "Git Log",
            ViewType::Branches => "Branches",
            ViewType::Tags => "Tags",
            ViewType::Remotes => "Remotes",
            ViewType::Results => "Results",
            ViewType::QueryHistory => "Query History",
        };
        format!("{}. {}", i + 1, name)
    }).collect();
    
    let tabs = Tabs::new(titles)
        .block(Block::default()
            .borders(Borders::ALL)
            .title(" Navigation (Tab/Shift-Tab to switch) ")
            .title_alignment(Alignment::Center))
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default()
            .fg(Color::Yellow)
            .bg(Color::DarkGray)
            .add_modifier(Modifier::BOLD))
        .select(app.current_tab)
        .divider(" | ");  // 添加分隔符使标签更清晰
    
    f.render_widget(tabs, area);
}

/// 渲染单视图
fn render_single_view(f: &mut Frame, app: &App, area: Rect) {
    if app.tabs.is_empty() {
        return;
    }
    
    match app.tabs[app.current_tab].view_type {
        ViewType::History => render_history_view(f, app, area, true),
        ViewType::Results => render_result_view(f, app, area, true),
        ViewType::Branches => render_branches_view(f, app, area),
        ViewType::Tags => render_tags_view(f, app, area),
        ViewType::Remotes => render_remotes_view(f, app, area),
        _ => {}
    }
}

/// 渲染历史视图
fn render_history_view(f: &mut Frame, app: &App, area: Rect, focused: bool) {
    let border_style = if focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::White)
    };

    // 分割区域
    let chunks = if app.show_details {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(area)
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(100)])
            .split(area)
    };

    // 根据当前标签页显示不同的内容
    if app.tabs[app.current_tab].view_type == ViewType::History {
        // Git 提交列表
        let items: Vec<ListItem> = app
            .git_commits
            .iter()
            .enumerate()
            .map(|(i, commit)| {
                let short_hash = &commit.hash[..8.min(commit.hash.len())];
                let timestamp = commit.timestamp.format("%m-%d %H:%M");
                
                let content = if app.syntax_highlight {
                    // 语法高亮 - 根据提交信息类型着色
                    if commit.message.starts_with("feat") {
                        format!("{} {} {} - {}", 
                            short_hash, 
                            timestamp,
                            commit.message,
                            commit.author
                        )
                    } else if commit.message.starts_with("fix") {
                        format!("{} {} {} - {}", 
                            short_hash, 
                            timestamp,
                            commit.message,
                            commit.author
                        )
                    } else {
                        format!("{} {} {} - {}", 
                            short_hash, 
                            timestamp,
                            commit.message,
                            commit.author
                        )
                    }
                } else {
                    format!("{} {} {} - {}", 
                        short_hash, 
                        timestamp,
                        commit.message,
                        commit.author
                    )
                };

                let style = if Some(i) == app.git_list_state.selected() {
                    Style::default()
                        .bg(Color::DarkGray)
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else if commit.message.starts_with("feat") {
                    Style::default().fg(Color::Green)
                } else if commit.message.starts_with("fix") {
                    Style::default().fg(Color::Red)
                } else if commit.message.starts_with("docs") {
                    Style::default().fg(Color::Blue)
                } else {
                    Style::default().fg(Color::White)
                };

                ListItem::new(content).style(style)
            })
            .collect();

        // Debug: 检查 items 是否为空
        if items.is_empty() && app.git_commits.len() > 0 {
            // 如果 git_commits 有数据但 items 为空，添加一个调试项目
            let debug_items = vec![
                ListItem::new(format!("DEBUG: {} commits loaded but no items generated!", app.git_commits.len()))
                    .style(Style::default().fg(Color::Red))
            ];
            let list = List::new(debug_items)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(" DEBUG: Git Log Issue ")
                        .border_style(border_style),
                );
            f.render_widget(list, chunks[0]);
            return;
        }

        let title = if app.search_mode {
            format!(" Git Log [/{}] ({} commits) ", app.search_filter, app.git_commits.len())
        } else {
            format!(" Git Log ({} commits) ", app.git_commits.len())
        };

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(title)
                    .border_style(border_style),
            );

        f.render_stateful_widget(list, chunks[0], &mut app.git_list_state.clone());

        // 详情面板 - 显示提交详情和 diff
        if app.show_details && chunks.len() > 1 {
            if let Some(selected) = app.git_list_state.selected() {
                if let Some(commit) = app.git_commits.get(selected) {
                    // 分割详情区域：基本信息 + diff 内容
                    let detail_chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([Constraint::Length(7), Constraint::Min(0)])
                        .split(chunks[1]);

                    // 基本提交信息
                    let basic_info = format!(
                        "Hash: {}\nMessage: {}\nAuthor: {}\nTime: {}\nRefs: {}",
                        commit.hash,
                        commit.message,
                        commit.author,
                        commit.timestamp.format("%Y-%m-%d %H:%M:%S"),
                        if commit.refs.is_empty() { "N/A" } else { &commit.refs }
                    );

                    let info_widget = Paragraph::new(basic_info)
                        .block(
                            Block::default()
                                .borders(Borders::ALL)
                                .title(" Commit Info ")
                                .border_style(border_style),
                        )
                        .wrap(Wrap { trim: true });

                    f.render_widget(info_widget, detail_chunks[0]);

                    // diff 内容
                    let diff_content = if let Some(diff) = &app.current_diff {
                        // 处理和格式化 diff 内容
                        let formatted_diff = format_diff_content(diff);
                        // 如果 diff 太长，安全地截断（避免在 UTF-8 字符中间切断）
                        if formatted_diff.len() > 8000 {
                            let truncated = safe_truncate(&formatted_diff, 8000);
                            format!("{}...\n\n[Diff too long, showing first ~8000 characters. Use ↑↓ to scroll]", truncated)
                        } else {
                            formatted_diff
                        }
                    } else {
                        "Loading diff...\n\nPress ↑↓ to navigate commits and view their diffs.".to_string()
                    };

                    let scroll_info = if app.result_scroll > 0 {
                        format!(" Git Diff (scroll: {}) ", app.result_scroll)
                    } else {
                        " Git Diff ".to_string()
                    };

                    let diff_widget = Paragraph::new(diff_content)
                        .block(
                            Block::default()
                                .borders(Borders::ALL)
                                .title(scroll_info)
                                .border_style(border_style),
                        )
                        .wrap(Wrap { trim: false })
                        .scroll((app.result_scroll, 0));

                    f.render_widget(diff_widget, detail_chunks[1]);
                }
            }
        }
    } else {
        // 原来的查询历史列表
        let items: Vec<ListItem> = app
            .entries
            .iter()
            .enumerate()
            .map(|(i, entry)| {
                let status_icon = if entry.success { "✅" } else { "❌" };
                let timestamp = entry.timestamp.format("%H:%M:%S");
                
                let content = if app.syntax_highlight {
                    // 语法高亮
                    let parts: Vec<&str> = entry.query.split(':').collect();
                    if parts.len() == 2 {
                        format!("{} {} {}:{}", 
                            status_icon, 
                            timestamp,
                            parts[0],
                            parts[1]
                        )
                    } else {
                        format!("{} {} {}", status_icon, timestamp, entry.query)
                    }
                } else {
                    format!("{} {} {}", status_icon, timestamp, entry.query)
                };

                let style = if Some(i) == app.list_state.selected() {
                    Style::default()
                        .bg(Color::DarkGray)
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else if entry.success {
                    Style::default().fg(Color::Green)
                } else {
                    Style::default().fg(Color::Red)
                };

                ListItem::new(content).style(style)
            })
            .collect();

        let title = if app.search_mode {
            format!(" Query History [/{}] ", app.search_filter)
        } else {
            " Query History ".to_string()
        };

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(title)
                    .border_style(border_style),
            );

        f.render_stateful_widget(list, chunks[0], &mut app.list_state.clone());

        // 详情面板
        if app.show_details && chunks.len() > 1 {
            if let Some(selected) = app.list_state.selected() {
                if let Some(entry) = app.entries.get(selected) {
                    let details_text = format!(
                        "Query: {}\nTime: {}\nType: {}\nResults: {}\nStatus: {}",
                        entry.query,
                        entry.timestamp.format("%Y-%m-%d %H:%M:%S"),
                        entry.query_type.as_deref().unwrap_or("unknown"),
                        entry.result_count.map_or("N/A".to_string(), |c| c.to_string()),
                        if entry.success { "Success" } else { "Failed" }
                    );

                    let details = Paragraph::new(details_text)
                        .block(
                            Block::default()
                                .borders(Borders::ALL)
                                .title(" Details ")
                                .border_style(border_style),
                        )
                        .wrap(Wrap { trim: true });

                    f.render_widget(details, chunks[1]);
                }
            }
        }
    }
}

/// 渲染结果视图
fn render_result_view(f: &mut Frame, app: &App, area: Rect, focused: bool) {
    let border_style = if focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::White)
    };

    // 查找结果标签页
    let result_content = app.tabs
        .iter()
        .find(|t| t.view_type == ViewType::Results)
        .map(|t| t.content.clone())
        .unwrap_or_else(|| "No results to display".to_string());

    let lines: Vec<String> = result_content.lines().map(|l| l.to_string()).collect();
    let start = app.result_scroll as usize;
    let visible_lines: Vec<ListItem> = lines
        .iter()
        .skip(start)
        .take(area.height as usize - 2)
        .map(|line| {
            let style = if line.contains("feat") {
                Style::default().fg(Color::Green)
            } else if line.contains("fix") {
                Style::default().fg(Color::Yellow)
            } else if line.contains("docs") {
                Style::default().fg(Color::Blue)
            } else {
                Style::default()
            };
            ListItem::new(line.as_str()).style(style)
        })
        .collect();

    let list = List::new(visible_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Results ")
                .border_style(border_style),
        );

    f.render_widget(list, area);

    // 滚动条
    if lines.len() > area.height as usize - 2 {
        let scrollbar = Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"));
        
        let mut scrollbar_state = ratatui::widgets::ScrollbarState::default()
            .content_length(lines.len())
            .position(app.result_scroll as usize);
        
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

/// 渲染帮助
fn render_help(f: &mut Frame, area: Rect) {
    let area = centered_rect(70, 85, area);
    f.render_widget(Clear, area);
    
    let help_text = vec![
        Line::from(""),
        Line::from(vec![Span::styled("Tab Navigation", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))]),
        Line::from("  1-4        Switch to tab 1-4 directly"),
        Line::from("  Tab        Next tab"),
        Line::from("  Shift+Tab  Previous tab"),
        Line::from(""),
        Line::from(vec![Span::styled("Navigation", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))]),
        Line::from("  ↑/k        Move up"),
        Line::from("  ↓/j        Move down"),
        Line::from("  g          Go to first"),
        Line::from("  G          Go to last"),
        Line::from("  f/PgDn     Page down"),
        Line::from("  b/PgUp     Page up"),
        Line::from(""),
        Line::from(vec![Span::styled("Actions", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))]),
        Line::from("  Enter/x    View commit diff / Execute action"),
        Line::from("  c          Checkout branch/tag"),
        Line::from("  p          Pull latest changes"),
        Line::from("  /          Search"),
        Line::from("  d          Toggle details"),
        Line::from("  r          Refresh git log"),
        Line::from("  h          Toggle syntax highlighting"),
        Line::from("  l          Toggle left panel"),
        Line::from(""),
        Line::from(vec![Span::styled("Window Management", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))]),
        Line::from("  Ctrl+s     Toggle split mode"),
        Line::from("  Ctrl+w     Switch focus"),
        Line::from("  :vsplit    Vertical split"),
        Line::from("  :hsplit    Horizontal split"),
        Line::from(""),
        Line::from(vec![Span::styled("Commands", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))]),
        Line::from("  :          Command mode"),
        Line::from("  :q         Quit"),
        Line::from("  :tab NAME  New tab"),
        Line::from("  ?          This help"),
        Line::from("  q/ESC      Quit"),
    ];
    
    let help = Paragraph::new(help_text)
        .block(Block::default()
            .title(" Help ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green)))
        .style(Style::default().fg(Color::White));
        
    f.render_widget(help, area);
}

/// 渲染命令行
fn render_command_line(f: &mut Frame, app: &App, area: Rect) {
    let input = Paragraph::new(format!(":{}", app.command_input))
        .style(Style::default().fg(Color::Yellow));
    f.render_widget(input, area);
}

/// 渲染分支视图
fn render_branches_view(f: &mut Frame, app: &App, area: Rect) {
    // 分割区域：左侧分支列表，右侧提交历史
    let chunks = if app.show_left_panel {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(30), // 分支列表
                Constraint::Percentage(70), // 提交历史
            ])
            .split(area)
    } else {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(100)])
            .split(area)
    };
    
    // 根据焦点位置设置边框颜色
    let left_border_style = if app.focused_pane == FocusedPane::Left {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::White)
    };
    
    // 渲染分支列表
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
            
            let prefix = if branch.is_current { "> " } else { "  " };
            let text = format!("{}{}", prefix, branch.name);
            
            ListItem::new(text).style(style)
        })
        .collect();
    
    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" Branches ({}) ", app.branches.len()))
                .border_style(left_border_style),
        )
        .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
        .highlight_symbol(">> ");
    
    f.render_stateful_widget(list, chunks[0], &mut app.branch_list_state.clone());
    
    // 如果显示左侧面板，则在右侧显示提交历史
    if app.show_left_panel && chunks.len() > 1 {
        let right_focused = app.focused_pane == FocusedPane::Right;
        render_commits_for_view(f, app, chunks[1], right_focused);
    }
}

/// 渲染标签视图
fn render_tags_view(f: &mut Frame, app: &App, area: Rect) {
    // 分割区域：左侧标签列表，右侧提交历史
    let chunks = if app.show_left_panel {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(30), // 标签列表
                Constraint::Percentage(70), // 提交历史
            ])
            .split(area)
    } else {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(100)])
            .split(area)
    };
    
    // 根据焦点位置设置边框颜色
    let left_border_style = if app.focused_pane == FocusedPane::Left {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::White)
    };
    
    let items: Vec<ListItem> = app.tags
        .iter()
        .map(|tag| {
            let text = if let Some(date) = &tag.date {
                format!("{} - {} ({})", 
                    tag.name, 
                    tag.message, 
                    date.format("%Y-%m-%d"))
            } else {
                format!("{} - {}", tag.name, tag.message)
            };
            
            ListItem::new(text).style(Style::default().fg(Color::Yellow))
        })
        .collect();
    
    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" Tags ({}) ", app.tags.len()))
                .border_style(left_border_style),
        )
        .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
        .highlight_symbol(">> ");
    
    f.render_stateful_widget(list, chunks[0], &mut app.tag_list_state.clone());
    
    // 如果显示左侧面板，则在右侧显示提交历史
    if app.show_left_panel && chunks.len() > 1 {
        let right_focused = app.focused_pane == FocusedPane::Right;
        render_commits_for_view(f, app, chunks[1], right_focused);
    }
}

/// 渲染远程仓库视图
fn render_remotes_view(f: &mut Frame, app: &App, area: Rect) {
    // 分割区域：左侧远程列表，右侧提交历史
    let chunks = if app.show_left_panel {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(30), // 远程列表
                Constraint::Percentage(70), // 提交历史
            ])
            .split(area)
    } else {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(100)])
            .split(area)
    };
    
    // 根据焦点位置设置边框颜色
    let left_border_style = if app.focused_pane == FocusedPane::Left {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::White)
    };
    
    let items: Vec<ListItem> = app.remotes
        .iter()
        .map(|remote| {
            let text = format!("{}: {}", remote.name, remote.url);
            ListItem::new(text).style(Style::default().fg(Color::Magenta))
        })
        .collect();
    
    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" Remotes ({}) ", app.remotes.len()))
                .border_style(left_border_style),
        )
        .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
        .highlight_symbol(">> ");
    
    f.render_stateful_widget(list, chunks[0], &mut app.remote_list_state.clone());
    
    // 如果显示左侧面板，则在右侧显示提交历史
    if app.show_left_panel && chunks.len() > 1 {
        let right_focused = app.focused_pane == FocusedPane::Right;
        render_commits_for_view(f, app, chunks[1], right_focused);
    }
}

/// 为当前视图渲染提交历史
fn render_commits_for_view(f: &mut Frame, app: &App, area: Rect, focused: bool) {
    // 分割区域以显示提交列表和详情
    let chunks = if app.show_details {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(area)
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(100)])
            .split(area)
    };
    
    // 根据焦点状态设置边框颜色
    let border_style = if focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::White)
    };
    
    // Git 提交列表
    let items: Vec<ListItem> = app
        .git_commits
        .iter()
        .enumerate()
        .map(|(_i, commit)| {
            let short_hash = &commit.hash[..8.min(commit.hash.len())];
            let timestamp = commit.timestamp.format("%m-%d %H:%M");
            
            let content = if app.syntax_highlight {
                // 语法高亮
                let (prefix, color) = if commit.message.starts_with("feat") {
                    ("✨", Color::Green)
                } else if commit.message.starts_with("fix") {
                    ("🐛", Color::Red)
                } else if commit.message.starts_with("docs") {
                    ("📚", Color::Blue)
                } else if commit.message.starts_with("style") {
                    ("💎", Color::Magenta)
                } else if commit.message.starts_with("refactor") {
                    ("♻️ ", Color::Yellow)
                } else if commit.message.starts_with("test") {
                    ("🧪", Color::Cyan)
                } else if commit.message.starts_with("chore") {
                    ("🔧", Color::Gray)
                } else {
                    ("●", Color::White)
                };
                
                Line::from(vec![
                    Span::styled(format!("{} ", prefix), Style::default().fg(color)),
                    Span::styled(format!("{} ", short_hash), Style::default().fg(Color::DarkGray)),
                    Span::styled(format!("[{}] ", timestamp), Style::default().fg(Color::Blue)),
                    Span::styled(&commit.message, Style::default().fg(color)),
                    Span::styled(format!(" - {}", commit.author), Style::default().fg(Color::Gray)),
                ])
            } else {
                Line::from(format!(
                    "{} [{}] {} - {}",
                    short_hash, timestamp, commit.message, commit.author
                ))
            };
            
            ListItem::new(content)
        })
        .collect();
    
    let title = match app.tabs[app.current_tab].view_type {
        ViewType::Branches => {
            if let Some(idx) = app.branch_list_state.selected() {
                if let Some(branch) = app.branches.get(idx) {
                    format!(" Commits for {} ", branch.name)
                } else {
                    " Commits ".to_string()
                }
            } else {
                " Commits ".to_string()
            }
        }
        ViewType::Tags => {
            if let Some(idx) = app.tag_list_state.selected() {
                if let Some(tag) = app.tags.get(idx) {
                    format!(" Commits for {} ", tag.name)
                } else {
                    " Commits ".to_string()
                }
            } else {
                " Commits ".to_string()
            }
        }
        _ => " Commits ".to_string(),
    };
    
    let git_list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(border_style),
        )
        .highlight_style(Style::default().bg(Color::DarkGray))
        .highlight_symbol(">> ");
    
    f.render_stateful_widget(git_list, chunks[0], &mut app.git_list_state.clone());
    
    // 如果显示详情，渲染 diff
    if app.show_details && chunks.len() > 1 {
        render_diff_content(f, app, chunks[1]);
    }
}

/// 渲染 diff 内容
fn render_diff_content(f: &mut Frame, app: &App, area: Rect) {
    let diff_content = if let Some(diff) = &app.current_diff {
        // 处理和格式化 diff 内容
        let formatted_diff = format_diff_content(diff);
        if formatted_diff.len() > 8000 {
            let truncated = safe_truncate(&formatted_diff, 8000);
            format!("{}...\n\n[Diff too long, showing first ~8000 characters. Use ↑↓ to scroll]", truncated)
        } else {
            formatted_diff
        }
    } else {
        "Loading diff...\n\nPress ↑↓ to navigate commits and view their diffs.".to_string()
    };

    let scroll_info = if app.result_scroll > 0 {
        format!(" Git Diff (scroll: {}) ", app.result_scroll)
    } else {
        " Git Diff ".to_string()
    };

    let diff_widget = Paragraph::new(diff_content)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(scroll_info)
                .border_style(Style::default().fg(Color::White)),
        )
        .wrap(Wrap { trim: false })
        .scroll((app.result_scroll, 0));

    f.render_widget(diff_widget, area);
}

/// 渲染状态栏和菜单栏
fn render_status_bar(f: &mut Frame, app: &App, area: Rect) {
    // 分割区域为状态栏和菜单栏
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // 状态信息
            Constraint::Length(1), // 菜单栏
            Constraint::Length(1), // 额外的快捷键行
        ])
        .split(area);
    
    // 渲染状态信息
    let mode = if app.search_mode {
        "SEARCH"
    } else {
        "NORMAL"
    };
    
    let split_info = match app.split_mode {
        SplitMode::None => "",
        SplitMode::Horizontal => " [H-SPLIT]",
        SplitMode::Vertical => " [V-SPLIT]",
    };
    
    let entry_count = match app.tabs[app.current_tab].view_type {
        ViewType::History => app.git_commits.len(),
        ViewType::Branches => app.branches.len(),
        ViewType::Tags => app.tags.len(),
        ViewType::Remotes => app.remotes.len(),
        _ => app.entries.len(),
    };

    let selected_info = if entry_count > 0 {
        let idx = match app.tabs[app.current_tab].view_type {
            ViewType::History => app.git_list_state.selected().unwrap_or(0),
            ViewType::Branches => app.branch_list_state.selected().unwrap_or(0),
            ViewType::Tags => app.tag_list_state.selected().unwrap_or(0),
            ViewType::Remotes => app.remote_list_state.selected().unwrap_or(0),
            _ => app.selected_index,
        };
        format!(" | {}/{}", idx + 1, entry_count)
    } else {
        String::new()
    };
    
    // 添加当前分支信息
    let branch_info = if !app.current_branch.is_empty() {
        format!(" | Branch: {}", app.current_branch)
    } else {
        String::new()
    };
    
    // 添加状态消息
    let status_msg = if let Some(msg) = &app.status_message {
        format!(" | {}", msg)
    } else {
        String::new()
    };

    let status = format!(
        " {} | Tab {}/{}{} items{}{}{}",
        mode,
        app.current_tab + 1,
        app.tabs.len(),
        selected_info,
        branch_info,
        status_msg,
        split_info
    );
    
    let status_bar = Paragraph::new(status)
        .style(Style::default().fg(Color::Cyan).bg(Color::DarkGray));
    
    f.render_widget(status_bar, chunks[0]);
    
    // 渲染主菜单栏
    render_main_menu_bar(f, chunks[1]);
    
    // 渲染额外的快捷键（第二行菜单）
    render_extra_shortcuts(f, chunks[2]);
}

/// 渲染主菜单栏
fn render_main_menu_bar(f: &mut Frame, area: Rect) {
    let menu_items = vec![
        ("j/↓", "Down", Color::Yellow),
        ("k/↑", "Up", Color::Yellow),
        ("Enter/c", "Checkout", Color::Green),
        ("p", "Pull", Color::Cyan),
        ("Tab", "Next Tab", Color::Magenta),
        ("S-Tab", "Prev Tab", Color::Magenta),
        ("l", "Toggle Panel", Color::Blue),
        ("/", "Search", Color::Blue),
        ("?", "Help", Color::Cyan),
        ("q", "Quit", Color::Red),
    ];
    
    let mut spans = Vec::new();
    for (i, (key, desc, color)) in menu_items.iter().enumerate() {
        if i > 0 {
            spans.push(Span::raw(" │ "));
        }
        spans.push(Span::styled(*key, Style::default().fg(*color).add_modifier(Modifier::BOLD)));
        spans.push(Span::raw(" "));
        spans.push(Span::styled(*desc, Style::default().fg(Color::Gray)));
    }
    
    let menu = Paragraph::new(Line::from(spans))
        .style(Style::default().bg(Color::Black))
        .alignment(Alignment::Center);
    
    f.render_widget(menu, area);
}

/// 渲染额外的快捷键
fn render_extra_shortcuts(f: &mut Frame, area: Rect) {
    let shortcuts = vec![
        ("g", "First", Color::Blue),
        ("G", "Last", Color::Blue),
        ("f/PgDn", "Page Down", Color::Blue),
        ("b/PgUp", "Page Up", Color::Blue),
        ("r", "Refresh", Color::Green),
        ("d", "Details", Color::Yellow),
        ("h", "Syntax", Color::Magenta),
        ("C-s", "Split", Color::Cyan),
        ("C-w", "Focus", Color::Cyan),
    ];
    
    let mut spans = Vec::new();
    for (i, (key, desc, color)) in shortcuts.iter().enumerate() {
        if i > 0 {
            spans.push(Span::raw(" │ "));
        }
        spans.push(Span::styled(*key, Style::default().fg(*color).add_modifier(Modifier::BOLD)));
        spans.push(Span::raw(" "));
        spans.push(Span::styled(*desc, Style::default().fg(Color::Gray)));
    }
    
    let menu = Paragraph::new(Line::from(spans))
        .style(Style::default().bg(Color::Black))
        .alignment(Alignment::Center);
    
    f.render_widget(menu, area);
}

/// 安全地截断字符串，确保不在 UTF-8 字符中间切断
fn safe_truncate(s: &str, max_bytes: usize) -> &str {
    if s.len() <= max_bytes {
        return s;
    }
    
    // 找到最接近 max_bytes 但不超过的字符边界
    let mut end = max_bytes;
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    
    // 如果找到了有效的边界，返回截断的字符串
    if end > 0 {
        &s[..end]
    } else {
        // 极端情况：如果前 max_bytes 字节都不是字符边界，返回空字符串
        ""
    }
}

/// 格式化 diff 内容，使其更易读
fn format_diff_content(diff: &str) -> String {
    let mut formatted = String::new();
    let mut in_diff_section = false;
    let mut commit_info_done = false;
    
    for line in diff.lines() {
        // 检测不同的部分
        if line.starts_with("commit ") {
            formatted.push_str(&format!("🔖 {}\n", line));
        } else if line.starts_with("Author: ") {
            formatted.push_str(&format!("👤 {}\n", line));
        } else if line.starts_with("Date: ") {
            formatted.push_str(&format!("📅 {}\n", line));
        } else if line.trim().is_empty() && !commit_info_done && !in_diff_section {
            formatted.push_str("\n");
        } else if line.starts_with("    ") && !commit_info_done {
            // 提交消息
            formatted.push_str(&format!("💬 {}\n", line.trim()));
        } else if line == "---" {
            formatted.push_str("═══════════════════════════════════════════════════════════\n");
            commit_info_done = true;
        } else if line.contains(" | ") && line.contains(" +++") || line.contains(" ---") {
            // 文件统计行，如 " file.txt | 123 +++++++++"
            formatted.push_str(&format!("📊 {}\n", line));
        } else if line.contains(" files changed, ") {
            // 总计统计行
            formatted.push_str(&format!("📈 {}\n", line));
        } else if line.starts_with("diff --git") {
            formatted.push_str(&format!("\n📁 {}\n", line));
            in_diff_section = true;
        } else if line.starts_with("new file mode ") {
            formatted.push_str(&format!("✨ {}\n", line));
        } else if line.starts_with("deleted file mode ") {
            formatted.push_str(&format!("🗑️  {}\n", line));
        } else if line.starts_with("index ") {
            formatted.push_str(&format!("🔍 {}\n", line));
        } else if line.starts_with("--- ") {
            formatted.push_str(&format!("📄 {}\n", line));
        } else if line.starts_with("+++ ") {
            formatted.push_str(&format!("📄 {}\n", line));
        } else if line.starts_with("@@") {
            formatted.push_str(&format!("📍 {}\n", line));
        } else if line.starts_with('+') && !line.starts_with("+++") {
            formatted.push_str(&format!("+ {}\n", &line[1..]));
        } else if line.starts_with('-') && !line.starts_with("---") && in_diff_section {
            formatted.push_str(&format!("- {}\n", &line[1..]));
        } else if in_diff_section && !line.starts_with("diff --git") {
            // 上下文行
            formatted.push_str(&format!("  {}\n", line));
        } else {
            // 其他行
            formatted.push_str(&format!("{}\n", line));
        }
    }
    
    // 如果内容为空，添加提示
    if formatted.trim().is_empty() {
        formatted = "No changes in this commit.".to_string();
    }
    
    formatted
}

/// 计算居中矩形
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

/// 显示查询历史的TUI界面
pub async fn show_history_tui() -> Result<()> {
    run_tui().await?;
    Ok(())
}


/// 测试 Git 提交加载功能（用于调试）
pub async fn test_git_commits_loading() -> Result<()> {
    let app = App::new().await?;
    println!("Loaded {} git commits", app.git_commits.len());
    for (i, commit) in app.git_commits.iter().enumerate().take(5) {
        println!("Commit {}: {} - {} by {}", 
            i + 1, 
            &commit.hash[..8], 
            commit.message, 
            commit.author
        );
    }
    Ok(())
}