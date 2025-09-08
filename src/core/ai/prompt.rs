use anyhow::Result;
use std::collections::HashMap;

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

impl PromptBuilder {
    /// 创建新的构建器
    pub fn new() -> Self {
        let mut builder = Self {
            templates: HashMap::new(),
        };
        
        // 加载默认模板
        builder.load_default_templates();
        builder
    }
    
    /// 加载默认模板
    fn load_default_templates(&mut self) {
        // Commit 提示词模板
        let commit_template = PromptTemplate::new(
            "commit",
            include_str!("../../../commit-prompt.txt"),
        );
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
        let template = self.templates
            .get("commit")
            .ok_or_else(|| anyhow::anyhow!("Commit template not found"))?;
        
        let mut values = HashMap::new();
        values.insert("git_diff".to_string(), diff.to_string());
        
        template.render(&values)
    }
    
    /// 构建标签提示词
    pub fn build_tag_prompt(&self, changes: &str, version: &str) -> Result<String> {
        let template = self.templates
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
        
        // 提取关键变更
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
                important_lines = 10; // 保留上下文行数
            } else if important_lines > 0 {
                optimized.push(line.to_string());
                important_lines -= 1;
            } else if line.starts_with('+') || line.starts_with('-') {
                if !line.starts_with("+++") && !line.starts_with("---") {
                    optimized.push(line.to_string());
                }
            }
        }
        
        let result = optimized.join("\n");
        if result.len() > max_chars {
            // 如果还是太长，截取前面部分
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
        let template = PromptTemplate::new(
            "test",
            "Hello {{name}}, your age is {{age}}",
        );
        
        assert_eq!(template.variables.len(), 2);
        assert!(template.variables.contains(&"name".to_string()));
        assert!(template.variables.contains(&"age".to_string()));
    }

    #[test]
    fn test_prompt_template_render() {
        let template = PromptTemplate::new(
            "test",
            "Hello {{name}}, welcome to {{place}}",
        );
        
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
}