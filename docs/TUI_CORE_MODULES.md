# TUI核心模块实现设计

## 模块文件结构设计

基于技术设计文档，以下是推荐的文件结构：

```
src/
├── tui_unified/
│   ├── mod.rs                    # 统一TUI模块入口
│   ├── app.rs                    # 主应用程序结构
│   ├── layout/
│   │   ├── mod.rs                # 布局管理器
│   │   ├── manager.rs            # 布局管理实现
│   │   └── modes.rs              # 布局模式定义
│   ├── focus/
│   │   ├── mod.rs                # 焦点管理系统
│   │   ├── manager.rs            # 焦点管理实现
│   │   └── ring.rs               # 焦点环管理
│   ├── state/
│   │   ├── mod.rs                # 状态管理系统
│   │   ├── app_state.rs          # 应用状态定义
│   │   ├── git_state.rs          # Git相关状态
│   │   └── ui_state.rs           # UI相关状态
│   ├── components/
│   │   ├── mod.rs                # 组件系统入口
│   │   ├── base/
│   │   │   ├── mod.rs            # 基础组件
│   │   │   ├── component.rs      # 组件trait定义
│   │   │   └── events.rs         # 事件处理
│   │   ├── panels/
│   │   │   ├── mod.rs            # 面板组件
│   │   │   ├── sidebar.rs        # 侧边栏面板
│   │   │   ├── content.rs        # 主内容面板
│   │   │   └── detail.rs         # 详情面板
│   │   ├── views/
│   │   │   ├── mod.rs            # 视图组件
│   │   │   ├── git_log.rs        # Git日志视图
│   │   │   ├── branches.rs       # 分支视图
│   │   │   ├── tags.rs           # 标签视图
│   │   │   ├── remotes.rs        # 远程仓库视图
│   │   │   ├── stash.rs          # Stash视图
│   │   │   └── query_history.rs  # 查询历史视图
│   │   ├── widgets/
│   │   │   ├── mod.rs            # 自定义组件
│   │   │   ├── diff_viewer.rs    # Diff查看器
│   │   │   ├── status_bar.rs     # 状态栏
│   │   │   ├── help_panel.rs     # 帮助面板
│   │   │   ├── search_box.rs     # 搜索框
│   │   │   └── progress_bar.rs   # 进度条
│   │   └── smart/
│   │       ├── mod.rs            # 智能组件
│   │       ├── branch_ops.rs     # 智能分支操作
│   │       ├── merge_assistant.rs # 合并助手
│   │       └── conflict_resolver.rs # 冲突解决器
│   ├── events/
│   │   ├── mod.rs                # 事件系统
│   │   ├── handler.rs            # 事件处理器
│   │   ├── router.rs             # 事件路由
│   │   └── types.rs              # 事件类型定义
│   ├── git/
│   │   ├── mod.rs                # Git集成模块
│   │   ├── interface.rs          # Git接口定义
│   │   ├── async_impl.rs         # 异步Git实现
│   │   ├── commands.rs           # Git命令封装
│   │   ├── parser.rs             # Git输出解析
│   │   └── models.rs             # Git数据模型
│   ├── cache/
│   │   ├── mod.rs                # 缓存系统
│   │   ├── manager.rs            # 缓存管理器
│   │   ├── git_cache.rs          # Git缓存
│   │   ├── ui_cache.rs           # UI缓存
│   │   └── file_cache.rs         # 文件缓存
│   ├── algorithms/
│   │   ├── mod.rs                # 算法模块
│   │   ├── smart_branch.rs       # 智能分支算法
│   │   ├── virtual_scroll.rs     # 虚拟滚动算法
│   │   ├── smart_search.rs       # 智能搜索算法
│   │   └── diff_algorithm.rs     # Diff算法
│   ├── async_manager/
│   │   ├── mod.rs                # 异步任务管理
│   │   ├── task_manager.rs       # 任务管理器
│   │   ├── executor.rs           # 任务执行器
│   │   └── event_bus.rs          # 事件总线
│   ├── config/
│   │   ├── mod.rs                # 配置管理
│   │   ├── app_config.rs         # 应用配置
│   │   ├── key_bindings.rs       # 按键绑定
│   │   └── themes.rs             # 主题配置
│   └── utils/
│       ├── mod.rs                # 工具函数
│       ├── terminal.rs           # 终端工具
│       ├── formatting.rs        # 格式化工具
│       └── validation.rs        # 验证工具
```

