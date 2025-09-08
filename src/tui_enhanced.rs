use crate::query_history::{QueryHistory, QueryHistoryEntry};
use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap, Clear, Tabs, Scrollbar, ScrollbarOrientation},
    Frame, Terminal,
};
use std::io;

/// 视图类型
#[derive(Clone, Debug, PartialEq)]
pub enum ViewType {
    History,
    Results,
    Diff,
    Stats,
}

/// 标签页
pub struct Tab {
    pub name: String,
    pub view_type: ViewType,
    pub content: String,
}

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
    /// 标签页列表
    tabs: Vec<Tab>,
    /// 当前标签页索引
    current_tab: usize,
    /// 分屏模式
    split_mode: SplitMode,
    /// 当前焦点窗口
    focused_pane: FocusedPane,
    /// 命令行模式
    command_mode: bool,
    /// 命令行输入
    command_input: String,
    /// 结果滚动位置
    result_scroll: u16,
    /// 高亮的查询语法
    syntax_highlight: bool,
}

/// 分屏模式
#[derive(Clone, Debug, PartialEq)]
pub enum SplitMode {
    None,
    Horizontal,
    Vertical,
}

/// 焦点窗口
#[derive(Clone, Debug, PartialEq)]
pub enum FocusedPane {
    Left,
    Right,
    Top,
    Bottom,
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

        let tabs = vec![
            Tab {
                name: "History".to_string(),
                view_type: ViewType::History,
                content: String::new(),
            },
        ];

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
            tabs,
            current_tab: 0,
            split_mode: SplitMode::None,
            focused_pane: FocusedPane::Left,
            command_mode: false,
            command_input: String::new(),
            result_scroll: 0,
            syntax_highlight: true,
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

    /// 切换到下一个标签页
    fn next_tab(&mut self) {
        if !self.tabs.is_empty() {
            self.current_tab = (self.current_tab + 1) % self.tabs.len();
        }
    }

    /// 切换到上一个标签页
    fn prev_tab(&mut self) {
        if !self.tabs.is_empty() {
            if self.current_tab == 0 {
                self.current_tab = self.tabs.len() - 1;
            } else {
                self.current_tab -= 1;
            }
        }
    }

    /// 切换分屏模式
    fn toggle_split(&mut self) {
        self.split_mode = match self.split_mode {
            SplitMode::None => SplitMode::Horizontal,
            SplitMode::Horizontal => SplitMode::Vertical,
            SplitMode::Vertical => SplitMode::None,
        };
    }

    /// 切换焦点窗口
    fn toggle_focus(&mut self) {
        self.focused_pane = match (&self.split_mode, &self.focused_pane) {
            (SplitMode::Horizontal, FocusedPane::Top) => FocusedPane::Bottom,
            (SplitMode::Horizontal, _) => FocusedPane::Top,
            (SplitMode::Vertical, FocusedPane::Left) => FocusedPane::Right,
            (SplitMode::Vertical, _) => FocusedPane::Left,
            _ => FocusedPane::Left,
        };
    }

    /// 执行选中的查询
    async fn execute_selected_query(&mut self) {
        if let Some(query) = self.get_selected_query() {
            self.execute_query = Some(query.clone());
            
            // 执行查询并获取结果
            use crate::config::Config;
            use crate::git::GitQuery;
            
            let _config = Config::new();
            match GitQuery::parse_query(&query) {
                Ok(filters) => {
                    match GitQuery::execute_query(&filters).await {
                        Ok(results) => {
                            let result_count = results.lines().count();
                            
                            // 创建新的结果标签页
                            let tab = Tab {
                                name: format!("Results: {}", query.chars().take(20).collect::<String>()),
                                view_type: ViewType::Results,
                                content: results.clone(),
                            };
                            
                            // 查找是否已存在相同的标签页
                            let existing = self.tabs.iter().position(|t| t.view_type == ViewType::Results);
                            if let Some(idx) = existing {
                                self.tabs[idx] = tab;
                                self.current_tab = idx;
                            } else {
                                self.tabs.push(tab);
                                self.current_tab = self.tabs.len() - 1;
                            }
                            
                            // 如果没有分屏，自动启用
                            if self.split_mode == SplitMode::None {
                                self.split_mode = SplitMode::Vertical;
                            }
                            
                            self.execution_result = Some(format!(
                                "Query executed: {} results found",
                                result_count
                            ));
                            
                            // 更新历史记录
                            let _ = self.history.add_entry(
                                query,
                                Some("execute".to_string()),
                                Some(result_count),
                                true
                            );
                        }
                        Err(e) => {
                            self.execution_result = Some(format!("Error executing query: {}", e));
                        }
                    }
                }
                Err(e) => {
                    self.execution_result = Some(format!("Error parsing query: {}", e));
                }
            }
        }
    }

