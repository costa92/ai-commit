use std::sync::Arc;
use tokio::sync::RwLock;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent}, 
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen}
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal
};

use crate::tui_unified::{
    state::AppState,
    layout::LayoutManager,
    focus::{FocusManager, FocusPanel},
    config::AppConfig,
    git::interface::GitRepositoryAPI,
    components::{
        panels::sidebar::SidebarPanel,
        views::{
            git_log::GitLogView,
            branches::BranchesView,
            tags::TagsView,
            remotes::RemotesView,
            stash::StashView,
            query_history::QueryHistoryView,
        },
        base::{
            component::Component,
            events::EventResult,
        },
        widgets::{
            search_box::SearchBox,
            commit_editor::CommitEditor,
        },
    },
    Result
};
use crate::diff_viewer::{DiffViewer};
use crate::core::ai::agents::manager::AgentManager;
use crate::core::ai::agents::{AgentTask, AgentContext, TaskType, AgentConfig};
use crate::config::Config;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AppMode {
    Normal,      // 正常浏览模式
    Search,      // 搜索模式
    Command,     // 命令模式
    Help,        // 帮助模式
    Diff,        // 全屏diff模式
    AICommit,    // AI提交模式
}

pub struct TuiUnifiedApp {
    // 核心状态
    state: Arc<RwLock<AppState>>,
    
    // 管理器
    layout_manager: LayoutManager,
    focus_manager: FocusManager,
    
    // 组件
    sidebar_panel: SidebarPanel,
    git_log_view: GitLogView,
    branches_view: BranchesView,
    tags_view: TagsView,
    remotes_view: RemotesView,
    stash_view: StashView,
    query_history_view: QueryHistoryView,
    search_box: SearchBox,
    diff_viewer: Option<DiffViewer>,
    commit_editor: CommitEditor,
    
    // 配置
    _config: AppConfig,
    
    // 运行状态
    should_quit: bool,
    current_mode: AppMode,
    
    // AI commit 功能
    agent_manager: Option<AgentManager>,
    ai_commit_message: Option<String>,
    ai_commit_mode: bool,
    ai_commit_editing: bool,
    ai_commit_status: Option<String>,
    ai_commit_push_prompt: bool, // 是否显示推送提示
}

impl TuiUnifiedApp {
    pub async fn new() -> Result<Self> {
        let config = AppConfig::load().unwrap_or_default();
        let state = Arc::new(RwLock::new(AppState::new(&config).await?));
        
        let mut focus_manager = FocusManager::new();
        focus_manager.set_focus(FocusPanel::Content);  // 默认焦点在内容区，因为默认视图是GitLog
        
        Ok(Self {
            state: Arc::clone(&state),
            layout_manager: LayoutManager::new(&config),
            focus_manager,
            sidebar_panel: SidebarPanel::new(),
            git_log_view: GitLogView::new(),
            branches_view: BranchesView::new(),
            tags_view: TagsView::new(),
            remotes_view: RemotesView::new(),
            stash_view: StashView::new(),
            query_history_view: QueryHistoryView::new(),
            search_box: SearchBox::new().with_placeholder("Search...".to_string()),
            diff_viewer: None,
            commit_editor: CommitEditor::new(),
            _config: config,
            should_quit: false,
            current_mode: AppMode::Normal,
            
            // AI commit 初始化
            agent_manager: None,
            ai_commit_message: None,
            ai_commit_mode: false,
            ai_commit_editing: false,
            ai_commit_status: None,
            ai_commit_push_prompt: false,
        })
    }
    
    pub async fn run() -> Result<()> {
        // 设置终端
        enable_raw_mode()?;
        let mut stdout = std::io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        
        // 创建应用实例
        let mut app = Self::new().await?;
        
        // 运行主循环
        let result = app.run_loop(&mut terminal).await;
        
        // 恢复终端
        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
        terminal.show_cursor()?;
        
        result
    }
    
    async fn run_loop<B>(&mut self, terminal: &mut Terminal<B>) -> Result<()> 
    where
        B: ratatui::backend::Backend,
    {
        // 初始化Git数据
        self.load_initial_git_data().await?;
        
        // 主事件循环
        loop {
            // 渲染UI
            terminal.draw(|f| self.render(f))?;
            
            // 处理事件
            if event::poll(std::time::Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    self.handle_key_event(key).await?;
                }
            }
            
            // 处理pending diff请求
            self.handle_pending_diff_request().await?;
            
            // 处理直接分支切换请求
            self.handle_direct_branch_switch_request().await?;
            
            // 检查退出条件
            if self.should_quit {
                break;
            }
        }
        
