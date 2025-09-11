# TUI 布局优化实现指南

## 实现概览

基于 [new-layout-design.md](./new-layout-design.md) 的设计方案，本指南提供具体的实现步骤和代码结构调整方案。

## Phase 1: 头部导航组件创建

### 1.1 创建 HeaderNavigation 组件

**文件位置：** `src/tui_unified/components/panels/header_navigation.rs`

```rust
use crossterm::event::KeyEvent;
use ratatui::{
    Frame, 
    layout::Rect, 
    widgets::{Block, Borders, Tabs}, 
    style::{Color, Style, Modifier},
    text::Spans
};
use crate::tui_unified::{
    state::AppState,
    components::base::{
        component::{Component, PanelComponent, PanelType},
        events::EventResult
    }
};

pub struct HeaderNavigation {
    focused: bool,
    selected_index: usize,
    tab_items: Vec<TabItem>,
}

struct TabItem {
    label: String,
    key: char,
    view_type: ViewType,
}

impl HeaderNavigation {
    pub fn new() -> Self {
        let tab_items = vec![
            TabItem {
                label: "Branches".to_string(),
                key: '1',
                view_type: ViewType::Branches,
            },
            TabItem {
                label: "Tags".to_string(),
                key: '2',
                view_type: ViewType::Tags,
            },
            TabItem {
                label: "Stash".to_string(),
                key: '3',
                view_type: ViewType::Stash,
            },
            TabItem {
                label: "Remotes".to_string(),
                key: '4',
                view_type: ViewType::Remotes,
            },
            TabItem {
                label: "History".to_string(),
                key: '5',
                view_type: ViewType::QueryHistory,
            },
        ];

        Self {
            focused: false,
            selected_index: 0,
            tab_items,
        }
    }
    
    /// 与当前视图同步
    fn sync_with_current_view(&mut self, state: &AppState) {
        let current_view = state.current_view;
        if let Some(index) = self.tab_items.iter().position(|item| item.view_type == current_view) {
            self.selected_index = index;
        }
    }
}

impl Component for HeaderNavigation {
    fn name(&self) -> &str {
        "HeaderNavigation"
    }

    fn render(&mut self, frame: &mut Frame, area: Rect, state: &AppState) {
        self.sync_with_current_view(state);
        
        let titles: Vec<Spans> = self.tab_items
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let style = if i == self.selected_index {
                    if self.focused {
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD)
                    }
                } else {
                    Style::default().fg(Color::Gray)
                };
                Spans::from(vec![Span::styled(format!("[{}] {}", item.key, item.label), style)])
            })
            .collect();

        let tabs = Tabs::new(titles)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(if self.focused {
                        Style::default().fg(Color::Yellow)
                    } else {
                        Style::default().fg(Color::White)
                    })
            )
            .select(self.selected_index)
            .highlight_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            );

        frame.render_widget(tabs, area);
    }

    fn handle_key_event(&mut self, key: KeyEvent, state: &mut AppState) -> EventResult {
        use crossterm::event::KeyCode;
        
        match key.code {
            KeyCode::Left | KeyCode::Char('h') => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                } else {
                    self.selected_index = self.tab_items.len() - 1;
                }
                // 切换视图
                let view_type = self.tab_items[self.selected_index].view_type;
                state.set_current_view(view_type);
                EventResult::Handled
            }
            KeyCode::Right | KeyCode::Char('l') => {
                if self.selected_index < self.tab_items.len() - 1 {
                    self.selected_index += 1;
                } else {
                    self.selected_index = 0;
                }
                // 切换视图
                let view_type = self.tab_items[self.selected_index].view_type;
                state.set_current_view(view_type);
                EventResult::Handled
            }
            KeyCode::Char(c) if c >= '1' && c <= '5' => {
                let index = (c as u8 - b'1') as usize;
                if index < self.tab_items.len() {
                    self.selected_index = index;
                    let view_type = self.tab_items[index].view_type;
                    state.set_current_view(view_type);
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
        (60, 3)
    }
}

impl PanelComponent for HeaderNavigation {
    fn panel_type(&self) -> PanelType {
        PanelType::Header
    }

    fn supports_scroll(&self) -> bool {
        false
    }

    fn scroll_position(&self) -> usize {
        0
    }

    fn set_scroll_position(&mut self, _position: usize) {
        // Header navigation doesn't support scrolling
    }
}
```

