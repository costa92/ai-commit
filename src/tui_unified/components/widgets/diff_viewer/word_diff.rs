use super::types::DiffLineType;
use super::DiffViewerComponent;
use ratatui::{
    style::{Color, Modifier, Style},
    text::Span,
};

impl DiffViewerComponent {
    /// 应用单词级差异高亮
    pub(super) fn apply_word_level_highlighting(
        &self,
        content: &str,
        line_type: &DiffLineType,
    ) -> Vec<Span<'static>> {
        if !self.word_level_diff {
            return vec![Span::raw(content.to_string())];
        }

        match line_type {
            DiffLineType::Added | DiffLineType::Removed => {
                self.highlight_word_differences(content, line_type)
            }
            _ => vec![Span::raw(content.to_string())],
        }
    }

    /// 高亮单词差异
    pub(super) fn highlight_word_differences(
        &self,
        content: &str,
        line_type: &DiffLineType,
    ) -> Vec<Span<'static>> {
        let mut spans = Vec::new();

        let clean_content = if let Some(s) = content
            .strip_prefix('+')
            .or_else(|| content.strip_prefix('-'))
        {
            s
        } else {
            content
        };

        let words = self.split_into_tokens(clean_content);

        for word in words.iter() {
            let base_style = match line_type {
                DiffLineType::Added => Style::default().fg(Color::Green),
                DiffLineType::Removed => Style::default().fg(Color::Red),
                _ => Style::default().fg(Color::White),
            };

            let word_style = if self.is_significant_change(word) {
                match line_type {
                    DiffLineType::Added => base_style.bg(Color::Green).add_modifier(Modifier::BOLD),
                    DiffLineType::Removed => base_style.bg(Color::Red).add_modifier(Modifier::BOLD),
                    _ => base_style,
                }
            } else {
                base_style
            };

            spans.push(Span::styled(word.clone(), word_style));
        }

        if spans.is_empty() {
            let style = match line_type {
                DiffLineType::Added => Style::default().fg(Color::Green),
                DiffLineType::Removed => Style::default().fg(Color::Red),
                _ => Style::default().fg(Color::White),
            };
            spans.push(Span::styled(content.to_string(), style));
        }

        spans
    }

    /// 将内容分割为token（单词、空格、标点符号）
    pub(super) fn split_into_tokens(&self, content: &str) -> Vec<String> {
        let mut tokens = Vec::new();
        let mut current_token = String::new();

        for ch in content.chars() {
            match ch {
                ' ' | '\t' => {
                    if !current_token.is_empty() {
                        tokens.push(current_token.clone());
                        current_token.clear();
                    }
                    tokens.push(ch.to_string());
                }
                '(' | ')' | '[' | ']' | '{' | '}' | '"' | '\'' | ',' | ';' | ':' | '.' => {
                    if !current_token.is_empty() {
                        tokens.push(current_token.clone());
                        current_token.clear();
                    }
                    tokens.push(ch.to_string());
                }
                _ => {
                    current_token.push(ch);
                }
            }
        }

        if !current_token.is_empty() {
            tokens.push(current_token);
        }

        tokens
    }

    /// 判断是否为显著变更（用于强化高亮）
    pub(super) fn is_significant_change(&self, token: &str) -> bool {
        let keywords = [
            "function",
            "fn",
            "def",
            "class",
            "struct",
            "enum",
            "impl",
            "trait",
            "let",
            "var",
            "const",
            "mut",
            "pub",
            "private",
            "public",
            "static",
            "if",
            "else",
            "match",
            "for",
            "while",
            "loop",
            "return",
            "break",
            "continue",
            "true",
            "false",
            "null",
            "undefined",
            "None",
            "Some",
        ];

        if token.parse::<f64>().is_ok()
            || (token.starts_with('"') && token.ends_with('"'))
            || (token.starts_with('\'') && token.ends_with('\''))
        {
            return true;
        }

        keywords
            .iter()
            .any(|&keyword| keyword == token.to_lowercase())
    }
}
