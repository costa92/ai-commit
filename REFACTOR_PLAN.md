# 文件布局优化重构方案

## 🎯 设计原则

1. **单一职责** - 每个文件只负责一个核心功能
2. **清晰分层** - 按功能域和抽象层次分组
3. **易于维护** - 减少文件间依赖，提高代码可读性
4. **功能独立** - 每个功能模块可独立测试和修改

## 📂 优化后的目录结构

```
src/
├── main.rs                     # 程序入口
├── lib.rs                      # 库入口
├── app/                        # 应用层
│   ├── mod.rs
│   ├── router.rs               # 命令路由器
│   └── dispatcher.rs           # 命令分发器
│
├── cli/                        # 命令行接口
│   ├── mod.rs
│   ├── args.rs                 # CLI参数定义
│   ├── parser.rs               # 参数解析器
│   └── validator.rs            # 参数验证器
│
├── config/                     # 配置管理
│   ├── mod.rs
│   ├── loader.rs               # 配置加载器
│   ├── validator.rs            # 配置验证器
│   ├── env.rs                  # 环境变量处理
│   └── file.rs                 # 配置文件处理
│
├── ai/                         # AI 相关功能
│   ├── mod.rs
│   ├── client.rs               # AI 客户端抽象
│   ├── ollama.rs               # Ollama 提供商
│   ├── deepseek.rs             # Deepseek 提供商
│   ├── siliconflow.rs          # SiliconFlow 提供商
│   ├── prompt.rs               # 提示词管理
│   └── diff_analyzer.rs        # 差异分析器
│
├── git/                        # Git 操作层
│   ├── mod.rs
│   │
│   ├── core/                   # 核心 Git 操作
│   │   ├── mod.rs
│   │   ├── status.rs           # Git 状态查询
│   │   ├── diff.rs             # Git 差异操作
│   │   ├── add.rs              # Git 添加操作
│   │   ├── commit.rs           # Git 提交操作
│   │   ├── push.rs             # Git 推送操作
│   │   └── reset.rs            # Git 重置操作
│   │
│   ├── branch/                 # 分支管理
│   │   ├── mod.rs
│   │   ├── create.rs           # 创建分支
│   │   ├── switch.rs           # 切换分支
│   │   ├── delete.rs           # 删除分支
│   │   └── merge.rs            # 合并分支
│   │
│   ├── tag/                    # 标签管理
│   │   ├── mod.rs
│   │   ├── create.rs           # 创建标签
│   │   ├── list.rs             # 列出标签
│   │   ├── delete.rs           # 删除标签
│   │   ├── info.rs             # 标签信息
│   │   ├── compare.rs          # 标签比较
│   │   └── version.rs          # 版本解析
│   │
│   ├── worktree/               # Worktree 管理
│   │   ├── mod.rs
│   │   ├── create.rs           # 创建 worktree
│   │   ├── list.rs             # 列出 worktree
│   │   ├── switch.rs           # 切换 worktree
│   │   ├── remove.rs           # 删除 worktree
│   │   ├── clear.rs            # 清理 worktree
│   │   └── info.rs             # Worktree 信息
│   │
│   ├── history/                # 历史记录
│   │   ├── mod.rs
│   │   ├── log.rs              # 基础日志
│   │   ├── graph.rs            # 图形化历史
│   │   ├── stats.rs            # 统计信息
│   │   ├── contributors.rs     # 贡献者统计
│   │   ├── search.rs           # 搜索提交
│   │   └── filter.rs           # 过滤器
│   │
│   ├── flow/                   # Git Flow 工作流
│   │   ├── mod.rs
│   │   ├── init.rs             # 初始化 Git Flow
│   │   ├── feature.rs          # Feature 分支管理
│   │   ├── hotfix.rs           # Hotfix 分支管理
│   │   ├── release.rs          # Release 分支管理
│   │   └── common.rs           # 通用工具
│   │
│   └── edit/                   # 提交编辑
│       ├── mod.rs
│       ├── amend.rs            # 修改提交
│       ├── rebase.rs           # 交互式 rebase
│       ├── reword.rs           # 重写消息
│       └── undo.rs             # 撤销操作
│
├── commands/                   # 命令处理层
│   ├── mod.rs
│   ├── commit.rs               # 提交命令
│   ├── tag.rs                  # 标签命令
│   ├── flow.rs                 # Flow 命令
│   ├── history.rs              # 历史命令
│   ├── worktree.rs             # Worktree 命令
│   ├── edit.rs                 # 编辑命令
│   │
│   └── enhanced/               # 增强功能命令
│       ├── mod.rs
│       ├── query.rs            # 查询命令
│       ├── watch.rs            # 监控命令
│       ├── diff_view.rs        # 差异查看命令
│       └── interactive.rs      # 交互式命令
│
├── query/                      # 查询系统
│   ├── mod.rs
│   ├── parser.rs               # 查询解析器
│   ├── executor.rs             # 查询执行器
│   ├── filter.rs               # 过滤器
│   └── storage.rs              # 保存的查询
│
├── monitor/                    # 监控系统
│   ├── mod.rs
│   ├── watcher.rs              # 文件监控器
│   ├── detector.rs             # 变化检测器
│   ├── notifier.rs             # 通知器
│   └── status.rs               # 状态跟踪
│
├── display/                    # 显示层
│   ├── mod.rs
│   ├── formatter.rs            # 格式化器
│   ├── colorizer.rs            # 颜色处理
│   ├── diff_viewer.rs          # 差异显示器
│   ├── table.rs                # 表格显示
│   └── progress.rs             # 进度显示
│
├── utils/                      # 工具模块
│   ├── mod.rs
│   ├── path.rs                 # 路径工具
│   ├── string.rs               # 字符串工具
│   ├── time.rs                 # 时间工具
│   ├── validation.rs           # 验证工具
│   └── process.rs              # 进程工具
│
└── i18n.rs                     # 国际化
```