### 1.2 更新焦点管理器

**文件：** `src/tui_unified/focus/manager.rs`

```rust
// 添加新的焦点面板类型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FocusPanel {
    Header,     // 新增：头部导航
    Sidebar,    // 重命名：左侧边栏（状态+列表）
    Content,    // 主内容区域
}

impl FocusManager {
    pub fn new() -> Self {
        let mut focus_ring = FocusRing::new();
        focus_ring.add_panel(FocusPanel::Header);
        focus_ring.add_panel(FocusPanel::Sidebar);
        focus_ring.add_panel(FocusPanel::Content);
        
        Self {
            focus_ring,
            current_focus: FocusPanel::Header, // 默认焦点在头部导航
        }
    }
}
```

### 1.3 更新组件基类

**文件：** `src/tui_unified/components/base/component.rs`

```rust
// 添加新的面板类型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PanelType {
    Header,    // 新增
    Sidebar,
    Content,
    Detail,
    Modal,
}
```

## Phase 2: 侧边栏组件重构

### 2.1 重构 SidebarPanel

**文件：** `src/tui_unified/components/panels/sidebar.rs`

```rust
// 主要修改：移除菜单逻辑，专注于状态显示和动态列表

pub struct SidebarPanel {
    focused: bool,
    // 移除: menu_items, selected_index 等菜单相关字段
    
    // 保留和新增：
    current_list_type: SidebarListType,
    selected_list_index: usize,
    
    // 各种列表的数据
    branches_list: Vec<BranchInfo>,
    tags_list: Vec<TagInfo>,
    stash_list: Vec<StashInfo>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SidebarListType {
    None,        // Remotes/History 时不显示列表
    Branches,
    Tags,
    Stash,
}

impl SidebarPanel {
    /// 根据当前视图更新侧边栏列表类型
    fn update_list_type_from_view(&mut self, state: &AppState) {
        let new_list_type = match state.current_view {
            ViewType::Branches => SidebarListType::Branches,
            ViewType::Tags => SidebarListType::Tags,
            ViewType::Stash => SidebarListType::Stash,
            ViewType::Remotes | ViewType::QueryHistory => SidebarListType::None,
        };
        
        if self.current_list_type != new_list_type {
            self.current_list_type = new_list_type;
            self.selected_list_index = 0; // 重置选择
        }
    }
    
    fn render_dynamic_list(&mut self, frame: &mut Frame, area: Rect, state: &AppState) {
        match self.current_list_type {
            SidebarListType::Branches => self.render_branches_list(frame, area, state),
            SidebarListType::Tags => self.render_tags_list(frame, area, state),
            SidebarListType::Stash => self.render_stash_list(frame, area, state),
            SidebarListType::None => self.render_quick_actions(frame, area, state),
        }
    }
    
    // ... 各种列表渲染方法
}

impl Component for SidebarPanel {
    fn render(&mut self, frame: &mut Frame, area: Rect, state: &AppState) {
        // 根据当前视图更新列表类型
        self.update_list_type_from_view(state);
        
        // 计算布局：上半部分仓库状态，下半部分动态列表
        let status_height = if area.height > 25 { 10 } else { 8 };
        let list_height = area.height.saturating_sub(status_height);
        
        let status_area = Rect { y: area.y, height: status_height, ..area };
        let list_area = Rect { y: area.y + status_height, height: list_height, ..area };
        
        // 渲染仓库状态（固定内容）
        self.render_repository_status(frame, status_area, state);
        
        // 渲染动态列表
        if list_height > 3 {
            self.render_dynamic_list(frame, list_area, state);
        }
    }
    
    fn handle_key_event(&mut self, key: KeyEvent, state: &mut AppState) -> EventResult {
        // 移除菜单导航逻辑，专注于列表导航
        use crossterm::event::KeyCode;
        
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.navigate_list_up(state);
                EventResult::Handled
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.navigate_list_down(state);
                EventResult::Handled
            }
            KeyCode::Enter => {
                self.select_current_list_item(state);
                EventResult::Handled
            }
            _ => EventResult::NotHandled
        }
    }
}
```

