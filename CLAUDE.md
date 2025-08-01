# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Development Commands

**Building and Running:**
```bash
# Build the project
make build

# Build in release mode
cargo build --release

# Run the tool directly
make run
# or
cargo run -- [args]

# Install to ~/.cargo/bin/
make install
```

**Testing and Linting:**
```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Check code formatting
cargo fmt --check

# Run clippy for linting
cargo clippy -- -D warnings
```

**Changelog and Git Utilities:**
```bash
# Generate changelog (requires git-cliff)
make changelog

# Get latest tag changelog
make latest-tag

# Generate changelog for specific tag
make tag-changelog tag=v1.0.0
```

## Architecture Overview

**ai-commit** is a Rust CLI tool that generates conventional commit messages using AI models. The application follows a modular architecture:

### Core Components

**1. Configuration System (`src/config/`)**
- Supports multiple configuration sources: CLI args → env vars → .env files
- Handles AI provider configurations (Ollama, Deepseek, SiliconFlow)
- Configuration priority: CLI args > environment variables > .env files
- Environment files loaded from: `~/.ai-commit/.env` or local `.env`

**2. AI Integration (`src/ai/`)**
- Supports multiple AI providers: Ollama (local), Deepseek, SiliconFlow
- Streaming response processing for real-time output
- Uses `commit-prompt.txt` for Conventional Commits prompt template
- Regex-based cleanup to extract clean commit messages from AI responses

**3. Git Operations (`src/git/`)**
- `commit.rs`: Standard git operations (add, commit, push, diff)
- `tag.rs`: Advanced tag management with semantic versioning
- Automatic tag version resolution and conflict avoidance
- Support for both commit and tag workflows

**4. CLI Interface (`src/cli/`)**
- Built with `clap` for argument parsing
- Supports both short and long argument forms
- Comprehensive tag creation and push options

**5. Internationalization (`src/internationalization.rs`)**
- Multi-language support (Chinese Simplified/Traditional, English)
- Centralized message management system

### Configuration Sources

The tool loads configuration in this priority order:
1. CLI arguments (highest priority)
2. Environment variables (prefixed with `AI_COMMIT_`)
3. `.env` files (user home: `~/.ai-commit/.env`, then local `.env`)
4. Default values (lowest priority)

### AI Provider Setup

**Ollama (default):**
- Requires local Ollama installation
- Default model: `mistral`
- Default URL: `http://localhost:11434/api/generate`

**Deepseek:**
- Requires API key: `AI_COMMIT_DEEPSEEK_API_KEY`
- Default URL: `https://api.deepseek.com/v1/chat/completions`

**SiliconFlow:**
- Requires API key: `AI_COMMIT_SILICONFLOW_API_KEY`
- Default URL: `https://api.siliconflow.cn/v1/chat/completions`

### Tag Management Logic

The tag system supports intelligent version resolution:
- When no base version specified: increments patch version from latest tag
- When major.minor specified: starts with .0 patch, finds next available
- When full version specified: uses exact version if available, otherwise increments
- Automatic conflict resolution by incrementing patch version

### Key Files

- `commit-prompt.txt`: Conventional Commits prompt template for AI
- `example.env`: Configuration template showing all available options
- `Makefile`: Build automation and development commands
- `cliff.toml`: Configuration for git-cliff changelog generation

### Development Notes

- **Testing Strategy**: Comprehensive test suite with 99+ tests covering:
  - Unit tests for all modules (inline with `#[cfg(test)]`)
  - Integration tests in `tests/integration_tests.rs`
  - Performance optimization validation
  - Concurrent access and thread safety tests
- **Performance Optimizations**: 
  - HTTP client singleton with connection reuse (50-80% faster connections)
  - Async/await conversion for Git operations
  - Stream processing with pre-allocated buffers
  - Caching systems for Git commands and prompt templates
  - Environment loading optimization with singleton pattern
- The application uses `anyhow` for error handling throughout
- All git operations converted to async using `tokio::process::Command`
- Streaming AI responses provide real-time feedback during generation
- Configuration validation ensures required API keys are present for cloud providers
- Memory allocation optimizations reduce heap usage by 30-50%

### Prompt Template Optimization

The `commit-prompt.txt` template has been optimized for strict Conventional Commits compliance:

**Key Requirements:**
- AI must output exactly: `<type>(<scope>): <subject>\n\n<body>`
- No markdown formatting, explanations, or additional text
- Subject must be Chinese and under 50 characters
- Types limited to: feat, fix, docs, style, refactor, test, chore

**Current Template Structure:**
```
只能输出以下格式，严禁任何其他内容：

<type>(<scope>): <subject>

要求：
1. 第一行必须是上述格式
2. subject必须是中文，不超过50字
3. 禁止输出英文、列表、解释、markdown
4. 禁止输出"The changes"、"1."、"*"等
5. 多文件修改时用高度概括的中文描述

示例：
feat(core): 重构核心模块
fix(api): 修复接口错误
refactor(all): 优化代码结构

以下是 git diff：
{{git_diff}}
```

**优化重点：**
- **极简模板**：移除body选项，强制单行输出，减少AI偏离的可能性
- **具体禁用模式**：明确禁止"The changes"、列表格式等常见英文模式
- **强化验证**：检测并拒绝包含英文描述、列表符号、解释性文本的响应
- **专门处理多文件场景**：要求高度概括而不列举文件名

### Test Coverage Summary

**Unit Tests (89 tests):**
- AI Module: 17 tests (HTTP client, request/response handling, error scenarios)
- Git Operations: 15 tests (async operations, command validation, error handling)
- Configuration: 18 tests (environment loading, validation, priority handling)
- Internationalization: 14 tests (language switching, message retrieval, concurrent access)
- CLI Arguments: 15 tests (argument parsing, validation, edge cases)
- Git Tag Management: 10 tests (version parsing, caching, thread safety)

**Integration Tests (10 tests):**
- Configuration system workflows
- CLI parsing and configuration integration
- Internationalization system integration
- Error handling across modules
- Performance optimization validation
- Concurrent access testing
- Full system integration scenarios

**Test Execution:**
```bash
# Run all tests
cargo test

# Run specific test module
cargo test ai::tests

# Run integration tests only
cargo test --test integration_tests

# Run tests with output
cargo test -- --nocapture
```