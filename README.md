# ai-commit

---
<!-- 徽章区 -->
<p align="left">
  <img src="https://img.shields.io/github/v/release/costa92/ai-commit?style=flat-square" alt="release"/>
  <img src="https://img.shields.io/github/actions/workflow/status/costa92/ai-commit/release.yml?branch=main&style=flat-square" alt="CI"/>
  <img src="https://img.shields.io/github/license/costa92/ai-commit?style=flat-square" alt="license"/>
  <img src="https://img.shields.io/github/downloads/costa92/ai-commit/total?style=flat-square" alt="downloads"/>
  <img src="https://img.shields.io/github/stars/costa92/ai-commit?style=flat-square" alt="stars"/>
  <img src="https://img.shields.io/github/issues/costa92/ai-commit?style=flat-square" alt="issues"/>
</p>

ai-commit 是一个基于 Rust 的智能 Git 提交工具，集成本地/云端大模型（如 Ollama、Deepseek、SiliconFlow），可自动根据代码变更生成符合 Conventional Commits 规范的中文提交信息，并支持 AI 驱动的 Code Review 需求文档生成，提升团队协作效率和提交规范性。

---

## 主要功能

- 自动生成规范化的 Git commit message（支持中文，主题不超 50 字）
- **🆕 AI 驱动的 Code Review 需求文档生成**，支持 Go、TypeScript、JavaScript、Rust 等语言特定分析
- 支持 Ollama、Deepseek、SiliconFlow 等多种 AI provider
- 可自定义模型、API 地址、API Key
- 自动 git add/commit/push，参数可控
- 支持自定义提交规范模板
- 命令行参数与 .env 配置灵活切换
- 调试模式支持，可控制输出详细程度
- Git worktree 开发模式，支持多分支并行开发

---

## 安装与运行

