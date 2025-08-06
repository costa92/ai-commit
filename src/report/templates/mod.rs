pub mod engine;
pub mod default_templates;

pub use engine::*;
pub use default_templates::*;

use anyhow::Result;
use serde_json::Value;
use std::collections::HashMap;

/// 模板引擎 trait
pub trait TemplateEngine: Send + Sync {
    /// 渲染模板
    fn render(&self, template: &str, context: &HashMap<String, Value>) -> Result<String>;

    /// 注册辅助函数
    fn register_helper(&mut self, name: &str, helper: Box<dyn TemplateHelper>);

    /// 验证模板语法
    fn validate_template(&self, template: &str) -> Result<()>;
}

/// 模板辅助函数 trait
pub trait TemplateHelper: Send + Sync {
    /// 执行辅助函数
    fn call(&self, args: &[Value]) -> Result<Value>;
}

/// 模板管理器
pub struct TemplateManager {
    engine: Box<dyn TemplateEngine>,
    templates: HashMap<String, String>,
}

impl TemplateManager {
    /// 创建新的模板管理器
    pub fn new(engine: Box<dyn TemplateEngine>) -> Self {
        let mut manager = Self {
            engine,
            templates: HashMap::new(),
        };

        // 注册默认模板
        manager.register_default_templates();

        // 注册默认辅助函数
        manager.register_default_helpers();

        manager
    }

    /// 注册模板
    pub fn register_template(&mut self, name: &str, template: &str) -> Result<()> {
        self.engine.validate_template(template)?;
        self.templates.insert(name.to_string(), template.to_string());
        Ok(())
    }

    /// 渲染模板
    pub fn render(&self, template_name: &str, context: &HashMap<String, Value>) -> Result<String> {
        let template = self.templates.get(template_name)
            .ok_or_else(|| anyhow::anyhow!("Template not found: {}", template_name))?;

        self.engine.render(template, context)
    }

    /// 获取可用模板列表
    pub fn list_templates(&self) -> Vec<String> {
        self.templates.keys().cloned().collect()
    }

    /// 注册默认模板
    fn register_default_templates(&mut self) {
        // Markdown 模板
        let _ = self.register_template("markdown_summary", MARKDOWN_SUMMARY_TEMPLATE);
        let _ = self.register_template("markdown_issues", MARKDOWN_ISSUES_TEMPLATE);
        let _ = self.register_template("markdown_statistics", MARKDOWN_STATISTICS_TEMPLATE);

        // JSON 模板
        let _ = self.register_template("json_full", JSON_FULL_TEMPLATE);

        // Text 模板
        let _ = self.register_template("text_summary", TEXT_SUMMARY_TEMPLATE);
        let _ = self.register_template("text_issues", TEXT_ISSUES_TEMPLATE);
    }

    /// 注册默认辅助函数
    fn register_default_helpers(&mut self) {
        self.engine.register_helper("format_duration", Box::new(FormatDurationHelper));
        self.engine.register_helper("format_percentage", Box::new(FormatPercentageHelper));
        self.engine.register_helper("severity_emoji", Box::new(SeverityEmojiHelper));
        self.engine.register_helper("risk_level_emoji", Box::new(RiskLevelEmojiHelper));
        self.engine.register_helper("progress_bar", Box::new(ProgressBarHelper));
        self.engine.register_helper("eq", Box::new(EqualityHelper));
        self.engine.register_helper("gt", Box::new(GreaterThanHelper));
        self.engine.register_helper("repeat", Box::new(RepeatHelper));
    }
}

/// 格式化持续时间辅助函数
struct FormatDurationHelper;

impl TemplateHelper for FormatDurationHelper {
    fn call(&self, args: &[Value]) -> Result<Value> {
        if args.len() != 1 {
            anyhow::bail!("format_duration expects 1 argument");
        }

        let seconds = args[0].as_u64()
            .ok_or_else(|| anyhow::anyhow!("format_duration expects a number"))?;

        let duration = std::time::Duration::from_secs(seconds);
        let formatted = crate::report::formatters::utils::format_duration(&duration);
        Ok(Value::String(formatted))
    }
}

/// 格式化百分比辅助函数
struct FormatPercentageHelper;