## Phase 3: 布局管理器更新

### 3.1 新增头部-内容布局

**文件：** `src/tui_unified/layout/manager.rs`

```rust
// 新增布局结果类型
pub struct HeaderContentLayout {
    pub header: Rect,
    pub sidebar: Rect,
    pub content: Rect,
    pub status_bar: Rect,
}

impl LayoutManager {
    pub fn calculate_header_content_layout(&self, area: Rect) -> HeaderContentLayout {
        let header_height = 3; // 头部导航高度
        
        // 垂直分割：头部导航 + 主体区域 + 状态栏
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(header_height),                                    // 头部导航
                Constraint::Length(area.height.saturating_sub(header_height + STATUS_BAR_HEIGHT)), // 主体区域
                Constraint::Length(STATUS_BAR_HEIGHT),                              // 状态栏
            ])
            .split(area);
        
        // 主体区域水平分割：侧边栏 + 内容区域
        let (sidebar_constraint, content_constraint) = 
            self.calculate_sidebar_content_constraints(area.width);
        
        let body_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                sidebar_constraint,   // 左侧边栏
                content_constraint,   // 主内容区域
            ])
            .split(main_chunks[1]);
        
        HeaderContentLayout {
            header: main_chunks[0],
            sidebar: body_chunks[0],
            content: body_chunks[1],
            status_bar: main_chunks[2],
        }
    }
    
    fn calculate_sidebar_content_constraints(&self, terminal_width: u16) -> (Constraint, Constraint) {
        match terminal_width {
            w if w < 100 => {
                (Constraint::Length(20), Constraint::Min(50))
            },
            w if w < 120 => {
                (Constraint::Percentage(20), Constraint::Percentage(80))
            },
            _ => {
                (Constraint::Percentage(18), Constraint::Percentage(82))
            }
        }
    }
}
```

## Phase 4: 主应用集成

### 4.1 更新 TuiUnifiedApp

**文件：** `src/tui_unified/app.rs`

