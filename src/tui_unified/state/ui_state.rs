use crate::tui_unified::focus::FocusPanel;
use ratatui::layout::Rect;

#[derive(Debug, Clone)]
pub struct LayoutState {
    pub sidebar_width: u16,
    pub content_width: u16,
    pub detail_width: u16,
    pub terminal_size: Rect,
    pub layout_mode: LayoutMode,
    pub panel_ratios: PanelRatios,
    pub min_panel_sizes: MinPanelSizes,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LayoutMode {
    Normal,
    SplitHorizontal,
    SplitVertical,
    FullScreen(FocusPanel),
    Minimized(FocusPanel),
}

#[derive(Debug, Clone)]
pub struct PanelRatios {
    pub sidebar_ratio: f32,
    pub content_ratio: f32,
    pub detail_ratio: f32,
}

#[derive(Debug, Clone)]
pub struct MinPanelSizes {
    pub sidebar_min: u16,
    pub content_min: u16,
    pub detail_min: u16,
}

#[derive(Debug, Clone)]
pub struct FocusState {
    pub current_panel: FocusPanel,
    pub panel_history: Vec<FocusPanel>,
    pub can_navigate: bool,
    pub focus_ring: FocusRing,
}

#[derive(Debug, Clone)]
pub struct FocusRing {
    pub panels: Vec<FocusPanel>,
    pub current_index: usize,
    pub wrap_around: bool,
}

#[derive(Debug, Clone)]
pub struct ModalState {
    pub modal_type: ModalType,
    pub title: String,
    pub content: String,
    pub size: ModalSize,
    pub position: ModalPosition,
    pub can_close: bool,
    pub buttons: Vec<ModalButton>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ModalType {
    Info,
    Warning,
    Error,
    Confirm,
    Input,
    Progress,
    Custom,
}

#[derive(Debug, Clone)]
pub struct ModalSize {
    pub width: u16,
    pub height: u16,
    pub max_width: Option<u16>,
    pub max_height: Option<u16>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ModalPosition {
    Center,
    Top,
    Bottom,
    Left,
    Right,
    Custom { x: u16, y: u16 },
}

#[derive(Debug, Clone)]
pub struct ModalButton {
    pub label: String,
    pub action: ModalAction,
    pub is_default: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ModalAction {
    Ok,
    Cancel,
    Yes,
    No,
    Retry,
    Close,
    Custom(String),
}

impl Default for LayoutState {
    fn default() -> Self {
        Self {
            sidebar_width: 20,
            content_width: 50,
            detail_width: 30,
            terminal_size: Rect::default(),
            layout_mode: LayoutMode::Normal,
            panel_ratios: PanelRatios::default(),
            min_panel_sizes: MinPanelSizes::default(),
        }
    }
}

impl Default for PanelRatios {
    fn default() -> Self {
        Self {
            sidebar_ratio: 0.2,
            content_ratio: 0.5,
            detail_ratio: 0.3,
        }
    }
}

impl Default for MinPanelSizes {
    fn default() -> Self {
        Self {
            sidebar_min: 15,
            content_min: 30,
            detail_min: 20,
        }
    }
}

impl Default for FocusState {
    fn default() -> Self {
        Self {
            current_panel: FocusPanel::Sidebar,
            panel_history: Vec::new(),
            can_navigate: true,
            focus_ring: FocusRing::default(),
        }
    }
}

impl Default for FocusRing {
    fn default() -> Self {
        Self {
            panels: vec![FocusPanel::Sidebar, FocusPanel::Content, FocusPanel::Detail],
            current_index: 0,
            wrap_around: true,
        }
    }
}

impl LayoutState {
    pub fn new(terminal_size: Rect) -> Self {
        let mut state = Self {
            terminal_size,
            ..Self::default()
        };
        state.calculate_panel_sizes();
        state
    }

    pub fn update_terminal_size(&mut self, size: Rect) {
        self.terminal_size = size;
        self.calculate_panel_sizes();
    }

