use super::LayoutMode;
use crate::tui_unified::{app::LayoutResult, config::AppConfig};
use ratatui::layout::{Constraint, Direction, Layout, Rect};

// 布局常量
pub const MIN_TERMINAL_WIDTH: u16 = 80;
pub const MIN_TERMINAL_HEIGHT: u16 = 24;
pub const STATUS_BAR_HEIGHT: u16 = 3;
pub const MIN_SIDEBAR_WIDTH: u16 = 15;
pub const MIN_CONTENT_WIDTH: u16 = 30;
pub const MIN_DETAIL_WIDTH: u16 = 20;

pub struct LayoutManager {
    pub mode: LayoutMode,
    pub sidebar_width: u16,
    pub content_width: u16,
    pub detail_width: u16,
    // 响应式参数
    pub min_sidebar_width: u16,
    pub min_content_width: u16,
    pub min_detail_width: u16,
    pub adaptive_resize: bool,
}

impl LayoutManager {
    pub fn new(_config: &AppConfig) -> Self {
        Self {
            mode: LayoutMode::Normal,
            sidebar_width: 20,
            content_width: 50,
            detail_width: 30,
            min_sidebar_width: MIN_SIDEBAR_WIDTH,
            min_content_width: MIN_CONTENT_WIDTH,
            min_detail_width: MIN_DETAIL_WIDTH,
            adaptive_resize: true,
        }
    }

    pub fn calculate_layout(&self, area: Rect) -> LayoutResult {
        // 检查终端最小尺寸
        if area.width < MIN_TERMINAL_WIDTH || area.height < MIN_TERMINAL_HEIGHT {
            return self.calculate_minimal_layout(area);
        }

        match self.mode {
            LayoutMode::Normal => self.calculate_normal_layout(area),
            LayoutMode::SplitHorizontal => self.calculate_horizontal_split_layout(area),
            LayoutMode::SplitVertical => self.calculate_vertical_split_layout(area),
            LayoutMode::FullScreen => self.calculate_fullscreen_layout(area),
        }
    }

