// Git 日志视图组件
use crate::tui_unified::{
    components::{
        base::{
            component::{Component, ViewComponent, ViewType},
            events::EventResult,
        },
        widgets::list::ListWidget,
    },
    state::{git_state::Commit, AppState},
};
use crossterm::event::KeyEvent;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};

/// Git 日志视图 - 显示提交历史
pub struct GitLogView {
    list_widget: ListWidget<Commit>,
    show_details: bool,
    commits: Vec<Commit>,
    list_state: ListState,
    focused: bool,
    selected_index: Option<usize>,
    // 新增：当前过滤的分支
    current_branch_filter: Option<String>,
}

impl Default for GitLogView {
    fn default() -> Self {
        Self::new()
    }
}

impl GitLogView {
    pub fn new() -> Self {
        let format_fn = Box::new(|commit: &Commit| -> String {
            // 获取短哈希
            let short_hash = &commit.hash[..8.min(commit.hash.len())];

            // 格式化时间戳
            let timestamp = commit.date.format("%m-%d %H:%M").to_string();

            // 获取提交消息的第一行
            let message = commit.message.lines().next().unwrap_or(&commit.message);

            // 组合格式：短哈希 [时间戳] 消息 - 作者
            format!(
                "{} [{}] {} - {}",
                short_hash, timestamp, message, commit.author
            )
        });

        let style_fn = Box::new(
            |_commit: &Commit, is_selected: bool, is_focused: bool| -> Style {
                if is_selected && is_focused {
                    Style::default().fg(Color::Black).bg(Color::Yellow)
                } else if is_selected {
                    Style::default().fg(Color::White).bg(Color::DarkGray)
                } else {
                    Style::default().fg(Color::White)
                }
            },
        );

        let search_fn = Box::new(|commit: &Commit, query: &str| -> bool {
            let query = query.to_lowercase();
            commit.message.to_lowercase().contains(&query)
                || commit.author.to_lowercase().contains(&query)
                || commit.author_email.to_lowercase().contains(&query)
                || commit.hash.to_lowercase().contains(&query)
        });

        let list_widget =
            ListWidget::new("Git Log".to_string(), format_fn, style_fn).with_search_fn(search_fn);

        let mut list_state = ListState::default();
        list_state.select(Some(0));

        Self {
            list_widget,
            show_details: false,
            commits: Vec::new(),
            list_state,
            focused: false,
            selected_index: None,
            current_branch_filter: None,
        }
    }

    pub fn selected_commit(&self) -> Option<&Commit> {
        self.selected_index.and_then(|idx| self.commits.get(idx))
    }

    pub fn set_selected_index(&mut self, index: Option<usize>) {
        self.list_widget.set_selected_index(index);
    }

    /// 设置分支过滤
    pub fn set_branch_filter(&mut self, branch_name: Option<String>) {
        self.current_branch_filter = branch_name;
        self.update_title();
    }

    /// 更新标题以反映当前分支过滤状态
    fn update_title(&mut self) {
        let title = if let Some(ref branch_name) = self.current_branch_filter {
            format!("Git Log - {}", branch_name)
        } else {
            "Git Log".to_string()
        };

        // 重新创建 ListWidget 来更新标题
        let format_fn = Box::new(|commit: &Commit| -> String {
            let short_hash = &commit.hash[..8.min(commit.hash.len())];
            let timestamp = commit.date.format("%m-%d %H:%M").to_string();
            let message = commit.message.lines().next().unwrap_or(&commit.message);
            format!(
                "{} [{}] {} - {}",
                short_hash, timestamp, message, commit.author
            )
        });

        let style_fn = Box::new(
            |_commit: &Commit, is_selected: bool, is_focused: bool| -> Style {
                if is_selected && is_focused {
                    Style::default().fg(Color::Black).bg(Color::Yellow)
                } else if is_selected {
                    Style::default().fg(Color::White).bg(Color::DarkGray)
                } else {
                    Style::default().fg(Color::White)
                }
            },
        );

        let search_fn = Box::new(|commit: &Commit, query: &str| -> bool {
            let query = query.to_lowercase();
            commit.message.to_lowercase().contains(&query)
                || commit.author.to_lowercase().contains(&query)
                || commit.author_email.to_lowercase().contains(&query)
                || commit.hash.to_lowercase().contains(&query)
        });

        let current_items = self.commits.clone();
        self.list_widget = ListWidget::new(title, format_fn, style_fn)
            .with_search_fn(search_fn)
            .with_items(current_items);
    }

    /// 更新commit列表数据
    pub fn update_commits(&mut self, commits: Vec<Commit>) {
        let has_commits = !commits.is_empty();
        self.commits = commits;
        self.list_widget.set_items(self.commits.clone());

        // 确保第一个项目被选中
        if has_commits {
            self.list_widget.set_focus(true);
            self.list_widget.set_selected_index(Some(0));
            self.selected_index = Some(0);
            self.list_state.select(Some(0));
        } else {
            self.selected_index = None;
            self.list_state.select(None);
        }
    }

    pub fn toggle_details(&mut self) {
        self.show_details = !self.show_details;
        self.update_title();
    }

    pub fn refresh_commits(&mut self, state: &AppState) {
        let commits = state.repo_state.commits.clone();
        let has_commits = !commits.is_empty();
        self.list_widget.set_items(commits);
        // 确保第一个项目被选中
        if has_commits {
            self.list_widget.set_focus(true);
            // 明确设置选择第一个索引
            self.list_widget.set_selected_index(Some(0));
        }
    }

