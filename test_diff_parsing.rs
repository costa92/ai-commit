#!/usr/bin/env rust-script

use std::fs;

// 简化的测试结构
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

#[derive(Debug, Clone)]
pub struct DiffLine {
    pub line_type: DiffLineType,
    pub content: String,
    pub old_line_no: Option<u32>,
    pub new_line_no: Option<u32>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DiffLineType {
    Header,
    FileHeader,
    Hunk,
    Context,
    Added,
    Removed,
    Binary,
}

fn main() {
    let diff_content = fs::read_to_string("/tmp/test_diff.txt")
        .expect("Failed to read test diff file");
    
    let (files, _lines) = parse_enhanced_diff(&diff_content);
    
    println!("=== DIFF PARSING TEST ===");
    println!("Total files found: {}", files.len());
    println!();
    
    for (i, file) in files.iter().enumerate().take(5) {  // 只显示前5个文件
        println!("File {}: {}", i, file.path);
        println!("  Lines: {}", file.lines.len());
        println!("  Additions: {}, Deletions: {}", file.additions, file.deletions);
        
        // 统计行类型
        let mut type_counts = std::collections::HashMap::new();
        for line in &file.lines {
            *type_counts.entry(format!("{:?}", line.line_type)).or_insert(0) += 1;
        }
        
        println!("  Line types:");
        for (line_type, count) in &type_counts {
            println!("    {}: {}", line_type, count);
        }
        
        // 显示一些示例内容
        println!("  Sample content:");
        for (j, line) in file.lines.iter().enumerate().take(3) {
            println!("    {}: {:?} - {}", j, line.line_type, 
                if line.content.len() > 60 { 
                    format!("{}...", &line.content[..60]) 
                } else { 
                    line.content.clone() 
                });
        }
        println!();
    }
    
    if files.len() > 5 {
        println!("... and {} more files", files.len() - 5);
    }
}

fn parse_enhanced_diff(content: &str) -> (Vec<DiffFile>, Vec<DiffLine>) {
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
                current_file = Some(DiffFile {
                    path: path.to_string(),
                    old_path: None,
                    is_binary: false,
                    is_image: false,
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
            // 使用简化的解析逻辑处理其他行
            let parsed_line = parse_single_line(line);
            
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

fn parse_single_line(line: &str) -> DiffLine {
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
    
    DiffLine {
        line_type,
        content: line.to_string(),
        old_line_no: None,
        new_line_no: None,
    }
}