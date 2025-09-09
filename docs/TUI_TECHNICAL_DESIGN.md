# TUI界面整合技术设计文档

## 概述

基于 `TUI_INTEGRATION_ANALYSIS.md` 的分析结果，本文档详细描述了统一TUI界面的技术架构、模块设计和实现方案。

## 技术架构设计

### 🏗️ 整体架构图

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                            TUI Unified Application                           │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌──────────────┐  ┌──────────────────┐  ┌──────────────────┐              │
│  │              │  │                  │  │                  │              │
│  │   Sidebar    │  │   Main Content   │  │   Detail Panel   │              │
│  │   Panel      │  │     Panel        │  │                  │              │
│  │   (20%)      │  │     (50%)        │  │     (30%)        │              │
│  │              │  │                  │  │                  │              │
│  │ ┌──────────┐ │  │ ┌──────────────┐ │  │ ┌──────────────┐ │              │
│  │ │Menu Items│ │  │ │Dynamic       │ │  │ │Info Panel   │ │              │
│  │ │  • Logs  │ │  │ │Content       │ │  │ │              │ │              │
│  │ │  • Branch│ │  │ │              │ │  │ │(40% height)  │ │              │
│  │ │  • Tags  │ │  │ │              │ │  │ └──────────────┘ │              │
│  │ │  • Remote│ │  │ │              │ │  │ ┌──────────────┐ │              │
│  │ │  • Stash │ │  │ │              │ │  │ │Diff Viewer   │ │              │
│  │ │  • Hist  │ │  │ │              │ │  │ │              │ │              │
│  │ └──────────┘ │  │ └──────────────┘ │  │ │(60% height)  │ │              │
│  └──────────────┘  └──────────────────┘  │ └──────────────┘ │              │
│                                          └──────────────────┘              │
├─────────────────────────────────────────────────────────────────────────────┤
│                       Smart Status Bar & Help System                        │
│  [Focus] [Operation] [KeyHints] [Selection] [Mode] [Git Status] [Help]      │
└─────────────────────────────────────────────────────────────────────────────┘

                              ↑ Layout Manager ↓
                              
┌─────────────────────────────────────────────────────────────────────────────┐
│                              Core Application Layer                          │
├─────────────────┬─────────────────┬─────────────────┬─────────────────────────┤
│   Event System  │  State Manager  │  Git Integration│    Component System     │
│                 │                 │                 │                         │
│ • Input Handler │ • Focus Manager │ • Git Commands  │ • Sidebar Components    │
│ • Key Bindings  │ • View Stack    │ • Data Caching  │ • Content Components    │
│ • Event Router  │ • Config State  │ • Async Ops     │ • Detail Components     │
│ • Hot Reload    │ • UI State      │ • Error Handle  │ • Diff Viewer           │
└─────────────────┴─────────────────┴─────────────────┴─────────────────────────┘

                              ↑ Data Layer ↓
                              
┌─────────────────────────────────────────────────────────────────────────────┐
│                                Data & Git Layer                             │
├─────────────────┬─────────────────┬─────────────────┬─────────────────────────┤
│   Git Commands  │   Data Models   │   Cache System  │       File System       │
│                 │                 │                 │                         │
│ • git log       │ • Commit        │ • LRU Cache     │ • Config Files          │
│ • git branch    │ • Branch        │ • Memory Cache  │ • Git Repository        │
│ • git diff      │ • Tag           │ • Command Cache │ • Stash Files           │
│ • git stash     │ • Remote        │ • Result Cache  │ • Temp Files            │
│ • git status    │ • Diff          │ • Smart Refresh │ • User Preferences      │
└─────────────────┴─────────────────┴─────────────────┴─────────────────────────┘
```

### 🧩 核心模块架构

#### 1. 布局管理系统 (Layout Manager)

```rust
// src/tui_unified/layout.rs
pub struct LayoutManager {
    pub sidebar_width: u16,      // 20% 
    pub content_width: u16,      // 50%
    pub detail_width: u16,       // 30%
    pub mode: LayoutMode,        // Normal | SplitHorizontal | SplitVertical | FullScreen
}

