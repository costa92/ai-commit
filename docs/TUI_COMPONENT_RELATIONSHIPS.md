# TUI组件关系图与数据流程设计

## 组件架构关系图

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                            TUI Unified Application                           │
│                                   (Root)                                    │
└─────────────────────────┬───────────────────────────────────────────────────┘
                          │
                          ▼
         ┌─────────────────────────────────────────────────────┐
         │                App State Manager                     │
         │  ┌─────────────────┐  ┌─────────────────────────────┐ │
         │  │   Global State  │  │      Event Bus              │ │
         │  │  • UI State     │  │  • Event Router             │ │
         │  │  • Git State    │  │  • Async Task Manager       │ │
         │  │  • Config       │  │  • Cache Manager            │ │
         │  └─────────────────┘  └─────────────────────────────┘ │
         └─────────────────────────────────────────────────────┘
                          │
         ┌────────────────┼────────────────┐
         ▼                ▼                ▼
┌─────────────────┐ ┌─────────────────┐ ┌─────────────────┐
│  Layout Manager │ │  Focus Manager  │ │ Component Tree  │
│                 │ │                 │ │                 │
│ • Panel Sizing  │ │ • Focus Ring    │ │ • Sidebar       │
│ • Mode Switching│ │ • Tab Navigation│ │ • Content       │
│ • Split Layouts │ │ • Focus History │ │ • Detail Panel  │
└─────────────────┘ └─────────────────┘ │ • Status Bar    │
                                        └─────────────────┘
                                                 │
                      ┌──────────────────────────┼──────────────────────────┐
                      ▼                          ▼                          ▼
              ┌─────────────────┐       ┌─────────────────┐       ┌─────────────────┐
              │  Sidebar Panel  │       │  Content Panel  │       │  Detail Panel   │
              │                 │       │                 │       │                 │
              │ ┌─────────────┐ │       │ ┌─────────────┐ │       │ ┌─────────────┐ │
              │ │Menu Item    │ │       │ │Dynamic View │ │       │ │Info Panel   │ │
              │ │Components   │ │       │ │Components   │ │       │ │             │ │
              │ │• Git Log    │ │       │ │• GitLogView │ │       │ │• Commit Info│ │
              │ │• Branches   │ │       │ │• BranchView │ │       │ │• Branch Info│ │
              │ │• Tags       │ │       │ │• TagsView   │ │       │ │• Tag Info   │ │
              │ │• Remotes    │ │◄─────►│ │• RemoteView │ │◄─────►│ │• Stash Info │ │
              │ │• Stash      │ │       │ │• StashView  │ │       │ └─────────────┘ │
              │ │• History    │ │       │ │• HistoryView│ │       │ ┌─────────────┐ │
              │ └─────────────┘ │       │ └─────────────┘ │       │ │Diff Viewer  │ │
              └─────────────────┘       └─────────────────┘       │ │             │ │
                                                                  │ │• Syntax HL  │ │
                                                                  │ │• Side-by-side│ │
                                                                  │ │• Tree View  │ │
                                                                  │ │• Full Screen│ │
                                                                  │ └─────────────┘ │
                                                                  └─────────────────┘
                      ▲                          ▲                          ▲
                      │                          │                          │
              ┌───────────────┐          ┌──────────────────┐       ┌──────────────────┐
              │Widget Library │          │   View Router    │       │  Smart Components │
              │               │          │                  │       │                  │
              │• List Widget  │          │• View Factory    │       │• Smart Branch Ops│
              │• Table Widget │          │• Route Matching  │       │• Conflict Resolver│
              │• Tree Widget  │          │• State Injection │       │• Merge Assistant │
              │• Search Box   │          │• Props Passing   │       │• Search Engine   │
              │• Progress Bar │          │• Lifecycle Mgmt  │       │• Batch Operations│
              │• Modal Dialog │          │                  │       │                  │
              └───────────────┘          └──────────────────┘       └──────────────────┘
                      ▲                                                       ▲
                      └─────────────────────────┬─────────────────────────────┘
                                                ▼
                                    ┌─────────────────────────────┐
                                    │      Service Layer          │
                                    │                             │
                                    │ ┌─────────────────────────┐ │
                                    │ │     Git Service         │ │
                                    │ │  • Async Git Commands   │ │
                                    │ │  • Data Parsing         │ │
                                    │ │  • Cache Management     │ │
                                    │ └─────────────────────────┘ │
                                    │ ┌─────────────────────────┐ │
                                    │ │   Configuration Service │ │
                                    │ │  • Settings Management  │ │
                                    │ │  • Theme Loading        │ │
                                    │ │  • Key Binding Config   │ │
                                    │ └─────────────────────────┘ │
                                    │ ┌─────────────────────────┐ │
                                    │ │    Storage Service      │ │
                                    │ │  • File I/O             │ │
                                    │ │  • Cache Storage        │ │
                                    │ │  • User Preferences     │ │
                                    │ └─────────────────────────┘ │
                                    └─────────────────────────────┘
