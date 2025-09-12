use super::FocusPanel;
use crate::tui_unified::layout::LayoutMode;

pub struct FocusManager {
    pub current_panel: FocusPanel,
    pub panel_history: Vec<FocusPanel>,
    pub focus_ring: Vec<FocusPanel>,
    current_index: usize,
    layout_mode: LayoutMode,
    // 焦点状态
    pub is_modal_active: bool,
    pub modal_focus_saved: Option<FocusPanel>,
    // 快速跳转
    pub last_focused_panel: Option<FocusPanel>,
}

impl Default for FocusManager {
    fn default() -> Self {
        Self::new()
    }
}

impl FocusManager {
    pub fn new() -> Self {
        Self {
            current_panel: FocusPanel::Sidebar,
            panel_history: Vec::new(),
            focus_ring: vec![FocusPanel::Sidebar, FocusPanel::Content, FocusPanel::Detail],
            current_index: 0,
            layout_mode: LayoutMode::Normal,
            is_modal_active: false,
            modal_focus_saved: None,
            last_focused_panel: None,
        }
    }

    // 基础焦点导航
    pub fn next_focus(&mut self) {
        if self.is_modal_active {
            return; // 模态窗口激活时不允许焦点切换
        }

        self.update_history();
        self.current_index = (self.current_index + 1) % self.focus_ring.len();
        self.current_panel = self.focus_ring[self.current_index];
    }

    pub fn prev_focus(&mut self) {
        if self.is_modal_active {
            return;
        }

        self.update_history();
        self.current_index = if self.current_index == 0 {
            self.focus_ring.len() - 1
        } else {
            self.current_index - 1
        };
        self.current_panel = self.focus_ring[self.current_index];
    }

    pub fn set_focus(&mut self, panel: FocusPanel) {
        if self.is_modal_active || self.current_panel == panel {
            return;
        }

        // 检查面板是否在当前布局中可用
        if !self.is_panel_available(panel) {
            return;
        }

        self.update_history();
        self.current_panel = panel;

        // 更新索引
        for (i, &p) in self.focus_ring.iter().enumerate() {
            if p == panel {
                self.current_index = i;
                break;
            }
        }
    }

    pub fn direct_focus(&mut self, panel: FocusPanel) -> bool {
        if self.is_modal_active {
            return false;
        }

        if self.is_panel_available(panel) {
            self.set_focus(panel);
            true
        } else {
            false
        }
    }

    // 快速跳转和历史导航
    pub fn jump_to_last_focused(&mut self) {
        if let Some(last_panel) = self.last_focused_panel {
            if self.is_panel_available(last_panel) && last_panel != self.current_panel {
                self.set_focus(last_panel);
            }
        }
    }

    pub fn go_back(&mut self) -> bool {
        if self.is_modal_active || self.panel_history.is_empty() {
            return false;
        }

        if let Some(prev_panel) = self.panel_history.pop() {
            if self.is_panel_available(prev_panel) {
                let current = self.current_panel;
                self.current_panel = prev_panel;
                self.last_focused_panel = Some(current);

                // 更新索引
                for (i, &p) in self.focus_ring.iter().enumerate() {
                    if p == prev_panel {
                        self.current_index = i;
                        break;
                    }
                }
                return true;
            }
        }
        false
    }

    // 模态窗口焦点管理
    pub fn enter_modal_mode(&mut self) {
        if !self.is_modal_active {
            self.modal_focus_saved = Some(self.current_panel);
            self.is_modal_active = true;
        }
    }

    pub fn exit_modal_mode(&mut self) {
        if self.is_modal_active {
            self.is_modal_active = false;
            if let Some(saved_panel) = self.modal_focus_saved.take() {
                if self.is_panel_available(saved_panel) {
                    self.set_focus(saved_panel);
                }
            }
        }
    }

    // 布局模式适配
    pub fn update_for_layout(&mut self, mode: LayoutMode) {
        self.layout_mode = mode;
        self.update_focus_ring();

        // 如果当前焦点面板在新布局中不可用，切换到第一个可用面板
        if !self.is_panel_available(self.current_panel) {
            if let Some(&first_panel) = self.focus_ring.first() {
                self.current_panel = first_panel;
                self.current_index = 0;
            }
        }
    }

    fn update_focus_ring(&mut self) {
        self.focus_ring.clear();

        match self.layout_mode {
            LayoutMode::Normal => {
                self.focus_ring =
                    vec![FocusPanel::Sidebar, FocusPanel::Content, FocusPanel::Detail];
            }
            LayoutMode::SplitHorizontal => {
                self.focus_ring =
                    vec![FocusPanel::Sidebar, FocusPanel::Content, FocusPanel::Detail];
            }
            LayoutMode::SplitVertical => {
                self.focus_ring =
                    vec![FocusPanel::Sidebar, FocusPanel::Content, FocusPanel::Detail];
            }
            LayoutMode::FullScreen => {
                self.focus_ring = vec![FocusPanel::Content]; // 只有主内容可获得焦点
            }
        }

        // 重新计算当前索引
        self.current_index = self
            .focus_ring
            .iter()
            .position(|&p| p == self.current_panel)
            .unwrap_or(0);
    }

    fn is_panel_available(&self, panel: FocusPanel) -> bool {
        match self.layout_mode {
            LayoutMode::FullScreen => panel == FocusPanel::Content,
            _ => self.focus_ring.contains(&panel),
        }
    }

    fn update_history(&mut self) {
        self.last_focused_panel = Some(self.current_panel);
        self.panel_history.push(self.current_panel);

        // 限制历史记录长度
        if self.panel_history.len() > 10 {
            self.panel_history.remove(0);
        }
    }

    // 获取器方法
    pub fn get_current_panel(&self) -> FocusPanel {
        self.current_panel
    }

    pub fn get_available_panels(&self) -> &Vec<FocusPanel> {
        &self.focus_ring
    }

    pub fn has_focus(&self, panel: FocusPanel) -> bool {
        self.current_panel == panel && !self.is_modal_active
    }

    pub fn can_navigate(&self) -> bool {
        !self.is_modal_active && self.focus_ring.len() > 1
    }

    pub fn get_focus_ring_size(&self) -> usize {
        self.focus_ring.len()
    }

    pub fn get_current_index(&self) -> usize {
        self.current_index
    }

    // 高级导航方法
    pub fn focus_sidebar(&mut self) -> bool {
        self.direct_focus(FocusPanel::Sidebar)
    }

    pub fn focus_content(&mut self) -> bool {
        self.direct_focus(FocusPanel::Content)
    }

    pub fn focus_detail(&mut self) -> bool {
        self.direct_focus(FocusPanel::Detail)
    }

    // 调试和状态方法
    pub fn get_history_size(&self) -> usize {
        self.panel_history.len()
    }

    pub fn clear_history(&mut self) {
        self.panel_history.clear();
        self.last_focused_panel = None;
    }

    pub fn get_layout_mode(&self) -> LayoutMode {
        self.layout_mode
    }
}