impl TemplateHelper for FormatPercentageHelper {
    fn call(&self, args: &[Value]) -> Result<Value> {
        if args.len() != 1 {
            anyhow::bail!("format_percentage expects 1 argument");
        }

        let value = args[0].as_f64()
            .ok_or_else(|| anyhow::anyhow!("format_percentage expects a number"))? as f32;

        let formatted = crate::report::formatters::utils::format_percentage(value);
        Ok(Value::String(formatted))
    }
}

/// 严重程度表情符号辅助函数
struct SeverityEmojiHelper;

impl TemplateHelper for SeverityEmojiHelper {
    fn call(&self, args: &[Value]) -> Result<Value> {
        if args.len() != 1 {
            anyhow::bail!("severity_emoji expects 1 argument");
        }

        let severity_str = args[0].as_str()
            .ok_or_else(|| anyhow::anyhow!("severity_emoji expects a string"))?;

        let emoji = match severity_str.to_lowercase().as_str() {
            "critical" => "🔴",
            "high" => "🟠",
            "medium" => "🟡",
            "low" => "🔵",
            "info" => "ℹ️",
            _ => "❓",
        };

        Ok(Value::String(emoji.to_string()))
    }
}

/// 进度条辅助函数
struct ProgressBarHelper;

impl TemplateHelper for ProgressBarHelper {
    fn call(&self, args: &[Value]) -> Result<Value> {
        if args.len() < 1 || args.len() > 2 {
            anyhow::bail!("progress_bar expects 1 or 2 arguments");
        }

        let value = args[0].as_f64()
            .ok_or_else(|| anyhow::anyhow!("progress_bar expects a number"))? as f32;

        let width = if args.len() > 1 {
            args[1].as_u64().unwrap_or(20) as usize
        } else {
            20
        };

        let progress_bar = crate::report::formatters::utils::generate_progress_bar(value, width);
        Ok(Value::String(progress_bar))
    }
}

/// 风险等级表情符号辅助函数
struct RiskLevelEmojiHelper;

impl TemplateHelper for RiskLevelEmojiHelper {
    fn call(&self, args: &[Value]) -> Result<Value> {
        if args.len() != 1 {
            anyhow::bail!("risk_level_emoji expects 1 argument");
        }

        let risk_level_str = args[0].as_str()
            .ok_or_else(|| anyhow::anyhow!("risk_level_emoji expects a string"))?;

        let emoji = match risk_level_str.to_lowercase().as_str() {
            "critical" => "🔴",
            "high" => "🟠",
            "medium" => "🟡",
            "low" => "🔵",
            _ => "❓",
        };

        Ok(Value::String(emoji.to_string()))
    }
}

/// 相等比较辅助函数
struct EqualityHelper;

impl TemplateHelper for EqualityHelper {
    fn call(&self, args: &[Value]) -> Result<Value> {
        if args.len() != 2 {
            anyhow::bail!("eq expects 2 arguments");
        }

        let result = match (&args[0], &args[1]) {
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Number(a), Value::Number(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            _ => false,
        };

        Ok(Value::Bool(result))
    }
}

/// 大于比较辅助函数
struct GreaterThanHelper;

impl TemplateHelper for GreaterThanHelper {
    fn call(&self, args: &[Value]) -> Result<Value> {
        if args.len() != 2 {
            anyhow::bail!("gt expects 2 arguments");
        }

        let result = match (&args[0], &args[1]) {
            (Value::Number(a), Value::Number(b)) => {
                a.as_f64().unwrap_or(0.0) > b.as_f64().unwrap_or(0.0)
            }
            _ => false,
        };

        Ok(Value::Bool(result))
    }
}

/// 重复字符串辅助函数
struct RepeatHelper;

impl TemplateHelper for RepeatHelper {
    fn call(&self, args: &[Value]) -> Result<Value> {
        if args.len() != 2 {
            anyhow::bail!("repeat expects 2 arguments");
        }

        let string = args[0].as_str()
            .ok_or_else(|| anyhow::anyhow!("repeat expects first argument to be a string"))?;

        let count = args[1].as_u64()
            .ok_or_else(|| anyhow::anyhow!("repeat expects second argument to be a number"))? as usize;

        Ok(Value::String(string.repeat(count)))
    }
}