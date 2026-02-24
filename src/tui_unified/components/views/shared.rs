use ratatui::style::{Color, Style};

/// 默认选中样式函数 - 适用于不需要 item-specific 样式的视图
///
/// - selected + focused: 黑字黄底（高亮焦点）
/// - selected only: 白字深灰底（选中但无焦点）
/// - default: 默认样式
pub fn default_selection_style<T>(_item: &T, is_selected: bool, is_focused: bool) -> Style {
    if is_selected && is_focused {
        Style::default().fg(Color::Black).bg(Color::Yellow)
    } else if is_selected {
        Style::default().fg(Color::White).bg(Color::DarkGray)
    } else {
        Style::default()
    }
}
