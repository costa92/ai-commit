# AI-Commit 简称 (aic) 设置指南

## 概述

`ai-commit` 支持使用 `aic` 作为简称，让命令输入更加便捷。

## 安装方式

### 方式 1: 使用 Cargo 安装（推荐）

```bash
# 构建并安装两个二进制文件
cargo build --release
cargo install --path .

# 或使用 Makefile
make install
```

安装后即可使用：
- `ai-commit` - 完整命令
- `aic` - 简称命令

### 方式 2: 使用安装脚本

```bash
# 运行安装脚本
./install.sh

# 或通过 Make
make install-with-script
```

脚本会自动：
1. 构建项目
2. 安装二进制文件
3. 创建 `aic` 链接
4. 可选配置 Shell 别名

### 方式 3: 手动创建符号链接

```bash
# 如果已安装 ai-commit
ln -s ~/.cargo/bin/ai-commit ~/.cargo/bin/aic

# 或使用 Make
make install-alias
```

### 方式 4: Shell 别名配置

#### Bash 用户

```bash
# 添加到 ~/.bashrc
echo "alias aic='ai-commit'" >> ~/.bashrc
source ~/.bashrc

# 或 source 提供的配置文件
source shell/ac.sh
```

#### Zsh 用户（支持自动补全）

```bash
# 添加到 ~/.zshrc
echo "source $(pwd)/shell/ac.zsh" >> ~/.zshrc
source ~/.zshrc
```

## 使用示例

### 基本命令

```bash
# 生成提交消息
aic commit generate
# 或更短的别名
aicg

# 生成并推送
aic commit generate --push
# 或
aicp

# 创建标签
aic tag create v1.0.0
# 或
aict v1.0.0

# Git Flow
aic flow feature start new-feature
# 或
aicff start new-feature

# 工作树
aic worktree create feature/test
# 或
aicw feature/test
```

### 快捷别名列表

| 别名 | 完整命令 | 说明 |
|------|---------|------|
| `aic` | `ai-commit` | 基本简称 |
| `aicc` | `ai-commit commit` | 提交操作 |
| `aicg` | `ai-commit commit generate` | 生成提交消息 |
| `aicp` | `ai-commit commit generate --push` | 生成并推送 |
| `aict` | `ai-commit tag` | 标签管理 |
| `aicf` | `ai-commit flow` | Git Flow |
| `aicw` | `ai-commit worktree` | 工作树管理 |
| `aich` | `ai-commit history` | 历史查看 |
| `aicff` | `ai-commit flow feature` | 功能分支 |
| `aicfh` | `ai-commit flow hotfix` | 修复分支 |
| `aicfr` | `ai-commit flow release` | 发布分支 |

### 函数别名（带参数）

```bash
# 快速提交并推送
aicp "fix: 修复登录问题"

# 创建并推送标签
aict-push v1.2.3

# 创建功能分支
aic-feature user-authentication

# 创建工作树
aic-worktree feature/new-ui
```

## 自动补全（ZSH）

ZSH 用户可以享受自动补全功能：

```bash
aic <TAB>          # 显示所有子命令
aic commit <TAB>   # 显示 commit 子命令
aic tag <TAB>      # 显示 tag 子命令
```

## 查看帮助

```bash
# 显示所有快捷命令
aic-help

# 显示 ai-commit 帮助
aic --help
```

## 卸载

```bash
# 删除二进制文件
rm ~/.cargo/bin/aic
rm ~/.cargo/bin/ai-commit

# 或删除别名（如果使用别名方式）
# 从 ~/.bashrc 或 ~/.zshrc 中删除相关行
```

## 故障排查

### aic 命令未找到

1. 确保 `~/.cargo/bin` 在 PATH 中：
```bash
echo $PATH | grep cargo
# 如果没有，添加到 PATH
export PATH="$HOME/.cargo/bin:$PATH"
```

2. 重新加载 Shell 配置：
```bash
source ~/.bashrc  # 或 ~/.zshrc
```

### 权限问题

```bash
chmod +x ~/.cargo/bin/aic
chmod +x ~/.cargo/bin/ai-commit
```

## 推荐配置

为了最佳体验，建议：

1. **ZSH 用户**：使用 `shell/ac.zsh` 获得自动补全
2. **Bash 用户**：使用 `shell/ac.sh` 获得快捷别名
3. **所有用户**：运行 `./install.sh` 进行完整安装
