use std::sync::Arc;
use std::collections::HashMap;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use sha2::{Sha256, Digest};
use base64::{Engine as _, engine::general_purpose};

use crate::notification::{
    NotificationProvider, NotificationMessage, NotificationResult, NotificationPlatform, NotificationSeverity
};

/// 飞书配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeishuConfig {
    pub webhook_url: String,
    pub secret: Option<String>,
    pub enable_interactive_cards: bool,
    pub enable_buttons: bool,
    pub timeout_seconds: u64,
}

impl Default for FeishuConfig {
    fn default() -> Self {
        Self {
            webhook_url: String::new(),
            secret: None,
            enable_interactive_cards: true,
            enable_buttons: true,
            timeout_seconds: 30,
        }
    }
}

/// 飞书通知提供商
pub struct FeishuProvider {
    config: FeishuConfig,
    client: Arc<reqwest::Client>,
}

impl FeishuProvider {
    pub fn new(config: FeishuConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_seconds))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        Self {
            config,
            client: Arc::new(client),
        }
    }

    /// 构建交互式卡片消息
    fn build_interactive_card(&self, message: &NotificationMessage) -> serde_json::Value {
        let color = self.get_severity_color(&message.severity);
        let score_emoji = self.get_score_emoji(message.score);

        let mut elements = Vec::new();

        // 主要内容区域
        elements.push(serde_json::json!({
            "tag": "div",
            "text": {
                "tag": "lark_md",
                "content": format!(
                    "**项目路径**: {}\n**时间**: {}\n\n{}",
                    message.project_path,
                    message.timestamp.format("%Y-%m-%d %H:%M:%S"),
                    message.content
                )
            }
        }));

        // 评分信息
        if let Some(score) = message.score {
            elements.push(serde_json::json!({
                "tag": "div",
                "fields": [
                    {
                        "is_short": true,
                        "text": {
                            "tag": "lark_md",
                            "content": format!("**质量评分**\n{:.1}/10", score)
                        }
                    },
                    {
                        "is_short": true,
                        "text": {
                            "tag": "lark_md",
                            "content": format!("**评级**\n{}", self.get_score_grade(score))
                        }
                    }
                ]
            }));
        }

        // 元数据信息
        if !message.metadata.is_empty() {
            let metadata_content = message.metadata.iter()
                .map(|(k, v)| format!("• **{}**: {}", k, v))
                .collect::<Vec<_>>()
                .join("\n");

            elements.push(serde_json::json!({
                "tag": "div",
                "text": {
                    "tag": "lark_md",
                    "content": format!("**详细信息**\n{}", metadata_content)
                }
            }));
        }

        // 分隔线
        elements.push(serde_json::json!({
            "tag": "hr"
        }));

        // 交互式按钮（如果启用）
        if self.config.enable_buttons {
            let mut actions = Vec::new();

            // 查看详情按钮
            if let Some(report_url) = message.template_data.get("report_url") {
                actions.push(serde_json::json!({
                    "tag": "button",
                    "text": {
                        "tag": "plain_text",
                        "content": "查看详细报告"
                    },
                    "type": "primary",
                    "url": report_url
                }));
            }

            // 项目链接按钮
            if let Some(project_url) = message.template_data.get("project_url") {
                actions.push(serde_json::json!({
                    "tag": "button",
                    "text": {
                        "tag": "plain_text",
                        "content": "打开项目"
                    },
                    "url": project_url
                }));
            }

            // 忽略通知按钮
            actions.push(serde_json::json!({
                "tag": "button",
                "text": {
                    "tag": "plain_text",
                    "content": "忽略"
                },
                "type": "default",
                "value": {
                    "action": "ignore",
                    "message_id": message.id
                }
            }));

            if !actions.is_empty() {
                elements.push(serde_json::json!({
                    "tag": "action",
                    "actions": actions
                }));
            }
        }

        serde_json::json!({
            "msg_type": "interactive",
            "card": {
                "config": {
                    "wide_screen_mode": true,
                    "enable_forward": true
                },
                "header": {
                    "title": {
                        "tag": "plain_text",
                        "content": format!("{} {}", score_emoji, message.title)
                    },
                    "template": color
                },
                "elements": elements
            }
        })
    }

    /// 构建简单文本消息
    fn build_text_message(&self, message: &NotificationMessage) -> serde_json::Value {
        let score_emoji = self.get_score_emoji(message.score);
        let score_text = message.score
            .map(|s| format!(" (评分: {:.1}/10)", s))
            .unwrap_or_default();

        let metadata_text = if message.metadata.is_empty() {
            String::new()
        } else {
            let metadata_lines = message.metadata.iter()
                .map(|(k, v)| format!("• {}: {}", k, v))
                .collect::<Vec<_>>()
                .join("\n");
            format!("\n\n详细信息:\n{}", metadata_lines)
        };

        let content = format!(
            "{} {}{}\n\n项目: {}\n时间: {}\n\n{}{}",
            score_emoji,
            message.title,
            score_text,
            message.project_path,
            message.timestamp.format("%Y-%m-%d %H:%M:%S"),
            message.content,
            metadata_text
        );

        serde_json::json!({
            "msg_type": "text",
            "content": {
                "text": content
            }
        })
    }

    /// 获取严重程度对应的颜色
    fn get_severity_color(&self, severity: &NotificationSeverity) -> &'static str {
        match severity {
            NotificationSeverity::Critical => "red",
            NotificationSeverity::Error => "orange",
            NotificationSeverity::Warning => "yellow",
            NotificationSeverity::Info => "blue",
        }
    }

    /// 获取评分对应的表情符号
    fn get_score_emoji(&self, score: Option<f32>) -> &'static str {
        match score {
            Some(s) if s >= 9.0 => "🎉",
            Some(s) if s >= 8.0 => "✅",
            Some(s) if s >= 7.0 => "👍",
            Some(s) if s >= 6.0 => "⚠️",
            Some(s) if s >= 5.0 => "❌",
            Some(_) => "🚨",
            None => "📊",
        }
    }

    /// 获取评分等级
    fn get_score_grade(&self, score: f32) -> &'static str {
        match score {
            s if s >= 9.0 => "优秀",
            s if s >= 8.0 => "良好",
            s if s >= 7.0 => "中等",
            s if s >= 6.0 => "及格",
            s if s >= 5.0 => "较差",
            _ => "很差",
        }
    }

    /// 生成签名（如果配置了密钥）
    fn generate_signature(&self, timestamp: i64, body: &str) -> Option<String> {
        if let Some(secret) = &self.config.secret {
            let string_to_sign = format!("{}\n{}", timestamp, secret);
            let mut hasher = Sha256::new();
            hasher.update(string_to_sign.as_bytes());
            let signature = hasher.finalize();
            Some(general_purpose::STANDARD.encode(signature))
        } else {
            None
        }
    }

    /// 验证webhook URL格式
    fn validate_webhook_url(&self) -> anyhow::Result<()> {
        if self.config.webhook_url.is_empty() {
            return Err(anyhow::anyhow!("Feishu webhook URL is empty"));
        }

        if !self.config.webhook_url.starts_with("https://") {
            return Err(anyhow::anyhow!("Feishu webhook URL must use HTTPS"));
        }

        if !self.config.webhook_url.contains("open.feishu.cn") &&
           !self.config.webhook_url.contains("open.larksuite.com") {
            return Err(anyhow::anyhow!("Invalid Feishu webhook URL domain"));
        }

        Ok(())
    }
}

