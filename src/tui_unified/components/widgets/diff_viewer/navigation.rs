use super::types::{DiffDisplayMode, DiffLineType};
use super::DiffViewerComponent;

impl DiffViewerComponent {
    /// 滚动到指定位置
    pub(super) fn scroll_to(&mut self, position: usize) {
        self.scroll_position = position.min(self.diff_lines.len().saturating_sub(1));
    }

    /// 向上滚动
    pub(super) fn scroll_up(&mut self, lines: usize) {
        self.scroll_position = self.scroll_position.saturating_sub(lines);
        if let Some(ref mut selected) = self.selected_line {
            *selected = (*selected).saturating_sub(lines);
        }
    }

    /// 向下滚动
    pub(super) fn scroll_down(&mut self, lines: usize) {
        let max_scroll = self.diff_lines.len().saturating_sub(1);
        self.scroll_position = (self.scroll_position + lines).min(max_scroll);
        if let Some(ref mut selected) = self.selected_line {
            *selected = (*selected + lines).min(self.diff_lines.len().saturating_sub(1));
        }
    }

    /// 切换行号显示
    pub fn toggle_line_numbers(&mut self) {
        self.show_line_numbers = !self.show_line_numbers;
    }

    /// 切换换行
    pub fn toggle_wrap(&mut self) {
        self.wrap_lines = !self.wrap_lines;
    }

    /// 切换语法高亮
    pub fn toggle_syntax_highlight(&mut self) {
        self.syntax_highlight = !self.syntax_highlight;
    }

    /// 获取当前选中行
    pub fn selected_line(&self) -> Option<&super::types::DiffLine> {
        self.selected_line.and_then(|idx| self.diff_lines.get(idx))
    }

    /// 切换显示模式 (Ctrl+t 切换到文件树模式, s 切换并排模式)
    pub fn toggle_display_mode(&mut self, target_mode: Option<DiffDisplayMode>) {
        self.display_mode = match target_mode {
            Some(mode) => mode,
            None => match self.display_mode {
                DiffDisplayMode::Unified => DiffDisplayMode::FileTree,
                DiffDisplayMode::FileTree => DiffDisplayMode::SideBySide,
                DiffDisplayMode::SideBySide => DiffDisplayMode::Unified,
            },
        };

        // 重置选择状态以适应新的显示模式
        match self.display_mode {
            DiffDisplayMode::FileTree | DiffDisplayMode::SideBySide => {
                self.selected_file = if !self.diff_files.is_empty() {
                    Some(0)
                } else {
                    None
                };
                self.selected_line = None;
                self.file_list_state.select(self.selected_file);
            }
            DiffDisplayMode::Unified => {
                self.selected_line = if !self.diff_lines.is_empty() {
                    Some(0)
                } else {
                    None
                };
                self.selected_file = None;
                self.file_list_state.select(None);
            }
        }
        self.scroll_position = 0;
    }

    /// 切换单词级diff高亮
    pub fn toggle_word_level_diff(&mut self) {
        self.word_level_diff = !self.word_level_diff;
    }

    /// 同步文件选择状态（确保业务逻辑与ListState一致）
    pub(super) fn sync_file_selection(&mut self) {
        self.file_list_state.select(self.selected_file);
    }

    /// 导航：向上
    pub(super) fn navigate_up(&mut self) {
        match self.display_mode {
            DiffDisplayMode::FileTree => {
                if let Some(current) = self.selected_file {
                    if current > 0 {
                        self.selected_file = Some(current - 1);
                    } else if !self.diff_files.is_empty() {
                        self.selected_file = Some(self.diff_files.len() - 1);
                    }
                } else if !self.diff_files.is_empty() {
                    self.selected_file = Some(self.diff_files.len() - 1);
                }
                self.sync_file_selection();
            }
            DiffDisplayMode::SideBySide => {
                self.scroll_up(1);
            }
            _ => {
                self.scroll_up(1);
            }
        }
    }

    /// 导航：向下
    pub(super) fn navigate_down(&mut self) {
        match self.display_mode {
            DiffDisplayMode::FileTree => {
                if let Some(current) = self.selected_file {
                    if current < self.diff_files.len().saturating_sub(1) {
                        self.selected_file = Some(current + 1);
                    } else if !self.diff_files.is_empty() {
                        self.selected_file = Some(0);
                    }
                } else if !self.diff_files.is_empty() {
                    self.selected_file = Some(0);
                }
                self.sync_file_selection();
            }
            DiffDisplayMode::SideBySide => {
                self.scroll_down(1);
            }
            _ => {
                self.scroll_down(1);
            }
        }
    }

    /// 导航：向上翻页
    pub(super) fn navigate_page_up(&mut self) {
        match self.display_mode {
            DiffDisplayMode::FileTree => {
                if let Some(current) = self.selected_file {
                    self.selected_file = Some(current.saturating_sub(5));
                }
            }
            DiffDisplayMode::SideBySide | DiffDisplayMode::Unified => {
                self.scroll_up(10);
            }
        }
    }

    /// 导航：向下翻页
    pub(super) fn navigate_page_down(&mut self) {
        match self.display_mode {
            DiffDisplayMode::FileTree => {
                if let Some(current) = self.selected_file {
                    self.selected_file =
                        Some((current + 5).min(self.diff_files.len().saturating_sub(1)));
                }
            }
            DiffDisplayMode::SideBySide | DiffDisplayMode::Unified => {
                self.scroll_down(10);
            }
        }
    }

    /// 导航：跳到开头
    pub(super) fn navigate_home(&mut self) {
        match self.display_mode {
            DiffDisplayMode::FileTree => {
                self.selected_file = if !self.diff_files.is_empty() {
                    Some(0)
                } else {
                    None
                };
            }
            _ => {
                self.scroll_to(0);
                self.selected_line = if !self.diff_lines.is_empty() {
                    Some(0)
                } else {
                    None
                };
            }
        }
    }

    /// 导航：跳到结尾
    pub(super) fn navigate_end(&mut self) {
        match self.display_mode {
            DiffDisplayMode::FileTree => {
                self.selected_file = if !self.diff_files.is_empty() {
                    Some(self.diff_files.len().saturating_sub(1))
                } else {
                    None
                };
            }
            _ => {
                let last_pos = self.diff_lines.len().saturating_sub(1);
                self.scroll_to(last_pos);
                self.selected_line = if !self.diff_lines.is_empty() {
                    Some(last_pos)
                } else {
                    None
                };
            }
        }
    }

    /// 进入文件详情（从文件树模式）
    pub(super) fn enter_file_details(&mut self) {
        if let Some(file_index) = self.selected_file {
            if let Some(file) = self.diff_files.get(file_index) {
                if file.is_binary {
                    self.display_mode = DiffDisplayMode::Unified;
                    self.selected_line = None;
                    self.scroll_position = 0;
                } else {
                    self.display_mode = DiffDisplayMode::Unified;

                    let mut line_start = 0;
                    let mut current_file_index = 0;

                    for (i, line) in self.diff_lines.iter().enumerate() {
                        if line.line_type == DiffLineType::Header
                            && line.content.contains(&file.path)
                        {
                            if current_file_index == file_index {
                                line_start = i;
                                break;
                            }
                            current_file_index += 1;
                        }
                    }

                    self.selected_line = Some(line_start);
                    self.scroll_to(line_start);
                }
            }
        }
    }
}
