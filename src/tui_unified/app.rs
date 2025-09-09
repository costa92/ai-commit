use std::sync::Arc;
use tokio::sync::RwLock;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers}, 
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen}
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal
};

use crate::tui_unified::{
    state::{AppState, ViewType},
    layout::{LayoutManager, LayoutMode},
    focus::{FocusManager, FocusPanel},
    config::AppConfig,
    Result
};

#[derive(Debug, Clone, PartialEq)]
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
    
    // 配置
    config: AppConfig,
    
    // 运行状态
    should_quit: bool,
    current_mode: AppMode,
}

impl TuiUnifiedApp {
    pub async fn new() -> Result<Self> {
        let config = AppConfig::load().unwrap_or_default();
        let state = Arc::new(RwLock::new(AppState::new(&config).await?));
        
        Ok(Self {
            state: Arc::clone(&state),
            layout_manager: LayoutManager::new(&config),
            focus_manager: FocusManager::new(),
            config,
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
        // 初始化Git数据 (暂时跳过，后续实现)
        // self.load_initial_git_data().await?;
        
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
        
        // 渲染占位符内容 (暂时使用简单的文本块)
        use ratatui::{
            widgets::{Block, Borders, Paragraph},
            text::Text,
            style::{Color, Style}
        };
        
        // 侧边栏
        let sidebar_style = if self.focus_manager.current_panel == FocusPanel::Sidebar {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::White)
        };
        let sidebar = Paragraph::new(Text::raw("📋 Sidebar\n\n• Git Log\n• Branches\n• Tags\n• Remotes\n• Stash\n• History"))
            .block(Block::default().title("Menu").borders(Borders::ALL).border_style(sidebar_style));
        frame.render_widget(sidebar, layout.sidebar);
        
        // 主内容区
        let content_style = if self.focus_manager.current_panel == FocusPanel::Content {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::White)
        };
        let content = Paragraph::new(Text::raw("📊 Main Content\n\nThis is where the dynamic content will be displayed:\n• Commit list\n• Branch list\n• Tag list\n• etc."))
            .block(Block::default().title("Content").borders(Borders::ALL).border_style(content_style));
        frame.render_widget(content, layout.content);
        
        // 详情面板
        let detail_style = if self.focus_manager.current_panel == FocusPanel::Detail {
            Style::default().fg(Color::Yellow)  
        } else {
            Style::default().fg(Color::White)
        };
        let detail = Paragraph::new(Text::raw("🔍 Detail Panel\n\nInfo:\n• Commit details\n• Branch info\n• Diff viewer"))
            .block(Block::default().title("Details").borders(Borders::ALL).border_style(detail_style));
        frame.render_widget(detail, layout.detail);
        
        // 状态栏
        let status_text = format!("TUI Unified | Mode: {:?} | Focus: {:?} | [Tab] Switch Focus | [q] Quit", 
            self.current_mode, self.focus_manager.current_panel);
        let status_bar = Paragraph::new(Text::raw(status_text))
            .block(Block::default().borders(Borders::TOP))
            .style(Style::default().bg(Color::DarkGray));
        frame.render_widget(status_bar, layout.status_bar);
    }
    
    async fn handle_key_event(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Char('q') => {
                self.should_quit = true;
            }
            KeyCode::Tab => {
                self.focus_manager.next_focus();
            }
            KeyCode::BackTab => {
                self.focus_manager.prev_focus();
            }
            KeyCode::Char('1') => {
                self.focus_manager.set_focus(FocusPanel::Sidebar);
                // TODO: Set view to GitLog
            }
            KeyCode::Char('2') => {
                self.focus_manager.set_focus(FocusPanel::Sidebar);
                // TODO: Set view to Branches  
            }
            KeyCode::Char('3') => {
                self.focus_manager.set_focus(FocusPanel::Sidebar);
                // TODO: Set view to Tags
            }
            KeyCode::Char('?') => {
                self.current_mode = if self.current_mode == AppMode::Help { 
                    AppMode::Normal 
                } else { 
                    AppMode::Help 
                };
            }
            KeyCode::Esc => {
                self.current_mode = AppMode::Normal;
            }
            _ => {
                // 其他键处理
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
        
        // Normal -> Help
        mode = AppMode::Help;
        assert_eq!(mode, AppMode::Help);
        
        // Help -> Normal (via ESC)
        mode = AppMode::Normal;
        assert_eq!(mode, AppMode::Normal);
    }
}