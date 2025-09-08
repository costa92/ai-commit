# AI 提供商配置指南

## 概述

ai-commit 采用了**配置驱动的提供商架构**，添加新的 AI 提供商变得非常简单。你只需要修改配置文件，无需修改代码即可支持新的 AI 服务。

## 配置文件位置

系统会按以下优先级顺序加载配置文件：

1. `./providers.toml` (当前目录)
2. `./config/providers.toml`
3. `/etc/ai-commit/providers.toml` (系统配置目录)
4. 内置默认配置 (备用)

**推荐**：将 `providers.toml` 放在项目根目录，这样便于版本控制和团队共享。

## 配置文件格式

### 基本结构

```toml
# providers.toml
[providers.供应商名称]
name = "内部标识符"
display_name = "显示名称"
default_url = "默认API地址"
requires_api_key = true/false
default_model = "默认模型"
supported_models = ["模型1", "模型2"]
api_format = "openai" | "ollama" | "custom"
description = "描述信息"
env_prefix = "环境变量前缀"
```

### 字段说明

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `name` | String | ✅ | 提供商内部标识符，用于命令行 `--provider` 参数 |
| `display_name` | String | ✅ | 用户友好的显示名称 |
| `default_url` | String | ✅ | 默认 API 服务地址 |
| `requires_api_key` | Boolean | ✅ | 是否需要 API Key |
| `default_model` | String | ✅ | 默认使用的模型 |
| `supported_models` | Array | ✅ | 支持的模型列表 |
| `api_format` | String | ✅ | API 格式：`openai`/`ollama`/`custom` |
| `env_prefix` | String | ✅ | 环境变量前缀，如 `AI_COMMIT_KIMI` |
| `description` | String | ✅ | 提供商描述信息 |

## 现有提供商配置

### 1. Ollama (本地服务)

Ollama 是一个本地运行的 AI 模型服务，支持多种开源大语言模型。

```toml
[providers.ollama]
name = "ollama"
display_name = "Ollama"
default_url = "http://localhost:11434/api/generate"
requires_api_key = false
default_model = "mistral"
supported_models = ["mistral", "llama3", "qwen2", "codellama", "gemma", "phi3"]
api_format = "ollama"
description = "本地 Ollama 服务，无需 API Key"
env_prefix = "AI_COMMIT_OLLAMA"
```

**安装和设置**：

1. **安装 Ollama**：
   ```bash
   # macOS
   brew install ollama
   
   # Linux
   curl -fsSL https://ollama.ai/install.sh | sh
   
   # Windows
   # 下载并安装 https://ollama.ai/download/windows
   ```

2. **启动 Ollama 服务**：
   ```bash
   ollama serve
   ```

3. **拉取推荐模型**：
   ```bash
   # 拉取 mistral 模型（默认）
   ollama pull mistral
   
   # 或其他支持的模型
   ollama pull llama3
   ollama pull qwen2
   ollama pull codellama
   ```

**环境变量配置**：
- `AI_COMMIT_OLLAMA_URL`: 自定义 Ollama 服务地址（可选）

**使用示例**：
```bash
# 使用默认配置
ai-commit --provider ollama

# 指定模型
ai-commit --provider ollama --model llama3

# 自定义服务地址
export AI_COMMIT_OLLAMA_URL="http://192.168.1.100:11434/api/generate"
ai-commit --provider ollama
```

**支持的模型**：
- `mistral`: 轻量级高性能模型（默认推荐）
- `llama3`: Meta 的 LLaMA 3 模型
- `qwen2`: 阿里巴巴通义千问 2.0
- `codellama`: 专门针对代码优化的 LLaMA
- `gemma`: Google 的 Gemma 模型
- `phi3`: 微软的 Phi-3 模型

**优势**：
- ✅ 完全本地运行，数据隐私安全
- ✅ 无需 API Key，无使用成本
- ✅ 支持离线使用
- ✅ 多种模型可选

