# Changelog

All notable changes to this project will be documented in this file.

## [unreleased]

### 📚 Documentation

- *(CHANGELOG.md)* 更新变更日志内容

### ⚙️ Miscellaneous Tasks

- *(Makefile)* 在构建目标中添加 changelog 依赖

## [0.1.1] - 2025-05-08

### 🚀 Features

- *(prompt)* 优化提示词文件读取逻辑
- *(main)* 添加调试信息打印

### 🐛 Bug Fixes

- *(prompt)* 移除调试用的打印语句
- *(args)* 修正 new-tag 参数的行为
- *(main)* 修正新标签创建逻辑的参数判断

### 🚜 Refactor

- *(prompt)* 优化提示词文件加载逻辑
- *(main)* 移除调试用的打印语句

### 📚 Documentation

- 更新文档以支持指定版本号创建新 tag
- *(commit-prompt)* 添加 Conventional Commits 中文生成提示词模板

### ⚙️ Miscellaneous Tasks

- *(Makefile)* 添加构建和安装命令
- *(Makefile)* 添加 run 目标并设为 build 前置依赖

## [0.1.0] - 2025-05-08

### 🚀 Features

- *(args)* 支持指定版本号创建新 tag

## [0.0.54] - 2025-05-08

### 📚 Documentation

- *(CI文档)* 更新CI构建与发布说明文档

## [0.0.53] - 2025-05-08

### 🚀 Features

- *(release.yml)* 增加对 musl 平台的构建支持并优化发布流程

## [0.0.52] - 2025-05-08

### 🐛 Bug Fixes

- *(release.yml)* 修正工作流中输出路径的变量名错误

## [0.0.51] - 2025-05-08

### 🚜 Refactor

- *(workflow)* 优化 CI 构建流程和归档逻辑

## [0.0.50] - 2025-05-08

### ⚙️ Miscellaneous Tasks

- *(workflow)* 优化发布流程步骤顺序

## [0.0.49] - 2025-05-08

### ⚙️ Miscellaneous Tasks

- *(release)* 完善发布流程中的目录和文档处理

## [0.0.48] - 2025-05-08

### ⚙️ Miscellaneous Tasks

- *(release)* 在发布流程中添加文件存在性检查

## [0.0.47] - 2025-05-08

### ⚙️ Miscellaneous Tasks

- *(ci)* 更新工作流中的目标平台条件判断
- *(ci)* 修正工作流中目标平台条件判断
- *(release)* 优化发布流程文档处理和校验逻辑

## [0.0.46] - 2025-05-08

### ⚙️ Miscellaneous Tasks

- *(release)* 在 Windows 目标中跳过测试步骤

## [0.0.45] - 2025-05-08

### ⚙️ Miscellaneous Tasks

- *(workflow)* 优化发布流程的归档和校验步骤

## [0.0.44] - 2025-05-08

### ⚙️ Miscellaneous Tasks

- *(release)* 移除生成校验和步骤

## [0.0.43] - 2025-05-08

### ⚙️ Miscellaneous Tasks

- *(release)* 修复压缩包路径生成问题

## [0.0.42] - 2025-05-08

### 🐛 Bug Fixes

- *(release)* 修复校验和生成时文件路径为空的问题

## [0.0.41] - 2025-05-08

### ⚙️ Miscellaneous Tasks

- *(release)* 优化发布流程中的文件打包逻辑

## [0.0.40] - 2025-05-08

### ⚙️ Miscellaneous Tasks

- *(ci)* 增加调试信息输出

## [0.0.39] - 2025-05-08

### 🐛 Bug Fixes

- *(workflow)* 修正发布流程中的归档路径问题

## [0.0.38] - 2025-05-08

### ⚙️ Miscellaneous Tasks

- *(workflow)* 为 Linux musl 目标添加 musl-tools 依赖

## [0.0.37] - 2025-05-08

### 📚 Documentation

- *(git)* 重构 git 模块文档并补充推送策略说明

### ⚙️ Miscellaneous Tasks

- *(release)* 更新发布工作流，添加测试和格式检查步骤

## [0.0.36] - 2025-05-08

### 📚 Documentation

- *(安装与使用)* 更新文档并新增推送分支选项

## [0.0.35] - 2025-05-08

### 🚀 Features

- *(git)* 重构 git 模块代码结构
- *(git/tag)* 增强推送标签功能，支持多分支同步

### 📚 Documentation

- *(CI/CD)* 新增CI构建与发布说明文档

## [0.0.34] - 2025-05-08

### ⚙️ Miscellaneous Tasks

- *(release)* 更新 GitHub Actions 发布工作流

## [0.0.33] - 2025-05-08

### ⚙️ Miscellaneous Tasks

- *(ci)* 优化 release 工作流配置

