use crossterm::{
    event::{self, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::core::ai::agents::manager::AgentManager;
use crate::diff_viewer::DiffViewer;
use crate::tui_unified::{
    components::{
        panels::sidebar::SidebarPanel,
        views::{
            branches::BranchesView, git_log::GitLogView, query_history::QueryHistoryView,
            remotes::RemotesView, stash::StashView, tags::TagsView,
        },
        widgets::{commit_editor::CommitEditor, search_box::SearchBox},
    },
    config::AppConfig,
    focus::{FocusManager, FocusPanel},
    layout::LayoutManager,
    state::AppState,
    Result,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AppMode {
    Normal,   // 正常浏览模式
    Search,   // 搜索模式
    Command,  // 命令模式
    Help,     // 帮助模式
    Diff,     // 全屏diff模式
    AICommit, // AI提交模式
}

pub struct TuiUnifiedApp {
    // 核心状态
    pub(crate) state: Arc<RwLock<AppState>>,

    // 管理器
    pub(crate) layout_manager: LayoutManager,
    pub(crate) focus_manager: FocusManager,

    // 组件
    pub(crate) sidebar_panel: SidebarPanel,
    pub(crate) git_log_view: GitLogView,
    pub(crate) branches_view: BranchesView,
    pub(crate) tags_view: TagsView,
    pub(crate) remotes_view: RemotesView,
    pub(crate) stash_view: StashView,
    pub(crate) query_history_view: QueryHistoryView,
    pub(crate) search_box: SearchBox,
    pub(crate) diff_viewer: Option<DiffViewer>,
    pub(crate) commit_editor: CommitEditor,

    // 配置
    pub(crate) _config: AppConfig,

    // 运行状态
    pub(crate) should_quit: bool,
    pub(crate) current_mode: AppMode,

    // AI commit 功能
    pub(crate) agent_manager: Option<AgentManager>,
    pub(crate) ai_commit_message: Option<String>,
    pub(crate) ai_commit_mode: bool,
    pub(crate) ai_commit_editing: bool,
    pub(crate) ai_commit_status: Option<String>,
    pub(crate) ai_commit_push_prompt: bool,
}

impl TuiUnifiedApp {
    pub async fn new() -> Result<Self> {
        let config = AppConfig::load().unwrap_or_default();
        let state = Arc::new(RwLock::new(AppState::new(&config).await?));

        let mut focus_manager = FocusManager::new();
        focus_manager.set_focus(FocusPanel::Content);

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

            agent_manager: None,
            ai_commit_message: None,
            ai_commit_mode: false,
            ai_commit_editing: false,
            ai_commit_status: None,
            ai_commit_push_prompt: false,
        })
    }

    pub async fn run() -> Result<()> {
        enable_raw_mode()?;
        let mut stdout = std::io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let mut app = Self::new().await?;

        let result = app.run_loop(&mut terminal).await;

        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
        terminal.show_cursor()?;

        result
    }

    async fn run_loop<B>(&mut self, terminal: &mut Terminal<B>) -> Result<()>
    where
        B: ratatui::backend::Backend,
    {
        self.load_initial_git_data().await?;

        loop {
            terminal.draw(|f| self.render(f))?;

            if event::poll(std::time::Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    self.handle_key_event(key).await?;
                }
            }

            self.handle_pending_diff_request().await?;
            self.handle_direct_branch_switch_request().await?;

            if self.should_quit {
                break;
            }
        }

        Ok(())
    }
}

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
        assert_eq!(mode, AppMode::Normal);

        mode = AppMode::Help;
        assert_eq!(mode, AppMode::Help);

        mode = AppMode::Normal;
        assert_eq!(mode, AppMode::Normal);
    }
}
