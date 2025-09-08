# AI-Commit 快捷键和命令速查表

## 🚀 快速开始

| 操作 | 快捷命令 | 完整命令 | 说明 |
|------|----------|----------|------|
| **快速提交** | `acg` | `ai-commit commit generate` | 生成 AI 提交消息 |
| **提交并推送** | `acp` | `ai-commit commit generate --push` | 生成提交并推送 |
| **创建标签** | `act v1.0.0` | `ai-commit tag create v1.0.0` | 创建版本标签 |
| **查看帮助** | `ac -h` | `ai-commit --help` | 显示帮助信息 |

## 📝 提交操作快捷键

### 基础提交
```bash
# 生成提交消息（仅显示）
acg                     # ai-commit commit generate

# 生成并执行提交
acg -y                  # ai-commit commit generate --yes

# 添加所有文件并提交
acg -a                  # ai-commit commit generate --add

# 提交并推送
acp                     # ai-commit commit generate --push

# 强制推送（解决冲突）
acpf                    # ai-commit commit generate --force-push --push
```

### 高级提交
```bash
# 修改最后一次提交
ac-amend               # ai-commit commit amend
ac-amend "新消息"      # ai-commit commit amend "新消息"

# 撤销最后一次提交
ac-undo                # ai-commit commit undo

# 交互式 rebase
ac-rebase HEAD~3       # ai-commit commit rebase HEAD~3
```

## 🏷️ 标签管理快捷键

```bash
# 创建标签
act v1.0.0             # ai-commit tag create v1.0.0
act-note v1.0.0 "说明" # ai-commit tag create v1.0.0 --note "说明"

# 推送标签
act-push v1.0.0        # ai-commit tag create v1.0.0 --push

# 列出标签
actl                   # ai-commit tag list

# 删除标签
actd v1.0.0           # ai-commit tag delete v1.0.0

# 比较标签
actc v1.0.0..v2.0.0   # ai-commit tag compare v1.0.0..v2.0.0
```

## 🌿 Git Flow 快捷键

### Feature 功能分支
```bash
# 开始功能
acff start login      # ai-commit flow feature start login
acff-s login          # 简写版本

# 完成功能
acff finish login     # ai-commit flow feature finish login
acff-f login          # 简写版本

# 列出功能分支
acff list             # ai-commit flow feature list
acff-l                # 简写版本
```

### Hotfix 修复分支
```bash
# 开始修复
acfh start critical   # ai-commit flow hotfix start critical
acfh-s critical       # 简写版本

# 完成修复
acfh finish critical  # ai-commit flow hotfix finish critical
acfh-f critical       # 简写版本
```

### Release 发布分支
```bash
# 开始发布
acfr start v1.0.0     # ai-commit flow release start v1.0.0
acfr-s v1.0.0         # 简写版本

# 完成发布
acfr finish v1.0.0    # ai-commit flow release finish v1.0.0
acfr-f v1.0.0         # 简写版本
```

## 🌲 工作树快捷键

```bash
# 创建工作树
acw feature/test      # ai-commit worktree create feature/test
acw-new feature/test  # 带新分支创建

# 切换工作树
acw-sw test          # ai-commit worktree switch test

# 列出工作树
acwl                 # ai-commit worktree list
acwl-v               # 详细列表

# 删除工作树
acw-rm test          # ai-commit worktree remove test

# 清理所有工作树
acw-clean            # ai-commit worktree clear
```

## 📊 历史查看快捷键

```bash
# 查看历史
ach                  # ai-commit history log
ach-graph            # 带图形显示

# 按作者筛选
ach-author "张三"    # ai-commit history log --author "张三"

# 按时间筛选
ach-today            # 今天的提交
ach-week             # 本周的提交
ach-month            # 本月的提交

# 查看统计
ach-stats            # ai-commit history stats
ach-contributors     # 贡献者统计

# 搜索提交
ach-search "关键词"  # ai-commit history search "关键词"
```

## ⚙️ 配置快捷键

```bash
# 设置 AI 提供商
ac-provider ollama   # ai-commit config set provider ollama
ac-provider deepseek # ai-commit config set provider deepseek
ac-provider silicon  # ai-commit config set provider siliconflow

# 设置模型
ac-model mistral     # ai-commit config set model mistral
ac-model gpt-4       # ai-commit config set model gpt-4

# 查看配置
ac-config            # ai-commit config list
ac-config-get key    # ai-commit config get key
```

## 🎯 组合快捷键（高效工作流）

### 场景 1: 快速修复并发布
```bash
# 1. 创建修复分支
acfh-s fix-login

# 2. 修改代码后提交
acp

# 3. 完成修复并标记版本
acfh-f fix-login
act-push v1.0.1
```

### 场景 2: 功能开发流程
```bash
# 1. 创建功能分支和工作树
acw feature/new-api
cd ../worktree-feature-new-api

# 2. 开发并提交
acg -a              # 添加所有文件并生成提交
acp                 # 推送到远程

# 3. 完成功能
acff-f new-api
```

### 场景 3: 批量操作
```bash
# 批量提交多个更改
ac-batch-commit() {
    acg -a -y
    acp
    act-push $(date +v%Y.%m.%d)
}
```

