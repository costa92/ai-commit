use super::{TemplateEngine, TemplateHelper};
use anyhow::Result;
use regex::Regex;
use serde_json::Value;
use std::collections::HashMap;

/// 简单模板引擎实现
pub struct SimpleTemplateEngine {
    helpers: HashMap<String, Box<dyn TemplateHelper>>,
}

impl SimpleTemplateEngine {
    /// 创建新的简单模板引擎
    pub fn new() -> Self {
        Self {
            helpers: HashMap::new(),
        }
    }
}

impl TemplateEngine for SimpleTemplateEngine {
    fn render(&self, template: &str, context: &HashMap<String, Value>) -> Result<String> {
        let mut result = template.to_string();

        // 处理变量替换 {{variable}}
        let var_regex = Regex::new(r"\{\{([^}]+)\}\}").unwrap();
        result = var_regex.replace_all(&result, |caps: &regex::Captures| {
            let var_name = caps.get(1).unwrap().as_str().trim();

            // 处理嵌套属性 (e.g., summary.total_issues)
            let value = self.get_nested_value(context, var_name);
            self.value_to_string(&value)
        }).to_string();

        // 处理条件语句 {{#if condition}}...{{/if}}
        result = self.process_conditionals(&result, context)?;

        // 处理循环语句 {{#each items}}...{{/each}}
        result = self.process_loops(&result, context)?;

        // 处理辅助函数 {{helper arg1 arg2}}
        result = self.process_helpers(&result, context)?;

        Ok(result)
    }

    fn register_helper(&mut self, name: &str, helper: Box<dyn TemplateHelper>) {
        self.helpers.insert(name.to_string(), helper);
    }

    fn validate_template(&self, template: &str) -> Result<()> {
        // 检查括号匹配
        let mut brace_count = 0;
        let mut chars = template.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '{' {
                if chars.peek() == Some(&'{') {
                    chars.next(); // 消费第二个 {
                    brace_count += 1;
                }
            } else if ch == '}' {
                if chars.peek() == Some(&'}') {
                    chars.next(); // 消费第二个 }
                    brace_count -= 1;
                }
            }
        }

        if brace_count != 0 {
            anyhow::bail!("Unmatched braces in template");
        }

        // 检查条件语句匹配
        let if_count = template.matches("{{#if").count();
        let endif_count = template.matches("{{/if}}").count();
        if if_count != endif_count {
            anyhow::bail!("Unmatched if statements in template");
        }

        // 检查循环语句匹配
        let each_count = template.matches("{{#each").count();
        let endeach_count = template.matches("{{/each}}").count();
        if each_count != endeach_count {
            anyhow::bail!("Unmatched each statements in template");
        }

        Ok(())
    }
}

impl SimpleTemplateEngine {
    /// 获取嵌套值
    fn get_nested_value(&self, context: &HashMap<String, Value>, path: &str) -> Value {
        let parts: Vec<&str> = path.split('.').collect();
        let mut current = context.get(parts[0]).unwrap_or(&Value::Null);

        for part in parts.iter().skip(1) {
            current = match current {
                Value::Object(obj) => obj.get(*part).unwrap_or(&Value::Null),
                Value::Array(arr) => {
                    if let Ok(index) = part.parse::<usize>() {
                        arr.get(index).unwrap_or(&Value::Null)
                    } else {
                        &Value::Null
                    }
                }
                _ => &Value::Null,
            };
        }

        current.clone()
    }

    /// 将值转换为字符串
    fn value_to_string(&self, value: &Value) -> String {
        match value {
            Value::String(s) => s.clone(),
            Value::Number(n) => n.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Null => String::new(),
            Value::Array(arr) => format!("[{} items]", arr.len()),
            Value::Object(_) => "[object]".to_string(),
        }
    }

    /// 处理条件语句
    fn process_conditionals(&self, template: &str, context: &HashMap<String, Value>) -> Result<String> {
        let if_regex = Regex::new(r"\{\{#if\s+([^}]+)\}\}(.*?)\{\{/if\}\}").unwrap();
        let mut result = template.to_string();

        while let Some(caps) = if_regex.captures(&result) {
            let condition = caps.get(1).unwrap().as_str().trim();
            let content = caps.get(2).unwrap().as_str();
            let full_match = caps.get(0).unwrap().as_str();

            let condition_value = self.get_nested_value(context, condition);
            let should_include = match condition_value {
                Value::Bool(b) => b,
                Value::Number(n) => n.as_f64().unwrap_or(0.0) != 0.0,
                Value::String(s) => !s.is_empty(),
                Value::Array(arr) => !arr.is_empty(),
                Value::Object(obj) => !obj.is_empty(),
                Value::Null => false,
            };

            let replacement = if should_include { content } else { "" };
            result = result.replace(full_match, replacement);
        }

        Ok(result)
    }

