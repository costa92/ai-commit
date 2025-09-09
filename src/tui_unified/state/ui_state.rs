// UI状态占位符
use crate::tui_unified::focus::FocusPanel;

pub struct LayoutState {
    pub sidebar_width: u16,
    pub content_width: u16,
    pub detail_width: u16,
}

pub struct FocusState {
    pub current_panel: FocusPanel,
    pub panel_history: Vec<FocusPanel>,
    pub can_navigate: bool,
}

pub struct ModalState {
    // TODO: 实现模态框状态
}