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
make test
# or
cargo test

# Run tests with verbose output
make test-verbose
# or
cargo test -- --nocapture

# Code quality checks
make check           # Run clippy + formatting check
make clippy          # Run clippy linting only
make fmt-check       # Check code formatting only
# or
cargo clippy -- -D warnings
cargo fmt --check

# Code formatting and fixes
make fmt             # Format code automatically
make fix             # Format code + fix clippy issues
# or
cargo fmt
cargo clippy --fix --allow-dirty --allow-staged

# Complete quality assurance
make qa              # Run tests + checks (full QA pipeline)
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
- `worktree.rs`: Git worktree management for parallel development
- Automatic tag version resolution and conflict avoidance
- Support for both commit and tag workflows

**4. CLI Interface (`src/cli/`)**
- Built with `clap` for argument parsing
- Supports both short and long argument forms
- Comprehensive tag creation and push options
- Full worktree management commands

**5. Internationalization (`src/internationalization.rs`)**
- Multi-language support (Chinese Simplified/Traditional, English)
- Centralized message management system

**6. Code Review System (`src/languages/`)**
- Multi-language code analysis engine supporting Rust, Go, JavaScript, TypeScript
- Intelligent feature detection using language-specific regex patterns
- Automated risk assessment and testing suggestions
- Configurable output formats (Markdown, JSON, Text)
- Automatic file output to `code-review/` directory with timestamp naming

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

### Debug Mode Configuration

The tool supports a debug mode that controls output verbosity through the `AI_COMMIT_DEBUG` environment variable:

**Debug Mode Off (Default):**
- `AI_COMMIT_DEBUG=false` or unset
- Only outputs final results
- Suppresses process information and timing details
- Ideal for production use and automation scripts

**Debug Mode On:**
- `AI_COMMIT_DEBUG=true` or `AI_COMMIT_DEBUG=1` 
- Shows detailed operation process
- Includes AI generation timing, large change detection, tag creation messages
- Case-insensitive: accepts `TRUE`, `True`, `1`
- Useful for development, debugging, and understanding tool behavior

**Debug Output Examples:**
- AI generation timing: "AI 生成 commit message 耗时: 1.23s"
- Large change detection: "检测到大型变更 (6个文件, 15967字符)，正在生成摘要..."  
- Tag operations: "Created new tag: v1.0.1", "Pushed tag v1.0.1 to remote"
- Empty changes: "No staged changes."

### Git Worktree Development Workflow

The tool supports Git worktree functionality for parallel development across multiple branches:

**Worktree Management Commands:**
- `--worktree-create BRANCH`: Create new worktree for specified branch
- `--worktree-switch NAME`: Switch current working directory to specified worktree  
- `--worktree-list`: List all available worktrees with branch info
- `--worktree-verbose, -v`: Enable verbose mode for worktree list (equivalent to `git worktree list -v`)
- `--worktree-porcelain`: Enable machine-readable output for worktree list (equivalent to `git worktree list --porcelain`)
- `--worktree-z, -z`: Use NUL character to terminate records (equivalent to `git worktree list -z`)
- `--worktree-expire TIME`: Show prunable annotation for old worktrees (equivalent to `git worktree list --expire`)
- `--worktree-remove NAME`: Remove specified worktree and cleanup references
- `--worktree-path PATH`: Specify custom path for worktree creation
- `--worktree-clear`: Clear all other worktrees except current one

**Worktree Creation Logic:**
- Default path: `../worktree-{branch-name}` (replaces `/` with `-`)
- Custom paths supported via `--worktree-path`
- Automatically tries existing branch first, then creates new branch if needed
- Smart branch name sanitization for filesystem compatibility

**Development Workflow Benefits:**
- Work on multiple features simultaneously without branch switching
- Isolated working directories for each feature/bugfix
- Seamless ai-commit integration within any worktree
- Automatic working directory switching and path resolution

**Safety Features:**
- Worktree conflict detection and resolution
- Automatic pruning of invalid worktree references
- Path validation and error handling
- Branch existence checking before worktree creation
- Bulk cleanup operations with detailed feedback
- Current worktree protection (never removes active worktree)

### Code Review System

The tool includes a comprehensive code review system that analyzes Git changes and provides detailed insights:

**Code Review Commands:**
- `--code-review`: Perform code review analysis on staged Git changes
- `--review-format FORMAT`: Output format (markdown, json, text) [default: markdown]
- `--review-output FILE`: Custom output file path (optional)
- `--review-files FILES`: Specify comma-separated file paths to review
- `--show-languages`: Display detected programming languages statistics only

**🤖 AI-Enhanced Code Review Commands:**
- `--ai-review`: Enable AI-powered code review (combines static analysis + AI intelligence)
- `--ai-review-type TYPE`: AI review type (general, security, performance, architecture) [default: general]
- `--ai-review-detail LEVEL`: AI review detail level (brief, detailed, comprehensive) [default: detailed]
- `--ai-language-specific`: Enable language-specific AI review rules [default: true]
- `--ai-include-static`: Include static analysis results in AI review [default: true]
- `--ai-review-lang LANG`: AI review output language (zh, en) [default: zh]

**Supported Languages:**
- **Rust**: Functions, structs, enums, traits, impl blocks, modules, use statements
- **Go**: Packages, functions, methods, structs, interfaces, imports, constants
- **JavaScript**: Functions, classes, imports/requires, exports, arrow functions
- **TypeScript**: Interfaces, classes, functions, type aliases, enums, imports/exports
- **Generic**: Fallback analyzer for unsupported languages

