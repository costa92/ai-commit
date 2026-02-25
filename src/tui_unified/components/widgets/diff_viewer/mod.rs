// Diff查看器组件 - 显示Git差异和语法高亮
mod navigation;
mod parsing;
mod rendering;
mod text_utils;
mod three_column;
mod types;
mod word_diff;

pub use types::{DiffDisplayMode, DiffFile, DiffLine, DiffLineType};

use crate::tui_unified::{
    components::base::{
        component::{Component, ViewComponent, ViewType},
        events::EventResult,
    },
    state::AppState,
};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{layout::Rect, widgets::ListState, Frame};
/// Diff查看器组件
pub struct DiffViewerComponent {
    pub(super) focused: bool,
    pub(super) diff_lines: Vec<DiffLine>,
    pub(super) diff_files: Vec<DiffFile>,
    pub(super) scroll_position: usize,
    pub(super) selected_line: Option<usize>,
    pub(super) selected_file: Option<usize>,
    pub(super) file_list_state: ListState,

    // 显示选项
    pub(super) display_mode: DiffDisplayMode,
    pub(super) show_line_numbers: bool,
    pub(super) wrap_lines: bool,
    pub(super) syntax_highlight: bool,
    pub(super) word_level_diff: bool,

    // 状态信息
    pub(super) current_file: Option<String>,
    pub(super) current_commit: Option<String>,
    pub(super) total_additions: u32,
    pub(super) total_deletions: u32,
}

impl DiffViewerComponent {
    pub fn new() -> Self {
        Self {
            focused: false,
            diff_lines: Vec::new(),
            diff_files: Vec::new(),
            scroll_position: 0,
            selected_line: None,
            selected_file: None,
            file_list_state: ListState::default(),

            // 显示选项
            display_mode: DiffDisplayMode::Unified,
            show_line_numbers: true,
            wrap_lines: false,
            syntax_highlight: true,
            word_level_diff: false,

            // 状态信息
            current_file: None,
            current_commit: None,
            total_additions: 0,
            total_deletions: 0,
        }
    }

    /// 设置diff内容
    pub fn set_diff(&mut self, diff_content: &str) {
        let (files, lines) = self.parse_enhanced_diff(diff_content);
        self.diff_files = files;
        self.diff_lines = lines;

        // 计算总的添加和删除行数
        self.total_additions = self.diff_files.iter().map(|f| f.additions).sum();
        self.total_deletions = self.diff_files.iter().map(|f| f.deletions).sum();

        self.scroll_position = 0;
        self.selected_line = if !self.diff_lines.is_empty() {
            Some(0)
        } else {
            None
        };
        self.selected_file = if !self.diff_files.is_empty() {
            Some(0)
        } else {
            None
        };

        // 同步更新file_list_state
        self.file_list_state.select(self.selected_file);
    }

    /// 设置当前文件和提交
    pub fn set_context(&mut self, file: Option<String>, commit: Option<String>) {
        self.current_file = file;
        self.current_commit = commit;
    }
}

impl Component for DiffViewerComponent {
    fn name(&self) -> &str {
        "DiffViewerComponent"
    }

    fn render(&mut self, frame: &mut Frame, area: Rect, _state: &AppState) {
        self.render_component(frame, area);
    }

