# TUI API接口设计文档

## API架构概述

TUI统一界面采用分层API设计，提供清晰的接口边界和统一的调用方式。

```
┌─────────────────────────────────────────────────────────────────┐
│                        Application Layer                        │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐ │
│  │   UI Components │  │  Event Handlers │  │  State Manager  │ │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
                                  ▼
┌─────────────────────────────────────────────────────────────────┐
│                         API Interface Layer                     │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐ │
│  │   Git API       │  │   Layout API    │  │   Event API     │ │
│  │                 │  │                 │  │                 │ │
│  │ • Repository    │  │ • Panel Manager │  │ • Key Bindings  │ │
│  │ • Commands      │  │ • Focus Control │  │ • Event Router  │ │
│  │ • Data Models   │  │ • Theme System  │  │ • Async Tasks   │ │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
                                  ▼
┌─────────────────────────────────────────────────────────────────┐
│                         Core Service Layer                      │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐ │
│  │   Git Service   │  │  Cache Service  │  │ Config Service  │ │
│  │                 │  │                 │  │                 │ │
│  │ • Command Exec  │  │ • Memory Cache  │  │ • Settings      │ │
│  │ • Parser Utils  │  │ • File Cache    │  │ • Themes        │ │
│  │ • Async Manager │  │ • Cache Policy  │  │ • Key Mappings  │ │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

## 1. Git API接口设计

### 1.1 基础Repository接口

```rust
#[async_trait]
pub trait GitRepositoryAPI: Send + Sync {
    // 仓库信息
    async fn get_repo_info(&self) -> Result<RepoInfo>;
    async fn get_current_branch(&self) -> Result<String>;
    async fn get_repo_status(&self) -> Result<RepoStatus>;
    async fn is_repo_dirty(&self) -> Result<bool>;
    
    // 提交操作
    async fn get_commits(&self, options: CommitListOptions) -> Result<Vec<Commit>>;
    async fn get_commit_details(&self, hash: &str) -> Result<CommitDetails>;
    async fn get_commit_diff(&self, hash: &str, options: DiffOptions) -> Result<String>;
    
    // 分支操作
    async fn get_branches(&self, filter: BranchFilter) -> Result<Vec<Branch>>;
    async fn create_branch(&self, name: &str, base: Option<&str>) -> Result<()>;
    async fn checkout_branch(&self, name: &str) -> Result<()>;
    async fn delete_branch(&self, name: &str, force: bool) -> Result<()>;
    async fn rename_branch(&self, old_name: &str, new_name: &str) -> Result<()>;
    
    // 远程操作
    async fn get_remotes(&self) -> Result<Vec<Remote>>;
    async fn fetch(&self, remote: Option<&str>) -> Result<()>;
    async fn pull(&self, remote: Option<&str>, branch: Option<&str>) -> Result<()>;
    async fn push(&self, remote: Option<&str>, branch: Option<&str>, force: bool) -> Result<()>;
    
    // 标签操作
    async fn get_tags(&self, filter: TagFilter) -> Result<Vec<Tag>>;
    async fn create_tag(&self, name: &str, message: Option<&str>, commit: Option<&str>) -> Result<()>;
    async fn delete_tag(&self, name: &str) -> Result<()>;
    
    // Stash操作
    async fn get_stash_list(&self) -> Result<Vec<StashEntry>>;
    async fn stash_save(&self, message: Option<&str>, include_untracked: bool) -> Result<()>;
    async fn stash_apply(&self, index: usize, keep: bool) -> Result<()>;
    async fn stash_drop(&self, index: usize) -> Result<()>;
    async fn stash_show(&self, index: usize, diff: bool) -> Result<String>;
}

// 查询选项结构
#[derive(Debug, Clone, Default)]
pub struct CommitListOptions {
    pub branch: Option<String>,
    pub limit: Option<usize>,
    pub skip: Option<usize>,
    pub since: Option<DateTime<Utc>>,
    pub until: Option<DateTime<Utc>>,
    pub author: Option<String>,
    pub grep: Option<String>,
    pub path: Option<PathBuf>,
}

#[derive(Debug, Clone, Default)]
pub struct DiffOptions {
    pub context_lines: Option<usize>,
    pub ignore_whitespace: bool,
    pub word_diff: bool,
    pub stat_only: bool,
    pub name_only: bool,
}

#[derive(Debug, Clone)]
pub enum BranchFilter {
    All,
    Local,
    Remote,
    Merged,
    NotMerged,
}