    /// 获取选中的查询
    pub fn get_selected_query(&self) -> Option<String> {
        self.list_state.selected()
            .and_then(|i| self.entries.get(i))
            .map(|entry| entry.query.clone())
    }

    /// 执行命令
    fn execute_command(&mut self) {
        let parts: Vec<&str> = self.command_input.split_whitespace().collect();
        if parts.is_empty() {
            return;
        }

        match parts[0] {
            "q" | "quit" => self.should_quit = true,
            "split" => self.toggle_split(),
            "vsplit" => self.split_mode = SplitMode::Vertical,
            "hsplit" => self.split_mode = SplitMode::Horizontal,
            "tab" => {
                if parts.len() > 1 {
                    let tab = Tab {
                        name: parts[1].to_string(),
                        view_type: ViewType::History,
                        content: String::new(),
                    };
                    self.tabs.push(tab);
                }
            }
            "help" => self.show_help = true,
            _ => {}
        }

        self.command_input.clear();
        self.command_mode = false;
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
                // 命令模式
                if app.command_mode {
                    match key.code {
                        KeyCode::Esc => {
                            app.command_mode = false;
                            app.command_input.clear();
                        }
                        KeyCode::Enter => {
                            app.execute_command();
                        }
                        KeyCode::Backspace => {
                            app.command_input.pop();
                        }
                        KeyCode::Char(c) => {
                            app.command_input.push(c);
                        }
                        _ => {}
                    }
                    continue;
                }

                // 搜索模式
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
                    continue;
                }

                // 帮助模式
                if app.show_help {
                    match key.code {
                        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('?') => {
                            app.show_help = false;
                        }
                        _ => {}
                    }
                    continue;
                }

                // 正常模式快捷键
                match (key.modifiers, key.code) {
                    // Ctrl组合键
                    (KeyModifiers::CONTROL, KeyCode::Char('c')) => {
                        app.should_quit = true;
                        return Ok(());
                    }
                    (KeyModifiers::CONTROL, KeyCode::Char('w')) => {
                        app.toggle_focus();
                    }
                    (KeyModifiers::CONTROL, KeyCode::Char('s')) => {
                        app.toggle_split();
                    }
                    // 普通按键
                    (_, KeyCode::Char('q')) => {
                        app.should_quit = true;
                        return Ok(());
                    }
                    (_, KeyCode::Char(':')) => {
                        app.command_mode = true;
                        app.command_input.clear();
                    }
                    (_, KeyCode::Tab) => {
                        if key.modifiers.contains(KeyModifiers::SHIFT) {
                            app.prev_tab();
                        } else {
                            app.next_tab();
                        }
                    }
                    (_, KeyCode::Enter) | (_, KeyCode::Char('x')) => {
                        app.execute_selected_query().await;
                    }
                    (_, KeyCode::Down) | (_, KeyCode::Char('j')) => {
                        if app.focused_pane == FocusedPane::Left || app.split_mode == SplitMode::None {
                            app.next();
                        } else {
                            app.result_scroll = app.result_scroll.saturating_add(1);
                        }
                    }
                    (_, KeyCode::Up) | (_, KeyCode::Char('k')) => {
                        if app.focused_pane == FocusedPane::Left || app.split_mode == SplitMode::None {
                            app.previous();
                        } else {
                            app.result_scroll = app.result_scroll.saturating_sub(1);
                        }
                    }
                    (_, KeyCode::Char('d')) => app.show_details = !app.show_details,
                    (_, KeyCode::Char('/')) => {
                        app.search_mode = true;
                        app.search_filter.clear();
                    }
                    (_, KeyCode::Char('?')) => {
                        app.show_help = true;
                    }
                    (_, KeyCode::Char('h')) => app.syntax_highlight = !app.syntax_highlight,
                    _ => {}
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
    // 显示帮助弹窗
    if app.show_help {
        render_help(f, f.size());
        return;
    }

    // 主布局
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),    // 标签栏
            Constraint::Min(0),       // 内容区
            Constraint::Length(1),    // 命令行/状态栏
        ])
        .split(f.size());

    // 渲染标签栏
    render_tabs(f, app, main_chunks[0]);

    // 根据分屏模式渲染内容
    match app.split_mode {
        SplitMode::None => {
            render_single_view(f, app, main_chunks[1]);
        }
        SplitMode::Horizontal => {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(main_chunks[1]);
            
            render_history_view(f, app, chunks[0], app.focused_pane == FocusedPane::Top);
            render_result_view(f, app, chunks[1], app.focused_pane == FocusedPane::Bottom);
        }
        SplitMode::Vertical => {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(main_chunks[1]);
            
            render_history_view(f, app, chunks[0], app.focused_pane == FocusedPane::Left);
            render_result_view(f, app, chunks[1], app.focused_pane == FocusedPane::Right);
        }
    }

    // 渲染命令行或状态栏
    if app.command_mode {
        render_command_line(f, app, main_chunks[2]);
    } else {
        render_status_bar(f, app, main_chunks[2]);
    }
}

