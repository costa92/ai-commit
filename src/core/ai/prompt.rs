use anyhow::Result;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::RwLock;

/// 提示词模板文件缓存
static PROMPT_FILE_CACHE: Lazy<RwLock<Option<String>>> = Lazy::new(|| RwLock::new(None));

/// 从文件系统加载提示词模板（带缓存）
///
/// 加载优先级:
/// 1. 本地 `commit-prompt.txt`
/// 2. 环境变量 `AI_COMMIT_PROMPT_PATH` 指定的文件
/// 3. 内置模板（编译时嵌入）
pub fn load_prompt_template_cached() -> String {
    // 检查缓存
    {
        let cache = PROMPT_FILE_CACHE.read().unwrap();
        if let Some(ref template) = *cache {
            return template.clone();
        }
    }

    // 加载模板
    let template = load_prompt_template_from_fs();
    *PROMPT_FILE_CACHE.write().unwrap() = Some(template.clone());

    template
}

/// 从文件系统加载提示词模板
fn load_prompt_template_from_fs() -> String {
    let default_path = "commit-prompt.txt";
    let prompt_path = if std::path::Path::new(default_path).exists() {
        default_path.to_string()
    } else {
        std::env::var("AI_COMMIT_PROMPT_PATH").unwrap_or_else(|_| default_path.to_owned())
    };

    if std::path::Path::new(&prompt_path).exists() {
        match std::fs::read_to_string(&prompt_path) {
            Ok(content) => content,
            Err(e) => {
                eprintln!("无法读取提示词文件 {}: {}，使用内置模板", prompt_path, e);
                include_str!("../../../commit-prompt.txt").to_owned()
            }
        }
    } else {
        include_str!("../../../commit-prompt.txt").to_owned()
    }
}

/// 获取替换了 diff 的完整提示词（兼容旧 API）
pub fn get_prompt(diff: &str) -> String {
    let template = load_prompt_template_cached();
    template.replace("{{git_diff}}", diff)
}

/// 清除提示词文件缓存
pub fn clear_prompt_cache() {
    *PROMPT_FILE_CACHE.write().unwrap() = None;
}

/// 提示词模板
#[derive(Debug, Clone)]
pub struct PromptTemplate {
    pub name: String,
    pub template: String,
    pub variables: Vec<String>,
}

impl PromptTemplate {
    /// 创建新的模板
    pub fn new(name: impl Into<String>, template: impl Into<String>) -> Self {
        let template_str = template.into();
        let variables = Self::extract_variables(&template_str);

        Self {
            name: name.into(),
            template: template_str,
            variables,
        }
    }

    /// 从模板中提取变量
    fn extract_variables(template: &str) -> Vec<String> {
        use regex::Regex;

        let re = Regex::new(r"\{\{(\w+)\}\}").unwrap();
        re.captures_iter(template)
            .map(|cap| cap[1].to_string())
            .collect()
    }

    /// 渲染模板
    pub fn render(&self, values: &HashMap<String, String>) -> Result<String> {
        let mut result = self.template.clone();

        for var in &self.variables {
            let value = values
                .get(var)
                .ok_or_else(|| anyhow::anyhow!("Missing variable: {}", var))?;
            result = result.replace(&format!("{{{{{}}}}}", var), value);
        }

        Ok(result)
    }
}

/// 提示词构建器
pub struct PromptBuilder {
    templates: HashMap<String, PromptTemplate>,
}

impl Default for PromptBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl PromptBuilder {
    /// 创建新的构建器
    pub fn new() -> Self {
        let mut builder = Self {
            templates: HashMap::new(),
        };

        builder.load_default_templates();
        builder
    }

    /// 加载默认模板
    fn load_default_templates(&mut self) {
        // 优先从文件加载 commit 模板
        let commit_template_str = load_prompt_template_cached();
        let commit_template = PromptTemplate::new("commit", commit_template_str);
        self.templates.insert("commit".to_string(), commit_template);

        // Tag 提示词模板
        let tag_template = PromptTemplate::new(
            "tag",
            r#"基于以下变更内容，生成版本 {{version}} 的发布说明。

要求：
1. 使用中文
2. 简洁明了，突出重点
3. 按功能分类列出主要变更

变更内容：
{{changes}}

输出格式：
## 版本 {{version}}

### 新功能
- 功能描述

### 改进
- 改进描述

### 修复
- 修复描述"#,
        );
        self.templates.insert("tag".to_string(), tag_template);
    }

    /// 构建提交提示词
    pub fn build_commit_prompt(&self, diff: &str) -> Result<String> {
        let template = self
            .templates
            .get("commit")
            .ok_or_else(|| anyhow::anyhow!("Commit template not found"))?;

        let mut values = HashMap::new();
        values.insert("git_diff".to_string(), diff.to_string());

        template.render(&values)
    }