## 🔧 自定义快捷键

### 创建你自己的快捷键

编辑 `~/.bashrc` 或 `~/.zshrc`:

```bash
# 超短命令
alias a='ac'
alias ag='acg'
alias ap='acp'

# 项目特定快捷键
alias deploy='acp && act-push $(date +v%Y.%m.%d) && echo "已部署"'
alias hotfix='acfh-s hotfix-$(date +%s)'
alias release='acfr-s v$(date +%Y.%m.%d)'

# 智能提交（根据文件类型）
smart-commit() {
    if [[ -n $(git status --porcelain | grep ".md") ]]; then
        ac commit generate -y -m "docs: 更新文档"
    elif [[ -n $(git status --porcelain | grep "test") ]]; then
        ac commit generate -y -m "test: 更新测试"
    else
        acg -y
    fi
}
```

## 📱 终端快捷键绑定

### iTerm2 / Terminal.app (macOS)

1. 打开偏好设置 → Keys → Key Bindings
2. 添加快捷键:
   - `⌘+G` → 发送文本: `acg\n`
   - `⌘+P` → 发送文本: `acp\n`
   - `⌘+T` → 发送文本: `act `

### VS Code 集成终端

在 `settings.json` 中添加:

```json
{
    "terminal.integrated.commandsToSkipShell": ["ac"],
    "terminal.integrated.macros": {
        "quickCommit": ["acg"],
        "quickPush": ["acp"],
        "quickTag": ["act"]
    }
}
```

### Tmux 快捷键

在 `~/.tmux.conf` 中添加:

```bash
# AI Commit 快捷键
bind-key g send-keys "acg" Enter
bind-key p send-keys "acp" Enter
bind-key t send-keys "act "
```

## 🎨 主题化提示符

### 在提示符中显示 AI Commit 状态

```bash
# Bash/Zsh 提示符
ac_prompt() {
    local branch=$(git branch --show-current 2>/dev/null)
    if [[ -n "$branch" ]]; then
        echo " ($branch)"
    fi
}

PS1='[\u@\h \W$(ac_prompt)]\$ '
```

### 使用 Oh My Zsh 插件

创建 `~/.oh-my-zsh/custom/plugins/ac/ac.plugin.zsh`:

```bash
# AI Commit Oh My Zsh 插件
plugins=(... ac)

# 自动加载快捷键
source /path/to/ai-commit/shell/ac.zsh
```

## 💡 专业技巧

### 1. 使用别名链
```bash
# 创建递进式别名
alias g='git'
alias ga='git add'
alias gc='ac commit generate'
alias gp='git push'
alias gcp='gc && gp'  # 组合操作
```

### 2. 条件快捷键
```bash
# 根据分支类型自动选择操作
smart-push() {
    local branch=$(git branch --show-current)
    case $branch in
        main|master)
            echo "在主分支，需要确认"
            read -p "确定要推送到主分支吗？(y/n) " -n 1 -r
            [[ $REPLY =~ ^[Yy]$ ]] && acp
            ;;
        feature/*)
            acp  # 功能分支直接推送
            ;;
        hotfix/*)
            acp && act-push "hotfix-$(date +%s)"
            ;;
        *)
            acg  # 其他分支只生成不推送
            ;;
    esac
}
```

### 3. 批量操作宏
```bash
# 每日工作流
daily-standup() {
    echo "📊 今日提交统计："
    ach-today
    echo "\n🎯 待处理任务："
    git status --short
    echo "\n💡 建议操作："
    [[ -n $(git status --porcelain) ]] && echo "  运行 'acg' 提交更改"
}

# 周报生成
weekly-report() {
    echo "📈 本周工作报告"
    echo "=================="
    ach-week
    echo "\n📊 统计信息："
    ach-stats
    echo "\n👥 贡献者："
    ach-contributors
}
```

## 📚 速查卡片

### 最常用的 10 个命令

| 排名 | 命令 | 用途 | 频率 |
|------|------|------|------|
| 1 | `acg` | 生成提交消息 | ⭐⭐⭐⭐⭐ |
| 2 | `acp` | 提交并推送 | ⭐⭐⭐⭐⭐ |
| 3 | `act` | 创建标签 | ⭐⭐⭐⭐ |
| 4 | `acff` | 功能分支 | ⭐⭐⭐⭐ |
| 5 | `ach` | 查看历史 | ⭐⭐⭐ |
| 6 | `acw` | 工作树管理 | ⭐⭐⭐ |
| 7 | `ac-amend` | 修改提交 | ⭐⭐⭐ |
| 8 | `acfh` | 修复分支 | ⭐⭐ |
| 9 | `ac-undo` | 撤销提交 | ⭐⭐ |
| 10 | `ac-config` | 查看配置 | ⭐ |

## 🎯 下一步

1. **安装快捷键**: 运行 `source shell/ac.zsh` (ZSH) 或 `source shell/ac.sh` (Bash)
2. **自定义配置**: 根据你的工作流添加个人快捷键
3. **练习使用**: 从最常用的 `acg` 和 `acp` 开始
4. **分享经验**: 将你的快捷键配置分享给团队

---

💡 **提示**: 运行 `ac-help` 查看所有可用的快捷命令！