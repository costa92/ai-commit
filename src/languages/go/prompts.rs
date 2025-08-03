/// Go 代码审查的 AI 提示词模板
pub const GO_CODE_REVIEW_PROMPT: &str = r#"
你是一个专业的 Go 代码审查专家。请对以下 Go 代码变更进行详细审查，并提供建设性的反馈。

## 审查重点：

### 1. Go 特有的最佳实践
- 错误处理（error 返回值和处理）
- 接口设计和组合优于继承
- 并发安全（goroutine 和 channel 使用）
- 内存管理和垃圾回收优化
- Go 惯用法（idioms）的应用

### 2. 代码质量
- 命名规范（驼峰命名法）
- 包设计和导入管理
- 函数和方法的简洁性
- 代码可读性和维护性

### 3. 性能考虑
- 避免不必要的内存分配
- 合理使用指针和值类型
- 并发模式的性能影响
- 编译优化机会

### 4. 安全性审查
- 数据竞争检测
- 内存安全问题
- 输入验证和边界检查
- 依赖项安全性

### 5. 测试覆盖
- 单元测试完整性
- 表驱动测试的使用
- 基准测试需求
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
- Go 惯用法的应用
- 架构优化建议

### 🧪 测试建议
- 需要添加的测试用例
- 测试策略改进

### 📚 学习资源
- 相关的 Go 文档链接
- 推荐的最佳实践

代码变更：
```go
{code_diff}
```

文件路径：{file_path}
变更统计：{change_stats}
"#;

pub const GO_CONCURRENCY_REVIEW_PROMPT: &str = r#"
作为 Go 并发编程专家，请专注于以下代码的并发安全性审查：

## 并发审查要点：

### 1. Goroutine 管理
- Goroutine 泄漏检测
- 适当的生命周期管理
- 错误处理和恢复机制

### 2. Channel 使用
- Channel 的正确使用模式
- 死锁风险评估
- 缓冲 channel 的大小选择

### 3. 同步原语
- Mutex 和 RWMutex 的使用
- sync.WaitGroup 的正确应用
- atomic 操作的合理性

### 4. 数据竞争
- 共享数据的保护
- 竞态条件检测
- 内存可见性问题

请重点关注以下代码的并发安全性：

```go
{code_diff}
```

文件：{file_path}
并发特征：{concurrency_features}
"#;

pub const GO_PERFORMANCE_REVIEW_PROMPT: &str = r#"
作为 Go 性能优化专家，请分析以下代码的性能特征：

## 性能审查要点：

### 1. 内存效率
- 避免不必要的堆分配
- 字符串处理优化
- 切片和映射的使用

### 2. 算法效率
- 时间复杂度分析
- 空间复杂度优化
- 数据结构选择

### 3. 并发性能
- Goroutine 池的使用
- 锁的粒度优化
- Channel 的性能影响

### 4. 编译器优化
- 内联函数建议
- 逃逸分析优化
- 编译器提示

请分析以下代码的性能：

```go
{code_diff}
```

文件：{file_path}
性能关键点：{performance_hotspots}
"#;

pub const GO_API_DESIGN_REVIEW_PROMPT: &str = r#"
作为 Go API 设计专家，请评估以下代码的 API 设计：

## API 设计审查要点：

### 1. 接口设计
- 接口的简洁性和组合性
- 依赖倒置原则的应用
- 向后兼容性考虑

### 2. 错误处理
- 错误类型的设计
- 错误包装和传播
- 上下文信息的提供

### 3. 包设计
- 包的职责单一性
- 导出标识符的设计
- 文档和示例

### 4. 生态系统集成
- 标准库的充分利用
- 第三方库的选择
- API 的 Go 惯用性

请评估以下代码的 API 设计：

```go
{code_diff}
```

包名：{package_name}
API 类型：{api_type}
"#;

/// 获取适合特定审查类型的 Go 提示词
pub fn get_go_prompt(review_type: &str) -> &'static str {
    match review_type {
        "concurrency" => GO_CONCURRENCY_REVIEW_PROMPT,
        "performance" => GO_PERFORMANCE_REVIEW_PROMPT,
        "api_design" => GO_API_DESIGN_REVIEW_PROMPT,
        _ => GO_CODE_REVIEW_PROMPT,
    }
}