## 🔄 重构步骤

### 阶段 1: 核心模块拆分
1. **拆分 git/worktree.rs** (871行)
   - 创建 git/worktree/ 目录
   - 按功能拆分为 6个文件
   
2. **拆分 commands/enhanced.rs** (826行)
   - 创建 commands/enhanced/ 目录
   - 按命令类型拆分为 4个文件

3. **拆分 git/watcher.rs** (800行)
   - 创建 monitor/ 目录
   - 拆分监控相关功能

### 阶段 2: 功能模块重组
1. **重组 Git 操作层**
   - 按操作类型分组 (core, branch, tag 等)
   - 每个具体操作一个文件

2. **重组 AI 模块**
   - 按提供商分离
   - 独立的客户端抽象

3. **重组命令处理**
   - 简化命令路由
   - 分离增强功能

### 阶段 3: 新功能模块
1. **查询系统独立**
   - 专门的查询解析和执行
   
2. **显示层抽象**
   - 统一的显示接口
   
3. **工具模块整合**
   - 通用工具函数集中

## 📊 优化效果预期

### 文件大小分布
- **超大文件 (>500行)**: 0个
- **大文件 (200-500行)**: <10个  
- **中等文件 (100-200行)**: ~20个
- **小文件 (<100行)**: ~40个

### 维护性提升
- ✅ 单一职责 - 每个文件职责清晰
- ✅ 低耦合 - 减少模块间依赖
- ✅ 高内聚 - 相关功能紧密组织
- ✅ 易测试 - 独立功能易于单元测试
- ✅ 易扩展 - 新功能可独立添加

### 开发体验
- 🔍 **更快定位** - 功能路径一目了然
- 🛠️ **独立开发** - 不同开发者可并行工作
- 🐛 **容易调试** - 问题范围更小
- 📚 **易于理解** - 新开发者快速上手
- 🔄 **安全重构** - 改动影响范围可控

## ⚡ 实施计划

1. **第1周**: 阶段1 - 核心模块拆分
2. **第2周**: 阶段2 - 功能模块重组  
3. **第3周**: 阶段3 - 新功能模块
4. **第4周**: 测试、文档更新、性能优化

每个阶段完成后进行完整的测试验证，确保功能正常。