# TUI 布局优化设计方案

## 当前布局分析

### 现有问题

基于截图分析，当前 `--tui-unified` 界面存在以下布局问题：

1. **侧边栏布局混乱**
   - Repository 信息、Branches 列表和 Menu 导航混合在一个区域
   - 信息层次不清晰，用户难以快速定位功能
   - 分支列表占用过多空间，影响菜单可见性

2. **头部菜单缺失**
   - 没有清晰的顶部导航栏
   - 用户需要在侧边栏中寻找功能入口
   - 缺少快速切换的视觉提示

3. **内容区域固定**
   - Git Log 视图占据主要内容区域
   - 无法根据选择的菜单项动态切换内容
   - Stash 等其他功能缺乏独立展示区域

4. **焦点管理复杂**
   - 多个交互区域混合在侧边栏中
   - Tab 键切换逻辑不直观
   - 用户体验不够流畅

## 新布局设计方案

### 设计目标

- **清晰的信息层次**：头部导航 + 左侧状态 + 主内容区域
- **直观的菜单切换**：顶部 Tab 式导航，快速功能切换
- **动态内容区域**：根据选择的菜单项切换对应的内容视图
- **简化的交互逻辑**：减少嵌套焦点，提升用户体验

### 新布局结构

**正常状态布局：**
```
┌─────────────────────────────────────────────────────────────────────────────┐
│ Header Navigation Bar                                                       │
│ [Branches] [Tags] [●Stash] [Remotes] [History]                             │
├─────────────────────────────────────────────────────────────────────────────┤
│                    │                                                        │
│  Repository Info   │           Main Content Area (Interactive)              │
│  ┌──────────────┐  │                                                        │
│  │ 📋 cms-api   │  │  Content changes based on header selection:            │
│  │ 🔀 main     │  │                                                        │
│  │ 📊 Stats    │  │  • Branches → Branch management with actions           │
│  │ [Clean]     │  │  • Tags → Tag list with operations                     │
│  └──────────────┘  │  • Stash → Git Log ⟷ Stash toggle (ENHANCED)        │
│                    │           ┌─ Interactive Git Log ─────────────┐       │
│  Dynamic List:     │           │ ► 18c4a3a fix(ui): 修复布局问题    │       │
│  ┌──────────────┐  │           │   8502563 fix(src): 修复焦点管理  │       │
│  │ ★ main       │  │           │   5f49518 feat: 添加diff修改块   │       │
│  │   feature/   │  │           │ [r]Reword [a]Amend [s]Squash     │       │
│  │   auth       │  │           │ [Enter]Diff [d]Details            │       │
│  │   develop    │  │           └───────────────────────────────────┘       │
│  └──────────────┘  │  • Remotes → Remote repositories management           │
│                    │  • History → Query history and search                 │
│                    │                                                        │
│  Commit Editor     │  Left sidebar syncs with header selection             │
│  (Modal Overlay)   │  Both areas support rich interactions                 │
├─────────────────────────────────────────────────────────────────────────────┤
│ Status: [NORMAL] Focus: Content | View: Stash | r-reword, Tab-focus, q-quit │
└─────────────────────────────────────────────────────────────────────────────┘
```

