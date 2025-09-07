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
- `core.rs`: Common Git operations and utilities (branch management, status checks)
- `commit.rs`: Standard git operations (add, commit, push, diff)
- `tag.rs`: Advanced tag management with semantic versioning
- `worktree.rs`: Git worktree management for parallel development
- `flow.rs`: Git Flow workflow support (feature, hotfix, release branches)
- `history.rs`: Git history viewing and analysis with filtering capabilities
- `edit.rs`: Commit editing and modification (amend, rebase, reword, undo)
- Automatic tag version resolution and conflict avoidance
- Support for both commit and tag workflows

**4. CLI Interface (`src/cli/`)**
- Built with `clap` for argument parsing
- Supports both short and long argument forms
- Comprehensive tag creation and push options
- Full worktree management commands
- Git Flow workflow commands
- History and log viewing options
- Commit editing and modification commands

**5. Command Routing (`src/commands/`)**
- Modular command structure with dedicated handlers
- `tag.rs`: Tag management command routing (list, delete, info, compare)
- `flow.rs`: Git Flow command handlers (feature, hotfix, release workflows)
- `history.rs`: History viewing command processing with advanced filtering
- `edit.rs`: Commit editing command handlers (amend, rebase, reword, undo)
- `mod.rs`: Central command routing system

**6. Internationalization (`src/internationalization.rs`)**
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

### Advanced Git Features

#### Git Worktree Development Workflow

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

#### Git Flow Workflow Support

The tool implements comprehensive Git Flow workflows for structured development:

**Git Flow Initialization:**
- `--flow-init`: Initialize Git Flow in repository with standard branch structure
- Sets up main/develop branches and configures branch prefixes

**Feature Workflow:**
- `--flow-feature-start NAME`: Start new feature branch from develop
- `--flow-feature-finish NAME`: Merge feature branch into develop and clean up
- `--flow-feature-list`: List all active feature branches

**Hotfix Workflow:**
- `--flow-hotfix-start NAME`: Start hotfix branch from main for urgent fixes
- `--flow-hotfix-finish NAME`: Merge hotfix into both main and develop
- `--flow-hotfix-list`: List all active hotfix branches

**Release Workflow:**
- `--flow-release-start NAME`: Start release branch from develop
- `--flow-release-finish NAME`: Merge release into main and develop, create tag
- `--flow-release-list`: List all active release branches

**Git Flow Benefits:**
- Structured branching model for team collaboration
- Automatic branch merging and cleanup
- Version tagging for releases
- Separation of features, fixes, and releases

#### Tag Management System

Enhanced tag management with semantic versioning support:

**Tag Operations:**
- `--tag-list`: List all repository tags with creation dates
- `--tag-delete TAG`: Delete specified tag locally and remotely
- `--tag-info TAG`: Show detailed tag information (author, date, message)
- `--tag-compare TAG1 TAG2`: Compare changes between two tags

**Automatic Version Resolution:**
- Smart version incrementation from latest tags
- Conflict resolution for duplicate versions
- Support for semantic versioning patterns

#### History and Log Analysis

Comprehensive git history analysis with advanced filtering:

**History Commands:**
- `--history`: Show formatted commit history with colors and graphs
- `--log-author AUTHOR`: Filter history by specific author
- `--log-since DATE`: Show commits since specified date
- `--log-until DATE`: Show commits until specified date
- `--log-graph`: Display branch graph visualization
- `--log-limit NUMBER`: Limit number of commits shown
- `--log-file PATH`: Show history for specific file or path

**Advanced Features:**
- `--log-stats`: Display file change statistics
- `--log-contributors`: Show contributor statistics with commit counts
- `--log-search TERM`: Search commit messages for specific terms
- `--log-branches`: Show branch graph with all branches

#### Commit Editing and Modification

Powerful commit editing capabilities for history management:

**Commit Editing Commands:**
- `--amend [MESSAGE]`: Modify the last commit message or add changes
- `--edit-commit HASH`: Edit specific commit interactively
- `--rebase-edit BASE`: Interactive rebase from base commit
- `--reword-commit HASH MESSAGE`: Change commit message for specific commit
- `--undo-commit`: Undo last commit (soft reset, keeps changes staged)

**Advanced Editing:**
- `--squash-commits FROM TO`: Squash multiple commits into one
- `--show-editable [LIMIT]`: Display list of recent editable commits
- Interactive rebase support with automatic editor setup
- Rebase status checking and conflict resolution

**Safety Features:**
- Commit existence validation before operations
- Automatic backup references for destructive operations
- Clear instructions for multi-step operations
- Status checking for ongoing rebase operations

### Legacy Tag Management Logic

The original tag system supports intelligent version resolution:
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

- **Testing Strategy**: Comprehensive test suite with 150+ tests covering:
  - Unit tests for all modules (inline with `#[cfg(test)]`)
  - Integration tests in `tests/integration_tests.rs`
  - New feature test coverage (Git Flow, history analysis, commit editing)
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

**Unit Tests (150+ tests):**
- AI Module: 17 tests (HTTP client, request/response handling, error scenarios)
- Git Core Operations: 12 tests (async operations, branch management, status checks)
- Git Commit Operations: 15 tests (commit, push, diff, async command validation)
- Git Tag Management: 18 tests (version parsing, caching, thread safety, comparison)
- Git Flow Workflow: 16 tests (feature, hotfix, release workflows, branch operations)
- Git History Analysis: 14 tests (filtering, formatting, contributor stats, file tracking)
- Git Edit Operations: 18 tests (amend, rebase, reword, undo, interactive editing)
- Configuration: 18 tests (environment loading, validation, priority handling)
- Internationalization: 14 tests (language switching, message retrieval, concurrent access)
- CLI Arguments: 15 tests (argument parsing, validation, edge cases, new command options)
- Command Routing: 8 tests (dispatch logic, error handling, command validation)

**Integration Tests (12 tests):**
- Configuration system workflows
- CLI parsing and configuration integration
- Internationalization system integration
- Command routing integration
- Error handling across modules
- Performance optimization validation
- Concurrent access testing
- Full system integration scenarios with new features

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

## Recent Feature Updates & Requirements

**New Requirements Support:**
- The application continuously evolves with new feature requests
- When implementing new features, ensure backward compatibility with existing configuration
- All new CLI arguments must include both help documentation and comprehensive test coverage
- New AI providers should follow the existing pattern in `src/ai/mod.rs` and `src/config/mod.rs`
- Worktree functionality expansions should maintain safety features and path validation

**Feature Request Workflow:**
1. Analyze existing architecture patterns before implementing new features
2. Update CLI argument definitions in `src/cli/args.rs` with proper help text
3. Add configuration support if needed in `src/config/mod.rs`
4. Implement core functionality following async patterns
5. Add comprehensive test coverage (unit + integration)
6. Update CLAUDE.md if architectural changes are made

**Help System Maintenance:**
- Ensure all CLI parameters are properly documented in help output
- Help text should list all supported options (e.g., all AI providers: ollama, deepseek, siliconflow)
- Parameter descriptions should be concise but complete
- Validate help output matches actual functionality

**Quality Assurance for New Features:**
- Run `make qa` after implementing new features
- Ensure all tests pass and code formatting is correct
- Test both positive and negative scenarios
- Verify configuration priority system works with new options
- Test integration with existing worktree and commit workflows

# important-instruction-reminders
Do what has been asked; nothing more, nothing less.
NEVER create files unless they're absolutely necessary for achieving your goal.
ALWAYS prefer editing an existing file to creating a new one.
NEVER proactively create documentation files (*.md) or README files. Only create documentation files if explicitly requested by the User.