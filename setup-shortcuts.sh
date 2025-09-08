#!/bin/bash

# AI-Commit 交互式快捷键配置脚本
# 帮助用户快速配置个性化快捷键

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m' # No Color

# 打印函数
print_header() {
    echo -e "\n${BOLD}${BLUE}═══════════════════════════════════════════════════════${NC}"
    echo -e "${BOLD}${CYAN}        AI-Commit 快捷键配置向导        ${NC}"
    echo -e "${BOLD}${BLUE}═══════════════════════════════════════════════════════${NC}\n"
}

info() {
    echo -e "${GREEN}✓${NC} $1"
}

warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

error() {
    echo -e "${RED}✗${NC} $1"
    exit 1
}

question() {
    echo -e "${PURPLE}?${NC} $1"
}

# 检测 Shell 类型
detect_shell() {
    if [ -n "$ZSH_VERSION" ]; then
        SHELL_TYPE="zsh"
        SHELL_RC="$HOME/.zshrc"
    elif [ -n "$BASH_VERSION" ]; then
        SHELL_TYPE="bash"
        SHELL_RC="$HOME/.bashrc"
    else
        SHELL_TYPE="unknown"
        SHELL_RC=""
    fi
}

# 检查 ai-commit 是否已安装
check_installation() {
    if command -v ai-commit &> /dev/null; then
        info "检测到 ai-commit 已安装"
        AI_COMMIT_PATH=$(which ai-commit)
    else
        warning "未检测到 ai-commit"
        echo -e "${YELLOW}请先安装 ai-commit:${NC}"
        echo "  cargo install --path ."
        echo "  或"
        echo "  make install"
        exit 1
    fi
    
    if command -v ac &> /dev/null; then
        info "检测到 ac 简称已可用"
    else
        warning "ac 简称未安装，将在配置中创建"
    fi
}

# 选择配置级别
select_config_level() {
    echo -e "\n${BOLD}选择配置级别:${NC}"
    echo "1) 最小配置 - 仅基本命令 (推荐新手)"
    echo "2) 标准配置 - 常用快捷键"
    echo "3) 完整配置 - 所有快捷键"
    echo "4) 自定义配置 - 选择需要的功能"
    
    read -p "请选择 [1-4]: " -n 1 level
    echo
    
    case $level in
        1) CONFIG_LEVEL="minimal" ;;
        2) CONFIG_LEVEL="standard" ;;
        3) CONFIG_LEVEL="full" ;;
        4) CONFIG_LEVEL="custom" ;;
        *) CONFIG_LEVEL="standard" ;;
    esac
}

# 生成最小配置
generate_minimal_config() {
    cat << 'EOF'
# AI-Commit 最小配置
alias ac='ai-commit'
alias acg='ai-commit commit generate'
alias acp='ai-commit commit generate --push'
alias act='ai-commit tag create'
alias ac-help='echo "acg: 生成提交 | acp: 提交并推送 | act: 创建标签"'
EOF
}

# 生成标准配置
generate_standard_config() {
    cat << 'EOF'
# AI-Commit 标准配置

# 基本命令
alias ac='ai-commit'

# 提交操作
alias acg='ai-commit commit generate'
alias acp='ai-commit commit generate --push'
alias acpf='ai-commit commit generate --force-push --push'
alias ac-amend='ai-commit commit amend'
alias ac-undo='ai-commit commit undo'

# 标签管理
alias act='ai-commit tag create'
alias actl='ai-commit tag list'
alias act-push='ai-commit tag create $1 --push'

# Git Flow
alias acf='ai-commit flow'
alias acff='ai-commit flow feature'
alias acfh='ai-commit flow hotfix'
alias acfr='ai-commit flow release'

# 工作树
alias acw='ai-commit worktree create'
alias acwl='ai-commit worktree list'

# 历史
alias ach='ai-commit history log'
alias ach-today='ai-commit history log --since today'

# 帮助
alias ac-help='ac --help'
EOF
}

# 生成完整配置
generate_full_config() {
    # 根据 Shell 类型生成
    if [ "$SHELL_TYPE" = "zsh" ]; then
        cat "$SCRIPT_DIR/shell/ac.zsh"
    else
        cat "$SCRIPT_DIR/shell/ac.sh"
    fi
}

