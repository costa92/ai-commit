# AI-Commit 快速参考卡

## 一页纸速查表（打印版）

```
┌─────────────────────────────────────────────────────────────────┐
│                    AI-COMMIT (AIC) 快速参考                      │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  【基础命令】                                                   │
│  aic                       # 等同于 ai-commit                  │
│  aic -h                    # 显示帮助                          │
│  aic --version             # 显示版本                          │
│                                                                 │
│  【提交操作】                                                   │
│  aicg                      # 生成提交消息                      │
│  aicg -y                   # 生成并确认提交                    │
│  aicg -a                   # 添加所有文件并生成                │
│  aicp                      # 生成、提交并推送                  │
│  aicpf                     # 强制推送（解决冲突）              │
│                                                                 │
│  【标签管理】                                                   │
│  aict v1.0.0               # 创建标签                          │
│  aict-push v1.0.0          # 创建并推送标签                    │
│  aictl                     # 列出所有标签                      │
│  aictd v1.0.0              # 删除标签                          │
│                                                                 │
│  【Git Flow】                                                   │
│  aicf init                 # 初始化 Git Flow                   │
│  aicff start NAME          # 开始功能分支                      │
│  aicff finish NAME         # 完成功能分支                      │
│  aicfh start NAME          # 开始修复分支                      │
│  aicfr start VERSION       # 开始发布分支                      │
│                                                                 │
│  【工作树】                                                     │
│  aicw BRANCH               # 创建工作树                        │
│  aicwl                     # 列出工作树                        │
│  aicw-sw NAME              # 切换工作树                        │
│  aicw-rm NAME              # 删除工作树                        │
│                                                                 │
│  【历史查看】                                                   │
│  aich                      # 查看提交历史                      │
│  aich-graph                # 图形化历史                        │
│  aich-author "name"        # 按作者筛选                        │
│  aich-today                # 今日提交                          │
│  aich-stats                # 统计信息                          │
│                                                                 │
│  【配置管理】                                                   │
│  aic-provider NAME         # 设置 AI 提供商                    │
│  aic-model NAME            # 设置 AI 模型                      │
│  aic-config                # 查看所有配置                      │
│                                                                 │
│  【修改操作】                                                   │
│  aic-amend                 # 修改最后提交                      │
│  aic-undo                  # 撤销最后提交                      │
│  aic-rebase HEAD~N         # 交互式变基                        │
│                                                                 │
├─────────────────────────────────────────────────────────────────┤
│  提示: 运行 aic-help 查看所有命令 | 文档: docs/SHORTCUTS.md    │
└─────────────────────────────────────────────────────────────────┘
```

## 场景化快速指南

### 🌅 每日开始工作
```bash
# 1. 查看昨天的工作
aich-yesterday

# 2. 拉取最新代码
git pull

# 3. 创建今日功能分支
aicff start today-work

# 4. 切换到工作树（可选）
aicw feature/today-work
```

### 🚀 快速提交更改
```bash
# 场景 1: 简单提交
aicg                    # 查看生成的消息
aicg -y                 # 确认并提交

# 场景 2: 提交并推送
aicp                    # 一键完成

# 场景 3: 修复刚才的提交
aic-amend "更正的消息"
```

### 🐛 紧急修复 Bug
```bash
# 1. 创建修复分支
aicfh start critical-fix

# 2. 修改代码...

# 3. 提交修复
aicp

# 4. 完成修复并打标签
aicfh finish critical-fix
aict-push v1.0.1-hotfix
```

### 📦 发布新版本
```bash
# 1. 创建发布分支
aicfr start v2.0.0

# 2. 更新版本号、文档等...

# 3. 提交更改
aicg -a -y

# 4. 完成发布
aicfr finish v2.0.0
aict-push v2.0.0
```

### 📊 生成报告
```bash
# 日报
daily-report() {
    echo "=== $(date +%Y-%m-%d) 日报 ==="
    aich-today
    echo "\n未提交更改:"
    git status -s
}

# 周报
weekly-report() {
    echo "=== 本周工作总结 ==="
    aich-week
    aich-stats
}

# 月报
monthly-report() {
    echo "=== 本月绩效 ==="
    aich-month
    aich-contributors
}
```

