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

ai-commit 是一个基于 Rust 的智能 Git 提交工具，集成本地/云端大模型（如 Ollama、Deepseek），可自动根据代码变更生成符合 Conventional Commits 规范的中文提交信息，提升团队协作效率和提交规范性。

---

## 主要功能

- 自动生成规范化的 Git commit message（支持中文，主题不超 50 字）
- 支持 Ollama、Deepseek 等多种 AI provider
- 可自定义模型、API 地址、API Key
- 自动 git add/commit/push，参数可控
- 支持自定义提交规范模板
- 命令行参数与 .env 配置灵活切换

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

| 简称/全称           | 说明                                         | 默认值      |
|---------------------|----------------------------------------------|-------------|
| -P, --provider      | AI 提交生成服务（ollama/deepseek）           | ollama      |
| -m, --model         | AI 模型名称                                  | mistral     |
| -n, --no-add        | 不自动执行 git add .                         | false       |
| -p, --push          | commit 后自动 git push                       | false       |
| -t, --new-tag [VER] | 创建新 tag（可指定版本号，如 -t v1.2.0）     |             |
| -s, --show-tag      | 显示最新的 tag 和备注                        | false       |
| -b, --push-branches | 推 tag 时同时推 master develop main 分支     | false       |

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

```sh
# 有变更时自动 commit 并打 tag
$ git add .
$ ai-commit -t -p

# 无变更时只打 tag
$ ai-commit -t -p

# 指定 tag note
$ ai-commit -t -p --tag-note "发布 v1.2.3"
```

## 配置说明

- 支持通过 `.env` 文件配置：
  - `AI_COMMIT_PROVIDER`