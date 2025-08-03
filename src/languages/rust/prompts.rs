/// Rust 代码审查的 AI 提示词模板

pub const RUST_CODE_REVIEW_PROMPT: &str = r#"
你是一个专业的 Rust 代码审查专家。请对以下 Rust 代码变更进行详细审查，并提供建设性的反馈。

## 审查重点：

### 1. Rust 特有的最佳实践
- 所有权和生命周期管理
- 借用检查器友好的代码设计
- 错误处理（Result/Option 使用）
- 零成本抽象的应用
- 内存安全和并发安全

### 2. 代码质量
- 函数和变量命名（遵循 snake_case）
- 类型和 trait 命名（遵循 PascalCase）
- 代码可读性和维护性
- 模块组织和可见性

### 3. 性能考虑
- 避免不必要的克隆和分配
- 合理使用引用和切片
- 编译时优化机会
- 异步代码的性能影响

### 4. 安全性审查
- unsafe 代码的正确性
- 数据竞争和内存泄漏风险
- 输入验证和边界检查
- 依赖项安全性

### 5. 测试覆盖
- 单元测试完整性
- 集成测试需求
- 文档测试的使用
- 错误路径测试

## 输出格式：
请提供以下格式的审查报告：

### 🔍 代码审查摘要
- 整体代码质量评分（1-10）
- 主要优点
- 需要改进的地方

### ⚠️ 关键问题
- 安全性问题
- 性能问题
- 潜在的运行时错误

### 💡 改进建议
- 具体的代码改进建议
- Rust 惯用法的应用
- 架构优化建议

### 🧪 测试建议
- 需要添加的测试用例
- 测试策略改进

### 📚 学习资源
- 相关的 Rust 文档链接
- 推荐的最佳实践

代码变更：
```rust
{code_diff}
```

文件路径：{file_path}
变更统计：{change_stats}
"#;

pub const RUST_SECURITY_REVIEW_PROMPT: &str = r#"
作为 Rust 安全专家，请专注于以下代码的安全性审查：

## 安全审查要点：

### 1. 内存安全
- 检查 unsafe 代码块的正确性
- 验证生命周期参数的安全性
- 识别潜在的悬垂指针或内存泄漏

### 2. 并发安全
- 数据竞争检测
- 死锁风险评估
- 原子操作的正确使用

### 3. 输入验证
- 边界检查
- 整数溢出保护
- 字符串处理安全性

### 4. 依赖安全
- 第三方 crate 的安全性
- 版本固定和漏洞检查

请重点关注以下代码的安全性：

```rust
{code_diff}
```

文件：{file_path}
"#;

pub const RUST_PERFORMANCE_REVIEW_PROMPT: &str = r#"
作为 Rust 性能优化专家，请分析以下代码的性能特征：

## 性能审查要点：

### 1. 内存效率
- 不必要的堆分配
- 克隆操作的优化机会
- 数据结构选择

### 2. 计算效率
- 算法复杂度分析
- 循环和迭代器优化
- 编译时计算机会

### 3. 并发性能
- 异步代码的性能影响
- 线程池使用
- 锁的粒度优化

### 4. 编译器优化
- 内联函数建议
- 零成本抽象的应用
- LLVM 优化提示

请分析以下代码的性能：

```rust
{code_diff}
```

文件：{file_path}
变更类型：{change_type}
"#;

pub const RUST_ARCHITECTURE_REVIEW_PROMPT: &str = r#"
作为 Rust 架构师，请评估以下代码的架构设计：

## 架构审查要点：

### 1. 模块设计
- 模块边界和职责分离
- 公共 API 的设计
- 向后兼容性考虑

### 2. Trait 设计
- Trait 的内聚性和扩展性
- 泛型约束的合理性
- 默认实现的使用

### 3. 错误处理
- 错误类型的设计
- 错误传播策略
- 恢复机制

### 4. 生态系统集成
- 标准库的充分利用
- 社区 crate 的选择
- API 设计的 Rust 惯用性

请评估以下代码的架构：

```rust
{code_diff}
```

模块：{module_name}
组件类型：{component_type}
"#;

/// 获取适合特定审查类型的 Rust 提示词
pub fn get_rust_prompt(review_type: &str) -> &'static str {
    match review_type {
        "security" => RUST_SECURITY_REVIEW_PROMPT,
        "performance" => RUST_PERFORMANCE_REVIEW_PROMPT,
        "architecture" => RUST_ARCHITECTURE_REVIEW_PROMPT,
        _ => RUST_CODE_REVIEW_PROMPT,
    }
}

/// 根据代码特征选择最适合的审查类型
pub fn suggest_review_type(code_content: &str) -> &'static str {
    if code_content.contains("unsafe") || code_content.contains("transmute") {
        "security"
    } else if code_content.contains("async") || code_content.contains("Arc") || code_content.contains("Mutex") {
        "performance"
    } else if code_content.contains("trait") || code_content.contains("impl") || code_content.contains("pub mod") {
        "architecture"
    } else {
        "general"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prompt_selection() {
        assert_eq!(get_rust_prompt("security"), RUST_SECURITY_REVIEW_PROMPT);
        assert_eq!(get_rust_prompt("performance"), RUST_PERFORMANCE_REVIEW_PROMPT);
        assert_eq!(get_rust_prompt("architecture"), RUST_ARCHITECTURE_REVIEW_PROMPT);
        assert_eq!(get_rust_prompt("general"), RUST_CODE_REVIEW_PROMPT);
    }

    #[test]
    fn test_review_type_suggestion() {
        assert_eq!(suggest_review_type("unsafe fn dangerous() {}"), "security");
        assert_eq!(suggest_review_type("async fn process() {}"), "performance");
        assert_eq!(suggest_review_type("trait MyTrait {}"), "architecture");
        assert_eq!(suggest_review_type("fn normal() {}"), "general");
    }

    #[test]
    fn test_prompt_contains_placeholders() {
        assert!(RUST_CODE_REVIEW_PROMPT.contains("{code_diff}"));
        assert!(RUST_CODE_REVIEW_PROMPT.contains("{file_path}"));
        assert!(RUST_SECURITY_REVIEW_PROMPT.contains("{code_diff}"));
        assert!(RUST_PERFORMANCE_REVIEW_PROMPT.contains("{change_type}"));
    }
}