# 自定义配置选择
custom_config_selection() {
    echo -e "\n${BOLD}选择要启用的功能:${NC}"
    
    # 功能选项
    declare -a features
    declare -a selected
    
    features=(
        "基本命令 (ac, acg, acp)"
        "标签管理 (act, actl, act-push)"
        "Git Flow (acf, acff, acfh, acfr)"
        "工作树管理 (acw, acwl)"
        "历史查看 (ach, ach-today, ach-stats)"
        "提交修改 (ac-amend, ac-undo)"
        "配置管理 (ac-config, ac-provider)"
        "智能函数 (smart-commit, daily-report)"
        "键盘绑定 (Ctrl+G, Ctrl+P)"
    )
    
    echo "选择要启用的功能 (空格选择，回车确认):"
    
    # 交互式选择
    for i in "${!features[@]}"; do
        selected[$i]=false
        echo "[ ] $((i+1)). ${features[$i]}"
    done
    
    echo -e "\n输入功能编号（用空格分隔，如: 1 2 3）或输入 'all' 选择全部:"
    read -r selections
    
    if [ "$selections" = "all" ]; then
        for i in "${!features[@]}"; do
            selected[$i]=true
        done
    else
        for num in $selections; do
            idx=$((num-1))
            if [ $idx -ge 0 ] && [ $idx -lt ${#features[@]} ]; then
                selected[$idx]=true
            fi
        done
    fi
    
    # 生成自定义配置
    generate_custom_config "${selected[@]}"
}

# 生成自定义配置内容
generate_custom_config() {
    local selected=("$@")
    
    echo "# AI-Commit 自定义配置"
    echo "# 生成时间: $(date)"
    echo ""
    
    # 基本命令
    if [ "${selected[0]}" = true ]; then
        echo "# 基本命令"
        echo "alias ac='ai-commit'"
        echo "alias acg='ai-commit commit generate'"
        echo "alias acp='ai-commit commit generate --push'"
        echo ""
    fi
    
    # 标签管理
    if [ "${selected[1]}" = true ]; then
        echo "# 标签管理"
        echo "alias act='ai-commit tag create'"
        echo "alias actl='ai-commit tag list'"
        echo "act-push() { ai-commit tag create \"\$1\" --push; }"
        echo ""
    fi
    
    # Git Flow
    if [ "${selected[2]}" = true ]; then
        echo "# Git Flow"
        echo "alias acf='ai-commit flow'"
        echo "alias acff='ai-commit flow feature'"
        echo "alias acfh='ai-commit flow hotfix'"
        echo "alias acfr='ai-commit flow release'"
        echo ""
    fi
    
    # 继续添加其他功能...
}

# 备份现有配置
backup_config() {
    if [ -f "$SHELL_RC" ]; then
        local backup_file="${SHELL_RC}.ac-backup.$(date +%Y%m%d_%H%M%S)"
        cp "$SHELL_RC" "$backup_file"
        info "已备份现有配置到: $backup_file"
    fi
}

# 写入配置
write_config() {
    local config_content="$1"
    local config_file="$HOME/.ac_shortcuts"
    
    # 写入独立配置文件
    echo "$config_content" > "$config_file"
    info "配置已写入: $config_file"
    
    # 检查是否已在 RC 文件中引用
    if ! grep -q "source.*\.ac_shortcuts" "$SHELL_RC" 2>/dev/null; then
        echo "" >> "$SHELL_RC"
        echo "# AI-Commit 快捷键配置" >> "$SHELL_RC"
        echo "[ -f ~/.ac_shortcuts ] && source ~/.ac_shortcuts" >> "$SHELL_RC"
        info "已添加配置引用到: $SHELL_RC"
    else
        info "配置引用已存在"
    fi
}

# 测试配置
test_config() {
    echo -e "\n${BOLD}测试配置...${NC}"
    
    # 重新加载配置
    if [ "$SHELL_TYPE" = "zsh" ]; then
        zsh -c "source $HOME/.ac_shortcuts && type ac"
    else
        bash -c "source $HOME/.ac_shortcuts && type ac"
    fi
    
    if [ $? -eq 0 ]; then
        info "配置测试成功！"
    else
        warning "配置可能需要重新加载 Shell"
    fi
}

# 显示使用说明
show_usage() {
    echo -e "\n${BOLD}${GREEN}✨ 配置完成！${NC}\n"
    
    echo -e "${BOLD}立即生效:${NC}"
    echo "  source $SHELL_RC"
    echo ""
    
    echo -e "${BOLD}常用命令:${NC}"
    echo "  acg    - 生成提交消息"
    echo "  acp    - 提交并推送"
    echo "  act    - 创建标签"
    echo "  ac-help - 查看所有快捷键"
    echo ""
    
    echo -e "${BOLD}配置文件:${NC}"
    echo "  ~/.ac_shortcuts - 快捷键配置"
    echo "  $SHELL_RC - Shell 配置文件"
    echo ""
    
    echo -e "${BOLD}更多信息:${NC}"
    echo "  查看文档: cat docs/SHORTCUTS.md"
    echo "  快速参考: cat docs/QUICK_REFERENCE.md"
}

# 主函数
main() {
    SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
    
    print_header
    
    # 检测环境
    detect_shell
    info "检测到 Shell: $SHELL_TYPE"
    
    # 检查安装
    check_installation
    
    # 选择配置级别
    select_config_level
    
    # 生成配置
    case $CONFIG_LEVEL in
        minimal)
            CONFIG_CONTENT=$(generate_minimal_config)
            ;;
        standard)
            CONFIG_CONTENT=$(generate_standard_config)
            ;;
        full)
            CONFIG_CONTENT=$(generate_full_config)
            ;;
        custom)
            CONFIG_CONTENT=$(custom_config_selection)
            ;;
    esac
    
    # 备份和写入
    echo -e "\n${BOLD}准备写入配置...${NC}"
    read -p "是否备份现有配置？(y/n) " -n 1 -r
    echo
    
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        backup_config
    fi
    
    write_config "$CONFIG_CONTENT"
    
    # 测试配置
    test_config
    
    # 显示使用说明
    show_usage
    
    # 询问是否立即加载
    read -p "是否立即加载配置？(y/n) " -n 1 -r
    echo
    
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        echo -e "\n${YELLOW}请运行以下命令加载配置:${NC}"
        echo -e "${BOLD}source $SHELL_RC${NC}"
    fi
}

# 运行主函数
main "$@"