```rust
use crate::tui_unified::components::panels::header_navigation::HeaderNavigation;

pub struct TuiUnifiedApp {
    // 新增头部导航组件
    header_navigation: HeaderNavigation,
    
    // 现有组件
    sidebar_panel: SidebarPanel,
    // ... 其他组件保持不变
    
    // 更新焦点管理
    focus_manager: FocusManager,
}

impl TuiUnifiedApp {
    pub async fn new() -> Result<Self> {
        // ...现有初始化逻辑
        
        let mut focus_manager = FocusManager::new();
        focus_manager.set_focus(FocusPanel::Header);  // 默认焦点在头部
        
        Ok(Self {
            header_navigation: HeaderNavigation::new(),
            sidebar_panel: SidebarPanel::new(),
            // ... 其他组件
            focus_manager,
            // ...
        })
    }
    
    fn render_ui(&mut self, frame: &mut Frame) -> Result<()> {
        let area = frame.size();
        
        // 使用新的布局计算
        let layout = self.layout_manager.calculate_header_content_layout(area);
        
        // 渲染头部导航
        self.header_navigation.set_focus(
            self.focus_manager.current_focus() == FocusPanel::Header
        );
        self.header_navigation.render(frame, layout.header, &state)?;
        
        // 渲染侧边栏
        self.sidebar_panel.set_focus(
            self.focus_manager.current_focus() == FocusPanel::Sidebar
        );
        self.sidebar_panel.render(frame, layout.sidebar, &state)?;
        
        // 根据当前视图渲染主内容区域
        self.render_main_content(frame, layout.content, &state)?;
        
        // 渲染状态栏
        self.render_status_bar(frame, layout.status_bar, &state)?;
        
        Ok(())
    }
    
    fn render_main_content(&mut self, frame: &mut Frame, area: Rect, state: &AppState) -> Result<()> {
        let is_focused = self.focus_manager.current_focus() == FocusPanel::Content;
        
        match state.current_view {
            ViewType::Branches => {
                self.branches_view.set_focus(is_focused);
                self.branches_view.render(frame, area, state)?;
            }
            ViewType::Tags => {
                self.tags_view.set_focus(is_focused);
                self.tags_view.render(frame, area, state)?;
            }
            ViewType::Stash => {
                // 特殊处理：Stash 视图包含 Git Log 切换
                self.render_stash_with_gitlog_toggle(frame, area, state, is_focused)?;
            }
            ViewType::Remotes => {
                self.remotes_view.set_focus(is_focused);
                self.remotes_view.render(frame, area, state)?;
            }
            ViewType::QueryHistory => {
                self.query_history_view.set_focus(is_focused);
                self.query_history_view.render(frame, area, state)?;
            }
        }
        
        Ok(())
    }
    
    fn render_stash_with_gitlog_toggle(
        &mut self, 
        frame: &mut Frame, 
        area: Rect, 
        state: &AppState, 
        is_focused: bool
    ) -> Result<()> {
        // 创建内部切换逻辑
        let toggle_height = 3;
        let content_height = area.height.saturating_sub(toggle_height);
        
        let toggle_area = Rect { height: toggle_height, ..area };
        let content_area = Rect { 
            y: area.y + toggle_height, 
            height: content_height, 
            ..area 
        };
        
        // 渲染切换标签栏
        self.render_stash_mode_toggle(frame, toggle_area, state)?;
        
        // 根据当前模式渲染内容
        match state.git_log_stash_mode {
            GitLogStashMode::GitLog => {
                self.git_log_view.set_focus(is_focused);
                self.git_log_view.render(frame, content_area, state)?;
            }
            GitLogStashMode::StashEntries => {
                self.stash_view.set_focus(is_focused);
                self.stash_view.render(frame, content_area, state)?;
            }
        }
        
        Ok(())
    }
    
    async fn handle_key_event(&mut self, key: KeyEvent) -> Result<bool> {
        let mut state = self.state.write().await;
        
        // 全局快捷键处理
        match key.code {
            KeyCode::Char('q') => return Ok(true), // 退出
            KeyCode::Tab => {
                // Tab 键切换焦点
                self.focus_manager.next_panel();
                return Ok(false);
            }
            KeyCode::Char('g') if state.current_view == ViewType::Stash => {
                // Stash 视图中的 'g' 键切换到 Git Log 模式
                state.git_log_stash_mode = GitLogStashMode::GitLog;
                return Ok(false);
            }
            KeyCode::Char('s') if state.current_view == ViewType::Stash => {
                // Stash 视图中的 's' 键切换到 Stash Entries 模式
                state.git_log_stash_mode = GitLogStashMode::StashEntries;
                return Ok(false);
            }
            _ => {}
        }
        
        // 将按键事件路由到当前焦点组件
        let event_result = match self.focus_manager.current_focus() {
            FocusPanel::Header => self.header_navigation.handle_key_event(key, &mut state),
            FocusPanel::Sidebar => self.sidebar_panel.handle_key_event(key, &mut state),
            FocusPanel::Content => self.handle_content_key_event(key, &mut state),
        };
        
        match event_result {
            EventResult::Handled => Ok(false),
            EventResult::NotHandled => Ok(false), // 忽略未处理的按键
        }
    }
}
```

## Phase 5: 状态管理更新

### 5.1 更新 AppState

**文件：** `src/tui_unified/state/app_state.rs`

```rust
// 新增枚举
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GitLogStashMode {
    GitLog,
    StashEntries,
}

pub struct AppState {
    // 现有字段...
    
    // 新增字段
    pub git_log_stash_mode: GitLogStashMode,
}

impl AppState {
    pub async fn new(config: &AppConfig) -> Result<Self> {
        // 现有初始化逻辑...
        
        Ok(Self {
            // 现有字段初始化...
            git_log_stash_mode: GitLogStashMode::GitLog, // 默认显示 Git Log
        })
    }
    
    // 新增方法
    pub fn toggle_git_log_stash_mode(&mut self) {
        self.git_log_stash_mode = match self.git_log_stash_mode {
            GitLogStashMode::GitLog => GitLogStashMode::StashEntries,
            GitLogStashMode::StashEntries => GitLogStashMode::GitLog,
        };
    }
    
    pub fn set_git_log_stash_mode(&mut self, mode: GitLogStashMode) {
        self.git_log_stash_mode = mode;
    }
}
```

## 测试计划

