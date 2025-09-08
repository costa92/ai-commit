# AI-Commit 快速参考卡

## 一页纸速查表（打印版）

```
┌─────────────────────────────────────────────────────────────────┐
│                    AI-COMMIT (AC) 快速参考                      │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  【基础命令】                                                   │
│  ac                        # 等同于 ai-commit                  │
│  ac -h                     # 显示帮助                          │
│  ac --version              # 显示版本                          │
│                                                                 │
│  【提交操作】                                                   │
│  acg                       # 生成提交消息                      │
│  acg -y                    # 生成并确认提交                    │
│  acg -a                    # 添加所有文件并生成                │
│  acp                       # 生成、提交并推送                  │
│  acpf                      # 强制推送（解决冲突）              │
│                                                                 │
│  【标签管理】                                                   │
│  act v1.0.0                # 创建标签                          │
│  act-push v1.0.0           # 创建并推送标签                    │
│  actl                      # 列出所有标签                      │
│  actd v1.0.0               # 删除标签                          │
│                                                                 │
│  【Git Flow】                                                   │
│  acf init                  # 初始化 Git Flow                   │
│  acff start NAME           # 开始功能分支                      │
│  acff finish NAME          # 完成功能分支                      │
│  acfh start NAME           # 开始修复分支                      │
│  acfr start VERSION        # 开始发布分支                      │
│                                                                 │
│  【工作树】                                                     │
│  acw BRANCH                # 创建工作树                        │
│  acwl                      # 列出工作树                        │
│  acw-sw NAME               # 切换工作树                        │
│  acw-rm NAME               # 删除工作树                        │
│                                                                 │
│  【历史查看】                                                   │
│  ach                       # 查看提交历史                      │
│  ach-graph                 # 图形化历史                        │
│  ach-author "name"         # 按作者筛选                        │
│  ach-today                 # 今日提交                          │
│  ach-stats                 # 统计信息                          │
│                                                                 │
│  【配置管理】                                                   │
│  ac-provider NAME          # 设置 AI 提供商                    │
│  ac-model NAME             # 设置 AI 模型                      │
│  ac-config                 # 查看所有配置                      │
│                                                                 │
│  【修改操作】                                                   │
│  ac-amend                  # 修改最后提交                      │
│  ac-undo                   # 撤销最后提交                      │
│  ac-rebase HEAD~N          # 交互式变基                        │
│                                                                 │
├─────────────────────────────────────────────────────────────────┤
│  提示: 运行 ac-help 查看所有命令 | 文档: docs/SHORTCUTS.md    │
└─────────────────────────────────────────────────────────────────┘
```

## 场景化快速指南

### 🌅 每日开始工作
```bash
# 1. 查看昨天的工作
ach-yesterday

# 2. 拉取最新代码
git pull

# 3. 创建今日功能分支
acff start today-work

# 4. 切换到工作树（可选）
acw feature/today-work
```

### 🚀 快速提交更改
```bash
# 场景 1: 简单提交
acg                    # 查看生成的消息
acg -y                 # 确认并提交

# 场景 2: 提交并推送
acp                    # 一键完成

# 场景 3: 修复刚才的提交
ac-amend "更正的消息"
```

### 🐛 紧急修复 Bug
```bash
# 1. 创建修复分支
acfh start critical-fix

# 2. 修改代码...

# 3. 提交修复
acp

# 4. 完成修复并打标签
acfh finish critical-fix
act-push v1.0.1-hotfix
```

### 📦 发布新版本
```bash
# 1. 创建发布分支
acfr start v2.0.0

# 2. 更新版本号、文档等...

# 3. 提交更改
acg -a -y

# 4. 完成发布
acfr finish v2.0.0
act-push v2.0.0
```

### 📊 生成报告
```bash
# 日报
daily-report() {
    echo "=== $(date +%Y-%m-%d) 日报 ==="
    ach-today
    echo "\n未提交更改:"
    git status -s
}

# 周报
weekly-report() {
    echo "=== 本周工作总结 ==="
    ach-week
    ach-stats
}

# 月报
monthly-report() {
    echo "=== 本月绩效 ==="
    ach-month
    ach-contributors
}
```