## 核心模块详细设计

### 1. 主应用程序 (app.rs)

```rust
use std::sync::Arc;
use tokio::sync::RwLock;
use crossterm::event::{Event, KeyEvent};
use ratatui::{Frame, Terminal};

pub struct TuiUnifiedApp {
    // 核心状态
    state: Arc<RwLock<AppState>>,
    
    // 管理器
    layout_manager: LayoutManager,
    focus_manager: FocusManager,
    event_router: EventRouter,
    async_task_manager: AsyncTaskManager,
    cache_manager: CacheManager,
    
    // 组件
    sidebar: SidebarPanel,
    content: ContentPanel,
    detail: DetailPanel,
    status_bar: StatusBarComponent,
    
    // 配置
    config: AppConfig,
    key_bindings: KeyBindings,
    
    // 运行状态
    should_quit: bool,
    current_mode: AppMode,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AppMode {
    Normal,      // 正常浏览模式
    Search,      // 搜索模式
    Command,     // 命令模式
    Help,        // 帮助模式
    Diff,        // 全屏diff模式
}

impl TuiUnifiedApp {
    pub async fn new() -> Result<Self> {
        let config = AppConfig::load()?;
        let state = Arc::new(RwLock::new(AppState::new(&config).await?));
        
        Ok(Self {
            state: Arc::clone(&state),
            layout_manager: LayoutManager::new(&config),
            focus_manager: FocusManager::new(),
            event_router: EventRouter::new(),
            async_task_manager: AsyncTaskManager::new(),
            cache_manager: CacheManager::new(config.cache_size),
            sidebar: SidebarPanel::new(),
            content: ContentPanel::new(),
            detail: DetailPanel::new(),
            status_bar: StatusBarComponent::new(),
            config,
            key_bindings: KeyBindings::default(),
            should_quit: false,
            current_mode: AppMode::Normal,
        })
    }
    
    pub async fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        // 初始化Git数据
        self.load_initial_git_data().await?;
        
        // 主事件循环
        loop {
            // 渲染UI
            terminal.draw(|f| self.render(f))?;
            
            // 处理事件
            if let Event::Key(key) = crossterm::event::read()? {
                self.handle_key_event(key).await?;
            }
            
            // 检查退出条件
            if self.should_quit {
                break;
            }
        }
        
        Ok(())
    }
    
    fn render(&mut self, frame: &mut Frame) {
        let state = self.state.try_read().unwrap();
        let layout = self.layout_manager.calculate_layout(frame.size());
        
        // 渲染主面板
        match self.layout_manager.mode {
            LayoutMode::Normal => {
                self.sidebar.render(frame, layout.sidebar, &state);
                self.content.render(frame, layout.content, &state);
                self.detail.render(frame, layout.detail, &state);
            }
            LayoutMode::FullScreen => {
                // 全屏diff模式
                self.detail.render_full_screen(frame, frame.size(), &state);
            }
            LayoutMode::SplitHorizontal | LayoutMode::SplitVertical => {
                // 分屏模式处理
                self.render_split_mode(frame, layout, &state);
            }
        }
        
        // 渲染状态栏
        self.status_bar.render(frame, layout.status_bar, &state);
        
        // 渲染模态框（如果有）
        if let Some(modal) = &state.modal {
            self.render_modal(frame, modal);
        }
    }
    
    async fn handle_key_event(&mut self, key: KeyEvent) -> Result<()> {
        let mut state = self.state.write().await;
        
        // 全局快捷键处理
        if let Some(action) = self.key_bindings.get_global_action(&key) {
            match action {
                GlobalAction::Quit => self.should_quit = true,
                GlobalAction::NextPanel => self.focus_manager.next_focus(),
                GlobalAction::PrevPanel => self.focus_manager.prev_focus(),
                GlobalAction::ToggleHelp => self.toggle_help_mode(),
                GlobalAction::Refresh => self.refresh_all_data().await?,
                _ => {}
            }
            return Ok(());
        }
        
        // 模式特定处理
        match self.current_mode {
            AppMode::Normal => self.handle_normal_mode_key(key, &mut state).await?,
            AppMode::Search => self.handle_search_mode_key(key, &mut state).await?,
            AppMode::Command => self.handle_command_mode_key(key, &mut state).await?,
            AppMode::Help => self.handle_help_mode_key(key, &mut state).await?,
            AppMode::Diff => self.handle_diff_mode_key(key, &mut state).await?,
        }
        
        Ok(())
    }
}
```