### 单元测试
1. `HeaderNavigation` 组件的导航逻辑
2. `SidebarPanel` 的列表切换逻辑  
3. 布局管理器的尺寸计算
4. 状态管理的视图切换

### 集成测试
1. 头部导航与侧边栏列表的联动
2. 焦点管理的正确切换
3. Stash 视图的双模式切换
4. 键盘快捷键的全面功能

### 用户体验测试
1. 不同终端尺寸下的响应式布局
2. 键盘导航的流畅性
3. 视图切换的性能表现
4. 错误状态的优雅处理

## 实现优先级

1. **高优先级**
   - HeaderNavigation 组件创建
   - 焦点管理器更新
   - 基础布局结构调整

2. **中优先级** 
   - SidebarPanel 重构
   - 主内容区域动态渲染
   - Stash 双模式切换

3. **低优先级**
   - 动画效果优化
   - 高级键盘快捷键
   - 主题和样式精细调整

## Phase 7: Diff Viewer 增强实现

### 7.1 更新 DiffViewer 组件支持多种显示模式

**文件位置：** `src/tui_unified/components/widgets/diff_viewer.rs`

```rust
use crossterm::event::KeyEvent;
use ratatui::{
    Frame, 
    layout::{Rect, Layout, Direction, Constraint}, 
    widgets::{Block, Borders, Paragraph, List, ListItem}, 
    style::{Color, Style, Modifier},
    text::{Spans, Span}
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DiffViewMode {
    Unified,      // 传统统一格式，+/- 标记
    SideBySide,   // 左右对比显示
    Split,        // 上下分割显示
}

pub struct DiffViewer {
    visible: bool,
    view_mode: DiffViewMode,
    
    // Diff 数据
    commit_info: Option<CommitInfo>,
    files: Vec<DiffFile>,
    current_file_index: usize,
    scroll_position: usize,
    horizontal_scroll: usize,
    
    // 搜索功能
    search_mode: bool,
    search_query: String,
    search_results: Vec<usize>, // 匹配行号
    current_search_index: usize,
    
    // 显示选项
    show_whitespace: bool,
    show_tab_chars: bool,
    
    // Side-by-Side 模式特定
    left_scroll: usize,
    right_scroll: usize,
    focused_panel: DiffPanel, // Left or Right
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DiffPanel {
    Left,   // Original
    Right,  // Modified
}

#[derive(Debug, Clone)]
pub struct DiffFile {
    pub path: String,
    pub old_path: Option<String>,
    pub status: FileStatus,
    pub additions: usize,
    pub deletions: usize,
    pub hunks: Vec<DiffHunk>,
    pub is_binary: bool,
    pub old_mode: Option<String>,
    pub new_mode: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FileStatus {
    Added,
    Deleted,
    Modified,
    Renamed,
    Copied,
}

#[derive(Debug, Clone)]
pub struct DiffHunk {
    pub old_start: usize,
    pub old_count: usize,
    pub new_start: usize,
    pub new_count: usize,
    pub lines: Vec<DiffLine>,
}

#[derive(Debug, Clone)]
pub struct DiffLine {
    pub line_type: DiffLineType,
    pub content: String,
    pub old_line_no: Option<usize>,
    pub new_line_no: Option<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DiffLineType {
    Context,
    Addition,
    Deletion,
    NoNewline,
}

impl DiffViewer {
    pub fn new() -> Self {
        Self {
            visible: false,
            view_mode: DiffViewMode::Unified,
            commit_info: None,
            files: Vec::new(),
            current_file_index: 0,
            scroll_position: 0,
            horizontal_scroll: 0,
            search_mode: false,
            search_query: String::new(),
            search_results: Vec::new(),
            current_search_index: 0,
            show_whitespace: false,
            show_tab_chars: false,
            left_scroll: 0,
            right_scroll: 0,
            focused_panel: DiffPanel::Left,
        }
    }
    
    pub fn show_diff(&mut self, commit: CommitInfo, files: Vec<DiffFile>) {
        self.commit_info = Some(commit);
        self.files = files;
        self.current_file_index = 0;
        self.scroll_position = 0;
        self.horizontal_scroll = 0;
        self.visible = true;
    }
    
    pub fn close(&mut self) {
        self.visible = false;
        self.commit_info = None;
        self.files.clear();
        self.search_mode = false;
        self.search_query.clear();
        self.search_results.clear();
    }
    
    pub fn is_visible(&self) -> bool {
        self.visible
    }
    
    fn get_current_file(&self) -> Option<&DiffFile> {
        self.files.get(self.current_file_index)
    }
    
    fn switch_view_mode(&mut self, mode: DiffViewMode) {
        self.view_mode = mode;
        // 重置滚动位置和焦点
        self.scroll_position = 0;
        self.horizontal_scroll = 0;
        self.left_scroll = 0;
        self.right_scroll = 0;
        self.focused_panel = DiffPanel::Left;
    }
    
    fn next_file(&mut self) {
        if self.current_file_index < self.files.len().saturating_sub(1) {
            self.current_file_index += 1;
            self.scroll_position = 0;
            self.horizontal_scroll = 0;
        }
    }
    
    fn prev_file(&mut self) {
        if self.current_file_index > 0 {
            self.current_file_index -= 1;
            self.scroll_position = 0;
            self.horizontal_scroll = 0;
        }
    }
    
    fn search_in_diff(&mut self, query: &str) {
        self.search_results.clear();
        
        if let Some(file) = self.get_current_file() {
            for (hunk_idx, hunk) in file.hunks.iter().enumerate() {
                for (line_idx, line) in hunk.lines.iter().enumerate() {
                    if line.content.contains(query) {
                        self.search_results.push(hunk_idx * 1000 + line_idx); // 简化的行号计算
                    }
                }
            }
        }
        
        self.current_search_index = 0;
        if !self.search_results.is_empty() {
            self.jump_to_search_result(0);
        }
    }
    
    fn jump_to_search_result(&mut self, index: usize) {
        if let Some(&result) = self.search_results.get(index) {
            self.scroll_position = result / 1000; // 简化的滚动计算
            self.current_search_index = index;
        }
    }
    
    fn process_whitespace_display(&self, content: &str) -> String {
        let mut result = content.to_string();
        
        if self.show_whitespace {
            result = result.replace(' ', "·");
        }
        
        if self.show_tab_chars {
            result = result.replace('\t', "→   ");
        }
        
        result
    }
}

impl Component for DiffViewer {
    fn name(&self) -> &str {
        "DiffViewer"
    }
    
    fn render(&mut self, frame: &mut Frame, area: Rect, _state: &AppState) {
        if !self.visible {
            return;
        }
        
        // 清除背景
        frame.render_widget(
            Block::default().style(Style::default().bg(Color::Black)), 
            area
        );
        
        match self.view_mode {
            DiffViewMode::Unified => self.render_unified_view(frame, area),
            DiffViewMode::SideBySide => self.render_side_by_side_view(frame, area),
            DiffViewMode::Split => self.render_split_view(frame, area),
        }
    }
    
    fn render_unified_view(&mut self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // 头部信息
                Constraint::Min(10),    // 主内容
                Constraint::Length(3),  // 控制信息
            ])
            .split(area);
        
        // 渲染头部信息
        self.render_header(frame, chunks[0]);
        
        // 渲染文件头和 diff 内容
        if let Some(file) = self.get_current_file() {
            let content_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(1),  // 文件路径
                    Constraint::Min(5),     // Diff 内容
                ])
                .split(chunks[1]);
            
            // 文件路径
            let file_path = format!("▼ {}: {}", 
                match file.status {
                    FileStatus::Added => "Added",
                    FileStatus::Deleted => "Deleted", 
                    FileStatus::Modified => "Modified",
                    FileStatus::Renamed => "Renamed",
                    FileStatus::Copied => "Copied",
                },
                file.path
            );
            
            frame.render_widget(
                Paragraph::new(file_path)
                    .style(Style::default().fg(Color::Cyan)),
                content_chunks[0]
            );
            
            // Diff 内容
            self.render_unified_diff_content(frame, content_chunks[1], file);
        }
        
        // 渲染控制栏
        self.render_controls(frame, chunks[2]);
    }
    
    fn render_side_by_side_view(&mut self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // 头部信息
                Constraint::Min(10),    // 主内容
                Constraint::Length(3),  // 控制信息
            ])
            .split(area);
        
        // 渲染头部信息
        self.render_header(frame, chunks[0]);
        
        // 主内容区域：左右分割
        if let Some(file) = self.get_current_file() {
            let content_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(1),  // 文件路径行
                    Constraint::Min(5),     // 分割内容
                ])
                .split(chunks[1]);
            
            // 文件路径（横跨两列）
            let file_header = format!(
                "▼ Original: {} ──┬── ▼ Modified: {}", 
                file.old_path.as_ref().unwrap_or(&file.path),
                file.path
            );
            frame.render_widget(
                Paragraph::new(file_header)
                    .style(Style::default().fg(Color::Cyan)),
                content_chunks[0]
            );
            
            // 左右分割的 diff 内容
            let side_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(50),  // 左侧原始内容
                    Constraint::Percentage(50),  // 右侧修改内容
                ])
                .split(content_chunks[1]);
            
            self.render_side_by_side_content(frame, side_chunks, file);
        }
        
        // 渲染控制栏
        self.render_controls(frame, chunks[2]);
    }
    
    fn render_split_view(&mut self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // 头部信息
                Constraint::Length(1),  // 文件路径
                Constraint::Min(8),     // 分割内容
                Constraint::Length(3),  // 控制信息
            ])
            .split(area);
        
        // 渲染头部信息
        self.render_header(frame, chunks[0]);
        
        if let Some(file) = self.get_current_file() {
            // 文件路径
            let file_path = format!("▼ File: {}", file.path);
            frame.render_widget(
                Paragraph::new(file_path)
                    .style(Style::default().fg(Color::Cyan)),
                chunks[1]
            );
            
            // 上下分割的内容
            let split_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(50),  // 上半部分：原始内容
                    Constraint::Percentage(50),  // 下半部分：修改内容
                ])
                .split(chunks[2]);
            
            self.render_split_content(frame, split_chunks, file);
        }
        
        // 渲染控制栏
        self.render_controls(frame, chunks[3]);
    }
    
    fn render_header(&self, frame: &mut Frame, area: Rect) {
        if let Some(ref commit) = self.commit_info {
            let header_text = format!(
                "Commit: {} | Files: {} | Mode: {} ({})",
                &commit.hash[..8],
                self.files.len(),
                match self.view_mode {
                    DiffViewMode::Unified => "Unified",
                    DiffViewMode::SideBySide => "Side-by-Side", 
                    DiffViewMode::Split => "Split",
                },
                match self.view_mode {
                    DiffViewMode::Unified => "1",
                    DiffViewMode::SideBySide => "2",
                    DiffViewMode::Split => "3",
                }
            );
            
            frame.render_widget(
                Paragraph::new(header_text)
                    .block(Block::default()
                        .title("Commit Info")
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::Yellow))),
                area
            );
        }
    }
    
    fn render_controls(&self, frame: &mut Frame, area: Rect) {
        let controls_text = format!(
            "File {}/{} | Scroll: {} | View Mode: {} | Keys: 1-Unified 2-Side-by-Side 3-Split q-Close",
            self.current_file_index + 1,
            self.files.len(),
            self.scroll_position,
            match self.view_mode {
                DiffViewMode::Unified => "Unified",
                DiffViewMode::SideBySide => "Side-by-Side",
                DiffViewMode::Split => "Split",
            }
        );
        
        frame.render_widget(
            Paragraph::new(controls_text)
                .block(Block::default()
                    .title("Controls")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan))),
            area
        );
    }
    
    // 实现具体的内容渲染方法
    fn render_unified_diff_content(&self, frame: &mut Frame, area: Rect, file: &DiffFile) {
        // 实现 Unified 模式的 diff 内容渲染
        // 显示 @@ 头部和 +/- 标记的行
    }
    
    fn render_side_by_side_content(&self, frame: &mut Frame, areas: [Rect; 2], file: &DiffFile) {
        // 实现 Side-by-Side 模式的内容渲染
        // 左侧显示原始内容，右侧显示修改内容
    }
    
    fn render_split_content(&self, frame: &mut Frame, areas: [Rect; 2], file: &DiffFile) {
        // 实现 Split 模式的内容渲染
        // 上半部分显示原始内容，下半部分显示修改内容
    }
    
    fn handle_key_event(&mut self, key: KeyEvent, _state: &mut AppState) -> EventResult {
        use crossterm::event::KeyCode;
        
        if self.search_mode {
            return self.handle_search_key_event(key);
        }
        
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.close();
                EventResult::Handled
            }
            KeyCode::Char('1') => {
                self.switch_view_mode(DiffViewMode::Unified);
                EventResult::Handled
            }
            KeyCode::Char('2') => {
                self.switch_view_mode(DiffViewMode::SideBySide);
                EventResult::Handled
            }
            KeyCode::Char('3') => {
                self.switch_view_mode(DiffViewMode::Split);
                EventResult::Handled
            }
            KeyCode::Char('n') => {
                self.next_file();
                EventResult::Handled
            }
            KeyCode::Char('p') => {
                self.prev_file();
                EventResult::Handled
            }
            KeyCode::Char('/') => {
                self.search_mode = true;
                self.search_query.clear();
                EventResult::Handled
            }
            KeyCode::Char('w') => {
                self.show_whitespace = !self.show_whitespace;
                EventResult::Handled
            }
            KeyCode::Char('t') => {
                self.show_tab_chars = !self.show_tab_chars;
                EventResult::Handled
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.scroll_position > 0 {
                    self.scroll_position -= 1;
                }
                EventResult::Handled
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.scroll_position += 1;
                EventResult::Handled
            }
            KeyCode::PageUp | KeyCode::Char('u') => {
                self.scroll_position = self.scroll_position.saturating_sub(10);
                EventResult::Handled
            }
            KeyCode::PageDown | KeyCode::Char('d') => {
                self.scroll_position += 10;
                EventResult::Handled
            }
            KeyCode::Home | KeyCode::Char('g') => {
                self.scroll_position = 0;
                EventResult::Handled
            }
            KeyCode::Left | KeyCode::Char('h') => {
                match self.view_mode {
                    DiffViewMode::SideBySide => {
                        self.focused_panel = DiffPanel::Left;
                    }
                    _ => {
                        self.horizontal_scroll = self.horizontal_scroll.saturating_sub(4);
                    }
                }
                EventResult::Handled
            }
            KeyCode::Right | KeyCode::Char('l') => {
                match self.view_mode {
                    DiffViewMode::SideBySide => {
                        self.focused_panel = DiffPanel::Right;
                    }
                    _ => {
                        self.horizontal_scroll += 4;
                    }
                }
                EventResult::Handled
            }
            _ => EventResult::NotHandled
        }
    }
    
    fn handle_search_key_event(&mut self, key: KeyEvent) -> EventResult {
        use crossterm::event::KeyCode;
        
        match key.code {
            KeyCode::Enter => {
                self.search_in_diff(&self.search_query);
                self.search_mode = false;
                EventResult::Handled
            }
            KeyCode::Esc => {
                self.search_mode = false;
                self.search_query.clear();
                EventResult::Handled
            }
            KeyCode::Char(c) => {
                self.search_query.push(c);
                EventResult::Handled
            }
            KeyCode::Backspace => {
                self.search_query.pop();
                EventResult::Handled
            }
            _ => EventResult::NotHandled
        }
    }
}
```

### 7.2 更新 AppState 支持 Diff Viewer

**文件：** `src/tui_unified/state/app_state.rs`

```rust
impl AppState {
    pub fn show_commit_diff(&mut self, commit: &CommitInfo) {
        // 异步加载 commit 的 diff 数据
        self.diff_viewer_state = Some(DiffViewerState {
            commit: commit.clone(),
            loading: true,
        });
    }
    
    pub fn set_diff_data(&mut self, files: Vec<DiffFile>) {
        if let Some(ref mut state) = self.diff_viewer_state {
            state.loading = false;
            state.files = files;
        }
    }
    
    pub fn is_diff_viewer_active(&self) -> bool {
        self.diff_viewer_state.as_ref().map(|s| !s.loading).unwrap_or(false)
    }
}

#[derive(Debug, Clone)]
pub struct DiffViewerState {
    pub commit: CommitInfo,
    pub loading: bool,
    pub files: Vec<DiffFile>,
}
```