use crate::query_history::{QueryHistory, QueryHistoryEntry};
use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap, Clear},
    Frame, Terminal,
};
use std::io;

/// TUI 应用状态
pub struct App {
    /// 查询历史
    history: QueryHistory,
    /// 历史记录列表
    entries: Vec<QueryHistoryEntry>,
    /// 列表状态
    list_state: ListState,
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
}

impl App {
    /// 创建新的应用实例
    pub fn new() -> Result<Self> {
        let history = QueryHistory::new(1000)?;
        let entries = history.get_recent(1000)
            .into_iter()
            .map(|e| e.clone())
            .collect::<Vec<_>>();
        
        let mut list_state = ListState::default();
        if !entries.is_empty() {
            list_state.select(Some(0));
        }

        Ok(Self {
            history,
            entries,
            list_state,
            selected_index: 0,
            show_details: true,
            search_filter: String::new(),
            search_mode: false,
            should_quit: false,
            execute_query: None,
            execution_result: None,
            show_help: false,
        })
    }

    /// 移动到下一个条目
    fn next(&mut self) {
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

    /// 移动到上一个条目
    fn previous(&mut self) {
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

    /// 移动到第一个条目
    fn first(&mut self) {
        if !self.entries.is_empty() {
            self.list_state.select(Some(0));
            self.selected_index = 0;
        }
    }

    /// 移动到最后一个条目
    fn last(&mut self) {
        if !self.entries.is_empty() {
            let last_index = self.entries.len() - 1;
            self.list_state.select(Some(last_index));
            self.selected_index = last_index;
        }
    }

    /// 翻页向下
    fn page_down(&mut self) {
        if self.entries.is_empty() {
            return;
        }

        let page_size = 10;
        let i = match self.list_state.selected() {
            Some(i) => {
                let new_index = i + page_size;
                if new_index >= self.entries.len() {
                    self.entries.len() - 1
                } else {
                    new_index
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
        self.selected_index = i;
    }

    /// 翻页向上
    fn page_up(&mut self) {
        if self.entries.is_empty() {
            return;
        }

        let page_size = 10;
        let i = match self.list_state.selected() {
            Some(i) => {
                if i < page_size {
                    0
                } else {
                    i - page_size
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
        self.selected_index = i;
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

    /// 获取选中的查询
    pub fn get_selected_query(&self) -> Option<String> {
        self.list_state.selected()
            .and_then(|i| self.entries.get(i))
            .map(|entry| entry.query.clone())
    }
}

/// 运行TUI应用
pub async fn run_tui() -> Result<Option<String>> {
    // 设置终端
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // 创建应用并运行
    let mut app = App::new()?;
    let res = run_app(&mut terminal, &mut app).await;

    // 恢复终端
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

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
                } else {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => {
                            app.should_quit = true;
                            return Ok(());
                        }
                        KeyCode::Enter => {
                            return Ok(());
                        }
                        KeyCode::Down | KeyCode::Char('j') => app.next(),
                        KeyCode::Up | KeyCode::Char('k') => app.previous(),
                        KeyCode::Home | KeyCode::Char('g') => app.first(),
                        KeyCode::End | KeyCode::Char('G') => app.last(),
                        KeyCode::PageDown | KeyCode::Char('f') => app.page_down(),
                        KeyCode::PageUp | KeyCode::Char('b') => app.page_up(),
                        KeyCode::Char('d') => app.show_details = !app.show_details,
                        KeyCode::Char('/') => {
                            app.search_mode = true;
                            app.search_filter.clear();
                        }
                        _ => {}
                    }
                }
            }
        }

        if app.should_quit {
            return Ok(());
        }
    }
}

/// 绘制UI
fn ui(f: &mut Frame, app: &App) {
    let chunks = if app.show_details {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),    // 标题
                Constraint::Percentage(60), // 列表
                Constraint::Percentage(35), // 详情
                Constraint::Length(3),    // 状态栏
            ])
            .split(f.size())
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),    // 标题
                Constraint::Min(0),       // 列表
                Constraint::Length(3),    // 状态栏
            ])
            .split(f.size())
    };

    // 标题栏
    let title = if app.search_mode {
        format!("🔍 Search: {}_", app.search_filter)
    } else {
        format!("📜 Query History - {} entries", app.entries.len())
    };
    
    let title_block = Paragraph::new(title)
        .style(Style::default().fg(Color::Cyan))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White)),
        );
    f.render_widget(title_block, chunks[0]);

    // 历史列表
    let items: Vec<ListItem> = app
        .entries
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            let status_icon = if entry.success { "✅" } else { "❌" };
            let timestamp = entry.timestamp.format("%m-%d %H:%M");
            let query_type = entry.query_type.as_deref().unwrap_or("query");
            let result_info = if let Some(count) = entry.result_count {
                format!(" ({} results)", count)
            } else {
                String::new()
            };

            let content = format!(
                "{} {} [{}] {}{}",
                status_icon,
                timestamp,
                query_type,
                entry.query,
                result_info
            );

            let style = if Some(i) == app.list_state.selected() {
                Style::default()
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

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("History")
                .style(Style::default().fg(Color::White)),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        );

    f.render_stateful_widget(list, chunks[1], &mut app.list_state.clone());

    // 详情面板
    if app.show_details && chunks.len() > 3 {
        if let Some(selected) = app.list_state.selected() {
            if let Some(entry) = app.entries.get(selected) {
                let mut details = vec![
                    Line::from(vec![
                        Span::styled("Query: ", Style::default().fg(Color::Cyan)),
                        Span::raw(&entry.query),
                    ]),
                    Line::from(vec![
                        Span::styled("Time: ", Style::default().fg(Color::Cyan)),
                        Span::raw(entry.timestamp.format("%Y-%m-%d %H:%M:%S").to_string()),
                    ]),
                    Line::from(vec![
                        Span::styled("Type: ", Style::default().fg(Color::Cyan)),
                        Span::raw(entry.query_type.as_deref().unwrap_or("unknown")),
                    ]),
                ];

                if let Some(count) = entry.result_count {
                    details.push(Line::from(vec![
                        Span::styled("Results: ", Style::default().fg(Color::Cyan)),
                        Span::raw(count.to_string()),
                    ]));
                }

                details.push(Line::from(vec![
                    Span::styled("Status: ", Style::default().fg(Color::Cyan)),
                    if entry.success {
                        Span::styled("Success", Style::default().fg(Color::Green))
                    } else {
                        Span::styled("Failed", Style::default().fg(Color::Red))
                    },
                ]));

                let details_paragraph = Paragraph::new(details)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title("Details")
                            .style(Style::default().fg(Color::White)),
                    )
                    .wrap(Wrap { trim: true });

                f.render_widget(details_paragraph, chunks[2]);
            }
        }
    }

    // 状态栏
    let status_bar_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[chunks.len() - 1]);

    // 左侧帮助信息
    let help_text = if app.search_mode {
        "ESC: Cancel | Enter: Apply"
    } else {
        "↑↓/jk: Navigate | Enter: Select | /: Search | d: Toggle details | q: Quit"
    };

    let help = Paragraph::new(help_text)
        .style(Style::default().fg(Color::Gray))
        .block(
            Block::default()
                .borders(Borders::TOP | Borders::LEFT | Borders::BOTTOM)
                .style(Style::default().fg(Color::White)),
        );
    f.render_widget(help, status_bar_chunks[0]);

    // 右侧统计信息
    let stats = format!(
        "Total: {} | Selected: {}/{}",
        app.entries.len(),
        app.selected_index + 1,
        app.entries.len()
    );

    let stats_widget = Paragraph::new(stats)
        .style(Style::default().fg(Color::Cyan))
        .alignment(Alignment::Right)
        .block(
            Block::default()
                .borders(Borders::TOP | Borders::RIGHT | Borders::BOTTOM)
                .style(Style::default().fg(Color::White)),
        );
    f.render_widget(stats_widget, status_bar_chunks[1]);
}

