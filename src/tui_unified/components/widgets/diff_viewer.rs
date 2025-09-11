// Diff查看器组件 - 显示Git差异和语法高亮
use crossterm::event::{KeyEvent, KeyCode};
use ratatui::{Frame, layout::Rect, widgets::{Block, Borders, List, ListItem, ListState, Scrollbar, ScrollbarOrientation, ScrollbarState}, text::{Line, Span, Text}, style::{Color, Style, Modifier}};
use crate::tui_unified::{
    state::AppState,
    components::base::{
        component::{Component, ViewComponent, ViewType},
        events::EventResult
    }
};

/// Diff行类型
#[derive(Debug, Clone, PartialEq)]
pub enum DiffLineType {
    Context,    // 上下文行
    Added,      // 添加的行
    Removed,    // 删除的行
    Header,     // 文件头
    Hunk,       // 代码块头
    FileTree,   // 文件树结构
    Binary,     // 二进制文件
}

/// Diff显示模式
#[derive(Debug, Clone, PartialEq)]
pub enum DiffDisplayMode {
    Unified,    // 统一diff模式（默认）
    SideBySide, // 并排对比模式
    FileTree,   // 文件树形diff
}

/// 文件信息
#[derive(Debug, Clone)]
pub struct DiffFile {
    pub path: String,
    pub old_path: Option<String>,
    pub is_binary: bool,
    pub is_image: bool,
    pub additions: u32,
    pub deletions: u32,
    pub lines: Vec<DiffLine>,
}

/// Diff行数据
#[derive(Debug, Clone)]
pub struct DiffLine {
    pub line_type: DiffLineType,
    pub content: String,
    pub old_line_no: Option<u32>,
    pub new_line_no: Option<u32>,
}

/// 文件树节点类型
#[derive(Debug, Clone)]
enum FileTreeNode {
    Directory(std::collections::BTreeMap<String, FileTreeNode>),
    File(usize), // 文件索引
}

/// 根据文件扩展名获取图标
fn get_file_icon(path: &str) -> Option<&'static str> {
    let extension = path.split('.').last()?.to_lowercase();
    match extension.as_str() {
        "rs" => Some("🦀 "),
        "py" => Some("🐍 "),
        "js" | "ts" => Some("⚡ "),
        "html" | "htm" => Some("🌐 "),
        "css" | "scss" | "sass" => Some("🎨 "),
        "json" => Some("📋 "),
        "xml" => Some("📰 "),
        "md" | "markdown" => Some("📝 "),
        "txt" => Some("📄 "),
        "toml" | "yaml" | "yml" => Some("⚙️ "),
        "sh" | "bash" => Some("🐚 "),
        "dockerfile" => Some("🐳 "),
        "go" => Some("🔷 "),
        "java" | "class" => Some("☕ "),
        "cpp" | "cc" | "cxx" | "c" | "h" | "hpp" => Some("⚡ "),
        "rb" => Some("💎 "),
        "php" => Some("🐘 "),
        "sql" => Some("🗄️ "),
        _ => None,
    }
}

/// Diff查看器组件
pub struct DiffViewerComponent {
    focused: bool,
    diff_lines: Vec<DiffLine>,
    diff_files: Vec<DiffFile>,
    scroll_position: usize,
    selected_line: Option<usize>,
    selected_file: Option<usize>,
    file_list_state: ListState, // 文件列表状态
    
    // 显示选项
    display_mode: DiffDisplayMode,
    show_line_numbers: bool,
    wrap_lines: bool,
    syntax_highlight: bool,
    word_level_diff: bool,
    
    // 状态信息
    current_file: Option<String>,
    current_commit: Option<String>,
    total_additions: u32,
    total_deletions: u32,
}

impl DiffViewerComponent {
    /// 安全地截断字符串，确保不会破坏UTF-8字符边界
    fn safe_truncate_path(path: &str, max_len: usize) -> String {
        if path.chars().count() <= max_len {
            path.to_string()
        } else {
            // 使用字符计数而不是字节长度来安全截断
            let chars: Vec<char> = path.chars().collect();
            if chars.len() > max_len {
                let suffix_len = max_len.saturating_sub(3); // 为"..."留出空间
                let start_index = chars.len().saturating_sub(suffix_len);
                let suffix: String = chars[start_index..].iter().collect();
                format!("...{}", suffix)
            } else {
                path.to_string()
            }
        }
    }

    /// 安全地截断内容，确保不会破坏UTF-8字符边界
    fn safe_truncate_content(content: &str, max_len: usize) -> String {
        if content.chars().count() <= max_len {
            content.to_string()
        } else {
            let chars: Vec<char> = content.chars().collect();
            let truncated: String = chars[..max_len.saturating_sub(3)].iter().collect();
            format!("{}...", truncated)
        }
    }

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
        self.selected_line = if !self.diff_lines.is_empty() { Some(0) } else { None };
        self.selected_file = if !self.diff_files.is_empty() { Some(0) } else { None };
        