pub enum LayoutMode {
    Normal,           // 标准三栏布局
    SplitHorizontal,  // 水平分屏
    SplitVertical,    // 垂直分屏  
    FullScreen,       // 全屏diff模式
}
```

#### 2. 焦点管理系统 (Focus Manager)

```rust
// src/tui_unified/focus.rs
pub struct FocusManager {
    pub current_panel: FocusPanel,
    pub panel_history: Vec<FocusPanel>,
    pub focus_ring: [FocusPanel; 3],
}

pub enum FocusPanel {
    Sidebar,    // 侧边栏 (焦点0)
    Content,    // 主内容 (焦点1) 
    Detail,     // 详情区 (焦点2)
}

// 焦点切换逻辑: Tab键顺序切换, Shift+Tab反向切换
impl FocusManager {
    pub fn next_focus(&mut self) -> FocusPanel { /* Tab */ }
    pub fn prev_focus(&mut self) -> FocusPanel { /* Shift+Tab */ }
    pub fn direct_focus(&mut self, panel: FocusPanel) { /* 直接跳转 */ }
}
```

#### 3. 状态管理系统 (State Manager)

```rust
// src/tui_unified/state.rs
pub struct AppState {
    // UI状态
    pub layout: LayoutManager,
    pub focus: FocusManager,
    pub current_view: ViewType,
    
    // Git数据状态
    pub repo_state: GitRepoState,
    pub selected_items: SelectionState,
    
    // 配置状态
    pub config: AppConfig,
    pub key_bindings: KeyBindings,
}

pub enum ViewType {
    GitLog,
    Branches,
    Tags,
    Remotes,
    Stash,
    QueryHistory,
}
```

#### 4. 组件系统架构

```rust
// src/tui_unified/components/
pub trait Component {
    fn render(&mut self, frame: &mut Frame, area: Rect, state: &AppState);
    fn handle_event(&mut self, event: &KeyEvent, state: &mut AppState) -> bool;
    fn update_data(&mut self, state: &AppState);
}

// 主要组件
pub struct SidebarComponent;      // 左侧菜单
pub struct ContentComponent;      // 中间内容 
pub struct DetailComponent;       // 右侧详情
pub struct DiffViewerComponent;   // 专业diff查看器
pub struct StatusBarComponent;    // 状态栏
```

## 核心功能模块设计

### 📋 侧边栏面板 (Sidebar Panel)

```rust
// src/tui_unified/panels/sidebar.rs
pub struct SidebarPanel {
    pub menu_items: Vec<MenuItem>,
    pub selected_index: usize,
    pub expanded_items: HashSet<usize>,
}

pub struct MenuItem {
    pub name: String,
    pub icon: &'static str,
    pub view_type: ViewType,
    pub shortcut: Option<char>,
    pub children: Vec<MenuItem>,    // 支持子菜单
    pub badge: Option<String>,      // 状态标记 (如分支数量)
}

// 菜单项配置
let menu_items = vec![
    MenuItem::new("📋 Git Log", ViewType::GitLog, Some('1')),
    MenuItem::new("🌿 Branches", ViewType::Branches, Some('2')),
    MenuItem::new("🏷️ Tags", ViewType::Tags, Some('3')),
    MenuItem::new("🌐 Remotes", ViewType::Remotes, Some('4')),
    MenuItem::new("📦 Stash", ViewType::Stash, Some('5')),
    MenuItem::new("📝 Query History", ViewType::QueryHistory, Some('6')),
];
```

### 📊 主内容面板 (Main Content Panel)

```rust
// src/tui_unified/panels/content.rs
pub struct ContentPanel {
    pub current_view: ViewType,
    pub git_log_view: GitLogView,
    pub branches_view: BranchesView, 
    pub tags_view: TagsView,
    pub remotes_view: RemotesView,
    pub stash_view: StashView,
    pub history_view: QueryHistoryView,
}

// 动态内容切换
impl ContentPanel {
    pub fn render_current_view(&mut self, frame: &mut Frame, area: Rect, state: &AppState) {
        match state.current_view {
            ViewType::GitLog => self.git_log_view.render(frame, area, state),
            ViewType::Branches => self.branches_view.render(frame, area, state),
            ViewType::Tags => self.tags_view.render(frame, area, state),
            // ... 其他视图
        }
    }
}
```

### 🔍 详情面板 (Detail Panel)

```rust
// src/tui_unified/panels/detail.rs
pub struct DetailPanel {
    pub info_panel: InfoPanel,      // 上部40% - 信息展示
    pub diff_panel: DiffPanel,      // 下部60% - diff显示
    pub split_ratio: (u16, u16),    // (40, 60) 可调节
}

