# 分支 Git Log 显示问题 - 最终修复方案

## 🔧 问题分析
基于您的截图，问题在于：
1. **在 Branches 视图中**，左侧应该显示分支信息，但显示的是错误内容
2. **右侧 Git Log** 显示的是全部提交，而不是当前选中分支的提交历史

## ✅ 完整修复方案

### 1. **分支选择实时更新** (`src/tui_unified/components/views/branches.rs`)

```rust
// 添加方向键监听，实时更新选中分支
KeyCode::Up | KeyCode::Down | KeyCode::Char('k') | KeyCode::Char('j') => {
    let old_selection = self.list_widget.selected_index();
    let result = self.list_widget.handle_key_event(key, state);
    let new_selection = self.list_widget.selected_index();
    
    // 如果选择发生变化，更新应用状态中的选中分支
    if old_selection != new_selection {
        self.update_selected_branch_in_state(state);
    }
    
    result
}

// 新增同步选中分支到应用状态的方法
pub fn update_selected_branch_in_state(&self, state: &mut AppState) {
    if let Some(selected_branch) = self.selected_branch() {
        state.select_branch(selected_branch.name.clone());
    }
}
```

### 2. **分支视图渲染优化** (`src/tui_unified/app.rs`)

```rust
// 在 Branches 视图渲染时，实时更新右侧 Git Log
crate::tui_unified::state::app_state::ViewType::Branches => {
    // 确保选中分支状态是最新的
    self.branches_view.update_selected_branch_in_state(&mut state);
    
    // 渲染分支列表
    self.branches_view.render(frame, chunks[0], &*state);
    
    // 根据选中的分支更新Git Log
    let selected_branch = state.selected_items.selected_branch.clone();
    self.git_log_view.set_branch_filter(selected_branch.clone());
    
    // 获取并显示选中分支的提交历史
    let commits_to_show = if let Some(ref branch_name) = selected_branch {
        self.get_branch_commits_sync(branch_name).unwrap_or_else(|_| {
            state.repo_state.commits.clone()
        })
    } else {
        state.repo_state.commits.clone()
    };
    
    self.git_log_view.update_commits(commits_to_show);
    self.git_log_view.render(frame, chunks[1], &*state);
}
```

### 3. **Git Log 分支过滤** (`src/tui_unified/components/views/git_log.rs`)

```rust
// 添加分支过滤支持
pub struct GitLogView {
    // ... 其他字段
    current_branch_filter: Option<String>, // 新增：当前过滤的分支
}

pub fn set_branch_filter(&mut self, branch_name: Option<String>) {
    self.current_branch_filter = branch_name;
    self.update_title(); // 更新标题显示分支名
}

fn update_title(&mut self) {
    let title = if let Some(ref branch_name) = self.current_branch_filter {
        format!("Git Log - {}", branch_name)
    } else {
        "Git Log".to_string()
    };
    // ... 重新创建 ListWidget 以更新标题
}
```

### 4. **分支提交历史获取** (`src/tui_unified/app.rs`)

```rust
// 新增获取特定分支提交历史的方法
fn get_branch_commits_sync(&self, branch_name: &str) -> anyhow::Result<Vec<Commit>> {
    let output = Command::new("git")
        .args([
            "log",
            branch_name,
            "--pretty=format:%H╬%an╬%ae╬%ai╬%s",
            "--max-count=100",
        ])
        .output()?;

    if !output.status.success() {
        return Err(anyhow::anyhow!("Failed to get branch commits: {}", 
            String::from_utf8_lossy(&output.stderr)));
    }

    // 解析 git log 输出并创建 Commit 对象
    // ... 解析逻辑
}
```

### 5. **侧边栏适配** (`src/tui_unified/components/panels/sidebar.rs`)

```rust
// 在 Git Log 视图中显示分支相关信息
match state.current_view {
    ViewType::GitLog => {
        let selected_branch_info = if let Some(ref branch_name) = state.selected_items.selected_branch {
            format!(
                "📋 Repository: {}\n\n🔍 Viewing Branch: {}\n📝 Showing commits for: {}\n\n",
                repo_summary.name,
                branch_name,
                branch_name
            )
        } else {
            // 显示全部提交的信息
        };
        (selected_branch_info, false) // 显示分支列表而不是导航菜单
    }
    _ => {
        // 其他视图显示标准信息
    }
}
```

## 🎯 修复后的预期行为

### Branches 视图 (`按 '2'`)
- **左侧**: 分支列表，当前分支用 `*` 标记，选中分支用 `►` 标记
- **右侧**: 显示当前选中分支的提交历史
- **实时更新**: 用方向键选择不同分支时，右侧Git Log实时更新

### Git Log 视图 (`从Branches按 '1'`)
- **左侧侧边栏**: 显示选中分支信息和分支列表
- **右侧内容区**: 显示该分支的提交历史
- **标题**: 显示 "Git Log - [分支名]"

## 🧪 测试方式

1. **启动应用**: `./ai-commit --tui-unified`
2. **进入分支视图**: 按 `2`
3. **选择分支**: 用方向键选择不同分支
4. **验证实时更新**: 右侧Git Log应该实时显示选中分支的提交
5. **切换到Git Log**: 按 `1`，验证显示正确的分支提交和标题

## 🔍 关键技术点

1. **实时选择监听**: 通过拦截方向键事件，在选择变化时立即更新应用状态
2. **分支提交过滤**: 使用 `git log [branch_name]` 获取特定分支的提交历史
3. **视图状态同步**: 确保 Git Log 视图的分支过滤状态与应用状态同步
4. **渲染时更新**: 在 Branches 视图渲染时，实时更新右侧 Git Log 内容

## 📝 修复文件汇总

- `src/tui_unified/components/views/branches.rs` - 分支选择监听
- `src/tui_unified/components/views/git_log.rs` - 分支过滤和标题更新  
- `src/tui_unified/app.rs` - 分支提交获取和渲染逻辑
- `src/tui_unified/components/panels/sidebar.rs` - 侧边栏内容适配

现在分支视图应该能正确显示：左侧分支列表 + 右侧选中分支的提交历史！