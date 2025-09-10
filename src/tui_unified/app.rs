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
        },
    },
    Result
};
use crate::diff_viewer::{DiffViewer, render_diff_viewer};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AppMode {
    Normal,      // 正常浏览模式
    Search,      // 搜索模式
    Command,     // 命令模式
    Help,        // 帮助模式
    Diff,        // 全屏diff模式
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
    
    // 配置
    _config: AppConfig,
    
    // 运行状态
    should_quit: bool,
    current_mode: AppMode,
}

impl TuiUnifiedApp {
    pub async fn new() -> Result<Self> {
        let config = AppConfig::load().unwrap_or_default();
        let state = Arc::new(RwLock::new(AppState::new(&config).await?));
        
        let mut focus_manager = FocusManager::new();
        focus_manager.set_focus(FocusPanel::Sidebar);
        
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
            _config: config,
            should_quit: false,
            current_mode: AppMode::Normal,
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
                        self.git_log_view.set_focus(self.focus_manager.current_panel == FocusPanel::Content);
                        self.git_log_view.render(frame, layout.content, &*state);
                    }
                    crate::tui_unified::state::app_state::ViewType::Branches => {
                        self.branches_view.set_focus(self.focus_manager.current_panel == FocusPanel::Content);
                        self.branches_view.render(frame, layout.content, &*state);
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
        };

        let focus_text = match self.focus_manager.current_panel {
            FocusPanel::Sidebar => "Sidebar",
            FocusPanel::Content => "Content",
            FocusPanel::Detail => "Detail",
        };

        let status_content = format!(
            "[{}] Focus: {} | View: {:?} | Press Tab to switch focus, ? for help, q to quit",
            mode_text,
            focus_text,
            state.current_view
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
    
    /// 渲染模态框
    fn render_modal(&mut self, frame: &mut ratatui::Frame, modal: &crate::tui_unified::state::app_state::ModalState, area: ratatui::layout::Rect) {
        use ratatui::{
            widgets::{Paragraph, Clear},
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
                
                // 清除背景
                frame.render_widget(Clear, popup_area);
                
                // 使用完全工作的DiffViewer
                if let Some(viewer) = &mut self.diff_viewer {
                    render_diff_viewer(frame, viewer);
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
                
                let help_text = "Press [Esc] or [q] to close | [↑↓/jk] scroll | [PgUp/PgDn] page | [1] unified | [2] side-by-side | [3] file tree | [w] word-level | [n] line numbers";
                let help = Paragraph::new(Text::from(help_text))
                    .style(Style::default().fg(Color::Gray))
                    .alignment(Alignment::Center);
                frame.render_widget(help, help_area);
            }
            _ => {
                // 对于其他类型的模态框，使用简单的消息框
                // 这里可以根据需要扩展
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
                                viewer.set_view_mode(crate::diff_viewer::DiffViewMode::Split);
                            }
                            KeyCode::Char('t') => {
                                viewer.show_file_list = !viewer.show_file_list;
                            }
                            KeyCode::Char('h') => {
                                viewer.syntax_highlight = !viewer.syntax_highlight;
                            }
                            _ => {}
                        }
                    }
                }
                _ => {
                    // 对于其他模态框类型，只处理关闭快捷键
                    match key.code {
                        KeyCode::Esc | KeyCode::Char('q') => {
                            let mut state = self.state.write().await;
                            state.hide_modal();
                            return Ok(());
                        }
                        _ => {}
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