pub struct InfoPanel {
    pub content: InfoContent,
    pub scroll_offset: usize,
}

pub enum InfoContent {
    CommitInfo(CommitDetails),
    BranchInfo(BranchDetails),
    TagInfo(TagDetails),
    RemoteInfo(RemoteDetails),
    StashInfo(StashDetails),
}

pub struct DiffPanel {
    pub diff_content: String,
    pub syntax_highlighting: bool,
    pub view_mode: DiffViewMode,
}

pub enum DiffViewMode {
    Unified,        // 统一diff
    SideBySide,     // 并排对比
    TreeView,       // 文件树形视图
}
```

## 数据流程设计

### 🔄 应用启动流程

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   App Launch    │───▶│  Load Config     │───▶│  Init Git Repo  │
│                 │    │  - Keys Config   │    │  - Repo Check   │
│                 │    │  - Theme Config  │    │  - Branch List  │
│                 │    │  - Layout Prefs  │    │  - Initial Data │
└─────────────────┘    └──────────────────┘    └─────────────────┘
                                                        │
┌─────────────────┐    ┌──────────────────┐            ▼
│   Render Loop   │◀───│  Setup UI State  │    ┌─────────────────┐
│                 │    │  - Focus: Sidebar│    │  Setup Event    │
│                 │    │  - View: GitLog  │    │  - Key Handler  │
│                 │    │  - Layout: Normal│    │  - Git Watcher  │
└─────────────────┘    └──────────────────┘    └─────────────────┘
```

### 🎯 事件处理流程

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│  User Input     │───▶│  Event Router    │───▶│  Component      │
│  - Key Press    │    │                  │    │  Event Handler  │
│  - Mouse Click  │    │  Router Logic:   │    │                 │
│  - Terminal     │    │  ├─ Global Keys  │    │  Handle Logic:  │
│    Resize       │    │  ├─ Panel Keys   │    │  ├─ Update Data │
└─────────────────┘    │  └─ Context Keys │    │  ├─ Change View │
                       └──────────────────┘    │  └─ Git Command │
                                               └─────────────────┘
                                                        │
┌─────────────────┐    ┌──────────────────┐            ▼
│   UI Update     │◀───│   State Update   │    ┌─────────────────┐
│                 │    │                  │    │  Side Effects   │
│ ├─ Render Comp  │    │ ├─ Focus Change  │    │                 │
│ ├─ Update Cache │    │ ├─ Data Refresh  │    │ ├─ Git Commands │
│ └─ Status Bar   │    │ └─ Config Save   │    │ ├─ File I/O     │
└─────────────────┘    └──────────────────┘    │ └─ Cache Update │
                                               └─────────────────┘
```

### 🚀 Git操作异步流程

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   Git Command   │───▶│   Async Executor │───▶│   Cache Check   │
│                 │    │                  │    │                 │
│  Examples:      │    │ ┌──────────────┐ │    │ ├─ Hit: Return  │
│  • git log      │    │ │ Tokio Spawn  │ │    │ │   Cached Data │
│  • git branch   │    │ │ Background   │ │    │ └─ Miss: Exec   │
│  • git diff     │    │ │ Execution    │ │    │     Git Command │
│  • git stash    │    │ └──────────────┘ │    │                 │
└─────────────────┘    └──────────────────┘    └─────────────────┘
                                                        │
┌─────────────────┐    ┌──────────────────┐            ▼
│   UI Progress   │◀───│  Result Process  │    ┌─────────────────┐
│                 │    │                  │    │  Command Exec   │
│ ├─ Loading Bar  │    │ ├─ Parse Output  │    │                 │
│ ├─ Spinner      │    │ ├─ Update Cache  │    │ ├─ Execute      │
│ └─ Status Text  │    │ ├─ Error Handle  │    │ ├─ Stream Parse │
└─────────────────┘    │ └─ Notify UI     │    │ └─ Error Catch  │
                       └──────────────────┘    └─────────────────┘
```