#[derive(Debug, Clone)]
pub enum TagFilter {
    All,
    Pattern(String),
    Since(DateTime<Utc>),
    Until(DateTime<Utc>),
}

// 数据模型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoInfo {
    pub path: PathBuf,
    pub is_bare: bool,
    pub head_branch: String,
    pub remote_url: Option<String>,
    pub total_commits: usize,
    pub contributors_count: usize,
}

#[derive(Debug, Clone)]
pub struct CommitDetails {
    pub commit: Commit,
    pub parents: Vec<String>,
    pub files_changed: Vec<FileChange>,
    pub stats: CommitStats,
    pub signature_status: SignatureStatus,
}

#[derive(Debug, Clone)]
pub struct FileChange {
    pub path: PathBuf,
    pub status: FileStatus,
    pub additions: usize,
    pub deletions: usize,
    pub binary: bool,
}

#[derive(Debug, Clone)]
pub enum FileStatus {
    Added,
    Modified,
    Deleted,
    Renamed { from: PathBuf },
    Copied { from: PathBuf },
}
```

### 1.2 智能Git操作接口

```rust
#[async_trait]
pub trait SmartGitAPI: GitRepositoryAPI {
    // 智能分支操作
    async fn smart_checkout(&self, branch: &str, auto_pull: bool) -> Result<CheckoutResult>;
    async fn get_branch_health(&self, branch: &str) -> Result<BranchHealth>;
    async fn suggest_merge_strategy(&self, source: &str, target: &str) -> Result<MergeStrategy>;
    
    // 冲突解决
    async fn detect_conflicts(&self, source: &str, target: &str) -> Result<Vec<ConflictInfo>>;
    async fn get_conflict_resolution_suggestions(&self, file_path: &Path) -> Result<Vec<ResolutionSuggestion>>;
    async fn apply_conflict_resolution(&self, file_path: &Path, resolution: Resolution) -> Result<()>;
    
    // 智能搜索
    async fn search_commits(&self, query: &SearchQuery) -> Result<Vec<SearchResult>>;
    async fn search_content(&self, pattern: &str, options: ContentSearchOptions) -> Result<Vec<ContentMatch>>;
    async fn get_related_commits(&self, commit_hash: &str) -> Result<Vec<RelatedCommit>>;
    
    // 批量操作
    async fn batch_branch_operations(&self, operations: Vec<BranchOperation>) -> Result<Vec<OperationResult>>;
    async fn batch_tag_operations(&self, operations: Vec<TagOperation>) -> Result<Vec<OperationResult>>;
    
    // 统计分析
    async fn get_commit_statistics(&self, options: StatisticsOptions) -> Result<CommitStatistics>;
    async fn get_contributor_analysis(&self, options: ContributorOptions) -> Result<ContributorAnalysis>;
    async fn get_activity_timeline(&self, options: TimelineOptions) -> Result<ActivityTimeline>;
}