1. 安装 Rust 环境（推荐 [rustup](https://rustup.rs/)）
2. 克隆本项目并进入目录
3. 构建并运行：
  
   ```bash
   cargo build --release
   cargo run -- [参数]
   ```

---

## 命令行参数

| 简称/全称                    | 说明                                         | 默认值      |
|------------------------------|----------------------------------------------|-------------|
| -P, --provider               | AI 提交生成服务（ollama/deepseek/siliconflow） | ollama      |
| -m, --model                  | AI 模型名称                                  | mistral     |
| -n, --no-add                 | 不自动执行 git add .                         | false       |
| -p, --push                   | commit 后自动 git push                       | false       |
| -t, --new-tag [VER]          | 创建新 tag（可指定版本号，如 -t v1.2.0）     |             |
| -s, --show-tag               | 显示最新的 tag 和备注                        | false       |
| -b, --push-branches          | 推 tag 时同时推 master develop main 分支     | false       |
| --worktree-create BRANCH     | 创建新的 Git worktree                        |             |
| --worktree-switch NAME       | 切换到指定的 worktree                        |             |
| --worktree-list              | 列出所有可用的 worktrees                     | false       |
| --worktree-verbose, -v       | worktree list 详细模式                       | false       |
| --worktree-porcelain         | worktree list 机器可读输出                   | false       |
| --worktree-z, -z             | worktree list 使用NUL字符终止记录            | false       |
| --worktree-expire TIME       | worktree list 显示过期时间注释               |             |
| --worktree-remove NAME       | 删除指定的 worktree                          |             |
| --worktree-path PATH         | 指定 worktree 创建的自定义路径               |             |
| --worktree-clear             | 清空除当前外的所有其他 worktrees             | false       |

> 所有参数均支持简称和全称，可混用。详见 `ai-commit --help`。

---

## 新特性与亮点

- 支持多平台（Linux/musl、macOS Intel/ARM、Windows）一键构建与发布
- CI/CD 自动化（GitHub Actions，自动归档、校验和、上传 Release）
- commit-prompt.txt 支持：可自定义提交规范模板，内置模板与文件内容始终一致
- 命令行参数与 .env 配置灵活切换，所有参数均有简称/全称
- tag 生成支持自动递增和大版本指定，推送策略安全可控
- 文档完善，覆盖安装、使用、CI、扩展、常见问题等

## 新建 tag 的行为说明

- 使用 `--new-tag` 或 `-t` 新建 tag 时：
  - **如果有已暂存（staged）的变更**：
    - 会自动生成一次 commit（commit message 优先用 `--tag-note`，否则用 AI 生成，有 diff 时用 AI，无 diff 时用默认 `manual tag`）。
    - 然后自动创建 tag，tag note 内容与 commit message 相同。
  - **如果没有已暂存变更**：
    - 只会创建 tag，不会生成新的 commit。
    - tag note 优先用 `--tag-note`，否则用默认 `manual tag`。

- `--tag-note` 参数优先级最高。
- 没有 `--tag-note` 且有 diff 时，tag note/commit message 用 AI 生成。
- 没有 `--tag-note` 且无 diff 时，tag note/commit message 用默认字符串 `manual tag`。

- 支持 `--push` 自动推送新 tag。

## 示例

### 基本使用

```sh
# 有变更时自动 commit 并打 tag
$ git add .
$ ai-commit -t -p

# 无变更时只打 tag
$ ai-commit -t -p

# 指定 tag note
$ ai-commit -t -p --tag-note "发布 v1.2.3"
```

### AI 提供商使用示例

```sh
# 使用 SiliconFlow（推荐）
$ AI_COMMIT_PROVIDER=siliconflow AI_COMMIT_SILICONFLOW_API_KEY=your-key ai-commit

# 使用 Deepseek
$ AI_COMMIT_PROVIDER=deepseek AI_COMMIT_DEEPSEEK_API_KEY=your-key ai-commit

# 使用本地 Ollama（默认，需要先启动 Ollama 服务）
$ ai-commit

# 通过命令行参数指定提供商
$ ai-commit --provider siliconflow --model Qwen/Qwen2.5-7B-Instruct
```

### Code Review 需求文档生成

```sh
# 生成 Go 项目的需求文档 (Markdown 格式)
$ ai-commit --generate-requirements --target-language go

# 生成 TypeScript 项目的需求文档 (JSON 格式)
$ ai-commit --gen-req --target-language typescript --output-format json

# 保存需求文档到指定文件
$ ai-commit --gen-req --target-language auto --output-file requirements.md

# 自动检测语言并生成详细的需求文档
$ ai-commit --generate-requirements --target-language auto --output-format markdown

# 支持的目标语言: go, typescript, javascript, rust, auto (自动检测)
# 支持的输出格式: markdown, json
```

**需求文档包含以下内容：**
- 📋 变更摘要和影响分析
- 🔄 文件变更详情和语言特性识别
- 🧪 语言特定的测试计划和策略
- 📊 架构影响评估和风险分析
- 🔒 安全考虑和迁移要求

### Git Worktree 开发模式示例

```sh
# 创建新的 worktree 用于功能开发
$ ai-commit --worktree-create feature/new-ui
# ✓ Worktree created at: ../worktree-feature-new-ui
#   To switch to this worktree, run: cd ../worktree-feature-new-ui

# 创建 worktree 并指定自定义路径
$ ai-commit --worktree-create feature/auth --worktree-path ~/dev/auth-feature
# ✓ Worktree created at: /Users/username/dev/auth-feature

# 列出所有可用的 worktrees
$ ai-commit --worktree-list
# Available worktrees:
#   refs/heads/main -> /Users/username/project [abc12345]
#   refs/heads/feature/new-ui -> /Users/username/worktree-feature-new-ui [def67890]

# 详细模式列出 worktrees (等同于 git worktree list -v)
$ ai-commit --worktree-list --worktree-verbose
# 或简写
$ ai-commit --worktree-list -v

# 机器可读格式输出 (等同于 git worktree list --porcelain)
$ ai-commit --worktree-list --worktree-porcelain

# 使用NUL字符分隔输出 (等同于 git worktree list -z)
$ ai-commit --worktree-list --worktree-z
# 或简写
$ ai-commit --worktree-list -z

# 显示过期时间注释 (等同于 git worktree list --expire 2weeks)
$ ai-commit --worktree-list --worktree-expire 2weeks

# 组合使用多个选项
$ ai-commit --worktree-list --worktree-porcelain --worktree-z --worktree-expire 1month

# 切换到指定的 worktree（注意：这会改变当前工作目录）
$ ai-commit --worktree-switch feature/new-ui
# ✓ Switched to worktree: /Users/username/worktree-feature-new-ui
#   Current branch: refs/heads/feature/new-ui
#   Working directory: /Users/username/worktree-feature-new-ui

# 在 worktree 中正常使用 ai-commit
$ ai-commit --provider deepseek --push

# 删除不需要的 worktree
$ ai-commit --worktree-remove feature/old-feature
# ✓ Removed worktree: feature/old-feature

# 组合使用：创建 worktree 并立即在其中提交
$ ai-commit --worktree-create hotfix/critical-bug && cd ../worktree-hotfix-critical-bug && ai-commit

# 清空除当前外的所有其他 worktrees（批量清理）
$ ai-commit --worktree-clear
# ✓ Cleared 3 other worktree(s)

# 在调试模式下清空其他 worktrees
$ AI_COMMIT_DEBUG=true ai-commit --worktree-clear
# ✓ Cleared 2 other worktree(s)
# Cleared all worktrees except current
```

### 调试模式示例

```sh
# 关闭调试模式（静默运行）
$ AI_COMMIT_DEBUG=false ai-commit

# 开启调试模式（显示详细过程）
$ AI_COMMIT_DEBUG=true ai-commit
# 输出示例：
# AI 生成 commit message 耗时: 1.23s
# Created new tag: v1.0.1

# 通过 .env 文件配置
$ echo "AI_COMMIT_DEBUG=true" >> .env
$ ai-commit
```

## 配置说明

### 环境变量配置

支持通过 `.env` 文件或环境变量配置：

| 环境变量 | 说明 | 默认值 |
|---------|------|--------|
| `AI_COMMIT_PROVIDER` | AI 提供商（ollama/deepseek/siliconflow） | ollama |
| `AI_COMMIT_MODEL` | AI 模型名称 | mistral |
| `AI_COMMIT_DEEPSEEK_API_KEY` | Deepseek API 密钥 | - |
| `AI_COMMIT_DEEPSEEK_URL` | Deepseek API 地址 | https://api.deepseek.com/v1/chat/completions |
| `AI_COMMIT_OLLAMA_URL` | Ollama API 地址 | http://localhost:11434/api/generate |
| `AI_COMMIT_SILICONFLOW_API_KEY` | SiliconFlow API 密钥 | - |
| `AI_COMMIT_SILICONFLOW_URL` | SiliconFlow API 地址 | https://api.siliconflow.cn/v1/chat/completions |
| `AI_COMMIT_DEBUG` | 调试模式（true/false/1/0） | false |

### AI 提供商配置

**Ollama（默认）：**
- 本地运行，需要先安装 Ollama
- 默认模型：`mistral`
- 默认地址：`http://localhost:11434/api/generate`

**Deepseek：**
- 云端服务，需要 API Key
- 设置：`AI_COMMIT_DEEPSEEK_API_KEY=your-key`
- 默认地址：`https://api.deepseek.com/v1/chat/completions`

**SiliconFlow：**
- 云端服务，需要 API Key  
- 设置：`AI_COMMIT_SILICONFLOW_API_KEY=your-key`
- 默认地址：`https://api.siliconflow.cn/v1/chat/completions`

### 调试模式

通过设置 `AI_COMMIT_DEBUG` 环境变量可以控制输出详细程度：

- **关闭调试模式**（默认）：`AI_COMMIT_DEBUG=false` 或不设置
  - 只输出最终结果，不显示过程信息
  - 适合日常使用和自动化脚本

- **开启调试模式**：`AI_COMMIT_DEBUG=true` 或 `AI_COMMIT_DEBUG=1`
  - 显示详细的操作过程
  - 包含 AI 生成耗时、大型变更检测、标签创建等信息
  - 适合调试和了解工具运行过程

### 配置文件

配置优先级（从高到低）：
1. 命令行参数
2. 环境变量（`AI_COMMIT_*`）
3. `.env` 文件（用户目录：`~/.ai-commit/.env`，然后是当前目录 `.env`）
4. 默认值

### 示例配置

创建 `.env` 文件：

```bash
# 使用 SiliconFlow（推荐）
AI_COMMIT_PROVIDER=siliconflow
AI_COMMIT_MODEL=Qwen/Qwen2.5-7B-Instruct
AI_COMMIT_SILICONFLOW_API_KEY=your-siliconflow-key

# 使用 Deepseek
AI_COMMIT_PROVIDER=deepseek
AI_COMMIT_MODEL=deepseek-chat
AI_COMMIT_DEEPSEEK_API_KEY=your-deepseek-key

# 使用本地 Ollama（默认）
AI_COMMIT_PROVIDER=ollama
AI_COMMIT_MODEL=mistral
AI_COMMIT_OLLAMA_URL=http://localhost:11434/api/generate

# 调试模式（开发时可开启）
AI_COMMIT_DEBUG=false
```