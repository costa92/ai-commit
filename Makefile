# 安装 git-cliff 工具
.PHONY: install-git-cliff
install-git-cliff:
	@echo "Installing git-cliff..."
	@if ! command -v git-cliff >/dev/null 2>&1; then \
		cargo install git-cliff; \
	else \
		echo "git-cliff is already installed"; \
	fi


# 初始化 git-cliff 配置
.PHONY: init-git-cliff
init-git-cliff:
	@echo "Initializing git-cliff..."
	@git-cliff --init > cliff.toml


# 生成 changelog
.PHONY: changelog
changelog:
	@echo "Generating changelog..."
	@git-cliff --config .git-cliff.toml --output CHANGELOG.md



# 最新tag 生成 changelog
.PHONY: latest-tag
latest-tag:
	@echo "Getting latest tag..."
	@git-cliff --config .git-cliff.toml --latest


# 指定tag 生成 changelog
.PHONY: tag-changelog
tag-changelog:
	@echo "Getting tag changelog..."
	@git-cliff --config .git-cliff.toml --tag $(tag)


# 运行默认程序 (ai-commit)
# 用法: make run ARGS="--help"
.PHONY: run
run:
	@echo "Running ai-commit..."
	@cargo run -- $(ARGS)

# 运行简化版本
# 用法: make run-aic ARGS="--help"
.PHONY: run-aic
run-aic:
	@echo "Running aic..."
	@cargo run --bin aic -- $(ARGS)


# 运行测试
.PHONY: test
test:
	@echo "Running tests..."
	@cargo test
	@echo "All tests completed successfully"


# 运行测试（详细输出）
.PHONY: test-verbose
test-verbose:
	@echo "Running tests with verbose output..."
	@cargo test -- --nocapture
	@echo "All tests completed successfully"


# 运行 clippy 检查
.PHONY: clippy
clippy:
	@echo "Running clippy linting..."
	@cargo clippy -- -D warnings
	@echo "Clippy check completed successfully"


# 代码格式化检查
.PHONY: fmt-check
fmt-check:
	@echo "Checking code formatting..."
	@cargo fmt --check
	@echo "Code formatting check completed successfully"


# 代码格式化（自动修复）
.PHONY: fmt
fmt:
	@echo "Formatting code..."
	@cargo fmt
	@echo "Code formatting completed successfully"


# 代码检测（clippy + 格式化检查）
.PHONY: check
check: clippy fmt-check
	@echo "All code checks completed successfully"


# 代码修复（格式化 + clippy 可修复的问题）
.PHONY: fix
fix:
	@echo "Running code fixes..."
	@cargo fmt
	@cargo clippy --fix --allow-dirty --allow-staged
	@echo "Code fixes completed successfully"


# 完整代码质量检查（测试 + 检查）
.PHONY: qa
qa: test check
	@echo "Quality assurance checks completed successfully"


# 构建项目 (跳过测试用于快速安装)
.PHONY: build-only
build-only:
	@echo "Building project..."
	@cargo build --release
	@echo "Build completed successfully"

# 构建项目
.PHONY: build
build: test
	@echo "Building project..."
	@cargo build --release
	@echo "Build completed successfully"

# 安装到系统 (跳过测试，快速安装)
.PHONY: install
install: build-only
	@echo "Installing to ~/.cargo/bin..."
	@if [ ! -d ~/.cargo/bin ]; then \
		mkdir -p ~/.cargo/bin; \
	fi
	@cp target/release/ai-commit ~/.cargo/bin/
	@cp target/release/aic ~/.cargo/bin/
	@echo "Installation completed successfully"
	@echo "You can now use both 'ai-commit' and 'aic' commands"

# 安装到系统 (包含完整测试)
.PHONY: install-with-test
install-with-test: build
	@echo "Installing to ~/.cargo/bin..."
	@if [ ! -d ~/.cargo/bin ]; then \
		mkdir -p ~/.cargo/bin; \
	fi
	@cp target/release/ai-commit ~/.cargo/bin/
	@cp target/release/aic ~/.cargo/bin/
	@echo "Installation completed successfully"
	@echo "You can now use both 'ai-commit' and 'aic' commands"

# 仅安装简称
.PHONY: install-alias
install-alias:
	@echo "Creating 'aic' alias..."
	@if [ -f ~/.cargo/bin/ai-commit ]; then \
		ln -sf ~/.cargo/bin/ai-commit ~/.cargo/bin/aic; \
		echo "Alias 'aic' created successfully"; \
	else \
		echo "Error: ai-commit not found. Run 'make install' first"; \
		exit 1; \
	fi

# 使用安装脚本安装
.PHONY: install-with-script
install-with-script:
	@echo "Running installation script..."
	@bash ./install.sh

