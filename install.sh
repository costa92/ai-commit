#!/bin/bash

# AI-Commit 安装脚本
# 支持安装 ai-commit 并创建 ac 简称

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# 打印信息
info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1"
    exit 1
}

warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

# 检查 Rust 是否安装
check_rust() {
    if ! command -v cargo &> /dev/null; then
        error "Rust/Cargo 未安装。请先安装 Rust: https://rustup.rs/"
    fi
    info "检测到 Rust 版本: $(rustc --version)"
}

# 构建项目
build_project() {
    info "开始构建 ai-commit..."
    cargo build --release
    info "构建完成！"
}

# 安装二进制文件
install_binaries() {
    local INSTALL_DIR="$HOME/.cargo/bin"
    
    # 确保安装目录存在
    mkdir -p "$INSTALL_DIR"
    
    # 安装 ai-commit
    if [ -f "target/release/ai-commit" ]; then
        cp target/release/ai-commit "$INSTALL_DIR/"
        chmod +x "$INSTALL_DIR/ai-commit"
        info "已安装 ai-commit 到 $INSTALL_DIR"
    else
        error "未找到 ai-commit 二进制文件"
    fi
    
    # 创建 ac 符号链接
    if [ -f "$INSTALL_DIR/ai-commit" ]; then
        ln -sf "$INSTALL_DIR/ai-commit" "$INSTALL_DIR/ac"
        info "已创建 'ac' 简称链接"
    fi
}

# 配置 Shell 别名（可选）
setup_shell_alias() {
    echo ""
    read -p "是否要在 Shell 配置中添加 'ac' 别名？(y/n) " -n 1 -r
    echo ""
    
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        local SHELL_RC=""
        
        # 检测 Shell 类型
        if [ -n "$ZSH_VERSION" ]; then
            SHELL_RC="$HOME/.zshrc"
        elif [ -n "$BASH_VERSION" ]; then
            SHELL_RC="$HOME/.bashrc"
        else
            warning "未检测到支持的 Shell，请手动添加别名"
            return
        fi
        
        # 检查是否已存在别名
        if ! grep -q "alias ac=" "$SHELL_RC" 2>/dev/null; then
            echo "" >> "$SHELL_RC"
            echo "# AI-Commit 简称" >> "$SHELL_RC"
            echo "alias ac='ai-commit'" >> "$SHELL_RC"
            info "已添加别名到 $SHELL_RC"
            info "请运行 'source $SHELL_RC' 或重新打开终端以生效"
        else
            info "别名已存在，跳过"
        fi
    fi
}

# 验证安装
verify_installation() {
    info "验证安装..."
    
    if command -v ai-commit &> /dev/null; then
        info "✓ ai-commit 已成功安装"
        ai-commit --version
    else
        error "ai-commit 安装失败"
    fi
    
    if command -v ac &> /dev/null; then
        info "✓ ac 简称已可用"
    else
        warning "ac 简称未生效，可能需要重新加载 PATH"
    fi
}

# 主函数
main() {
    echo "========================================="
    echo "   AI-Commit 安装脚本"
    echo "========================================="
    echo ""
    
    info "开始安装..."
    
    check_rust
    build_project
    install_binaries
    setup_shell_alias
    verify_installation
    
    echo ""
    echo "========================================="
    info "安装完成！"
    echo ""
    echo "使用方法："
    echo "  ai-commit [命令]  # 完整命令"
    echo "  ac [命令]         # 简称"
    echo ""
    echo "示例："
    echo "  ac commit generate"
    echo "  ac tag create v1.0.0"
    echo "  ac flow init"
    echo "========================================="
}

# 运行主函数
main "$@"