#[derive(Debug, Clone)]
pub struct CheckoutResult {
    pub success: bool,
    pub previous_branch: String,
    pub pulled_changes: Option<PullResult>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct BranchHealth {
    pub score: u8,  // 0-100
    pub issues: Vec<HealthIssue>,
    pub recommendations: Vec<String>,
    pub last_activity: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub enum HealthIssue {
    BehindRemote(usize),
    AheadRemote(usize),
    Stale(Duration),
    LargeUnpushedCommits(usize),
    ConflictPotential(f32), // 0.0-1.0
}

#[derive(Debug, Clone)]
pub struct SearchQuery {
    pub text: Option<String>,
    pub author: Option<String>,
    pub date_range: Option<(DateTime<Utc>, DateTime<Utc>)>,
    pub file_path: Option<PathBuf>,
    pub branch: Option<String>,
    pub regex: bool,
    pub case_sensitive: bool,
}

#[derive(Debug, Clone)]
pub struct ContentSearchOptions {
    pub file_types: Vec<String>,
    pub exclude_paths: Vec<PathBuf>,
    pub context_lines: usize,
    pub max_results: Option<usize>,
}

#[derive(Debug, Clone)]
pub enum BranchOperation {
    Create { name: String, base: Option<String> },
    Delete { name: String, force: bool },
    Rename { old_name: String, new_name: String },
    Merge { source: String, target: String },
}

#[derive(Debug, Clone)]
pub enum TagOperation {
    Create { name: String, message: Option<String>, commit: Option<String> },
    Delete { name: String },
    Push { name: String, remote: String },
}
```

## 2. Layout API接口设计

### 2.1 布局管理接口

```rust
pub trait LayoutAPI: Send + Sync {
    // 布局计算
    fn calculate_layout(&self, terminal_size: Rect) -> LayoutResult;
    fn get_panel_constraints(&self, mode: LayoutMode) -> PanelConstraints;
    fn resize_panel(&mut self, panel: PanelType, size_delta: i16) -> Result<()>;
    
    // 模式管理
    fn set_layout_mode(&mut self, mode: LayoutMode) -> Result<()>;
    fn get_current_mode(&self) -> LayoutMode;
    fn toggle_mode(&mut self, mode: LayoutMode) -> Result<LayoutMode>;
    
    // 面板管理
    fn show_panel(&mut self, panel: PanelType) -> Result<()>;
    fn hide_panel(&mut self, panel: PanelType) -> Result<()>;
    fn is_panel_visible(&self, panel: PanelType) -> bool;
    fn get_panel_area(&self, panel: PanelType) -> Option<Rect>;
    
    // 焦点管理
    fn get_focused_panel(&self) -> PanelType;
    fn set_focus(&mut self, panel: PanelType) -> Result<()>;
    fn next_focus(&mut self) -> PanelType;
    fn previous_focus(&mut self) -> PanelType;
    fn can_focus(&self, panel: PanelType) -> bool;
    
    // 布局持久化
    fn save_layout(&self) -> Result<()>;
    fn load_layout(&mut self) -> Result<()>;
    fn reset_to_default(&mut self) -> Result<()>;
}

#[derive(Debug, Clone)]
pub struct LayoutResult {
    pub sidebar: Rect,
    pub content: Rect,
    pub detail: Rect,
    pub status_bar: Rect,
    pub modal_area: Option<Rect>,
}

#[derive(Debug, Clone)]
pub struct PanelConstraints {
    pub sidebar: Constraint,
    pub content: Constraint,
    pub detail: Constraint,
    pub min_width: u16,
    pub min_height: u16,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PanelType {
    Sidebar,
    Content,
    Detail,
    StatusBar,
    Modal,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LayoutMode {
    Normal,           // 三栏布局
    SplitHorizontal,  // 水平分屏
    SplitVertical,    // 垂直分屏
    FullScreen,       // 全屏模式
    Minimal,          // 最小化界面
}

// 约束类型
pub use ratatui::layout::Constraint;
```

### 2.2 主题系统接口

```rust
pub trait ThemeAPI: Send + Sync {
    // 主题管理
    fn get_current_theme(&self) -> &Theme;
    fn set_theme(&mut self, theme: Theme) -> Result<()>;
    fn load_theme(&mut self, name: &str) -> Result<Theme>;
    fn save_theme(&self, name: &str, theme: &Theme) -> Result<()>;
    fn list_themes(&self) -> Vec<String>;
    
    // 颜色获取
    fn get_color(&self, element: ThemeElement) -> Color;
    fn get_style(&self, element: ThemeElement) -> Style;
    fn get_color_scheme(&self) -> &ColorScheme;
    
    // 主题定制
    fn customize_color(&mut self, element: ThemeElement, color: Color) -> Result<()>;
    fn customize_style(&mut self, element: ThemeElement, style: Style) -> Result<()>;
    fn reset_element(&mut self, element: ThemeElement) -> Result<()>;
    
    // 主题导入导出
    fn export_theme(&self, path: &Path) -> Result<()>;
    fn import_theme(&mut self, path: &Path) -> Result<Theme>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub name: String,
    pub description: String,
    pub author: String,
    pub version: String,
    pub colors: ColorScheme,
    pub styles: StyleScheme,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorScheme {
    pub background: Color,
    pub foreground: Color,
    pub primary: Color,
    pub secondary: Color,
    pub accent: Color,
    pub success: Color,
    pub warning: Color,
    pub error: Color,
    pub info: Color,
    pub selection: Color,
    pub border: Color,
    pub shadow: Color,
}

#[derive(Debug, Clone)]
pub enum ThemeElement {
    Background,
    Text,
    Border,
    Selection,
    Accent,
    Success,
    Warning,
    Error,
    Info,
    GitAdd,
    GitModify,
    GitDelete,
    BranchLocal,
    BranchRemote,
    CommitHash,
    CommitAuthor,
    CommitMessage,
    DiffAdd,
    DiffDelete,
    DiffContext,
    StatusBar,
    MenuActive,
    MenuInactive,
}

pub use ratatui::style::{Color, Style, Modifier};
```

## 3. Event API接口设计

### 3.1 事件处理接口

```rust
#[async_trait]
pub trait EventAPI: Send + Sync {
    // 事件注册
    fn register_handler<F>(&mut self, event_type: EventType, handler: F) -> Result<HandlerId>
    where
        F: Fn(Event) -> EventResult + Send + Sync + 'static;
    
    fn unregister_handler(&mut self, handler_id: HandlerId) -> Result<()>;
    
    // 事件发送
    async fn emit_event(&self, event: Event) -> Result<()>;
    fn emit_sync_event(&self, event: Event) -> Result<()>;
    
    // 事件订阅
    fn subscribe<F>(&mut self, filter: EventFilter, callback: F) -> Result<SubscriptionId>
    where
        F: Fn(Event) -> () + Send + Sync + 'static;
    
    fn unsubscribe(&mut self, subscription_id: SubscriptionId) -> Result<()>;
    
    // 按键绑定
    fn bind_key(&mut self, key: KeyBinding, action: Action) -> Result<()>;
    fn unbind_key(&mut self, key: KeyBinding) -> Result<()>;
    fn get_key_bindings(&self) -> &HashMap<KeyBinding, Action>;
    fn reset_key_bindings(&mut self) -> Result<()>;
    
    // 事件历史
    fn get_event_history(&self) -> &[Event];
    fn clear_event_history(&mut self);
}

#[derive(Debug, Clone)]
pub enum EventType {
    KeyPress,
    MouseClick,
    WindowResize,
    GitDataUpdated,
    ConfigChanged,
    ThemeChanged,
    FocusChanged,
    StateChanged,
    AsyncTaskCompleted,
    Error,
}

#[derive(Debug, Clone)]
pub enum Event {
    Key(KeyEvent),
    Mouse(MouseEvent),
    Resize { width: u16, height: u16 },
    GitUpdate { data_type: GitDataType },
    StateChange { change: StateChange },
    Focus { panel: PanelType, previous: PanelType },
    Error { message: String, source: Option<String> },
    AsyncResult { task_id: TaskId, result: AsyncTaskResult },
    Custom { name: String, data: serde_json::Value },
}

#[derive(Debug, Clone)]
pub enum EventResult {
    Handled,
    NotHandled,
    Propagate,
    Stop,
    Async(BoxFuture<'static, Result<()>>),
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct KeyBinding {
    pub key: crossterm::event::KeyCode,
    pub modifiers: crossterm::event::KeyModifiers,
}

#[derive(Debug, Clone)]
pub enum Action {
    Quit,
    NextPanel,
    PrevPanel,
    Refresh,
    Search,
    ToggleHelp,
    GitCheckout,
    GitPull,
    GitPush,
    ShowDiff,
    Custom(String),
}

pub struct EventFilter {
    pub event_types: Option<Vec<EventType>>,
    pub source: Option<String>,
    pub priority: Option<EventPriority>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EventPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

pub type HandlerId = uuid::Uuid;
pub type SubscriptionId = uuid::Uuid;
```

### 3.2 异步任务管理接口

```rust
#[async_trait]
pub trait AsyncTaskAPI: Send + Sync {
    // 任务管理
    async fn spawn_task<F, T>(&self, name: &str, future: F) -> Result<TaskId>
    where
        F: Future<Output = Result<T>> + Send + 'static,
        T: Send + 'static;
    
    async fn spawn_git_task<F, T>(&self, name: &str, future: F) -> Result<TaskId>
    where
        F: Future<Output = Result<T>> + Send + 'static,
        T: Send + 'static;
    
    fn cancel_task(&self, task_id: TaskId) -> Result<()>;
    fn get_task_status(&self, task_id: TaskId) -> Option<TaskStatus>;
    fn list_active_tasks(&self) -> Vec<TaskInfo>;
    
    // 进度跟踪
    async fn update_task_progress(&self, task_id: TaskId, progress: f64) -> Result<()>;
    async fn update_task_message(&self, task_id: TaskId, message: &str) -> Result<()>;
    
    // 任务结果处理
    async fn wait_for_task<T>(&self, task_id: TaskId) -> Result<T>
    where
        T: Send + 'static;
    
    fn on_task_complete<F>(&self, task_id: TaskId, callback: F) -> Result<()>
    where
        F: FnOnce(AsyncTaskResult) + Send + 'static;
    
    // 批量任务管理
    async fn spawn_batch_tasks<F, T>(&self, tasks: Vec<(&str, F)>) -> Result<Vec<TaskId>>
    where
        F: Future<Output = Result<T>> + Send + 'static,
        T: Send + 'static;
    
    async fn wait_for_all(&self, task_ids: &[TaskId]) -> Result<Vec<AsyncTaskResult>>;
    
    // 任务调度
    fn set_max_concurrent_tasks(&mut self, max: usize);
    fn get_task_queue_size(&self) -> usize;
    fn pause_task_execution(&self);
    fn resume_task_execution(&self);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TaskId(pub uuid::Uuid);

#[derive(Debug, Clone)]
pub struct TaskInfo {
    pub id: TaskId,
    pub name: String,
    pub status: TaskStatus,
    pub progress: Option<f64>,
    pub message: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone)]
pub enum AsyncTaskResult {
    Success(serde_json::Value),
    Error(String),
    Cancelled,
}
```

## 4. 配置API接口设计

```rust
pub trait ConfigAPI: Send + Sync {
    // 配置读取
    fn get_config<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>>;
    fn get_or_default<T: DeserializeOwned + Default>(&self, key: &str) -> T;
    fn get_string(&self, key: &str) -> Option<String>;
    fn get_bool(&self, key: &str) -> Option<bool>;
    fn get_int(&self, key: &str) -> Option<i64>;
    fn get_float(&self, key: &str) -> Option<f64>;
    
    // 配置写入
    fn set_config<T: Serialize>(&mut self, key: &str, value: &T) -> Result<()>;
    fn set_string(&mut self, key: &str, value: &str) -> Result<()>;
    fn set_bool(&mut self, key: &str, value: bool) -> Result<()>;
    fn set_int(&mut self, key: &str, value: i64) -> Result<()>;
    fn set_float(&mut self, key: &str, value: f64) -> Result<()>;
    
    // 配置管理
    fn has_key(&self, key: &str) -> bool;
    fn remove_key(&mut self, key: &str) -> Result<()>;
    fn list_keys(&self) -> Vec<String>;
    fn clear_all(&mut self) -> Result<()>;
    
    // 配置持久化
    fn save(&self) -> Result<()>;
    fn reload(&mut self) -> Result<()>;
    fn backup(&self, path: &Path) -> Result<()>;
    fn restore(&mut self, path: &Path) -> Result<()>;
    
    // 配置监听
    fn on_change<F>(&mut self, key: &str, callback: F) -> Result<ChangeListenerId>
    where
        F: Fn(&str, &serde_json::Value, &serde_json::Value) + Send + Sync + 'static;
    
    fn remove_listener(&mut self, listener_id: ChangeListenerId) -> Result<()>;
}

pub type ChangeListenerId = uuid::Uuid;

// 配置结构定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub ui: UiConfig,
    pub git: GitConfig,
    pub theme: ThemeConfig,
    pub keybindings: KeyBindingConfig,
    pub cache: CacheConfig,
    pub performance: PerformanceConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    pub layout_mode: LayoutMode,
    pub panel_sizes: HashMap<String, u16>,
    pub show_line_numbers: bool,
    pub show_git_stats: bool,
    pub auto_refresh_interval: Option<u64>,
    pub max_commits_display: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitConfig {
    pub default_branch: String,
    pub auto_fetch_interval: Option<u64>,
    pub show_signatures: bool,
    pub diff_context_lines: usize,
    pub commit_template: Option<String>,
}
```

## 5. 统一错误处理

```rust
#[derive(Debug, thiserror::Error)]
pub enum TuiApiError {
    #[error("Git operation failed: {0}")]
    GitError(#[from] GitError),
    
    #[error("Layout error: {0}")]
    LayoutError(String),
    
    #[error("Event handling error: {0}")]
    EventError(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("Theme error: {0}")]
    ThemeError(String),
    
    #[error("Task execution error: {0}")]
    TaskError(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    SerdeError(#[from] serde_json::Error),
    
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
    
    #[error("Resource not found: {0}")]
    NotFound(String),
}

pub type Result<T> = std::result::Result<T, TuiApiError>;

// 错误恢复策略
pub trait ErrorRecovery {
    fn can_recover(&self, error: &TuiApiError) -> bool;
    fn recover(&self, error: &TuiApiError) -> Result<()>;
    fn get_recovery_suggestions(&self, error: &TuiApiError) -> Vec<String>;
}
```

这个API接口设计文档提供了完整的接口定义，涵盖了TUI应用的所有核心功能模块，为开发提供了清晰的API契约。