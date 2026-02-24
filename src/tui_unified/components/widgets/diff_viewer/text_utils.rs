use super::DiffViewerComponent;

impl DiffViewerComponent {
    /// 安全地截断字符串，确保不会破坏UTF-8字符边界
    pub(super) fn safe_truncate_path(path: &str, max_len: usize) -> String {
        if path.chars().count() <= max_len {
            path.to_string()
        } else {
            let chars: Vec<char> = path.chars().collect();
            if chars.len() > max_len {
                let suffix_len = max_len.saturating_sub(3);
                let start_index = chars.len().saturating_sub(suffix_len);
                let suffix: String = chars[start_index..].iter().collect();
                format!("...{}", suffix)
            } else {
                path.to_string()
            }
        }
    }

    /// 安全地截断内容，确保不会破坏UTF-8字符边界
    pub(super) fn safe_truncate_content(content: &str, max_len: usize) -> String {
        if content.chars().count() <= max_len {
            content.to_string()
        } else {
            let chars: Vec<char> = content.chars().collect();
            let truncated: String = chars[..max_len.saturating_sub(3)].iter().collect();
            format!("{}...", truncated)
        }
    }

    /// 截断内容到指定显示宽度（UTF-8字符边界安全）
    #[allow(dead_code)]
    pub(super) fn truncate_content(&self, content: &str, max_width: usize) -> String {
        if max_width == 0 {
            return String::new();
        }

        let mut display_width = 0;
        let mut char_end = 0;

        for (i, ch) in content.char_indices() {
            let char_width = match ch {
                '\u{4e00}'..='\u{9fff}' |
                '\u{3400}'..='\u{4dbf}' |
                '\u{20000}'..='\u{2a6df}' |
                '\u{2a700}'..='\u{2b73f}' |
                '\u{2b740}'..='\u{2b81f}' |
                '\u{2b820}'..='\u{2ceaf}' |
                '\u{2ceb0}'..='\u{2ebef}' |
                '\u{30000}'..='\u{3134f}' |
                '\u{ac00}'..='\u{d7af}' |
                '\u{3040}'..='\u{309f}' |
                '\u{30a0}'..='\u{30ff}' |
                '\u{ff01}'..='\u{ff60}' |
                '\u{ffe0}'..='\u{ffe6}'
                => 2,
                '\t' => 4,
                _ => 1,
            };

            if display_width + char_width > max_width {
                break;
            }

            display_width += char_width;
            char_end = i + ch.len_utf8();
        }

        if char_end >= content.len() {
            content.to_string()
        } else if display_width < max_width {
            format!("{}…", &content[..char_end])
        } else if char_end > 0 {
            let mut prev_end = 0;
            for (i, _) in content.char_indices() {
                if i >= char_end {
                    break;
                }
                prev_end = i;
            }
            if prev_end > 0 {
                format!("{}…", &content[..prev_end])
            } else {
                "…".to_string()
            }
        } else {
            "…".to_string()
        }
    }

    /// 格式化并排显示内容
    #[allow(dead_code)]
    pub(super) fn format_side_content(
        &self,
        content: &str,
        line_no: Option<u32>,
        max_width: usize,
        _is_old: bool,
    ) -> String {
        if max_width < 10 {
            return String::new();
        }

        let prefix = if let Some(no) = line_no {
            if self.show_line_numbers {
                format!("{:4} ", no)
            } else {
                String::new()
            }
        } else if self.show_line_numbers {
            "     ".to_string()
        } else {
            String::new()
        };

        let clean_content = if let Some(s) = content
            .strip_prefix('+')
            .or_else(|| content.strip_prefix('-'))
            .or_else(|| content.strip_prefix(' '))
        {
            s
        } else {
            content
        };

        let available_width = max_width.saturating_sub(prefix.len());
        let truncated_content = self.truncate_content(clean_content, available_width);
        let padded_content = format!(
            "{}{}",
            truncated_content,
            " ".repeat(available_width.saturating_sub(truncated_content.len()))
        );

        format!("{}{}", prefix, padded_content)
    }

    /// 获取文件扩展名
    pub(super) fn get_file_extension(&self, path: &str) -> Option<String> {
        path.split('.').next_back().map(|s| s.to_string())
    }
}