```

## 数据流程图

### 1. 应用启动流程

```
┌─────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   main()    │───▶│  App::new()     │───▶│ Config Loading  │
│             │    │                 │    │ • Load settings │
│ • CLI Args  │    │ • Init State    │    │ • Load themes   │
│ • Setup     │    │ • Setup         │    │ • Key bindings  │
└─────────────┘    │   Components    │    └─────────────────┘
                   └─────────────────┘             │
                            │                      ▼
                            ▼               ┌─────────────────┐
                   ┌─────────────────┐      │ Git Repository  │
                   │  Event Loop     │      │ Initialization  │
                   │                 │      │                 │
                   │ • Input Handle  │◄─────│ • Detect repo   │
                   │ • Render Loop   │      │ • Load branches │
                   │ • State Update  │      │ • Initial data  │
                   └─────────────────┘      └─────────────────┘
```

### 2. 用户交互流程

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   User Input    │───▶│  Event Router   │───▶│ Component Tree  │
│                 │    │                 │    │                 │
│ • Key Press     │    │ • Route Event   │    │ • Handle Event  │
│ • Mouse Click   │    │ • Check Binding │    │ • Update State  │
│ • Terminal      │    │ • Priority      │    │ • Trigger       │
│   Resize        │    │   Handling      │    │   Actions       │
└─────────────────┘    └─────────────────┘    └─────────────────┘
                                                        │
                       ┌─────────────────┐              ▼
                       │   Side Effects  │    ┌─────────────────┐
                       │                 │◄───│  State Changes  │
                       │ • Git Commands  │    │                 │
                       │ • File I/O      │    │ • Focus Change  │
                       │ • Cache Update  │    │ • View Switch   │
                       │ • Config Save   │    │ • Data Refresh  │
                       └─────────────────┘    │ • Layout Update │
                                │             └─────────────────┘
                                ▼                       │
                       ┌─────────────────┐              ▼
                       │  Async Tasks    │    ┌─────────────────┐
                       │                 │    │   UI Re-render  │
                       │ • Background    │    │                 │
                       │   Git Ops       │───▶│ • Component     │
                       │ • Progress      │    │   Updates       │
                       │   Updates       │    │ • Layout Calc   │
                       │ • Result        │    │ • Style Apply   │
                       │   Callbacks     │    └─────────────────┘
                       └─────────────────┘
```

### 3. Git操作数据流

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│ Git Operation   │───▶│   Cache Check   │───▶│  Command Exec   │
│ Request         │    │                 │    │                 │
│                 │    │ • Key Lookup    │    │ • Async Spawn   │
│ • get_commits() │    │ • TTL Check     │    │ • Process Git   │
│ • get_branches()│    │ • Return if Hit │    │ • Stream Parse  │
│ • checkout()    │    │                 │    │ • Error Handle  │
│ • diff()        │    │ • Continue if   │    │                 │
└─────────────────┘    │   Miss          │    └─────────────────┘
                       └─────────────────┘             │
                                │                      ▼
                ┌─────────────────────────┐    ┌─────────────────┐
                │    Data Processing      │◄───│   Raw Output    │
                │                         │    │                 │
                │ • Parse Git Output      │    │ • stdout        │
                │ • Create Data Models    │    │ • stderr        │
                │ • Apply Transformations │    │ • Exit Code     │
                │ • Validate Results      │    └─────────────────┘
                └─────────────────────────┘
                              │
                              ▼
                ┌─────────────────────────┐    ┌─────────────────┐
                │    Cache Update         │───▶│  State Update   │
                │                         │    │                 │
                │ • Store Results         │    │ • Notify        │
                │ • Update TTL            │    │   Components    │
                │ • Invalidate Related    │    │ • Trigger       │
                │   Entries               │    │   Re-render     │
                └─────────────────────────┘    └─────────────────┘
```

### 4. 组件生命周期流程

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│ Component Init  │───▶│   Mount Phase   │───▶│  Active Phase   │
│                 │    │                 │    │                 │
│ • new()         │    │ • mount()       │    │ • render()      │
│ • Props Setup   │    │ • Subscribe     │    │ • handle_event()│
│ • Initial State │    │   Events        │    │ • update()      │
└─────────────────┘    │ • Load Data     │    │ • Lifecycle     │
                       │ • Setup         │    │   Hooks         │
                       │   Resources     │    └─────────────────┘
                       └─────────────────┘             │
                                                       │
                       ┌─────────────────┐             │
                       │ Unmount Phase   │◄────────────┘
                       │                 │
                       │ • unmount()     │
                       │ • Cleanup       │
                       │   Resources     │
                       │ • Unsubscribe   │
                       │   Events        │
                       └─────────────────┘
```

## 组件依赖关系

### 1. 核心依赖图