    /// 创建彩色的提交项显示（静态版本）
    fn create_colored_commit_item_static(commit: &Commit, is_selected: bool) -> ListItem<'_> {
        // 获取短哈希
        let short_hash = &commit.hash[..8.min(commit.hash.len())];

        // 格式化时间戳
        let timestamp = commit.date.format("%m-%d %H:%M").to_string();

        // 获取提交消息的第一行
        let message = commit.message.lines().next().unwrap_or(&commit.message);

        // 根据选中状态确定颜色
        let hash_color = if is_selected {
            Color::Yellow
        } else {
            Color::DarkGray
        };
        let time_color = if is_selected {
            Color::Cyan
        } else {
            Color::Blue
        };
        let message_color = if is_selected {
            Color::White
        } else {
            Color::White
        };
        let author_color = if is_selected {
            Color::LightGreen
        } else {
            Color::Gray
        };

        // 使用多个 Span 创建彩色显示
        let content = Line::from(vec![
            Span::styled(
                format!("{} ", short_hash),
                Style::default().fg(hash_color).add_modifier(Modifier::BOLD),
            ),
            Span::styled(format!("[{}] ", timestamp), Style::default().fg(time_color)),
            Span::styled(message.to_string(), Style::default().fg(message_color)),
            Span::styled(
                format!(" - {}", commit.author),
                Style::default().fg(author_color),
            ),
        ]);

        ListItem::new(content)
    }
}

impl Component for GitLogView {
    fn name(&self) -> &str {
        "GitLogView"
    }

    fn render(&mut self, frame: &mut Frame, area: Rect, state: &AppState) {
        // 确保提交列表是最新的
        if self.list_widget.len() != state.repo_state.commits.len() {
            self.refresh_commits(state);
        }

        // 获取需要的值避免借用冲突
        let commits = &self.commits;
        let selected_index = self.selected_index;
        let focused = self.focused;

        // 创建彩色的列表项
        let list_items: Vec<ListItem> = commits
            .iter()
            .enumerate()
            .map(|(i, commit)| {
                let is_selected = Some(i) == selected_index;
                Self::create_colored_commit_item_static(commit, is_selected)
            })
            .collect();

        // 边框样式
        let border_style = if focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::White)
        };

        // 标题
        let title = format!("📊 Git Log ({} commits)", commits.len());

        // 创建列表
        let list = List::new(list_items)
            .block(
                Block::default()
                    .title(title)
                    .borders(Borders::ALL)
                    .border_style(border_style),
            )
            .highlight_style(if focused {
                Style::default().fg(Color::Black).bg(Color::Yellow)
            } else {
                Style::default().fg(Color::White).bg(Color::DarkGray)
            });

        frame.render_stateful_widget(list, area, &mut self.list_state);
    }

    fn handle_key_event(&mut self, key: KeyEvent, state: &mut AppState) -> EventResult {
        use crossterm::event::KeyCode;

        // 处理视图特定的按键
        match key.code {
            KeyCode::Tab => {
                self.toggle_details();
                EventResult::Handled
            }
            KeyCode::Enter => {
                // 请求显示选中提交的diff
                if let Some(selected_commit) = self.selected_commit() {
                    state.request_diff(selected_commit.hash.clone());
                    EventResult::Handled
                } else {
                    EventResult::NotHandled
                }
            }
            KeyCode::Char('r') => {
                // 刷新提交历史
                self.refresh_commits(state);
                EventResult::Handled
            }
            KeyCode::Char('p') => {
                // 拉取最新代码
                state.request_git_pull();
                EventResult::Handled
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if let Some(current) = self.selected_index {
                    if current > 0 {
                        self.selected_index = Some(current - 1);
                        self.list_state.select(Some(current - 1));
                    } else {
                        let last = self.commits.len().saturating_sub(1);
                        self.selected_index = Some(last);
                        self.list_state.select(Some(last));
                    }
                }
                EventResult::Handled
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if let Some(current) = self.selected_index {
                    if current < self.commits.len().saturating_sub(1) {
                        self.selected_index = Some(current + 1);
                        self.list_state.select(Some(current + 1));
                    } else {
                        self.selected_index = Some(0);
                        self.list_state.select(Some(0));
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
        self.list_widget.set_focus(focused);
    }

    fn can_focus(&self) -> bool {
        self.list_widget.can_focus()
    }

    fn min_size(&self) -> (u16, u16) {
        (60, 20)
    }
}

impl ViewComponent for GitLogView {
    fn view_type(&self) -> ViewType {
        ViewType::GitLog
    }

    fn title(&self) -> String {
        if self.show_details {
            "Git Log (Details)".to_string()
        } else {
            "Git Log".to_string()
        }
    }

    fn supports_search(&self) -> bool {
        true
    }

    fn search(&mut self, query: &str) -> EventResult {
        // TODO: 实现按提交消息或作者搜索
        self.list_widget.search(query)
    }

    fn clear_search(&mut self) -> EventResult {
        self.list_widget.clear_search()
    }

    fn selected_index(&self) -> Option<usize> {
        self.list_widget.selected_index()
    }

    fn set_selected_index(&mut self, index: Option<usize>) {
        if let Some(idx) = index {
            if idx < self.list_widget.len() {
                self.list_widget
                    .set_items(self.list_widget.items().to_vec());
            }
        }
    }
}

pub struct BranchesView;
pub struct TagsView;
pub struct RemotesView;
pub struct StashView;
pub struct QueryHistoryView;
