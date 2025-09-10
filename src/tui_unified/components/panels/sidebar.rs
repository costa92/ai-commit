// 侧边栏面板组件实现
use crossterm::event::KeyEvent;
use ratatui::{Frame, layout::Rect, widgets::{Block, Borders, Paragraph, List, ListItem}, text::Text, style::{Color, Style}};
use crate::tui_unified::{
    state::AppState,
    components::base::{
        component::{Component, PanelComponent, PanelType},
        events::EventResult
    }
};

/// 侧边栏面板 - 显示导航菜单和仓库状态
pub struct SidebarPanel {
    focused: bool,
    selected_index: usize,
    menu_items: Vec<MenuItem>,
}

struct MenuItem {
    label: String,
    key: char,
    description: String,
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
        };
        
        if new_index < self.menu_items.len() {
            self.selected_index = new_index;
        }
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

        // 创建仓库状态信息
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

        // 创建菜单项列表
        let menu_items: Vec<ListItem> = self.menu_items
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let selected = if i == self.selected_index { "► " } else { "  " };
                let content = format!("{}[{}] {}", selected, item.key, item.label);
                ListItem::new(Text::raw(content))
                    .style(if i == self.selected_index && self.focused {
                        Style::default().fg(Color::Yellow)
                    } else {
                        Style::default()
                    })
            })
            .collect();

        // 组合完整内容
        let full_content = format!("{}\n📋 Navigation:\n", status_content);
        let status_paragraph = Paragraph::new(Text::raw(full_content));

        // 计算布局：上半部分显示状态，下半部分显示菜单
        let status_height = area.height.saturating_sub(10);
        let status_area = Rect {
            x: area.x,
            y: area.y,
            width: area.width,
            height: status_height,
        };
        let menu_area = Rect {
            x: area.x,
            y: area.y + status_height,
            width: area.width,
            height: area.height - status_height,
        };

        // 渲染状态信息
        frame.render_widget(
            status_paragraph.block(Block::default().title("Repository").borders(Borders::ALL).border_style(style)),
            status_area
        );

        // 渲染菜单列表
        frame.render_widget(
            List::new(menu_items).block(Block::default().title("Menu").borders(Borders::ALL).border_style(style)),
            menu_area
        );
    }

    fn handle_key_event(&mut self, key: KeyEvent, state: &mut AppState) -> EventResult {
        use crossterm::event::KeyCode;
        
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                } else {
                    self.selected_index = self.menu_items.len() - 1;
                }
                EventResult::Handled
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.selected_index < self.menu_items.len() - 1 {
                    self.selected_index += 1;
                } else {
                    self.selected_index = 0;
                }
                EventResult::Handled
            }
            KeyCode::Enter => {
                // 根据选中的菜单项切换视图
                match self.selected_index {
                    0 => state.set_current_view(crate::tui_unified::state::app_state::ViewType::GitLog),
                    1 => state.set_current_view(crate::tui_unified::state::app_state::ViewType::Tags),
                    2 => state.set_current_view(crate::tui_unified::state::app_state::ViewType::Remotes),
                    3 => state.set_current_view(crate::tui_unified::state::app_state::ViewType::Stash),
                    4 => state.set_current_view(crate::tui_unified::state::app_state::ViewType::QueryHistory),
                    _ => {}
                }
                EventResult::Handled
            }
            KeyCode::Char(c) if c >= '1' && c <= '5' => {
                let index = (c as u8 - b'1') as usize;
                if index < self.menu_items.len() {
                    self.selected_index = index;
                    // 直接切换视图
                    match index {
                        0 => state.set_current_view(crate::tui_unified::state::app_state::ViewType::GitLog),
                        1 => state.set_current_view(crate::tui_unified::state::app_state::ViewType::Tags),
                        2 => state.set_current_view(crate::tui_unified::state::app_state::ViewType::Remotes),
                        3 => state.set_current_view(crate::tui_unified::state::app_state::ViewType::Stash),
                        4 => state.set_current_view(crate::tui_unified::state::app_state::ViewType::QueryHistory),
                        _ => {}
                    }
                }
                EventResult::Handled
            }
            _ => EventResult::NotHandled
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