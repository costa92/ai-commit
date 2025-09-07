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

ai-commit 是一个基于 Rust 的功能丰富的智能 Git 工具，集成本地/云端大模型（如 Ollama、Deepseek、SiliconFlow），提供 AI 智能提交、Git Flow 工作流、历史日志查看、提交编辑、Worktree 管理等全套 Git 操作功能。可自动生成符合 Conventional Commits 规范的中文提交信息，支持复杂的 Git 工作流管理，大幅提升开发效率和代码质量管理。

---

## 主要功能

### 🤖 核心 AI 提交功能
- 自动生成规范化的 Git commit message（支持中文，主题不超 50 字）
- 支持 Ollama、Deepseek、SiliconFlow 等多种 AI provider
- 可自定义模型、API 地址、API Key
- 自动 git add/commit/push，参数可控
- 支持自定义提交规范模板

### 🏷️ Tag 管理功能
- 智能创建和管理 Git tags
- 列出、删除、查看 tag 详细信息
- 比较不同 tags 之间的差异
- 自动版本递增和冲突解决
- 推送 tags 时同步主分支

### 🌊 Git Flow 工作流
- 完整的 Git Flow 工作流支持
- Feature/Hotfix/Release 分支管理
- 自动化分支创建、合并和清理
- 符合业界标准的分支策略

### 📊 历史日志查看
- 美化的提交历史展示
- 多维度过滤（作者、时间、文件、关键词）
- 图形化分支历史和统计信息
- 贡献者统计和活跃度分析

### 🌳 Git Worktree 管理
- 多分支并行开发支持
- Worktree 创建、切换、清理
- 支持自定义路径和批量操作
- 完整的生命周期管理

### ✏️ 提交编辑功能
- 修改历史提交（amend、rebase、reword）
- 交互式提交编辑和撤销
- 安全的提交历史修改

### 🔍 高级查询监控（GRV-inspired）
- 复合条件查询和过滤
- 实时仓库变化监控
- 增强的差异查看和交互式历史浏览

### 🛠️ 配置和环境
- 命令行参数与 .env 配置灵活切换
- 调试模式支持，可控制输出详细程度
- 多语言支持（中文/英文）
- 多种配置源优先级管理

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

### 基础 AI 提交参数

| 简称/全称        | 说明                                         | 默认值      |
|------------------|----------------------------------------------|-------------|
| -P, --provider   | AI 提交生成服务（ollama/deepseek/siliconflow） | ollama      |
| -m, --model      | AI 模型名称                                  | mistral     |
| -n, --no-add     | 不自动执行 git add .                         | false       |
| -p, --push       | commit 后自动 git push                       | false       |

### Tag 管理参数

| 参数                    | 说明                                         | 默认值      |
|-------------------------|----------------------------------------------|-------------|
| -t, --new-tag [VER]     | 创建新 tag（可指定版本号，如 -t v1.2.0）     |             |
| --tag-note NOTE         | tag 备注内容，不指定则用 AI 生成             |             |
| -s, --show-tag          | 显示最新的 tag 和备注                        | false       |
| -b, --push-branches     | 推 tag 时同时推 master develop main 分支     | false       |
| --tag-list              | 列出所有 tags                                | false       |
| --tag-delete TAG        | 删除指定的 tag（本地和远程）                 |             |
| --tag-info TAG          | 显示指定 tag 的详细信息                      |             |
| --tag-compare TAG1..TAG2| 比较两个 tags 之间的差异                     |             |

### Git Flow 工作流参数

| 参数                        | 说明                                       |
|-----------------------------|---------------------------------------------|
| --flow-init                 | 初始化 git flow 仓库结构                   |
| --flow-feature-start NAME   | 开始新的 feature 分支                      |
| --flow-feature-finish NAME  | 完成 feature 分支（合并到 develop）        |
| --flow-hotfix-start NAME    | 开始新的 hotfix 分支                       |
| --flow-hotfix-finish NAME   | 完成 hotfix 分支（合并到 main 和 develop） |
| --flow-release-start VERSION| 开始新的 release 分支                      |
| --flow-release-finish VERSION| 完成 release 分支（合并到 main 和 develop，创建 tag）|

