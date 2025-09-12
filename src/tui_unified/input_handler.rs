use crossterm::event::{KeyCode, KeyEvent};

use super::app::AppMode;
use crate::tui_unified::{
    components::base::{component::Component, events::EventResult},
    focus::FocusPanel,
    Result,
};

impl super::app::TuiUnifiedApp {
    pub(crate) async fn handle_key_event(&mut self, key: KeyEvent) -> Result<()> {
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
                let _result = self.search_box.handle_key_event(key, &mut state);
            }
        }

        Ok(())
    }

    async fn handle_normal_mode_key(&mut self, key: KeyEvent) -> Result<()> {
        let current_panel = self.focus_manager.current_panel;
        let mut state = self.state.write().await;

        // 首先尝试让获得焦点的组件处理事件
        let handled = match current_panel {
            FocusPanel::Sidebar => self.sidebar_panel.handle_key_event(key, &mut state),
            FocusPanel::Content => match state.current_view {
                crate::tui_unified::state::app_state::ViewType::GitLog => {
                    self.git_log_view.handle_key_event(key, &mut state)
                }
                crate::tui_unified::state::app_state::ViewType::Branches => {
                    self.branches_view.handle_key_event(key, &mut state)
                }
                crate::tui_unified::state::app_state::ViewType::Tags => {
                    self.tags_view.handle_key_event(key, &mut state)
                }
                crate::tui_unified::state::app_state::ViewType::Remotes => {
                    self.remotes_view.handle_key_event(key, &mut state)
                }
                crate::tui_unified::state::app_state::ViewType::Stash => {
                    self.stash_view.handle_key_event(key, &mut state)
                }
                crate::tui_unified::state::app_state::ViewType::QueryHistory => {
                    self.query_history_view.handle_key_event(key, &mut state)
                }
            },
            _ => EventResult::NotHandled,
        };

        // 如果组件没有处理，则处理全局快捷键
        if matches!(handled, EventResult::NotHandled) {
            match key.code {
                KeyCode::Char('1') => {
                    // 如果当前在 Branches 视图，更新选中分支到应用状态
                    if state.get_current_view()
                        == crate::tui_unified::state::app_state::ViewType::Branches
                    {
                        self.branches_view
                            .update_selected_branch_in_state(&mut state);
                    }

                    state.set_current_view(crate::tui_unified::state::app_state::ViewType::GitLog);
                    self.focus_manager.set_focus(FocusPanel::Content);

                    // 根据选中的分支设置 Git Log 过滤
                    let selected_branch = state.selected_items.selected_branch.clone();
                    self.git_log_view.set_branch_filter(selected_branch.clone());

                    // 如果有选中分支，获取该分支的提交历史；否则显示全部提交
                    let commits_to_show = if let Some(ref branch_name) = selected_branch {
                        // 异步获取分支提交历史 - 这里简化为同步调用
                        self.get_branch_commits_sync(branch_name)
                            .unwrap_or_else(|_| {
                                // 如果获取失败，回退到显示所有提交
                                state.repo_state.commits.clone()
                            })
                    } else {
                        state.repo_state.commits.clone()
                    };

                    // 更新 Git Log 显示的提交
                    self.git_log_view.update_commits(commits_to_show);

                    // 确保GitLogView有正确的选择状态
                    if !state.repo_state.commits.is_empty() {
                        self.git_log_view.set_focus(true);
                        self.git_log_view.set_selected_index(Some(0));
                    }
                }
                KeyCode::Char('2') => {
                    state
                        .set_current_view(crate::tui_unified::state::app_state::ViewType::Branches);
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
                    state.set_current_view(
                        crate::tui_unified::state::app_state::ViewType::QueryHistory,
                    );
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
                            crate::tui_unified::state::app_state::NotificationLevel::Error,
                        );
                    } else {
                        let mut state = self.state.write().await;
                        state.add_notification(
                            "Refreshed successfully".to_string(),
                            crate::tui_unified::state::app_state::NotificationLevel::Success,
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
}
