# AI-Commit 重构计划

## 目标

1. **提升代码可读性**：模块化设计，职责单一
2. **增强扩展性**：使用 trait 和接口，易于添加新功能
3. **简化命令行**：采用子命令模式，分组管理功能
4. **优化性能**：并行处理、缓存机制、连接复用
5. **完善测试**：每个模块都有完整的单元测试

## 架构改进

### 1. 新的目录结构

```
src/
├── core/                    # 核心业务逻辑
│   ├── ai/                 # AI 相关
│   │   ├── mod.rs          # AI 服务接口
│   │   ├── provider.rs     # Provider trait 定义
│   │   ├── prompt.rs       # 提示词管理
│   │   └── providers/      # 具体实现
│   │       ├── ollama.rs
│   │       ├── deepseek.rs
│   │       └── siliconflow.rs
│   ├── git/                # Git 操作
│   │   ├── mod.rs          # Git 统一接口
│   │   ├── repository.rs   # 仓库操作
│   │   ├── commit.rs       # 提交管理
│   │   ├── branch.rs       # 分支管理
│   │   ├── tag.rs          # 标签管理
│   │   └── worktree.rs     # 工作树管理
│   └── config/             # 配置管理
│       ├── mod.rs          # 配置接口
│       ├── loader.rs       # 配置加载
│       └── validator.rs    # 配置验证
├── cli/                    # 命令行接口
│   ├── mod.rs             # CLI 主入口
│   ├── commands/          # 子命令实现
│   │   ├── commit.rs      # commit 子命令
│   │   ├── tag.rs         # tag 子命令
│   │   ├── flow.rs        # flow 子命令
│   │   ├── worktree.rs    # worktree 子命令
│   │   ├── history.rs     # history 子命令
│   │   └── config.rs      # config 子命令
│   └── parser.rs          # 命令解析器
├── utils/                  # 工具函数
│   ├── cache.rs           # 缓存管理
│   ├── async_pool.rs      # 异步任务池
│   └── format.rs          # 格式化工具
└── tests/                 # 测试
    ├── unit/              # 单元测试
    └── integration/       # 集成测试
```

### 2. 命令行简化（子命令模式）

**之前**：100+ 个平铺的参数
```bash
ai-commit --new-tag v1.0.0 --tag-note "release" --push --force-push
```

**重构后**：结构化的子命令
```bash
ai-commit tag create v1.0.0 --note "release" --push
ai-commit commit generate --push
ai-commit flow feature start user-auth
ai-commit worktree create feature/new-ui
```

### 3. 核心改进点

#### AI 模块改进
- **Provider Trait**：统一的 AI 提供商接口
- **动态加载**：使用工厂模式创建提供商
- **提示词模板**：可配置、可扩展的模板系统
- **流式处理**：优化内存使用，实时输出

#### Git 模块改进
- **Repository 抽象**：封装底层 Git 操作
- **Manager 模式**：每种操作有专门的管理器
- **异步执行**：所有 Git 命令异步化
- **批量操作**：并行执行多个命令

#### 配置系统改进
- **分层配置**：全局、项目、环境变量
- **类型安全**：使用 serde 进行序列化
- **验证机制**：自动验证配置有效性
- **缓存优化**：减少重复读取

### 4. 性能优化

1. **HTTP 连接复用**
   - 使用全局客户端单例
   - 连接池管理
   - 超时和重试机制

2. **并行处理**
   - Git 命令并行执行
   - 异步 I/O 操作
   - 任务池管理

3. **内存优化**
   - 预分配缓冲区
   - 流式处理大文件
   - 智能 diff 分析

4. **缓存策略**
   - 配置缓存
   - Git 状态缓存
   - 提示词模板缓存

### 5. 测试策略

每个模块都包含完整的单元测试：

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_functionality() {
        // 测试基本功能
    }

    #[test]
    fn test_edge_cases() {
        // 测试边界情况
    }

    #[tokio::test]
    async fn test_async_operations() {
        // 测试异步操作
    }
}
```

### 6. 使用示例

#### 基本使用
```bash
# 生成并提交
ai-commit commit generate

# 创建标签
ai-commit tag create v1.0.0

# Git Flow
ai-commit flow init
ai-commit flow feature start new-feature
ai-commit flow feature finish new-feature

# 工作树管理
ai-commit worktree list
ai-commit worktree create feature/test
```

#### 高级使用
```bash
# 自定义 AI 提供商
ai-commit config set provider deepseek
ai-commit config set model gpt-4

# 批量操作
ai-commit commit generate --add --push --tag v1.0.0

# 历史分析
ai-commit history log --author "张三" --since "2024-01-01"
ai-commit history stats --format json
```

## 实施步骤

1. ✅ 分析现有代码架构问题
2. ✅ 设计新的命令行架构
3. ✅ 重构 AI 模块（提升扩展性）
4. 🔄 重构 Git 模块（单一职责）
5. ⏳ 实现新的 CLI 子命令结构
6. ⏳ 优化性能瓶颈
7. ⏳ 添加完整的单元测试
8. ⏳ 编写迁移指南
9. ⏳ 更新文档

## 优势对比

| 方面 | 重构前 | 重构后 |
|------|--------|--------|
| 命令复杂度 | 100+ 参数 | 结构化子命令 |
| 代码可读性 | 单文件过大 | 模块化设计 |
| 扩展性 | 修改困难 | 易于扩展 |
| 测试覆盖 | 部分测试 | 完整测试 |
| 性能 | 串行执行 | 并行优化 |
| 维护成本 | 高 | 低 |

## 迁移建议

1. **向后兼容**：保留旧命令别名
2. **渐进迁移**：逐步替换模块
3. **文档更新**：提供详细迁移指南
4. **测试验证**：确保功能不受影响

## 下一步行动

1. 完成剩余模块的重构实现
2. 实现完整的 CLI 子命令系统
3. 添加性能基准测试
4. 编写用户迁移文档
5. 发布 v2.0.0 版本