# ai-commit

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

| 参数           | 说明                                   | 默认值      |
|----------------|----------------------------------------|-------------|
| --provider     | AI 提交生成服务（ollama/deepseek）     | ollama      |
| --model/-m     | AI 模型名称                            | mistral     |
| --no-add       | 不自动执行 git add .                   | false       |
| --push         | commit 后自动 git push                 | false       |

---

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