## 关键算法设计

### 🔍 智能分支操作算法

```rust
// src/tui_unified/algorithms/smart_branch.rs
pub struct SmartBranchManager {
    cache: BranchCache,
    remote_status: HashMap<String, RemoteStatus>,
}

impl SmartBranchManager {
    // 一键切换+拉取算法
    pub async fn smart_checkout_and_pull(&self, branch: &str) -> Result<()> {
        // 1. 检查工作区状态
        let work_tree_clean = self.check_work_tree_status().await?;
        if !work_tree_clean {
            return Err("Working tree has uncommitted changes".into());
        }
        
        // 2. 检查远程状态 
        let remote_status = self.get_remote_status(branch).await?;
        let needs_pull = matches!(remote_status, RemoteStatus::Behind(_));
        
        // 3. 执行切换
        self.git_checkout(branch).await?;
        
        // 4. 自动拉取(如果需要)
        if needs_pull {
            self.git_pull().await?;
        }
        
        // 5. 更新缓存
        self.refresh_cache().await?;
        
        Ok(())
    }
    
    // 分支健康度检测
    pub async fn calculate_branch_health(&self, branch: &str) -> BranchHealth {
        let mut score = 100;
        let mut issues = Vec::new();
        
        // 检查是否落后于远程
        if let Ok(status) = self.get_remote_status(branch).await {
            match status {
                RemoteStatus::Behind(commits) => {
                    score -= commits * 5; // 每个落后提交扣5分
                    issues.push(format!("Behind by {} commits", commits));
                }
                RemoteStatus::Diverged(ahead, behind) => {
                    score -= (ahead + behind) * 3;
                    issues.push(format!("Diverged: +{} -{}", ahead, behind));
                }
                _ => {}
            }
        }
        
        // 检查是否有未推送的提交
        if let Ok(unpushed) = self.count_unpushed_commits(branch).await {
            if unpushed > 10 {
                score -= 20;
                issues.push(format!("{} unpushed commits", unpushed));
            }
        }
        
        BranchHealth { score, issues }
    }
}
```

### 📈 虚拟滚动算法 (大仓库优化)

```rust
// src/tui_unified/algorithms/virtual_scroll.rs
pub struct VirtualScrollManager<T> {
    items: Vec<T>,
    visible_range: (usize, usize),
    viewport_height: usize,
    scroll_offset: usize,
    buffer_size: usize, // 缓冲区大小
}

impl<T> VirtualScrollManager<T> {
    pub fn new(viewport_height: usize) -> Self {
        Self {
            items: Vec::new(),
            visible_range: (0, 0),
            viewport_height,
            scroll_offset: 0,
            buffer_size: viewport_height * 2, // 缓冲区是可视区域的2倍
        }
    }
    
    // 计算可见范围
    pub fn calculate_visible_range(&mut self) {
        let start = self.scroll_offset.saturating_sub(self.buffer_size / 2);
        let end = (self.scroll_offset + self.viewport_height + self.buffer_size / 2)
            .min(self.items.len());
        self.visible_range = (start, end);
    }
    
    // 获取可见项目
    pub fn get_visible_items(&self) -> &[T] {
        let (start, end) = self.visible_range;
        &self.items[start..end]
    }
    
    // 滚动到指定位置
    pub fn scroll_to(&mut self, index: usize) {
        self.scroll_offset = index;
        self.calculate_visible_range();
    }
}
```

### 🔍 智能搜索算法

