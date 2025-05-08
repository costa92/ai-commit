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
	

# 运行
.PHONY: run
run:
	@echo "Running..."
	@cargo run 


# 构建项目
.PHONY: build
build: run
	@echo "Building project..."
	@cargo build --release
	@echo "Build completed successfully"

# 安装到系统
.PHONY: install
install: build
	@echo "Installing to ~/.cargo/bin..."
	@if [ ! -d ~/.cargo/bin ]; then \
		mkdir -p ~/.cargo/bin; \
	fi
	@cp target/release/ai-commit ~/.cargo/bin/
	@echo "Installation completed successfully"


