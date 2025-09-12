use super::app::{AppMode, LayoutResult, TuiUnifiedApp};
use crate::tui_unified::components::base::component::Component;
use crate::tui_unified::focus::FocusPanel;
use crate::tui_unified::state::AppState;

impl TuiUnifiedApp {
    pub(crate) fn render(&mut self, frame: &mut ratatui::Frame) {
        // 计算布局
        let layout = self.layout_manager.calculate_layout(frame.size());

        // 检查是否能获取状态读锁
        let modal_info = match self.state.try_read() {
            Ok(state) => {
                let modal_clone = state.modal.clone();
                (true, modal_clone)
            }
            Err(_) => (false, None),
        };

        match self.state.try_write() {
            Ok(mut state) => {
                // 设置组件焦点状态
                self.sidebar_panel
                    .set_focus(self.focus_manager.current_panel == FocusPanel::Sidebar);

                let current_view = state.current_view;

                // 渲染侧边栏
                self.sidebar_panel.render(frame, layout.sidebar, &state);

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

                        let content_focused =
                            self.focus_manager.current_panel == FocusPanel::Content;

                        // 渲染git log
                        self.git_log_view.set_focus(content_focused);
                        self.git_log_view.render(frame, chunks[0], &state);

                        // 渲染分支列表
                        self.branches_view.set_focus(false); // 分支列表在git log视图中不获得焦点
                        self.branches_view.render(frame, chunks[1], &state);
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

                        let content_focused =
                            self.focus_manager.current_panel == FocusPanel::Content;

                        // 确保选中分支状态是最新的
                        self.branches_view
                            .update_selected_branch_in_state(&mut state);

                        // 渲染分支列表
                        self.branches_view.set_focus(content_focused);
                        self.branches_view.render(frame, chunks[0], &state);

                        // 根据选中的分支更新Git Log并渲染
                        let selected_branch = state.selected_items.selected_branch.clone();
                        self.git_log_view.set_branch_filter(selected_branch.clone());

                        // 获取并显示选中分支的提交历史
                        let commits_to_show = if let Some(ref branch_name) = selected_branch {
                            self.get_branch_commits_sync(branch_name)
                                .unwrap_or_else(|_| state.repo_state.commits.clone())
                        } else {
                            state.repo_state.commits.clone()
                        };

                        self.git_log_view.update_commits(commits_to_show);
                        self.git_log_view.set_focus(false); // git log在分支视图中不获得焦点
                        self.git_log_view.render(frame, chunks[1], &state);
                    }
                    crate::tui_unified::state::app_state::ViewType::Tags => {
                        self.tags_view
                            .set_focus(self.focus_manager.current_panel == FocusPanel::Content);
                        self.tags_view.render(frame, layout.content, &state);
                    }
                    crate::tui_unified::state::app_state::ViewType::Remotes => {
                        self.remotes_view
                            .set_focus(self.focus_manager.current_panel == FocusPanel::Content);
                        self.remotes_view.render(frame, layout.content, &state);
                    }
                    crate::tui_unified::state::app_state::ViewType::Stash => {
                        self.stash_view
                            .set_focus(self.focus_manager.current_panel == FocusPanel::Content);
                        self.stash_view.render(frame, layout.content, &state);
                    }
                    crate::tui_unified::state::app_state::ViewType::QueryHistory => {
                        self.query_history_view
                            .set_focus(self.focus_manager.current_panel == FocusPanel::Content);
                        self.query_history_view
                            .render(frame, layout.content, &state);
                    }
                }

                // 渲染搜索框（如果在搜索模式）
                if self.current_mode == AppMode::Search {
                    self.search_box.set_focus(true);
                    self.search_box.set_search_active(true);
                    self.search_box.render(frame, layout.status_bar, &state);
                } else {
                    self.search_box.set_focus(false);
                    self.search_box.set_search_active(false);
                    // 渲染状态栏
                    self.render_status_bar(frame, layout.status_bar, &state);
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

    /// 渲染状态栏
    fn render_status_bar(
        &self,
        frame: &mut ratatui::Frame,
        area: ratatui::layout::Rect,
        state: &AppState,
    ) {
        use ratatui::{
            style::{Color, Style},
            text::Text,
            widgets::{Block, Borders, Paragraph},
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
            crate::tui_unified::state::app_state::ViewType::GitLog => {
                "p for pull, Enter to view diff"
            }
            crate::tui_unified::state::app_state::ViewType::Branches => {
                "Enter to switch branch, Tab to show remotes"
            }
            crate::tui_unified::state::app_state::ViewType::Tags => "Enter to view tag details",
            crate::tui_unified::state::app_state::ViewType::Remotes => {
                "Enter to view remote details"
            }
            crate::tui_unified::state::app_state::ViewType::Stash => "Enter to view stash details",
            crate::tui_unified::state::app_state::ViewType::QueryHistory => {
                "Enter to execute query"
            }
        };

        let status_content = format!(
            "[{}] Focus: {} | View: {:?} | {} | Tab-focus, c-AI commit, r-refresh, ?-help, q-quit",
            mode_text, focus_text, state.current_view, view_specific_keys
        );

        let status_bar = Paragraph::new(Text::raw(status_content))
            .block(Block::default().borders(Borders::ALL))
            .style(Style::default().fg(Color::White));

        frame.render_widget(status_bar, area);
    }

    /// 渲染加载状态 (静态方法以避免借用冲突)
    fn render_loading_state_static(frame: &mut ratatui::Frame, layout: LayoutResult) {
        use ratatui::{
            style::{Color, Style},
            text::Text,
            widgets::{Block, Borders, Paragraph},
        };

        let loading_style = Style::default().fg(Color::Yellow);

        // 侧边栏
        let sidebar = Paragraph::new(Text::raw(
            "📋 Loading Repository...\n\n⏳ Please wait while\nGit data is being loaded",
        ))
        .block(
            Block::default()
                .title("Menu")
                .borders(Borders::ALL)
                .border_style(loading_style),
        );
        frame.render_widget(sidebar, layout.sidebar);

        // 主内容区
        let content = Paragraph::new(Text::raw("🔄 Loading Git Data...\n\nThis may take a moment depending on\nthe size of your repository.\n\nInitializing:\n• Repository status\n• Commit history\n• Branch information\n• Repository metadata"))
            .block(Block::default().title("Loading").borders(Borders::ALL).border_style(loading_style));
        frame.render_widget(content, layout.content);

        // 详情面板
        let detail = Paragraph::new(Text::raw(
            "⏳ Initializing...\n\nGit data will be available\nonce loading completes.",
        ))
        .block(
            Block::default()
                .title("Details")
                .borders(Borders::ALL)
                .border_style(loading_style),
        );
        frame.render_widget(detail, layout.detail);

        // 状态栏
        let status_text = "🔄 Loading Git repository data... | [q] Quit";
        let status_bar = Paragraph::new(Text::raw(status_text))
            .block(Block::default().borders(Borders::TOP))
            .style(Style::default().bg(Color::DarkGray));
        frame.render_widget(status_bar, layout.status_bar);
    }
}