### 历史日志查看参数

| 参数                    | 说明                           |
|-------------------------|--------------------------------|
| --history               | 显示提交历史（美化格式）       |
| --log-author AUTHOR     | 按作者过滤历史记录             |
| --log-since DATE        | 显示指定时间之后的历史记录     |
| --log-until DATE        | 显示指定时间之前的历史记录     |
| --log-graph             | 显示图形化分支历史             |
| --log-limit N           | 限制显示的提交数量             |
| --log-file PATH         | 按文件路径过滤历史记录         |
| --log-stats             | 显示提交统计信息               |
| --log-contributors      | 显示贡献者统计                 |
| --log-search TERM       | 搜索提交消息中的关键词         |
| --log-branches          | 显示所有分支的历史图           |

### Git Worktree 管理参数

| 参数                     | 说明                                  |
|--------------------------|---------------------------------------|
| --worktree-create BRANCH | 创建新的 Git worktree                 |
| --worktree-switch NAME   | 切换到指定的 worktree                 |
| --worktree-list          | 列出所有可用的 worktrees              |
| --worktree-verbose, -v   | worktree list 详细模式                |
| --worktree-porcelain     | worktree list 机器可读输出            |
| --worktree-z, -z         | worktree list 使用NUL字符终止记录     |
| --worktree-expire TIME   | worktree list 显示过期时间注释        |
| --worktree-remove NAME   | 删除指定的 worktree                   |
| --worktree-path PATH     | 指定 worktree 创建的自定义路径        |
| --worktree-clear         | 清空除当前外的所有其他 worktrees      |

### 提交编辑参数

| 参数                        | 说明                                       |
|-----------------------------|---------------------------------------------|
| --amend                     | 修改最后一次提交                           |
| --edit-commit COMMIT_HASH   | 交互式修改指定的提交（使用 rebase）        |
| --rebase-edit BASE_COMMIT   | 交互式 rebase 修改多个提交                 |
| --reword-commit COMMIT_HASH | 重写提交消息（不改变内容）                 |
| --undo-commit               | 撤销最后一次提交（保留文件修改）           |

### 高级查询监控参数

| 参数                   | 说明                       |
|------------------------|----------------------------|
| --query QUERY          | 查询过滤器（支持复合条件） |
| --watch                | 监控仓库变化               |
| --diff-view COMMIT     | 显示增强的差异查看         |
| --interactive-history  | 交互式历史浏览             |

> 所有参数均支持简称和全称，可混用。详见 `ai-commit --help`。

---

## 功能对应的 Git 命令表

下表展示了 ai-commit 参数与对应的原生 Git 命令的关系：

### 基础 Git 操作

| ai-commit 参数 | 对应 Git 命令 | 说明 |
|----------------|---------------|------|
| （默认行为）| `git add .` + `git commit -m` | AI 生成提交消息并提交 |
| -n, --no-add | 跳过 `git add .` | 只提交已暂存的文件 |
| -p, --push | `git push` | 提交后自动推送 |

### Tag 管理功能

| ai-commit 参数 | 对应 Git 命令 | 说明 |
|----------------|---------------|------|
| -t, --new-tag | `git tag -a` + `git push --tags` | 创建带注释的标签 |
| --tag-list | `git tag -l` + `git show-ref --tags` | 列出所有标签 |
| --tag-delete | `git tag -d` + `git push --delete origin` | 删除本地和远程标签 |
| --tag-info | `git show` + `git log --oneline` | 显示标签详细信息 |
| --tag-compare | `git log --oneline TAG1..TAG2` | 比较标签间差异 |
| -s, --show-tag | `git describe --tags` + `git tag -l -n` | 显示最新标签信息 |

### Git Flow 工作流

