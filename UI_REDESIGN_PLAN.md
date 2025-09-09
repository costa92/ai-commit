# Enhanced TUI UI Flow Redesign

## 设计原则

1. **层级导航**：采用层级结构，而非平行 Tab 结构
2. **面包屑导航**：显示当前位置路径
3. **统一的返回机制**：ESC 或 Backspace 返回上一级
4. **上下文相关**：根据当前选择动态显示相关内容

## UI 层级结构

### Level 0: 主界面（Main Menu）
```
┌─────────────────────────────────────────┐
│ Git Repository Overview                 │
├─────────────────────────────────────────┤
│ > Branches (5)                          │
│   Tags (12)                             │
│   Remotes (2)                           │
│   Current Branch Log                    │
│   Query History                         │
├─────────────────────────────────────────┤
│ [Enter] Select  [q] Quit                │
└─────────────────────────────────────────┘
```

### Level 1: 分支/标签/远程列表
```
┌─────────────────────────────────────────┐
│ Main > Branches                         │
├─────────────────────────────────────────┤
│ > * main                                │
│     feature/new-ui                      │
│     feature/auth                        │
│     hotfix/bug-123                      │
├─────────────────────────────────────────┤
│ [Enter] View Commits  [c] Checkout      │
│ [ESC] Back  [q] Quit                    │
└─────────────────────────────────────────┘
```

### Level 2: 提交历史
```
┌─────────────────────────────────────────────────────────┐
│ Main > Branches > main                                  │
├─────────────────────────────────────────────────────────┤
│ > a8e31de feat: Add new feature         2025-09-09     │
│   b7d42cf fix: Fix bug in auth          2025-09-08     │
│   c6e53bg docs: Update README           2025-09-07     │
├─────────────────────────────────────────────────────────┤
│ [Enter] View Diff  [ESC] Back  [q] Quit                 │
└─────────────────────────────────────────────────────────┘
```

### Level 3: Diff 视图
```
┌─────────────────────────────────────────────────────────┐
│ Main > Branches > main > a8e31de                        │
├─────────────────────────────────────────────────────────┤
│ Files Changed (3)          │ Diff                       │
│ > src/main.rs              │ @@ -10,5 +10,8 @@         │
│   src/lib.rs               │ -old line                  │
│   README.md                │ +new line                  │
│                            │ +another new line          │
├─────────────────────────────────────────────────────────┤
│ [j/k] Navigate Files  [J/K] Scroll Diff  [ESC] Back    │
└─────────────────────────────────────────────────────────┘
```

## 导航逻辑

### 前进导航
- **Enter**: 进入下一级
- **Tab**: 在同级别的不同选项间切换（如果有多个面板）

### 后退导航
- **ESC**: 返回上一级
- **Backspace**: 返回上一级（备选）
- **q**: 在主界面退出，在子界面返回上一级

### 快捷操作
- **c**: Checkout（在分支/标签视图）
- **p**: Pull（在任何视图）
- **/**: 搜索（在列表视图）
- **r**: 刷新当前视图

## 状态管理

### ViewStack（视图栈）
```rust
enum View {
    MainMenu,
    BranchList,
    TagList,
    RemoteList,
    CommitList { ref_name: String },
    DiffView { commit_hash: String },
    QueryHistory,
}

struct ViewStack {
    stack: Vec<View>,
    current_index: usize,
}
```

### 导航方法
- `push_view(view: View)`: 进入新视图
- `pop_view()`: 返回上一级
- `replace_view(view: View)`: 替换当前视图
- `can_go_back()`: 检查是否可以返回

## 实现步骤

1. **创建 ViewStack 结构**
   - 管理视图历史
   - 支持前进/后退

2. **重构 App 结构**
   - 移除 Tab 系统
   - 添加 ViewStack
   - 添加面包屑路径

3. **实现主菜单**
   - 显示所有可用选项
   - 处理选择逻辑

4. **实现层级导航**
   - Enter 进入下一级
   - ESC 返回上一级
   - 维护选择状态

5. **优化渲染逻辑**
   - 根据当前视图渲染
   - 显示面包屑导航
   - 显示上下文相关的快捷键提示

## 优势

1. **清晰的层级关系**：用户始终知道自己在哪里
2. **一致的导航体验**：Enter 进入，ESC 返回
3. **上下文相关**：只显示当前可用的操作
4. **减少认知负担**：不需要记住复杂的快捷键
5. **更好的可发现性**：主菜单展示所有功能