## 键盘映射建议

### macOS 用户
```bash
# 在 ~/.zshrc 中添加
bindkey -s '^G' 'aicg\n'        # Ctrl+G: 生成提交
bindkey -s '^P' 'aicp\n'        # Ctrl+P: 推送
bindkey -s '^T' 'aict '         # Ctrl+T: 标签
```

### Linux 用户
```bash
# 在 ~/.bashrc 中添加
bind '"\C-g": "aicg\n"'         # Ctrl+G: 生成提交
bind '"\C-p": "aicp\n"'         # Ctrl+P: 推送
bind '"\C-t": "aict "'          # Ctrl+T: 标签
```

### Windows (Git Bash) 用户
```bash
# 在 ~/.bashrc 中添加
alias aicg='winpty ai-commit commit generate'
alias aicp='aicg --push'
```

## 效率对比

| 操作 | 传统方式 | AIC 快捷方式 | 节省击键 |
|------|----------|-------------|----------|
| 生成提交 | `ai-commit commit generate` (27键) | `aicg` (4键) | 85% |
| 提交并推送 | `git add . && git commit -m "..." && git push` (40+键) | `aicp` (4键) | 90% |
| 创建标签 | `git tag v1.0.0 && git push origin v1.0.0` (38键) | `aict-push v1.0.0` (16键) | 58% |
| 功能分支 | `git checkout -b feature/x && git push -u origin feature/x` (50+键) | `aicff start x` (13键) | 74% |

## 个性化配置模板

```bash
# === ~/.aic_custom ===
# 个人 AI-Commit 快捷配置

# 团队约定的提交类型
alias aicg-feat='aicg --type feat'
alias aicg-fix='aicg --type fix'
alias aicg-docs='aicg --type docs'
alias aicg-test='aicg --type test'

# 项目特定命令
alias deploy-dev='aicp && ssh dev "cd /app && git pull"'
alias deploy-prod='aict-push $(date +v%Y.%m.%d) && ssh prod "cd /app && git pull"'

# 智能命令
smart-commit() {
    local files=$(git diff --name-only)

    # 根据文件类型自动选择提交类型
    if echo "$files" | grep -q "\.md$"; then
        aicg-docs
    elif echo "$files" | grep -q "test"; then
        aicg-test
    elif echo "$files" | grep -q "\.tsx\?$"; then
        aicg-feat
    else
        aicg
    fi
}

# 工作流程宏
morning() {
    echo "☕ 早上好！准备开始工作..."
    git pull
    aich-today
    echo "\n📋 今日待办:"
    # 这里可以集成你的待办系统
}

evening() {
    echo "🌙 准备下班..."
    git status
    read -p "是否提交今日工作? (y/n) " -n 1 -r
    echo
    [[ $REPLY =~ ^[Yy]$ ]] && aicp
    echo "✨ 今日完成:"
    aich-today
}

# 在 ~/.zshrc 或 ~/.bashrc 中引入
source ~/.aic_custom
```

## 故障排查快速解决

| 问题 | 解决方案 |
|------|----------|
| `aic: command not found` | 运行 `make install` 或 `./install.sh` |
| 快捷键不生效 | `source ~/.zshrc` 或 `source ~/.bashrc` |
| AI 生成失败 | 检查配置: `aic-config` |
| 推送被拒绝 | 使用 `aicpf` 强制推送 |
| 工作树冲突 | `aicw-clean` 清理后重建 |

## 团队协作建议

1. **统一快捷键配置**
   ```bash
   # team-shortcuts.sh
   curl -O https://your-team/aic-shortcuts.sh
   source aic-shortcuts.sh
   ```

2. **共享配置模板**
   ```bash
   # 导出配置
   aic config export > team-config.json

   # 导入配置
   aic config import team-config.json
   ```

3. **标准化工作流**
   - 早会: `morning` → 查看昨日进度
   - 开发: `aicff start` → 功能开发
   - 提交: `aicp` → 及时同步
   - 下班: `evening` → 总结提交

---

💡 **黄金法则**: 如果一个命令你每天用超过 3 次，就应该为它创建快捷键！