| ai-commit 参数 | 对应 Git 命令 | 说明 |
|----------------|---------------|------|
| --flow-init | `git branch develop` + 分支设置 | 初始化 Git Flow 结构 |
| --flow-feature-start | `git checkout -b feature/NAME develop` | 从 develop 创建 feature 分支 |
| --flow-feature-finish | `git checkout develop` + `git merge --no-ff` | 合并 feature 到 develop |
| --flow-hotfix-start | `git checkout -b hotfix/NAME main` | 从 main 创建 hotfix 分支 |
| --flow-hotfix-finish | `git checkout main` + `git merge` + `git checkout develop` + `git merge` | 合并到 main 和 develop |
| --flow-release-start | `git checkout -b release/VER develop` | 从 develop 创建 release 分支 |
| --flow-release-finish | `git checkout main` + `git merge` + `git tag` + `git checkout develop` + `git merge` | 完整的发布流程 |

### 历史日志查看

| ai-commit 参数 | 对应 Git 命令 | 说明 |
|----------------|---------------|------|
| --history | `git log --oneline --decorate --color` | 美化的提交历史 |
| --log-author | `git log --author="AUTHOR"` | 按作者过滤 |
| --log-since | `git log --since="DATE"` | 指定时间之后的提交 |
| --log-until | `git log --until="DATE"` | 指定时间之前的提交 |
| --log-graph | `git log --graph --all --oneline` | 图形化分支历史 |
| --log-limit | `git log -n NUMBER` | 限制显示数量 |
| --log-file | `git log --follow -- PATH` | 文件历史记录 |
| --log-stats | `git log --stat` | 显示提交统计 |
| --log-contributors | `git shortlog -sn` | 贡献者统计 |
| --log-search | `git log --grep="TERM"` | 搜索提交消息 |
| --log-branches | `git log --graph --all --decorate` | 所有分支的历史图 |

### Git Worktree 管理

| ai-commit 参数 | 对应 Git 命令 | 说明 |
|----------------|---------------|------|
| --worktree-create | `git worktree add PATH BRANCH` | 创建新的工作树 |
| --worktree-switch | `cd WORKTREE_PATH` | 切换到指定工作树 |
| --worktree-list | `git worktree list` | 列出所有工作树 |
| --worktree-verbose | `git worktree list -v` | 详细模式列出工作树 |
| --worktree-porcelain | `git worktree list --porcelain` | 机器可读格式输出 |
| --worktree-z | `git worktree list -z` | NUL 字符分隔输出 |
| --worktree-expire | `git worktree list --expire TIME` | 显示过期时间注释 |
| --worktree-remove | `git worktree remove NAME` + `git worktree prune` | 删除工作树并清理 |
| --worktree-clear | `git worktree remove` + `git worktree prune` | 批量清理工作树 |

### 提交编辑功能

| ai-commit 参数 | 对应 Git 命令 | 说明 |
|----------------|---------------|------|
| --amend | `git commit --amend` | 修改最后一次提交 |
| --edit-commit | `git rebase -i COMMIT^` | 交互式 rebase 编辑提交 |
| --rebase-edit | `git rebase -i BASE_COMMIT` | 交互式 rebase 多个提交 |
| --reword-commit | `git rebase -i COMMIT^` (reword) | 重写提交消息 |
| --undo-commit | `git reset --soft HEAD^` | 撤销提交保留修改 |

### 高级查询监控

| ai-commit 参数 | 对应 Git 命令组合 | 说明 |
|----------------|-------------------|------|
| --query | `git log` + 多种过滤器组合 | 复合条件查询（自定义解析） |
| --watch | `git status` + 文件系统监控 | 实时监控仓库变化 |
| --diff-view | `git show COMMIT` + 彩色输出 | 增强差异查看 |
| --interactive-history | `git log` + 交互式界面 | 交互式历史浏览 |

### 特殊功能

| ai-commit 功能 | 实现方式 | 说明 |
|----------------|----------|------|
| AI 提交消息生成 | `git diff` + AI API 调用 + `git commit -m` | 智能分析差异生成符合规范的提交消息 |
| 自动版本递增 | `git tag -l` + 语义版本解析 + `git tag` | 智能解析现有标签并自动递增版本号 |
| 多 AI 提供商支持 | HTTP 客户端 + 多种 API 适配 | 支持 Ollama、Deepseek、SiliconFlow 等 |
| 调试模式 | 环境变量控制 + 详细日志输出 | 可控制的详细过程输出 |