#[async_trait]
impl NotificationProvider for FeishuProvider {
    fn platform(&self) -> NotificationPlatform {
        NotificationPlatform::Feishu
    }

    async fn send_notification(&self, message: &NotificationMessage) -> anyhow::Result<NotificationResult> {
        // 验证配置
        self.validate_webhook_url()?;

        // 构建消息体
        let payload = if self.config.enable_interactive_cards {
            self.build_interactive_card(message)
        } else {
            self.build_text_message(message)
        };

        let body = serde_json::to_string(&payload)?;
        let timestamp = Utc::now().timestamp();

        // 构建请求
        let mut request_builder = self.client
            .post(&self.config.webhook_url)
            .header("Content-Type", "application/json")
            .body(body.clone());

        // 添加签名头（如果配置了密钥）
        if let Some(signature) = self.generate_signature(timestamp, &body) {
            request_builder = request_builder
                .header("X-Lark-Request-Timestamp", timestamp.to_string())
                .header("X-Lark-Request-Nonce", uuid::Uuid::new_v4().to_string())
                .header("X-Lark-Signature", signature);
        }

        // 发送请求
        let response = request_builder.send().await?;
        let status = response.status();

        if status.is_success() {
            let response_text = response.text().await?;

            // 解析响应以检查是否真正成功
            if let Ok(response_json) = serde_json::from_str::<serde_json::Value>(&response_text) {
                if let Some(code) = response_json.get("code").and_then(|c| c.as_i64()) {
                    if code != 0 {
                        let msg = response_json.get("msg")
                            .and_then(|m| m.as_str())
                            .unwrap_or("Unknown error");
                        return Ok(NotificationResult::failure(
                            message.id.clone(),
                            NotificationPlatform::Feishu,
                            format!("Feishu API error: {} (code: {})", msg, code),
                            0,
                        ));
                    }
                }
            }

            log::info!("Successfully sent Feishu notification for message: {}", message.id);
            Ok(NotificationResult::success(
                message.id.clone(),
                NotificationPlatform::Feishu,
            ))
        } else {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            let error_msg = format!("HTTP {}: {}", status, error_text);

            log::error!("Failed to send Feishu notification: {}", error_msg);
            Ok(NotificationResult::failure(
                message.id.clone(),
                NotificationPlatform::Feishu,
                error_msg,
                0,
            ))
        }
    }

