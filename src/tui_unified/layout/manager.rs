use ratatui::layout::{Constraint, Direction, Layout, Rect};
use crate::tui_unified::{
    config::AppConfig,
    app::LayoutResult
};
use super::LayoutMode;

pub struct LayoutManager {
    pub mode: LayoutMode,
    pub sidebar_width: u16,
    pub content_width: u16,
    pub detail_width: u16,
}

impl LayoutManager {
    pub fn new(config: &AppConfig) -> Self {
        Self {
            mode: LayoutMode::Normal,
            sidebar_width: 20,
            content_width: 50,
            detail_width: 30,
        }
    }
    
    pub fn calculate_layout(&self, area: Rect) -> LayoutResult {
        match self.mode {
            LayoutMode::Normal => self.calculate_normal_layout(area),
            LayoutMode::FullScreen => self.calculate_fullscreen_layout(area),
            _ => self.calculate_normal_layout(area), // 暂时都用普通布局
        }
    }
    
    fn calculate_normal_layout(&self, area: Rect) -> LayoutResult {
        // 主要区域 (除了状态栏)
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(area.height.saturating_sub(3)), // 主要内容
                Constraint::Length(3), // 状态栏
            ])
            .split(area);
        
        // 三栏布局
        let content_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(20), // 侧边栏
                Constraint::Percentage(50), // 主内容
                Constraint::Percentage(30), // 详情面板
            ])
            .split(main_chunks[0]);
        
        LayoutResult {
            sidebar: content_chunks[0],
            content: content_chunks[1],
            detail: content_chunks[2],
            status_bar: main_chunks[1],
        }
    }
    
    fn calculate_fullscreen_layout(&self, area: Rect) -> LayoutResult {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(area.height.saturating_sub(3)),
                Constraint::Length(3),
            ])
            .split(area);
        
        LayoutResult {
            sidebar: Rect::default(), // 隐藏
            content: chunks[0], // 全屏
            detail: Rect::default(), // 隐藏
            status_bar: chunks[1],
        }
    }
    
    pub fn set_mode(&mut self, mode: LayoutMode) {
        self.mode = mode;
    }
}