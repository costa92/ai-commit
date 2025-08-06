/// AI 审查提示词模板
pub struct ReviewPromptTemplates;

impl ReviewPromptTemplates {
    /// Go 语言代码审查模板
    pub fn go_review_template() -> &'static str {
        r#"你是一个专业的 Go 语言代码审查专家。请对以下 Go 代码进行全面审查，并提供详细的分析和建议。

## 审查重点：
1. **代码规范**：检查是否符合 Go 语言官方编码规范
2. **错误处理**：评估错误处理的完整性和正确性
3. **并发安全**：检查 goroutine 和 channel 的使用是否安全
4. **性能优化**：识别潜在的性能问题和优化机会
5. **内存管理**：检查是否存在内存泄漏风险
6. **接口设计**：评估接口的设计是否合理
7. **测试覆盖**：建议需要添加的测试用例

## 代码信息：
- 文件路径：{file_path}
- 代码内容：
```go
{code_content}
```

## 请按以下格式输出审查结果：

### 质量评分
**总分：** [1-10分]

### 主要问题
1. **[问题类型]** - [问题描述]
   - 位置：第X行
   - 建议：[具体修复建议]

### 优化建议
1. **[优化类型]** - [优化描述]
   - 原因：[为什么需要优化]
   - 方案：[具体优化方案]

### 最佳实践建议
- [Go 语言最佳实践建议]

### 学习资源
- [相关学习资源链接和说明]

请确保审查结果专业、准确、有建设性。"#
    }

    /// Rust 语言代码审查模板
    pub fn rust_review_template() -> &'static str {
        r#"你是一个专业的 Rust 语言代码审查专家。请对以下 Rust 代码进行全面审查，并提供详细的分析和建议。

## 审查重点：
1. **内存安全**：检查所有权、借用和生命周期的正确使用
2. **错误处理**：评估 Result 和 Option 的使用是否恰当
3. **性能优化**：识别零成本抽象和性能优化机会
4. **并发安全**：检查线程安全和异步代码的正确性
5. **代码风格**：确保符合 Rust 社区约定
6. **类型设计**：评估类型系统的使用是否充分
7. **unsafe 代码**：如果存在，检查其安全性和必要性

## 代码信息：
- 文件路径：{file_path}
- 代码内容：
```rust
{code_content}
```

## 请按以下格式输出审查结果：

### 质量评分
**总分：** [1-10分]

### 内存安全分析
- **所有权检查：** [分析结果]
- **借用检查：** [分析结果]
- **生命周期：** [分析结果]

### 主要问题
1. **[问题类型]** - [问题描述]
   - 位置：第X行
   - 建议：[具体修复建议]

### 性能优化建议
1. **[优化类型]** - [优化描述]
   - 影响：[性能影响分析]
   - 方案：[具体优化方案]

### Rust 最佳实践
- [Rust 特有的最佳实践建议]

### 学习资源
- [Rust 相关学习资源链接和说明]

请确保审查结果体现 Rust 语言的特色和优势。"#
    }

    /// TypeScript 语言代码审查模板
    pub fn typescript_review_template() -> &'static str {
        r#"你是一个专业的 TypeScript 语言代码审查专家。请对以下 TypeScript 代码进行全面审查，并提供详细的分析和建议。

## 审查重点：
1. **类型安全**：检查类型定义和使用的正确性
2. **异步处理**：评估 Promise、async/await 的使用
3. **模块化设计**：检查模块导入导出的合理性
4. **错误处理**：评估异常处理和错误边界
5. **性能优化**：识别潜在的性能问题
6. **代码风格**：确保符合 TypeScript 最佳实践
7. **兼容性**：检查浏览器和 Node.js 兼容性

## 代码信息：
- 文件路径：{file_path}
- 代码内容：
```typescript
{code_content}
```

## 请按以下格式输出审查结果：

### 质量评分
**总分：** [1-10分]

### 类型安全分析
- **类型定义：** [分析结果]
- **类型推断：** [分析结果]
- **泛型使用：** [分析结果]

### 主要问题
1. **[问题类型]** - [问题描述]
   - 位置：第X行
   - 建议：[具体修复建议]

### 异步代码审查
- **Promise 使用：** [分析结果]
- **错误处理：** [分析结果]
- **性能考虑：** [分析结果]

### 优化建议
1. **[优化类型]** - [优化描述]
   - 原因：[为什么需要优化]
   - 方案：[具体优化方案]

### TypeScript 最佳实践
- [TypeScript 特有的最佳实践建议]

### 学习资源
- [TypeScript 相关学习资源链接和说明]

请确保审查结果专业且针对 TypeScript 的特点。"#
    }

    /// 通用代码审查模板
    pub fn generic_review_template() -> &'static str {
        r#"你是一个专业的代码审查专家。请对以下代码进行全面审查，并提供详细的分析和建议。

## 审查重点：
1. **代码质量**：检查代码的可读性、可维护性
2. **逻辑正确性**：评估代码逻辑是否正确
3. **性能考虑**：识别潜在的性能问题
4. **安全性**：检查可能的安全漏洞
5. **最佳实践**：确保符合编程最佳实践
6. **文档注释**：评估代码注释的完整性

## 代码信息：
- 文件路径：{file_path}
- 编程语言：{language}
- 代码内容：
```{language}
{code_content}
```

## 请按以下格式输出审查结果：

### 质量评分
**总分：** [1-10分]