    /// 构建标签提示词
    pub fn build_tag_prompt(&self, changes: &str, version: &str) -> Result<String> {
        let template = self
            .templates
            .get("tag")
            .ok_or_else(|| anyhow::anyhow!("Tag template not found"))?;

        let mut values = HashMap::new();
        values.insert("changes".to_string(), changes.to_string());
        values.insert("version".to_string(), version.to_string());

        template.render(&values)
    }

    /// 添加自定义模板
    pub fn add_template(&mut self, template: PromptTemplate) {
        self.templates.insert(template.name.clone(), template);
    }

    /// 获取模板
    pub fn get_template(&self, name: &str) -> Option<&PromptTemplate> {
        self.templates.get(name)
    }
}

/// 提示词优化器
pub struct PromptOptimizer;

impl PromptOptimizer {
    /// 优化大文件的提示词
    pub fn optimize_for_large_diff(diff: &str, max_chars: usize) -> String {
        if diff.len() <= max_chars {
            return diff.to_string();
        }

        let lines: Vec<&str> = diff.lines().collect();
        let mut optimized = Vec::new();
        let mut current_file = String::new();
        let mut important_lines = 0;

        for line in lines {
            if line.starts_with("diff --git") {
                if !current_file.is_empty() {
                    optimized.push(current_file.clone());
                }
                current_file = line.to_string();
                important_lines = 0;
            } else if line.starts_with("@@") {
                optimized.push(line.to_string());
                important_lines = 10;
            } else if important_lines > 0 {
                optimized.push(line.to_string());
                important_lines -= 1;
            } else if (line.starts_with('+') || line.starts_with('-'))
                && !line.starts_with("+++")
                && !line.starts_with("---")
            {
                optimized.push(line.to_string());
            }
        }

        let result = optimized.join("\n");
        if result.is_empty() {
            // 非 diff 格式的内容直接截断
            return diff.chars().take(max_chars).collect();
        }
        if result.len() > max_chars {
            result.chars().take(max_chars).collect()
        } else {
            result
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prompt_template_variables() {
        let template = PromptTemplate::new("test", "Hello {{name}}, your age is {{age}}");

        assert_eq!(template.variables.len(), 2);
        assert!(template.variables.contains(&"name".to_string()));
        assert!(template.variables.contains(&"age".to_string()));
    }

    #[test]
    fn test_prompt_template_render() {
        let template = PromptTemplate::new("test", "Hello {{name}}, welcome to {{place}}");

        let mut values = HashMap::new();
        values.insert("name".to_string(), "Alice".to_string());
        values.insert("place".to_string(), "Wonderland".to_string());

        let result = template.render(&values).unwrap();
        assert_eq!(result, "Hello Alice, welcome to Wonderland");
    }

    #[test]
    fn test_prompt_template_render_missing_variable() {
        let template = PromptTemplate::new("test", "Hello {{name}}");
        let values = HashMap::new();

        assert!(template.render(&values).is_err());
    }

    #[test]
    fn test_prompt_builder_default_templates() {
        let builder = PromptBuilder::new();

        assert!(builder.get_template("commit").is_some());
        assert!(builder.get_template("tag").is_some());
    }

    #[test]
    fn test_prompt_builder_add_template() {
        let mut builder = PromptBuilder::new();
        let template = PromptTemplate::new("custom", "Custom {{var}}");

        builder.add_template(template);
        assert!(builder.get_template("custom").is_some());
    }

    #[test]
    fn test_prompt_optimizer_small_diff() {
        let diff = "small diff content";
        let optimized = PromptOptimizer::optimize_for_large_diff(diff, 100);

        assert_eq!(optimized, diff);
    }

    #[test]
    fn test_prompt_optimizer_large_diff() {
        let diff = "a".repeat(1000);
        let optimized = PromptOptimizer::optimize_for_large_diff(&diff, 100);

        assert_eq!(optimized.len(), 100);
    }

    #[test]
    fn test_get_prompt_replaces_diff() {
        clear_prompt_cache();
        let prompt = get_prompt("test diff content");
        assert!(prompt.contains("test diff content"));
        assert!(!prompt.contains("{{git_diff}}"));
    }

    #[test]
    fn test_get_prompt_cached() {
        clear_prompt_cache();
        let p1 = get_prompt("diff1");
        let p2 = get_prompt("diff2");
        assert!(p1.contains("diff1"));
        assert!(p2.contains("diff2"));
    }

    #[test]
    fn test_clear_prompt_cache() {
        clear_prompt_cache();
        let cache = PROMPT_FILE_CACHE.read().unwrap();
        assert!(cache.is_none());
    }
}
