#!/usr/bin/env rust-script

use std::fs;

// 导入测试所需的结构（简化版本）
#[derive(Debug, Clone)]
pub struct DiffFile {
    pub path: String,
    pub lines: Vec<DiffLine>,
}

#[derive(Debug, Clone)]
pub struct DiffLine {
    pub line_type: DiffLineType,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DiffLineType {
    Header, FileHeader, Hunk, Context, Added, Removed, Binary,
}

fn main() {
    let diff_content = fs::read_to_string("/tmp/test_diff.txt")
        .expect("Failed to read test diff file");
    
    let (files, _) = parse_enhanced_diff(&diff_content);
    
    println!("=== CONTENT EXTRACTION TEST ===");
    
    // 测试前几个文件的内容提取
    for (i, file) in files.iter().enumerate().take(3) {
        println!("\n--- File {}: {} ---", i, file.path);
        
        let old_content = extract_file_old_lines(file);
        let new_content = extract_file_new_lines(file);
        
        println!("Old content lines: {}", old_content.len());
        if !old_content.is_empty() {
            for (j, line) in old_content.iter().enumerate().take(3) {
                println!("  Old {}: {}", j, if line.len() > 80 { format!("{}...", &line[..80]) } else { line.clone() });
            }
            if old_content.len() > 3 {
                println!("  ... and {} more old lines", old_content.len() - 3);
            }
        }
        
        println!("New content lines: {}", new_content.len());
        if !new_content.is_empty() {
            for (j, line) in new_content.iter().enumerate().take(3) {
                println!("  New {}: {}", j, if line.len() > 80 { format!("{}...", &line[..80]) } else { line.clone() });
            }
            if new_content.len() > 3 {
                println!("  ... and {} more new lines", new_content.len() - 3);
            }
        }
    }
}

// 简化的解析函数（只关注核心逻辑）
fn parse_enhanced_diff(content: &str) -> (Vec<DiffFile>, Vec<DiffLine>) {
    let mut files = Vec::new();
    let mut current_file: Option<DiffFile> = None;
    let mut current_lines = Vec::new();
    
    for line in content.lines() {
        if line.starts_with("diff --git") {
            // 保存之前的文件
            if let Some(mut file) = current_file.take() {
                file.lines = current_lines;
                files.push(file);
                current_lines = Vec::new();
            }
            
            // 创建新文件
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                let path = parts[3].trim_start_matches("b/");
                current_file = Some(DiffFile {
                    path: path.to_string(),
                    lines: Vec::new(),
                });
            }
        } else {
            // 解析行类型
            let line_type = if line.starts_with("@@") {
                DiffLineType::Hunk
            } else if line.starts_with("+++") || line.starts_with("---") {
                DiffLineType::FileHeader
            } else if line.starts_with("index") || line.starts_with("diff") {
                DiffLineType::Header
            } else if line.starts_with("+") {
                DiffLineType::Added
            } else if line.starts_with("-") {
                DiffLineType::Removed
            } else if line.starts_with(" ") {
                DiffLineType::Context
            } else {
                DiffLineType::Header
            };
            
            let diff_line = DiffLine {
                line_type,
                content: line.to_string(),
            };
            
            current_lines.push(diff_line);
        }
    }
    
    // 保存最后一个文件
    if let Some(mut file) = current_file {
        file.lines = current_lines;
        files.push(file);
    }
    
    (files, Vec::new())
}

// 模拟我们的内容提取函数
fn extract_file_old_lines(file: &DiffFile) -> Vec<String> {
    let mut old_lines = Vec::new();
    
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
    
    old_lines
}

fn extract_file_new_lines(file: &DiffFile) -> Vec<String> {
    let mut new_lines = Vec::new();
    
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
    
    new_lines
}