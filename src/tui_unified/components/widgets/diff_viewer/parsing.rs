use super::types::{DiffFile, DiffLine, DiffLineType};
use super::DiffViewerComponent;

impl DiffViewerComponent {
    /// 增强的diff解析（支持多文件和元数据）
    pub(super) fn parse_enhanced_diff(&self, content: &str) -> (Vec<DiffFile>, Vec<DiffLine>) {
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
    pub(super) fn parse_single_line(
        &self,
        line: &str,
        old_line_no: &mut u32,
        new_line_no: &mut u32,
    ) -> DiffLine {
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
            (
                DiffLineType::Context,
                Some(*old_line_no),
                Some(*new_line_no),
            )
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

    /// 解析hunk头部信息
    pub(super) fn parse_hunk_header(&self, line: &str) -> Option<(u32, u32)> {
        // 简单的hunk头解析：@@ -old_start,old_count +new_start,new_count @@
        if let Some(start) = line.find("-") {
            if let Some(end) = line.find(" +") {
                let old_part = &line[start + 1..end];
                if let Some(comma) = old_part.find(",") {
                    if let Ok(old_start) = old_part[..comma].parse::<u32>() {
                        let new_start = line[end + 2..].split(',').next()?.parse::<u32>().ok()?;
                        return Some((old_start, new_start));
                    }
                }
            }
        }
        None
    }

    /// 原有的parse_diff方法，保持向后兼容
    #[allow(dead_code)]
    pub(super) fn parse_diff(&self, content: &str) -> Vec<DiffLine> {
        let mut lines = Vec::new();
        let mut old_line_no = 0u32;
        let mut new_line_no = 0u32;

        for line in content.lines() {
            let parsed_line = self.parse_single_line(line, &mut old_line_no, &mut new_line_no);
            lines.push(parsed_line);
        }

        lines
    }

    /// 检查是否为图片文件
    pub(super) fn is_image_file(&self, path: &str) -> bool {
        let image_extensions = [
            ".png", ".jpg", ".jpeg", ".gif", ".bmp", ".svg", ".webp", ".tiff", ".tif", ".ico",
            ".avif", ".heic", ".heif",
        ];
        let lower_path = path.to_lowercase();
        image_extensions.iter().any(|ext| lower_path.ends_with(ext))
    }

    /// 检查是否为二进制文件类型
    pub(super) fn is_likely_binary_file(&self, path: &str) -> bool {
        let binary_extensions = [
            // 可执行文件
            ".exe", ".dll", ".so", ".dylib", ".a", ".lib", ".bin", // 压缩文件
            ".zip", ".tar", ".gz", ".bz2", ".xz", ".7z", ".rar", // 媒体文件
            ".mp3", ".mp4", ".avi", ".mkv", ".wav", ".flac", ".ogg", // 办公文档
            ".pdf", ".doc", ".docx", ".xls", ".xlsx", ".ppt", ".pptx", // 数据库
            ".db", ".sqlite", ".sqlite3", ".mdb", // 其他二进制格式
            ".pyc", ".class", ".jar", ".dex", ".apk",
        ];
        let lower_path = path.to_lowercase();
        binary_extensions
            .iter()
            .any(|ext| lower_path.ends_with(ext))
            || self.is_image_file(path)
    }
}