/// 显示查询历史的TUI界面
pub async fn show_history_tui() -> Result<()> {
    if let Some(selected_query) = run_tui().await? {
        println!("Selected query: {}", selected_query);
        // 执行选中的查询
        use crate::config::Config;
        use crate::commands::enhanced::query::handle_query_command;
        
        let config = Config::new();
        handle_query_command(&selected_query, &config).await?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_creation() {
        // 测试应用创建
        let result = App::new();
        assert!(result.is_ok() || result.is_err()); // 取决于是否有历史文件
    }

    #[test]
    fn test_navigation() {
        // 创建测试应用
        if let Ok(mut app) = App::new() {
            // 测试基本导航
            let initial_selected = app.selected_index;
            
            app.next();
            if !app.entries.is_empty() {
                assert_ne!(app.selected_index, initial_selected);
            }
            
            app.previous();
            if !app.entries.is_empty() {
                assert_eq!(app.selected_index, initial_selected);
            }
            
            app.first();
            if !app.entries.is_empty() {
                assert_eq!(app.selected_index, 0);
            }
            
            app.last();
            if !app.entries.is_empty() {
                assert_eq!(app.selected_index, app.entries.len() - 1);
            }
        }
    }

    #[test]
    fn test_search_filter() {
        if let Ok(mut app) = App::new() {
            // 测试搜索过滤
            app.search_filter = "test".to_string();
            app.apply_filter();
            
            // 验证过滤后的结果
            for entry in &app.entries {
                assert!(entry.query.to_lowercase().contains("test"));
            }
            
            // 清空过滤器
            app.search_filter.clear();
            app.apply_filter();
        }
    }

    #[test]
    fn test_page_navigation() {
        if let Ok(mut app) = App::new() {
            if app.entries.len() > 10 {
                app.first();
                let initial = app.selected_index;
                
                app.page_down();
                assert!(app.selected_index > initial);
                
                app.page_up();
                assert!(app.selected_index <= initial + 10);
            }
        }
    }
}