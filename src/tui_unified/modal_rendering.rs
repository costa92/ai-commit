use crossterm::event::KeyEvent;
use std::collections::HashMap;

use crate::tui_unified::components::base::component::Component;
use crate::tui_unified::Result;

impl super::app::TuiUnifiedApp {
    /// 渲染模态框
    pub(crate) fn render_modal(
        &mut self,
        frame: &mut ratatui::Frame,
        modal: &crate::tui_unified::state::app_state::ModalState,
        area: ratatui::layout::Rect,
    ) {
        use ratatui::{
            layout::{Alignment, Constraint, Direction, Layout},
            style::{Color, Style},
            text::Text,
            widgets::Paragraph,
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

                // 更新视口高度（popup_area 减去 info(3) + status(4) + borders(4)）
                if let Some(viewer) = &mut self.diff_viewer {
                    viewer.viewport_height = popup_area.height.saturating_sub(11);
                }

                // 预填充渲染缓存（避免每帧重新解析 diff）
                self.ensure_diff_cache();

                // 使用自定义的DiffViewer渲染，限制在popup区域内
                if let Some(viewer) = &self.diff_viewer {
                    self.render_diff_viewer_in_area(frame, viewer, popup_area);
                } else {
                    // 如果diff_viewer没有初始化，显示loading
                    let loading_paragraph = ratatui::widgets::Paragraph::new("Loading diff...")
                        .block(
                            ratatui::widgets::Block::default()
                                .borders(ratatui::widgets::Borders::ALL)
                                .title("Diff Viewer"),
                        );
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
                    .style(Style::default().fg(Color::Gray).bg(Color::Black))
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
                            self.commit_editor.render(frame, popup_area, &state);
                        }
                        Err(_) => {
                            // 如果无法获取状态，使用一个静态的虚拟状态
                            static DUMMY_STATE: std::sync::LazyLock<
                                crate::tui_unified::state::AppState,
                            > = std::sync::LazyLock::new(|| crate::tui_unified::state::AppState {
                                layout: Default::default(),
                                focus: Default::default(),
                                current_view:
                                    crate::tui_unified::state::app_state::ViewType::GitLog,
                                modal: None,
                                repo_state: Default::default(),
                                selected_items: Default::default(),
                                search_state: Default::default(),
                                config: crate::tui_unified::config::AppConfig::default(),
                                loading_tasks: HashMap::new(),
                                notifications: Vec::new(),
                                new_layout: Default::default(),
                            });
                            self.commit_editor.render(frame, popup_area, &DUMMY_STATE);
                        }
                    }
                } else {
                    // 非编辑模式：显示生成的消息
                    let ai_commit_content = if let Some(ref message) = self.ai_commit_message {
                        format!(
                            "Status: {}\n\n📝 Generated Commit Message:\n\n{}",
                            self.ai_commit_status
                                .as_ref()
                                .unwrap_or(&"Ready".to_string()),
                            message.trim()
                        )
                    } else {
                        format!(
                            "🤖 {}",
                            self.ai_commit_status
                                .as_ref()
                                .unwrap_or(&"Generating commit message...".to_string())
                        )
                    };

                    let ai_commit_block = Paragraph::new(Text::from(ai_commit_content))
                        .block(
                            Block::default()
                                .borders(Borders::ALL)
                                .title("AI Commit")
                                .border_style(Style::default().fg(Color::Green)),
                        )
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
            crate::tui_unified::state::app_state::ModalType::AIReview
            | crate::tui_unified::state::app_state::ModalType::AIRefactor => {
                // AI Review / Refactor 结果模态框（大面积，可滚动）
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
                            Constraint::Length(4),
                            Constraint::Min(60),
                            Constraint::Length(4),
                        ])
                        .split(vertical[1])[1]
                };

                // 使用专门的背景清除方法
                self.clear_modal_background(frame, area);

                use ratatui::widgets::{Block, Borders, Wrap};

                let (title, border_color) = match modal.modal_type {
                    crate::tui_unified::state::app_state::ModalType::AIReview => {
                        ("AI Code Review", Color::Cyan)
                    }
                    crate::tui_unified::state::app_state::ModalType::AIRefactor => {
                        ("AI Refactor Suggestions", Color::Magenta)
                    }
                    _ => unreachable!(),
                };

                let content_block = Paragraph::new(Text::from(modal.content.clone()))
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title(title)
                            .border_style(Style::default().fg(border_color)),
                    )
                    .style(Style::default().fg(Color::White))
                    .wrap(Wrap { trim: false });

                frame.render_widget(content_block, popup_area);

                // 帮助文本
                let help_area = ratatui::layout::Rect {
                    x: popup_area.x,
                    y: popup_area.y + popup_area.height,
                    width: popup_area.width,
                    height: 1,
                };

                let help_text = "[Esc] or [q] Close";
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
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title(modal.title.clone())
                            .border_style(Style::default().fg(Color::Yellow)),
                    )
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
    pub(crate) async fn handle_modal_key(&mut self, key: KeyEvent) -> Result<()> {
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
                            self.diff_viewer = None;
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
                                viewer.clamp_scroll();
                            }
                            KeyCode::Char('K') => {
                                viewer.diff_scroll = viewer.diff_scroll.saturating_sub(1);
                            }
                            KeyCode::PageDown => {
                                viewer.diff_scroll = viewer.diff_scroll.saturating_add(10);
                                viewer.clamp_scroll();
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
                crate::tui_unified::state::app_state::ModalType::AIReview
                | crate::tui_unified::state::app_state::ModalType::AIRefactor => {
                    // AI Review/Refactor 模态框：只处理关闭键
                    match key.code {
                        KeyCode::Esc | KeyCode::Char('q') => {
                            drop(state);
                            let mut state = self.state.write().await;
                            state.hide_modal();
                            return Ok(());
                        }
                        _ => {}
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
                            if modal.modal_type
                                == crate::tui_unified::state::app_state::ModalType::GitPull
                            {
                                drop(state); // 显式释放读锁
                                return self.confirm_git_pull().await;
                            }
                            // 在分支切换模式下，Enter确认切换
                            else if modal.modal_type
                                == crate::tui_unified::state::app_state::ModalType::BranchSwitch
                            {
                                drop(state); // 显式释放读锁
                                return self.confirm_branch_switch().await;
                            }
                            // 在AI commit推送提示模式下，Enter等于确认推送
                            else if self.ai_commit_mode && self.ai_commit_push_prompt {
                                drop(state); // 显式释放读锁
                                return self.confirm_push().await;
                            }
                            // 在AI commit模式下按Enter确认提交
                            else if self.ai_commit_mode
                                && !self.ai_commit_editing
                                && self.ai_commit_message.is_some()
                            {
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
                                self.ai_commit_message = Some(edited_content);
                                self.ai_commit_status = Some("Message edited".to_string());

                                // 不需要重新显示模态框，因为渲染逻辑会自动切换到非编辑模式显示
                                // 现在用户可以按 Enter 提交或 Esc 取消
                            }
                        }
                        _ => {
                            // 在AI commit编辑模式下，将键盘事件转发给编辑器
                            if self.ai_commit_mode && self.ai_commit_editing {
                                let mut dummy_state = crate::tui_unified::state::AppState::new(
                                    &crate::tui_unified::config::AppConfig::default(),
                                )
                                .await
                                .unwrap_or_else(|_| {
                                    // 如果创建失败，创建一个基本的虚拟状态
                                    crate::tui_unified::state::AppState {
                                        layout: Default::default(),
                                        focus: Default::default(),
                                        current_view:
                                            crate::tui_unified::state::app_state::ViewType::GitLog,
                                        modal: None,
                                        repo_state: Default::default(),
                                        selected_items: Default::default(),
                                        search_state: Default::default(),
                                        config: crate::tui_unified::config::AppConfig::default(),
                                        loading_tasks: HashMap::new(),
                                        notifications: Vec::new(),
                                        new_layout: Default::default(),
                                    }
                                });
                                let _result =
                                    self.commit_editor.handle_key_event(key, &mut dummy_state);
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
