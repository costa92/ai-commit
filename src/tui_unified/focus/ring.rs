use super::FocusPanel;
use crate::tui_unified::layout::LayoutMode;

/// 高级焦点环实现，支持动态焦点管理和智能导航
pub struct FocusRing {
    panels: Vec<FocusPanel>,
    current_index: usize,
    layout_mode: LayoutMode,
    // 智能导航功能
    navigation_history: Vec<usize>,
    preferred_order: Vec<FocusPanel>,
    _skip_hidden: bool, // 保留用于未来功能
}

impl FocusRing {
    pub fn new() -> Self {
        Self {
            panels: vec![FocusPanel::Sidebar, FocusPanel::Content, FocusPanel::Detail],
            current_index: 0,
            layout_mode: LayoutMode::Normal,
            navigation_history: Vec::new(),
            preferred_order: vec![FocusPanel::Sidebar, FocusPanel::Content, FocusPanel::Detail],
            _skip_hidden: true,
        }
    }

    pub fn with_layout(layout_mode: LayoutMode) -> Self {
        let mut ring = Self::new();
        ring.update_for_layout(layout_mode);
        ring
    }

    /// 更新焦点环以适应布局模式
    pub fn update_for_layout(&mut self, layout_mode: LayoutMode) {
        self.layout_mode = layout_mode;

        // 根据布局模式更新可用面板
        match layout_mode {
            LayoutMode::Normal => {
                self.panels = vec![FocusPanel::Sidebar, FocusPanel::Content, FocusPanel::Detail];
            }
            LayoutMode::SplitHorizontal => {
                self.panels = vec![FocusPanel::Sidebar, FocusPanel::Content, FocusPanel::Detail];
            }
            LayoutMode::SplitVertical => {
                self.panels = vec![FocusPanel::Sidebar, FocusPanel::Content, FocusPanel::Detail];
            }
            LayoutMode::FullScreen => {
                self.panels = vec![FocusPanel::Content]; // 只有主内容
            }
        }

        // 验证当前索引
        if self.current_index >= self.panels.len() {
            self.current_index = 0;
        }
    }

    /// 移动到下一个焦点
    pub fn next(&mut self) -> Option<FocusPanel> {
        if self.panels.is_empty() {
            return None;
        }

        self.save_navigation_state();
        self.current_index = (self.current_index + 1) % self.panels.len();
        Some(self.panels[self.current_index])
    }

    /// 移动到上一个焦点
    pub fn prev(&mut self) -> Option<FocusPanel> {
        if self.panels.is_empty() {
            return None;
        }

        self.save_navigation_state();
        self.current_index = if self.current_index == 0 {
            self.panels.len() - 1
        } else {
            self.current_index - 1
        };
        Some(self.panels[self.current_index])
    }

    /// 直接跳转到指定面板
    pub fn jump_to(&mut self, panel: FocusPanel) -> bool {
        if let Some(index) = self.panels.iter().position(|&p| p == panel) {
            self.save_navigation_state();
            self.current_index = index;
            true
        } else {
            false
        }
    }

    /// 智能导航到最相关的面板
    pub fn navigate_smart(&mut self, direction: NavigationDirection) -> Option<FocusPanel> {
        match direction {
            NavigationDirection::Forward => self.next(),
            NavigationDirection::Backward => self.prev(),
            NavigationDirection::ToContent => {
                self.jump_to(FocusPanel::Content);
                Some(FocusPanel::Content)
            }
            NavigationDirection::ToSidebar => {
                if self.jump_to(FocusPanel::Sidebar) {
                    Some(FocusPanel::Sidebar)
                } else {
                    self.next()
                }
            }
            NavigationDirection::ToDetail => {
                if self.jump_to(FocusPanel::Detail) {
                    Some(FocusPanel::Detail)
                } else {
                    self.next()
                }
            }
        }
    }

    /// 获取当前焦点面板
    pub fn current(&self) -> Option<FocusPanel> {
        self.panels.get(self.current_index).copied()
    }

    /// 获取当前索引
    pub fn current_index(&self) -> usize {
        self.current_index
    }

    /// 获取所有可用面板
    pub fn available_panels(&self) -> &[FocusPanel] {
        &self.panels
    }

    /// 检查面板是否可用
    pub fn is_available(&self, panel: FocusPanel) -> bool {
        self.panels.contains(&panel)
    }