**Commit 编辑模态状态：**
```
┌─────────────────────────────────────────────────────────────────────────────┐
│ Header Navigation Bar                                                       │
│ [Branches] [Tags] [●Stash] [Remotes] [History]                             │
├─────────────────────────────────────────────────────────────────────────────┤
│              ┌─ Edit Commit Message ─────────────────────────┐              │
│              │                                               │              │
│              │ Editing: 18c4a3a [2024-09-11 01:44]         │              │
│              │ Author: costa <costa@helltalk.com>           │              │
│              │                                               │              │
│              │ ┌─ Original Message ─────────────────────┐   │              │
│              │ │ fix(ui): 修复侧边栏布局问题              │   │              │
│              │ └─────────────────────────────────────────┘   │              │
│              │                                               │              │
│              │ ┌─ New Message ───────────────────────────┐   │              │
│              │ │ fix(ui): 重构侧边栏布局系统             │   │              │
│              │ │ │                                       │   │              │
│              │ │ • 分离仓库状态和动态列表组件             │   │              │
│              │ │ • 优化焦点管理和交互逻辑                 │   │              │
│              │ │ • 改善信息层次和用户体验                 │   │              │
│              │ └─────────────────────────────────────────┘   │              │
│              │                                               │              │
│              │ [Ctrl+S] Save  [Esc] Cancel  [Ctrl+P] AI     │              │
│              └───────────────────────────────────────────────┘              │
├─────────────────────────────────────────────────────────────────────────────┤
│ Status: [EDIT] Commit Editor Active | Ctrl+S-save, Esc-cancel               │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 核心改进点

#### 1. 头部导航栏 (Header Navigation)

**特性：**
- 水平 Tab 式布局：`[Branches] [Tags] [Stash] [Remotes] [History]`
- 移除 Git Log 独立菜单项（Git Log 整合到 Stash 切换中）
- 高亮显示当前选中的功能模块
- 支持数字键快速切换（1-5）

**视觉设计：**
```
┌─[Branches]─[Tags]─[●Stash]─[Remotes]─[History]──────────────────────────────┐
│                                                                             │
```
- 选中项用 `●` 或高亮背景标识
- 未选中项保持普通样式
- 支持鼠标点击和键盘导航

#### 2. 左侧边栏动态内容

**设计原则：**
- 上半部分：固定的仓库状态信息
- 下半部分：根据头部菜单选择动态显示对应列表

**固定仓库状态区域：**
```
┌─ Repository Status ─┐
│ 📋 cms-api          │
│ 🔀 main            │
│ 📊 1024 commits    │
│ 🌿 45 branches     │
│ 🏷️ 12 tags         │
│ 📡 3 remotes       │
│ 💾 2 stashes       │
│ [Clean/Dirty]       │
└─────────────────────┘
```

**动态列表区域（根据头部菜单选择）：**

当选择 **Branches** 时：
```
┌─ Branches List ─────┐
│ ★ main              │
│   feature/auth      │
│   develop          │
│   hotfix/v1.2      │
│   release/v2.0     │
│ ...                │
└─────────────────────┘
```

当选择 **Tags** 时：
```
┌─ Tags List ─────────┐
│ v2.1.0  [09-11]    │
│ v2.0.5  [09-08]    │
│ v2.0.0  [08-15]    │
│ v1.9.9  [08-01]    │
│ ...                │
└─────────────────────┘
```

当选择 **Stash** 时：
```
┌─ Stash List ────────┐
│ [0] WIP: feature    │
│ [1] temp changes    │
│ [2] backup work     │
│ ...                │
└─────────────────────┘
```

当选择 **Remotes** 或 **History** 时：
```
┌─ Quick Actions ─────┐
│ [Fetch] [Pull]      │
│ [Push] [Sync]       │
│                     │
│ Recent:             │
│ • Last fetch: 2h    │
│ • Last push: 1d     │
└─────────────────────┘
```

#### 3. 内容区域与侧边栏联动设计

**核心原则：** 头部导航作为主控制器，同时驱动左侧边栏列表和主内容区域的变化

**联动逻辑表：**

| 头部选择 | 左侧边栏显示 | 主内容区域显示 | 交互说明 |
|---------|-------------|---------------|----------|
| **Branches** | Branches List | Branch Management Interface | 左侧选择分支，右侧显示分支详情和操作 |
| **Tags** | Tags List | Tag Details & Operations | 左侧选择标签，右侧显示标签信息和管理 |
| **Stash** | Stash List | Git Log ⟷ Stash Toggle | 左侧选择stash项，右侧切换Git Log/Stash视图 |
| **Remotes** | Quick Actions | Remote Management | 左侧显示快捷操作，右侧显示远程仓库详情 |
| **History** | Quick Actions | Query History Interface | 左侧显示搜索快捷键，右侧显示历史查询 |

**详细内容区域设计：**

**Branches 主内容区域：**
```
┌─ Branch Management ────────────────────────────────────────┐
│                                                            │
│ Selected: ★ main                                           │
│ ┌────────────────────────────────────────────────────────┐ │
│ │ 📊 Branch Details:                                     │ │
│ │ • Upstream: origin/main                                │ │
│ │ • Ahead: 2 commits    Behind: 0 commits               │ │
│ │ • Last commit: fix(ui): 修复侧边栏布局问题              │ │
│ │ • Author: costa       Date: 2024-09-11                │ │
│ └────────────────────────────────────────────────────────┘ │
│                                                            │
│ 🔧 Actions:                                               │
│ [Checkout] [Create New] [Delete] [Merge] [Push] [Pull]    │
│                                                            │
│ 📜 Recent Commits (from selected branch):                 │
│ • 18c4a3a fix(ui): 修复侧边栏布局问题                      │
│ • 8502563 fix(src): 修复TUI界面焦点管理                    │
│ • 5f49518 feat(diff_viewer): 添加 Diff 修改块            │
└────────────────────────────────────────────────────────────┘
```

**Tags 主内容区域：**
```
┌─ Tag Management ───────────────────────────────────────────┐
│                                                            │
│ Selected: v2.1.0                                          │
│ ┌────────────────────────────────────────────────────────┐ │
│ │ 🏷️ Tag Details:                                        │ │
│ │ • Date: 2024-09-11 15:45:32                           │ │
│ │ • Commit: 18c4a3a                                     │ │
│ │ • Message: Latest release                             │ │
│ │ • Tagger: costa <costa@helltalk.com>                  │ │
│ └────────────────────────────────────────────────────────┘ │
│                                                            │
│ 🔧 Actions:                                               │
│ [Create Tag] [Delete] [Push Tag] [Compare] [Show Diff]     │
│                                                            │
│ 📋 Tag Comparison:                                        │
│ • v2.1.0 ← v2.0.5: 15 commits, 23 files changed         │
│ • Files: +1,847 -892 lines                              │
└────────────────────────────────────────────────────────────┘
```

**Stash 主内容区域（特殊双模式切换）：**
```
┌─ Content View ─────────────────────────────────────────────┐
│ Mode Toggle: [●Git Log] [Stash Entries]    (g/s to toggle)│
│ ┌────────────────────────────────────────────────────────┐ │
│ │                                                        │ │
│ │ [Git Log Mode] - Interactive Commit History            │ │
│ │ ► 18c4a3a [09-11 01:44] fix(ui): 修复侧边栏布局问题    │ │
│ │   8502563 [09-11 01:44] fix(src): 修复TUI界面焦点管理  │ │
│ │   5f49518 [09-11 01:44] feat: 添加 Diff 修改块       │ │
│ │                                                        │ │
│ │ Quick Actions for Selected Commit:                     │ │
│ │ [r] Reword  [Enter] Show Diff  [a] Amend              │ │
│ │                                                        │ │
│ │ 或显示                                                  │ │
│ │                                                        │ │
│ │ [Stash Entries Mode]                                   │ │
│ │ 💾 stash@{0}: WIP on main: 18c4a3a fix ui             │ │
│ │ 💾 stash@{1}: temp work on feature branch             │ │
│ │                                                        │ │
│ └────────────────────────────────────────────────────────┘ │
│ Actions: [Apply] [Pop] [Drop] [Show Diff] [Create Stash]   │
└────────────────────────────────────────────────────────────┘
```

#### 4. Git Diff Viewer 集成设计

**Diff 查看器模态界面：**

当按 `Enter` 或 `d` 键查看 commit 差异时，打开全屏 Diff Viewer：

**Unified 模式 (默认)：**
```
┌─────────────────────────────────────────────────────────────────────────────┐
│ Commit Info                                                                 │
│ Commit: 9bd566fb | Files: 10 | Mode: Unified (1)                          │
├─────────────────────────────────────────────────────────────────────────────┤
│ ▼ Original: src/core/ai/agents/commit_agent.rs                             │
│ @@ -194,18 +194,53 @@ impl CommitAgent {                                     │
│     0                                                                       │
│     1         /// 清理提交消息                                               │
│     2     fn clean_commit_message(&self, message: &str) -> String {         │
│ -   3         // 只取第一行，去除多余空白和引号                                │
│ -   4         let cleaned = message.lines()                                 │
│ -   5             .next()                                                   │
│ +   3         // 寻找符合 Conventional Commits 格式的行                      │
│ +   4         for line in message.lines() {                                │
│ +   5             let trimmed_line = line.trim();                          │
│ +   6                                                                       │
│ +   7         // 跳过空行和明显的解释性文本                                  │
│ +   8         if trimmed_line.is_empty() ||                                │
│ +   9             trimmed_line.starts_with("Here") ||                      │
│ +  10             trimmed_line.starts_with("Based") ||                     │
│ +  11             trimmed_line.starts_with("The") ||                       │
│ +  12             trimmed_line.starts_with("This") ||                      │
│ +  13             trimmed_line.starts_with("Analysis")                     │
│ +  14         {                                                            │
│ +  15             continue;                                                │
│                                                                             │
├─────────────────────────────────────────────────────────────────────────────┤
│ Controls                                                                    │
│ File 1/10 | Scroll: 0 | View Mode: Unified | Keys: 1-Unified 2-Side-by-Side 3-Split q-Close │
└─────────────────────────────────────────────────────────────────────────────┘
```

**Side-by-Side 模式：**
```
┌─────────────────────────────────────────────────────────────────────────────┐
│ Commit Info                                                                 │
│ Commit: 9bd566fb | Files: 10 | Mode: Side-by-Side (2)                     │
├─────────────────────────────────────────────────────────────────────────────┤
│ ▼ Original: src/core/ai/agents/commit_agent.rs ──┬── ▼ Modified: src/core/ai/agents/commit_agent.rs │
│ @@ -194,18 +194,53 @@ impl CommitAgent {           │ @@ -194,18 +194,53 @@ impl CommitAgent {         │
│   0                                               │   0                                             │
│   1     /// 清理提交消息                           │   1     /// 清理提交消息                         │
│   2   fn clean_commit_message(&self, message: &str) → │   2   fn clean_commit_message(&self, message: &str) → │
│ - 3     // 只取第一行，去除多余空白和引号            │ + 3     // 寻找符合 Conventional Commits 格式的行  │
│ - 4     let cleaned = message.lines()              │ + 4     for line in message.lines() {            │
│ - 5         .next()                                │ + 5         let trimmed_line = line.trim();       │
│                                                   │ + 6                                             │
│                                                   │ + 7     // 跳过空行和明显的解释性文本            │
│                                                   │ + 8     if trimmed_line.is_empty() ||          │
│                                                   │ + 9         trimmed_line.starts_with("Here") || │
│                                                   │ + 10        trimmed_line.starts_with("Based") ││
│                                                   │ + 11        trimmed_line.starts_with("The") || │
│                                                   │ + 12        trimmed_line.starts_with("This") ││
│                                                   │ + 13        trimmed_line.starts_with("Analysis") │
│                                                   │ + 14    {                                       │
│                                                   │ + 15        continue;                           │
├─────────────────────────────────────────────────────────────────────────────┤
│ Controls                                                                    │
│ File 1/10 | Scroll: 0 | View Mode: Side-by-Side | Keys: 1-Unified 2-Side-by-Side 3-Split q-Close │
└─────────────────────────────────────────────────────────────────────────────┘
```

**Split 模式：**
```
┌─────────────────────────────────────────────────────────────────────────────┐
│ Commit Info                                                                 │
│ Commit: 9bd566fb | Files: 10 | Mode: Split (3)                            │
├─────────────────────────────────────────────────────────────────────────────┤
│ ▼ File: src/core/ai/agents/commit_agent.rs                                │
│ ┌─ Original ─────────────────────────────────────────────────────────────┐ │
│ │ @@ -194,18 +194,53 @@ impl CommitAgent {                                │ │
│ │   0                                                                    │ │
│ │   1     /// 清理提交消息                                                │ │
│ │   2   fn clean_commit_message(&self, message: &str) -> String {        │ │
│ │ - 3     // 只取第一行，去除多余空白和引号                               │ │
│ │ - 4     let cleaned = message.lines()                                  │ │
│ │ - 5         .next()                                                    │ │
│ └────────────────────────────────────────────────────────────────────────┘ │
│ ┌─ Modified ─────────────────────────────────────────────────────────────┐ │
│ │ @@ -194,18 +194,53 @@ impl CommitAgent {                                │ │
│ │   0                                                                    │ │
│ │   1     /// 清理提交消息                                                │ │
│ │   2   fn clean_commit_message(&self, message: &str) -> String {        │ │
│ │ + 3     // 寻找符合 Conventional Commits 格式的行                       │ │
│ │ + 4     for line in message.lines() {                                  │ │
│ │ + 5         let trimmed_line = line.trim();                            │ │
│ │ + 6                                                                    │ │
│ │ + 7     // 跳过空行和明显的解释性文本                                   │ │
│ │ + 8     if trimmed_line.is_empty() ||                                 │ │
│ └────────────────────────────────────────────────────────────────────────┘ │
├─────────────────────────────────────────────────────────────────────────────┤
│ Controls                                                                    │
│ File 1/10 | Scroll: 0 | View Mode: Split | Keys: 1-Unified 2-Side-by-Side 3-Split q-Close │
└─────────────────────────────────────────────────────────────────────────────┘
```

**Diff Viewer 交互逻辑：**

**模式切换：**
- `1` - 切换到 Unified 模式（单列显示，+/- 标记）
- `2` - 切换到 Side-by-Side 模式（左右对比）
- `3` - 切换到 Split 模式（上下分割显示）

**导航操作：**
- `↑`/`↓` 或 `j`/`k` - 行内滚动
- `PgUp`/`PgDn` 或 `u`/`d` - 页面滚动
- `Home`/`End` 或 `g`/`G` - 跳到开始/结束
- `←`/`→` 或 `h`/`l` - 在 Side-by-Side 模式下左右焦点切换

**文件导航：**
- `n` - 下一个文件
- `p` - 上一个文件
- `f` - 显示文件列表，快速跳转

**其他操作：**
- `q` 或 `Esc` - 关闭 Diff Viewer，返回 Git Log
- `/` - 搜索模式，在 diff 内容中搜索
- `w` - 切换空白字符显示/隐藏
- `t` - 切换 Tab 字符可视化

**文件头信息显示：**
```
┌─ File Header ──────────────────────────────────────────┐
│ 📁 src/core/ai/agents/commit_agent.rs                 │
│ 📊 Changes: +18 lines, -5 lines                       │
│ 🔍 Binary: No | Mode: 100644 → 100644                │
│ 📝 Status: Modified                                   │
└────────────────────────────────────────────────────────┘
```

#### 5. Git Log 快速编辑功能

**Commit 信息修改支持：**

当在 Git Log 模式下选中某个 commit 时，支持以下快速编辑操作：

```
Selected Commit: ► 18c4a3a fix(ui): 修复侧边栏布局问题

Available Actions:
┌─ Commit Edit Options ──────────────────────────────────────┐
│ [r] Reword Message    - 修改 commit 信息（git rebase -i）  │
│ [a] Amend Last       - 修改最后一个 commit（仅限最新）     │
│ [s] Squash          - 压缩到上一个 commit                  │
│ [d] Show Diff       - 查看变更详情                        │
│ [Enter] Quick Diff  - 弹出 diff 查看器                    │
└────────────────────────────────────────────────────────────┘
```

**Reword 模式界面设计：**
```
┌─ Edit Commit Message ──────────────────────────────────────┐
│                                                            │
│ Editing: 18c4a3a [2024-09-11 01:44]                       │
│ Author: costa <costa@helltalk.com>                         │
│                                                            │
│ ┌─ Original Message ────────────────────────────────────┐   │
│ │ fix(ui): 修复侧边栏布局问题                             │   │
│ └──────────────────────────────────────────────────────┘   │
│                                                            │
│ ┌─ New Message ─────────────────────────────────────────┐   │
│ │ fix(ui): 修复侧边栏布局问题                             │   │
│ │ │                                                     │   │
│ │ Additional details can be added here...               │   │
│ └──────────────────────────────────────────────────────┘   │
│                                                            │
│ 💡 Tips:                                                   │
│ • First line should be under 50 chars                     │
│ • Use conventional commit format: type(scope): message    │
│ • Separate body with blank line                           │
│                                                            │
│ [Ctrl+S] Save & Apply  [Esc] Cancel  [Ctrl+P] AI Enhance  │
└────────────────────────────────────────────────────────────┘
```

**AI 增强功能（可选）：**
```
┌─ AI Commit Message Enhancement ────────────────────────────┐
│                                                            │
│ Original: fix(ui): 修复侧边栏布局问题                       │
│                                                            │
│ 🤖 AI Suggestions:                                         │
│ ┌────────────────────────────────────────────────────────┐ │
│ │ 1. fix(ui): 重构侧边栏布局，解决信息层次混乱问题         │ │
│ │                                                        │ │
│ │ • 分离仓库状态和菜单导航                                │ │
│ │ • 优化焦点管理逻辑                                     │ │
│ │ • 改善用户交互体验                                     │ │
│ └────────────────────────────────────────────────────────┘ │
│                                                            │
│ │ 2. fix(tui): 修复侧边栏组件布局和焦点管理问题            │ │
│ │                                                        │ │
│ │ 解决了侧边栏中Repository信息、分支列表和菜单导航         │ │
│ │ 混合显示导致的信息层次不清和交互复杂的问题               │ │
│ └────────────────────────────────────────────────────────┘ │
│                                                            │
│ [1-2] Select  [Enter] Use Selected  [Esc] Manual Edit     │
└────────────────────────────────────────────────────────────┘
```

### 交互逻辑设计

#### 1. 导航切换

**键盘快捷键：**
- `1` - Branches 视图
- `2` - Tags 视图  
- `3` - Stash 视图（默认显示 Git Log）
- `4` - Remotes 视图
- `5` - History 视图

**焦点管理：**
- `Tab` 键在头部导航和主内容区之间切换
- 头部导航内使用左右箭头键切换
- 主内容区内使用上下箭头键和 Enter 键操作

#### 2. Git Log 交互逻辑

**Git Log 视图快捷键：**
- `↑`/`↓` 或 `j`/`k` - 在 commit 列表中导航
- `Enter` - 显示选中 commit 的 diff 详情（全屏 Diff Viewer）
- `d` - 显示选中 commit 的详细 diff（同 Enter）
- `r` - 重写选中 commit 的信息（Reword）
- `a` - 修改最后一个 commit（Amend，仅限最新 commit）
- `s` - 压缩选中 commit 到上一个（Squash）
- `Ctrl+P` - 在编辑模式下调用 AI 增强建议

**Diff Viewer 模态界面快捷键：**
- `1` - 切换到 Unified 模式
- `2` - 切换到 Side-by-Side 模式  
- `3` - 切换到 Split 模式
- `↑`/`↓` 或 `j`/`k` - 垂直滚动查看 diff
- `←`/`→` 或 `h`/`l` - 水平滚动或在 Side-by-Side 模式切换焦点
- `PgUp`/`PgDn` 或 `u`/`d` - 页面滚动
- `Home`/`End` 或 `g`/`G` - 跳到开始/结束
- `n` - 切换到下一个修改的文件
- `p` - 切换到上一个修改的文件
- `f` - 显示文件列表，快速跳转到指定文件
- `/` - 在 diff 内容中搜索文本
- `w` - 切换空白字符显示/隐藏
- `t` - 切换 Tab 字符可视化显示
- `q` 或 `Esc` - 关闭 Diff Viewer，返回 Git Log

**Reword 编辑模式：**
- `Ctrl+S` - 保存并应用更改（执行 git rebase）
- `Esc` - 取消编辑，返回 Git Log 视图
- `Ctrl+P` - 调用 AI 增强当前 commit 信息
- 支持多行编辑，第一行为主题，空行后为详细描述

**安全限制：**
- 只能编辑尚未推送到远程的 commit
- 编辑已推送的 commit 时显示警告提示
- Amend 操作仅限于最新的 commit
- Squash 操作不能应用于第一个 commit

#### 3. Stash 视图特殊逻辑

在 Stash 视图中提供内部切换：
- `g` 键：切换到 Git Log 子视图
- `s` 键：切换到 Stash Entries 子视图
- 顶部显示当前子视图状态

#### 3. 状态栏信息

```
[NORMAL] Focus: Content | View: Branches | Tab-focus, ←→-nav, q-quit
```

显示内容：
- 当前模式 (NORMAL/SEARCH/COMMAND)
- 当前焦点区域 (Header/Content) 
- 当前视图 (Branches/Tags/Stash/...)
- 相关快捷键提示

## 实现规划

### Phase 1: 头部导航重构
- 创建 `HeaderNavigation` 组件
- 重构 `SidebarPanel` 移除菜单部分
- 更新焦点管理器支持新的区域

### Phase 2: 内容区域动态化
- 实现内容区域根据头部选择切换视图
- 优化各个视图组件的独立渲染
- 添加 Stash 视图的内部切换逻辑

### Phase 3: 交互优化
- 更新键盘事件路由
- 优化状态栏信息显示
- 添加视图切换的动画效果（可选）

### Phase 4: 测试与完善
- 全面测试各个视图的切换
- 优化响应式布局适配
- 性能优化和用户体验提升

## 技术实现要点

### 1. 组件结构调整

```rust
// 新增头部导航组件
pub struct HeaderNavigation {
    selected_index: usize,
    menu_items: Vec<NavItem>,
    focused: bool,
}

// 简化侧边栏组件
pub struct SidebarPanel {
    // 移除 menu_items 和相关菜单逻辑
    // 专注于仓库状态显示
}

// 内容区域管理器
pub struct ContentAreaManager {
    current_view: ViewType,
    git_log_stash_mode: GitLogStashMode, // Git Log 或 Stash 子模式
}
```

### 2. 状态管理更新

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ViewType {
    Branches,
    Tags, 
    Stash,    // 包含 Git Log 和 Stash Entries 子模式
    Remotes,
    QueryHistory,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GitLogStashMode {
    GitLog,
    StashEntries,
}
```

### 3. 布局管理器扩展

```rust
impl LayoutManager {
    pub fn calculate_header_content_layout(&self, area: Rect) -> HeaderContentLayout {
        // 计算头部导航 + 左侧信息 + 主内容的布局
    }
}
```

## GRV 功能分析与缺失功能补充

### 基于 GRV 的功能对比分析

通过研究 GRV (Git Repository Viewer) 的设计理念，我们识别出以下需要补充的重要功能：

| 功能分类 | 我们已有 | GRV 特性 | 需要补充 |
|---------|----------|----------|----------|
| **视图管理** | 5个基础视图 | 多视图布局、分屏显示 | ✅ 分屏布局支持 |
| **搜索过滤** | 基础搜索 | 高级过滤查询、正则搜索 | ✅ 高级过滤系统 |
| **Git Status** | 基础状态显示 | 完整的 Git Status 视图 | ✅ Git Status 管理界面 |
| **Blame 视图** | ❌ 缺失 | Git Blame 集成 | ✅ File Blame 查看器 |
| **配置系统** | 基础配置 | 运行时配置、主题系统 | ✅ 动态配置界面 |
| **命令模式** | 基础快捷键 | Vim风格命令模式 | ✅ 命令行接口 |
| **Refs 管理** | 基础分支标签 | 完整的 Refs 视图 | ✅ 引用管理系统 |
| **工作区操作** | 基础 commit 编辑 | Stage/Unstage 文件操作 | ✅ 工作区文件管理 |

## 新增功能的布局设计

### 6. Git Status 工作区管理视图

**完整的 Git Status 界面设计：**
```
┌─────────────────────────────────────────────────────────────────────────────┐
│ Header Navigation Bar                                                       │
│ [Branches] [Tags] [Stash] [●Status] [Remotes] [History]                    │
├─────────────────────────────────────────────────────────────────────────────┤
│                    │                                                        │
│  Repository Info   │              Git Status Management                     │
│  ┌──────────────┐  │                                                        │
│  │ 📋 cms-api   │  │  ┌─ Changes to be committed ─────────────────────┐    │
│  │ 🔀 main     │  │  │ ● src/tui/layout/manager.rs         [Modified] │    │
│  │ 📊 Clean    │  │  │ ● docs/new-layout-design.md        [Added]    │    │
│  │ [2 staged]  │  │  │ ● src/components/header.rs          [Renamed]  │    │
│  └──────────────┘  │  └───────────────────────────────────────────────┘    │
│                    │                                                        │
│  Quick Actions:    │  ┌─ Changes not staged for commit ────────────────┐    │
│  ┌──────────────┐  │  │   src/main.rs                      [Modified] │    │
│  │ [c] Commit   │  │  │   README.md                        [Modified] │    │
│  │ [a] Add All  │  │  │   .gitignore                       [Modified] │    │
│  │ [r] Reset    │  │  │                                                │    │
│  │ [d] Discard  │  │  │ Actions: [Space]Stage [u]Unstage [d]Discard   │    │
│  └──────────────┘  │  └───────────────────────────────────────────────┘    │
│                    │                                                        │
│                    │  ┌─ Untracked files ──────────────────────────────┐    │
│                    │  │   temp/debug.log                              │    │
│                    │  │   build/                                      │    │
│                    │  │   node_modules/                               │    │
│                    │  │                                               │    │
│                    │  │ Actions: [a]Add [i]Ignore [D]Delete           │    │
│                    │  └───────────────────────────────────────────────┘    │
├─────────────────────────────────────────────────────────────────────────────┤
│ Status: [STATUS] Focus: Content | Files: 8 changed | Space-stage, c-commit │
└─────────────────────────────────────────────────────────────────────────────┘
```

**Status 视图交互逻辑：**
- `Space` - Stage/Unstage 选中的文件
- `a` - Add 所有文件到暂存区
- `u` - Unstage 选中的文件
- `d` - 丢弃选中文件的修改
- `c` - 打开 Commit 编辑器
- `Enter` - 显示文件的 diff
- `r` - Reset 暂存区
- `i` - 添加到 .gitignore

### 7. File Blame 查看器

**Blame 模态界面设计：**
```
┌─────────────────────────────────────────────────────────────────────────────┐
│ File Blame                                                                  │
│ File: src/tui/layout/manager.rs | Lines: 280 | [b]lame [f]ile [q]uit       │
├─────────────────────────────────────────────────────────────────────────────┤
│ ┌─ Git Blame ─────────────────────────────────────────────────────────────┐ │
│ │ 18c4a3a costa  2024-09-11  1│ use ratatui::layout::{Constraint, Direction, Layout, Rect}; │
│ │ 18c4a3a costa  2024-09-11  2│ use crate::tui_unified::{                                   │
│ │ 8502563 costa  2024-09-10  3│     config::AppConfig,                                      │
│ │ 8502563 costa  2024-09-10  4│     app::LayoutResult                                       │
│ │ 18c4a3a costa  2024-09-11  5│ };                                                          │
│ │ 5f49518 alice  2024-09-09  6│ use super::LayoutMode;                                      │
│ │ 18c4a3a costa  2024-09-11  7│                                                             │
│ │ 8502563 costa  2024-09-10  8│ // 布局常量                                                  │
│ │ 8502563 costa  2024-09-10  9│ pub const MIN_TERMINAL_WIDTH: u16 = 80;                     │
│ │ 8502563 costa  2024-09-10 10│ pub const MIN_TERMINAL_HEIGHT: u16 = 24;                    │
│ │ 18c4a3a costa  2024-09-11 11│ pub const STATUS_BAR_HEIGHT: u16 = 3;                      │
│ └─────────────────────────────────────────────────────────────────────────┘ │
│                                                                             │
│ ┌─ Commit Details ────────────────────────────────────────────────────────┐ │
│ │ Selected: 18c4a3a - fix(ui): 修复侧边栏布局问题                         │ │
│ │ Author: costa <costa@helltalk.com>                                      │ │
│ │ Date: 2024-09-11 15:45:32                                              │ │
│ │                                                                         │ │
│ │ 修复了侧边栏中Repository信息、分支列表和菜单导航                          │ │
│ │ 混合显示导致的信息层次不清和交互复杂的问题                               │ │
│ │                                                                         │ │
│ │ [Enter] Show Full Diff  [c] Checkout  [r] Revert Line                  │ │
│ └─────────────────────────────────────────────────────────────────────────┘ │
├─────────────────────────────────────────────────────────────────────────────┤
│ Controls: ↑↓-navigate, Enter-commit details, c-checkout, r-revert, q-close │
└─────────────────────────────────────────────────────────────────────────────┘
```

**Blame 视图功能：**
- 逐行显示提交信息和作者
- 选中行显示完整 commit 详情
- 支持跳转到 commit 或查看 diff
- 支持 checkout 特定 commit
- 支持 revert 特定行的修改

### 8. 高级搜索与过滤系统

**搜索模式界面：**
```
┌─────────────────────────────────────────────────────────────────────────────┐
│ Advanced Search & Filter                                                    │
│ Mode: [SEARCH] | Type: Commit Search | Pattern: /fix.*ui/                  │
├─────────────────────────────────────────────────────────────────────────────┤
│ ┌─ Search Configuration ─────────────────────┐ ┌─ Search Results ──────────┐ │
│ │                                             │ │                           │ │
│ │ 🔍 Search Target:                           │ │ 📋 Found 12 matches:      │ │
│ │ [●] Commit Messages  [ ] Author Names      │ │                           │ │
│ │ [ ] File Contents    [ ] File Names        │ │ ► 18c4a3a fix(ui): 修复... │ │
│ │ [ ] Commit Hashes    [ ] Branch Names      │ │   8502563 fix(src): 修复... │ │
│ │                                             │ │   5f49518 feat(diff): ... │ │
│ │ 🎯 Filter Options:                          │ │   2a1b3c4 fix(ui): 优化... │ │
│ │ Date Range: [2024-09-01] to [2024-09-11]   │ │   ...                     │ │
│ │ Author: [costa] Branch: [main]              │ │                           │ │
│ │ File Path: [src/tui/*]                      │ │ Actions:                  │ │
│ │                                             │ │ [Enter] View [d] Diff     │ │
│ │ 🔧 Search Type:                             │ │ [f] Filter [c] Clear      │ │
│ │ [●] Regex  [ ] Exact  [ ] Fuzzy             │ │                           │ │
│ │                                             │ │                           │ │
│ │ Pattern: /fix.*ui/___________________       │ │                           │ │
│ │                                             │ │                           │ │
│ │ [Enter] Search  [Esc] Cancel               │ │                           │ │
│ └─────────────────────────────────────────────┘ └───────────────────────────┘ │
├─────────────────────────────────────────────────────────────────────────────┤
│ Status: [SEARCH] 12 results | Enter-apply, Tab-focus, Esc-cancel, ?-help   │
└─────────────────────────────────────────────────────────────────────────────┘
```

**搜索功能特性：**
- 多目标搜索（commit、作者、文件等）
- 日期范围和作者过滤
- 正则表达式、精确匹配、模糊搜索
- 实时搜索结果预览
- 搜索历史保存

### 9. 分屏布局系统

**分屏模式设计：**
```
┌─────────────────────────────────────────────────────────────────────────────┐
│ Header Navigation Bar                                                       │
│ [●Branches] [Tags] [Stash] [Status] [History] | Layout: [Split] [v][h][+]  │
├──────────────────────────────┬──────────────────────────────────────────────┤
│            Panel 1           │               Panel 2                        │
│  Repository Info & Branch    │                                              │
│  ┌──────────────────────────┐│  ┌─ Commit Details ─────────────────────────┐│
│  │ 📋 cms-api              ││  │                                          ││
│  │ 🔀 main                 ││  │ Selected: 18c4a3a                        ││
│  │ 📊 1024 commits         ││  │ Author: costa                            ││
│  └──────────────────────────┘│  │ Date: 2024-09-11                        ││
│                              │  │                                          ││
│  ┌─ Branches List ──────────┐│  │ fix(ui): 修复侧边栏布局问题               ││
│  │ ★ main                   ││  │                                          ││
│  │   feature/auth           ││  │ 修复了侧边栏中Repository信息、             ││
│  │   develop               ││  │ 分支列表和菜单导航混合显示导致的           ││
│  │   hotfix/v1.2           ││  │ 信息层次不清和交互复杂的问题               ││
│  │   release/v2.0          ││  │                                          ││
│  └──────────────────────────┘│  │ Files Changed: 3                         ││
│                              │  │ ┌────────────────────────────────────────┐││
│                              │  │ │ src/tui/layout/manager.rs      [+15-5]│││
│                              │  │ │ src/components/sidebar.rs      [+8-12]│││
│                              │  │ │ docs/design.md                 [+45-0]│││
│                              │  │ └────────────────────────────────────────┘││
│                              │  │                                          ││
│                              │  │ [Enter] Full Diff  [f] Files  [b] Blame ││
│                              │  └──────────────────────────────────────────┘│
├──────────────────────────────┴──────────────────────────────────────────────┤
│ Layout: Split View | [Ctrl+V] Vertical [Ctrl+H] Horizontal [Ctrl+C] Close  │
└─────────────────────────────────────────────────────────────────────────────┘
```

**分屏功能：**
- `Ctrl+V` - 垂直分屏
- `Ctrl+H` - 水平分屏
- `Ctrl+W` - 在面板间切换焦点
- `Ctrl+C` - 关闭分屏，回到单面板
- `Ctrl++`/`Ctrl+-` - 调整面板大小

### 10. 命令模式界面

**Vim 风格命令行：**
```
┌─────────────────────────────────────────────────────────────────────────────┐
│ Header Navigation Bar                                                       │
│ [Branches] [Tags] [●Stash] [Remotes] [History]                             │
├─────────────────────────────────────────────────────────────────────────────┤
│                    │                                                        │
│  Repository Info   │           Main Content Area                            │
│  ┌──────────────┐  │                                                        │
│  │ 📋 cms-api   │  │  ┌─ Interactive Git Log ─────────────┐                │
│  │ 🔀 main     │  │  │ ► 18c4a3a fix(ui): 修复布局问题    │                │
│  │ 📊 Stats    │  │  │   8502563 fix(src): 修复焦点管理  │                │
│  │ [Clean]     │  │  │   5f49518 feat: 添加diff修改块   │                │
│  └──────────────┘  │  │                               │                │
│                    │  │                               │                │
│  Dynamic List:     │  │                               │                │
│  ┌──────────────┐  │  │                               │                │
│  │ ★ main       │  │  │                               │                │
│  │   feature/   │  │  └───────────────────────────────────┘                │
│  │   auth       │  │                                                        │
│  │   develop    │  │                                                        │
│  └──────────────┘  │                                                        │
├─────────────────────────────────────────────────────────────────────────────┤
│ Command Mode: :git checkout -b feature/new-layout______________________     │
├─────────────────────────────────────────────────────────────────────────────┤
│ [COMMAND] Type command | :q-quit :w-write :git-git cmd | Esc-cancel         │
└─────────────────────────────────────────────────────────────────────────────┘
```

**命令系统功能：**
- `:git <command>` - 执行 git 命令
- `:search <pattern>` - 搜索内容
- `:filter <criteria>` - 应用过滤器
- `:set <option>` - 运行时配置
- `:vsplit` / `:hsplit` - 分屏操作
- `:theme <name>` - 切换主题
- `:help` - 显示帮助
- `:q` - 退出

### 11. 动态配置界面

**配置管理模态窗口：**
```
┌─────────────────────────────────────────────────────────────────────────────┐
│ Configuration Management                                                    │
│ [General] [Appearance] [Keybindings] [Git] [Performance]                  │
├─────────────────────────────────────────────────────────────────────────────┤
│ ┌─ General Settings ──────────────────┐ ┌─ Current Values ─────────────────┐ │
│ │                                     │ │                                  │ │
│ │ ⚙️ Core Options:                     │ │ 📝 Active Configuration:         │ │
│ │ [●] Show commit graph               │ │                                  │ │
│ │ [●] Enable mouse support            │ │ commit_graph: true               │ │
│ │ [ ] Auto-refresh repository         │ │ mouse_support: true              │ │
│ │ [●] Show line numbers               │ │ auto_refresh: false              │ │
│ │                                     │ │ line_numbers: true               │ │
│ │ 🎨 Display Options:                  │ │ theme: "dark"                    │ │
│ │ Theme: [Dark ▼] [Custom...]         │ │ date_format: "relative"          │ │
│ │ Date Format: [Relative ▼]           │ │ tab_width: 4                     │ │
│ │ Tab Width: [4____]                  │ │                                  │ │
│ │                                     │ │ 📊 Statistics:                   │ │
│ │ 🔧 Advanced:                         │ │ Config file: ~/.ai-commit.toml   │ │
│ │ Max commits: [1000_____]            │ │ Last modified: 2024-09-11        │ │
│ │ Diff context: [3_____]              │ │ Size: 2.4KB                     │ │
│ │ [●] Syntax highlighting             │ │                                  │ │
│ │                                     │ │ [Apply] [Reset] [Export]         │ │
│ │ [Save] [Reset] [Cancel]             │ │                                  │ │
│ └─────────────────────────────────────┘ └──────────────────────────────────┘ │
├─────────────────────────────────────────────────────────────────────────────┤
│ Config: General | Tab-switch sections, Space-toggle, Enter-apply, Esc-exit │
└─────────────────────────────────────────────────────────────────────────────┘
```

**配置系统特性：**
- 分类配置管理（General、Appearance、Keys等）
- 实时配置预览和应用
- 配置文件导入/导出
- 配置重置和备份
- 主题自定义系统

## 更新后的完整设计流程

### 扩展的头部导航

**新的导航栏设计：**
```
┌─[Branches]─[Tags]─[●Stash]─[Status]─[Remotes]─[History]─[⚙️Config]─[📋Help]─┐
│                                                                             │
```

**新增导航项：**
- `Status` (6) - Git 工作区状态管理
- `Config` (7) - 动态配置界面  
- `Help` (8) - 帮助和快捷键参考

### 增强的视图切换逻辑

**扩展的联动逻辑表：**

| 头部选择 | 左侧边栏显示 | 主内容区域显示 | 特殊功能 |
|---------|-------------|---------------|----------|
| **Branches** | Branches List | Branch Management + Commit Graph | 支持分屏显示详情 |
| **Tags** | Tags List | Tag Details & Comparison | 支持标签对比和创建 |
| **Stash** | Stash List | Git Log ⟷ Stash Toggle | 支持 blame 和搜索 |
| **Status** | File Status | Working Directory Management | Stage/Unstage 操作 |
| **Remotes** | Remotes List | Remote Management + Sync | 远程分支和同步操作 |
| **History** | Search Filters | Advanced Search Interface | 高级搜索和过滤 |
| **Config** | Config Categories | Dynamic Configuration | 实时配置和主题 |
| **Help** | Help Topics | Documentation & Shortcuts | 上下文敏感帮助 |

### 全局功能增强

**新增全局快捷键：**
- `:` - 进入命令模式
- `Ctrl+V` / `Ctrl+H` - 分屏操作
- `Ctrl+W` - 分屏焦点切换
- `b` - 在文件视图中显示 blame
- `?` - 上下文敏感帮助
- `F1` - 全局帮助
- `F12` - 配置界面

这个增强后的设计整合了 GRV 的最佳实践，提供了更专业和完整的 Git 仓库管理体验。