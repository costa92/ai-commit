#[cfg(test)]
mod layout_tests {
    use crate::tui_unified::{
        config::AppConfig,
        layout::{LayoutManager, LayoutMode, manager::{STATUS_BAR_HEIGHT, MIN_TERMINAL_WIDTH, MIN_TERMINAL_HEIGHT}}
    };
    use ratatui::layout::Rect;

    #[test]
    fn test_layout_manager_creation() {
        let config = AppConfig::default();
        let manager = LayoutManager::new(&config);
        
        assert_eq!(manager.mode, LayoutMode::Normal);
        assert_eq!(manager.sidebar_width, 20);
        assert_eq!(manager.content_width, 50);
        assert_eq!(manager.detail_width, 30);
        assert!(manager.adaptive_resize);
    }

    #[test]
    fn test_normal_layout_calculation() {
        let config = AppConfig::default();
        let manager = LayoutManager::new(&config);
        let area = Rect::new(0, 0, 120, 40);
        
        let layout = manager.calculate_layout(area);
        
        // 检查基本结构
        assert!(layout.sidebar.width > 0);
        assert!(layout.content.width > 0);
        assert!(layout.detail.width > 0);
        assert_eq!(layout.status_bar.height, STATUS_BAR_HEIGHT);
        
        // 检查总宽度分配正确
        let total_width = layout.sidebar.width + layout.content.width + layout.detail.width;
        assert_eq!(total_width, area.width);
    }

    #[test]
    fn test_fullscreen_layout() {
        let config = AppConfig::default();
        let mut manager = LayoutManager::new(&config);
        manager.set_mode(LayoutMode::FullScreen);
        
        let area = Rect::new(0, 0, 120, 40);
        let layout = manager.calculate_layout(area);
        
        // 全屏模式下侧边栏和详情面板应该隐藏
        assert_eq!(layout.sidebar.width, 0);
        assert_eq!(layout.detail.width, 0);
        assert_eq!(layout.content.width, area.width);
        assert_eq!(layout.status_bar.height, STATUS_BAR_HEIGHT);
    }

    #[test]
    fn test_horizontal_split_layout() {
        let config = AppConfig::default();
        let mut manager = LayoutManager::new(&config);
        manager.set_mode(LayoutMode::SplitHorizontal);
        
        let area = Rect::new(0, 0, 120, 40);
        let layout = manager.calculate_layout(area);
        
        // 水平分屏模式应该有所有三个面板
        assert!(layout.sidebar.width > 0);
        assert!(layout.content.width > 0);
        assert!(layout.detail.width > 0);
        
        // 内容和详情应该垂直分割
        assert_eq!(layout.content.x, layout.detail.x);
        assert!(layout.content.y < layout.detail.y);
    }

    #[test]
    fn test_vertical_split_layout() {
        let config = AppConfig::default();
        let mut manager = LayoutManager::new(&config);
        manager.set_mode(LayoutMode::SplitVertical);
        
        let area = Rect::new(0, 0, 120, 40);
        let layout = manager.calculate_layout(area);
        
        // 垂直分屏模式应该有所有三个面板
        assert!(layout.sidebar.width > 0);
        assert!(layout.content.width > 0);
        assert!(layout.detail.width > 0);
        
        // 详情面板应该在底部
        assert!(layout.detail.y > layout.content.y);
        assert!(layout.detail.y > layout.sidebar.y);
    }

    #[test]
    fn test_minimal_layout_for_small_terminal() {
        let config = AppConfig::default();
        let manager = LayoutManager::new(&config);
        
        // 小于最小尺寸的终端
        let small_area = Rect::new(0, 0, 60, 20);
        let layout = manager.calculate_layout(small_area);
        
        // 小终端应该使用最小化布局（隐藏侧边栏和详情）
        assert_eq!(layout.sidebar.width, 0);
        assert_eq!(layout.detail.width, 0);
        assert_eq!(layout.content.width, small_area.width);
    }