/// 根据代码特征选择最适合的审查类型
pub fn suggest_go_review_type(code_content: &str) -> &'static str {
    if code_content.contains("go ")
        || code_content.contains("chan ")
        || code_content.contains("sync.")
    {
        "concurrency"
    } else if code_content.contains("benchmark")
        || code_content.contains("time.")
        || code_content.contains("make(")
    {
        "performance"
    } else if code_content.contains("interface")
        || code_content.contains("type ")
        || code_content.contains("func (")
    {
        "api_design"
    } else {
        "general"
    }
}

/// 检测 Go 代码中的并发特征
pub fn detect_concurrency_features(code_content: &str) -> Vec<String> {
    let mut features = Vec::new();

    if code_content.contains("go ") {
        features.push("Goroutine 启动".to_string());
    }
    if code_content.contains("chan ") || code_content.contains("make(chan") {
        features.push("Channel 使用".to_string());
    }
    if code_content.contains("sync.Mutex") || code_content.contains("sync.RWMutex") {
        features.push("互斥锁".to_string());
    }
    if code_content.contains("sync.WaitGroup") {
        features.push("等待组".to_string());
    }
    if code_content.contains("atomic.") {
        features.push("原子操作".to_string());
    }
    if code_content.contains("context.") {
        features.push("上下文管理".to_string());
    }

    features
}

/// 检测性能热点
pub fn detect_performance_hotspots(code_content: &str) -> Vec<String> {
    let mut hotspots = Vec::new();

    if code_content.contains("make([]") && code_content.contains("append(") {
        hotspots.push("切片频繁扩容".to_string());
    }
    if code_content.contains("strings.") && code_content.contains("+") {
        hotspots.push("字符串拼接".to_string());
    }
    if code_content.contains("for") && code_content.contains("range") {
        hotspots.push("循环遍历".to_string());
    }
    if code_content.contains("new(") || code_content.contains("&") {
        hotspots.push("内存分配".to_string());
    }
    if code_content.contains("json.") || code_content.contains("xml.") {
        hotspots.push("序列化操作".to_string());
    }

    hotspots
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_go_prompt_selection() {
        assert_eq!(get_go_prompt("concurrency"), GO_CONCURRENCY_REVIEW_PROMPT);
        assert_eq!(get_go_prompt("performance"), GO_PERFORMANCE_REVIEW_PROMPT);
        assert_eq!(get_go_prompt("api_design"), GO_API_DESIGN_REVIEW_PROMPT);
        assert_eq!(get_go_prompt("general"), GO_CODE_REVIEW_PROMPT);
    }

    #[test]
    fn test_go_review_type_suggestion() {
        assert_eq!(
            suggest_go_review_type("go func() { doWork() }()"),
            "concurrency"
        );
        assert_eq!(
            suggest_go_review_type("ch := make(chan int)"),
            "concurrency"
        );
        assert_eq!(
            suggest_go_review_type("func BenchmarkTest() {}"),
            "performance"
        );
        assert_eq!(
            suggest_go_review_type("type Writer interface {}"),
            "api_design"
        );
        assert_eq!(suggest_go_review_type("func normal() {}"), "general");
    }

    #[test]
    fn test_concurrency_features_detection() {
        let code = "go func() { ch := make(chan int); sync.Mutex{} }()";
        let features = detect_concurrency_features(code);

        assert!(features.contains(&"Goroutine 启动".to_string()));
        assert!(features.contains(&"Channel 使用".to_string()));
        assert!(features.contains(&"互斥锁".to_string()));
    }

    #[test]
    fn test_performance_hotspots_detection() {
        let code = "arr := make([]int, 0); arr = append(arr, 1); str := str1 + str2";
        let hotspots = detect_performance_hotspots(code);

        assert!(hotspots.contains(&"切片频繁扩容".to_string()));
        assert!(hotspots.contains(&"字符串拼接".to_string()));
    }

    #[test]
    fn test_prompt_contains_placeholders() {
        assert!(GO_CODE_REVIEW_PROMPT.contains("{code_diff}"));
        assert!(GO_CODE_REVIEW_PROMPT.contains("{file_path}"));
        assert!(GO_CONCURRENCY_REVIEW_PROMPT.contains("{concurrency_features}"));
        assert!(GO_PERFORMANCE_REVIEW_PROMPT.contains("{performance_hotspots}"));
    }
}