### 2. 状态管理 (state/app_state.rs)

```rust
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct AppState {
    // UI状态
    pub layout: LayoutState,
    pub focus: FocusState,
    pub current_view: ViewType,
    pub modal: Option<ModalState>,
    
    // Git数据状态
    pub repo_state: GitRepoState,
    pub selected_items: SelectionState,
    pub search_state: SearchState,
    
    // 配置状态
    pub config: AppConfig,
    pub theme: Theme,
    
    // 运行时状态
    pub loading_tasks: HashMap<TaskId, LoadingTask>,
    pub error_messages: Vec<ErrorMessage>,
    pub notifications: Vec<Notification>,
}

#[derive(Debug, Clone)]
pub struct LayoutState {
    pub mode: LayoutMode,
    pub sidebar_width: u16,
    pub content_width: u16,
    pub detail_width: u16,
    pub detail_split_ratio: (u16, u16),
}

#[derive(Debug, Clone)]
pub struct FocusState {
    pub current_panel: FocusPanel,
    pub panel_history: Vec<FocusPanel>,
    pub can_navigate: bool,
}

#[derive(Debug, Clone)]
pub struct GitRepoState {
    pub current_branch: String,
    pub repo_path: PathBuf,
    pub commits: Vec<Commit>,
    pub branches: Vec<Branch>,
    pub tags: Vec<Tag>,
    pub remotes: Vec<Remote>,
    pub stash_list: Vec<Stash>,
    pub query_history: Vec<QueryEntry>,
    pub repo_status: RepoStatus,
}

#[derive(Debug, Clone)]
pub struct SelectionState {
    pub selected_commit: Option<CommitId>,
    pub selected_branch: Option<String>,
    pub selected_tag: Option<String>,
    pub selected_remote: Option<String>,
    pub selected_stash: Option<usize>,
    pub multi_selection: Vec<SelectableItem>,
}

#[derive(Debug, Clone)]
pub struct SearchState {
    pub query: String,
    pub filters: SearchFilters,
    pub results: Vec<SearchResult>,
    pub is_active: bool,
    pub search_history: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchFilters {
    pub author: Option<String>,
    pub date_range: Option<DateRange>,
    pub file_path: Option<PathBuf>,
    pub message_pattern: Option<String>,
    pub branch: Option<String>,
}

impl AppState {
    pub async fn new(config: &AppConfig) -> Result<Self> {
        let repo_path = std::env::current_dir()?;
        let repo_state = GitRepoState::load(&repo_path).await?;
        
        Ok(Self {
            layout: LayoutState::default(),
            focus: FocusState::new(),
            current_view: ViewType::GitLog,
            modal: None,
            repo_state,
            selected_items: SelectionState::default(),
            search_state: SearchState::default(),
            config: config.clone(),
            theme: Theme::load(&config.theme_name)?,
            loading_tasks: HashMap::new(),
            error_messages: Vec::new(),
            notifications: Vec::new(),
        })
    }
    
    // 状态更新方法
    pub fn set_current_view(&mut self, view: ViewType) {
        self.current_view = view;
        self.clear_selections();
    }
    
    pub fn select_commit(&mut self, commit_id: CommitId) {
        self.selected_items.selected_commit = Some(commit_id);
        // 触发详情面板更新
        self.request_detail_update();
    }
    
    pub fn add_loading_task(&mut self, task_id: TaskId, task: LoadingTask) {
        self.loading_tasks.insert(task_id, task);
    }
    
    pub fn remove_loading_task(&mut self, task_id: TaskId) {
        self.loading_tasks.remove(&task_id);
    }
    
    pub fn add_notification(&mut self, message: String, level: NotificationLevel) {
        self.notifications.push(Notification {
            message,
            level,
            timestamp: std::time::Instant::now(),
        });
        
        // 限制通知数量
        if self.notifications.len() > 10 {
            self.notifications.remove(0);
        }
    }
    
    // 辅助方法
    pub fn is_loading(&self) -> bool {
        !self.loading_tasks.is_empty()
    }
    
    pub fn get_current_selection(&self) -> Option<SelectableItem> {
        match self.current_view {
            ViewType::GitLog => self.selected_items.selected_commit.map(SelectableItem::Commit),
            ViewType::Branches => self.selected_items.selected_branch.clone().map(SelectableItem::Branch),
            ViewType::Tags => self.selected_items.selected_tag.clone().map(SelectableItem::Tag),
            ViewType::Remotes => self.selected_items.selected_remote.clone().map(SelectableItem::Remote),
            ViewType::Stash => self.selected_items.selected_stash.map(SelectableItem::Stash),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum SelectableItem {
    Commit(CommitId),
    Branch(String),
    Tag(String),
    Remote(String),
    Stash(usize),
    QueryEntry(usize),
}

#[derive(Debug, Clone)]
pub struct LoadingTask {
    pub name: String,
    pub progress: Option<f64>,
    pub message: String,
    pub started_at: std::time::Instant,
}

#[derive(Debug, Clone)]
pub struct Notification {
    pub message: String,
    pub level: NotificationLevel,
    pub timestamp: std::time::Instant,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NotificationLevel {
    Info,
    Warning,
    Error,
    Success,
}
```