    #[test]
    fn test_responsive_constraints() {
        let config = AppConfig::default();
        let manager = LayoutManager::new(&config);
        
        // 测试不同终端宽度的响应式行为
        let small_terminal = Rect::new(0, 0, 90, 30);
        let medium_terminal = Rect::new(0, 0, 110, 30);
        let large_terminal = Rect::new(0, 0, 150, 30);
        
        let small_layout = manager.calculate_layout(small_terminal);
        let medium_layout = manager.calculate_layout(medium_terminal);
        let large_layout = manager.calculate_layout(large_terminal);
        
        // 小终端应该有最小的侧边栏
        assert_eq!(small_layout.sidebar.width, manager.min_sidebar_width);
        
        // 大终端应该给内容更多空间
        assert!(large_layout.content.width > medium_layout.content.width);
        assert!(medium_layout.content.width > small_layout.content.width);
    }

    #[test]
    fn test_layout_mode_cycling() {
        let config = AppConfig::default();
        let mut manager = LayoutManager::new(&config);
        
        assert_eq!(manager.get_mode(), LayoutMode::Normal);
        
        manager.cycle_layout_mode();
        assert_eq!(manager.get_mode(), LayoutMode::SplitHorizontal);
        
        manager.cycle_layout_mode();
        assert_eq!(manager.get_mode(), LayoutMode::SplitVertical);
        
        manager.cycle_layout_mode();
        assert_eq!(manager.get_mode(), LayoutMode::FullScreen);
        
        manager.cycle_layout_mode();
        assert_eq!(manager.get_mode(), LayoutMode::Normal);
    }

    #[test]
    fn test_fullscreen_toggle() {
        let config = AppConfig::default();
        let mut manager = LayoutManager::new(&config);
        
        assert_eq!(manager.get_mode(), LayoutMode::Normal);
        
        manager.toggle_fullscreen();
        assert_eq!(manager.get_mode(), LayoutMode::FullScreen);
        
        manager.toggle_fullscreen();
        assert_eq!(manager.get_mode(), LayoutMode::Normal);
    }

    #[test]
    fn test_panel_width_adjustment() {
        let config = AppConfig::default();
        let mut manager = LayoutManager::new(&config);
        
        let initial_sidebar_width = manager.sidebar_width;
        let initial_detail_width = manager.detail_width;
        
        // 增加侧边栏宽度
        manager.adjust_sidebar_width(5);
        assert_eq!(manager.sidebar_width, initial_sidebar_width + 5);
        
        // 减少侧边栏宽度
        manager.adjust_sidebar_width(-3);
        assert_eq!(manager.sidebar_width, initial_sidebar_width + 2);
        
        // 测试最小宽度限制
        manager.adjust_sidebar_width(-100);
        assert_eq!(manager.sidebar_width, manager.min_sidebar_width);
        
        // 测试详情面板宽度调整
        manager.adjust_detail_width(10);
        assert_eq!(manager.detail_width, initial_detail_width + 10);
    }

    #[test]
    fn test_layout_validation() {
        let config = AppConfig::default();
        let manager = LayoutManager::new(&config);
        
        let valid_area = Rect::new(0, 0, 100, 30);
        let invalid_area = Rect::new(0, 0, 60, 20);
        
        assert!(manager.validate_layout(valid_area));
        assert!(!manager.validate_layout(invalid_area));
        
        let (min_width, min_height) = manager.get_required_minimum_size();
        assert_eq!(min_width, MIN_TERMINAL_WIDTH);
        assert_eq!(min_height, MIN_TERMINAL_HEIGHT);
    }

    #[test]
    fn test_layout_reset() {
        let config = AppConfig::default();
        let mut manager = LayoutManager::new(&config);
        
        // 修改一些设置
        manager.set_mode(LayoutMode::FullScreen);
        manager.adjust_sidebar_width(10);
        manager.adjust_detail_width(-5);
        
        // 重置布局
        manager.reset_layout();
        
        // 验证已重置为默认值
        assert_eq!(manager.mode, LayoutMode::Normal);
        assert_eq!(manager.sidebar_width, 20);
        assert_eq!(manager.content_width, 50);
        assert_eq!(manager.detail_width, 30);
    }
}