### 主要问题
1. **[问题类型]** - [问题描述]
   - 位置：第X行
   - 建议：[具体修复建议]

### 优化建议
1. **[优化类型]** - [优化描述]
   - 原因：[为什么需要优化]
   - 方案：[具体优化方案]

### 最佳实践建议
- [编程最佳实践建议]

### 学习资源
- [相关学习资源链接和说明]

请确保审查结果专业、准确、有建设性。"#
    }

    /// 质量评分提示模板
    pub fn quality_scoring_template() -> &'static str {
        r#"请对以下代码进行质量评分（1-10分），并简要说明评分理由。

## 评分标准：
- **9-10分**：优秀 - 代码质量极高，几乎无需改进
- **7-8分**：良好 - 代码质量较好，有少量改进空间
- **5-6分**：一般 - 代码基本可用，但有明显改进空间
- **3-4分**：较差 - 代码存在较多问题，需要重构
- **1-2分**：很差 - 代码存在严重问题，建议重写

## 代码信息：
- 文件路径：{file_path}
- 编程语言：{language}
- 代码内容：
```{language}
{code_content}
```

请输出：
**评分：** [1-10分]
**理由：** [简要说明评分理由，不超过100字]"#
    }

    /// 改进建议生成模板
    pub fn improvement_suggestion_template() -> &'static str {
        r#"请为以下代码提供具体的改进建议，包括代码示例。

## 代码信息：
- 文件路径：{file_path}
- 编程语言：{language}
- 代码内容：
```{language}
{code_content}
```

## 请按以下格式输出改进建议：

### 改进建议
1. **[改进类型]** - [改进描述]
   - **问题：** [具体问题说明]
   - **建议：** [改进方案]
   - **示例：**
   ```{language}
   // 改进后的代码示例
   ```

### 重构建议
- [如果需要重构，提供重构建议]

### 最佳实践
- [相关最佳实践建议]

请确保建议具体可行，并提供代码示例。"#
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_go_review_template() {
        let template = ReviewPromptTemplates::go_review_template();
        assert!(template.contains("Go 语言代码审查专家"));
        assert!(template.contains("并发安全"));
        assert!(template.contains("goroutine"));
        assert!(template.contains("{file_path}"));
        assert!(template.contains("{code_content}"));
    }

    #[test]
    fn test_rust_review_template() {
        let template = ReviewPromptTemplates::rust_review_template();
        assert!(template.contains("Rust 语言代码审查专家"));
        assert!(template.contains("内存安全"));
        assert!(template.contains("所有权"));
        assert!(template.contains("借用"));
        assert!(template.contains("{file_path}"));
        assert!(template.contains("{code_content}"));
    }

    #[test]
    fn test_typescript_review_template() {
        let template = ReviewPromptTemplates::typescript_review_template();
        assert!(template.contains("TypeScript 语言代码审查专家"));
        assert!(template.contains("类型安全"));
        assert!(template.contains("async/await"));
        assert!(template.contains("{file_path}"));
        assert!(template.contains("{code_content}"));
    }

    #[test]
    fn test_generic_review_template() {
        let template = ReviewPromptTemplates::generic_review_template();
        assert!(template.contains("代码审查专家"));
        assert!(template.contains("代码质量"));
        assert!(template.contains("{file_path}"));
        assert!(template.contains("{language}"));
        assert!(template.contains("{code_content}"));
    }

    #[test]
    fn test_quality_scoring_template() {
        let template = ReviewPromptTemplates::quality_scoring_template();
        assert!(template.contains("质量评分"));
        assert!(template.contains("1-10分"));
        assert!(template.contains("评分标准"));
        assert!(template.contains("{file_path}"));
        assert!(template.contains("{language}"));
        assert!(template.contains("{code_content}"));
    }

    #[test]
    fn test_improvement_suggestion_template() {
        let template = ReviewPromptTemplates::improvement_suggestion_template();
        assert!(template.contains("改进建议"));
        assert!(template.contains("代码示例"));
        assert!(template.contains("{file_path}"));
        assert!(template.contains("{language}"));
        assert!(template.contains("{code_content}"));
    }

    #[test]
    fn test_all_templates_have_placeholders() {
        let templates = vec![
            ReviewPromptTemplates::go_review_template(),
            ReviewPromptTemplates::rust_review_template(),
            ReviewPromptTemplates::typescript_review_template(),
            ReviewPromptTemplates::generic_review_template(),
            ReviewPromptTemplates::quality_scoring_template(),
            ReviewPromptTemplates::improvement_suggestion_template(),
        ];

        for template in templates {
            // 所有模板都应该包含文件路径占位符
            assert!(template.contains("{file_path}"));
            // 所有模板都应该包含代码内容占位符
            assert!(template.contains("{code_content}"));
        }
    }

    #[test]
    fn test_template_formatting() {
        let templates = vec![
            ("go", ReviewPromptTemplates::go_review_template()),
            ("rust", ReviewPromptTemplates::rust_review_template()),
            ("typescript", ReviewPromptTemplates::typescript_review_template()),
        ];

        for (lang, template) in templates {
            // 检查模板格式是否正确
            assert!(template.contains("## 审查重点"));
            assert!(template.contains("### 质量评分"));
            assert!(template.contains("### 主要问题"));
            assert!(template.contains("### 学习资源"));

            // 检查代码块格式
            assert!(template.contains(&format!("```{}", lang)));
        }
    }
}