    fn is_configured(&self) -> bool {
        !self.config.webhook_url.is_empty() && self.validate_webhook_url().is_ok()
    }

    fn supports_rich_content(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn create_test_config() -> FeishuConfig {
        FeishuConfig {
            webhook_url: "https://open.feishu.cn/open-apis/bot/v2/hook/test".to_string(),
            secret: Some("test_secret".to_string()),
            enable_interactive_cards: true,
            enable_buttons: true,
            timeout_seconds: 30,
        }
    }

    fn create_test_message() -> NotificationMessage {
        let mut message = NotificationMessage::new(
            "代码审查完成".to_string(),
            "发现了一些需要注意的问题，请查看详细报告。".to_string(),
            NotificationSeverity::Warning,
            "/test/project".to_string(),
        );

        message = message.with_score(7.5);
        message = message.with_metadata("issues_count".to_string(), "5".to_string());
        message = message.with_metadata("files_analyzed".to_string(), "23".to_string());
        message = message.with_template_data("report_url".to_string(),
            serde_json::Value::String("https://example.com/report/123".to_string()));
        message = message.with_template_data("project_url".to_string(),
            serde_json::Value::String("https://github.com/example/project".to_string()));

        message
    }

    #[test]
    fn test_feishu_provider_creation() {
        let config = create_test_config();
        let provider = FeishuProvider::new(config.clone());

        assert_eq!(provider.platform(), NotificationPlatform::Feishu);
        assert!(provider.is_configured());
        assert!(provider.supports_rich_content());
    }

    #[test]
    fn test_feishu_provider_not_configured() {
        let config = FeishuConfig::default();
        let provider = FeishuProvider::new(config);

        assert!(!provider.is_configured());
    }

    #[test]
    fn test_invalid_webhook_url() {
        let mut config = create_test_config();
        config.webhook_url = "http://invalid.com/webhook".to_string();
        let provider = FeishuProvider::new(config);

        assert!(!provider.is_configured());
    }

    #[test]
    fn test_build_interactive_card() {
        let config = create_test_config();
        let provider = FeishuProvider::new(config);
        let message = create_test_message();

        let card = provider.build_interactive_card(&message);

        // 验证卡片结构
        assert_eq!(card["msg_type"], "interactive");
        assert!(card["card"]["config"]["wide_screen_mode"].as_bool().unwrap());
        assert!(card["card"]["config"]["enable_forward"].as_bool().unwrap());

        // 验证标题
        let title = card["card"]["header"]["title"]["content"].as_str().unwrap();
        assert!(title.contains("代码审查完成"));
        assert!(title.contains("👍")); // 7.5分对应的emoji

        // 验证颜色
        assert_eq!(card["card"]["header"]["template"], "yellow");

        // 验证元素
        let elements = card["card"]["elements"].as_array().unwrap();
        assert!(!elements.is_empty());

        // 验证按钮
        let action_element = elements.iter()
            .find(|e| e["tag"] == "action")
            .expect("Should have action element");
        let actions = action_element["actions"].as_array().unwrap();
        assert!(!actions.is_empty());
    }

    #[test]
    fn test_build_text_message() {
        let mut config = create_test_config();
        config.enable_interactive_cards = false;
        let provider = FeishuProvider::new(config);
        let message = create_test_message();

        let text_msg = provider.build_text_message(&message);

        assert_eq!(text_msg["msg_type"], "text");
        let content = text_msg["content"]["text"].as_str().unwrap();
        assert!(content.contains("代码审查完成"));
        assert!(content.contains("7.5/10"));
        assert!(content.contains("/test/project"));
        assert!(content.contains("issues_count: 5"));
    }

    #[test]
    fn test_severity_colors() {
        let config = create_test_config();
        let provider = FeishuProvider::new(config);

        assert_eq!(provider.get_severity_color(&NotificationSeverity::Critical), "red");
        assert_eq!(provider.get_severity_color(&NotificationSeverity::Error), "orange");
        assert_eq!(provider.get_severity_color(&NotificationSeverity::Warning), "yellow");
        assert_eq!(provider.get_severity_color(&NotificationSeverity::Info), "blue");
    }

    #[test]
    fn test_score_emojis() {
        let config = create_test_config();
        let provider = FeishuProvider::new(config);

        assert_eq!(provider.get_score_emoji(Some(9.5)), "🎉");
        assert_eq!(provider.get_score_emoji(Some(8.5)), "✅");
        assert_eq!(provider.get_score_emoji(Some(7.5)), "👍");
        assert_eq!(provider.get_score_emoji(Some(6.5)), "⚠️");
        assert_eq!(provider.get_score_emoji(Some(5.5)), "❌");
        assert_eq!(provider.get_score_emoji(Some(3.0)), "🚨");
        assert_eq!(provider.get_score_emoji(None), "📊");
    }

    #[test]
    fn test_score_grades() {
        let config = create_test_config();
        let provider = FeishuProvider::new(config);

        assert_eq!(provider.get_score_grade(9.5), "优秀");
        assert_eq!(provider.get_score_grade(8.5), "良好");
        assert_eq!(provider.get_score_grade(7.5), "中等");
        assert_eq!(provider.get_score_grade(6.5), "及格");
        assert_eq!(provider.get_score_grade(5.5), "较差");
        assert_eq!(provider.get_score_grade(3.0), "很差");
    }

    #[test]
    fn test_signature_generation() {
        let config = create_test_config();
        let provider = FeishuProvider::new(config);

        let signature = provider.generate_signature(1234567890, "test body");
        assert!(signature.is_some());
        assert!(!signature.unwrap().is_empty());

        // 测试无密钥情况
        let mut config_no_secret = create_test_config();
        config_no_secret.secret = None;
        let provider_no_secret = FeishuProvider::new(config_no_secret);

        let signature_none = provider_no_secret.generate_signature(1234567890, "test body");
        assert!(signature_none.is_none());
    }

    #[test]
    fn test_webhook_url_validation() {
        let config = create_test_config();
        let provider = FeishuProvider::new(config);
        assert!(provider.validate_webhook_url().is_ok());

        // 测试空URL
        let mut config_empty = create_test_config();
        config_empty.webhook_url = String::new();
        let provider_empty = FeishuProvider::new(config_empty);
        assert!(provider_empty.validate_webhook_url().is_err());

        // 测试HTTP URL
        let mut config_http = create_test_config();
        config_http.webhook_url = "http://open.feishu.cn/webhook".to_string();
        let provider_http = FeishuProvider::new(config_http);
        assert!(provider_http.validate_webhook_url().is_err());

        // 测试无效域名
        let mut config_invalid = create_test_config();
        config_invalid.webhook_url = "https://invalid.com/webhook".to_string();
        let provider_invalid = FeishuProvider::new(config_invalid);
        assert!(provider_invalid.validate_webhook_url().is_err());
    }
}