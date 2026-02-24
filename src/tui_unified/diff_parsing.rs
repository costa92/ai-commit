use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
};

impl super::app::TuiUnifiedApp {
    /// 解析 diff 内容用于并排显示
    pub(super) fn parse_diff_for_side_by_side(
        &self,
        diff_content: &str,
    ) -> (
        Vec<ratatui::text::Line<'static>>,
        Vec<ratatui::text::Line<'static>>,
    ) {
        let mut left_lines = Vec::new();
        let mut right_lines = Vec::new();
        let mut old_line_num = 0u32;
        let mut new_line_num = 0u32;

        // 收集所有行并按块进行处理
        let lines: Vec<&str> = diff_content.lines().collect();
        let mut i = 0;
        let mut in_diff = false;

        while i < lines.len() {
            let line = lines[i];

            // 跳过 diff --git 之前的 commit metadata（Author, Date, message 等）
            if line.starts_with("diff --git") {
                in_diff = true;
            }
            if !in_diff {
                i += 1;
                continue;
            }

            if line.starts_with("@@") {
                // 解析行号信息：@@ -old_start,old_count +new_start,new_count @@ [optional context]
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 {
                    if let Some(old_part) = parts[1].strip_prefix('-') {
                        if let Some((start, _)) = old_part.split_once(',') {
                            old_line_num = start.parse().unwrap_or(0);
                        } else {
                            old_line_num = old_part.parse().unwrap_or(0);
                        }
                    }
                    if let Some(new_part) = parts[2].strip_prefix('+') {
                        if let Some((start, _)) = new_part.split_once(',') {
                            new_line_num = start.parse().unwrap_or(0);
                        } else {
                            new_line_num = new_part.parse().unwrap_or(0);
                        }
                    }
                }

                let header_line = Line::from(Span::styled(
                    line.to_string(),
                    Style::default().fg(Color::Cyan),
                ));
                left_lines.push(header_line.clone());
                right_lines.push(header_line);
                i += 1;
                continue;
            }

            if line.starts_with("diff --git")
                || line.starts_with("index")
                || line.starts_with("---")
                || line.starts_with("+++")
            {
                i += 1;
                continue;
            }

            if line.starts_with('-') {
                // 收集连续的删除行
                let mut removed_lines = Vec::new();
                while i < lines.len() && lines[i].starts_with('-') {
                    removed_lines.push(lines[i]);
                    i += 1;
                }

                // 收集后续的添加行
                let mut added_lines = Vec::new();
                while i < lines.len() && lines[i].starts_with('+') {
                    added_lines.push(lines[i]);
                    i += 1;
                }

                // 处理删除和添加行的对齐
                let max_lines = removed_lines.len().max(added_lines.len());

                for j in 0..max_lines {
                    if j < removed_lines.len() {
                        // 有删除行，在左侧显示
                        let line_content = removed_lines[j]
                            .strip_prefix('-')
                            .unwrap_or(removed_lines[j]);
                        let formatted_line =
                            format!("{:4} │ {}", old_line_num + j as u32, line_content);
                        left_lines.push(Line::from(Span::styled(
                            formatted_line.to_string(),
                            Style::default().fg(Color::Red),
                        )));
                    } else {
                        // 没有删除行，左侧显示空行
                        left_lines.push(Line::from(Span::styled(
                            "     │".to_string(),
                            Style::default().fg(Color::DarkGray),
                        )));
                    }

                    if j < added_lines.len() {
                        // 有添加行，在右侧显示
                        let line_content =
                            added_lines[j].strip_prefix('+').unwrap_or(added_lines[j]);
                        let formatted_line =
                            format!("{:4} │ {}", new_line_num + j as u32, line_content);
                        right_lines.push(Line::from(Span::styled(
                            formatted_line.to_string(),
                            Style::default().fg(Color::Green),
                        )));
                    } else {
                        // 没有添加行，右侧显示空行
                        right_lines.push(Line::from(Span::styled(
                            "     │".to_string(),
                            Style::default().fg(Color::DarkGray),
                        )));
                    }
                }

                old_line_num += removed_lines.len() as u32;
                new_line_num += added_lines.len() as u32;
            } else if let Some(line_content) = line.strip_prefix('+') {
                // 只有添加行（没有前面的删除行）
                let formatted_line = format!("{:4} │ {}", new_line_num, line_content);
                right_lines.push(Line::from(Span::styled(
                    formatted_line.to_string(),
                    Style::default().fg(Color::Green),
                )));

                // 左边显示空行
                left_lines.push(Line::from(Span::styled(
                    "     │".to_string(),
                    Style::default().fg(Color::DarkGray),
                )));

                new_line_num += 1;
                i += 1;
            } else if let Some(line_content) = line.strip_prefix(' ') {
                // 上下文行：两边都显示
                let left_formatted = format!("{:4} │ {}", old_line_num, line_content);
                let right_formatted = format!("{:4} │ {}", new_line_num, line_content);

                left_lines.push(Line::from(Span::styled(
                    left_formatted.to_string(),
                    Style::default().fg(Color::White),
                )));
                right_lines.push(Line::from(Span::styled(
                    right_formatted.to_string(),
                    Style::default().fg(Color::White),
                )));

                old_line_num += 1;
                new_line_num += 1;
                i += 1;
            } else if !line.is_empty() {
                // 其他内容行（如文件名等）：两边都显示
                let header_line = Line::from(Span::styled(
                    line.to_string(),
                    Style::default().fg(Color::Yellow),
                ));
                left_lines.push(header_line.clone());
                right_lines.push(header_line);
                i += 1;
            } else {
                i += 1;
            }
        }

        (left_lines, right_lines)
    }