```rust
// src/tui_unified/algorithms/smart_search.rs
pub struct SmartSearchEngine {
    index: SearchIndex,
    filters: Vec<SearchFilter>,
}

pub struct SearchIndex {
    commit_messages: HashMap<String, Vec<CommitId>>,
    authors: HashMap<String, Vec<CommitId>>,
    files: HashMap<PathBuf, Vec<CommitId>>,
    full_text: Option<TantivyIndex>, // 可选的全文搜索引擎
}

impl SmartSearchEngine {
    // 复合条件搜索
    pub async fn search(&self, query: &SearchQuery) -> Vec<SearchResult> {
        let mut results = Vec::new();
        
        // 1. 文本搜索
        if let Some(text) = &query.text {
            results.extend(self.search_text(text).await?);
        }
        
        // 2. 作者过滤
        if let Some(author) = &query.author {
            results = self.filter_by_author(results, author);
        }
        
        // 3. 时间范围过滤
        if let Some(date_range) = &query.date_range {
            results = self.filter_by_date_range(results, date_range);
        }
        
        // 4. 文件路径过滤
        if let Some(path) = &query.file_path {
            results = self.filter_by_file_path(results, path);
        }
        
        // 5. 结果排序和去重
        results.sort_by(|a, b| b.relevance_score.cmp(&a.relevance_score));
        results.dedup_by_key(|r| r.commit_id);
        
        results
    }
}
```

## API接口设计

### 🔧 Git操作接口

```rust
// src/tui_unified/git/interface.rs
#[async_trait]
pub trait GitInterface {
    // 基础Git操作
    async fn get_commits(&self, branch: Option<&str>, limit: Option<usize>) -> Result<Vec<Commit>>;
    async fn get_branches(&self) -> Result<Vec<Branch>>;
    async fn get_tags(&self) -> Result<Vec<Tag>>;
    async fn get_remotes(&self) -> Result<Vec<Remote>>;
    async fn get_stash_list(&self) -> Result<Vec<Stash>>;
    
    // 分支操作
    async fn checkout_branch(&self, branch: &str) -> Result<()>;
    async fn create_branch(&self, name: &str, base: Option<&str>) -> Result<()>;
    async fn delete_branch(&self, name: &str, force: bool) -> Result<()>;
    async fn pull_branch(&self, branch: Option<&str>) -> Result<()>;
    async fn push_branch(&self, branch: &str, force: bool) -> Result<()>;
    
    // Diff操作
    async fn get_commit_diff(&self, commit: &str) -> Result<String>;
    async fn get_file_diff(&self, file: &Path, base: Option<&str>) -> Result<String>;
    async fn get_staged_diff(&self) -> Result<String>;
    
    // Stash操作
    async fn stash_save(&self, message: Option<&str>) -> Result<()>;
    async fn stash_apply(&self, index: usize) -> Result<()>;
    async fn stash_drop(&self, index: usize) -> Result<()>;
    async fn stash_show(&self, index: usize) -> Result<String>;
}

// 实现Git接口
pub struct AsyncGitImpl {
    repo_path: PathBuf,
    command_cache: Arc<RwLock<HashMap<String, CachedResult>>>,
}

#[async_trait]
impl GitInterface for AsyncGitImpl {
    async fn get_commits(&self, branch: Option<&str>, limit: Option<usize>) -> Result<Vec<Commit>> {
        let cache_key = format!("commits_{}_{}", branch.unwrap_or("HEAD"), limit.unwrap_or(100));
        
        // 检查缓存
        if let Some(cached) = self.get_cached_result(&cache_key).await? {
            return Ok(cached);
        }
        
        // 执行Git命令
        let mut cmd = tokio::process::Command::new("git");
        cmd.arg("log")
           .arg("--format=%H|%an|%ae|%at|%s")
           .current_dir(&self.repo_path);
           
        if let Some(branch) = branch {
            cmd.arg(branch);
        }
        
        if let Some(limit) = limit {
            cmd.arg(format!("-{}", limit));
        }
        
        let output = cmd.output().await?;
        let commits = self.parse_commit_output(&output.stdout)?;
        
        // 缓存结果
        self.cache_result(&cache_key, &commits, Duration::from_secs(60)).await?;
        
        Ok(commits)
    }
}
```

### 🎨 组件接口设计

