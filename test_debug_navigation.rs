#!/usr/bin/env rust-script

//! Test script to demonstrate the enhanced debug functionality
//! This simulates the TUI interaction to understand navigation issues

use std::fs;

// Mock the essential structures from diff_viewer.rs
#[derive(Debug, Clone)]
pub struct DiffFile {
    pub path: String,
    pub additions: u32,
    pub deletions: u32,
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

#[derive(Debug, Clone, PartialEq)]
pub enum DiffDisplayMode {
    Unified,
    FileTree,
    SideBySide,
}

// Mock ListState from ratatui
#[derive(Debug)]
struct MockListState {
    selected: Option<usize>,
}

impl MockListState {
    fn new() -> Self {
        Self { selected: None }
    }
    
    fn select(&mut self, index: Option<usize>) {
        self.selected = index;
        eprintln!("ListState.select({:?}) called", index);
    }
    
    fn selected(&self) -> Option<usize> {
        self.selected
    }
}

// Mock DiffViewer with enhanced debug functionality
struct MockDiffViewer {
    diff_files: Vec<DiffFile>,
    selected_file: Option<usize>,
    file_list_state: MockListState,
    display_mode: DiffDisplayMode,
}

impl MockDiffViewer {
    fn new() -> Self {
        Self {
            diff_files: Vec::new(),
            selected_file: None,
            file_list_state: MockListState::new(),
            display_mode: DiffDisplayMode::SideBySide,
        }
    }
    
    /// 同步文件选择状态（确保业务逻辑与ListState一致）
    fn sync_file_selection(&mut self) {
        self.file_list_state.select(self.selected_file);
        eprintln!("sync_file_selection: selected_file={:?}, list_state.selected={:?}", 
                 self.selected_file, self.file_list_state.selected());
    }
    
    fn set_diff(&mut self, diff_content: &str) {
        println!("\n=== set_diff called ===");
        
        let (files, _) = self.parse_enhanced_diff(diff_content);
        self.diff_files = files;
        self.selected_file = if !self.diff_files.is_empty() { Some(0) } else { None };
        
        // 使用统一的同步方法
        self.sync_file_selection();
        eprintln!("set_diff: initialized with {} files, selected_file={:?}", 
                 self.diff_files.len(), self.selected_file);
    }
    
    fn navigate_up(&mut self) {
        eprintln!("\n=== navigate_up called ===");
        
        match self.display_mode {
            DiffDisplayMode::FileTree | DiffDisplayMode::SideBySide => {
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
                
                // 同步状态并输出调试信息
                self.sync_file_selection();
                eprintln!("navigate_up: selected_file = {:?}, total_files = {}", 
                         self.selected_file, self.diff_files.len());
            }
            _ => {}
        }
    }
    
    fn navigate_down(&mut self) {
        eprintln!("\n=== navigate_down called ===");
        
        match self.display_mode {
            DiffDisplayMode::FileTree | DiffDisplayMode::SideBySide => {
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
                
                // 同步状态并输出调试信息
                self.sync_file_selection();
                eprintln!("navigate_down: selected_file = {:?}, total_files = {}", 
                         self.selected_file, self.diff_files.len());
            }
            _ => {}
        }
    }
    
    fn render_file_list(&mut self) {
        eprintln!("\n=== render_file_list called ===");
        
        // 首先确保ListState与业务逻辑状态同步
        self.file_list_state.select(self.selected_file);
        
        eprintln!("render_file_list: selected_file={:?}, list_state.selected={:?}, total_files={}", 
                 self.selected_file, self.file_list_state.selected(), self.diff_files.len());
        
        // 模拟渲染文件列表
        for (i, file) in self.diff_files.iter().enumerate().take(5) {
            let is_selected = Some(i) == self.selected_file;
            let is_list_selected = self.file_list_state.selected() == Some(i);
            
            let debug_marker = if is_selected && is_list_selected {
                "✓" // 两个状态一致
            } else if is_selected {
                "!" // 业务逻辑选中但ListState未选中
            } else if is_list_selected {
                "?" // ListState选中但业务逻辑未选中
            } else {
                " " // 都未选中
            };
            
            println!("  {}{} {}", debug_marker, i, file.path);
        }
        
        if self.diff_files.len() > 5 {
            println!("  ... and {} more files", self.diff_files.len() - 5);
        }
    }
    
    // 简化的解析函数
    fn parse_enhanced_diff(&self, content: &str) -> (Vec<DiffFile>, Vec<DiffLine>) {
        let mut files = Vec::new();
        let mut current_file: Option<DiffFile> = None;
        
        for line in content.lines() {
            if line.starts_with("diff --git") {
                // 保存之前的文件
                if let Some(file) = current_file.take() {
                    files.push(file);
                }
                
                // 创建新文件
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 4 {
                    let path = parts[3].trim_start_matches("b/");
                    current_file = Some(DiffFile {
                        path: path.to_string(),
                        additions: (1..=10).collect::<Vec<u32>>()[files.len() % 10], // 模拟不同的additions
                        deletions: (0..=5).collect::<Vec<u32>>()[files.len() % 6],  // 模拟不同的deletions
                        lines: Vec::new(),
                    });
                }
            }
        }
        
        // 保存最后一个文件
        if let Some(file) = current_file {
            files.push(file);
        }
        
        (files, Vec::new())
    }
}

fn main() {
    println!("=== ENHANCED DEBUG NAVIGATION TEST ===");
    
    // 读取测试数据
    let diff_content = match fs::read_to_string("/tmp/test_diff.txt") {
        Ok(content) => content,
        Err(_) => {
            println!("Warning: Cannot read /tmp/test_diff.txt, using minimal test data");
            "diff --git a/file1.rs b/file1.rs\ndiff --git a/file2.rs b/file2.rs\ndiff --git a/file3.rs b/file3.rs".to_string()
        }
    };
    
    let mut viewer = MockDiffViewer::new();
    
    // 测试场景1：设置diff数据
    viewer.set_diff(&diff_content);
    viewer.render_file_list();
    
    // 测试场景2：向下导航几次
    println!("\n--- Testing DOWN navigation ---");
    for i in 0..3 {
        println!("\nStep {}: Navigate DOWN", i + 1);
        viewer.navigate_down();
        viewer.render_file_list();
    }
    
    // 测试场景3：向上导航几次
    println!("\n--- Testing UP navigation ---");
    for i in 0..3 {
        println!("\nStep {}: Navigate UP", i + 1);
        viewer.navigate_up();
        viewer.render_file_list();
    }
    
    // 测试场景4：循环导航测试
    println!("\n--- Testing circular navigation ---");
    println!("Going to last file and wrapping around...");
    for _ in 0..viewer.diff_files.len() + 2 {
        viewer.navigate_down();
    }
    viewer.render_file_list();
    
    println!("\n=== Debug Test Completed ===");
    println!("Key observations:");
    println!("- ✓ means both business logic and ListState are in sync");
    println!("- ! means business logic selected but ListState is not");
    println!("- ? means ListState selected but business logic is not");
    println!("- Look for sync_file_selection and render_file_list debug output");
}