    /// 解析 diff 内容用于分割显示
    pub(super) fn parse_diff_for_split(
        &self,
        diff_content: &str,
    ) -> (
        Vec<ratatui::text::Line<'static>>,
        Vec<ratatui::text::Line<'static>>,
    ) {
        let mut removed_lines = Vec::new();
        let mut added_lines = Vec::new();
        let mut old_line_num = 0u32;
        let mut new_line_num = 0u32;
        let mut in_diff = false;

        for line in diff_content.lines() {
            // 跳过 diff --git 之前的 commit metadata
            if line.starts_with("diff --git") {
                in_diff = true;
            }
            if !in_diff {
                continue;
            }

            if line.starts_with("@@") {
                // 解析行号信息：@@ -old_start,old_count +new_start,new_count @@ [optional context]
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 {
                    if let Some(old_part) = parts[1].strip_prefix('-') {
                        if let Some((start, _)) = old_part.split_once(',') {
                            old_line_num = start.parse().unwrap_or(0);
                        } else {
                            old_line_num = old_part.parse().unwrap_or(0);
                        }
                    }
                    if let Some(new_part) = parts[2].strip_prefix('+') {
                        if let Some((start, _)) = new_part.split_once(',') {
                            new_line_num = start.parse().unwrap_or(0);
                        } else {
                            new_line_num = new_part.parse().unwrap_or(0);
                        }
                    }
                }

                let header_line = Line::from(Span::styled(
                    line.to_string(),
                    Style::default().fg(Color::Cyan),
                ));
                removed_lines.push(header_line.clone());
                added_lines.push(header_line);
                continue;
            }

            if line.starts_with("diff --git")
                || line.starts_with("index")
                || line.starts_with("---")
                || line.starts_with("+++")
            {
                continue;
            }

            if let Some(line_content) = line.strip_prefix('-') {
                // 删除的行
                let formatted_line = format!("{:4} │ {}", old_line_num, line_content);
                removed_lines.push(Line::from(Span::styled(
                    formatted_line.to_string(),
                    Style::default().fg(Color::Red),
                )));
                old_line_num += 1;
            } else if let Some(line_content) = line.strip_prefix('+') {
                // 添加的行
                let formatted_line = format!("{:4} │ {}", new_line_num, line_content);
                added_lines.push(Line::from(Span::styled(
                    formatted_line.to_string(),
                    Style::default().fg(Color::Green),
                )));
                new_line_num += 1;
            } else if let Some(line_content) = line.strip_prefix(' ') {
                // 上下文行：两边都显示
                let old_formatted = format!("{:4} │ {}", old_line_num, line_content);
                let new_formatted = format!("{:4} │ {}", new_line_num, line_content);

                removed_lines.push(Line::from(Span::styled(
                    old_formatted.to_string(),
                    Style::default().fg(Color::White),
                )));
                added_lines.push(Line::from(Span::styled(
                    new_formatted.to_string(),
                    Style::default().fg(Color::White),
                )));

                old_line_num += 1;
                new_line_num += 1;
            } else if !line.is_empty() {
                // 其他内容行
                let header_line = Line::from(Span::styled(
                    line.to_string(),
                    Style::default().fg(Color::Yellow),
                ));
                removed_lines.push(header_line.clone());
                added_lines.push(header_line);
            }
        }

        (removed_lines, added_lines)
    }