```rust
// src/tui_unified/components/interface.rs
pub trait ComponentInterface {
    type Props;
    type State;
    
    // 组件生命周期
    fn new(props: Self::Props) -> Self;
    fn mount(&mut self, state: &AppState);
    fn unmount(&mut self);
    
    // 渲染接口
    fn render(&mut self, frame: &mut Frame, area: Rect, state: &AppState);
    fn get_cursor_position(&self) -> Option<(u16, u16)>;
    
    // 事件处理
    fn handle_key_event(&mut self, key: KeyEvent, state: &mut AppState) -> EventResult;
    fn handle_mouse_event(&mut self, mouse: MouseEvent, state: &mut AppState) -> EventResult;
    fn handle_custom_event(&mut self, event: CustomEvent, state: &mut AppState) -> EventResult;
    
    // 数据更新
    fn should_update(&self, old_state: &AppState, new_state: &AppState) -> bool;
    fn update(&mut self, state: &AppState);
    
    // 焦点管理
    fn can_focus(&self) -> bool { true }
    fn on_focus(&mut self, state: &AppState);
    fn on_blur(&mut self, state: &AppState);
}

pub enum EventResult {
    Handled,
    NotHandled,
    Bubble,     // 继续向上传播
    Navigate(Navigation), // 导航到其他组件
}

pub enum Navigation {
    NextPanel,
    PrevPanel,
    ToPanel(FocusPanel),
    ToView(ViewType),
}
```

## 性能优化设计

### ⚡ 缓存系统设计

```rust
// src/tui_unified/cache/mod.rs
pub struct CacheManager {
    pub git_cache: GitCache,
    pub ui_cache: UiCache,
    pub file_cache: FileCache,
}

pub struct GitCache {
    commits: LruCache<String, Vec<Commit>>,
    branches: LruCache<String, Vec<Branch>>,
    diffs: LruCache<String, String>,
    ttl: HashMap<String, Instant>,
}

impl GitCache {
    pub fn new(capacity: usize) -> Self {
        Self {
            commits: LruCache::new(capacity),
            branches: LruCache::new(capacity / 2),
            diffs: LruCache::new(capacity),
            ttl: HashMap::new(),
        }
    }
    
    pub fn get<T: Clone>(&mut self, key: &str) -> Option<T> {
        // 检查TTL
        if let Some(expire_time) = self.ttl.get(key) {
            if Instant::now() > *expire_time {
                self.invalidate(key);
                return None;
            }
        }
        
        // 从对应缓存获取
        if key.starts_with("commits_") {
            self.commits.get(key).cloned()
        } else if key.starts_with("branches_") {
            self.branches.get(key).cloned()
        } else if key.starts_with("diff_") {
            self.diffs.get(key).cloned()
        } else {
            None
        }
    }
    
    pub fn set<T: Clone>(&mut self, key: String, value: T, ttl: Duration) {
        let expire_time = Instant::now() + ttl;
        self.ttl.insert(key.clone(), expire_time);
        
        if key.starts_with("commits_") {
            if let Ok(commits) = bincode::serialize(&value) {
                self.commits.put(key, commits);
            }
        }
        // ... 其他缓存类型
    }
}
```

### 🔄 异步处理优化

```rust
// src/tui_unified/async_manager.rs
pub struct AsyncTaskManager {
    runtime: tokio::runtime::Handle,
    active_tasks: Arc<RwLock<HashMap<TaskId, JoinHandle<()>>>>,
    task_counter: Arc<AtomicU64>,
}

impl AsyncTaskManager {
    pub fn spawn_git_task<F, T>(&self, name: &str, future: F) -> TaskId 
    where
        F: Future<Output = Result<T>> + Send + 'static,
        T: Send + 'static,
    {
        let task_id = TaskId(self.task_counter.fetch_add(1, Ordering::SeqCst));
        let active_tasks = Arc::clone(&self.active_tasks);
        
        let handle = self.runtime.spawn(async move {
            match future.await {
                Ok(result) => {
                    // 发送结果到UI线程
                    EventBus::publish(TaskComplete { task_id, result });
                }
                Err(err) => {
                    // 发送错误到UI线程
                    EventBus::publish(TaskError { task_id, error: err });
                }
            }
            
            // 清理任务
            active_tasks.write().await.remove(&task_id);
        });
        
        self.active_tasks.write().unwrap().insert(task_id, handle);
        task_id
    }
    
    pub fn cancel_task(&self, task_id: TaskId) {
        if let Some(handle) = self.active_tasks.write().unwrap().remove(&task_id) {
            handle.abort();
        }
    }
}
```

这个技术设计文档提供了完整的架构设计、模块划分、数据流程、算法设计和性能优化方案，为TUI界面整合提供了详细的技术指导。