    /// 获取面板数量
    pub fn len(&self) -> usize {
        self.panels.len()
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.panels.is_empty()
    }

    /// 获取下一个面板（不移动焦点）
    pub fn peek_next(&self) -> Option<FocusPanel> {
        if self.panels.is_empty() {
            return None;
        }
        let next_index = (self.current_index + 1) % self.panels.len();
        Some(self.panels[next_index])
    }

    /// 获取上一个面板（不移动焦点）
    pub fn peek_prev(&self) -> Option<FocusPanel> {
        if self.panels.is_empty() {
            return None;
        }
        let prev_index = if self.current_index == 0 {
            self.panels.len() - 1
        } else {
            self.current_index - 1
        };
        Some(self.panels[prev_index])
    }

    /// 重置到默认状态
    pub fn reset(&mut self) {
        self.current_index = 0;
        self.navigation_history.clear();
        self.update_for_layout(self.layout_mode);
    }

    /// 设置首选导航顺序
    pub fn set_preferred_order(&mut self, order: Vec<FocusPanel>) {
        self.preferred_order = order;
        // 重新排列当前面板以匹配首选顺序
        self.apply_preferred_order();
    }

    /// 回到上一个焦点位置
    pub fn go_back(&mut self) -> Option<FocusPanel> {
        if let Some(last_index) = self.navigation_history.pop() {
            if last_index < self.panels.len() {
                self.current_index = last_index;
                return Some(self.panels[self.current_index]);
            }
        }
        None
    }

    // 私有辅助方法
    fn save_navigation_state(&mut self) {
        self.navigation_history.push(self.current_index);
        // 限制历史记录长度
        if self.navigation_history.len() > 5 {
            self.navigation_history.remove(0);
        }
    }

    fn apply_preferred_order(&mut self) {
        let mut new_panels = Vec::new();

        // 按首选顺序添加可用面板
        for preferred_panel in &self.preferred_order {
            if self.panels.contains(preferred_panel) {
                new_panels.push(*preferred_panel);
            }
        }

        // 添加任何不在首选列表中的面板
        for panel in &self.panels {
            if !new_panels.contains(panel) {
                new_panels.push(*panel);
            }
        }

        // 保存当前面板
        let current_panel = self.current();
        self.panels = new_panels;

        // 恢复当前焦点位置
        if let Some(panel) = current_panel {
            if let Some(index) = self.panels.iter().position(|&p| p == panel) {
                self.current_index = index;
            } else {
                self.current_index = 0;
            }
        }
    }
}

/// 导航方向枚举
pub enum NavigationDirection {
    Forward,   // 向前导航
    Backward,  // 向后导航
    ToContent, // 跳转到内容面板
    ToSidebar, // 跳转到侧边栏
    ToDetail,  // 跳转到详情面板
}

impl Default for FocusRing {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_focus_ring_creation() {
        let ring = FocusRing::new();
        assert_eq!(ring.len(), 3);
        assert_eq!(ring.current(), Some(FocusPanel::Sidebar));
    }

    #[test]
    fn test_focus_navigation() {
        let mut ring = FocusRing::new();

        // 测试前进导航
        assert_eq!(ring.next(), Some(FocusPanel::Content));
        assert_eq!(ring.next(), Some(FocusPanel::Detail));
        assert_eq!(ring.next(), Some(FocusPanel::Sidebar)); // 循环回到开始

        // 测试后退导航
        assert_eq!(ring.prev(), Some(FocusPanel::Detail));
        assert_eq!(ring.prev(), Some(FocusPanel::Content));
    }

    #[test]
    fn test_layout_adaptation() {
        let mut ring = FocusRing::new();

        // 全屏模式只有内容面板
        ring.update_for_layout(LayoutMode::FullScreen);
        assert_eq!(ring.len(), 1);
        assert_eq!(ring.current(), Some(FocusPanel::Content));

        // 返回正常模式
        ring.update_for_layout(LayoutMode::Normal);
        assert_eq!(ring.len(), 3);
    }

    #[test]
    fn test_direct_jump() {
        let mut ring = FocusRing::new();

        assert!(ring.jump_to(FocusPanel::Detail));
        assert_eq!(ring.current(), Some(FocusPanel::Detail));

        // 在全屏模式下跳转到不可用的面板应该失败
        ring.update_for_layout(LayoutMode::FullScreen);
        assert!(!ring.jump_to(FocusPanel::Sidebar));
    }
}