/// 渲染标签栏
fn render_tabs(f: &mut Frame, app: &App, area: Rect) {
    let titles: Vec<String> = app.tabs.iter().map(|t| t.name.clone()).collect();
    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::BOTTOM))
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .select(app.current_tab);
    
    f.render_widget(tabs, area);
}

/// 渲染单视图
fn render_single_view(f: &mut Frame, app: &App, area: Rect) {
    if app.tabs.is_empty() {
        return;
    }
    
    match app.tabs[app.current_tab].view_type {
        ViewType::History => render_history_view(f, app, area, true),
        ViewType::Results => render_result_view(f, app, area, true),
        _ => {}
    }
}

/// 渲染历史视图
fn render_history_view(f: &mut Frame, app: &App, area: Rect, focused: bool) {
    let border_style = if focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::White)
    };

    // 分割区域
    let chunks = if app.show_details {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(area)
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(100)])
            .split(area)
    };

    // 历史列表
    let items: Vec<ListItem> = app
        .entries
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            let status_icon = if entry.success { "✅" } else { "❌" };
            let timestamp = entry.timestamp.format("%H:%M:%S");
            
            let content = if app.syntax_highlight {
                // 语法高亮
                let parts: Vec<&str> = entry.query.split(':').collect();
                if parts.len() == 2 {
                    format!("{} {} {}:{}", 
                        status_icon, 
                        timestamp,
                        parts[0],
                        parts[1]
                    )
                } else {
                    format!("{} {} {}", status_icon, timestamp, entry.query)
                }
            } else {
                format!("{} {} {}", status_icon, timestamp, entry.query)
            };

            let style = if Some(i) == app.list_state.selected() {
                Style::default()
                    .bg(Color::DarkGray)
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

    let title = if app.search_mode {
        format!(" History [/{}] ", app.search_filter)
    } else {
        " History ".to_string()
    };

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(border_style),
        );

    f.render_stateful_widget(list, chunks[0], &mut app.list_state.clone());

    // 详情面板
    if app.show_details && chunks.len() > 1 {
        if let Some(selected) = app.list_state.selected() {
            if let Some(entry) = app.entries.get(selected) {
                let details_text = format!(
                    "Query: {}\nTime: {}\nType: {}\nResults: {}\nStatus: {}",
                    entry.query,
                    entry.timestamp.format("%Y-%m-%d %H:%M:%S"),
                    entry.query_type.as_deref().unwrap_or("unknown"),
                    entry.result_count.map_or("N/A".to_string(), |c| c.to_string()),
                    if entry.success { "Success" } else { "Failed" }
                );

                let details = Paragraph::new(details_text)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title(" Details ")
                            .border_style(border_style),
                    )
                    .wrap(Wrap { trim: true });

                f.render_widget(details, chunks[1]);
            }
        }
    }
}