        Ok(())
    }
    
    fn render(&mut self, frame: &mut ratatui::Frame) {
        // 计算布局
        let layout = self.layout_manager.calculate_layout(frame.size());
        
        // 检查是否能获取状态读锁
        let modal_info = match self.state.try_read() {
            Ok(state) => {
                let modal_clone = state.modal.clone();
                (true, modal_clone)
            }
            Err(_) => (false, None)
        };
        
        match self.state.try_read() {
            Ok(state) => {
                // 设置组件焦点状态
                self.sidebar_panel.set_focus(self.focus_manager.current_panel == FocusPanel::Sidebar);
                
                let current_view = state.current_view;
                
                // 渲染侧边栏
                self.sidebar_panel.render(frame, layout.sidebar, &*state);
                
                // 根据当前视图渲染主内容区
                match current_view {
                    crate::tui_unified::state::app_state::ViewType::GitLog => {
                        // Git Log 视图：左侧显示git log，右侧显示分支列表
                        use ratatui::layout::{Constraint, Direction, Layout};
                        
                        // 分割区域：左侧60%显示git log，右侧40%显示分支列表
                        let chunks = Layout::default()
                            .direction(Direction::Horizontal)
                            .constraints([
                                Constraint::Percentage(60), // Git log
                                Constraint::Percentage(40), // 分支列表
                            ])
                            .split(layout.content);

                        let content_focused = self.focus_manager.current_panel == FocusPanel::Content;
                        
                        // 渲染git log
                        self.git_log_view.set_focus(content_focused);
                        self.git_log_view.render(frame, chunks[0], &*state);
                        
                        // 渲染分支列表
                        self.branches_view.set_focus(false); // 分支列表在git log视图中不获得焦点
                        self.branches_view.render(frame, chunks[1], &*state);
                    }
                    crate::tui_unified::state::app_state::ViewType::Branches => {
                        // 分支视图：左侧显示分支列表，右侧显示该分支的git log
                        use ratatui::layout::{Constraint, Direction, Layout};
                        
                        // 分割区域：左侧40%显示分支列表，右侧60%显示git log
                        let chunks = Layout::default()
                            .direction(Direction::Horizontal)
                            .constraints([
                                Constraint::Percentage(40), // 分支列表
                                Constraint::Percentage(60), // Git log
                            ])
                            .split(layout.content);

                        let content_focused = self.focus_manager.current_panel == FocusPanel::Content;
                        
                        // 渲染分支列表
                        self.branches_view.set_focus(content_focused);
                        self.branches_view.render(frame, chunks[0], &*state);
                        
                        // 渲染git log
                        self.git_log_view.set_focus(false); // git log在分支视图中不获得焦点
                        self.git_log_view.render(frame, chunks[1], &*state);
                    }
                    crate::tui_unified::state::app_state::ViewType::Tags => {
                        self.tags_view.set_focus(self.focus_manager.current_panel == FocusPanel::Content);
                        self.tags_view.render(frame, layout.content, &*state);
                    }
                    crate::tui_unified::state::app_state::ViewType::Remotes => {
                        self.remotes_view.set_focus(self.focus_manager.current_panel == FocusPanel::Content);
                        self.remotes_view.render(frame, layout.content, &*state);
                    }
                    crate::tui_unified::state::app_state::ViewType::Stash => {
                        self.stash_view.set_focus(self.focus_manager.current_panel == FocusPanel::Content);
                        self.stash_view.render(frame, layout.content, &*state);
                    }
                    crate::tui_unified::state::app_state::ViewType::QueryHistory => {
                        self.query_history_view.set_focus(self.focus_manager.current_panel == FocusPanel::Content);
                        self.query_history_view.render(frame, layout.content, &*state);
                    }
                }
                
                // 渲染搜索框（如果在搜索模式）
                if self.current_mode == AppMode::Search {
                    self.search_box.set_focus(true);
                    self.search_box.set_search_active(true);
                    self.search_box.render(frame, layout.status_bar, &*state);
                } else {
                    self.search_box.set_focus(false);
                    self.search_box.set_search_active(false);
                    // 渲染状态栏
                    self.render_status_bar(frame, layout.status_bar, &*state);
                }
                
            }
            Err(_) => {
                // 如果无法获取读锁，显示加载状态
                Self::render_loading_state_static(frame, layout);
            }
        }
        
        // 渲染模态框（如果有的话）
        if let Some(modal) = modal_info.1 {
            self.render_modal(frame, &modal, frame.size());
        }
    }

    /// 渲染占位符视图
    fn render_placeholder_view(&self, frame: &mut ratatui::Frame, area: ratatui::layout::Rect, view_type: crate::tui_unified::state::app_state::ViewType) {
        use ratatui::{
            widgets::{Block, Borders, Paragraph},
            text::Text,
            style::{Color, Style}
        };

        let view_name = match view_type {
            crate::tui_unified::state::app_state::ViewType::Tags => "Tags",
            crate::tui_unified::state::app_state::ViewType::Remotes => "Remotes",
            crate::tui_unified::state::app_state::ViewType::Stash => "Stash",
            crate::tui_unified::state::app_state::ViewType::QueryHistory => "Query History",
            _ => "Unknown View",
        };

        let content = format!("🚧 {} View\n\nThis view is not yet implemented.\nPress 1-6 to switch to other views.", view_name);
        let paragraph = Paragraph::new(Text::raw(content))
            .block(Block::default().title(view_name).borders(Borders::ALL).border_style(Style::default().fg(Color::Gray)));
        
        frame.render_widget(paragraph, area);
    }

    /// 渲染状态栏
    fn render_status_bar(&self, frame: &mut ratatui::Frame, area: ratatui::layout::Rect, state: &AppState) {
        use ratatui::{
            widgets::{Block, Borders, Paragraph},
            text::Text,
            style::{Color, Style}
        };

        let mode_text = match self.current_mode {
            AppMode::Normal => "NORMAL",
            AppMode::Search => "SEARCH",
            AppMode::Command => "COMMAND",
            AppMode::Help => "HELP",
            AppMode::Diff => "DIFF",
            AppMode::AICommit => "AI COMMIT",
        };

        let focus_text = match self.focus_manager.current_panel {
            FocusPanel::Sidebar => "Sidebar",
            FocusPanel::Content => "Content",
            FocusPanel::Detail => "Detail",
        };

        let view_specific_keys = match state.current_view {
            crate::tui_unified::state::app_state::ViewType::GitLog => "p for pull, Enter to view diff",
            crate::tui_unified::state::app_state::ViewType::Branches => "Enter to switch branch, Tab to show remotes",
            crate::tui_unified::state::app_state::ViewType::Tags => "Enter to view tag details", 
            crate::tui_unified::state::app_state::ViewType::Remotes => "Enter to view remote details",
            crate::tui_unified::state::app_state::ViewType::Stash => "Enter to view stash details",
            crate::tui_unified::state::app_state::ViewType::QueryHistory => "Enter to execute query",
        };

        let status_content = format!(
            "[{}] Focus: {} | View: {:?} | {} | Tab-focus, c-AI commit, r-refresh, ?-help, q-quit",
            mode_text,
            focus_text,
            state.current_view,
            view_specific_keys
        );

        let status_bar = Paragraph::new(Text::raw(status_content))
            .block(Block::default().borders(Borders::ALL))
            .style(Style::default().fg(Color::White));

        frame.render_widget(status_bar, area);
    }

    
    /// 使用状态数据渲染界面 (静态方法以避免借用冲突)
    fn render_with_state_static(frame: &mut ratatui::Frame, layout: LayoutResult, state: &AppState, focus_manager: &FocusManager, current_mode: AppMode) {
        use ratatui::{
            widgets::{Block, Borders, Paragraph},
            text::Text,
            style::{Color, Style}
        };
        
        // 侧边栏 - 显示导航菜单和仓库状态
        let sidebar_style = if focus_manager.current_panel == FocusPanel::Sidebar {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::White)
        };
        
        let repo_summary = state.repo_state.get_repo_summary();
        let sidebar_content = format!(
            "📋 Repository: {}\n\n🔀 Branch: {}\n📝 Commits: {}\n🌲 Branches: {}\n🏷️ Tags: {}\n📡 Remotes: {}\n💾 Stashes: {}\n\n{} View Options:\n• [1] Git Log {}\n• [2] Branches\n• [3] Tags\n• [4] Remotes\n• [5] Stash\n• [6] History",
            repo_summary.name,
            if repo_summary.current_branch.is_empty() { "None" } else { &repo_summary.current_branch },
            repo_summary.total_commits,
            repo_summary.total_branches,
            repo_summary.total_tags,
            repo_summary.total_remotes,
            repo_summary.total_stashes,
            match state.current_view {
                crate::tui_unified::state::app_state::ViewType::GitLog => "📊",
                crate::tui_unified::state::app_state::ViewType::Branches => "🌲",
                crate::tui_unified::state::app_state::ViewType::Tags => "🏷️",
                crate::tui_unified::state::app_state::ViewType::Remotes => "📡",
                crate::tui_unified::state::app_state::ViewType::Stash => "💾",
                crate::tui_unified::state::app_state::ViewType::QueryHistory => "📜",
            },
            if matches!(state.current_view, crate::tui_unified::state::app_state::ViewType::GitLog) { "◀" } else { "" }
        );
        
        let sidebar = Paragraph::new(Text::raw(sidebar_content))
            .block(Block::default().title("Menu").borders(Borders::ALL).border_style(sidebar_style));
        frame.render_widget(sidebar, layout.sidebar);
        
        // 主内容区 - 根据当前视图显示不同内容
        let content_style = if focus_manager.current_panel == FocusPanel::Content {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::White)
        };
        
        let content_title = match state.current_view {
            crate::tui_unified::state::app_state::ViewType::GitLog => "Git Log",
            crate::tui_unified::state::app_state::ViewType::Branches => "Branches",
            crate::tui_unified::state::app_state::ViewType::Tags => "Tags",
            crate::tui_unified::state::app_state::ViewType::Remotes => "Remotes",
            crate::tui_unified::state::app_state::ViewType::Stash => "Stash",
            crate::tui_unified::state::app_state::ViewType::QueryHistory => "Query History",
        };
        
        let content_text = match state.current_view {
            crate::tui_unified::state::app_state::ViewType::GitLog => {
                if state.repo_state.commits.is_empty() {
                    "📝 No commits found in repository\n\nThis could mean:\n• Empty repository\n• Git data not yet loaded\n• Repository not accessible".to_string()
                } else {
                    let mut commit_list = String::new();
                    for (_i, commit) in state.repo_state.commits.iter().take(10).enumerate() {
                        let selected = if Some(&commit.hash) == state.selected_items.selected_commit.as_ref() { "► " } else { "  " };
                        commit_list.push_str(&format!(
                            "{}{} {} - {} ({})\n",
                            selected,
                            &commit.short_hash,
                            commit.subject.chars().take(50).collect::<String>(),
                            commit.author,
                            commit.date.format("%Y-%m-%d")
                        ));
                    }
                    if state.repo_state.commits.len() > 10 {
                        commit_list.push_str(&format!("\n... and {} more commits", state.repo_state.commits.len() - 10));
                    }
                    commit_list
                }
            },
            crate::tui_unified::state::app_state::ViewType::Branches => {
                if state.repo_state.branches.is_empty() {
                    "🌲 No branches found\n\nThis could mean:\n• Branch data not yet loaded\n• Repository not accessible".to_string()
                } else {
                    let mut branch_list = String::new();
                    for branch in &state.repo_state.branches {
                        let current = if branch.is_current { "* " } else { "  " };
                        let selected = if Some(&branch.name) == state.selected_items.selected_branch.as_ref() { "► " } else { "  " };
                        let upstream = if let Some(ref upstream) = branch.upstream {
                            format!(" -> {}", upstream)
                        } else {
                            String::new()
                        };
                        branch_list.push_str(&format!(
                            "{}{}{}{}\n",
                            selected, current, branch.name, upstream
                        ));
                    }
                    branch_list
                }
            },
            crate::tui_unified::state::app_state::ViewType::Tags => {
                if state.repo_state.tags.is_empty() {
                    "🏷️ No tags found\n\nThis could mean:\n• No tags created yet\n• Tag data not yet loaded".to_string()
                } else {
                    let mut tag_list = String::new();
                    for tag in &state.repo_state.tags {
                        let selected = if Some(&tag.name) == state.selected_items.selected_tag.as_ref() { "► " } else { "  " };
                        let annotated = if tag.is_annotated { " (annotated)" } else { "" };
                        tag_list.push_str(&format!(
                            "{}{} -> {}{}\n",
                            selected, tag.name, &tag.commit_hash[..8.min(tag.commit_hash.len())], annotated
                        ));
                    }
                    tag_list
                }
            },
            crate::tui_unified::state::app_state::ViewType::Remotes => {
                if state.repo_state.remotes.is_empty() {
                    "📡 No remotes configured\n\nThis could mean:\n• Local repository only\n• Remote data not yet loaded".to_string()
                } else {
                    let mut remote_list = String::new();
                    for remote in &state.repo_state.remotes {
                        let selected = if Some(&remote.name) == state.selected_items.selected_remote.as_ref() { "► " } else { "  " };
                        let default = if remote.is_default { " (default)" } else { "" };
                        remote_list.push_str(&format!(
                            "{}{}{}\n  📍 {}\n",
                            selected, remote.name, default, remote.url
                        ));
                    }
                    remote_list
                }
            },
            crate::tui_unified::state::app_state::ViewType::Stash => {
                if state.repo_state.stashes.is_empty() {
                    "💾 No stashes found\n\nStashes allow you to save work in progress\nwithout making a commit.".to_string()
                } else {
                    let mut stash_list = String::new();
                    for stash in &state.repo_state.stashes {
                        let selected = if Some(&stash.index.to_string()) == state.selected_items.selected_stash.as_ref() { "► " } else { "  " };
                        stash_list.push_str(&format!(
                            "{}stash@{{{}}}: {} - {} files\n  📝 {}\n",
                            selected, stash.index, stash.branch, stash.files_changed, stash.message
                        ));
                    }
                    stash_list
                }
            },
            crate::tui_unified::state::app_state::ViewType::QueryHistory => {
                "📜 Query History\n\nPrevious searches and queries\nwill be displayed here.".to_string()
            },
        };
        
        let content = Paragraph::new(Text::raw(content_text))
            .block(Block::default().title(content_title).borders(Borders::ALL).border_style(content_style));
        frame.render_widget(content, layout.content);
        
        // 详情面板 - 显示选中项的详细信息
        let detail_style = if focus_manager.current_panel == FocusPanel::Detail {
            Style::default().fg(Color::Yellow)  
        } else {
            Style::default().fg(Color::White)
        };
        
        let detail_content = match state.current_view {
            crate::tui_unified::state::app_state::ViewType::GitLog => {
                if let Some(ref selected_commit) = state.selected_items.selected_commit {
                    if let Some(commit) = state.repo_state.get_commit_by_hash(selected_commit) {
                        format!(
                            "🔍 Commit Details\n\nHash: {}\nAuthor: {} <{}>\nDate: {}\nFiles: {} changed\n+{} -{}\n\nMessage:\n{}{}",
                            commit.hash,
                            commit.author,
                            commit.author_email,
                            commit.date.format("%Y-%m-%d %H:%M:%S"),
                            commit.files_changed,
                            commit.insertions,
                            commit.deletions,
                            commit.subject,
                            commit.body.as_ref().map(|b| format!("\n\n{}", b)).unwrap_or_default()
                        )
                    } else {
                        "🔍 Commit Details\n\nNo commit selected or\ncommit not found".to_string()
                    }
                } else {
                    "🔍 Commit Details\n\nSelect a commit from the\nlist to view details".to_string()
                }
            },
            crate::tui_unified::state::app_state::ViewType::Branches => {
                if let Some(ref selected_branch) = state.selected_items.selected_branch {
                    if let Some(branch) = state.repo_state.get_branch_by_name(selected_branch) {
                        format!(
                            "🔍 Branch Details\n\nName: {}\nFull Name: {}\nCurrent: {}\nRemote: {}\nUpstream: {}\nAhead: {} / Behind: {}\nLast Commit: {}\nLast Updated: {}",
                            branch.name,
                            branch.full_name,
                            if branch.is_current { "Yes" } else { "No" },
                            if branch.is_remote { "Yes" } else { "No" },
                            branch.upstream.as_deref().unwrap_or("None"),
                            branch.ahead_count,
                            branch.behind_count,
                            branch.last_commit.as_deref().unwrap_or("Unknown"),
                            branch.last_updated.format("%Y-%m-%d %H:%M:%S")
                        )
                    } else {
                        "🔍 Branch Details\n\nBranch not found".to_string()
                    }
                } else {
                    "🔍 Branch Details\n\nSelect a branch from the\nlist to view details".to_string()
                }
            },
            crate::tui_unified::state::app_state::ViewType::Tags => {
                if let Some(ref selected_tag) = state.selected_items.selected_tag {
                    if let Some(tag) = state.repo_state.get_tag_by_name(selected_tag) {
                        format!(
                            "🔍 Tag Details\n\nName: {}\nCommit: {}\nType: {}\nTagger: {}\nDate: {}\n\nMessage:\n{}",
                            tag.name,
                            &tag.commit_hash[..8.min(tag.commit_hash.len())],
                            if tag.is_annotated { "Annotated" } else { "Lightweight" },
                            tag.tagger.as_deref().unwrap_or("Unknown"),
                            tag.date.format("%Y-%m-%d %H:%M:%S"),
                            tag.message.as_deref().unwrap_or("No message")
                        )
                    } else {
                        "🔍 Tag Details\n\nTag not found".to_string()
                    }
                } else {
                    "🔍 Tag Details\n\nSelect a tag from the\nlist to view details".to_string()
                }
            },
            _ => {
                "🔍 Detail Panel\n\nSelect an item from the\nlist to view details".to_string()
            }
        };
        
        let detail = Paragraph::new(Text::raw(detail_content))
            .block(Block::default().title("Details").borders(Borders::ALL).border_style(detail_style));
        frame.render_widget(detail, layout.detail);
        
        // 状态栏 - 显示更多状态信息
        let status_info = if repo_summary.is_dirty {
            format!("📝 {} changes", repo_summary.pending_changes)
        } else {
            "✅ Clean".to_string()
        };
        
        let branch_info = if !repo_summary.current_branch.is_empty() {
            if repo_summary.ahead_count > 0 || repo_summary.behind_count > 0 {
                format!("🔀 {} (↑{} ↓{})", repo_summary.current_branch, repo_summary.ahead_count, repo_summary.behind_count)
            } else {
                format!("🔀 {}", repo_summary.current_branch)
            }
        } else {
            "🔀 No branch".to_string()
        };
        
        let status_text = format!("{} | {} | {} | Mode: {:?} | Focus: {:?} | [Tab] Switch | [1-6] Views | [q] Quit", 
            branch_info, status_info, content_title, current_mode, focus_manager.current_panel);
            
        let status_bar = Paragraph::new(Text::raw(status_text))
            .block(Block::default().borders(Borders::TOP))
            .style(Style::default().bg(Color::DarkGray));
        frame.render_widget(status_bar, layout.status_bar);
    }
    
    async fn handle_key_event(&mut self, key: KeyEvent) -> Result<()> {
        // 优先检查模态框
        {
            let state = self.state.read().await;
            if state.is_modal_active() {
                drop(state);
                return self.handle_modal_key(key).await;
            }
        }
        
        // 全局按键处理
        match key.code {
            KeyCode::Char('q') => {
                self.should_quit = true;
                return Ok(());
            }
            KeyCode::Char('/') => {
                self.current_mode = AppMode::Search;
                return Ok(());
            }
            KeyCode::Esc => {
                if self.current_mode == AppMode::Search {
                    self.current_mode = AppMode::Normal;
                    self.search_box.clear();
                } else {
                    self.current_mode = AppMode::Normal;
                }
                return Ok(());
            }
            KeyCode::Char('?') => {
                self.current_mode = if self.current_mode == AppMode::Help { 
                    AppMode::Normal 
                } else { 
                    AppMode::Help 
                };
                return Ok(());
            }
            KeyCode::Char('c') => {
                // AI Commit 功能
                if !self.ai_commit_mode {
                    return self.enter_ai_commit_mode().await;
                }
            }
            KeyCode::Tab => {
                if self.current_mode == AppMode::Normal {
                    self.focus_manager.next_focus();
                    return Ok(());
                }
            }
            KeyCode::BackTab => {
                if self.current_mode == AppMode::Normal {
                    self.focus_manager.prev_focus();
                    return Ok(());
                }
            }
            _ => {}
        }

        // 模式特定的按键处理
        match self.current_mode {
            AppMode::Search => {
                self.handle_search_mode_key(key).await?;
            }
            AppMode::Help => {
                // Help模式下只处理退出键
                return Ok(());
            }
            AppMode::Normal => {
                self.handle_normal_mode_key(key).await?;
            }
            _ => {}
        }
        
        Ok(())
    }

    async fn handle_search_mode_key(&mut self, key: KeyEvent) -> Result<()> {
        // 处理搜索框特定事件
        match key.code {
            KeyCode::Enter => {
                let query = self.search_box.get_input().to_string();
                if !query.is_empty() {
                    // 执行搜索
                    self.execute_search(&query).await?;
                }
                self.current_mode = AppMode::Normal;
            }
            _ => {
                // 让搜索框处理其他输入
                let mut state = self.state.write().await;
                let _result = self.search_box.handle_key_event(key, &mut *state);
            }
        }
        
        Ok(())
    }

    async fn handle_normal_mode_key(&mut self, key: KeyEvent) -> Result<()> {
        let current_panel = self.focus_manager.current_panel;
        let mut state = self.state.write().await;
        
        // 首先尝试让获得焦点的组件处理事件
        let handled = match current_panel {
            FocusPanel::Sidebar => {
                self.sidebar_panel.handle_key_event(key, &mut *state)
            }
            FocusPanel::Content => {
                match state.current_view {
                    crate::tui_unified::state::app_state::ViewType::GitLog => {
                        self.git_log_view.handle_key_event(key, &mut *state)
                    }
                    crate::tui_unified::state::app_state::ViewType::Branches => {
                        self.branches_view.handle_key_event(key, &mut *state)
                    }
                    crate::tui_unified::state::app_state::ViewType::Tags => {
                        self.tags_view.handle_key_event(key, &mut *state)
                    }
                    crate::tui_unified::state::app_state::ViewType::Remotes => {
                        self.remotes_view.handle_key_event(key, &mut *state)
                    }
                    crate::tui_unified::state::app_state::ViewType::Stash => {
                        self.stash_view.handle_key_event(key, &mut *state)
                    }
                    crate::tui_unified::state::app_state::ViewType::QueryHistory => {
                        self.query_history_view.handle_key_event(key, &mut *state)
                    }
                }
            }
            _ => EventResult::NotHandled
        };

        // 如果组件没有处理，则处理全局快捷键
        if matches!(handled, EventResult::NotHandled) {
            match key.code {
                KeyCode::Char('1') => {
                    state.set_current_view(crate::tui_unified::state::app_state::ViewType::GitLog);
                    self.focus_manager.set_focus(FocusPanel::Content);
                    // 确保GitLogView有正确的选择状态
                    if !state.repo_state.commits.is_empty() {
                        self.git_log_view.set_focus(true);
                        self.git_log_view.set_selected_index(Some(0));
                    }
                }
                KeyCode::Char('2') => {
                    state.set_current_view(crate::tui_unified::state::app_state::ViewType::Branches);
                    self.focus_manager.set_focus(FocusPanel::Content);
                }
                KeyCode::Char('3') => {
                    state.set_current_view(crate::tui_unified::state::app_state::ViewType::Tags);
                    self.focus_manager.set_focus(FocusPanel::Content);
                }
                KeyCode::Char('4') => {
                    state.set_current_view(crate::tui_unified::state::app_state::ViewType::Remotes);
                    self.focus_manager.set_focus(FocusPanel::Content);
                }
                KeyCode::Char('5') => {
                    state.set_current_view(crate::tui_unified::state::app_state::ViewType::Stash);
                    self.focus_manager.set_focus(FocusPanel::Content);
                }
                KeyCode::Char('6') => {
                    state.set_current_view(crate::tui_unified::state::app_state::ViewType::QueryHistory);
                    self.focus_manager.set_focus(FocusPanel::Content);
                }
                KeyCode::Tab => {
                    // 在侧边栏和内容区之间切换焦点
                    match self.focus_manager.current_panel {
                        FocusPanel::Sidebar => {
                            self.focus_manager.set_focus(FocusPanel::Content);
                        }
                        FocusPanel::Content => {
                            self.focus_manager.set_focus(FocusPanel::Sidebar);
                        }
                        FocusPanel::Detail => {
                            // 从详情区切换到侧边栏
                            self.focus_manager.set_focus(FocusPanel::Sidebar);
                        }
                    }
                }
                KeyCode::Char('r') | KeyCode::Char('R') => {
                    // 释放写锁，然后执行刷新操作
                    let current_view = state.current_view;
                    drop(state);
                    if let Err(e) = self.refresh_current_view(current_view).await {
                        let mut state = self.state.write().await;
                        state.add_notification(
                            format!("Refresh failed: {}", e),
                            crate::tui_unified::state::app_state::NotificationLevel::Error
                        );
                    } else {
                        let mut state = self.state.write().await;
                        state.add_notification(
                            "Refreshed successfully".to_string(),
                            crate::tui_unified::state::app_state::NotificationLevel::Success
                        );
                    }
                    return Ok(()); // 提前返回，因为我们已经处理了状态
                }
                _ => {}
            }
        }
        
        Ok(())
    }

    async fn execute_search(&mut self, query: &str) -> Result<()> {
        use crate::tui_unified::components::base::component::ViewComponent;
        
        let state = self.state.read().await;
        let current_view = state.current_view;
        drop(state); // 释放读锁
        
        match current_view {
            crate::tui_unified::state::app_state::ViewType::GitLog => {
                self.git_log_view.search(query);
            }
            crate::tui_unified::state::app_state::ViewType::Branches => {
                self.branches_view.search(query);
            }
            crate::tui_unified::state::app_state::ViewType::Tags => {
                self.tags_view.search(query);
            }
            crate::tui_unified::state::app_state::ViewType::Remotes => {
                self.remotes_view.search(query);
            }
            crate::tui_unified::state::app_state::ViewType::Stash => {
                self.stash_view.search(query);
            }
            crate::tui_unified::state::app_state::ViewType::QueryHistory => {
                self.query_history_view.search(query);
            }
        }
        
        Ok(())
    }
    
    /// 加载初始Git数据
    async fn load_initial_git_data(&mut self) -> Result<()> {
        // 获取当前目录作为Git仓库路径
        let repo_path = std::env::current_dir()?;
        
        // 创建AsyncGitImpl实例
        let git = crate::tui_unified::git::interface::AsyncGitImpl::new(repo_path.clone());
        
        // 获取写锁访问状态
        let mut state = self.state.write().await;
        
        // 加载基础Git数据
        match git.get_current_branch().await {
            Ok(branch) => {
                state.repo_state.update_current_branch(branch);
            }
            Err(e) => {
                // 如果获取分支失败，可能不是Git仓库，记录但继续
                eprintln!("Warning: Failed to get current branch: {}", e);
            }
        }
        
        // 加载提交历史
        match git.get_commits(Some(100)).await {
            Ok(commits_data) => {
                // 转换为内部数据结构
                let commits: Vec<crate::tui_unified::state::git_state::Commit> = commits_data
                    .into_iter()
                    .map(|c| crate::tui_unified::state::git_state::Commit {
                        hash: c.hash.clone(),
                        short_hash: c.hash[..8.min(c.hash.len())].to_string(),
                        author: c.author.clone(),
                        author_email: format!("{}@example.com", c.author), // Git interface doesn't provide email yet
                        committer: c.author.clone(),
                        committer_email: format!("{}@example.com", c.author),
                        date: chrono::DateTime::parse_from_str(
                            &format!("{} 00:00:00 +0000", c.date), 
                            "%Y-%m-%d %H:%M:%S %z"
                        )
                        .unwrap_or_else(|_| chrono::Utc::now().into())
                        .with_timezone(&chrono::Utc),
                        message: c.message.clone(),
                        subject: c.message,
                        body: None,
                        parents: Vec::new(),
                        refs: Vec::new(),
                        files_changed: c.files_changed as usize,
                        insertions: 0,
                        deletions: 0,
                    })
                    .collect();
                
                state.repo_state.update_commits(commits);
            }
            Err(e) => {
                eprintln!("Warning: Failed to load commits: {}", e);
            }
        }
        
        // 加载分支信息
        match git.get_branches().await {
            Ok(branches_data) => {
                let branches: Vec<crate::tui_unified::state::git_state::Branch> = branches_data
                    .into_iter()
                    .map(|b| crate::tui_unified::state::git_state::Branch {
                        name: b.name.clone(),
                        full_name: format!("refs/heads/{}", b.name),
                        is_current: b.is_current,
                        is_remote: false,
                        upstream: b.upstream,
                        last_commit: None,
                        ahead_count: 0,
                        behind_count: 0,
                        last_updated: chrono::Utc::now(),
                    })
                    .collect();
                
                state.repo_state.update_branches(branches);
            }
            Err(e) => {
                eprintln!("Warning: Failed to load branches: {}", e);
            }
        }
        
        // 加载仓库状态
        match git.get_status().await {
            Ok(status_text) => {
                // 简单的状态解析 - 如果状态文本包含文件变更信息则认为不干净
                let is_clean = status_text.trim() == "Working tree clean";
                
                state.repo_state.update_status(crate::tui_unified::state::git_state::RepoStatus {
                    staged_files: Vec::new(),
                    unstaged_files: Vec::new(),
                    untracked_files: Vec::new(),
                    conflicts: Vec::new(),
                    ahead_count: 0,
                    behind_count: 0,
                    is_clean,
                    is_detached: false,
                });
            }
            Err(e) => {
                eprintln!("Warning: Failed to get repository status: {}", e);
            }
        }

        // 加载标签信息
        match git.get_tags().await {
            Ok(tags_data) => {
                let tags: Vec<crate::tui_unified::state::git_state::Tag> = tags_data
                    .into_iter()
                    .map(|t| crate::tui_unified::state::git_state::Tag {
                        name: t.name,
                        commit_hash: t.commit_hash,
                        message: t.message,
                        tagger: None,
                        date: chrono::Utc::now(), // TODO: Parse actual date from Git
                        is_annotated: true, // TODO: Detect if annotated
                    })
                    .collect();
                
                state.repo_state.update_tags(tags);
            }
            Err(e) => {
                eprintln!("Warning: Failed to load tags: {}", e);
            }
        }

        // 加载远程仓库信息
        match git.get_remotes().await {
            Ok(remotes_data) => {
                let remotes: Vec<crate::tui_unified::state::git_state::Remote> = remotes_data
                    .into_iter()
                    .map(|r| crate::tui_unified::state::git_state::Remote {
                        name: r.name.clone(),
                        url: r.url,
                        fetch_url: r.name.clone(), // TODO: Get actual fetch URL
                        push_url: None,
                        is_default: r.name == "origin",
                    })
                    .collect();
                
                state.repo_state.update_remotes(remotes);
            }
            Err(e) => {
                eprintln!("Warning: Failed to load remotes: {}", e);
            }
        }

        // 加载储藏信息
        match git.get_stashes().await {
            Ok(stashes_data) => {
                let stashes: Vec<crate::tui_unified::state::git_state::Stash> = stashes_data
                    .into_iter()
                    .map(|s| crate::tui_unified::state::git_state::Stash {
                        index: s.index as usize,
                        hash: format!("stash@{{{}}}", s.index), // Use stash reference as hash
                        branch: s.branch,
                        message: s.message,
                        date: chrono::Utc::now(), // TODO: Parse actual date from Git
                        files_changed: 0, // TODO: Get actual file count
                    })
                    .collect();
                
                state.repo_state.update_stashes(stashes);
            }
            Err(e) => {
                eprintln!("Warning: Failed to load stashes: {}", e);
            }
        }
        
        // 释放状态锁
        drop(state);
        
        // 加载各组件的数据
        let state_ref = &*self.state.read().await;
        self.remotes_view.load_remotes(state_ref).await;
        self.stash_view.load_stashes(state_ref).await;
        self.query_history_view.load_history().await;
        
        // 更新GitLogView的commit数据
        let commits = state_ref.repo_state.commits.clone();
        let has_commits = !commits.is_empty();
        self.git_log_view.update_commits(commits);
        
        // 确保GitLogView获得焦点（因为它是默认视图）
        if has_commits {
            self.git_log_view.set_focus(true);
        }
        
        Ok(())
    }
    
    /// 渲染加载状态 (静态方法以避免借用冲突)
    fn render_loading_state_static(frame: &mut ratatui::Frame, layout: LayoutResult) {
        use ratatui::{
            widgets::{Block, Borders, Paragraph},
            text::Text,
            style::{Color, Style}
        };
        
        let loading_style = Style::default().fg(Color::Yellow);
        
        // 侧边栏
        let sidebar = Paragraph::new(Text::raw("📋 Loading Repository...\n\n⏳ Please wait while\nGit data is being loaded"))
            .block(Block::default().title("Menu").borders(Borders::ALL).border_style(loading_style));
        frame.render_widget(sidebar, layout.sidebar);
        
        // 主内容区
        let content = Paragraph::new(Text::raw("🔄 Loading Git Data...\n\nThis may take a moment depending on\nthe size of your repository.\n\nInitializing:\n• Repository status\n• Commit history\n• Branch information\n• Repository metadata"))
            .block(Block::default().title("Loading").borders(Borders::ALL).border_style(loading_style));
        frame.render_widget(content, layout.content);
        
        // 详情面板
        let detail = Paragraph::new(Text::raw("⏳ Initializing...\n\nGit data will be available\nonce loading completes."))
            .block(Block::default().title("Details").borders(Borders::ALL).border_style(loading_style));
        frame.render_widget(detail, layout.detail);
        
        // 状态栏
        let status_text = "🔄 Loading Git repository data... | [q] Quit";
        let status_bar = Paragraph::new(Text::raw(status_text))
            .block(Block::default().borders(Borders::TOP))
            .style(Style::default().bg(Color::DarkGray));
        frame.render_widget(status_bar, layout.status_bar);
    }
    
    /// 清除模态框背景，确保不会有底层内容泄露
    fn clear_modal_background(&self, frame: &mut ratatui::Frame, area: ratatui::layout::Rect) {
        use ratatui::widgets::{Block, Clear, Paragraph};
        use ratatui::text::Text;
        
        // 首先清除整个屏幕区域
        frame.render_widget(Clear, area);
        
        // 创建一个完全不透明的背景填充
        let background_text = " ".repeat((area.width as usize) * (area.height as usize));
        let background_paragraph = Paragraph::new(Text::from(background_text))
            .style(ratatui::style::Style::default()
                .bg(ratatui::style::Color::Black)
                .fg(ratatui::style::Color::Black));
        frame.render_widget(background_paragraph, area);
        
        // 再次渲染一个Block来确保完全遮蔽
        let background_block = Block::default()
            .style(ratatui::style::Style::default()
                .bg(ratatui::style::Color::Black));
        frame.render_widget(background_block, area);
    }

    /// 在指定区域内渲染diff viewer，而不是全屏渲染
    fn render_diff_viewer_in_area(&self, frame: &mut ratatui::Frame, viewer: &DiffViewer, area: ratatui::layout::Rect) {
        use ratatui::{
            widgets::{Block, Borders, Paragraph},
            layout::{Constraint, Direction, Layout},
            text::{Text},
            style::{Color, Style}
        };

        // 主布局：顶部信息栏 + 内容区 + 底部状态栏
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),   // 顶部信息
                Constraint::Min(0),      // 内容区
                Constraint::Length(4),   // 状态栏 (增加高度以显示更多信息)
            ])
            .split(area);
        
        // 渲染顶部信息
        let commit_info_text = format!("Commit: {} | Files: {} | Mode: {}", 
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
            .style(Style::default().fg(Color::White));
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
            .style(Style::default().fg(Color::Yellow));
        frame.render_widget(status_paragraph, main_chunks[2]);
    }

    fn render_diff_content_by_mode(&self, frame: &mut ratatui::Frame, viewer: &DiffViewer, area: ratatui::layout::Rect) {
        use ratatui::{
            widgets::{Block, Borders, Paragraph},
            layout::{Constraint, Direction, Layout},
            style::{Color, Style}
        };

        let diff_content = if !viewer.current_diff.is_empty() {
            viewer.current_diff.clone()
        } else {
            "No diff content available".to_string()
        };

        match viewer.view_mode {
            crate::diff_viewer::DiffViewMode::Unified => {
                // 统一格式：带行号的语法高亮显示
                let lines = self.parse_diff_for_unified(&diff_content);

                // 获取当前文件名，用于显示在标题中
                let current_file_name = if !viewer.files.is_empty() {
                    let file = &viewer.files[viewer.selected_file];
                    // 如果路径太长，截断显示
                    if file.path.len() > 35 {
                        format!("...{}", &file.path[file.path.len()-32..])
                    } else {
                        file.path.clone()
                    }
                } else {
                    "Unknown".to_string()
                };

                let diff_paragraph = Paragraph::new(lines)
                    .block(Block::default().borders(Borders::ALL).title(format!("📄 Unified Diff: {}", current_file_name)))
                    .style(Style::default().fg(Color::White))
                    .scroll((viewer.diff_scroll, 0))
                    .wrap(ratatui::widgets::Wrap { trim: false });
                frame.render_widget(diff_paragraph, area);
            }
            crate::diff_viewer::DiffViewMode::SideBySide => {
                // 并排格式：左右分栏显示
                let horizontal_chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([
                        Constraint::Percentage(50),
                        Constraint::Percentage(50),
                    ])
                    .split(area);

                // 解析diff内容，构建并排视图
                let (left_lines, right_lines) = self.parse_diff_for_side_by_side(&diff_content);

                // 获取当前文件名，用于显示在标题中
                let current_file_name = if !viewer.files.is_empty() {
                    let file = &viewer.files[viewer.selected_file];
                    // 如果路径太长，截断显示
                    if file.path.len() > 35 {
                        format!("...{}", &file.path[file.path.len()-32..])
                    } else {
                        file.path.clone()
                    }
                } else {
                    "Unknown".to_string()
                };

                let left_paragraph = Paragraph::new(left_lines)
                    .block(Block::default().borders(Borders::ALL).title(format!("🔻 Original: {}", current_file_name)))
                    .style(Style::default().fg(Color::White))
                    .scroll((viewer.diff_scroll, 0))
                    .wrap(ratatui::widgets::Wrap { trim: false });
                frame.render_widget(left_paragraph, horizontal_chunks[0]);

                let right_paragraph = Paragraph::new(right_lines)
                    .block(Block::default().borders(Borders::ALL).title(format!("🔺 Modified: {}", current_file_name)))
                    .style(Style::default().fg(Color::White))
                    .scroll((viewer.diff_scroll, 0))
                    .wrap(ratatui::widgets::Wrap { trim: false });
                frame.render_widget(right_paragraph, horizontal_chunks[1]);
            }
            crate::diff_viewer::DiffViewMode::Split => {
                // 分割格式：上下分栏显示
                let vertical_chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Percentage(50),
                        Constraint::Percentage(50),
                    ])
                    .split(area);

                // 解析diff内容，构建上下分割视图
                let (removed_lines, added_lines) = self.parse_diff_for_split(&diff_content);

                // 获取当前文件名，用于显示在标题中
                let current_file_name = if !viewer.files.is_empty() {
                    let file = &viewer.files[viewer.selected_file];
                    // 如果路径太长，截断显示
                    if file.path.len() > 35 {
                        format!("...{}", &file.path[file.path.len()-32..])
                    } else {
                        file.path.clone()
                    }
                } else {
                    "Unknown".to_string()
                };

                let top_paragraph = Paragraph::new(removed_lines)
                    .block(Block::default().borders(Borders::ALL).title(format!("🗑️ Removed (-): {}", current_file_name)))
                    .style(Style::default().fg(Color::White))
                    .scroll((viewer.diff_scroll, 0))
                    .wrap(ratatui::widgets::Wrap { trim: false });
                frame.render_widget(top_paragraph, vertical_chunks[0]);

                let bottom_paragraph = Paragraph::new(added_lines)
                    .block(Block::default().borders(Borders::ALL).title(format!("➕ Added (+): {}", current_file_name)))
                    .style(Style::default().fg(Color::White))
                    .scroll((viewer.diff_scroll, 0))
                    .wrap(ratatui::widgets::Wrap { trim: false });
                frame.render_widget(bottom_paragraph, vertical_chunks[1]);
            }
        }
    }

    /// 解析 diff 内容用于并排显示
    fn parse_diff_for_side_by_side(&self, diff_content: &str) -> (Vec<ratatui::text::Line<'static>>, Vec<ratatui::text::Line<'static>>) {
        use ratatui::{text::{Line, Span}, style::{Color, Style}};
        
        let mut left_lines = Vec::new();
        let mut right_lines = Vec::new();
        let mut old_line_num = 0u32;
        let mut new_line_num = 0u32;
        
        // 收集所有行并按块进行处理
        let lines: Vec<&str> = diff_content.lines().collect();
        let mut i = 0;
        
        while i < lines.len() {
            let line = lines[i];
            
            if line.starts_with("@@") {
                // 解析行号信息：@@ -old_start,old_count +new_start,new_count @@
                if let Some(captures) = line.strip_prefix("@@").and_then(|s| s.strip_suffix("@@")) {
                    let parts: Vec<&str> = captures.trim().split_whitespace().collect();
                    if parts.len() >= 2 {
                        if let Some(old_part) = parts[0].strip_prefix('-') {
                            if let Some((start, _)) = old_part.split_once(',') {
                                old_line_num = start.parse().unwrap_or(0);
                            } else {
                                old_line_num = old_part.parse().unwrap_or(0);
                            }
                        }
                        if let Some(new_part) = parts[1].strip_prefix('+') {
                            if let Some((start, _)) = new_part.split_once(',') {
                                new_line_num = start.parse().unwrap_or(0);
                            } else {
                                new_line_num = new_part.parse().unwrap_or(0);
                            }
                        }
                    }
                }
                
                let header_line = Line::from(Span::styled(line.to_string(), Style::default().fg(Color::Cyan)));
                left_lines.push(header_line.clone());
                right_lines.push(header_line);
                i += 1;
                continue;
            }
            
            if line.starts_with("diff --git") || line.starts_with("index") || 
               line.starts_with("---") || line.starts_with("+++") {
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
                        let line_content = &removed_lines[j][1..];
                        let formatted_line = format!("{:4} │ {}", old_line_num + j as u32, line_content);
                        left_lines.push(Line::from(Span::styled(formatted_line.to_string(), Style::default().fg(Color::Red))));
                    } else {
                        // 没有删除行，左侧显示空行
                        left_lines.push(Line::from(Span::styled("     │".to_string(), Style::default().fg(Color::DarkGray))));
                    }
                    
                    if j < added_lines.len() {
                        // 有添加行，在右侧显示
                        let line_content = &added_lines[j][1..];
                        let formatted_line = format!("{:4} │ {}", new_line_num + j as u32, line_content);
                        right_lines.push(Line::from(Span::styled(formatted_line.to_string(), Style::default().fg(Color::Green))));
                    } else {
                        // 没有添加行，右侧显示空行
                        right_lines.push(Line::from(Span::styled("     │".to_string(), Style::default().fg(Color::DarkGray))));
                    }
                }
                
                old_line_num += removed_lines.len() as u32;
                new_line_num += added_lines.len() as u32;
                
            } else if line.starts_with('+') {
                // 只有添加行（没有前面的删除行）
                let line_content = &line[1..];
                let formatted_line = format!("{:4} │ {}", new_line_num, line_content);
                right_lines.push(Line::from(Span::styled(formatted_line.to_string(), Style::default().fg(Color::Green))));
                
                // 左边显示空行
                left_lines.push(Line::from(Span::styled("     │".to_string(), Style::default().fg(Color::DarkGray))));
                
                new_line_num += 1;
                i += 1;
                
            } else if line.starts_with(' ') {
                // 上下文行：两边都显示
                let line_content = &line[1..];
                let left_formatted = format!("{:4} │ {}", old_line_num, line_content);
                let right_formatted = format!("{:4} │ {}", new_line_num, line_content);
                
                left_lines.push(Line::from(Span::styled(left_formatted.to_string(), Style::default().fg(Color::White))));
                right_lines.push(Line::from(Span::styled(right_formatted.to_string(), Style::default().fg(Color::White))));
                
                old_line_num += 1;
                new_line_num += 1;
                i += 1;
                
            } else if !line.is_empty() {
                // 其他内容行（如文件名等）：两边都显示
                let header_line = Line::from(Span::styled(line.to_string(), Style::default().fg(Color::Yellow)));
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
    fn parse_diff_for_split(&self, diff_content: &str) -> (Vec<ratatui::text::Line<'static>>, Vec<ratatui::text::Line<'static>>) {
        use ratatui::{text::{Line, Span}, style::{Color, Style}};
        
        let mut removed_lines = Vec::new();
        let mut added_lines = Vec::new();
        let mut old_line_num = 0u32;
        let mut new_line_num = 0u32;
        
        for line in diff_content.lines() {
            if line.starts_with("@@") {
                // 解析行号信息
                if let Some(captures) = line.strip_prefix("@@").and_then(|s| s.strip_suffix("@@")) {
                    let parts: Vec<&str> = captures.trim().split_whitespace().collect();
                    if parts.len() >= 2 {
                        if let Some(old_part) = parts[0].strip_prefix('-') {
                            if let Some((start, _)) = old_part.split_once(',') {
                                old_line_num = start.parse().unwrap_or(0);
                            } else {
                                old_line_num = old_part.parse().unwrap_or(0);
                            }
                        }
                        if let Some(new_part) = parts[1].strip_prefix('+') {
                            if let Some((start, _)) = new_part.split_once(',') {
                                new_line_num = start.parse().unwrap_or(0);
                            } else {
                                new_line_num = new_part.parse().unwrap_or(0);
                            }
                        }
                    }
                }
                
                let header_line = Line::from(Span::styled(line.to_string(), Style::default().fg(Color::Cyan)));
                removed_lines.push(header_line.clone());
                added_lines.push(header_line);
                continue;
            }
            
            if line.starts_with("diff --git") || line.starts_with("index") || 
               line.starts_with("---") || line.starts_with("+++") {
                continue;
            }
            
            if line.starts_with('-') {
                // 删除的行
                let line_content = &line[1..];
                let formatted_line = format!("{:4} │ {}", old_line_num, line_content);
                removed_lines.push(Line::from(Span::styled(formatted_line.to_string(), Style::default().fg(Color::Red))));
                old_line_num += 1;
            } else if line.starts_with('+') {
                // 添加的行
                let line_content = &line[1..];
                let formatted_line = format!("{:4} │ {}", new_line_num, line_content);
                added_lines.push(Line::from(Span::styled(formatted_line.to_string(), Style::default().fg(Color::Green))));
                new_line_num += 1;
            } else if line.starts_with(' ') {
                // 上下文行：两边都显示
                let line_content = &line[1..];
                let old_formatted = format!("{:4} │ {}", old_line_num, line_content);
                let new_formatted = format!("{:4} │ {}", new_line_num, line_content);
                
                removed_lines.push(Line::from(Span::styled(old_formatted.to_string(), Style::default().fg(Color::White))));
                added_lines.push(Line::from(Span::styled(new_formatted.to_string(), Style::default().fg(Color::White))));
                
                old_line_num += 1;
                new_line_num += 1;
            } else if !line.is_empty() {
                // 其他内容行
                let header_line = Line::from(Span::styled(line.to_string(), Style::default().fg(Color::Yellow)));
                removed_lines.push(header_line.clone());
                added_lines.push(header_line);
            }
        }
        
        (removed_lines, added_lines)
    }

    /// 解析 diff 内容用于统一显示
    fn parse_diff_for_unified(&self, diff_content: &str) -> Vec<ratatui::text::Line<'static>> {
        use ratatui::{text::{Line, Span}, style::{Color, Style}};
        
        let mut lines = Vec::new();
        let mut old_line_num = 0u32;
        let mut new_line_num = 0u32;
        
        for line in diff_content.lines() {
            if line.starts_with("@@") {
                // 解析行号信息
                if let Some(captures) = line.strip_prefix("@@").and_then(|s| s.strip_suffix("@@")) {
                    let parts: Vec<&str> = captures.trim().split_whitespace().collect();
                    if parts.len() >= 2 {
                        if let Some(old_part) = parts[0].strip_prefix('-') {
                            if let Some((start, _)) = old_part.split_once(',') {
                                old_line_num = start.parse().unwrap_or(0);
                            } else {
                                old_line_num = old_part.parse().unwrap_or(0);
                            }
                        }
                        if let Some(new_part) = parts[1].strip_prefix('+') {
                            if let Some((start, _)) = new_part.split_once(',') {
                                new_line_num = start.parse().unwrap_or(0);
                            } else {
                                new_line_num = new_part.parse().unwrap_or(0);
                            }
                        }
                    }
                }
                lines.push(Line::from(Span::styled(line.to_string(), Style::default().fg(Color::Cyan))));
                continue;
            }
            
            if line.starts_with("diff --git") {
                lines.push(Line::from(Span::styled(line.to_string(), Style::default().fg(Color::Yellow))));
                continue;
            }
            
            if line.starts_with("index") || line.starts_with("---") || line.starts_with("+++") {
                continue;
            }
            
            if line.starts_with('-') {
                // 删除的行
                let line_content = &line[1..];
                let formatted_line = format!("{:4}   │ -{}", old_line_num, line_content);
                lines.push(Line::from(Span::styled(formatted_line.to_string(), Style::default().fg(Color::Red))));
                old_line_num += 1;
            } else if line.starts_with('+') {
                // 添加的行
                let line_content = &line[1..];
                let formatted_line = format!("   {:4} │ +{}", new_line_num, line_content);
                lines.push(Line::from(Span::styled(formatted_line.to_string(), Style::default().fg(Color::Green))));
                new_line_num += 1;
            } else if line.starts_with(' ') {
                // 上下文行
                let line_content = &line[1..];
                let formatted_line = format!("{:4}:{:4} │  {}", old_line_num, new_line_num, line_content);
                lines.push(Line::from(Span::styled(formatted_line.to_string(), Style::default().fg(Color::White))));
                old_line_num += 1;
                new_line_num += 1;
            } else if !line.is_empty() {
                // 其他内容行
                lines.push(Line::from(Span::styled(line.to_string(), Style::default().fg(Color::White))));
            }
        }
        
        lines
    }

    /// 渲染模态框
    fn render_modal(&mut self, frame: &mut ratatui::Frame, modal: &crate::tui_unified::state::app_state::ModalState, area: ratatui::layout::Rect) {
        use ratatui::{
            widgets::{Paragraph},
            layout::{Constraint, Direction, Layout, Alignment},
            text::{Text},
            style::{Color, Style}
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
                
                // 使用自定义的DiffViewer渲染，限制在popup区域内
                if let Some(viewer) = &self.diff_viewer {
                    self.render_diff_viewer_in_area(frame, viewer, popup_area);
                } else {
                    // 如果diff_viewer没有初始化，显示loading
                    let loading_paragraph = ratatui::widgets::Paragraph::new("Loading diff...")
                        .block(ratatui::widgets::Block::default()
                            .borders(ratatui::widgets::Borders::ALL)
                            .title("Diff Viewer"));
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
                    .style(Style::default().fg(Color::Gray))
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
                            self.commit_editor.render(frame, popup_area, &*state);
                        }
                        Err(_) => {
                            // 如果无法获取状态，使用一个静态的虚拟状态
                            static DUMMY_STATE: std::sync::LazyLock<crate::tui_unified::state::AppState> = std::sync::LazyLock::new(|| {
                                crate::tui_unified::state::AppState {
                                    layout: Default::default(),
                                    focus: Default::default(),
                                    current_view: crate::tui_unified::state::app_state::ViewType::GitLog,
                                    modal: None,
                                    repo_state: Default::default(),
                                    selected_items: Default::default(),
                                    search_state: Default::default(),
                                    config: crate::tui_unified::config::AppConfig::default(),
                                    loading_tasks: HashMap::new(),
                                    notifications: Vec::new(),
                                }
                            });
                            self.commit_editor.render(frame, popup_area, &*DUMMY_STATE);
                        }
                    }
                } else {
                    // 非编辑模式：显示生成的消息
                    let ai_commit_content = if let Some(ref message) = self.ai_commit_message {
                        format!("Status: {}\n\n📝 Generated Commit Message:\n\n{}", 
                            self.ai_commit_status.as_ref().unwrap_or(&"Ready".to_string()),
                            message.trim())
                    } else {
                        format!("🤖 {}", self.ai_commit_status.as_ref().unwrap_or(&"Generating commit message...".to_string()))
                    };
                    
                    let ai_commit_block = Paragraph::new(Text::from(ai_commit_content))
                        .block(Block::default()
                            .borders(Borders::ALL)
                            .title("AI Commit")
                            .border_style(Style::default().fg(Color::Green)))
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
                    .block(Block::default()
                        .borders(Borders::ALL)
                        .title(modal.title.clone())
                        .border_style(Style::default().fg(Color::Yellow)))
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
    async fn handle_modal_key(&mut self, key: KeyEvent) -> Result<()> {
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
                            }
                            KeyCode::Char('K') => {
                                viewer.diff_scroll = viewer.diff_scroll.saturating_sub(1);
                            }
                            KeyCode::PageDown => {
                                viewer.diff_scroll = viewer.diff_scroll.saturating_add(10);
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
                            if modal.modal_type == crate::tui_unified::state::app_state::ModalType::GitPull {
                                drop(state); // 显式释放读锁
                                return self.confirm_git_pull().await;
                            }
                            // 在分支切换模式下，Enter确认切换
                            else if modal.modal_type == crate::tui_unified::state::app_state::ModalType::BranchSwitch {
                                drop(state); // 显式释放读锁
                                return self.confirm_branch_switch().await;
                            }
                            // 在AI commit推送提示模式下，Enter等于确认推送
                            else if self.ai_commit_mode && self.ai_commit_push_prompt {
                                drop(state); // 显式释放读锁
                                return self.confirm_push().await;
                            }
                            // 在AI commit模式下按Enter确认提交
                            else if self.ai_commit_mode && !self.ai_commit_editing && self.ai_commit_message.is_some() {
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
                                self.ai_commit_message = Some(edited_content.clone());
                                self.ai_commit_status = Some("Message edited".to_string());
                                
                                // 不需要重新显示模态框，因为渲染逻辑会自动切换到非编辑模式显示
                                // 现在用户可以按 Enter 提交或 Esc 取消
                            }
                        }
                        _ => {
                            // 在AI commit编辑模式下，将键盘事件转发给编辑器
                            if self.ai_commit_mode && self.ai_commit_editing {
                                let mut dummy_state = crate::tui_unified::state::AppState::new(&crate::tui_unified::config::AppConfig::default()).await.unwrap_or_else(|_| {
                                    // 如果创建失败，创建一个基本的虚拟状态
                                    crate::tui_unified::state::AppState {
                                        layout: Default::default(),
                                        focus: Default::default(),
                                        current_view: crate::tui_unified::state::app_state::ViewType::GitLog,
                                        modal: None,
                                        repo_state: Default::default(),
                                        selected_items: Default::default(),
                                        search_state: Default::default(),
                                        config: crate::tui_unified::config::AppConfig::default(),
                                        loading_tasks: HashMap::new(),
                                        notifications: Vec::new(),
                                    }
                                });
                                let _result = self.commit_editor.handle_key_event(key, &mut dummy_state);
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }
    
    /// 处理pending diff请求
    async fn handle_pending_diff_request(&mut self) -> Result<()> {
        // 获取并清除pending diff请求
        let commit_hash = {
            let mut state = self.state.write().await;
            state.get_pending_diff_commit()
        };
        
        if let Some(hash) = commit_hash {
            // 获取当前目录作为Git仓库路径
            let repo_path = std::env::current_dir()?;
            let _git = crate::tui_unified::git::interface::AsyncGitImpl::new(repo_path);
            
            // 创建DiffViewer实例
            match DiffViewer::new(&hash).await {
                Ok(diff_viewer) => {
                    // 保存diff_viewer实例
                    self.diff_viewer = Some(diff_viewer);
                    
                    // 显示diff弹窗（传入空的内容，因为DiffViewer自己管理内容）
                    let mut state = self.state.write().await;
                    state.show_diff_modal(hash, String::new());
                }
                Err(e) => {
                    // 显示错误通知
                    let mut state = self.state.write().await;
                    state.add_notification(
                        format!("Failed to create diff viewer: {}", e),
                        crate::tui_unified::state::app_state::NotificationLevel::Error
                    );
                }
            }
        }
        
        Ok(())
    }

    async fn handle_direct_branch_switch_request(&mut self) -> Result<()> {
        // 获取并清除直接分支切换请求
        let branch_name = {
            let mut state = self.state.write().await;
            state.get_direct_branch_switch()
        };
        
        if let Some(branch_name) = branch_name {
            // 直接切换分支
            self.checkout_branch_directly(&branch_name).await?;
        }
        
        Ok(())
    }
    
    /// 重新加载 Git 数据（在提交后刷新）
    async fn reload_git_data(&mut self) -> Result<()> {
        // 直接调用现有的加载逻辑
        self.load_initial_git_data().await
    }

    /// 刷新当前视图的数据
    async fn refresh_current_view(&mut self, view_type: crate::tui_unified::state::app_state::ViewType) -> Result<()> {
        match view_type {
            crate::tui_unified::state::app_state::ViewType::GitLog => {
                self.refresh_git_log().await
            }
            crate::tui_unified::state::app_state::ViewType::Branches => {
                self.refresh_branches().await
            }
            crate::tui_unified::state::app_state::ViewType::Tags => {
                self.refresh_tags().await
            }
            crate::tui_unified::state::app_state::ViewType::Remotes => {
                self.refresh_remotes().await
            }
            crate::tui_unified::state::app_state::ViewType::Stash => {
                self.refresh_stash().await
            }
            crate::tui_unified::state::app_state::ViewType::QueryHistory => {
                self.refresh_query_history().await
            }
        }
    }
    
    /// 进入 AI commit 模式
    async fn enter_ai_commit_mode(&mut self) -> Result<()> {
        // 使用新的函数获取所有变更（包括未暂存的）
        let diff = match crate::git::get_all_changes_diff().await {
            Ok(diff) => {
                if diff.trim().is_empty() {
                    let mut state = self.state.write().await;
                    state.add_notification(
                        "No changes to commit".to_string(),
                        crate::tui_unified::state::app_state::NotificationLevel::Warning
                    );
                    return Ok(());
                }
                diff
            }
            Err(e) => {
                let mut state = self.state.write().await;
                state.add_notification(
                    format!("Failed to get changes: {}", e),
                    crate::tui_unified::state::app_state::NotificationLevel::Error
                );
                return Ok(());
            }
        };

        // 初始化 Agent Manager（如果还没有）
        if self.agent_manager.is_none() {
            let agent_manager = AgentManager::with_default_context();
            self.agent_manager = Some(agent_manager);
        }

        // 设置状态
        self.ai_commit_mode = true;
        self.ai_commit_status = Some("Generating commit message...".to_string());
        self.current_mode = AppMode::AICommit;

        // 显示 AI commit 模态框
        {
            let mut state = self.state.write().await;
            state.show_ai_commit_modal(
                "".to_string(), 
                "Generating commit message...".to_string()
            );
        }

        // 生成 commit message
        self.generate_commit_message(diff).await
    }

    /// 生成 AI commit message
    async fn generate_commit_message(&mut self, diff: String) -> Result<()> {
        if let Some(ref mut agent_manager) = self.agent_manager {
            // 创建配置
            let config = Config::new();
            
            // 更新 Agent 配置
            let mut env_vars = std::env::vars().collect::<HashMap<String, String>>();
            
            // 添加 API Key 配置
            if let Some(api_key) = config.get_api_key() {
                env_vars.insert("API_KEY".to_string(), api_key);
            }
            
            // 设置 API URL
            let api_url = config.get_url();
            env_vars.insert("API_URL".to_string(), api_url);
            
            let agent_config = AgentConfig {
                provider: config.provider.clone(),
                model: config.model.clone(),
                temperature: 0.7,
                max_tokens: 2000,
                stream: true,
                max_retries: 3,
                timeout_secs: 60,
            };
            
            let context = AgentContext {
                working_dir: std::env::current_dir()?,
                env_vars,
                config: agent_config,
                history: vec![],
            };
            
            // 更新管理器上下文
            agent_manager.update_context(context);
            
            // 获取或创建 Commit Agent
            match agent_manager.get_or_create_agent("commit").await {
                Ok(commit_agent) => {
                    // 创建任务
                    let task = AgentTask::new(TaskType::GenerateCommit, diff);
                    
                    // 执行任务
                    match commit_agent.execute(task, agent_manager.context()).await {
                        Ok(result) => {
                            if result.success {
                                self.ai_commit_message = Some(result.content.clone());
                                self.ai_commit_status = Some("Commit message generated successfully".to_string());
                                
                                // 更新模态框内容
                                let mut state = self.state.write().await;
                                state.show_ai_commit_modal(
                                    result.content, 
                                    "Commit message generated successfully".to_string()
                                );
                            } else {
                                self.ai_commit_status = Some("Failed to generate commit message".to_string());
                                
                                // 更新模态框内容
                                let mut state = self.state.write().await;
                                state.show_ai_commit_modal(
                                    "".to_string(), 
                                    "Failed to generate commit message".to_string()
                                );
                            }
                        }
                        Err(e) => {
                            self.ai_commit_status = Some(format!("Error: {}", e));
                            
                            // 更新模态框内容
                            let mut state = self.state.write().await;
                            state.show_ai_commit_modal(
                                "".to_string(), 
                                format!("Error: {}", e)
                            );
                        }
                    }
                }
                Err(e) => {
                    self.ai_commit_status = Some(format!("Failed to create agent: {}", e));
                    
                    // 更新模态框内容
                    let mut state = self.state.write().await;
                    state.show_ai_commit_modal(
                        "".to_string(), 
                        format!("Failed to create agent: {}", e)
                    );
                }
            }
        }
        
        Ok(())
    }

    /// 确认并提交 AI 生成的 commit message
    async fn confirm_ai_commit(&mut self) -> Result<()> {
        if let Some(ref message) = self.ai_commit_message {
            // 首先检查是否有暂存的变更
            let staged_diff = match crate::git::get_git_diff().await {
                Ok(diff) => diff,
                Err(e) => {
                    let mut state = self.state.write().await;
                    state.add_notification(
                        format!("Failed to check staged changes: {}", e),
                        crate::tui_unified::state::app_state::NotificationLevel::Error
                    );
                    return Ok(());
                }
            };
            
            // 如果没有暂存变更，先自动添加所有变更
            if staged_diff.trim().is_empty() {
                match crate::git::git_add_all().await {
                    Ok(_) => {
                        let mut state = self.state.write().await;
                        state.add_notification(
                            "Changes staged automatically".to_string(),
                            crate::tui_unified::state::app_state::NotificationLevel::Info
                        );
                        drop(state);
                    }
                    Err(e) => {
                        let mut state = self.state.write().await;
                        state.add_notification(
                            format!("Failed to stage changes: {}", e),
                            crate::tui_unified::state::app_state::NotificationLevel::Error
                        );
                        return Ok(());
                    }
                }
            }
            
            // 现在执行提交
            match crate::git::git_commit(message).await {
                Ok(_) => {
                    let mut state = self.state.write().await;
                    state.add_notification(
                        "Commit successful!".to_string(),
                        crate::tui_unified::state::app_state::NotificationLevel::Info
                    );
                    drop(state);
                    
                    // 重新加载 Git 数据以显示新的提交
                    if let Err(e) = self.reload_git_data().await {
                        let mut state = self.state.write().await;
                        state.add_notification(
                            format!("Failed to reload git data: {}", e),
                            crate::tui_unified::state::app_state::NotificationLevel::Warning
                        );
                        drop(state);
                    }
                    
                    // 显示推送提示而不是立即退出
                    self.ai_commit_push_prompt = true;
                    self.ai_commit_status = Some("Commit successful! Push to remote?".to_string());
                    
                    // 显示推送提示模态框
                    let mut state = self.state.write().await;
                    state.show_ai_commit_push_modal("Commit successful!".to_string());
                }
                Err(e) => {
                    let mut state = self.state.write().await;
                    state.add_notification(
                        format!("Commit failed: {}", e),
                        crate::tui_unified::state::app_state::NotificationLevel::Error
                    );
                }
            }
        }
        
        Ok(())
    }

    /// 退出 AI commit 模式
    fn exit_ai_commit_mode(&mut self) {
        self.ai_commit_mode = false;
        self.ai_commit_editing = false;
        self.ai_commit_message = None;
        self.ai_commit_status = None;
        self.ai_commit_push_prompt = false;
        self.current_mode = AppMode::Normal;
        
        // 重置编辑器状态
        self.commit_editor.set_focused(false);
        self.commit_editor.set_content("");
    }

    /// 确认推送到远程
    async fn confirm_push(&mut self) -> Result<()> {
        // 执行 git push
        match crate::git::git_push().await {
            Ok(_) => {
                let mut state = self.state.write().await;
                state.add_notification(
                    "Push successful!".to_string(),
                    crate::tui_unified::state::app_state::NotificationLevel::Success
                );
                state.hide_modal();
                drop(state);
                
                // 完成推送后退出AI commit模式
                self.exit_ai_commit_mode();
            }
            Err(e) => {
                let mut state = self.state.write().await;
                state.add_notification(
                    format!("Push failed: {}", e),
                    crate::tui_unified::state::app_state::NotificationLevel::Error
                );
                // 推送失败时不退出AI commit模式，让用户可以重试
                self.ai_commit_status = Some(format!("Push failed: {}", e));
                state.show_ai_commit_push_modal(format!("Push failed: {}. Try again?", e));
            }
        }
        
        Ok(())
    }

    async fn confirm_git_pull(&mut self) -> Result<()> {
        // 隐藏模态框
        {
            let mut state = self.state.write().await;
            state.hide_modal();
        }
        
        // 执行 git pull
        let result = tokio::process::Command::new("git")
            .arg("pull")
            .output()
            .await;
            
        match result {
            Ok(output) => {
                let mut state = self.state.write().await;
                if output.status.success() {
                    let pull_output = String::from_utf8_lossy(&output.stdout);
                    let notification_msg = if pull_output.contains("Already up to date") {
                        "Already up to date".to_string()
                    } else {
                        "Pull completed successfully!".to_string()
                    };
                    
                    state.add_notification(
                        notification_msg,
                        crate::tui_unified::state::app_state::NotificationLevel::Success
                    );
                    drop(state);
                    
                    // 拉取成功后刷新git log
                    if let Err(e) = self.refresh_current_view(crate::tui_unified::state::app_state::ViewType::GitLog).await {
                        let mut state = self.state.write().await;
                        state.add_notification(
                            format!("Failed to refresh git log: {}", e),
                            crate::tui_unified::state::app_state::NotificationLevel::Warning
                        );
                    }
                } else {
                    let error_output = String::from_utf8_lossy(&output.stderr);
                    state.add_notification(
                        format!("Pull failed: {}", error_output),
                        crate::tui_unified::state::app_state::NotificationLevel::Error
                    );
                }
            }
            Err(e) => {
                let mut state = self.state.write().await;
                state.add_notification(
                    format!("Failed to execute git pull: {}", e),
                    crate::tui_unified::state::app_state::NotificationLevel::Error
                );
            }
        }
        
        Ok(())
    }

    /// 直接切换分支（仿照 tui_enhanced 的实现）
    async fn checkout_branch_directly(&mut self, branch_name: &str) -> Result<()> {
        let output = tokio::process::Command::new("git")
            .args(["checkout", branch_name])
            .output()
            .await?;
        
        let mut state = self.state.write().await;
        if output.status.success() {
            state.add_notification(
                format!("Switched to branch '{}'", branch_name),
                crate::tui_unified::state::app_state::NotificationLevel::Success
            );
            drop(state);
            
            // 重新加载分支列表和提交记录
            let _ = self.refresh_current_view(crate::tui_unified::state::app_state::ViewType::Branches).await;
            let _ = self.refresh_current_view(crate::tui_unified::state::app_state::ViewType::GitLog).await;
        } else {
            let error = String::from_utf8_lossy(&output.stderr);
            state.add_notification(
                format!("Failed to switch branch: {}", error),
                crate::tui_unified::state::app_state::NotificationLevel::Error
            );
        }
        
        Ok(())
    }

    async fn confirm_branch_switch(&mut self) -> Result<()> {
        // 获取待切换的分支名
        let branch_name = {
            let mut state = self.state.write().await;
            state.hide_modal();
            state.get_pending_branch_switch()
        };
        
        let branch_name = match branch_name {
            Some(name) => name,
            None => {
                let mut state = self.state.write().await;
                state.add_notification(
                    "No branch selected for switching".to_string(),
                    crate::tui_unified::state::app_state::NotificationLevel::Error
                );
                return Ok(());
            }
        };
        
        // 执行分支切换
        let switch_result = tokio::process::Command::new("git")
            .arg("checkout")
            .arg(&branch_name)
            .output()
            .await;
            
        match switch_result {
            Ok(output) => {
                let mut state = self.state.write().await;
                if output.status.success() {
                    state.add_notification(
                        format!("Switched to branch '{}'", branch_name),
                        crate::tui_unified::state::app_state::NotificationLevel::Success
                    );
                    drop(state);
                    
                    // 分支切换成功后刷新相关视图
                    let _ = self.refresh_current_view(crate::tui_unified::state::app_state::ViewType::Branches).await;
                    let _ = self.refresh_current_view(crate::tui_unified::state::app_state::ViewType::GitLog).await;
                } else {
                    let error_output = String::from_utf8_lossy(&output.stderr);
                    state.add_notification(
                        format!("Failed to switch branch: {}", error_output),
                        crate::tui_unified::state::app_state::NotificationLevel::Error
                    );
                }
            }
            Err(e) => {
                let mut state = self.state.write().await;
                state.add_notification(
                    format!("Failed to execute git checkout: {}", e),
                    crate::tui_unified::state::app_state::NotificationLevel::Error
                );
            }
        }
        
        Ok(())
    }

    /// 跳过推送
    fn skip_push(&mut self) {
        // 关闭模态框并退出AI commit模式
        self.exit_ai_commit_mode();
    }

    /// 刷新Git Log视图
    async fn refresh_git_log(&mut self) -> Result<()> {
        let repo_path = std::env::current_dir()?;
        let git = crate::tui_unified::git::interface::AsyncGitImpl::new(repo_path);
        
        match git.get_commits(Some(100)).await {
            Ok(commits_data) => {
                // 转换为内部数据结构
                let commits: Vec<crate::tui_unified::state::git_state::Commit> = commits_data
                    .into_iter()
                    .map(|c| crate::tui_unified::state::git_state::Commit {
                        hash: c.hash.clone(),
                        short_hash: c.hash[..8.min(c.hash.len())].to_string(),
                        author: c.author.clone(),
                        author_email: format!("{}@example.com", c.author),
                        committer: c.author.clone(),
                        committer_email: format!("{}@example.com", c.author),
                        date: chrono::DateTime::parse_from_str(
                            &format!("{} 00:00:00 +0000", c.date), 
                            "%Y-%m-%d %H:%M:%S %z"
                        )
                        .unwrap_or_else(|_| chrono::Utc::now().into())
                        .with_timezone(&chrono::Utc),
                        message: c.message.clone(),
                        subject: c.message,
                        body: None,
                        parents: Vec::new(),
                        refs: Vec::new(),
                        files_changed: c.files_changed as usize,
                        insertions: 0,
                        deletions: 0,
                    })
                    .collect();
                
                // 更新状态
                let mut state = self.state.write().await;
                state.repo_state.update_commits(commits.clone());
                drop(state);
                
                // 更新GitLogView
                self.git_log_view.update_commits(commits);
                Ok(())
            }
            Err(e) => Err(anyhow::anyhow!("Git operation failed: {}", e).into())
        }
    }

    /// 刷新Branches视图
    async fn refresh_branches(&mut self) -> Result<()> {
        let repo_path = std::env::current_dir()?;
        let git = crate::tui_unified::git::interface::AsyncGitImpl::new(repo_path);
        
        match git.get_branches().await {
            Ok(branches_data) => {
                let branches: Vec<crate::tui_unified::state::git_state::Branch> = branches_data
                    .into_iter()
                    .map(|b| crate::tui_unified::state::git_state::Branch {
                        name: b.name.clone(),
                        full_name: format!("refs/heads/{}", b.name),
                        is_current: b.is_current,
                        is_remote: false,
                        upstream: b.upstream,
                        last_commit: None,
                        ahead_count: 0,
                        behind_count: 0,
                        last_updated: chrono::Utc::now(),
                    })
                    .collect();
                
                // 更新状态
                let mut state = self.state.write().await;
                state.repo_state.update_branches(branches);
                Ok(())
            }
            Err(e) => Err(anyhow::anyhow!("Git operation failed: {}", e).into())
        }
    }

    /// 刷新Tags视图
    async fn refresh_tags(&mut self) -> Result<()> {
        let repo_path = std::env::current_dir()?;
        let git = crate::tui_unified::git::interface::AsyncGitImpl::new(repo_path);
        
        match git.get_tags().await {
            Ok(tags_data) => {
                let tags: Vec<crate::tui_unified::state::git_state::Tag> = tags_data
                    .into_iter()
                    .map(|t| crate::tui_unified::state::git_state::Tag {
                        name: t.name,
                        commit_hash: t.commit_hash,
                        message: t.message,
                        tagger: None,
                        date: chrono::Utc::now(),
                        is_annotated: true,
                    })
                    .collect();
                
                // 更新状态
                let mut state = self.state.write().await;
                state.repo_state.update_tags(tags);
                Ok(())
            }
            Err(e) => Err(anyhow::anyhow!("Git operation failed: {}", e).into())
        }
    }

    /// 刷新Remotes视图
    async fn refresh_remotes(&mut self) -> Result<()> {
        let repo_path = std::env::current_dir()?;
        let git = crate::tui_unified::git::interface::AsyncGitImpl::new(repo_path);
        
        match git.get_remotes().await {
            Ok(remotes_data) => {
                let remotes: Vec<crate::tui_unified::state::git_state::Remote> = remotes_data
                    .into_iter()
                    .map(|r| crate::tui_unified::state::git_state::Remote {
                        name: r.name.clone(),
                        url: r.url,
                        fetch_url: r.name.clone(),
                        push_url: None,
                        is_default: r.name == "origin",
                    })
                    .collect();
                
                // 更新状态并通知视图
                let mut state = self.state.write().await;
                state.repo_state.update_remotes(remotes);
                let state_ref = &*state;
                drop(state);
                
                // 通知RemotesView重新加载数据
                let state_ref = &*self.state.read().await;
                self.remotes_view.load_remotes(state_ref).await;
                Ok(())
            }
            Err(e) => Err(anyhow::anyhow!("Git operation failed: {}", e).into())
        }
    }

    /// 刷新Stash视图
    async fn refresh_stash(&mut self) -> Result<()> {
        let repo_path = std::env::current_dir()?;
        let git = crate::tui_unified::git::interface::AsyncGitImpl::new(repo_path);
        
        match git.get_stashes().await {
            Ok(stashes_data) => {
                let stashes: Vec<crate::tui_unified::state::git_state::Stash> = stashes_data
                    .into_iter()
                    .map(|s| crate::tui_unified::state::git_state::Stash {
                        index: s.index as usize,
                        hash: format!("stash@{{{}}}", s.index),
                        branch: s.branch,
                        message: s.message,
                        date: chrono::Utc::now(),
                        files_changed: 0,
                    })
                    .collect();
                
                // 更新状态并通知视图
                let mut state = self.state.write().await;
                state.repo_state.update_stashes(stashes);
                let state_ref = &*state;
                drop(state);
                
                // 通知StashView重新加载数据
                let state_ref = &*self.state.read().await;
                self.stash_view.load_stashes(state_ref).await;
                Ok(())
            }
            Err(e) => Err(anyhow::anyhow!("Git operation failed: {}", e).into())
        }
    }

    /// 刷新Query History视图
    async fn refresh_query_history(&mut self) -> Result<()> {
        // 重新加载查询历史
        self.query_history_view.load_history().await;
        Ok(())
    }
}

// 为了编译成功，先创建一些基础结构
pub struct LayoutResult {
    pub sidebar: ratatui::layout::Rect,
    pub content: ratatui::layout::Rect,
    pub detail: ratatui::layout::Rect,
    pub status_bar: ratatui::layout::Rect,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_tui_app_creation() {
        let app = TuiUnifiedApp::new().await;
        assert!(app.is_ok(), "TUI app should be created successfully");
        
        let app = app.unwrap();
        assert_eq!(app.current_mode, AppMode::Normal);
        assert!(!app.should_quit);
    }
    
    #[test]
    fn test_app_mode_transitions() {
        let mut mode = AppMode::Normal;
        assert_eq!(mode, AppMode::Normal);  // 验证初始状态
        
        // Normal -> Help
        mode = AppMode::Help;
        assert_eq!(mode, AppMode::Help);
        
        // Help -> Normal (via ESC)
        mode = AppMode::Normal;
        assert_eq!(mode, AppMode::Normal);
    }
}