### 3. 组件基础系统 (components/base/component.rs)

```rust
use async_trait::async_trait;
use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::{Frame, layout::Rect};

#[async_trait]
pub trait Component: Send + Sync {
    type Props: Clone;
    
    // 组件生命周期
    fn new(props: Self::Props) -> Self where Self: Sized;
    async fn mount(&mut self, state: &AppState);
    async fn unmount(&mut self);
    
    // 渲染系统
    fn render(&mut self, frame: &mut Frame, area: Rect, state: &AppState);
    fn get_cursor_position(&self) -> Option<(u16, u16)> { None }
    
    // 事件处理
    async fn handle_key_event(&mut self, key: KeyEvent, state: &mut AppState) -> EventResult;
    async fn handle_mouse_event(&mut self, mouse: MouseEvent, state: &mut AppState) -> EventResult {
        EventResult::NotHandled
    }
    async fn handle_custom_event(&mut self, event: CustomEvent, state: &mut AppState) -> EventResult {
        EventResult::NotHandled
    }
    
    // 数据更新
    fn should_update(&self, old_state: &AppState, new_state: &AppState) -> bool;
    async fn update(&mut self, state: &AppState);
    
    // 焦点管理
    fn can_focus(&self) -> bool { true }
    async fn on_focus(&mut self, state: &AppState) {}
    async fn on_blur(&mut self, state: &AppState) {}
    
    // 验证
    fn validate(&self) -> Vec<ValidationError> { Vec::new() }
}

#[derive(Debug, Clone)]
pub enum EventResult {
    Handled,
    NotHandled,
    Bubble,
    Navigate(Navigation),
    StateChange(StateChange),
    AsyncTask(AsyncTask),
}

#[derive(Debug, Clone)]
pub enum Navigation {
    NextPanel,
    PrevPanel,
    ToPanel(FocusPanel),
    ToView(ViewType),
    Back,
    Forward,
}

#[derive(Debug, Clone)]
pub enum StateChange {
    SelectItem(SelectableItem),
    SetView(ViewType),
    UpdateFilter(SearchFilters),
    SetMode(AppMode),
    ShowModal(ModalType),
    HideModal,
}

#[derive(Debug, Clone)]
pub enum AsyncTask {
    GitCommand(GitCommand),
    LoadData(DataLoadRequest),
    Search(SearchQuery),
    FileOperation(FileOperation),
}

#[derive(Debug, Clone)]
pub enum CustomEvent {
    GitDataUpdated(GitDataType),
    SearchCompleted(SearchResult),
    TaskProgress(TaskId, f64),
    TaskCompleted(TaskId),
    TaskFailed(TaskId, String),
    ConfigChanged(ConfigChange),
}

// 组件工厂
pub struct ComponentFactory;

impl ComponentFactory {
    pub fn create_sidebar() -> Box<dyn Component<Props = SidebarProps>> {
        Box::new(SidebarPanel::new(SidebarProps::default()))
    }
    
    pub fn create_content_panel() -> Box<dyn Component<Props = ContentProps>> {
        Box::new(ContentPanel::new(ContentProps::default()))
    }
    
    pub fn create_detail_panel() -> Box<dyn Component<Props = DetailProps>> {
        Box::new(DetailPanel::new(DetailProps::default()))
    }
    
    pub fn create_diff_viewer() -> Box<dyn Component<Props = DiffViewerProps>> {
        Box::new(DiffViewerComponent::new(DiffViewerProps::default()))
    }
}

// 组件注册系统
pub struct ComponentRegistry {
    components: HashMap<String, Box<dyn Component<Props = ()>>>,
    factories: HashMap<String, fn() -> Box<dyn Component<Props = ()>>>,
}

impl ComponentRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            components: HashMap::new(),
            factories: HashMap::new(),
        };
        
        // 注册内置组件
        registry.register_factory("sidebar", || ComponentFactory::create_sidebar());
        registry.register_factory("content", || ComponentFactory::create_content_panel());
        registry.register_factory("detail", || ComponentFactory::create_detail_panel());
        registry.register_factory("diff_viewer", || ComponentFactory::create_diff_viewer());
        
        registry
    }
    
    pub fn register_factory<F>(&mut self, name: &str, factory: F) 
    where
        F: Fn() -> Box<dyn Component<Props = ()>> + 'static,
    {
        self.factories.insert(name.to_string(), Box::new(factory));
    }
    
    pub fn create_component(&mut self, name: &str) -> Option<Box<dyn Component<Props = ()>>> {
        self.factories.get(name).map(|factory| factory())
    }
}
```