        // 同步更新file_list_state
        self.file_list_state.select(self.selected_file);
    }

    /// 设置当前文件和提交
    pub fn set_context(&mut self, file: Option<String>, commit: Option<String>) {
        self.current_file = file;
        self.current_commit = commit;
    }

    /// 增强的diff解析（支持多文件和元数据）
    fn parse_enhanced_diff(&self, content: &str) -> (Vec<DiffFile>, Vec<DiffLine>) {
        let mut files = Vec::new();
        let mut all_lines = Vec::new();
        let mut current_file: Option<DiffFile> = None;
        let mut current_lines = Vec::new();
        
        let lines: Vec<&str> = content.lines().collect();
        let mut i = 0;
        
        while i < lines.len() {
            let line = lines[i];
            
            if line.starts_with("diff --git") {
                // 保存之前的文件
                if let Some(mut file) = current_file.take() {
                    file.lines = current_lines;
                    files.push(file);
                    current_lines = Vec::new();
                }
                
                // 解析文件路径
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 4 {
                    let path = parts[3].trim_start_matches("b/");
                    let is_image = self.is_image_file(path);
                    let is_binary = self.is_likely_binary_file(path);
                    current_file = Some(DiffFile {
                        path: path.to_string(),
                        old_path: None,
                        is_binary,
                        is_image,
                        additions: 0,
                        deletions: 0,
                        lines: Vec::new(),
                    });
                }
            } else if line.starts_with("Binary files") {
                if let Some(ref mut file) = current_file {
                    file.is_binary = true;
                    current_lines.push(DiffLine {
                        line_type: DiffLineType::Binary,
                        content: line.to_string(),
                        old_line_no: None,
                        new_line_no: None,
                    });
                }
            } else {
                // 使用原有的解析逻辑处理其他行
                let parsed_line = self.parse_single_line(line, &mut 0, &mut 0);
                
                if let Some(ref mut file) = current_file {
                    match parsed_line.line_type {
                        DiffLineType::Added => file.additions += 1,
                        DiffLineType::Removed => file.deletions += 1,
                        _ => {}
                    }
                }
                
                current_lines.push(parsed_line.clone());
                all_lines.push(parsed_line);
            }
            
            i += 1;
        }
        
        // 保存最后一个文件
        if let Some(mut file) = current_file {
            file.lines = current_lines;
            files.push(file);
        }
        
        (files, all_lines)
    }
    
    /// 解析单行diff内容
    fn parse_single_line(&self, line: &str, old_line_no: &mut u32, new_line_no: &mut u32) -> DiffLine {
        let (line_type, old_no, new_no) = if line.starts_with("@@") {
            // 解析hunk头: @@ -old_start,old_count +new_start,new_count @@
            if let Some(captures) = self.parse_hunk_header(line) {
                *old_line_no = captures.0;
                *new_line_no = captures.1;
            }
            (DiffLineType::Hunk, None, None)
        } else if line.starts_with("+++") || line.starts_with("---") {
            (DiffLineType::Header, None, None)
        } else if line.starts_with("+") {
            *new_line_no += 1;
            (DiffLineType::Added, None, Some(*new_line_no))
        } else if line.starts_with("-") {
            *old_line_no += 1;
            (DiffLineType::Removed, Some(*old_line_no), None)
        } else if line.starts_with(" ") || line.is_empty() {
            *old_line_no += 1;
            *new_line_no += 1;
            (DiffLineType::Context, Some(*old_line_no), Some(*new_line_no))
        } else if line.starts_with("\\") && line.contains("No newline at end of file") {
            // 处理 "\ No newline at end of file" 标记
            (DiffLineType::Context, None, None)
        } else {
            (DiffLineType::Header, None, None)
        };

        DiffLine {
            line_type,
            content: line.to_string(),
            old_line_no: old_no,
            new_line_no: new_no,
        }
    }
    
    /// 检查是否为图片文件
    fn is_image_file(&self, path: &str) -> bool {
        let image_extensions = [
            ".png", ".jpg", ".jpeg", ".gif", ".bmp", ".svg", ".webp", 
            ".tiff", ".tif", ".ico", ".avif", ".heic", ".heif"
        ];
        let lower_path = path.to_lowercase();
        image_extensions.iter().any(|ext| lower_path.ends_with(ext))
    }

    /// 检查是否为二进制文件类型
    fn is_likely_binary_file(&self, path: &str) -> bool {
        let binary_extensions = [
            // 可执行文件
            ".exe", ".dll", ".so", ".dylib", ".a", ".lib", ".bin",
            // 压缩文件
            ".zip", ".tar", ".gz", ".bz2", ".xz", ".7z", ".rar",
            // 媒体文件
            ".mp3", ".mp4", ".avi", ".mkv", ".wav", ".flac", ".ogg",
            // 办公文档
            ".pdf", ".doc", ".docx", ".xls", ".xlsx", ".ppt", ".pptx",
            // 数据库
            ".db", ".sqlite", ".sqlite3", ".mdb",
            // 其他二进制格式
            ".pyc", ".class", ".jar", ".dex", ".apk"
        ];
        let lower_path = path.to_lowercase();
        binary_extensions.iter().any(|ext| lower_path.ends_with(ext)) || self.is_image_file(path)
    }
    
    /// 原有的parse_diff方法，保持向后兼容
    fn parse_diff(&self, content: &str) -> Vec<DiffLine> {
        let mut lines = Vec::new();
        let mut old_line_no = 0u32;
        let mut new_line_no = 0u32;

        for line in content.lines() {
            let parsed_line = self.parse_single_line(line, &mut old_line_no, &mut new_line_no);
            lines.push(parsed_line);
        }

        lines
    }

    /// 解析hunk头部信息
    fn parse_hunk_header(&self, line: &str) -> Option<(u32, u32)> {
        // 简单的hunk头解析：@@ -old_start,old_count +new_start,new_count @@
        if let Some(start) = line.find("-") {
            if let Some(end) = line.find(" +") {
                let old_part = &line[start+1..end];
                if let Some(comma) = old_part.find(",") {
                    if let Ok(old_start) = old_part[..comma].parse::<u32>() {
                        let new_start = line[end+2..].split(',').next()?.parse::<u32>().ok()?;
                        return Some((old_start, new_start));
                    }
                }
            }
        }
        None
    }

    /// 获取diff行的样式
    fn get_line_style(&self, line: &DiffLine, is_selected: bool) -> Style {
        let base_style = match line.line_type {
            DiffLineType::Added => Style::default().fg(Color::Green),
            DiffLineType::Removed => Style::default().fg(Color::Red),
            DiffLineType::Header => Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            DiffLineType::Hunk => Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD),
            DiffLineType::Context => {
                // 特殊处理 "No newline at end of file" 行
                if line.content.starts_with("\\") && line.content.contains("No newline at end of file") {
                    Style::default().fg(Color::Gray).add_modifier(Modifier::ITALIC)
                } else {
                    Style::default().fg(Color::White)
                }
            },
            DiffLineType::FileTree => Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            DiffLineType::Binary => Style::default().fg(Color::Magenta).add_modifier(Modifier::ITALIC),
        };

        if is_selected && self.focused {
            base_style.bg(Color::DarkGray)
        } else {
            base_style
        }
    }

    /// 格式化显示行
    fn format_line(&self, line: &DiffLine) -> String {
        // 特殊处理 "No newline at end of file" 行
        if line.content.starts_with("\\") && line.content.contains("No newline at end of file") {
            return "⚠ No newline at end of file".to_string();
        }
        
        if self.show_line_numbers {
            let old_no = line.old_line_no.map_or("    ".to_string(), |n| format!("{:4}", n));
            let new_no = line.new_line_no.map_or("    ".to_string(), |n| format!("{:4}", n));
            format!("{} {} | {}", old_no, new_no, line.content)
        } else {
            line.content.clone()
        }
    }

    /// 滚动到指定位置
    fn scroll_to(&mut self, position: usize) {
        self.scroll_position = position.min(self.diff_lines.len().saturating_sub(1));
    }

    /// 向上滚动
    fn scroll_up(&mut self, lines: usize) {
        self.scroll_position = self.scroll_position.saturating_sub(lines);
        if let Some(ref mut selected) = self.selected_line {
            *selected = (*selected).saturating_sub(lines);
        }
    }

    /// 向下滚动  
    fn scroll_down(&mut self, lines: usize) {
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
    pub fn selected_line(&self) -> Option<&DiffLine> {
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
            }
        };
        
        // 重置选择状态以适应新的显示模式
        match self.display_mode {
            DiffDisplayMode::FileTree | DiffDisplayMode::SideBySide => {
                // 文件树模式和并排模式都需要选择文件
                self.selected_file = if !self.diff_files.is_empty() { Some(0) } else { None };
                self.selected_line = None;
                // 同步更新file_list_state
                self.file_list_state.select(self.selected_file);
            }
            DiffDisplayMode::Unified => {
                // 统一模式选择行
                self.selected_line = if !self.diff_lines.is_empty() { Some(0) } else { None };
                self.selected_file = None;
                // 清除file_list_state选择
                self.file_list_state.select(None);
            }
        }
        self.scroll_position = 0;
    }

    /// 切换单词级diff高亮
    pub fn toggle_word_level_diff(&mut self) {
        self.word_level_diff = !self.word_level_diff;
    }

    /// 应用单词级差异高亮
    fn apply_word_level_highlighting(&self, content: &str, line_type: &DiffLineType) -> Vec<Span<'static>> {
        if !self.word_level_diff {
            return vec![Span::raw(content.to_string())];
        }

        match line_type {
            DiffLineType::Added | DiffLineType::Removed => {
                // 对于添加和删除的行，尝试进行单词级高亮
                self.highlight_word_differences(content, line_type)
            }
            _ => vec![Span::raw(content.to_string())],
        }
    }

    /// 高亮单词差异
    fn highlight_word_differences(&self, content: &str, line_type: &DiffLineType) -> Vec<Span<'static>> {
        let mut spans = Vec::new();
        
        // 移除行首的+/-标记
        let clean_content = if content.starts_with('+') || content.starts_with('-') {
            &content[1..]
        } else {
            content
        };

        // 按单词分割
        let words = self.split_into_tokens(clean_content);
        
        for (_i, word) in words.iter().enumerate() {
            let base_style = match line_type {
                DiffLineType::Added => Style::default().fg(Color::Green),
                DiffLineType::Removed => Style::default().fg(Color::Red),
                _ => Style::default().fg(Color::White),
            };

            // 在单词级模式下，某些单词可以有更强的高亮
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

        // 如果没有单词，返回原始内容
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
    fn split_into_tokens(&self, content: &str) -> Vec<String> {
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
    fn is_significant_change(&self, token: &str) -> bool {
        // 关键字或重要标识符
        let keywords = [
            "function", "fn", "def", "class", "struct", "enum", "impl", "trait",
            "let", "var", "const", "mut", "pub", "private", "public", "static",
            "if", "else", "match", "for", "while", "loop", "return", "break",
            "continue", "true", "false", "null", "undefined", "None", "Some"
        ];

        // 数字或字符串字面量
        if token.parse::<f64>().is_ok() || 
           (token.starts_with('"') && token.ends_with('"')) ||
           (token.starts_with('\'') && token.ends_with('\'')) {
            return true;
        }

        // 关键字
        keywords.iter().any(|&keyword| keyword == token.to_lowercase())
    }

    /// 生成图片/二进制文件对比信息
    fn generate_binary_comparison_view(&self, file: &DiffFile) -> Vec<ListItem> {
        let mut items = Vec::new();
        
        // 文件标题
        items.push(ListItem::new(Line::from(vec![
            Span::styled("📦 ", Style::default().fg(Color::Magenta)),
            Span::styled(format!("Binary File: {}", file.path), 
                        Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        ])));
        
        items.push(ListItem::new(Line::from(Span::raw("")))); // 空行
        
        if file.is_image {
            // 图片文件特殊处理
            items.push(ListItem::new(Line::from(vec![
                Span::styled("🖼️  ", Style::default().fg(Color::Yellow)),
                Span::styled("Image File Detected", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            ])));
            
            items.push(ListItem::new(Line::from(vec![
                Span::styled("   Type: ", Style::default().fg(Color::Gray)),
                Span::styled(self.get_file_extension(&file.path).unwrap_or_else(|| "Unknown".to_string()), Style::default().fg(Color::White)),
            ])));
            
            items.push(ListItem::new(Line::from(Span::raw(""))));
            
            // 图片文件的metadata显示
            items.push(ListItem::new(Line::from(vec![
                Span::styled("   📏 ", Style::default().fg(Color::Blue)),
                Span::styled("Image comparison not available in terminal", Style::default().fg(Color::Gray)),
            ])));
            
            items.push(ListItem::new(Line::from(vec![
                Span::styled("   💡 ", Style::default().fg(Color::Yellow)),
                Span::styled("Tip: Use external image diff tools for visual comparison", Style::default().fg(Color::Gray)),
            ])));
            
        } else {
            // 普通二进制文件
            items.push(ListItem::new(Line::from(vec![
                Span::styled("📦  ", Style::default().fg(Color::Magenta)),
                Span::styled("Binary File", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
            ])));
            
            items.push(ListItem::new(Line::from(vec![
                Span::styled("   Extension: ", Style::default().fg(Color::Gray)),
                Span::styled(self.get_file_extension(&file.path).unwrap_or_else(|| "None".to_string()), Style::default().fg(Color::White)),
            ])));
        }
        
        items.push(ListItem::new(Line::from(Span::raw(""))));
        
        // 变更统计
        if file.additions > 0 || file.deletions > 0 {
            items.push(ListItem::new(Line::from(vec![
                Span::styled("📊 Changes:", Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD)),
            ])));
            
            if file.additions > 0 {
                items.push(ListItem::new(Line::from(vec![
                    Span::styled("   +", Style::default().fg(Color::Green)),
                    Span::styled(format!("{} additions", file.additions), Style::default().fg(Color::Green)),
                ])));
            }
            
            if file.deletions > 0 {
                items.push(ListItem::new(Line::from(vec![
                    Span::styled("   -", Style::default().fg(Color::Red)),
                    Span::styled(format!("{} deletions", file.deletions), Style::default().fg(Color::Red)),
                ])));
            }
        } else {
            items.push(ListItem::new(Line::from(vec![
                Span::styled("ℹ️  ", Style::default().fg(Color::Blue)),
                Span::styled("File modified (binary diff cannot be displayed)", Style::default().fg(Color::Gray)),
            ])));
        }
        
        items.push(ListItem::new(Line::from(Span::raw(""))));
        
        // 操作提示
        items.push(ListItem::new(Line::from(vec![
            Span::styled("⌨️  Controls:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ])));
        
        items.push(ListItem::new(Line::from(vec![
            Span::styled("   • ", Style::default().fg(Color::Gray)),
            Span::styled("ESC", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::styled(" - Return to file tree", Style::default().fg(Color::Gray)),
        ])));
        
        items.push(ListItem::new(Line::from(vec![
            Span::styled("   • ", Style::default().fg(Color::Gray)),
            Span::styled("1/2/3", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::styled(" - Switch view modes", Style::default().fg(Color::Gray)),
        ])));
        
        items
    }

    /// 获取文件扩展名
    fn get_file_extension(&self, path: &str) -> Option<String> {
        path.split('.').last().map(|s| s.to_string())
    }

    /// 生成文件树显示内容
    fn generate_file_tree_view(&self) -> Vec<ListItem> {
        let mut items = Vec::new();
        
        // 添加概览信息
        items.push(ListItem::new(Line::from(vec![
            Span::styled("📊 ", Style::default().fg(Color::Blue)),
            Span::styled(format!("Diff Summary: {} files", self.diff_files.len()), 
                        Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        ])));
        
        items.push(ListItem::new(Line::from(vec![
            Span::styled("  +", Style::default().fg(Color::Green)),
            Span::styled(format!("{} additions", self.total_additions), Style::default().fg(Color::Green)),
            Span::styled("  -", Style::default().fg(Color::Red)),
            Span::styled(format!("{} deletions", self.total_deletions), Style::default().fg(Color::Red)),
        ])));
        
        items.push(ListItem::new(Line::from(Span::raw("")))); // 空行分隔
        
        // 按目录结构组织文件
        let mut file_tree = std::collections::BTreeMap::new();
        for (i, file) in self.diff_files.iter().enumerate() {
            let path_parts: Vec<&str> = file.path.split('/').collect();
            
            // 构建目录树结构
            self.insert_file_into_tree(&mut file_tree, &path_parts, i);
        }
        
        // 递归渲染文件树
        self.render_tree_node(&file_tree, 0, &mut items);
        
        items
    }
    
    /// 递归渲染文件树节点
    fn render_tree_node(&self, 
                       tree: &std::collections::BTreeMap<String, FileTreeNode>, 
                       depth: usize, 
                       items: &mut Vec<ListItem>) {
        for (name, node) in tree {
            let indent = "  ".repeat(depth);
            
            match node {
                FileTreeNode::Directory(subtree) => {
                    // 渲染目录
                    let icon = if subtree.is_empty() { "📁 " } else { "📂 " };
                    items.push(ListItem::new(Line::from(vec![
                        Span::raw(indent),
                        Span::styled(icon, Style::default().fg(Color::Blue)),
                        Span::styled(name.clone(), 
                                   Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD)),
                    ])));
                    
                    // 递归渲染子目录
                    self.render_tree_node(subtree, depth + 1, items);
                }
                FileTreeNode::File(file_index) => {
                    if let Some(file) = self.diff_files.get(*file_index) {
                        // 选择文件图标
                        let icon = if file.is_binary {
                            if file.is_image { "🖼️ " } else { "📦 " }
                        } else {
                            match get_file_icon(&file.path) {
                                Some(icon) => icon,
                                None => "📄 ",
                            }
                        };
                        
                        // 文件状态颜色
                        let (status_color, status_text) = if file.is_binary {
                            (Color::Magenta, " (binary)".to_string())
                        } else {
                            match (file.additions, file.deletions) {
                                (0, 0) => (Color::Gray, "".to_string()),
                                (a, 0) => (Color::Green, format!(" (+{})", a)),
                                (0, d) => (Color::Red, format!(" (-{})", d)),
                                (a, d) => (Color::Yellow, format!(" (+{}, -{})", a, d)),
                            }
                        };
                        
                        let is_selected = self.selected_file == Some(*file_index);
                        let file_style = if is_selected && self.focused {
                            Style::default().fg(Color::White).bg(Color::DarkGray)
                        } else {
                            Style::default().fg(Color::White)
                        };
                        
                        items.push(ListItem::new(Line::from(vec![
                            Span::raw(indent),
                            Span::styled(icon, Style::default().fg(Color::Yellow)),
                            Span::styled(name.clone(), file_style),
                            Span::styled(status_text, Style::default().fg(status_color)),
                        ])));
                    }
                }
            }
        }
    }

    /// 将文件插入目录树
    fn insert_file_into_tree(&self, 
                            tree: &mut std::collections::BTreeMap<String, FileTreeNode>, 
                            path_parts: &[&str], 
                            file_index: usize) {
        if path_parts.is_empty() {
            return;
        }
        
        let part = path_parts[0].to_string();
        let is_file = path_parts.len() == 1;
        
        if is_file {
            tree.insert(part, FileTreeNode::File(file_index));
        } else {
            let entry = tree.entry(part)
                .or_insert_with(|| FileTreeNode::Directory(std::collections::BTreeMap::new()));
            
            if let FileTreeNode::Directory(ref mut subtree) = entry {
                self.insert_file_into_tree(subtree, &path_parts[1..], file_index);
            }
        }
    }

    /// 生成统一diff视图
    fn generate_unified_view(&self, visible_height: usize) -> Vec<ListItem> {
        // 检查是否正在显示二进制文件详情
        if let Some(file_index) = self.selected_file {
            if let Some(file) = self.diff_files.get(file_index) {
                if file.is_binary && self.display_mode == DiffDisplayMode::Unified {
                    // 显示二进制文件对比视图
                    return self.generate_binary_comparison_view(file);
                }
            }
        }

        // 常规的文本diff视图
        self.diff_lines
            .iter()
            .skip(self.scroll_position)
            .take(visible_height)
            .enumerate()
            .map(|(i, line)| {
                let is_selected = self.selected_line == Some(self.scroll_position + i);
                
                if self.word_level_diff {
                    // 使用单词级高亮
                    let line_content = self.format_line(line);
                    let spans = self.apply_word_level_highlighting(&line_content, &line.line_type);
                    
                    // 为选中行添加背景色
                    let final_spans = if is_selected && self.focused {
                        spans.into_iter().map(|span| {
                            let mut new_style = span.style;
                            new_style.bg = Some(Color::DarkGray);
                            Span::styled(span.content, new_style)
                        }).collect()
                    } else {
                        spans
                    };
                    
                    ListItem::new(Line::from(final_spans))
                } else {
                    // 使用传统行级高亮
                    let style = self.get_line_style(line, is_selected);
                    let content = self.format_line(line);
                    ListItem::new(Line::from(Span::styled(content, style)))
                }
            })
            .collect()
    }

    /// 生成并排对比视图
    fn generate_side_by_side_view(&self, area_width: u16, visible_height: usize) -> Vec<ListItem> {
        let mut result = Vec::new();
        let half_width = (area_width.saturating_sub(4)) / 2; // 减去边框和分隔符
        
        // 存储左右两侧的行数据 (暂时保留，未来可能用于更复杂的配对逻辑)
        let mut _left_lines: Vec<String> = Vec::new();
        let mut _right_lines: Vec<String> = Vec::new();
        
        // 处理可见范围的diff行
        let visible_lines: Vec<&DiffLine> = self.diff_lines
            .iter()
            .skip(self.scroll_position)
            .take(visible_height)
            .collect();
        
        // 按行配对处理
        for (i, line) in visible_lines.iter().enumerate() {
            let is_selected = self.selected_line == Some(self.scroll_position + i);
            
            match line.line_type {
                DiffLineType::Header | DiffLineType::Hunk => {
                    // 头部信息跨越两列显示
                    let style = self.get_line_style(line, is_selected);
                    let content = self.truncate_content(&line.content, area_width.saturating_sub(2) as usize);
                    result.push(ListItem::new(Line::from(Span::styled(content, style))));
                }
                DiffLineType::Context => {
                    // 检查是否为 "No newline at end of file" 标记
                    if line.content.starts_with("\\") && line.content.contains("No newline at end of file") {
                        // 特殊处理：以灰色显示，跨越整行
                        let notice_style = if is_selected {
                            Style::default().fg(Color::Gray).bg(Color::DarkGray)
                        } else {
                            Style::default().fg(Color::Gray)
                        };
                        
                        // 将提示信息居中显示
                        let notice_text = "⚠ No newline at end of file";
                        let centered_content = format!("{:^width$}", notice_text, width = area_width.saturating_sub(2) as usize);
                        result.push(ListItem::new(Line::from(Span::styled(centered_content, notice_style))));
                    } else {
                        // 普通上下文行在两侧都显示
                        let left_content = self.format_side_content(&line.content, line.old_line_no, half_width as usize, true);
                        let right_content = self.format_side_content(&line.content, line.new_line_no, half_width as usize, false);
                        
                        let left_style = if is_selected { 
                            Style::default().fg(Color::White).bg(Color::DarkGray)
                        } else { 
                            Style::default().fg(Color::White) 
                        };
                        
                        result.push(ListItem::new(Line::from(vec![
                            Span::styled(left_content, left_style),
                            Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
                            Span::styled(right_content, left_style),
                        ])));
                    }
                }
                DiffLineType::Added => {
                    // 添加的行只在右侧显示
                    let left_content = " ".repeat(half_width as usize);
                    let right_content = self.format_side_content(&line.content, line.new_line_no, half_width as usize, false);
                    
                    if self.word_level_diff {
                        // 使用单词级高亮
                        let right_spans = self.apply_word_level_highlighting(&right_content, &line.line_type);
                        let mut spans = vec![
                            Span::styled(left_content, Style::default()),
                            Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
                        ];
                        spans.extend(right_spans);
                        result.push(ListItem::new(Line::from(spans)));
                    } else {
                        let right_style = if is_selected {
                            Style::default().fg(Color::Green).bg(Color::DarkGray)
                        } else {
                            Style::default().fg(Color::Green)
                        };
                        
                        result.push(ListItem::new(Line::from(vec![
                            Span::styled(left_content, Style::default()),
                            Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
                            Span::styled(right_content, right_style),
                        ])));
                    }
                }
                DiffLineType::Removed => {
                    // 删除的行只在左侧显示
                    let left_content = self.format_side_content(&line.content, line.old_line_no, half_width as usize, true);
                    let right_content = " ".repeat(half_width as usize);
                    
                    if self.word_level_diff {
                        // 使用单词级高亮
                        let left_spans = self.apply_word_level_highlighting(&left_content, &line.line_type);
                        let mut spans = vec![];
                        spans.extend(left_spans);
                        spans.extend(vec![
                            Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
                            Span::styled(right_content, Style::default()),
                        ]);
                        result.push(ListItem::new(Line::from(spans)));
                    } else {
                        let left_style = if is_selected {
                            Style::default().fg(Color::Red).bg(Color::DarkGray)
                        } else {
                            Style::default().fg(Color::Red)
                        };
                        
                        result.push(ListItem::new(Line::from(vec![
                            Span::styled(left_content, left_style),
                            Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
                            Span::styled(right_content, Style::default()),
                        ])));
                    }
                }
                DiffLineType::Binary => {
                    // 二进制文件信息跨越两列显示
                    let style = self.get_line_style(line, is_selected);
                    let content = self.truncate_content(&line.content, area_width.saturating_sub(2) as usize);
                    result.push(ListItem::new(Line::from(Span::styled(content, style))));
                }
                DiffLineType::FileTree => {
                    // 文件树信息不应出现在并排模式中
                }
            }
        }
        
        result
    }
    
    /// 格式化并排显示内容
    fn format_side_content(&self, content: &str, line_no: Option<u32>, max_width: usize, _is_old: bool) -> String {
        if max_width < 10 {
            return String::new();
        }
        
        let prefix = if let Some(no) = line_no {
            if self.show_line_numbers {
                format!("{:4} ", no)
            } else {
                String::new()
            }
        } else {
            if self.show_line_numbers {
                "     ".to_string()
            } else {
                String::new()
            }
        };
        
        // 移除原始的+/-前缀
        let clean_content = if content.starts_with('+') || content.starts_with('-') || content.starts_with(' ') {
            &content[1..]
        } else {
            content
        };
        
        let available_width = max_width.saturating_sub(prefix.len());
        let truncated_content = self.truncate_content(clean_content, available_width);
        let padded_content = format!("{}{}", truncated_content, " ".repeat(available_width.saturating_sub(truncated_content.len())));
        
        format!("{}{}", prefix, padded_content)
    }
    
    /// 截断内容到指定显示宽度（UTF-8字符边界安全）
    fn truncate_content(&self, content: &str, max_width: usize) -> String {
        if max_width == 0 {
            return String::new();
        }
        
        let mut display_width = 0;
        let mut char_end = 0;
        
        // 计算能显示的字符数，考虑不同字符的显示宽度
        for (i, ch) in content.char_indices() {
            let char_width = match ch {
                // CJK字符（中文、日文、韩文）通常占2个显示宽度
                '\u{4e00}'..='\u{9fff}' |   // CJK统一汉字
                '\u{3400}'..='\u{4dbf}' |   // CJK扩展A
                '\u{20000}'..='\u{2a6df}' | // CJK扩展B
                '\u{2a700}'..='\u{2b73f}' | // CJK扩展C
                '\u{2b740}'..='\u{2b81f}' | // CJK扩展D
                '\u{2b820}'..='\u{2ceaf}' | // CJK扩展E
                '\u{2ceb0}'..='\u{2ebef}' | // CJK扩展F
                '\u{30000}'..='\u{3134f}' | // CJK扩展G
                '\u{ac00}'..='\u{d7af}' |   // 韩文音节
                '\u{3040}'..='\u{309f}' |   // 平假名
                '\u{30a0}'..='\u{30ff}' |   // 片假名
                '\u{ff01}'..='\u{ff60}' |   // 全角ASCII
                '\u{ffe0}'..='\u{ffe6}'     // 全角符号
                => 2,
                // 制表符通常显示为4个空格
                '\t' => 4,
                // 其他字符（包括ASCII、拉丁字母等）占1个显示宽度
                _ => 1,
            };
            
            if display_width + char_width > max_width {
                break;
            }
            
            display_width += char_width;
            char_end = i + ch.len_utf8();
        }
        
        if char_end >= content.len() {
            // 整个字符串都能显示
            content.to_string()
        } else if display_width + 1 <= max_width {
            // 能显示省略号
            format!("{}…", &content[..char_end])
        } else if char_end > 0 {
            // 需要为省略号腾出空间，去掉最后一个字符
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

    /// 同步文件选择状态（确保业务逻辑与ListState一致）
    fn sync_file_selection(&mut self) {
        self.file_list_state.select(self.selected_file);
    }

    /// 导航：向上
    fn navigate_up(&mut self) {
        match self.display_mode {
            DiffDisplayMode::FileTree => {
                // 文件列表导航
                if let Some(current) = self.selected_file {
                    if current > 0 {
                        self.selected_file = Some(current - 1);
                    } else if !self.diff_files.is_empty() {
                        // 循环到最后一个文件
                        self.selected_file = Some(self.diff_files.len() - 1);
                    }
                } else if !self.diff_files.is_empty() {
                    // 如果没有选中文件，选中最后一个
                    self.selected_file = Some(self.diff_files.len() - 1);
                }
                
                // 同步状态
                self.sync_file_selection();
            }
            DiffDisplayMode::SideBySide => {
                // 在Side-by-Side模式下，向上滚动内容
                self.scroll_up(1);
            }
            _ => {
                self.scroll_up(1);
            }
        }
    }

    /// 导航：向下
    fn navigate_down(&mut self) {
        match self.display_mode {
            DiffDisplayMode::FileTree => {
                // 文件列表导航
                if let Some(current) = self.selected_file {
                    if current < self.diff_files.len().saturating_sub(1) {
                        self.selected_file = Some(current + 1);
                    } else if !self.diff_files.is_empty() {
                        // 循环到第一个文件
                        self.selected_file = Some(0);
                    }
                } else if !self.diff_files.is_empty() {
                    // 如果没有选中文件，选中第一个
                    self.selected_file = Some(0);
                }
                
                // 同步状态
                self.sync_file_selection();
            }
            DiffDisplayMode::SideBySide => {
                // 在Side-by-Side模式下，向下滚动内容
                self.scroll_down(1);
            }
            _ => {
                self.scroll_down(1);
            }
        }
    }

    /// 导航：向上翻页
    fn navigate_page_up(&mut self) {
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
    fn navigate_page_down(&mut self) {
        match self.display_mode {
            DiffDisplayMode::FileTree => {
                if let Some(current) = self.selected_file {
                    self.selected_file = Some((current + 5).min(self.diff_files.len().saturating_sub(1)));
                }
            }
            DiffDisplayMode::SideBySide | DiffDisplayMode::Unified => {
                self.scroll_down(10);
            }
        }
    }

    /// 导航：跳到开头
    fn navigate_home(&mut self) {
        match self.display_mode {
            DiffDisplayMode::FileTree => {
                self.selected_file = if !self.diff_files.is_empty() { Some(0) } else { None };
            }
            _ => {
                self.scroll_to(0);
                self.selected_line = if !self.diff_lines.is_empty() { Some(0) } else { None };
            }
        }
    }

    /// 导航：跳到结尾
    fn navigate_end(&mut self) {
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
                self.selected_line = if !self.diff_lines.is_empty() { Some(last_pos) } else { None };
            }
        }
    }

    /// 进入文件详情（从文件树模式）
    fn enter_file_details(&mut self) {
        if let Some(file_index) = self.selected_file {
            if let Some(file) = self.diff_files.get(file_index) {
                if file.is_binary {
                    // 对于二进制文件，切换到特殊的二进制对比模式
                    self.display_mode = DiffDisplayMode::Unified; // 使用unified模式但显示特殊内容
                    self.selected_line = None; // 二进制文件没有行选择
                    self.scroll_position = 0;
                } else {
                    // 对于文本文件，切换到统一diff模式并定位到选中文件的第一行
                    self.display_mode = DiffDisplayMode::Unified;
                    
                    // 查找文件在diff_lines中的起始位置
                    let mut line_start = 0;
                    let mut current_file_index = 0;
                    
                    for (i, line) in self.diff_lines.iter().enumerate() {
                        if line.line_type == DiffLineType::Header && 
                           line.content.contains(&file.path) {
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

    /// 渲染三列布局：文件列表、旧内容、新内容
    fn render_three_column_layout(&mut self, frame: &mut Frame, area: Rect, _title: &str) {
        use ratatui::layout::{Constraint, Direction, Layout};
        
        // 创建三列布局：30%文件列表, 35%旧内容, 35%新内容
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(30), // 文件列表
                Constraint::Percentage(35), // 旧文件内容
                Constraint::Percentage(35), // 新文件内容
            ])
            .split(area);

        // 渲染文件列表
        self.render_file_list(frame, chunks[0]);
        
        // 渲染旧文件内容
        self.render_old_file_content(frame, chunks[1]);
        
        // 渲染新文件内容  
        self.render_new_file_content(frame, chunks[2]);
    }

    /// 渲染文件列表
    fn render_file_list(&mut self, frame: &mut Frame, area: Rect) {
        let border_style = if self.focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::White)
        };

        // 确保ListState与业务逻辑状态同步
        self.file_list_state.select(self.selected_file);
        
        // 生成文件列表项
        let file_items: Vec<ListItem> = self.diff_files
            .iter()
            .enumerate()
            .map(|(_i, file)| {
                // 文件状态图标（根据additions和deletions推断）
                let status_icon = if file.additions > 0 && file.deletions > 0 {
                    "📝" // 修改文件
                } else if file.additions > 0 {
                    "📄" // 新增文件
                } else if file.deletions > 0 {
                    "🗑️" // 删除文件
                } else {
                    "📄" // 其他情况
                };
                
                // 文件路径（截断长路径）
                let display_name = Self::safe_truncate_path(&file.path, 25);
                
                let content = format!("{} {}", status_icon, display_name);
                
                ListItem::new(Text::raw(content))
            })
            .collect();

        // 添加选择状态到标题
        let title = if let Some(selected) = self.selected_file {
            format!("📁 Files ({}/{})", selected + 1, self.diff_files.len())
        } else {
            "📁 Files".to_string()
        };

        let file_list = List::new(file_items)
            .block(
                Block::default()
                    .title(title)
                    .borders(Borders::ALL)
                    .border_style(border_style)
            )
            .highlight_style(
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            );
        
        frame.render_stateful_widget(file_list, area, &mut self.file_list_state);
    }

    /// 渲染旧文件内容
    fn render_old_file_content(&self, frame: &mut Frame, area: Rect) {
        let border_style = Style::default().fg(Color::Red);
        
        let old_content = self.get_old_file_content();
        let old_lines: Vec<ListItem> = old_content
            .into_iter()
            .map(|line| ListItem::new(Text::raw(line)))
            .collect();

        // 构建标题，显示当前选中的文件名（截断长路径）
        let title = if let Some(file_index) = self.selected_file {
            if let Some(file) = self.diff_files.get(file_index) {
                let display_path = Self::safe_truncate_path(&file.path, 40);
                format!("🔻 Old (-): {}", display_path)
            } else {
                "🔻 Old (-)".to_string()
            }
        } else {
            "🔻 Old (-)".to_string()
        };

        let old_list = List::new(old_lines)
            .block(
                Block::default()
                    .title(title)
                    .borders(Borders::ALL)
                    .border_style(border_style)
            );

        frame.render_widget(old_list, area);
    }

    /// 渲染新文件内容
    fn render_new_file_content(&self, frame: &mut Frame, area: Rect) {
        let border_style = Style::default().fg(Color::Green);
        
        let new_content = self.get_new_file_content();
        let new_lines: Vec<ListItem> = new_content
            .into_iter()
            .map(|line| ListItem::new(Text::raw(line)))
            .collect();

        // 构建标题，显示当前选中的文件名（截断长路径）
        let title = if let Some(file_index) = self.selected_file {
            if let Some(file) = self.diff_files.get(file_index) {
                let display_path = Self::safe_truncate_path(&file.path, 40);
                format!("🔺 New (+): {}", display_path)
            } else {
                "🔺 New (+)".to_string()
            }
        } else {
            "🔺 New (+)".to_string()
        };

        let new_list = List::new(new_lines)
            .block(
                Block::default()
                    .title(title)
                    .borders(Borders::ALL)
                    .border_style(border_style)
            );

        frame.render_widget(new_list, area);
    }

    /// 获取当前选中文件的旧内容
    fn get_old_file_content(&self) -> Vec<String> {
        if let Some(file_index) = self.selected_file {
            if let Some(file) = self.diff_files.get(file_index) {
                return self.extract_file_old_lines(file);
            }
        }
        
        // 如果没有选中文件，返回提示信息
        vec![
            "".to_string(),
            "  Select a file from the".to_string(), 
            "  file list to view its".to_string(),
            "  old content here.".to_string(),
            "".to_string(),
        ]
    }

    /// 获取当前选中文件的新内容
    fn get_new_file_content(&self) -> Vec<String> {
        if let Some(file_index) = self.selected_file {
            if let Some(file) = self.diff_files.get(file_index) {
                return self.extract_file_new_lines(file);
            }
        }
        
        // 如果没有选中文件，返回提示信息
        vec![
            "".to_string(),
            "  Select a file from the".to_string(),
            "  file list to view its".to_string(), 
            "  new content here.".to_string(),
            "".to_string(),
        ]
    }

    /// 从diff文件中提取旧内容行
    fn extract_file_old_lines(&self, file: &DiffFile) -> Vec<String> {
        let mut old_lines = Vec::new();
        
        // 使用文件中已有的lines字段
        for line in &file.lines {
            match line.line_type {
                DiffLineType::Context | DiffLineType::Removed => {
                    // 上下文行和删除行包含旧内容
                    let content = if line.content.starts_with(' ') || line.content.starts_with('-') {
                        line.content[1..].to_string() // 去掉前缀符号
                    } else {
                        line.content.clone()
                    };
                    old_lines.push(content);
                }
                _ => {}
            }
        }
        
        // 调试信息：显示文件和行数信息
        if old_lines.is_empty() {
            old_lines.push(format!("DEBUG - File: {}", file.path));
            old_lines.push(format!("Total lines in file: {}", file.lines.len()));
            old_lines.push(format!("Selected file index: {:?}", self.selected_file));
            old_lines.push("Line types and content:".to_string());
            for (i, line) in file.lines.iter().enumerate() {
                if i < 5 {  // 只显示前5行避免过多信息
                    old_lines.push(format!("  {}: {:?} - {}", i, line.line_type, 
                        Self::safe_truncate_content(&line.content, 50)));
                }
            }
            if file.lines.len() > 5 {
                old_lines.push(format!("  ... and {} more", file.lines.len() - 5));
            }
            
            // 统计不同类型的行数
            let mut type_counts = std::collections::HashMap::new();
            for line in &file.lines {
                *type_counts.entry(format!("{:?}", line.line_type)).or_insert(0) += 1;
            }
            old_lines.push("Type counts:".to_string());
            for (line_type, count) in type_counts {
                old_lines.push(format!("  {}: {}", line_type, count));
            }
        }
        
        old_lines
    }

    /// 从diff文件中提取新内容行
    fn extract_file_new_lines(&self, file: &DiffFile) -> Vec<String> {
        let mut new_lines = Vec::new();
        
        // 使用文件中已有的lines字段
        for line in &file.lines {
            match line.line_type {
                DiffLineType::Context | DiffLineType::Added => {
                    // 上下文行和添加行包含新内容
                    let content = if line.content.starts_with(' ') || line.content.starts_with('+') {
                        line.content[1..].to_string() // 去掉前缀符号
                    } else {
                        line.content.clone()
                    };
                    new_lines.push(content);
                }
                _ => {}
            }
        }
        
        // 调试信息：显示文件和行数信息
        if new_lines.is_empty() {
            new_lines.push(format!("DEBUG - File: {}", file.path));
            new_lines.push(format!("Total lines in file: {}", file.lines.len()));
            new_lines.push(format!("Selected file index: {:?}", self.selected_file));
            new_lines.push("Line types and content:".to_string());
            for (i, line) in file.lines.iter().enumerate() {
                if i < 5 {  // 只显示前5行避免过多信息
                    new_lines.push(format!("  {}: {:?} - {}", i, line.line_type, 
                        Self::safe_truncate_content(&line.content, 50)));
                }
            }
            if file.lines.len() > 5 {
                new_lines.push(format!("  ... and {} more", file.lines.len() - 5));
            }
            
            // 统计不同类型的行数
            let mut type_counts = std::collections::HashMap::new();
            for line in &file.lines {
                *type_counts.entry(format!("{:?}", line.line_type)).or_insert(0) += 1;
            }
            new_lines.push("Type counts:".to_string());
            for (line_type, count) in type_counts {
                new_lines.push(format!("  {}: {}", line_type, count));
            }
        }
        
        new_lines
    }
}

impl Component for DiffViewerComponent {
    fn name(&self) -> &str {
        "DiffViewerComponent"
    }

    fn render(&mut self, frame: &mut Frame, area: Rect, _state: &AppState) {
        let border_style = if self.focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::White)
        };

        // 构建标题 - 包含显示模式和特性信息
        let mut title_parts = vec![];
        let mode_indicator = match self.display_mode {
            DiffDisplayMode::Unified => "📄 Unified Diff",
            DiffDisplayMode::SideBySide => "⚖️ Side-by-Side Diff",
            DiffDisplayMode::FileTree => "🌳 File Tree Diff",
        };
        title_parts.push(mode_indicator.to_string());
        
        // 添加特性指示器
        let mut features = vec![];
        if self.word_level_diff {
            features.push("🔍Word");
        }
        if self.show_line_numbers {
            features.push("📊Line#");
        }
        if !features.is_empty() {
            title_parts.push(format!(" [{}]", features.join(" ")));
        }
        
        if let Some(ref file) = self.current_file {
            title_parts.push(format!(" - {}", file));
        }
        if let Some(ref commit) = self.current_commit {
            title_parts.push(format!(" ({})", &commit[..8.min(commit.len())]));
        }
        let title = title_parts.join("");

        // 根据显示模式选择渲染内容
        let visible_lines = match self.display_mode {
            DiffDisplayMode::FileTree => {
                // 文件树模式
                self.generate_file_tree_view()
            }
            DiffDisplayMode::SideBySide => {
                // 三列模式：使用三个独立区域渲染文件列表、旧内容、新内容
                self.render_three_column_layout(frame, area, &title);
                return; // 直接返回，不使用通用的List渲染
            }
            DiffDisplayMode::Unified => {
                // 统一diff模式
                self.generate_unified_view(area.height.saturating_sub(2) as usize)
            }
        };

        let list = List::new(visible_lines)
            .block(
                Block::default()
                    .title(title)
                    .borders(Borders::ALL)
                    .border_style(border_style)
            );

        frame.render_widget(list, area);

        // 渲染滚动条
        let content_len = match self.display_mode {
            DiffDisplayMode::FileTree => self.diff_files.len() + 3, // 额外的概览行
            _ => self.diff_lines.len(),
        };
        
        let visible_height = area.height.saturating_sub(2) as usize;
        if content_len > visible_height {
            let scrollbar_area = Rect {
                x: area.x + area.width - 1,
                y: area.y + 1,
                width: 1,
                height: area.height - 2,
            };

            let mut scrollbar_state = ScrollbarState::default()
                .content_length(content_len)
                .viewport_content_length(visible_height)
                .position(self.scroll_position);

            frame.render_stateful_widget(
                Scrollbar::default()
                    .orientation(ScrollbarOrientation::VerticalRight)
                    .begin_symbol(Some("↑"))
                    .end_symbol(Some("↓")),
                scrollbar_area,
                &mut scrollbar_state,
            );
        }
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
                    // 切换到上一个文件
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
                    // 切换到下一个文件
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
            // ESC 键不在组件内处理，交由上层处理模态框关闭
            // 如果需要返回文件树模式，可以使用其他快捷键，比如Backspace
            (KeyCode::Backspace, _) => {
                if self.display_mode != DiffDisplayMode::FileTree {
                    self.toggle_display_mode(Some(DiffDisplayMode::FileTree));
                }
                EventResult::Handled
            }
            _ => EventResult::NotHandled
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
        // 从当前位置开始搜索
        let start_pos = self.selected_line.unwrap_or(0);
        
        for (i, line) in self.diff_lines.iter().enumerate().skip(start_pos + 1) {
            if line.content.to_lowercase().contains(&query) {
                self.selected_line = Some(i);
                // 确保选中行可见
                let visible_height = 20; // 估算可见高度
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