```
AppState (Central State)
    ▲
    │ (read/write)
    │
    ├─► Layout Manager ────► Panel Components
    │                           │
    ├─► Focus Manager ──────────┤
    │                           │
    ├─► Event Router ───────────┤
    │                           │
    └─► Theme Manager ──────────┤
                                │
                        ┌───────▼──────┐
                        │   Components │
                        │              │
                        │ ┌──────────┐ │
                        │ │ Sidebar  │ │
                        │ └──────────┘ │
                        │ ┌──────────┐ │
                        │ │ Content  │ │
                        │ └──────────┘ │
                        │ ┌──────────┐ │
                        │ │ Detail   │ │
                        │ └──────────┘ │
                        └──────────────┘
                                │
                                ▼
                        ┌──────────────┐
                        │    Widgets   │
                        │              │
                        │ • List       │
                        │ • Table      │
                        │ • Tree       │
                        │ • DiffViewer │
                        │ • SearchBox  │
                        └──────────────┘
                                │
                                ▼
                        ┌──────────────┐
                        │   Services   │
                        │              │
                        │ • Git        │
                        │ • Config     │
                        │ • Cache      │
                        │ • Storage    │
                        └──────────────┘
```

### 2. 通信模式图

```
┌─────────────────────────────────────────────────────────────────┐
│                         Event-Driven Architecture               │
└─────────────────────────────────────────────────────────────────┘
                                  │
         ┌────────────────────────┼────────────────────────┐
         ▼                        ▼                        ▼
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Components    │    │   Event Bus     │    │    Services     │
│                 │    │                 │    │                 │
│ • Emit Events   │───▶│ • Route Events  │───▶│ • Execute Tasks │
│ • Listen Events │◄───│ • Manage        │◄───│ • Return        │
│ • Update UI     │    │   Subscriptions │    │   Results       │
│                 │    │ • Handle        │    │ • Emit Events   │
└─────────────────┘    │   Priorities    │    └─────────────────┘
         ▲              └─────────────────┘                ▲
         │                        ▲                        │
         │                        │                        │
         └────────────────────────┼────────────────────────┘
                                  ▼
                     ┌─────────────────────────┐
                     │      State Manager      │
                     │                         │
                     │ • Central State Store   │
                     │ • State Change Events   │
                     │ • Component Subscriptions│
                     │ • Persistence           │
                     └─────────────────────────┘
```

### 3. 智能组件协作图

```
┌─────────────────────────────────────────────────────────────────┐
│                    Smart Components Layer                       │
└─────────────────────────────────────────────────────────────────┘
                                  │
         ┌────────────────────────┼────────────────────────┐
         ▼                        ▼                        ▼
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│ Smart Branch    │    │ Conflict        │    │ Search Engine   │
│ Operations      │    │ Resolver        │    │                 │
│                 │    │                 │    │                 │
│ • Health Check  │    │ • Detect        │    │ • Index Build   │
│ • Auto Pull     │    │   Conflicts     │    │ • Query Parse   │
│ • Merge         │    │ • Suggest       │    │ • Result Rank   │
│   Strategy      │◄──►│   Resolution    │◄──►│ • Filter Chain  │
│ • Risk          │    │ • Apply Fixes   │    │ • Cache Results │
│   Assessment    │    │ • Validate      │    └─────────────────┘
└─────────────────┘    │   Changes       │                ▲
         ▲              └─────────────────┘                │
         │                       ▲                        │
         │                       │                        │
         └───────────────────────┼────────────────────────┘
                                 ▼
                    ┌─────────────────────────┐
                    │     AI Integration      │
                    │                         │
                    │ • Pattern Recognition   │
                    │ • Suggestion Engine     │
                    │ • Auto-completion       │
                    │ • Predictive Analysis   │
                    └─────────────────────────┘
```

## 性能优化策略

### 1. 渲染优化

```
Component Render Optimization:
┌─────────────────────────────────────────┐
│              Render Pipeline             │
└─────────────────────────────────────────┘
                    │
                    ▼
          ┌─────────────────┐
          │  Dirty Checking │
          │                 │
          │ • State Compare │
          │ • Props Diff    │
          │ • Skip if Same  │
          └─────────────────┘
                    │ (if dirty)
                    ▼
          ┌─────────────────┐
          │ Virtual Render  │
          │                 │
          │ • Build VDOM    │
          │ • Diff Against  │
          │   Previous      │
          └─────────────────┘
                    │
                    ▼
          ┌─────────────────┐
          │ Selective       │
          │ Re-render       │
          │                 │
          │ • Update Only   │
          │   Changed Parts │
          │ • Batch Updates │
          └─────────────────┘
```

### 2. 数据流优化

```
Data Flow Optimization:
┌─────────────────────────────────────────┐
│             Caching Strategy             │
└─────────────────────────────────────────┘
                    │
         ┌──────────┼──────────┐
         ▼          ▼          ▼
┌──────────────┐ ┌──────────────┐ ┌──────────────┐
│ Memory Cache │ │   Disk Cache │ │ Smart Cache  │
│              │ │              │ │              │
│ • LRU Policy │ │ • Persistent │ │ • Predictive │
│ • Fast Access│ │   Storage    │ │   Loading    │
│ • Size Limit │ │ • Large Data │ │ • Background │
│              │ │   Sets       │ │   Refresh    │
└──────────────┘ └──────────────┘ └──────────────┘
```

这个组件关系图和数据流程设计文档提供了完整的架构视图，展示了组件之间的依赖关系、通信模式和数据流转过程，为TUI界面的开发提供了清晰的架构指导。