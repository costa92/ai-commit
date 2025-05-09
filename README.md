
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



## 配置说明

- 支持通过 `.env` 文件配置：
  - `AI_COMMIT_PROVIDER`：AI 服务类型（ollama/deepseek）
  - `AI_COMMIT_MODEL`：模型名称
  - `AI_COMMIT_OLLAMA_URL`：Ollama API 地址
  - `AI_COMMIT_DEEPSEEK_URL`：Deepseek API 地址
  - `AI_COMMIT_DEEPSEEK_API_KEY`：Deepseek API Key
  - `AI_COMMIT_PROMPT_PATH`：自定义提示词模板路径

---

## 提交规范

- 采用 Conventional Commits 规范，示例：
  
  ```bash
  feat(parser): 支持数组解析
  
  在新解析模块中增加了对数组的解析能力。
  ```

- 主题必须为中文，且不超过 50 个字
- 规范内容可在 `commit-prompt.txt` 配置

---

## 文档

- [使用文档](docs/安装与使用.md)
- [需求文档](docs/需求文档.md)
- [技术文档](docs/技术文档.md)

---

## 贡献

- 欢迎 issue/PR 反馈与贡献
- 推荐使用 `cargo fmt` 格式化代码