## 键盘映射建议

### macOS 用户
```bash
# 在 ~/.zshrc 中添加
bindkey -s '^G' 'acg\n'        # Ctrl+G: 生成提交
bindkey -s '^P' 'acp\n'        # Ctrl+P: 推送
bindkey -s '^T' 'act '         # Ctrl+T: 标签
```

### Linux 用户
```bash
# 在 ~/.bashrc 中添加
bind '"\C-g": "acg\n"'         # Ctrl+G: 生成提交
bind '"\C-p": "acp\n"'         # Ctrl+P: 推送
bind '"\C-t": "act "'          # Ctrl+T: 标签
```

### Windows (Git Bash) 用户
```bash
# 在 ~/.bashrc 中添加
alias acg='winpty ai-commit commit generate'
alias acp='acg --push'
```

## 效率对比

| 操作 | 传统方式 | AC 快捷方式 | 节省击键 |
|------|----------|-------------|----------|
| 生成提交 | `ai-commit commit generate` (27键) | `acg` (3键) | 89% |
| 提交并推送 | `git add . && git commit -m "..." && git push` (40+键) | `acp` (3键) | 93% |
| 创建标签 | `git tag v1.0.0 && git push origin v1.0.0` (38键) | `act-push v1.0.0` (15键) | 61% |
| 功能分支 | `git checkout -b feature/x && git push -u origin feature/x` (50+键) | `acff start x` (12键) | 76% |

## 个性化配置模板

```bash
# === ~/.ac_custom ===
# 个人 AI-Commit 快捷配置

# 团队约定的提交类型
alias acg-feat='acg --type feat'
alias acg-fix='acg --type fix'
alias acg-docs='acg --type docs'
alias acg-test='acg --type test'

# 项目特定命令
alias deploy-dev='acp && ssh dev "cd /app && git pull"'
alias deploy-prod='act-push $(date +v%Y.%m.%d) && ssh prod "cd /app && git pull"'

# 智能命令
smart-commit() {
    local files=$(git diff --name-only)
    
    # 根据文件类型自动选择提交类型
    if echo "$files" | grep -q "\.md$"; then
        acg-docs
    elif echo "$files" | grep -q "test"; then
        acg-test
    elif echo "$files" | grep -q "\.tsx\?$"; then
        acg-feat
    else
        acg
    fi
}

# 工作流程宏
morning() {
    echo "☕ 早上好！准备开始工作..."
    git pull
    ach-today
    echo "\n📋 今日待办:"
    # 这里可以集成你的待办系统
}

evening() {
    echo "🌙 准备下班..."
    git status
    read -p "是否提交今日工作? (y/n) " -n 1 -r
    echo
    [[ $REPLY =~ ^[Yy]$ ]] && acp
    echo "✨ 今日完成:"
    ach-today
}

# 在 ~/.zshrc 或 ~/.bashrc 中引入
source ~/.ac_custom
```

## 故障排查快速解决

| 问题 | 解决方案 |
|------|----------|
| `ac: command not found` | 运行 `make install` 或 `./install.sh` |
| 快捷键不生效 | `source ~/.zshrc` 或 `source ~/.bashrc` |
| AI 生成失败 | 检查配置: `ac-config` |
| 推送被拒绝 | 使用 `acpf` 强制推送 |
| 工作树冲突 | `acw-clean` 清理后重建 |

## 团队协作建议

1. **统一快捷键配置**
   ```bash
   # team-shortcuts.sh
   curl -O https://your-team/ac-shortcuts.sh
   source ac-shortcuts.sh
   ```

2. **共享配置模板**
   ```bash
   # 导出配置
   ac config export > team-config.json
   
   # 导入配置
   ac config import team-config.json
   ```

3. **标准化工作流**
   - 早会: `morning` → 查看昨日进度
   - 开发: `acff start` → 功能开发
   - 提交: `acp` → 及时同步
   - 下班: `evening` → 总结提交

---

💡 **黄金法则**: 如果一个命令你每天用超过 3 次，就应该为它创建快捷键！