### 4. Git接口实现 (git/async_impl.rs)

```rust
use async_trait::async_trait;
use tokio::process::Command;
use std::process::Stdio;

pub struct AsyncGitImpl {
    repo_path: PathBuf,
    cache: Arc<RwLock<GitCache>>,
    command_timeout: Duration,
}

impl AsyncGitImpl {
    pub fn new(repo_path: PathBuf) -> Self {
        Self {
            repo_path,
            cache: Arc::new(RwLock::new(GitCache::new(1000))),
            command_timeout: Duration::from_secs(30),
        }
    }
    
    async fn execute_git_command(&self, args: &[&str]) -> Result<String> {
        let mut cmd = Command::new("git");
        cmd.args(args)
           .current_dir(&self.repo_path)
           .stdout(Stdio::piped())
           .stderr(Stdio::piped());
           
        let output = tokio::time::timeout(self.command_timeout, cmd.output()).await??;
        
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(GitError::CommandFailed(error.to_string()).into());
        }
        
        Ok(String::from_utf8(output.stdout)?)
    }
    
    async fn execute_git_command_with_cache(&self, cache_key: &str, args: &[&str], ttl: Duration) -> Result<String> {
        // 检查缓存
        if let Some(cached) = self.cache.read().await.get(cache_key) {
            return Ok(cached);
        }
        
        // 执行命令
        let result = self.execute_git_command(args).await?;
        
        // 缓存结果
        self.cache.write().await.set(cache_key.to_string(), result.clone(), ttl);
        
        Ok(result)
    }
}

#[async_trait]
impl GitInterface for AsyncGitImpl {
    async fn get_commits(&self, branch: Option<&str>, limit: Option<usize>) -> Result<Vec<Commit>> {
        let cache_key = format!("commits_{}_{}", branch.unwrap_or("HEAD"), limit.unwrap_or(100));
        
        let mut args = vec![
            "log",
            "--format=%H|%an|%ae|%at|%s|%b",
            "--date=unix"
        ];
        
        if let Some(branch) = branch {
            args.push(branch);
        }
        
        if let Some(limit) = limit {
            args.push(&format!("-{}", limit));
        }
        
        let output = self.execute_git_command_with_cache(&cache_key, &args, Duration::from_secs(60)).await?;
        let commits = self.parse_commit_output(&output)?;
        
        Ok(commits)
    }
    
    async fn get_branches(&self) -> Result<Vec<Branch>> {
        let args = ["branch", "-a", "--format=%(refname:short)|%(upstream)|%(ahead-behind)|%(committerdate:unix)"];
        let output = self.execute_git_command_with_cache("branches", &args, Duration::from_secs(30)).await?;
        let branches = self.parse_branch_output(&output)?;
        
        Ok(branches)
    }
    
    async fn checkout_branch(&self, branch: &str) -> Result<()> {
        let args = ["checkout", branch];
        self.execute_git_command(&args).await?;
        
        // 清理相关缓存
        self.cache.write().await.invalidate_pattern("commits_");
        self.cache.write().await.invalidate_pattern("branches");
        
        Ok(())
    }
    
    async fn get_commit_diff(&self, commit: &str) -> Result<String> {
        let cache_key = format!("diff_{}", commit);
        let args = ["show", "--format=", commit];
        let diff = self.execute_git_command_with_cache(&cache_key, &args, Duration::from_secs(300)).await?;
        
        Ok(diff)
    }
    
    async fn stash_save(&self, message: Option<&str>) -> Result<()> {
        let mut args = vec!["stash", "push"];
        if let Some(msg) = message {
            args.extend(&["-m", msg]);
        }
        
        self.execute_git_command(&args).await?;
        
        // 清理stash缓存
        self.cache.write().await.invalidate("stash_list");
        
        Ok(())
    }
}

impl AsyncGitImpl {
    fn parse_commit_output(&self, output: &str) -> Result<Vec<Commit>> {
        let mut commits = Vec::new();
        
        for line in output.lines() {
            if line.trim().is_empty() {
                continue;
            }
            
            let parts: Vec<&str> = line.split('|').collect();
            if parts.len() >= 5 {
                commits.push(Commit {
                    hash: parts[0].to_string(),
                    author: parts[1].to_string(),
                    email: parts[2].to_string(),
                    timestamp: parts[3].parse::<i64>().unwrap_or(0),
                    subject: parts[4].to_string(),
                    body: parts.get(5).unwrap_or(&"").to_string(),
                });
            }
        }
        
        Ok(commits)
    }
    
    fn parse_branch_output(&self, output: &str) -> Result<Vec<Branch>> {
        let mut branches = Vec::new();
        
        for line in output.lines() {
            if line.trim().is_empty() {
                continue;
            }
            
            let parts: Vec<&str> = line.split('|').collect();
            if parts.len() >= 4 {
                let ahead_behind = self.parse_ahead_behind(parts[2]);
                
                branches.push(Branch {
                    name: parts[0].to_string(),
                    upstream: if parts[1].is_empty() { None } else { Some(parts[1].to_string()) },
                    ahead: ahead_behind.0,
                    behind: ahead_behind.1,
                    last_commit_time: parts[3].parse::<i64>().unwrap_or(0),
                    is_current: false, // 需要额外查询当前分支
                });
            }
        }
        
        Ok(branches)
    }
    
    fn parse_ahead_behind(&self, input: &str) -> (usize, usize) {
        if input.is_empty() {
            return (0, 0);
        }
        
        let parts: Vec<&str> = input.split_whitespace().collect();
        if parts.len() == 2 {
            let ahead = parts[0].parse::<usize>().unwrap_or(0);
            let behind = parts[1].parse::<usize>().unwrap_or(0);
            (ahead, behind)
        } else {
            (0, 0)
        }
    }
}
```

这个核心模块设计文档提供了详细的文件结构、主要模块实现和关键接口定义，为TUI界面的实际开发提供了完整的技术指导。