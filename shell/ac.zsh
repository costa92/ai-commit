#!/bin/zsh
# AI-Commit ZSH 别名配置和自动补全
# 将此文件 source 到你的 .zshrc 中

# 基本别名
alias ac='ai-commit'

# 常用命令别名
alias acc='ai-commit commit'
alias acg='ai-commit commit generate'
alias acp='ai-commit commit generate --push'
alias act='ai-commit tag'
alias acf='ai-commit flow'
alias acw='ai-commit worktree'
alias ach='ai-commit history'

# Git Flow 快捷方式
alias acff='ai-commit flow feature'
alias acfh='ai-commit flow hotfix'
alias acfr='ai-commit flow release'

# 带参数的函数别名
ac-commit() {
    ai-commit commit generate "$@"
}

ac-tag() {
    ai-commit tag create "$@"
}

ac-feature() {
    ai-commit flow feature start "$@"
}

ac-worktree() {
    ai-commit worktree create "$@"
}

# 快速提交并推送
acp() {
    ai-commit commit generate --add --push "$@"
}

# 快速创建标签并推送
act-push() {
    ai-commit tag create "$1" --push
}

# ZSH 自动补全
if [[ -n "$ZSH_VERSION" ]]; then
    # ac 命令的自动补全
    _ac_completion() {
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
    
    compdef _ac_completion ac
fi

# 显示帮助信息
ac-help() {
    echo "AI-Commit 快捷命令："
    echo "  ac         - ai-commit 简称"
    echo "  acc        - ai-commit commit"
    echo "  acg        - 生成提交消息"
    echo "  acp        - 生成并推送"
    echo "  act        - 标签管理"
    echo "  acf        - Git Flow"
    echo "  acw        - 工作树管理"
    echo "  ach        - 历史查看"
    echo ""
    echo "函数别名："
    echo "  ac-commit  - 生成提交消息（可带参数）"
    echo "  ac-tag     - 创建标签（可带参数）"
    echo "  ac-feature - 创建功能分支"
    echo "  acp        - 快速提交并推送"
    echo "  act-push   - 创建并推送标签"
    echo ""
    echo "提示：ZSH 用户可享受自动补全功能"
}