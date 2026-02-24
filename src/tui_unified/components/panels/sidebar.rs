// 侧边栏面板组件实现
use crate::tui_unified::{
    components::base::{
        component::{Component, PanelComponent, PanelType},
        events::EventResult,
    },
    state::AppState,
};
use crossterm::event::KeyEvent;
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::Text,
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

/// 侧边栏面板 - 显示导航菜单、仓库状态和分支列表
pub struct SidebarPanel {
    focused: bool,
    selected_index: usize,
    menu_items: Vec<MenuItem>,
    // 分支列表相关字段
    branches_focused: bool,
    selected_branch_index: usize,
    show_branches: bool,
}

struct MenuItem {
    label: String,
    key: char,
    #[allow(dead_code)]
    description: String,
}

impl Default for SidebarPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl SidebarPanel {
    pub fn new() -> Self {
        let menu_items = vec![
            MenuItem {
                label: "📊 Git Log".to_string(),
                key: '1',
                description: "View commit history with branches".to_string(),
            },
            MenuItem {
                label: "🏷️ Tags".to_string(),
                key: '2',
                description: "View tags".to_string(),
            },
            MenuItem {
                label: "📡 Remotes".to_string(),
                key: '3',
                description: "Manage remotes".to_string(),
            },
            MenuItem {
                label: "💾 Stash".to_string(),
                key: '4',
                description: "Manage stash".to_string(),
            },
            MenuItem {
                label: "📜 History".to_string(),
                key: '5',
                description: "Query history".to_string(),
            },
        ];

        Self {
            focused: false,
            selected_index: 0,
            menu_items,
            branches_focused: false,
            selected_branch_index: 0,
            show_branches: true, // 默认显示分支列表
        }
    }

    /// 根据当前视图同步选择状态，确保三角形标记与视图状态一致
    fn sync_selection_with_current_view(&mut self, state: &AppState) {
        let new_index = match state.current_view {
            crate::tui_unified::state::app_state::ViewType::GitLog => 0,
            crate::tui_unified::state::app_state::ViewType::Branches => 0, // Branches 现在集成到 GitLog 中
            crate::tui_unified::state::app_state::ViewType::Tags => 1,
            crate::tui_unified::state::app_state::ViewType::Remotes => 2,
            crate::tui_unified::state::app_state::ViewType::Stash => 3,
            crate::tui_unified::state::app_state::ViewType::QueryHistory => 4,
            crate::tui_unified::state::app_state::ViewType::Staging => 5,
        };

        if new_index < self.menu_items.len() {
            self.selected_index = new_index;
        }
    }

    /// 渲染分支列表
    fn render_branches_list(&mut self, frame: &mut Frame, area: Rect, state: &AppState) {
        use ratatui::{
            style::{Color, Modifier, Style},
            text::Text,
            widgets::{List, ListItem, ListState},
        };

        // 获取分支列表
        let branches = &state.repo_state.branches;
        let current_branch = &state.repo_state.current_branch;

        // 创建分支列表项
        let branch_items: Vec<ListItem> = branches
            .iter()
            .enumerate()
            .map(|(i, branch)| {
                let is_current = branch.name == *current_branch;
                let is_selected = i == self.selected_branch_index && self.branches_focused;

                // 格式化分支显示
                let prefix = if is_current { "★ " } else { "  " };
                let content = format!("{}{}", prefix, branch.name);

                let style = if is_selected && self.branches_focused {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else if is_current {
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };

                ListItem::new(Text::raw(content)).style(style)
            })
            .collect();

        // 边框样式
        let border_style = if self.branches_focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::White)
        };

        // 创建标题
        let title = format!("🌿 Branches ({})", branches.len());

        // 渲染分支列表
        let mut list_state = ListState::default();
        if self.branches_focused && !branches.is_empty() {
            list_state.select(Some(self.selected_branch_index));
        }

        frame.render_stateful_widget(
            List::new(branch_items).block(
                Block::default()
                    .title(title)
                    .borders(Borders::ALL)
                    .border_style(border_style),
            ),
            area,
            &mut list_state,
        );
    }
}

impl Component for SidebarPanel {
    fn name(&self) -> &str {
        "SidebarPanel"
    }

