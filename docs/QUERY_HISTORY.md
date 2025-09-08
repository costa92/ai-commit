# 查询历史功能文档

## 概述

ai-commit 工具现已支持查询历史记录功能，类似于 GRV (Git Repository Viewer) 的提示历史功能。该功能允许用户：

- 查看之前执行的查询历史
- 获取查询统计信息
- 交互式浏览和重新执行历史查询
- 清空查询历史

## 功能特性

### 1. 自动记录查询

每次执行查询时，系统会自动记录：
- 查询内容
- 执行时间
- 查询类型（execute, save 等）
- 结果数量
- 执行状态（成功/失败）

### 2. 持久化存储

查询历史保存在用户目录下：
```
~/.ai-commit/query_history.json
```

默认保留最近 1000 条查询记录。

## 使用方法

### 查看查询历史

```bash
# 显示最近的 20 条查询记录
ai-commit --query-history

# 输出示例：
📜 Query History (showing last 20 entries):
────────────────────────────────────────────────────────────
✅ 2025-09-08 19:04:42 [execute] author:costa
   └─ Results: 176

✅ 2025-09-08 19:04:42 [execute] type:fix
   └─ Results: 207
────────────────────────────────────────────────────────────
Total queries in history: 5
```

### 查看统计信息

```bash
# 显示查询统计
ai-commit --query-stats

# 输出示例：
📊 Query History Statistics:
────────────────────────────────────────
Total queries:      5
Successful queries: 5 (100.0%)
Failed queries:     0 (0.0%)

Query types:
  execute: 5
```

### 交互式浏览历史

```bash
# 启动交互式浏览器
ai-commit --query-browse

# 用户可以选择历史查询并重新执行
📜 Select a query from history:
────────────────────────────────────────────────────────────
 1. ✅ author:costa
 2. ✅ type:fix
 3. ✅ since:2024-01-01
────────────────────────────────────────────────────────────
Enter number (1-3) or 'q' to quit: 
```

### 清空查询历史

```bash
# 清空所有查询历史记录
ai-commit --query-clear

# 输出：
Query history cleared.
```

## 查询语法支持

系统支持以下查询条件：

- `author:name` - 按作者筛选
- `message:text` - 按提交消息筛选
- `since:date` - 从指定日期开始
- `until:date` - 到指定日期结束
- `file:path` - 按文件路径筛选
- `branch:name` - 按分支筛选
- `tag:name` - 按标签筛选

### 复合查询

支持 AND/OR 逻辑操作：

```bash
# AND 操作
ai-commit --query "author:john AND message:feat"

# OR 操作
ai-commit --query "author:john OR author:jane"

# 复杂查询
ai-commit --query "since:2024-01-01 AND (author:john OR author:jane) AND message:feat"
```

## 保存查询

可以保存常用查询供以后使用：

```bash
# 保存查询
ai-commit --query "save:my_query:author:john,since:2024-01-01"

# 列出保存的查询
ai-commit --query "list"
```

## 帮助信息

```bash
# 显示查询帮助
ai-commit --query "help"

# 输出包含：
- 查询语法说明
- 支持的字段
- 历史命令列表
```

## 配置选项

在配置文件中可以设置：

```toml
# 最大历史记录数（默认 1000）
prompt-history-size = 1000
```

## UI 集成

查询历史功能已完全集成到命令行界面：

1. **状态图标**：
   - ✅ 成功的查询
   - ❌ 失败的查询

2. **颜色编码**：
   - 绿色：成功查询
   - 红色：失败查询
   - 蓝色：查询类型标签

3. **格式化输出**：
   - 时间戳显示
   - 查询类型标签
   - 结果数量统计

## 实现细节

### 模块结构

```rust
src/
├── query_history.rs      # 查询历史核心模块
├── commands/
│   └── enhanced/
│       ├── mod.rs       # 增强命令路由
│       └── query.rs     # 查询命令处理
└── cli/
    └── args.rs          # CLI 参数定义
```

### 主要组件

1. **QueryHistory**：管理查询历史的核心结构
2. **QueryHistoryEntry**：单个历史记录条目
3. **QueryHistoryStats**：统计信息结构

### 数据持久化

使用 JSON 格式存储历史记录，支持：
- 自动加载和保存
- 循环缓冲（超过限制自动删除旧记录）
- 错误恢复（损坏文件自动重建）

## 性能优化

- 使用 `VecDeque` 实现高效的循环缓冲
- 异步 I/O 操作避免阻塞
- 增量更新而非全量重写

## 错误处理

- 文件系统错误自动恢复
- JSON 解析错误时重建历史
- 查询执行失败也会记录

## 未来改进

1. 支持更多查询类型
2. 添加查询模板功能
3. 支持导出/导入历史
4. 添加查询性能分析
5. 支持查询结果缓存

## 示例工作流

```bash
# 1. 执行一些查询
ai-commit --query "author:john"
ai-commit --query "since:2024-01-01"
ai-commit --query "message:feat"

# 2. 查看历史
ai-commit --query-history

# 3. 查看统计
ai-commit --query-stats

# 4. 交互式重新执行
ai-commit --query-browse
# 选择要重新执行的查询

# 5. 清理历史（如需要）
ai-commit --query-clear
```

## 故障排除

### 历史文件损坏

如果历史文件损坏，系统会自动重建：
```bash
rm ~/.ai-commit/query_history.json
# 下次执行查询时会自动创建新文件
```

### 权限问题

确保有写入权限：
```bash
chmod 755 ~/.ai-commit
chmod 644 ~/.ai-commit/query_history.json
```

## 总结

查询历史功能为 ai-commit 工具增加了强大的查询管理能力，让用户能够：
- 追踪查询历史
- 分析查询模式
- 快速重用查询
- 提高工作效率

该功能完全集成到现有命令行界面，提供直观的用户体验。