    fn handle_key_event(&mut self, key: KeyEvent, _state: &mut AppState) -> EventResult {
        use crossterm::event::KeyModifiers;

        match (key.code, key.modifiers) {
            // 数字键1：统一diff模式
            (KeyCode::Char('1'), KeyModifiers::NONE) => {
                self.toggle_display_mode(Some(DiffDisplayMode::Unified));
                EventResult::Handled
            }
            // 数字键2：并排对比模式
            (KeyCode::Char('2'), KeyModifiers::NONE) => {
                self.toggle_display_mode(Some(DiffDisplayMode::SideBySide));
                EventResult::Handled
            }
            // 数字键3：文件树模式
            (KeyCode::Char('3'), KeyModifiers::NONE) => {
                self.toggle_display_mode(Some(DiffDisplayMode::FileTree));
                EventResult::Handled
            }
            // Ctrl+t 也可切换到文件树模式（保持向后兼容）
            (KeyCode::Char('t'), KeyModifiers::CONTROL) => {
                self.toggle_display_mode(Some(DiffDisplayMode::FileTree));
                EventResult::Handled
            }
            // w 键切换单词级diff高亮
            (KeyCode::Char('w'), KeyModifiers::NONE) => {
                self.toggle_word_level_diff();
                EventResult::Handled
            }
            // 基本导航
            (KeyCode::Up, _) | (KeyCode::Char('k'), KeyModifiers::NONE) => {
                self.navigate_up();
                EventResult::Handled
            }
            (KeyCode::Down, _) | (KeyCode::Char('j'), KeyModifiers::NONE) => {
                self.navigate_down();
                EventResult::Handled
            }
            (KeyCode::PageUp, _) | (KeyCode::Char('u'), KeyModifiers::NONE) => {
                self.navigate_page_up();
                EventResult::Handled
            }
            (KeyCode::PageDown, _) | (KeyCode::Char('d'), KeyModifiers::NONE) => {
                self.navigate_page_down();
                EventResult::Handled
            }
            (KeyCode::Home, _) | (KeyCode::Char('g'), KeyModifiers::NONE) => {
                self.navigate_home();
                EventResult::Handled
            }
            (KeyCode::End, _) | (KeyCode::Char('G'), KeyModifiers::SHIFT) => {
                self.navigate_end();
                EventResult::Handled
            }
            // 在Side-by-Side模式下，左右箭头键用于在文件之间切换
            (KeyCode::Left, _) => {
                if self.display_mode == DiffDisplayMode::SideBySide {
                    if let Some(current) = self.selected_file {
                        if current > 0 {
                            self.selected_file = Some(current - 1);
                        } else if !self.diff_files.is_empty() {
                            self.selected_file = Some(self.diff_files.len() - 1);
                        }
                        self.sync_file_selection();
                    }
                    EventResult::Handled
                } else {
                    EventResult::NotHandled
                }
            }
            (KeyCode::Right, _) => {
                if self.display_mode == DiffDisplayMode::SideBySide {
                    if let Some(current) = self.selected_file {
                        if current < self.diff_files.len().saturating_sub(1) {
                            self.selected_file = Some(current + 1);
                        } else if !self.diff_files.is_empty() {
                            self.selected_file = Some(0);
                        }
                        self.sync_file_selection();
                    }
                    EventResult::Handled
                } else {
                    EventResult::NotHandled
                }
            }
            // 其他快捷键
            (KeyCode::Char('n'), KeyModifiers::NONE) => {
                self.toggle_line_numbers();
                EventResult::Handled
            }
            (KeyCode::Char('h'), KeyModifiers::NONE) => {
                self.toggle_syntax_highlight();
                EventResult::Handled
            }
            // Enter键进入文件详情（在文件树模式下）
            (KeyCode::Enter, _) => {
                if self.display_mode == DiffDisplayMode::FileTree {
                    self.enter_file_details();
                }
                EventResult::Handled
            }
            // Backspace返回文件树模式
            (KeyCode::Backspace, _) => {
                if self.display_mode != DiffDisplayMode::FileTree {
                    self.toggle_display_mode(Some(DiffDisplayMode::FileTree));
                }
                EventResult::Handled
            }
            _ => EventResult::NotHandled,
        }
    }

    fn is_focused(&self) -> bool {
        self.focused
    }

    fn set_focus(&mut self, focused: bool) {
        self.focused = focused;
    }

    fn can_focus(&self) -> bool {
        true
    }

    fn min_size(&self) -> (u16, u16) {
        (60, 20)
    }
}

impl ViewComponent for DiffViewerComponent {
    fn view_type(&self) -> ViewType {
        ViewType::DiffViewer
    }

    fn title(&self) -> String {
        if let Some(ref file) = self.current_file {
            format!("Diff - {}", file)
        } else {
            "Diff Viewer".to_string()
        }
    }

    fn supports_search(&self) -> bool {
        true
    }

    fn search(&mut self, query: &str) -> EventResult {
        if query.is_empty() {
            return EventResult::Handled;
        }

        let query = query.to_lowercase();
        let start_pos = self.selected_line.unwrap_or(0);

        for (i, line) in self.diff_lines.iter().enumerate().skip(start_pos + 1) {
            if line.content.to_lowercase().contains(&query) {
                self.selected_line = Some(i);
                let visible_height = 20;
                if i < self.scroll_position || i >= self.scroll_position + visible_height {
                    self.scroll_position = i.saturating_sub(visible_height / 2);
                }
                return EventResult::Handled;
            }
        }

        // 如果没找到，从头开始搜索
        for (i, line) in self.diff_lines.iter().enumerate().take(start_pos) {
            if line.content.to_lowercase().contains(&query) {
                self.selected_line = Some(i);
                let visible_height = 20;
                if i < self.scroll_position || i >= self.scroll_position + visible_height {
                    self.scroll_position = i.saturating_sub(visible_height / 2);
                }
                return EventResult::Handled;
            }
        }

        EventResult::Handled
    }

    fn clear_search(&mut self) -> EventResult {
        EventResult::Handled
    }

    fn selected_index(&self) -> Option<usize> {
        self.selected_line
    }

    fn set_selected_index(&mut self, index: Option<usize>) {
        self.selected_line = index;
        if let Some(idx) = index {
            if idx < self.diff_lines.len() {
                let visible_height = 20;
                if idx < self.scroll_position || idx >= self.scroll_position + visible_height {
                    self.scroll_position = idx.saturating_sub(visible_height / 2);
                }
            }
        }
    }
}

impl Default for DiffViewerComponent {
    fn default() -> Self {
        Self::new()
    }
}
