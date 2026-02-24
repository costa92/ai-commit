#!/bin/zsh
# AI-Commit ZSH 别名配置和自动补全
# 将此文件 source 到你的 .zshrc 中

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

# ZSH 自动补全
if [[ -n "$ZSH_VERSION" ]]; then
    # aic 命令的自动补全
    _aic_completion() {
        local -a subcmds
        subcmds=(
            'commit:AI commit operations'
            'tag:Tag management'
            'flow:Git Flow workflow'
            'worktree:Worktree management'
            'history:View history'
            'config:Configuration'
        )

        if (( CURRENT == 2 )); then
            _describe 'command' subcmds
        elif (( CURRENT == 3 )); then
            case ${words[2]} in
                commit)
                    local -a commit_cmds
                    commit_cmds=(
                        'generate:Generate commit message'
                        'push:Commit and push'
                        'amend:Amend last commit'
                    )
                    _describe 'commit command' commit_cmds
                    ;;
                tag)
                    local -a tag_cmds
                    tag_cmds=(
                        'create:Create new tag'
                        'list:List tags'
                        'delete:Delete tag'
                        'compare:Compare tags'
                    )
                    _describe 'tag command' tag_cmds
                    ;;
                flow)
                    local -a flow_cmds
                    flow_cmds=(
                        'init:Initialize Git Flow'
                        'feature:Feature branch'
                        'hotfix:Hotfix branch'
                        'release:Release branch'
                    )
                    _describe 'flow command' flow_cmds
                    ;;
                worktree)
                    local -a worktree_cmds
                    worktree_cmds=(
                        'create:Create worktree'
                        'list:List worktrees'
                        'switch:Switch worktree'
                        'remove:Remove worktree'
                    )
                    _describe 'worktree command' worktree_cmds
                    ;;
            esac
        fi
    }

    compdef _aic_completion aic
fi

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
    echo ""
    echo "提示：ZSH 用户可享受自动补全功能"
}