    pub fn calculate_panel_sizes(&mut self) {
        let total_width = self.terminal_size.width.saturating_sub(2); // borders

        match self.layout_mode {
            LayoutMode::Normal => {
                self.sidebar_width = (total_width as f32 * self.panel_ratios.sidebar_ratio) as u16;
                self.content_width = (total_width as f32 * self.panel_ratios.content_ratio) as u16;
                self.detail_width =
                    total_width.saturating_sub(self.sidebar_width + self.content_width);
            }
            LayoutMode::FullScreen(ref panel) => match panel {
                FocusPanel::Sidebar => {
                    self.sidebar_width = total_width;
                    self.content_width = 0;
                    self.detail_width = 0;
                }
                FocusPanel::Content => {
                    self.sidebar_width = 0;
                    self.content_width = total_width;
                    self.detail_width = 0;
                }
                FocusPanel::Detail => {
                    self.sidebar_width = 0;
                    self.content_width = 0;
                    self.detail_width = total_width;
                }
            },
            _ => {
                // 其他布局模式的计算
                self.calculate_panel_sizes();
            }
        }

        // 确保不小于最小尺寸
        self.sidebar_width = self.sidebar_width.max(self.min_panel_sizes.sidebar_min);
        self.content_width = self.content_width.max(self.min_panel_sizes.content_min);
        self.detail_width = self.detail_width.max(self.min_panel_sizes.detail_min);
    }

    pub fn set_layout_mode(&mut self, mode: LayoutMode) {
        self.layout_mode = mode;
        self.calculate_panel_sizes();
    }

    pub fn adjust_panel_ratios(&mut self, sidebar_delta: f32, content_delta: f32) {
        self.panel_ratios.sidebar_ratio =
            (self.panel_ratios.sidebar_ratio + sidebar_delta).clamp(0.1, 0.8);
        self.panel_ratios.content_ratio =
            (self.panel_ratios.content_ratio + content_delta).clamp(0.1, 0.8);
        self.panel_ratios.detail_ratio =
            1.0 - self.panel_ratios.sidebar_ratio - self.panel_ratios.content_ratio;
        self.panel_ratios.detail_ratio = self.panel_ratios.detail_ratio.max(0.1);
        self.calculate_panel_sizes();
    }

    pub fn can_fit_panels(&self) -> bool {
        let min_total = self.min_panel_sizes.sidebar_min
            + self.min_panel_sizes.content_min
            + self.min_panel_sizes.detail_min;
        self.terminal_size.width >= min_total + 2 // +2 for borders
    }
}

impl FocusRing {
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> FocusPanel {
        if self.panels.is_empty() {
            return FocusPanel::Sidebar;
        }

        if self.wrap_around {
            self.current_index = (self.current_index + 1) % self.panels.len();
        } else {
            self.current_index = (self.current_index + 1).min(self.panels.len() - 1);
        }

        self.panels[self.current_index]
    }

    pub fn previous(&mut self) -> FocusPanel {
        if self.panels.is_empty() {
            return FocusPanel::Sidebar;
        }

        if self.wrap_around {
            self.current_index = if self.current_index == 0 {
                self.panels.len() - 1
            } else {
                self.current_index - 1
            };
        } else {
            self.current_index = self.current_index.saturating_sub(1);
        }

        self.panels[self.current_index]
    }

    pub fn set_current(&mut self, panel: FocusPanel) {
        if let Some(index) = self.panels.iter().position(|p| *p == panel) {
            self.current_index = index;
        }
    }

    pub fn current(&self) -> FocusPanel {
        self.panels
            .get(self.current_index)
            .cloned()
            .unwrap_or(FocusPanel::Sidebar)
    }
}

impl ModalState {
    pub fn new_info(title: String, content: String) -> Self {
        Self {
            modal_type: ModalType::Info,
            title,
            content,
            size: ModalSize::default(),
            position: ModalPosition::Center,
            can_close: true,
            buttons: vec![ModalButton {
                label: "确定".to_string(),
                action: ModalAction::Ok,
                is_default: true,
            }],
        }
    }

    pub fn new_confirm(title: String, content: String) -> Self {
        Self {
            modal_type: ModalType::Confirm,
            title,
            content,
            size: ModalSize::default(),
            position: ModalPosition::Center,
            can_close: true,
            buttons: vec![
                ModalButton {
                    label: "是".to_string(),
                    action: ModalAction::Yes,
                    is_default: true,
                },
                ModalButton {
                    label: "否".to_string(),
                    action: ModalAction::No,
                    is_default: false,
                },
            ],
        }
    }

    pub fn new_error(title: String, content: String) -> Self {
        Self {
            modal_type: ModalType::Error,
            title,
            content,
            size: ModalSize::default(),
            position: ModalPosition::Center,
            can_close: true,
            buttons: vec![ModalButton {
                label: "确定".to_string(),
                action: ModalAction::Ok,
                is_default: true,
            }],
        }
    }
}

impl Default for ModalSize {
    fn default() -> Self {
        Self {
            width: 60,
            height: 20,
            max_width: Some(120),
            max_height: Some(40),
        }
    }
}