    fn calculate_normal_layout(&self, area: Rect) -> LayoutResult {
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(area.height.saturating_sub(STATUS_BAR_HEIGHT)),
                Constraint::Length(STATUS_BAR_HEIGHT),
            ])
            .split(area);

        // 响应式三栏布局
        let (sidebar_constraint, content_constraint, detail_constraint) =
            self.calculate_responsive_constraints(area.width);

        let content_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                sidebar_constraint, // 侧边栏
                content_constraint, // 主内容
                detail_constraint,  // 详情面板
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
                Constraint::Length(area.height.saturating_sub(STATUS_BAR_HEIGHT)),
                Constraint::Length(STATUS_BAR_HEIGHT),
            ])
            .split(area);

        LayoutResult {
            sidebar: Rect::default(), // 隐藏
            content: chunks[0],       // 全屏
            detail: Rect::default(),  // 隐藏
            status_bar: chunks[1],
        }
    }

    fn calculate_horizontal_split_layout(&self, area: Rect) -> LayoutResult {
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(area.height.saturating_sub(STATUS_BAR_HEIGHT)),
                Constraint::Length(STATUS_BAR_HEIGHT),
            ])
            .split(area);

        // 水平分屏：侧边栏 | 上下分屏(主内容/详情)
        let horizontal_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(self.min_sidebar_width), // 侧边栏
                Constraint::Min(40),                        // 分屏区域
            ])
            .split(main_chunks[0]);

        let split_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(60), // 主内容 (上)
                Constraint::Percentage(40), // 详情 (下)
            ])
            .split(horizontal_chunks[1]);

        LayoutResult {
            sidebar: horizontal_chunks[0],
            content: split_chunks[0],
            detail: split_chunks[1],
            status_bar: main_chunks[1],
        }
    }

    fn calculate_vertical_split_layout(&self, area: Rect) -> LayoutResult {
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(area.height.saturating_sub(STATUS_BAR_HEIGHT)),
                Constraint::Length(STATUS_BAR_HEIGHT),
            ])
            .split(area);

        // 垂直分屏：左右分屏 + 底部详情面板
        let vertical_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(70), // 主分屏区域
                Constraint::Percentage(30), // 详情面板
            ])
            .split(main_chunks[0]);

        let split_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(25), // 侧边栏
                Constraint::Percentage(75), // 主内容
            ])
            .split(vertical_chunks[0]);

        LayoutResult {
            sidebar: split_chunks[0],
            content: split_chunks[1],
            detail: vertical_chunks[1],
            status_bar: main_chunks[1],
        }
    }

    fn calculate_minimal_layout(&self, area: Rect) -> LayoutResult {
        // 最小化布局：只显示主内容和状态栏
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(area.height.saturating_sub(STATUS_BAR_HEIGHT)),
                Constraint::Length(STATUS_BAR_HEIGHT),
            ])
            .split(area);

        LayoutResult {
            sidebar: Rect::default(), // 隐藏
            content: chunks[0],       // 全屏主内容
            detail: Rect::default(),  // 隐藏
            status_bar: chunks[1],
        }
    }

    // 响应式约束计算
    fn calculate_responsive_constraints(
        &self,
        terminal_width: u16,
    ) -> (Constraint, Constraint, Constraint) {
        if !self.adaptive_resize {
            // 固定比例模式
            return (
                Constraint::Percentage(20),
                Constraint::Percentage(50),
                Constraint::Percentage(30),
            );
        }

        // 根据终端宽度自适应调整
        match terminal_width {
            w if w < 100 => {
                // 小屏幕：缩小侧边栏，保持主内容
                (
                    Constraint::Length(self.min_sidebar_width),
                    Constraint::Min(self.min_content_width),
                    Constraint::Length(self.min_detail_width),
                )
            }
            w if w < 120 => {
                // 中等屏幕：平衡分配
                (
                    Constraint::Percentage(18),
                    Constraint::Percentage(52),
                    Constraint::Percentage(30),
                )
            }
            _ => {
                // 大屏幕：给主内容更多空间
                (
                    Constraint::Percentage(15),
                    Constraint::Percentage(60),
                    Constraint::Percentage(25),
                )
            }
        }
    }

    // 布局管理方法
    pub fn set_mode(&mut self, mode: LayoutMode) {
        self.mode = mode;
    }

    pub fn get_mode(&self) -> LayoutMode {
        self.mode
    }

    pub fn toggle_fullscreen(&mut self) {
        self.mode = match self.mode {
            LayoutMode::FullScreen => LayoutMode::Normal,
            _ => LayoutMode::FullScreen,
        };
    }

    pub fn cycle_layout_mode(&mut self) {
        self.mode = match self.mode {
            LayoutMode::Normal => LayoutMode::SplitHorizontal,
            LayoutMode::SplitHorizontal => LayoutMode::SplitVertical,
            LayoutMode::SplitVertical => LayoutMode::FullScreen,
            LayoutMode::FullScreen => LayoutMode::Normal,
        };
    }

    // 面板尺寸调整
    pub fn adjust_sidebar_width(&mut self, delta: i16) {
        let new_width =
            (self.sidebar_width as i16 + delta).max(self.min_sidebar_width as i16) as u16;
        self.sidebar_width = new_width.min(40); // 最大40个字符宽度
    }

    pub fn adjust_detail_width(&mut self, delta: i16) {
        let new_width = (self.detail_width as i16 + delta).max(self.min_detail_width as i16) as u16;
        self.detail_width = new_width.min(50); // 最大50个字符宽度
    }

    pub fn reset_layout(&mut self) {
        self.sidebar_width = 20;
        self.content_width = 50;
        self.detail_width = 30;
        self.mode = LayoutMode::Normal;
    }

    // 布局验证
    pub fn validate_layout(&self, area: Rect) -> bool {
        area.width >= MIN_TERMINAL_WIDTH && area.height >= MIN_TERMINAL_HEIGHT
    }

    pub fn get_required_minimum_size(&self) -> (u16, u16) {
        (MIN_TERMINAL_WIDTH, MIN_TERMINAL_HEIGHT)
    }
}
