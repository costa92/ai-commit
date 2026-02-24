#!/bin/bash
# AI-Commit Shell 别名配置
# 将此文件 source 到你的 .bashrc 或 .zshrc 中

# 基本别名
alias aic='ai-commit'

# 常用命令别名
alias aicc='ai-commit commit'
alias aicg='ai-commit commit generate'
alias aicp='ai-commit commit generate --push'
alias aict='ai-commit tag'
alias aicf='ai-commit flow'
alias aicw='ai-commit worktree'
alias aich='ai-commit history'

# Git Flow 快捷方式
alias aicff='ai-commit flow feature'
alias aicfh='ai-commit flow hotfix'
alias aicfr='ai-commit flow release'

# 带参数的函数别名
aic-commit() {
    ai-commit commit generate "$@"
}

aic-tag() {
    ai-commit tag create "$@"
}

aic-feature() {
    ai-commit flow feature start "$@"
}

aic-worktree() {
    ai-commit worktree create "$@"
}

# 快速提交并推送
aicp() {
    ai-commit commit generate --add --push "$@"
}

# 快速创建标签并推送
aict-push() {
    ai-commit tag create "$1" --push
}

# 显示帮助信息
aic-help() {
    echo "AI-Commit 快捷命令："
    echo "  aic        - ai-commit 简称"
    echo "  aicc       - ai-commit commit"
    echo "  aicg       - 生成提交消息"
    echo "  aicp       - 生成并推送"
    echo "  aict       - 标签管理"
    echo "  aicf       - Git Flow"
    echo "  aicw       - 工作树管理"
    echo "  aich       - 历史查看"
    echo ""
    echo "函数别名："
    echo "  aic-commit  - 生成提交消息（可带参数）"
    echo "  aic-tag     - 创建标签（可带参数）"
    echo "  aic-feature - 创建功能分支"
    echo "  aicp        - 快速提交并推送"
    echo "  aict-push   - 创建并推送标签"
}
