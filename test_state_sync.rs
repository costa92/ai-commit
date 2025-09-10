#!/usr/bin/env rust-script

// 模拟ListState行为
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
        println!("ListState.select({:?}) called", index);
    }
}

// 模拟我们的组件状态
struct MockDiffViewer {
    selected_file: Option<usize>,
    file_list_state: MockListState,
    diff_files_count: usize,
}

impl MockDiffViewer {
    fn new() -> Self {
        Self {
            selected_file: None,
            file_list_state: MockListState::new(),
            diff_files_count: 0,
        }
    }
    
    fn set_diff(&mut self, file_count: usize) {
        println!("\n=== set_diff called with {} files ===", file_count);
        
        self.diff_files_count = file_count;
        self.selected_file = if file_count > 0 { Some(0) } else { None };
        
        // 同步更新file_list_state
        self.file_list_state.select(self.selected_file);
        
        println!("After set_diff: selected_file = {:?}", self.selected_file);
        println!("After set_diff: list_state.selected = {:?}", self.file_list_state.selected);
    }
    
    fn navigate_down(&mut self) {
        println!("\n=== navigate_down called ===");
        println!("Before: selected_file = {:?}, total_files = {}", self.selected_file, self.diff_files_count);
        
        if let Some(current) = self.selected_file {
            if current < self.diff_files_count.saturating_sub(1) {
                self.selected_file = Some(current + 1);
            } else if self.diff_files_count > 0 {
                self.selected_file = Some(0);
            }
        } else if self.diff_files_count > 0 {
            self.selected_file = Some(0);
        }
        
        // 同步更新file_list_state
        self.file_list_state.select(self.selected_file);
        
        println!("After: selected_file = {:?}", self.selected_file);
        println!("After: list_state.selected = {:?}", self.file_list_state.selected);
    }
    
    fn navigate_up(&mut self) {
        println!("\n=== navigate_up called ===");
        println!("Before: selected_file = {:?}, total_files = {}", self.selected_file, self.diff_files_count);
        
        if let Some(current) = self.selected_file {
            if current > 0 {
                self.selected_file = Some(current - 1);
            } else if self.diff_files_count > 0 {
                self.selected_file = Some(self.diff_files_count - 1);
            }
        } else if self.diff_files_count > 0 {
            self.selected_file = Some(self.diff_files_count - 1);
        }
        
        // 同步更新file_list_state
        self.file_list_state.select(self.selected_file);
        
        println!("After: selected_file = {:?}", self.selected_file);
        println!("After: list_state.selected = {:?}", self.file_list_state.selected);
    }
    
    fn render_file_list(&mut self) {
        println!("\n=== render_file_list called ===");
        
        // 同步选择状态到file_list_state（渲染时的保险措施）
        self.file_list_state.select(self.selected_file);
        
        println!("Rendering file list with selected_file = {:?}", self.selected_file);
        println!("ListState.selected = {:?}", self.file_list_state.selected);
        
        // 模拟列表项渲染
        for i in 0..self.diff_files_count {
            let is_selected = Some(i) == self.selected_file;
            let list_highlight = self.file_list_state.selected == Some(i);
            
            println!("  File {}: business_logic_selected={}, list_state_selected={}", 
                     i, is_selected, list_highlight);
                     
            if is_selected != list_highlight {
                println!("    ⚠️  STATE MISMATCH!");
            }
        }
    }
}

fn main() {
    println!("=== MOCK DIFF VIEWER STATE SYNCHRONIZATION TEST ===");
    
    let mut viewer = MockDiffViewer::new();
    
    // 测试场景1：设置diff数据
    viewer.set_diff(5);  // 模拟5个文件
    viewer.render_file_list();
    
    // 测试场景2：向下导航
    viewer.navigate_down();
    viewer.render_file_list();
    
    viewer.navigate_down();
    viewer.render_file_list();
    
    // 测试场景3：向上导航
    viewer.navigate_up();
    viewer.render_file_list();
    
    // 测试场景4：循环导航
    viewer.navigate_up();
    viewer.navigate_up();
    viewer.navigate_up();  // 应该循环到最后一个
    viewer.render_file_list();
    
    println!("\n=== Test completed ===");
}