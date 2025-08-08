use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use handlebars::Handlebars;

use crate::notification::{NotificationMessage, NotificationPlatform, NotificationSeverity};

/// 模板引擎
pub struct TemplateEngine {
    handlebars: Handlebars<'static>,
    templates: HashMap<String, String>,
}

impl TemplateEngine {
    pub fn new() -> Self {
        let mut handlebars = Handlebars::new();

        // 注册辅助函数
        handlebars.register_helper("severity_color", Box::new(severity_color_helper));
        handlebars.register_helper("score_emoji", Box::new(score_emoji_helper));
        handlebars.register_helper("format_timestamp", Box::new(format_timestamp_helper));

        let mut engine = Self {
            handlebars,
            templates: HashMap::new(),
        };

        // 加载默认模板
        engine.load_default_templates();
        engine
    }

    /// 加载默认模板
    fn load_default_templates(&mut self) {
        // 飞书卡片模板
        let feishu_template = r#"
{
  "msg_type": "interactive",
  "card": {
    "config": {
      "wide_screen_mode": true,
      "enable_forward": true
    },
    "header": {
      "title": {
        "tag": "plain_text",
        "content": "{{score_emoji score}} {{title}}"
      },
      "template": "{{severity_color severity}}"
    },
    "elements": [
      {
        "tag": "div",
        "text": {
          "tag": "lark_md",
          "content": "**项目路径**: {{project_path}}\n**时间**: {{format_timestamp timestamp}}\n\n{{content}}"
        }
      }
      {{#if score}}
      ,{
        "tag": "div",
        "fields": [
          {
            "is_short": true,
            "text": {
              "tag": "lark_md",
              "content": "**质量评分**\n{{score}}/10"
            }
          }
        ]
      }
      {{/if}}
      {{#if metadata}}
      ,{
        "tag": "div",
        "text": {
          "tag": "lark_md",
          "content": "**详细信息**\n{{#each metadata}}{{@key}}: {{this}}\n{{/each}}"
        }
      }
      {{/if}}
    ]
  }
}
"#;

        // 微信文本模板
        let wechat_template = r#"
📊 **{{title}}**

🏷️ **项目**: {{project_path}}
⏰ **时间**: {{format_timestamp timestamp}}
{{#if score}}📈 **评分**: {{score}}/10{{/if}}

📝 **内容**:
{{content}}

{{#if metadata}}
📋 **详细信息**:
{{#each metadata}}• {{@key}}: {{this}}
{{/each}}
{{/if}}
"#;

        // 钉钉Markdown模板
        let dingtalk_template = r#"
# {{score_emoji score}} {{title}}

**项目路径**: {{project_path}}
**时间**: {{format_timestamp timestamp}}
{{#if score}}**质量评分**: {{score}}/10{{/if}}

## 详细内容

{{content}}

{{#if metadata}}
## 详细信息
{{#each metadata}}
- **{{@key}}**: {{this}}
{{/each}}
{{/if}}
"#;

        // 邮件HTML模板
        let email_template = r#"
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>{{title}}</title>
    <style>
        body { font-family: Arial, sans-serif; line-height: 1.6; color: #333; }
        .header { background-color: {{severity_color severity}}; color: white; padding: 20px; border-radius: 5px 5px 0 0; }
        .content { padding: 20px; border: 1px solid #ddd; border-top: none; border-radius: 0 0 5px 5px; }
        .metadata { background-color: #f9f9f9; padding: 15px; margin-top: 20px; border-radius: 5px; }
        .score { font-size: 24px; font-weight: bold; color: #2196F3; }
    </style>
</head>
<body>
    <div class="header">
        <h1>{{score_emoji score}} {{title}}</h1>
    </div>
    <div class="content">
        <p><strong>项目路径:</strong> {{project_path}}</p>
        <p><strong>时间:</strong> {{format_timestamp timestamp}}</p>
        {{#if score}}<p><strong>质量评分:</strong> <span class="score">{{score}}/10</span></p>{{/if}}

        <h2>详细内容</h2>
        <div>{{content}}</div>

        {{#if metadata}}
        <div class="metadata">
            <h3>详细信息</h3>
            <ul>
            {{#each metadata}}
                <li><strong>{{@key}}:</strong> {{this}}</li>
            {{/each}}
            </ul>
        </div>
        {{/if}}
    </div>
</body>
</html>
"#;

        self.register_template("feishu_default", feishu_template);
        self.register_template("wechat_default", wechat_template);
        self.register_template("dingtalk_default", dingtalk_template);
        self.register_template("email_default", email_template);
    }

    /// 注册模板
    pub fn register_template(&mut self, name: &str, template: &str) -> anyhow::Result<()> {
        self.handlebars.register_template_string(name, template)?;
        self.templates.insert(name.to_string(), template.to_string());
        Ok(())
    }

    /// 渲染模板
    pub fn render(&self, template_name: &str, message: &NotificationMessage) -> anyhow::Result<String> {
        let context = self.create_template_context(message);
        let rendered = self.handlebars.render(template_name, &context)?;
        Ok(rendered)
    }

    /// 获取平台默认模板名称
    pub fn get_default_template_name(&self, platform: &NotificationPlatform) -> String {
        match platform {
            NotificationPlatform::Feishu => "feishu_default".to_string(),
            NotificationPlatform::WeChat => "wechat_default".to_string(),
            NotificationPlatform::DingTalk => "dingtalk_default".to_string(),
            NotificationPlatform::Email => "email_default".to_string(),
        }
    }

    /// 创建模板上下文
    fn create_template_context(&self, message: &NotificationMessage) -> serde_json::Value {
        serde_json::json!({
            "id": message.id,
            "title": message.title,
            "content": message.content,
            "severity": message.severity,
            "project_path": message.project_path,
            "score": message.score,
            "timestamp": message.timestamp,
            "metadata": message.metadata,
            "template_data": message.template_data
        })
    }

    /// 列出所有模板
    pub fn list_templates(&self) -> Vec<String> {
        self.templates.keys().cloned().collect()
    }

    /// 检查模板是否存在
    pub fn has_template(&self, name: &str) -> bool {
        self.templates.contains_key(name)
    }
}

impl Default for TemplateEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// 严重程度颜色辅助函数
fn severity_color_helper(
    h: &handlebars::Helper,
    _: &Handlebars,
    _: &handlebars::Context,
    _: &mut handlebars::RenderContext,
    out: &mut dyn handlebars::Output,
) -> handlebars::HelperResult {
    let severity = h.param(0)
        .and_then(|v| v.value().as_str())
        .unwrap_or("Info");

    let color = match severity {
        "Critical" => "red",
        "Error" => "orange",
        "Warning" => "yellow",
        "Info" => "blue",
        _ => "grey",
    };

    out.write(color)?;
    Ok(())
}

/// 评分表情辅助函数
fn score_emoji_helper(
    h: &handlebars::Helper,
    _: &Handlebars,
    _: &handlebars::Context,
    _: &mut handlebars::RenderContext,
    out: &mut dyn handlebars::Output,
) -> handlebars::HelperResult {
    let score = h.param(0)
        .and_then(|v| v.value().as_f64())
        .unwrap_or(0.0);

    let emoji = match score {
        s if s >= 9.0 => "🎉",
        s if s >= 8.0 => "✅",
        s if s >= 7.0 => "👍",
        s if s >= 6.0 => "⚠️",
        s if s >= 5.0 => "❌",
        _ => "🚨",
    };

    out.write(emoji)?;
    Ok(())
}

/// 时间格式化辅助函数
fn format_timestamp_helper(
    h: &handlebars::Helper,
    _: &Handlebars,
    _: &handlebars::Context,
    _: &mut handlebars::RenderContext,
    out: &mut dyn handlebars::Output,
) -> handlebars::HelperResult {
    let timestamp = h.param(0)
        .and_then(|v| v.value().as_str())
        .unwrap_or("");

    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(timestamp) {
        let formatted = dt.format("%Y-%m-%d %H:%M:%S").to_string();
        out.write(&formatted)?;
    } else {
        out.write(timestamp)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_template_engine_creation() {
        let engine = TemplateEngine::new();
        assert!(engine.has_template("feishu_default"));
        assert!(engine.has_template("wechat_default"));
        assert!(engine.has_template("dingtalk_default"));
        assert!(engine.has_template("email_default"));
    }

    #[test]
    fn test_template_registration() {
        let mut engine = TemplateEngine::new();
        let template = "Hello {{title}}!";

        engine.register_template("test_template", template).unwrap();
        assert!(engine.has_template("test_template"));
    }

    #[test]
    fn test_template_rendering() {
        let engine = TemplateEngine::new();
        let message = NotificationMessage::new(
            "Test Title".to_string(),
            "Test Content".to_string(),
            NotificationSeverity::Info,
            "/test/project".to_string(),
        );

        let rendered = engine.render("wechat_default", &message).unwrap();
        assert!(rendered.contains("Test Title"));
        assert!(rendered.contains("Test Content"));
        assert!(rendered.contains("/test/project"));
    }

    #[test]
    fn test_default_template_names() {
        let engine = TemplateEngine::new();

        assert_eq!(engine.get_default_template_name(&NotificationPlatform::Feishu), "feishu_default");
        assert_eq!(engine.get_default_template_name(&NotificationPlatform::WeChat), "wechat_default");
        assert_eq!(engine.get_default_template_name(&NotificationPlatform::DingTalk), "dingtalk_default");
        assert_eq!(engine.get_default_template_name(&NotificationPlatform::Email), "email_default");
    }
}