/// 渲染结果视图
fn render_result_view(f: &mut Frame, app: &App, area: Rect, focused: bool) {
    let border_style = if focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::White)
    };

    // 查找结果标签页
    let result_content = app.tabs
        .iter()
        .find(|t| t.view_type == ViewType::Results)
        .map(|t| t.content.clone())
        .unwrap_or_else(|| "No results to display".to_string());

    let lines: Vec<String> = result_content.lines().map(|l| l.to_string()).collect();
    let start = app.result_scroll as usize;
    let visible_lines: Vec<ListItem> = lines
        .iter()
        .skip(start)
        .take(area.height as usize - 2)
        .map(|line| {
            let style = if line.contains("feat") {
                Style::default().fg(Color::Green)
            } else if line.contains("fix") {
                Style::default().fg(Color::Yellow)
            } else if line.contains("docs") {
                Style::default().fg(Color::Blue)
            } else {
                Style::default()
            };
            ListItem::new(line.as_str()).style(style)
        })
        .collect();

    let list = List::new(visible_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Results ")
                .border_style(border_style),
        );

    f.render_widget(list, area);

    // 滚动条
    if lines.len() > area.height as usize - 2 {
        let scrollbar = Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"));
        
        let mut scrollbar_state = ratatui::widgets::ScrollbarState::default()
            .content_length(lines.len())
            .position(app.result_scroll as usize);
        
        f.render_stateful_widget(
            scrollbar,
            area.inner(&ratatui::layout::Margin {
                vertical: 1,
                horizontal: 0,
            }),
            &mut scrollbar_state,
        );
    }
}

/// 渲染帮助
fn render_help(f: &mut Frame, area: Rect) {
    let area = centered_rect(70, 80, area);
    f.render_widget(Clear, area);
    
    let help_text = vec![
        Line::from(""),
        Line::from(vec![Span::styled("Navigation", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))]),
        Line::from("  ↑/k        Move up"),
        Line::from("  ↓/j        Move down"),
        Line::from("  Tab        Next tab"),
        Line::from("  Shift+Tab  Previous tab"),
        Line::from(""),
        Line::from(vec![Span::styled("Actions", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))]),
        Line::from("  Enter/x    Execute query"),
        Line::from("  /          Search"),
        Line::from("  d          Toggle details"),
        Line::from("  h          Toggle syntax highlighting"),
        Line::from(""),
        Line::from(vec![Span::styled("Window Management", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))]),
        Line::from("  Ctrl+s     Toggle split mode"),
        Line::from("  Ctrl+w     Switch focus"),
        Line::from("  :vsplit    Vertical split"),
        Line::from("  :hsplit    Horizontal split"),
        Line::from(""),
        Line::from(vec![Span::styled("Commands", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))]),
        Line::from("  :          Command mode"),
        Line::from("  :q         Quit"),
        Line::from("  :tab NAME  New tab"),
        Line::from("  ?          This help"),
    ];
    
    let help = Paragraph::new(help_text)
        .block(Block::default()
            .title(" Help ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green)))
        .style(Style::default().fg(Color::White));
        
    f.render_widget(help, area);
}

/// 渲染命令行
fn render_command_line(f: &mut Frame, app: &App, area: Rect) {
    let input = Paragraph::new(format!(":{}", app.command_input))
        .style(Style::default().fg(Color::Yellow));
    f.render_widget(input, area);
}

/// 渲染状态栏
fn render_status_bar(f: &mut Frame, app: &App, area: Rect) {
    let mode = if app.search_mode {
        "SEARCH"
    } else {
        "NORMAL"
    };
    
    let split_info = match app.split_mode {
        SplitMode::None => "",
        SplitMode::Horizontal => " [H-SPLIT]",
        SplitMode::Vertical => " [V-SPLIT]",
    };
    
    let status = format!(
        " {} | Tab {}/{} | {} entries{}",
        mode,
        app.current_tab + 1,
        app.tabs.len(),
        app.entries.len(),
        split_info
    );
    
    let status_bar = Paragraph::new(status)
        .style(Style::default().fg(Color::Cyan).bg(Color::DarkGray));
    
    f.render_widget(status_bar, area);
}

/// 计算居中矩形
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

/// 显示查询历史的TUI界面
pub async fn show_history_tui() -> Result<()> {
    run_tui().await?;
    Ok(())
}