**Language-Specific Features:**
- **Regex-based pattern matching** for accurate code feature detection
- **Scope suggestions** based on file paths and project structure
- **Change pattern analysis** to identify common modification types
- **Risk assessment** focused on API compatibility and breaking changes
- **Testing suggestions** tailored to each programming language

**AI Review Features:**
- **Multi-dimensional Quality Scoring**: Overall quality, security, performance, maintainability (1-10 scale)
- **Language-specific Rules**: Professional review rules tailored for Rust, Go, JavaScript, TypeScript
- **Security Vulnerability Detection**: Identify potential security risks and best practice violations
- **Performance Optimization Suggestions**: Specific performance improvement recommendations
- **Learning Resource Recommendations**: Relevant learning materials and documentation links
- **Static Analysis Integration**: Incorporates static analysis tool results for comprehensive quality assessment

**Output Features:**
- **Default Output**: Saves to `code-review/review_YYYYMMDD_HHMMSS.{ext}` automatically
- **Timestamp naming** prevents file conflicts
- **Multiple formats**: Markdown (default), JSON for automation, Text for simple output
- **Smart content optimization**: Automatically detects long reports (>10,000 chars) and generates optimized versions
- **Comprehensive reporting**: Summary statistics, change patterns, risk assessment, test suggestions
- **File-level analysis** with detected features and line numbers
- **AI Review Results** (when `--ai-review` enabled):
  - Overall quality scores and dimensional ratings
  - Critical issue identification and prioritization
  - Specific improvement recommendations and best practices
  - Learning resources and reference documentation links

**Code Review Report Structure:**
```markdown
# 代码审查报告

## 📊 摘要统计
- 总文件数、检测特征数、编程语言统计

## 🔍 变更模式分析  
- 代码变更类型识别和影响分析

## ⚠️ 风险评估
- API兼容性、依赖变更、架构影响

## 🧪 测试建议
- 语言特定的测试策略和最佳实践

## 📁 详细文件分析
- 每个文件的特征检测和作用域建议
```

**Usage Examples:**
```bash
# Basic code review (outputs to code-review/ directory)
ai-commit --code-review

# JSON format for CI/CD integration
ai-commit --code-review --review-format json

# Custom output location
ai-commit --code-review --review-output reports/review.md

# Review specific files
ai-commit --code-review --review-files "src/main.rs,lib/utils.go"

# Language statistics only
ai-commit --show-languages
```

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

- **Testing Strategy**: Comprehensive test suite with 230+ tests covering:
  - Unit tests for all modules (inline with `#[cfg(test)]`)
  - Integration tests in `tests/integration_tests.rs`
  - Performance optimization validation
  - Concurrent access and thread safety tests
  - **Smart commit message optimization**: Automatic length detection and secondary generation
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
- **Intelligent commit message optimization**: Automatically detects messages over 100 characters and generates optimized versions

### Prompt Template Optimization

The `commit-prompt.txt` template has been optimized for strict Conventional Commits compliance:

**Key Requirements:**
- AI must output exactly: `<type>(<scope>): <subject>\n\n<body>`
- No markdown formatting, explanations, or additional text
- Subject must be Chinese and under 50 characters
- Types limited to: feat, fix, docs, style, refactor, test, chore

**Current Template Structure:**
```
输出格式：<type>(<scope>): <subject>

type: feat|fix|docs|style|refactor|test|chore
subject: 中文，不超过50字

错误示例（禁止）：
"These are good changes..."
"Here's a breakdown:"
"**Overall Assessment:**"
任何英文分析或解释

正确示例：
feat(api): 添加用户认证功能
fix(ui): 修复按钮显示问题  
refactor(core): 重构数据处理逻辑

git diff:
{{git_diff}}
```

**优化重点：**
- **极简命令式模板**：直接指定输出格式，无多余解释
- **明确反面示例**：直接展示禁止的英文分析模式
- **全面验证逻辑**：检测20+种英文描述模式，包括：
  - `"These are"`, `"Here's a"`, `"The changes"`
  - `"Overall Assessment"`, `"breakdown"`, `"suggestions"`
  - `"**"`, `"good changes"`, `"clean"`, `"helpful"`
  - `"address"`, `"improve"`, `"1."`, `"*"`
- **正面格式验证**：确保输出以有效type开头，包含冒号，长度合理
- **专门处理大文件场景**：强制概括而非详细分析

### Test Coverage Summary

**Unit Tests (239+ tests):**
- AI Module: 23+ tests (HTTP client, request/response handling, error scenarios, **smart optimization**)
- Git Operations: 15 tests (async operations, command validation, error handling)
- Configuration: 18 tests (environment loading, validation, priority handling)
- Internationalization: 14 tests (language switching, message retrieval, concurrent access)
- CLI Arguments: 15 tests (argument parsing, validation, edge cases)
- Git Tag Management: 10 tests (version parsing, caching, thread safety)
- Language Analysis: 73+ tests (feature detection, Default implementations, scope analysis)
- Code Review: 15+ tests (report generation, file analysis, language detection)
- **Smart Commit Optimization**: 6+ tests (fallback message generation, optimization prompts)

**New Test Categories (Added in v0.2.0):**
- **Default Implementation Tests**: Verify all language analyzers' Default trait implementations
- **analyze_file_changes Tests**: Comprehensive testing of file change analysis logic
- **Language-Specific Pattern Tests**: Test regex patterns for Rust, Go, JS, TS feature detection
- **Code Review Integration Tests**: End-to-end testing of review report generation

**Integration Tests (12 tests):**
- Configuration system workflows
- CLI parsing and configuration integration
- Internationalization system integration
- Error handling across modules
- Performance optimization validation (2 tests for HTTP client and env loading)
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