## [0.0.32] - 2025-05-08

### ⚙️ Miscellaneous Tasks

- *(release)* 移除 OpenSSL 相关依赖和构建步骤

## [0.0.31] - 2025-05-07

### ⚙️ Miscellaneous Tasks

- *(release)* 优化 CI 配置并添加 musl OpenSSL 支持

## [0.0.30] - 2025-05-07

### ⚙️ Miscellaneous Tasks

- *(ci)* 添加 musl 构建的静态链接配置

## [0.0.29] - 2025-05-07

### ⚙️ Miscellaneous Tasks

- *(workflow)* 移除 OpenSSL 相关配置注释

## [0.0.28] - 2025-05-07

### ⚙️ Miscellaneous Tasks

- *(ci)* 更新缓存键名以禁用缓存

## [0.0.26] - 2025-05-07

### ⚙️ Miscellaneous Tasks

- *(Cargo.toml)* 将Rust版本从2024降级至2021

## [0.0.25] - 2025-05-07

### ⚙️ Miscellaneous Tasks

- *(release)* 使用 cross 简化 musl 构建流程

## [0.0.24] - 2025-05-07

### ⚙️ Miscellaneous Tasks

- *(workflows)* 改进 mman.h 文件查找逻辑

## [0.0.23] - 2025-05-07

### ⚙️ Miscellaneous Tasks

- *(ci)* 为musl构建添加linux/mman.h符号链接

## [0.0.22] - 2025-05-07

### ⚙️ Miscellaneous Tasks

- *(ci)* 添加 linux-libc-dev 到 CI 依赖

## [0.0.21] - 2025-05-07

### ⚙️ Miscellaneous Tasks

- *(release)* 优化CI构建流程和归档路径

## [0.0.20] - 2025-05-07

### ⚙️ Miscellaneous Tasks

- *(release)* 更新 OpenSSL 版本并简化打包逻辑

## [0.0.19] - 2025-05-07

### ⚙️ Miscellaneous Tasks

- *(workflow)* 更新 GitHub Actions 依赖版本

## [0.0.18] - 2025-05-07

### ⚙️ Miscellaneous Tasks

- *(workflow)* 优化 release.yml 工作流配置

## [0.0.17] - 2025-05-07

### ⚙️ Miscellaneous Tasks

- *(release)* 优化 CI 构建流程和 OpenSSL 缓存策略

## [0.0.16] - 2025-05-07

### ⚙️ Miscellaneous Tasks

- *(ci)* 为 musl 构建添加 OpenSSL 支持

## [0.0.13] - 2025-05-07

### ⚙️ Miscellaneous Tasks

- *(ci)* 简化 Linux musl 构建依赖

## [0.0.12] - 2025-05-07

### ⚙️ Miscellaneous Tasks

- *(release)* 添加 Linux 交叉编译的 OpenSSL 依赖

## [0.0.10] - 2025-05-07

### ⚙️ Miscellaneous Tasks

- *(ci)* 优化跨平台构建工作流配置

## [0.0.9] - 2025-05-07

### ⚙️ Miscellaneous Tasks

- *(release)* 添加跨平台编译依赖并修复打包命令

## [0.0.8] - 2025-05-07

### 🐛 Bug Fixes

- *(git)* 修复重复标签创建问题

### ⚙️ Miscellaneous Tasks

- *(release)* 优化 GitHub Actions 发布流程

## [0.0.6] - 2025-05-07

### ⚙️ Miscellaneous Tasks

- *(release)* 优化CI流程并更新文档

## [0.0.4] - 2025-05-07

### 📚 Documentation

- *(docs)* 更新文档以包含新增的 tag 功能

## [0.0.3] - 2025-05-07

### 🚀 Features

- *(args)* 新增显示最新 tag 信息的命令行参数

### ⚙️ Miscellaneous Tasks

- *(release)* 优化发布流程和文档说明

## [0.0.2] - 2025-05-07

### 📚 Documentation

- *(安装与使用)* 新增项目安装与使用文档
- *(安装与使用)* 更新环境配置文件位置说明
- *(readme)* 更新文档结构和安装说明

### ⚙️ Miscellaneous Tasks

- *(ci)* 添加 ARM 架构的 macOS 构建支持
- *(config)* 重命名环境变量示例文件

## [0.0.1] - 2025-05-07

### 🚀 Features

- *(init)* 初始化项目结构和基础配置
- *(args)* 添加自动 push 选项
- *(core)* 重构代码为模块化结构

### 🚜 Refactor

- *(ai)* 优化代码结构并添加允许未使用字段标记

### 📚 Documentation

- *(readme)* 完善项目文档结构

### ⚙️ Miscellaneous Tasks

- *(ci)* 添加 GitHub Actions 发布二进制文件工作流

<!-- generated by git-cliff -->