**注意事项**：
- 需要本地安装 Ollama 服务
- 首次使用需要下载模型文件（通常几GB）
- 对硬件性能有一定要求

### 2. Deepseek

Deepseek 是深度求索公司提供的 AI 服务，擅长代码理解和生成，性价比极高。

```toml
[providers.deepseek]
name = "deepseek"
display_name = "Deepseek"
default_url = "https://api.deepseek.com/v1/chat/completions"
requires_api_key = true
default_model = "deepseek-chat"
supported_models = ["deepseek-chat", "deepseek-coder"]
api_format = "openai"
description = "深度求索 AI 服务，需要 API Key"
env_prefix = "AI_COMMIT_DEEPSEEK"
```

**获取 API Key**：

1. **注册账号**：
   - 访问 [https://platform.deepseek.com/](https://platform.deepseek.com/)
   - 注册账号并完成身份验证

2. **获取 API Key**：
   - 登录控制台，进入 API Keys 页面
   - 点击"创建新密钥"
   - 复制并保存 API Key（sk-xxx 格式）

3. **充值余额**：
   - 新用户通常有免费额度
   - 需要时在控制台进行充值

**环境变量配置**：
- `AI_COMMIT_DEEPSEEK_API_KEY`: Deepseek API Key (**必需**)
- `AI_COMMIT_DEEPSEEK_URL`: 自定义 API 地址（可选）

**使用示例**：
```bash
# 设置 API Key
export AI_COMMIT_DEEPSEEK_API_KEY="sk-xxxxxxxxxxxxxxxx"

# 使用 Deepseek
ai-commit --provider deepseek

# 使用代码专用模型
ai-commit --provider deepseek --model deepseek-coder

# 通过 .env 文件配置
echo "AI_COMMIT_DEEPSEEK_API_KEY=sk-xxxxxxxxxxxxxxxx" >> ~/.ai-commit/.env
```

**支持的模型**：
- `deepseek-chat`: 通用对话模型，平衡性能和成本
- `deepseek-coder`: 代码专用模型，更适合编程任务（**推荐用于 commit**）

**定价优势**：
- ✅ 价格极具竞争力（约 $0.14/1M tokens 输入）
- ✅ 代码理解和生成能力优秀
- ✅ 支持中文和英文
- ✅ API 兼容 OpenAI 格式

**最佳实践**：
```bash
# 建议配置：使用代码专用模型 + 环境变量
export AI_COMMIT_PROVIDER=deepseek
export AI_COMMIT_MODEL=deepseek-coder
export AI_COMMIT_DEEPSEEK_API_KEY="your-api-key"

# 直接使用
ai-commit
```

### 3. SiliconFlow

SiliconFlow（硅基流动）是一个高性能的 AI 推理平台，提供多种开源模型的云端推理服务。

```toml
[providers.siliconflow]
name = "siliconflow"
display_name = "SiliconFlow"
default_url = "https://api.siliconflow.cn/v1/chat/completions"
requires_api_key = true
default_model = "qwen/Qwen2-7B-Instruct"
supported_models = [
    "qwen/Qwen2-7B-Instruct",
    "qwen/Qwen2-72B-Instruct",
    "deepseek-ai/deepseek-coder-6.7b-instruct",
    "01-ai/Yi-34B-Chat-4bits"
]
api_format = "openai"
description = "硅基流动 AI 服务，需要 API Key"
env_prefix = "AI_COMMIT_SILICONFLOW"
```

**获取 API Key**：

1. **注册账号**：
   - 访问 [https://siliconflow.cn/](https://siliconflow.cn/)
   - 使用邮箱或手机号注册账号

2. **获取 API Key**：
   - 登录后进入控制台
   - 导航到 "API 密钥" 或 "API Keys" 页面
   - 创建新的 API 密钥
   - 复制密钥（通常以 sk- 开头）

3. **获取免费额度**：
   - 新用户通常有免费试用额度
   - 查看计费页面了解具体额度

**环境变量配置**：
- `AI_COMMIT_SILICONFLOW_API_KEY`: SiliconFlow API Key (**必需**)
- `AI_COMMIT_SILICONFLOW_URL`: 自定义 API 地址（可选）

**使用示例**：
```bash
# 设置 API Key
export AI_COMMIT_SILICONFLOW_API_KEY="sk-xxxxxxxxxxxxxxxx"

# 使用默认模型
ai-commit --provider siliconflow

# 使用大型模型（推荐）
ai-commit --provider siliconflow --model "qwen/Qwen2-72B-Instruct"

# 使用代码专用模型
ai-commit --provider siliconflow --model "deepseek-ai/deepseek-coder-6.7b-instruct"

# 通过配置文件设置
cat >> ~/.ai-commit/.env << EOF
AI_COMMIT_PROVIDER=siliconflow
AI_COMMIT_MODEL=qwen/Qwen2-72B-Instruct
AI_COMMIT_SILICONFLOW_API_KEY=sk-xxxxxxxxxxxxxxxx
EOF
```

**支持的模型**：
- `qwen/Qwen2-7B-Instruct`: 阿里通义千问 7B，平衡性能和速度（默认）
- `qwen/Qwen2-72B-Instruct`: 通义千问 72B，更强的推理能力（**推荐**）
- `deepseek-ai/deepseek-coder-6.7b-instruct`: 代码专用模型
- `01-ai/Yi-34B-Chat-4bits`: 零一万物 Yi 模型，中文能力强

**平台优势**：
- ✅ 多种开源模型可选
- ✅ 高性能推理，响应速度快
- ✅ 支持中文和英文
- ✅ 价格相对合理
- ✅ 国内访问稳定

**模型选择建议**：
```bash
# 追求质量：使用 72B 模型
export AI_COMMIT_MODEL="qwen/Qwen2-72B-Instruct"

# 专门处理代码：使用代码专用模型
export AI_COMMIT_MODEL="deepseek-ai/deepseek-coder-6.7b-instruct"

# 平衡性能：使用默认 7B 模型
export AI_COMMIT_MODEL="qwen/Qwen2-7B-Instruct"
```

**注意事项**：
- 模型名称必须使用完整路径（包含组织名）
- 不同模型的计费标准可能不同
- 建议先用小模型测试，确认效果后再使用大模型

### 4. Kimi

Kimi 是月之暗面（Moonshot AI）开发的大语言模型，具有超长上下文理解能力，擅长处理复杂的推理任务。

```toml
[providers.kimi]
name = "kimi"
display_name = "Kimi"
default_url = "https://api.moonshot.cn/v1/chat/completions"
requires_api_key = true
default_model = "moonshot-v1-8k"
supported_models = ["moonshot-v1-8k", "moonshot-v1-32k", "moonshot-v1-128k"]
api_format = "openai"
description = "月之暗面 Kimi AI 服务，需要 API Key"
env_prefix = "AI_COMMIT_KIMI"
```

**获取 API Key**：

1. **注册账号**：
   - 访问 [https://platform.moonshot.cn/](https://platform.moonshot.cn/)
   - 使用邮箱注册并验证账号

2. **获取 API Key**：
   - 登录开放平台控制台
   - 进入 "API Key 管理" 页面
   - 点击 "新建" 创建 API Key
   - 复制并安全保存 API Key

3. **充值和计费**：
   - 新用户通常有一定免费额度
   - 可在控制台查看用量和进行充值
   - 按 token 使用量计费

**环境变量配置**：
- `AI_COMMIT_KIMI_API_KEY`: Kimi API Key (**必需**)
- `AI_COMMIT_KIMI_URL`: 自定义 API 地址（可选）

**使用示例**：
```bash
# 设置 API Key
export AI_COMMIT_KIMI_API_KEY="sk-xxxxxxxxxxxxxxxx"

# 使用默认模型（8k 上下文）
ai-commit --provider kimi

# 使用更大上下文模型（适合大型项目）
ai-commit --provider kimi --model moonshot-v1-32k

# 处理超大变更（128k 上下文）
ai-commit --provider kimi --model moonshot-v1-128k

# 配置为默认提供商
cat > ~/.ai-commit/.env << EOF
AI_COMMIT_PROVIDER=kimi
AI_COMMIT_MODEL=moonshot-v1-32k
AI_COMMIT_KIMI_API_KEY=your-kimi-api-key
EOF
```

**支持的模型**：
- `moonshot-v1-8k`: 8K 上下文长度，适合常规提交（默认）
- `moonshot-v1-32k`: 32K 上下文长度，适合中大型变更（**推荐**）
- `moonshot-v1-128k`: 128K 上下文长度，适合超大型重构

**核心优势**：
- ✅ 超长上下文理解能力，可处理大型代码变更
- ✅ 优秀的中文理解和生成能力
- ✅ 强大的推理和逻辑分析能力
- ✅ API 完全兼容 OpenAI 格式
- ✅ 响应速度快，服务稳定

**使用场景**：
```bash
# 小型提交：使用 8k 模型
export AI_COMMIT_MODEL=moonshot-v1-8k

# 中型重构：使用 32k 模型（推荐）
export AI_COMMIT_MODEL=moonshot-v1-32k

# 大型重构：使用 128k 模型
export AI_COMMIT_MODEL=moonshot-v1-128k
```

**性能建议**：
- 对于日常小型提交，使用 `moonshot-v1-8k` 即可满足需求且成本较低
- 对于包含多个文件的中型变更，推荐使用 `moonshot-v1-32k`
- 只有在处理超大型重构时才需要使用 `moonshot-v1-128k`
- 可根据项目规模和预算选择合适的模型

**最佳实践**：
```bash
# 推荐配置：平衡性能和成本
export AI_COMMIT_PROVIDER=kimi
export AI_COMMIT_MODEL=moonshot-v1-32k
export AI_COMMIT_KIMI_API_KEY="your-api-key"

# 针对不同场景快速切换
alias ai-commit-small="ai-commit --model moonshot-v1-8k"
alias ai-commit-large="ai-commit --model moonshot-v1-128k"
```

## 提供商对比和选择指南

### 快速对比表

| 特性 | Ollama | Deepseek | SiliconFlow | Kimi |
|------|--------|----------|-------------|------|
| **成本** | 免费 | 极低 | 中等 | 中等 |
| **隐私性** | ✅ 完全本地 | ❌ 云端 | ❌ 云端 | ❌ 云端 |
| **安装难度** | 中等 | 简单 | 简单 | 简单 |
| **中文能力** | ✅ 优秀 | ✅ 优秀 | ✅ 优秀 | ✅ 出色 |
| **代码理解** | ✅ 良好 | ✅ 出色 | ✅ 优秀 | ✅ 优秀 |
| **响应速度** | 取决于硬件 | ✅ 快 | ✅ 快 | ✅ 快 |
| **上下文长度** | 模型依赖 | 标准 | 标准 | ✅ 超长 |
| **离线使用** | ✅ 支持 | ❌ 不支持 | ❌ 不支持 | ❌ 不支持 |

### 使用场景推荐

#### 1. 个人开发者（追求隐私和零成本）
**推荐：Ollama**
```bash
# 一次性设置，永久免费使用
export AI_COMMIT_PROVIDER=ollama
export AI_COMMIT_MODEL=mistral
```

**优势**：
- 完全本地运行，代码不会上传到云端
- 零使用成本，适合个人项目
- 支持离线使用

#### 2. 专业开发者（追求性价比）
**推荐：Deepseek**
```bash
# 极高性价比，代码理解能力强
export AI_COMMIT_PROVIDER=deepseek
export AI_COMMIT_MODEL=deepseek-coder
export AI_COMMIT_DEEPSEEK_API_KEY="your-api-key"
```

**优势**：
- 价格极具竞争力
- deepseek-coder 专门针对代码优化
- 支持中文和英文

#### 3. 团队协作（追求稳定性和多模型选择）
**推荐：SiliconFlow**
```bash
# 多模型可选，国内访问稳定
export AI_COMMIT_PROVIDER=siliconflow
export AI_COMMIT_MODEL="qwen/Qwen2-72B-Instruct"
export AI_COMMIT_SILICONFLOW_API_KEY="your-api-key"
```

**优势**：
- 多种开源模型可选
- 国内访问稳定
- 不同规模项目可选择不同模型

#### 4. 大型项目（处理复杂重构）
**推荐：Kimi**
```bash
# 超长上下文，适合大型变更
export AI_COMMIT_PROVIDER=kimi
export AI_COMMIT_MODEL=moonshot-v1-128k
export AI_COMMIT_KIMI_API_KEY="your-api-key"
```

**优势**：
- 超长上下文理解能力
- 优秀的推理和逻辑分析
- 适合处理大型重构和复杂变更

### 成本对比

#### 典型使用场景成本估算（每月100次commit）

| 提供商 | 模型 | 估算月成本 | 说明 |
|--------|------|------------|------|
| Ollama | mistral | ¥0 | 完全免费，本地运行 |
| Deepseek | deepseek-coder | ¥5-15 | 性价比极高 |
| SiliconFlow | qwen/Qwen2-7B | ¥10-25 | 中等成本 |
| SiliconFlow | qwen/Qwen2-72B | ¥25-50 | 质量更高 |
| Kimi | moonshot-v1-8k | ¥15-30 | 标准上下文 |
| Kimi | moonshot-v1-32k | ¥30-60 | 大上下文 |

*注：实际成本会因使用频率、提交大小等因素而变化*

### 混合使用策略

你可以为不同场景配置不同的提供商：

```bash
# 创建多个配置别名
alias ai-commit-free="AI_COMMIT_PROVIDER=ollama ai-commit"
alias ai-commit-fast="AI_COMMIT_PROVIDER=deepseek AI_COMMIT_MODEL=deepseek-coder ai-commit"
alias ai-commit-large="AI_COMMIT_PROVIDER=kimi AI_COMMIT_MODEL=moonshot-v1-128k ai-commit"

# 根据情况选择使用
ai-commit-free      # 日常小提交，使用免费本地服务
ai-commit-fast      # 需要高质量时，使用性价比最高的云服务
ai-commit-large     # 大型重构时，使用长上下文模型
```

## 如何添加新的提供商

### 方式一：修改配置文件

只需在 `providers.toml` 中添加新的配置段即可：

```toml
# 示例：添加 Claude 支持
[providers.claude]
name = "claude"
display_name = "Anthropic Claude"
default_url = "https://api.anthropic.com/v1/messages"
requires_api_key = true
default_model = "claude-3-sonnet-20240229"
supported_models = [
    "claude-3-sonnet-20240229", 
    "claude-3-haiku-20240307",
    "claude-3-opus-20240229"
]
api_format = "openai"  # 或 "custom" 如果需要特殊处理
description = "Anthropic Claude AI 服务，需要 API Key"
env_prefix = "AI_COMMIT_CLAUDE"
```

添加后即可通过以下方式使用：

```bash
# 设置环境变量
export AI_COMMIT_CLAUDE_API_KEY="your-claude-api-key"

# 使用命令行
ai-commit --provider claude --model claude-3-sonnet-20240229

# 或设置为默认
export AI_COMMIT_PROVIDER=claude
ai-commit
```

### 方式二：添加本地服务

```toml
[providers.local_llm]
name = "local_llm"
display_name = "Local LLM Server"
default_url = "http://localhost:8080/v1/chat/completions"
requires_api_key = false
default_model = "local-model"
supported_models = ["local-model", "custom-model"]
api_format = "openai"
description = "本地 LLM 服务器"
env_prefix = "AI_COMMIT_LOCAL_LLM"
```

## API 格式说明

### OpenAI 格式 (`api_format = "openai"`)

适用于兼容 OpenAI API 的服务，包括：
- Deepseek
- SiliconFlow  
- Kimi
- Claude (需要适配)
- 大部分云服务提供商

请求格式：
```json
{
  "model": "model-name",
  "messages": [
    {"role": "user", "content": "prompt"}
  ],
  "stream": true,
  "temperature": 0.7,
  "max_tokens": 500
}
```

### Ollama 格式 (`api_format = "ollama"`)

适用于 Ollama 本地服务。

请求格式：
```json
{
  "model": "model-name",
  "prompt": "prompt text",
  "stream": true
}
```

### 自定义格式 (`api_format = "custom"`)

如果你的 AI 服务使用特殊的 API 格式，可以设置为 `custom`，但需要在代码中添加相应的处理逻辑。

## 环境变量配置

### 统一的环境变量格式

每个提供商都遵循统一的环境变量命名规范：

- `AI_COMMIT_<PREFIX>_API_KEY`: API 密钥
- `AI_COMMIT_<PREFIX>_URL`: 自定义 API 地址
- `AI_COMMIT_PROVIDER`: 默认提供商
- `AI_COMMIT_MODEL`: 默认模型
- `AI_COMMIT_DEBUG`: 调试模式

### 配置优先级

1. **命令行参数** (最高优先级)
2. **环境变量**
3. **配置文件**
4. **默认值** (最低优先级)

### .env 文件示例

创建 `.env` 文件或 `~/.ai-commit/.env`：

```env
# 全局配置
AI_COMMIT_PROVIDER=kimi
AI_COMMIT_MODEL=moonshot-v1-8k
AI_COMMIT_DEBUG=false

# Kimi 配置
AI_COMMIT_KIMI_API_KEY=your-kimi-api-key

# Deepseek 配置 (备用)
AI_COMMIT_DEEPSEEK_API_KEY=your-deepseek-api-key

# 自定义 Ollama 地址
AI_COMMIT_OLLAMA_URL=http://192.168.1.100:11434/api/generate
```

## 使用方法

### 基本用法

```bash
# 使用默认提供商和模型
ai-commit

# 指定提供商
ai-commit --provider deepseek

# 指定提供商和模型
ai-commit --provider kimi --model moonshot-v1-32k

# 查看可用的提供商
ai-commit --help  # 会显示所有支持的提供商
```

### 高级用法

```bash
# 自动提交并推送
ai-commit --provider siliconflow --push

# 跳过确认
ai-commit --yes --provider ollama

# 创建标签
ai-commit --new-tag v1.0.0 --provider deepseek
```

## 故障排除

### 1. 配置文件未生效

检查文件位置和格式：
```bash
# 检查当前配置
ai-commit --provider nonexistent  # 会显示配置加载信息

# 验证 TOML 格式
toml-validate providers.toml  # 如果有 toml 工具
```

### 2. API Key 未设置

```bash
# 检查环境变量
env | grep AI_COMMIT

# 临时设置
export AI_COMMIT_DEEPSEEK_API_KEY="your-key"
ai-commit --provider deepseek
```

### 3. 网络连接问题

```bash
# 测试 API 连通性
curl -X POST https://api.deepseek.com/v1/chat/completions \
  -H "Authorization: Bearer your-api-key" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "deepseek-chat",
    "messages": [{"role": "user", "content": "test"}],
    "max_tokens": 10
  }'
```

### 4. 调试模式

```bash
export AI_COMMIT_DEBUG=true
ai-commit --provider your-provider
```

## 扩展和自定义

### 团队配置共享

1. 在项目根目录创建 `providers.toml`
2. 提交到版本控制系统
3. 团队成员只需设置各自的 API Key

### 多环境配置

```bash
# 开发环境
cp providers.dev.toml providers.toml

# 生产环境  
cp providers.prod.toml providers.toml
```

### 配置模板

你可以创建配置模板供他人使用：

```toml
# providers.template.toml
[providers.your_service]
name = "your_service"
display_name = "Your AI Service"
default_url = "https://your-api.com/v1/chat"
requires_api_key = true
default_model = "your-default-model"
supported_models = ["model1", "model2"]
api_format = "openai"
description = "Your custom AI service"
env_prefix = "AI_COMMIT_YOUR_SERVICE"
```

## 快速参考

### 环境变量快速设置

```bash
# === Ollama（免费本地） ===
export AI_COMMIT_PROVIDER=ollama
export AI_COMMIT_MODEL=mistral
# 可选：自定义服务地址
export AI_COMMIT_OLLAMA_URL="http://localhost:11434/api/generate"

# === Deepseek（性价比最高） ===
export AI_COMMIT_PROVIDER=deepseek
export AI_COMMIT_MODEL=deepseek-coder
export AI_COMMIT_DEEPSEEK_API_KEY="sk-xxxxxxxxxxxxxxxx"

# === SiliconFlow（多模型选择） ===
export AI_COMMIT_PROVIDER=siliconflow
export AI_COMMIT_MODEL="qwen/Qwen2-72B-Instruct"
export AI_COMMIT_SILICONFLOW_API_KEY="sk-xxxxxxxxxxxxxxxx"

# === Kimi（超长上下文） ===
export AI_COMMIT_PROVIDER=kimi
export AI_COMMIT_MODEL=moonshot-v1-32k
export AI_COMMIT_KIMI_API_KEY="sk-xxxxxxxxxxxxxxxx"
```

### 命令行快速使用

```bash
# 不同提供商快速切换
ai-commit --provider ollama --model mistral
ai-commit --provider deepseek --model deepseek-coder
ai-commit --provider siliconflow --model "qwen/Qwen2-72B-Instruct"
ai-commit --provider kimi --model moonshot-v1-32k

# 组合参数使用
ai-commit --provider deepseek --push --yes
ai-commit --provider kimi --model moonshot-v1-128k --new-tag v1.0.0
```

### 推荐配置模板

#### .env 文件模板

```env
# ~/.ai-commit/.env 或项目根目录 .env

# === 基础配置 ===
AI_COMMIT_PROVIDER=deepseek
AI_COMMIT_MODEL=deepseek-coder
AI_COMMIT_DEBUG=false

# === 提供商 API Keys ===
AI_COMMIT_DEEPSEEK_API_KEY=sk-your-deepseek-key
AI_COMMIT_SILICONFLOW_API_KEY=sk-your-siliconflow-key
AI_COMMIT_KIMI_API_KEY=sk-your-kimi-key

# === 自定义 URLs（可选）===
# AI_COMMIT_OLLAMA_URL=http://localhost:11434/api/generate
# AI_COMMIT_DEEPSEEK_URL=https://api.deepseek.com/v1/chat/completions
```

### 故障排除检查清单

1. **提供商不可用**
   ```bash
   # 检查可用提供商
   ai-commit --help | grep -A5 "provider"
   ```

2. **API Key 错误**
   ```bash
   # 检查环境变量
   env | grep AI_COMMIT
   ```

3. **模型不支持**
   ```bash
   # 使用默认模型测试
   ai-commit --provider your-provider
   ```

4. **网络连接问题**
   ```bash
   # 启用调试模式
   export AI_COMMIT_DEBUG=true
   ai-commit --provider your-provider
   ```

### 联系信息

- **项目地址**：[ai-commit GitHub](https://github.com/your-org/ai-commit)
- **问题反馈**：创建 GitHub Issue
- **文档更新**：欢迎提交 Pull Request

---

通过这个配置驱动的架构，添加新的 AI 提供商变得前所未有的简单！只需要修改配置文件，无需任何代码更改。