---

## 新特性与亮点

### 🚀 全面升级的 Git 工具集
- **完整的 Git Flow 支持**：Feature/Hotfix/Release 分支自动化管理
- **智能 Tag 管理**：版本自动递增、冲突解决、批量操作
- **高级历史分析**：多维度过滤、统计分析、图形化展示
- **强大的提交编辑**：安全的历史修改、交互式 rebase
- **Worktree 并行开发**：多分支同时开发，提升开发效率

### 🤖 AI 智能化升级
- **优化的提示模板**：极简指令式模板，减少英文污染
- **多模型支持**：Ollama、Deepseek、SiliconFlow 无缝切换
- **智能差异分析**：大文件变更自动摘要生成
- **调试模式**：详细过程展示，问题快速定位

### 🔍 GRV-inspired 功能
- **复合条件查询**：支持作者、时间、类型等多维度组合查询
- **实时监控**：仓库变化实时跟踪和通知
- **增强差异查看**：彩色语法高亮、统计信息展示
- **交互式历史浏览**：类似 GRV 的直观操作体验

### 🛠️ 开发体验优化
- **多平台支持**：Linux/musl、macOS Intel/ARM、Windows 一键构建
- **CI/CD 自动化**：GitHub Actions 自动发布和测试
- **灵活配置系统**：CLI 参数、环境变量、配置文件多级优先级
- **完整测试覆盖**：326+ 单元测试，确保功能稳定性
- **丰富文档**：详细使用说明和最佳实践指南

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

### Tag 管理示例

```sh
# 列出所有 tags
$ ai-commit --tag-list

# 查看特定 tag 信息
$ ai-commit --tag-info v1.0.0

# 比较两个 tags 的差异
$ ai-commit --tag-compare v1.0.0..v1.1.0

# 删除指定 tag
$ ai-commit --tag-delete v0.9.0-beta
```

### Git Flow 工作流示例

```sh
# 初始化 Git Flow
$ ai-commit --flow-init

# 开始新功能开发
$ ai-commit --flow-feature-start user-auth
# 在 feature/user-auth 分支上开发...
# 完成功能开发
$ ai-commit --flow-feature-finish user-auth

# 开始 hotfix
$ ai-commit --flow-hotfix-start critical-bug
# 修复完成后
$ ai-commit --flow-hotfix-finish critical-bug

# 开始 release
$ ai-commit --flow-release-start v1.2.0
# 准备发布后
$ ai-commit --flow-release-finish v1.2.0
```

### 历史日志查看示例

```sh
# 查看美化的提交历史
$ ai-commit --history

# 查看图形化分支历史
$ ai-commit --log-graph

# 按作者过滤
$ ai-commit --log-author "张三"

# 按时间范围查看
$ ai-commit --log-since "2024-01-01" --log-until "2024-12-31"

# 查看指定文件的历史
$ ai-commit --log-file src/main.rs

# 显示贡献者统计
$ ai-commit --log-contributors

# 搜索提交消息
$ ai-commit --log-search "修复"

# 组合使用多个选项
$ ai-commit --history --log-author "李四" --log-limit 10 --log-graph
```

### 提交编辑示例

```sh
# 修改最后一次提交
$ ai-commit --amend

# 编辑指定提交（交互式 rebase）
$ ai-commit --edit-commit abc1234

# 重写提交消息
$ ai-commit --reword-commit def5678

# 撤销最后一次提交（保留修改）
$ ai-commit --undo-commit

# 交互式修改多个提交
$ ai-commit --rebase-edit HEAD~5
```

### 高级查询监控示例

```sh
# 复合条件查询
$ ai-commit --query "author:张三,since:2024-01-01,type:feat"

# 监控仓库变化（实时）
$ ai-commit --watch

# 增强差异查看
$ ai-commit --diff-view HEAD~1

# 交互式历史浏览
$ ai-commit --interactive-history
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