## 🎨 GRV-Style TUI 功能（新增）

### 查询历史系统

#### 基础查询命令
```bash
# 查看查询历史
aic --query-history          # 显示最近 20 条查询记录

# 查看统计信息
aic --query-stats            # 显示查询统计

# 清空历史
aic --query-clear            # 清空所有查询历史

# 执行查询
aic --query "author:costa"   # 按作者查询
aic --query "message:feat"   # 按消息查询
aic --query "since:2024-01-01" # 按日期查询
```

#### TUI 界面

##### 标准 TUI (`--query-tui`)
- 简洁的单窗口界面
- 查询历史浏览和执行
- 详情面板显示
- 搜索和过滤功能

##### 增强版 TUI (`--query-tui-pro`) 🆕
基于 GRV (Git Repository Viewer) 的专业界面：

**核心特性：**
1. **多窗格布局**
   - 支持水平/垂直分屏
   - `Ctrl+s`: 循环切换分屏模式
   - `Ctrl+w`: 切换焦点窗格

2. **标签页系统**
   - 多标签管理不同视图
   - `Tab`: 下一个标签
   - `Shift+Tab`: 上一个标签
   - `:tab NAME`: 创建新标签

3. **Vim 风格命令模式**
   ```
   :q, :quit    退出
   :vsplit      垂直分屏
   :hsplit      水平分屏
   :tab NAME    新建标签
   :help        显示帮助
   ```

4. **语法高亮**
   - `h`: 切换高亮
   - 自动着色查询类型
   - 结果类型颜色编码

5. **专业 UI 元素**
   - 滚动条和位置指示器
   - 焦点指示（黄色边框）
   - 状态栏显示模式和统计
   - 完整的帮助系统（`?`）

### 快捷键对照表

| 功能 | 标准 TUI | 增强版 TUI |
|------|---------|-----------|
| 移动 | ↑↓/jk | ↑↓/jk |
| 执行 | Enter | Enter/x |
| 搜索 | / | / |
| 帮助 | ? | ? |
| 退出 | q | q/:q |
| 详情 | d | d |
| 分屏 | - | Ctrl+s |
| 切换焦点 | - | Ctrl+w |
| 标签页 | - | Tab/Shift+Tab |
| 命令模式 | - | : |
| 语法高亮 | - | h |

### 使用场景

#### 场景 1: 查找特定作者的提交
```bash
# 命令行快速查询
aic --query "author:张三"

# 或使用 TUI 交互式查找
aic --query-tui-pro
# 然后按 / 搜索 author:张三
```

#### 场景 2: 分析最近的功能开发
```bash
# 查看所有 feat 类型的提交
aic --query "message:feat"

# 在增强版 TUI 中对比多个查询结果
aic --query-tui-pro
# 执行多个查询，每个在新标签中显示
```

#### 场景 3: 团队代码审查
```bash
# 查看今天的所有提交
aic --query "since:$(date +%Y-%m-%d)"

# 使用增强版 TUI 的分屏功能
aic --query-tui-pro
# Ctrl+s 启用分屏，同时查看历史和结果
```

### 配置快捷键

```bash
# 添加到 ~/.zshrc 或 ~/.bashrc
alias aicq='ai-commit --query'              # 快速查询
alias aicqh='ai-commit --query-history'     # 查看历史
alias aicqt='ai-commit --query-tui'         # 标准 TUI
alias aicqp='ai-commit --query-tui-pro'     # 增强版 TUI

# 智能查询函数
query-today() {
    aic --query "since:$(date +%Y-%m-%d)"
}

query-my-commits() {
    aic --query "author:$(git config user.name)"
}
```

### 与 GRV 的对比

| 特性 | GRV | ai-commit TUI |
|------|-----|---------------|
| 多标签 | ✅ | ✅ |
| 分屏视图 | ✅ | ✅ |
| Vim 命令 | ✅ | ✅ |
| 语法高亮 | ✅ | ✅ |
| Git 集成 | ✅ | ✅ |
| AI 查询 | ❌ | ✅ |
| 历史持久化 | ❌ | ✅ |