    /// 处理循环语句
    fn process_loops(&self, template: &str, context: &HashMap<String, Value>) -> Result<String> {
        let each_regex = Regex::new(r"\{\{#each\s+([^}]+)\}\}(.*?)\{\{/each\}\}").unwrap();
        let mut result = template.to_string();

        while let Some(caps) = each_regex.captures(&result) {
            let array_name = caps.get(1).unwrap().as_str().trim();
            let item_template = caps.get(2).unwrap().as_str();
            let full_match = caps.get(0).unwrap().as_str();

            let array_value = self.get_nested_value(context, array_name);
            let mut loop_result = String::new();

            if let Value::Array(items) = array_value {
                for (index, item) in items.iter().enumerate() {
                    let mut item_context = context.clone();
                    item_context.insert("this".to_string(), item.clone());
                    item_context.insert("@index".to_string(), Value::Number(index.into()));

                    let rendered_item = self.render_with_context(item_template, &item_context)?;
                    loop_result.push_str(&rendered_item);
                }
            }

            result = result.replace(full_match, &loop_result);
        }

        Ok(result)
    }

    /// 处理辅助函数
    fn process_helpers(&self, template: &str, _context: &HashMap<String, Value>) -> Result<String> {
        let helper_regex = Regex::new(r"\{\{([a-zA-Z_][a-zA-Z0-9_]*)\s+([^}]+)\}\}").unwrap();
        let mut result = template.to_string();

        for caps in helper_regex.captures_iter(template) {
            let helper_name = caps.get(1).unwrap().as_str();
            let args_str = caps.get(2).unwrap().as_str();
            let full_match = caps.get(0).unwrap().as_str();

            if let Some(helper) = self.helpers.get(helper_name) {
                let args = self.parse_helper_args(args_str)?;
                let helper_result = helper.call(&args)?;
                let replacement = self.value_to_string(&helper_result);
                result = result.replace(full_match, &replacement);
            }
        }

        Ok(result)
    }

    /// 解析辅助函数参数
    fn parse_helper_args(&self, args_str: &str) -> Result<Vec<Value>> {
        let mut args = Vec::new();
        let parts: Vec<&str> = args_str.split_whitespace().collect();

        for part in parts {
            if part.starts_with('"') && part.ends_with('"') {
                // 字符串参数
                let s = part.trim_matches('"');
                args.push(Value::String(s.to_string()));
            } else if let Ok(n) = part.parse::<f64>() {
                // 数字参数
                args.push(Value::Number(serde_json::Number::from_f64(n).unwrap_or_else(|| serde_json::Number::from(0))));
            } else if part == "true" {
                args.push(Value::Bool(true));
            } else if part == "false" {
                args.push(Value::Bool(false));
            } else {
                // 变量引用 - 这里简化处理，实际应该从上下文获取
                args.push(Value::String(part.to_string()));
            }
        }

        Ok(args)
    }

    /// 使用指定上下文渲染模板
    fn render_with_context(&self, template: &str, context: &HashMap<String, Value>) -> Result<String> {
        self.render(template, context)
    }
}

impl Default for SimpleTemplateEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_variable_substitution() {
        let engine = SimpleTemplateEngine::new();
        let mut context = HashMap::new();
        context.insert("name".to_string(), Value::String("Test".to_string()));
        context.insert("count".to_string(), Value::Number(42.into()));

        let template = "Hello {{name}}, you have {{count}} items.";
        let result = engine.render(template, &context).unwrap();
        assert_eq!(result, "Hello Test, you have 42 items.");
    }

    #[test]
    fn test_nested_properties() {
        let engine = SimpleTemplateEngine::new();
        let mut context = HashMap::new();
        let mut user = serde_json::Map::new();
        user.insert("name".to_string(), Value::String("John".to_string()));
        context.insert("user".to_string(), Value::Object(user));

        let template = "User: {{user.name}}";
        let result = engine.render(template, &context).unwrap();
        assert_eq!(result, "User: John");
    }

    #[test]
    fn test_conditional_rendering() {
        let engine = SimpleTemplateEngine::new();
        let mut context = HashMap::new();
        context.insert("show_details".to_string(), Value::Bool(true));

        let template = "{{#if show_details}}Details are shown{{/if}}";
        let result = engine.render(template, &context).unwrap();
        assert_eq!(result, "Details are shown");

        context.insert("show_details".to_string(), Value::Bool(false));
        let result = engine.render(template, &context).unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_template_validation() {
        let engine = SimpleTemplateEngine::new();

        // 有效模板
        assert!(engine.validate_template("{{name}}").is_ok());
        assert!(engine.validate_template("{{#if condition}}content{{/if}}").is_ok());

        // 无效模板
        assert!(engine.validate_template("{{name}").is_err());
        assert!(engine.validate_template("{{#if condition}}content").is_err());
    }
}