    /// 解析 diff 内容用于统一显示
    pub(super) fn parse_diff_for_unified(
        &self,
        diff_content: &str,
    ) -> Vec<ratatui::text::Line<'static>> {
        let mut lines = Vec::new();
        let mut old_line_num = 0u32;
        let mut new_line_num = 0u32;
        let mut in_diff = false;

        for line in diff_content.lines() {
            // 跳过 diff --git 之前的 commit metadata
            if line.starts_with("diff --git") {
                in_diff = true;
            }
            if !in_diff {
                continue;
            }

            if line.starts_with("@@") {
                // 解析行号信息：@@ -old_start,old_count +new_start,new_count @@ [optional context]
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 {
                    if let Some(old_part) = parts[1].strip_prefix('-') {
                        if let Some((start, _)) = old_part.split_once(',') {
                            old_line_num = start.parse().unwrap_or(0);
                        } else {
                            old_line_num = old_part.parse().unwrap_or(0);
                        }
                    }
                    if let Some(new_part) = parts[2].strip_prefix('+') {
                        if let Some((start, _)) = new_part.split_once(',') {
                            new_line_num = start.parse().unwrap_or(0);
                        } else {
                            new_line_num = new_part.parse().unwrap_or(0);
                        }
                    }
                }
                lines.push(Line::from(Span::styled(
                    line.to_string(),
                    Style::default().fg(Color::Cyan),
                )));
                continue;
            }

            if line.starts_with("diff --git") {
                lines.push(Line::from(Span::styled(
                    line.to_string(),
                    Style::default().fg(Color::Yellow),
                )));
                continue;
            }

            if line.starts_with("index") || line.starts_with("---") || line.starts_with("+++") {
                continue;
            }

            if let Some(line_content) = line.strip_prefix('-') {
                // 删除的行
                let formatted_line = format!("{:4}   │ -{}", old_line_num, line_content);
                lines.push(Line::from(Span::styled(
                    formatted_line.to_string(),
                    Style::default().fg(Color::Red),
                )));
                old_line_num += 1;
            } else if let Some(line_content) = line.strip_prefix('+') {
                // 添加的行
                let formatted_line = format!("   {:4} │ +{}", new_line_num, line_content);
                lines.push(Line::from(Span::styled(
                    formatted_line.to_string(),
                    Style::default().fg(Color::Green),
                )));
                new_line_num += 1;
            } else if let Some(line_content) = line.strip_prefix(' ') {
                // 上下文行
                let formatted_line =
                    format!("{:4}:{:4} │  {}", old_line_num, new_line_num, line_content);
                lines.push(Line::from(Span::styled(
                    formatted_line.to_string(),
                    Style::default().fg(Color::White),
                )));
                old_line_num += 1;
                new_line_num += 1;
            } else if !line.is_empty() {
                // 其他内容行
                lines.push(Line::from(Span::styled(
                    line.to_string(),
                    Style::default().fg(Color::White),
                )));
            }
        }

        lines
    }
}