    fn render(&mut self, frame: &mut Frame, area: Rect, state: &AppState) {
        // 根据当前视图同步选择状态，确保三角形标记正确
        self.sync_selection_with_current_view(state);

        let style = if self.focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::White)
        };

        // 根据当前视图创建不同的内容
        let (status_content, should_show_menu) = match state.current_view {
            crate::tui_unified::state::app_state::ViewType::GitLog => {
                // 在 Git Log 视图中，显示选中的分支信息和分支列表
                let repo_summary = state.repo_state.get_repo_summary();
                let selected_branch_info = if let Some(ref branch_name) =
                    state.selected_items.selected_branch
                {
                    format!(
                        "📋 Repository: {}\n\n🔍 Viewing Branch: {}\n📝 Showing commits for: {}\n\n",
                        repo_summary.name,
                        branch_name,
                        branch_name
                    )
                } else {
                    format!(
                        "📋 Repository: {}\n\n📝 All Commits: {}\n🌲 Total Branches: {}\n\n",
                        repo_summary.name, repo_summary.total_commits, repo_summary.total_branches
                    )
                };
                (selected_branch_info, false) // 不显示导航菜单，而显示分支列表
            }
            _ => {
                // 其他视图显示标准的仓库状态信息
                let repo_summary = state.repo_state.get_repo_summary();
                let status_content = format!(
                    "📋 Repository: {}\n\n🔀 Branch: {}\n📝 Commits: {}\n🌲 Branches: {}\n🏷️ Tags: {}\n📡 Remotes: {}\n💾 Stashes: {}\n",
                    repo_summary.name,
                    if repo_summary.current_branch.is_empty() { "None" } else { &repo_summary.current_branch },
                    repo_summary.total_commits,
                    repo_summary.total_branches,
                    repo_summary.total_tags,
                    repo_summary.total_remotes,
                    repo_summary.total_stashes,
                );
                (status_content, true) // 显示导航菜单
            }
        };

        // 根据视图类型创建不同的列表项
        let (list_items, list_title) = if should_show_menu {
            // 显示导航菜单
            let menu_items: Vec<ListItem> = self
                .menu_items
                .iter()
                .enumerate()
                .map(|(i, item)| {
                    let selected = if i == self.selected_index {
                        "► "
                    } else {
                        "  "
                    };
                    let content = format!("{}[{}] {}", selected, item.key, item.label);
                    ListItem::new(Text::raw(content)).style(
                        if i == self.selected_index && self.focused {
                            Style::default().fg(Color::Yellow)
                        } else {
                            Style::default()
                        },
                    )
                })
                .collect();
            (menu_items, "📋 Navigation:")
        } else {
            // 在 Git Log 视图中显示分支列表
            let branch_items: Vec<ListItem> = state
                .repo_state
                .branches
                .iter()
                .map(|branch| {
                    let indicator = if branch.is_current { "* " } else { "  " };
                    let selected =
                        if Some(&branch.name) == state.selected_items.selected_branch.as_ref() {
                            "► "
                        } else {
                            "  "
                        };
                    let content = format!("{}{}{}", selected, indicator, branch.name);
                    ListItem::new(Text::raw(content)).style(
                        if Some(&branch.name) == state.selected_items.selected_branch.as_ref()
                            && self.focused
                        {
                            Style::default().fg(Color::Yellow)
                        } else if branch.is_current {
                            Style::default().fg(Color::Green)
                        } else {
                            Style::default()
                        },
                    )
                })
                .collect();
            (branch_items, "🌲 Branches:")
        };

        // 组合完整内容
        let full_content = format!("{}\n{}:\n", status_content, list_title);
        let status_paragraph = Paragraph::new(Text::raw(full_content));

        // 计算布局：三部分显示 - 状态、分支列表、菜单
        let status_height = if area.height > 30 { 8 } else { 6 };
        let branches_height = if self.show_branches && area.height > 20 {
            (area.height - status_height).saturating_sub(8)
        } else {
            0
        };
        let menu_height = area.height.saturating_sub(status_height + branches_height);

        let status_area = Rect {
            x: area.x,
            y: area.y,
            width: area.width,
            height: status_height,
        };
        let branches_area = Rect {
            x: area.x,
            y: area.y + status_height,
            width: area.width,
            height: branches_height,
        };
        let menu_area = Rect {
            x: area.x,
            y: area.y + status_height + branches_height,
            width: area.width,
            height: menu_height,
        };

        // 渲染状态信息
        frame.render_widget(
            status_paragraph.block(
                Block::default()
                    .title("Repository")
                    .borders(Borders::ALL)
                    .border_style(style),
            ),
            status_area,
        );

        // 渲染分支列表
        if self.show_branches && branches_height > 2 {
            self.render_branches_list(frame, branches_area, state);
        }

        // 渲染列表（菜单或分支列表）
        frame.render_widget(
            List::new(list_items).block(
                Block::default()
                    .title(list_title)
                    .borders(Borders::ALL)
                    .border_style(style),
            ),
            menu_area,
        );
    }

    fn handle_key_event(&mut self, key: KeyEvent, state: &mut AppState) -> EventResult {
        use crossterm::event::KeyCode;

        match key.code {
            KeyCode::Tab => {
                // Tab键在菜单和分支列表之间切换焦点
                if self.show_branches && !state.repo_state.branches.is_empty() {
                    self.branches_focused = !self.branches_focused;
                }
                EventResult::Handled
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.branches_focused {
                    // 分支列表导航
                    if !state.repo_state.branches.is_empty() {
                        if self.selected_branch_index > 0 {
                            self.selected_branch_index -= 1;
                        } else {
                            self.selected_branch_index = state.repo_state.branches.len() - 1;
                        }
                    }
                } else {
                    // 菜单导航
                    if self.selected_index > 0 {
                        self.selected_index -= 1;
                    } else {
                        self.selected_index = self.menu_items.len() - 1;
                    }
                }
                EventResult::Handled
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.branches_focused {
                    // 分支列表导航
                    if !state.repo_state.branches.is_empty() {
                        if self.selected_branch_index < state.repo_state.branches.len() - 1 {
                            self.selected_branch_index += 1;
                        } else {
                            self.selected_branch_index = 0;
                        }
                    }
                } else {
                    // 菜单导航
                    if self.selected_index < self.menu_items.len() - 1 {
                        self.selected_index += 1;
                    } else {
                        self.selected_index = 0;
                    }
                }
                EventResult::Handled
            }
            KeyCode::Enter => {
                if self.branches_focused {
                    // 切换到选中的分支
                    if let Some(branch) = state.repo_state.branches.get(self.selected_branch_index)
                    {
                        // 由于switch_to_branch是async方法，我们需要在这里使用state的通知系统
                        // 创建一个分支切换请求
                        state.request_branch_switch(branch.name.clone());
                    }
                } else {
                    // 根据选中的菜单项切换视图
                    match self.selected_index {
                        0 => state.set_current_view(
                            crate::tui_unified::state::app_state::ViewType::GitLog,
                        ),
                        1 => state
                            .set_current_view(crate::tui_unified::state::app_state::ViewType::Tags),
                        2 => state.set_current_view(
                            crate::tui_unified::state::app_state::ViewType::Remotes,
                        ),
                        3 => state.set_current_view(
                            crate::tui_unified::state::app_state::ViewType::Stash,
                        ),
                        4 => state.set_current_view(
                            crate::tui_unified::state::app_state::ViewType::QueryHistory,
                        ),
                        _ => {}
                    }
                }
                EventResult::Handled
            }
            KeyCode::Char('b') => {
                // 'b'键切换分支列表显示/隐藏
                self.show_branches = !self.show_branches;
                if !self.show_branches {
                    self.branches_focused = false;
                }
                EventResult::Handled
            }
            KeyCode::Char(c) if ('1'..='5').contains(&c) => {
                // 数字键快速切换视图（只在菜单模式下工作）
                if !self.branches_focused {
                    let index = (c as u8 - b'1') as usize;
                    if index < self.menu_items.len() {
                        self.selected_index = index;
                        // 直接切换视图
                        match index {
                            0 => state.set_current_view(
                                crate::tui_unified::state::app_state::ViewType::GitLog,
                            ),
                            1 => state.set_current_view(
                                crate::tui_unified::state::app_state::ViewType::Tags,
                            ),
                            2 => state.set_current_view(
                                crate::tui_unified::state::app_state::ViewType::Remotes,
                            ),
                            3 => state.set_current_view(
                                crate::tui_unified::state::app_state::ViewType::Stash,
                            ),
                            4 => state.set_current_view(
                                crate::tui_unified::state::app_state::ViewType::QueryHistory,
                            ),
                            _ => {}
                        }
                    }
                }
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
        (25, 15)
    }
}

impl PanelComponent for SidebarPanel {
    fn panel_type(&self) -> PanelType {
        PanelType::Sidebar
    }

    fn supports_scroll(&self) -> bool {
        true
    }

    fn scroll_position(&self) -> usize {
        self.selected_index
    }

    fn set_scroll_position(&mut self, position: usize) {
        if position < self.menu_items.len() {
            self.selected_index = position;
        }
    }
}

// 占位符：其他面板组件
pub struct ContentPanel;
pub struct DetailPanel;
