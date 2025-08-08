use std::sync::Arc;
use std::collections::HashMap;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

use crate::notification::{
    NotificationProvider, NotificationMessage, NotificationResult, NotificationPlatform, NotificationSeverity
};

/// 微信企业版配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeChatConfig {
    pub webhook_url: String,
    pub corp_id: Option<String>,
    pub corp_secret: Option<String>,
    pub agent_id: Option<String>,
    pub enable_markdown: bool,
    pub enable_mentions: bool,
    pub timeout_seconds: u64,
    pub max_content_length: usize,
}

impl Default for WeChatConfig {
    fn default() -> Self {
        Self {
            webhook_url: String::new(),
            corp_id: None,
            corp_secret: None,
            agent_id: None,
            enable_markdown: true,
            enable_mentions: true,
            timeout_seconds: 30,
            max_content_length: 4096,
        }
    }
}

/// 微信消息类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WeChatMessageType {
    Text,
    Markdown,
    Image,
    News,
    File,
    TextCard,
    TemplateCard,
}

/// 微信文本消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeChatTextMessage {
    pub content: String,
    pub mentioned_list: Option<Vec<String>>,
    pub mentioned_mobile_list: Option<Vec<String>>,
}

/// 微信Markdown消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeChatMarkdownMessage {
    pub content: String,
}

/// 微信图文消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeChatNewsMessage {
    pub articles: Vec<WeChatArticle>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeChatArticle {
    pub title: String,
    pub description: String,
    pub url: String,
    pub picurl: Option<String>,
}

/// 微信模板卡片消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeChatTemplateCardMessage {
    pub card_type: String,
    pub source: WeChatCardSource,
    pub main_title: WeChatCardTitle,
    pub emphasis_content: Option<WeChatEmphasisContent>,
    pub quote_area: Option<WeChatQuoteArea>,
    pub sub_title_text: Option<String>,
    pub horizontal_content_list: Option<Vec<WeChatHorizontalContent>>,
    pub jump_list: Option<Vec<WeChatJumpElement>>,
    pub card_action: Option<WeChatCardAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeChatCardSource {
    pub icon_url: Option<String>,
    pub desc: String,
    pub desc_color: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeChatCardTitle {
    pub title: String,
    pub desc: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeChatEmphasisContent {
    pub title: String,
    pub desc: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeChatQuoteArea {
    pub r#type: u32,
    pub url: Option<String>,
    pub appid: Option<String>,
    pub pagepath: Option<String>,
    pub title: Option<String>,
    pub quote_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeChatHorizontalContent {
    pub keyname: String,
    pub value: String,
    pub r#type: Option<u32>,
    pub url: Option<String>,
    pub media_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeChatJumpElement {
    pub r#type: u32,
    pub title: String,
    pub url: Option<String>,
    pub appid: Option<String>,
    pub pagepath: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeChatCardAction {
    pub r#type: u32,
    pub url: Option<String>,
    pub appid: Option<String>,
    pub pagepath: Option<String>,
}

/// 微信企业版通知提供商
pub struct WeChatProvider {
    config: WeChatConfig,
    client: Arc<reqwest::Client>,
}

impl WeChatProvider {
    pub fn new(config: WeChatConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_seconds))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        Self {
            config,
            client: Arc::new(client),
        }
    }

    /// 构建文本消息
    pub fn build_text_message(&self, message: &NotificationMessage) -> serde_json::Value {
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

        // 截断过长的内容（安全处理UTF-8字符边界）
        let truncated_content = if content.len() > self.config.max_content_length {
            let truncate_at = self.config.max_content_length.saturating_sub(50);
            let safe_truncate_at = self.find_safe_truncate_point(&content, truncate_at);
            format!("{}...\n\n[内容过长，已截断]", &content[..safe_truncate_at])
        } else {
            content
        };

        let mut text_message = WeChatTextMessage {
            content: truncated_content,
            mentioned_list: None,
            mentioned_mobile_list: None,
        };

        // 添加@提醒（如果启用）
        if self.config.enable_mentions {
            if let Some(mentions) = message.template_data.get("mentions") {
                if let Some(mention_list) = mentions.as_array() {
                    let mentions: Vec<String> = mention_list.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect();
                    if !mentions.is_empty() {
                        text_message.mentioned_list = Some(mentions);
                    }
                }
            }

            if let Some(mobile_mentions) = message.template_data.get("mobile_mentions") {
                if let Some(mobile_list) = mobile_mentions.as_array() {
                    let mobiles: Vec<String> = mobile_list.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect();
                    if !mobiles.is_empty() {
                        text_message.mentioned_mobile_list = Some(mobiles);
                    }
                }
            }
        }

        serde_json::json!({
            "msgtype": "text",
            "text": text_message
        })
    }

    /// 构建Markdown消息
    pub fn build_markdown_message(&self, message: &NotificationMessage) -> serde_json::Value {
        let score_emoji = self.get_score_emoji(message.score);
        let severity_color = self.get_severity_color(&message.severity);

        let mut markdown_content = format!(
            "# {} {}\n\n",
            score_emoji,
            message.title
        );

        // 基本信息
        markdown_content.push_str(&format!(
            "**项目路径**: `{}`\n**时间**: {}\n**严重程度**: <font color=\"{}\">{}</font>\n\n",
            message.project_path,
            message.timestamp.format("%Y-%m-%d %H:%M:%S"),
            severity_color,
            self.get_severity_text(&message.severity)
        ));

        // 评分信息
        if let Some(score) = message.score {
            markdown_content.push_str(&format!(
                "**质量评分**: {:.1}/10 ({})\n\n",
                score,
                self.get_score_grade(score)
            ));
        }

        // 主要内容
        markdown_content.push_str("## 详细信息\n\n");
        markdown_content.push_str(&message.content);
        markdown_content.push_str("\n\n");

        // 元数据信息
        if !message.metadata.is_empty() {
            markdown_content.push_str("## 统计信息\n\n");
            for (key, value) in &message.metadata {
                markdown_content.push_str(&format!("- **{}**: {}\n", key, value));
            }
            markdown_content.push_str("\n");
        }

        // 添加链接
        if let Some(report_url) = message.template_data.get("report_url") {
            if let Some(url) = report_url.as_str() {
                markdown_content.push_str(&format!("[查看详细报告]({})\n", url));
            }
        }

        if let Some(project_url) = message.template_data.get("project_url") {
            if let Some(url) = project_url.as_str() {
                markdown_content.push_str(&format!("[打开项目]({})\n", url));
            }
        }

        // 截断过长的内容（安全处理UTF-8字符边界）
        let truncated_content = if markdown_content.len() > self.config.max_content_length {
            let truncate_at = self.config.max_content_length.saturating_sub(100);
            let safe_truncate_at = self.find_safe_truncate_point(&markdown_content, truncate_at);
            format!("{}...\n\n---\n*内容过长，已截断*", &markdown_content[..safe_truncate_at])
        } else {
            markdown_content
        };

        serde_json::json!({
            "msgtype": "markdown",
            "markdown": WeChatMarkdownMessage {
                content: truncated_content
            }
        })
    }

    /// 构建模板卡片消息
    pub fn build_template_card_message(&self, message: &NotificationMessage) -> serde_json::Value {
        let score_emoji = self.get_score_emoji(message.score);
        let severity_color_code = self.get_severity_color_code(&message.severity);

        let mut horizontal_content = Vec::new();

        // 添加基本信息
        horizontal_content.push(WeChatHorizontalContent {
            keyname: "项目路径".to_string(),
            value: message.project_path.clone(),
            r#type: None,
            url: message.template_data.get("project_url")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            media_id: None,
        });

        horizontal_content.push(WeChatHorizontalContent {
            keyname: "检测时间".to_string(),
            value: message.timestamp.format("%Y-%m-%d %H:%M:%S").to_string(),
            r#type: None,
            url: None,
            media_id: None,
        });

        // 添加评分信息
        if let Some(score) = message.score {
            horizontal_content.push(WeChatHorizontalContent {
                keyname: "质量评分".to_string(),
                value: format!("{:.1}/10 ({})", score, self.get_score_grade(score)),
                r#type: None,
                url: None,
                media_id: None,
            });
        }

        // 添加元数据
        for (key, value) in &message.metadata {
            horizontal_content.push(WeChatHorizontalContent {
                keyname: key.clone(),
                value: value.clone(),
                r#type: None,
                url: None,
                media_id: None,
            });
        }

        // 构建跳转链接
        let mut jump_list = Vec::new();
        if let Some(report_url) = message.template_data.get("report_url") {
            if let Some(url) = report_url.as_str() {
                jump_list.push(WeChatJumpElement {
                    r#type: 1, // URL类型
                    title: "查看详细报告".to_string(),
                    url: Some(url.to_string()),
                    appid: None,
                    pagepath: None,
                });
            }
        }

        if let Some(project_url) = message.template_data.get("project_url") {
            if let Some(url) = project_url.as_str() {
                jump_list.push(WeChatJumpElement {
                    r#type: 1, // URL类型
                    title: "打开项目".to_string(),
                    url: Some(url.to_string()),
                    appid: None,
                    pagepath: None,
                });
            }
        }

        let template_card = WeChatTemplateCardMessage {
            card_type: "text_notice".to_string(),
            source: WeChatCardSource {
                icon_url: Some("https://wework.qpic.cn/wwpic/252813_jOfDHtcISzuodLa_1629280209/0".to_string()),
                desc: "AI-Commit 代码审查系统".to_string(),
                desc_color: Some(0x000000),
            },
            main_title: WeChatCardTitle {
                title: format!("{} {}", score_emoji, message.title),
                desc: Some(format!("严重程度: {}", self.get_severity_text(&message.severity))),
            },
            emphasis_content: message.score.map(|score| WeChatEmphasisContent {
                title: format!("{:.1}", score),
                desc: Some("质量评分".to_string()),
            }),
            quote_area: Some(WeChatQuoteArea {
                r#type: 0, // 文本类型
                url: None,
                appid: None,
                pagepath: None,
                title: None,
                quote_text: message.content.clone(),
            }),
            sub_title_text: Some(format!("项目: {}", message.project_path)),
            horizontal_content_list: if horizontal_content.is_empty() { None } else { Some(horizontal_content) },
            jump_list: if jump_list.is_empty() { None } else { Some(jump_list) },
            card_action: message.template_data.get("report_url")
                .and_then(|v| v.as_str())
                .map(|url| WeChatCardAction {
                    r#type: 1, // URL类型
                    url: Some(url.to_string()),
                    appid: None,
                    pagepath: None,
                }),
        };

        serde_json::json!({
            "msgtype": "template_card",
            "template_card": template_card
        })
    }

    /// 获取严重程度对应的颜色
    fn get_severity_color(&self, severity: &NotificationSeverity) -> &'static str {
        match severity {
            NotificationSeverity::Critical => "#FF0000",
            NotificationSeverity::Error => "#FF6600",
            NotificationSeverity::Warning => "#FFAA00",
            NotificationSeverity::Info => "#0066FF",
        }
    }

    /// 获取严重程度对应的颜色代码
    fn get_severity_color_code(&self, severity: &NotificationSeverity) -> u32 {
        match severity {
            NotificationSeverity::Critical => 0xFF0000,
            NotificationSeverity::Error => 0xFF6600,
            NotificationSeverity::Warning => 0xFFAA00,
            NotificationSeverity::Info => 0x0066FF,
        }
    }

    /// 获取严重程度文本
    fn get_severity_text(&self, severity: &NotificationSeverity) -> &'static str {
        match severity {
            NotificationSeverity::Critical => "严重",
            NotificationSeverity::Error => "错误",
            NotificationSeverity::Warning => "警告",
            NotificationSeverity::Info => "信息",
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

    /// 验证webhook URL格式
    fn validate_webhook_url(&self) -> anyhow::Result<()> {
        if self.config.webhook_url.is_empty() {
            return Err(anyhow::anyhow!("WeChat webhook URL is empty"));
        }

        if !self.config.webhook_url.starts_with("https://") {
            return Err(anyhow::anyhow!("WeChat webhook URL must use HTTPS"));
        }

        if !self.config.webhook_url.contains("qyapi.weixin.qq.com") {
            return Err(anyhow::anyhow!("Invalid WeChat webhook URL domain"));
        }

        Ok(())
    }

    /// 找到安全的截断点，避免在UTF-8字符中间截断
    fn find_safe_truncate_point(&self, content: &str, max_len: usize) -> usize {
        if content.len() <= max_len {
            return content.len();
        }

        // 从目标位置向前查找，找到一个安全的UTF-8字符边界
        let mut truncate_at = max_len;
        while truncate_at > 0 && !content.is_char_boundary(truncate_at) {
            truncate_at -= 1;
        }

        // 如果找不到合适的边界，至少保证不为0
        if truncate_at == 0 && max_len > 0 {
            // 找到第一个字符边界
            for i in 1..=max_len.min(content.len()) {
                if content.is_char_boundary(i) {
                    truncate_at = i;
                    break;
                }
            }
        }

        truncate_at
    }

    /// 选择消息格式
    pub fn choose_message_format(&self, message: &NotificationMessage) -> serde_json::Value {
        // 如果有模板数据且支持富文本，优先使用模板卡片
        if !message.template_data.is_empty() && self.supports_rich_content() {
            return self.build_template_card_message(message);
        }

        // 如果启用Markdown且内容不太长，使用Markdown
        if self.config.enable_markdown && message.content.len() < 2000 {
            return self.build_markdown_message(message);
        }

        // 默认使用文本消息
        self.build_text_message(message)
    }
}

#[async_trait]
impl NotificationProvider for WeChatProvider {
    fn platform(&self) -> NotificationPlatform {
        NotificationPlatform::WeChat
    }

    async fn send_notification(&self, message: &NotificationMessage) -> anyhow::Result<NotificationResult> {
        // 验证配置
        self.validate_webhook_url()?;

        // 选择合适的消息格式
        let payload = self.choose_message_format(message);
        let body = serde_json::to_string(&payload)?;

        // 发送请求
        let response = self.client
            .post(&self.config.webhook_url)
            .header("Content-Type", "application/json")
            .body(body)
            .send()
            .await?;

        let status = response.status();

        if status.is_success() {
            let response_text = response.text().await?;

            // 解析响应以检查是否真正成功
            if let Ok(response_json) = serde_json::from_str::<serde_json::Value>(&response_text) {
                if let Some(errcode) = response_json.get("errcode").and_then(|c| c.as_i64()) {
                    if errcode != 0 {
                        let errmsg = response_json.get("errmsg")
                            .and_then(|m| m.as_str())
                            .unwrap_or("Unknown error");
                        return Ok(NotificationResult::failure(
                            message.id.clone(),
                            NotificationPlatform::WeChat,
                            format!("WeChat API error: {} (errcode: {})", errmsg, errcode),
                            0,
                        ));
                    }
                }
            }

            log::info!("Successfully sent WeChat notification for message: {}", message.id);
            Ok(NotificationResult::success(
                message.id.clone(),
                NotificationPlatform::WeChat,
            ))
        } else {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            let error_msg = format!("HTTP {}: {}", status, error_text);

            log::error!("Failed to send WeChat notification: {}", error_msg);
            Ok(NotificationResult::failure(
                message.id.clone(),
                NotificationPlatform::WeChat,
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
#[
cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn create_test_config() -> WeChatConfig {
        WeChatConfig {
            webhook_url: "https://qyapi.weixin.qq.com/cgi-bin/webhook/send?key=test".to_string(),
            corp_id: Some("test_corp_id".to_string()),
            corp_secret: Some("test_corp_secret".to_string()),
            agent_id: Some("1000001".to_string()),
            enable_markdown: true,
            enable_mentions: true,
            timeout_seconds: 30,
            max_content_length: 4096,
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

    fn create_message_with_mentions() -> NotificationMessage {
        let mut message = create_test_message();
        message = message.with_template_data("mentions".to_string(),
            serde_json::Value::Array(vec![
                serde_json::Value::String("@all".to_string()),
                serde_json::Value::String("user1".to_string()),
            ]));
        message = message.with_template_data("mobile_mentions".to_string(),
            serde_json::Value::Array(vec![
                serde_json::Value::String("13800138000".to_string()),
            ]));
        message
    }

    #[test]
    fn test_wechat_provider_creation() {
        let config = create_test_config();
        let provider = WeChatProvider::new(config.clone());

        assert_eq!(provider.platform(), NotificationPlatform::WeChat);
        assert!(provider.is_configured());
        assert!(provider.supports_rich_content());
    }

    #[test]
    fn test_wechat_provider_not_configured() {
        let config = WeChatConfig::default();
        let provider = WeChatProvider::new(config);

        assert!(!provider.is_configured());
    }

    #[test]
    fn test_invalid_webhook_url() {
        let mut config = create_test_config();
        config.webhook_url = "http://invalid.com/webhook".to_string();
        let provider = WeChatProvider::new(config);

        assert!(!provider.is_configured());
    }

    #[test]
    fn test_build_text_message() {
        let config = create_test_config();
        let provider = WeChatProvider::new(config);
        let message = create_test_message();

        let text_msg = provider.build_text_message(&message);

        assert_eq!(text_msg["msgtype"], "text");
        let text_content = text_msg["text"]["content"].as_str().unwrap();
        assert!(text_content.contains("代码审查完成"));
        assert!(text_content.contains("7.5/10"));
        assert!(text_content.contains("/test/project"));
        assert!(text_content.contains("issues_count: 5"));
        assert!(text_content.contains("👍")); // 7.5分对应的emoji
    }

    #[test]
    fn test_build_text_message_with_mentions() {
        let config = create_test_config();
        let provider = WeChatProvider::new(config);
        let message = create_message_with_mentions();

        let text_msg = provider.build_text_message(&message);

        assert_eq!(text_msg["msgtype"], "text");

        let mentioned_list = text_msg["text"]["mentioned_list"].as_array().unwrap();
        assert_eq!(mentioned_list.len(), 2);
        assert_eq!(mentioned_list[0], "@all");
        assert_eq!(mentioned_list[1], "user1");

        let mobile_list = text_msg["text"]["mentioned_mobile_list"].as_array().unwrap();
        assert_eq!(mobile_list.len(), 1);
        assert_eq!(mobile_list[0], "13800138000");
    }

    #[test]
    fn test_build_markdown_message() {
        let config = create_test_config();
        let provider = WeChatProvider::new(config);
        let message = create_test_message();

        let markdown_msg = provider.build_markdown_message(&message);

        assert_eq!(markdown_msg["msgtype"], "markdown");
        let markdown_content = markdown_msg["markdown"]["content"].as_str().unwrap();
        assert!(markdown_content.contains("# 👍 代码审查完成"));
        assert!(markdown_content.contains("**项目路径**: `/test/project`"));
        assert!(markdown_content.contains("**质量评分**: 7.5/10 (中等)"));
        assert!(markdown_content.contains("**严重程度**: <font color=\"#FFAA00\">警告</font>"));
        assert!(markdown_content.contains("[查看详细报告](https://example.com/report/123)"));
        assert!(markdown_content.contains("[打开项目](https://github.com/example/project)"));
    }

    #[test]
    fn test_build_template_card_message() {
        let config = create_test_config();
        let provider = WeChatProvider::new(config);
        let message = create_test_message();

        let card_msg = provider.build_template_card_message(&message);

        assert_eq!(card_msg["msgtype"], "template_card");

        let template_card = &card_msg["template_card"];
        assert_eq!(template_card["card_type"], "text_notice");

        // 验证标题
        let main_title = &template_card["main_title"];
        assert!(main_title["title"].as_str().unwrap().contains("👍 代码审查完成"));
        assert!(main_title["desc"].as_str().unwrap().contains("严重程度: 警告"));

        // 验证评分强调内容
        let emphasis_content = &template_card["emphasis_content"];
        assert_eq!(emphasis_content["title"], "7.5");
        assert_eq!(emphasis_content["desc"], "质量评分");

        // 验证引用区域
        let quote_area = &template_card["quote_area"];
        assert_eq!(quote_area["type"], 0);
        assert!(quote_area["quote_text"].as_str().unwrap().contains("发现了一些需要注意的问题"));

        // 验证水平内容列表
        let horizontal_content = template_card["horizontal_content_list"].as_array().unwrap();
        assert!(!horizontal_content.is_empty());

        // 验证项目路径
        let project_item = &horizontal_content[0];
        assert_eq!(project_item["keyname"], "项目路径");
        assert_eq!(project_item["value"], "/test/project");

        // 验证跳转链接
        let jump_list = template_card["jump_list"].as_array().unwrap();
        assert_eq!(jump_list.len(), 2);

        let report_jump = &jump_list[0];
        assert_eq!(report_jump["title"], "查看详细报告");
        assert_eq!(report_jump["url"], "https://example.com/report/123");

        let project_jump = &jump_list[1];
        assert_eq!(project_jump["title"], "打开项目");
        assert_eq!(project_jump["url"], "https://github.com/example/project");
    }

    #[test]
    fn test_content_truncation() {
        let mut config = create_test_config();
        config.max_content_length = 100;
        let provider = WeChatProvider::new(config);

        let mut message = create_test_message();
        message.content = "这是一个非常长的内容".repeat(20); // 创建超长内容

        let text_msg = provider.build_text_message(&message);
        let content = text_msg["text"]["content"].as_str().unwrap();

        assert!(content.len() <= 100);
        assert!(content.contains("[内容过长，已截断]"));
    }

    #[test]
    fn test_markdown_content_truncation() {
        let mut config = create_test_config();
        config.max_content_length = 200;
        let provider = WeChatProvider::new(config);

        let mut message = create_test_message();
        message.content = "这是一个非常长的内容".repeat(20); // 创建超长内容

        let markdown_msg = provider.build_markdown_message(&message);
        let content = markdown_msg["markdown"]["content"].as_str().unwrap();

        assert!(content.len() <= 200);
        assert!(content.contains("*内容过长，已截断*"));
    }

    #[test]
    fn test_severity_colors() {
        let config = create_test_config();
        let provider = WeChatProvider::new(config);

        assert_eq!(provider.get_severity_color(&NotificationSeverity::Critical), "#FF0000");
        assert_eq!(provider.get_severity_color(&NotificationSeverity::Error), "#FF6600");
        assert_eq!(provider.get_severity_color(&NotificationSeverity::Warning), "#FFAA00");
        assert_eq!(provider.get_severity_color(&NotificationSeverity::Info), "#0066FF");
    }

    #[test]
    fn test_severity_color_codes() {
        let config = create_test_config();
        let provider = WeChatProvider::new(config);

        assert_eq!(provider.get_severity_color_code(&NotificationSeverity::Critical), 0xFF0000);
        assert_eq!(provider.get_severity_color_code(&NotificationSeverity::Error), 0xFF6600);
        assert_eq!(provider.get_severity_color_code(&NotificationSeverity::Warning), 0xFFAA00);
        assert_eq!(provider.get_severity_color_code(&NotificationSeverity::Info), 0x0066FF);
    }

    #[test]
    fn test_severity_text() {
        let config = create_test_config();
        let provider = WeChatProvider::new(config);

        assert_eq!(provider.get_severity_text(&NotificationSeverity::Critical), "严重");
        assert_eq!(provider.get_severity_text(&NotificationSeverity::Error), "错误");
        assert_eq!(provider.get_severity_text(&NotificationSeverity::Warning), "警告");
        assert_eq!(provider.get_severity_text(&NotificationSeverity::Info), "信息");
    }

    #[test]
    fn test_score_emojis() {
        let config = create_test_config();
        let provider = WeChatProvider::new(config);

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
        let provider = WeChatProvider::new(config);

        assert_eq!(provider.get_score_grade(9.5), "优秀");
        assert_eq!(provider.get_score_grade(8.5), "良好");
        assert_eq!(provider.get_score_grade(7.5), "中等");
        assert_eq!(provider.get_score_grade(6.5), "及格");
        assert_eq!(provider.get_score_grade(5.5), "较差");
        assert_eq!(provider.get_score_grade(3.0), "很差");
    }

    #[test]
    fn test_webhook_url_validation() {
        let config = create_test_config();
        let provider = WeChatProvider::new(config);
        assert!(provider.validate_webhook_url().is_ok());

        // 测试空URL
        let mut config_empty = create_test_config();
        config_empty.webhook_url = String::new();
        let provider_empty = WeChatProvider::new(config_empty);
        assert!(provider_empty.validate_webhook_url().is_err());

        // 测试HTTP URL
        let mut config_http = create_test_config();
        config_http.webhook_url = "http://qyapi.weixin.qq.com/webhook".to_string();
        let provider_http = WeChatProvider::new(config_http);
        assert!(provider_http.validate_webhook_url().is_err());

        // 测试无效域名
        let mut config_invalid = create_test_config();
        config_invalid.webhook_url = "https://invalid.com/webhook".to_string();
        let provider_invalid = WeChatProvider::new(config_invalid);
        assert!(provider_invalid.validate_webhook_url().is_err());
    }

    #[test]
    fn test_choose_message_format() {
        let config = create_test_config();
        let provider = WeChatProvider::new(config);

        // 测试有模板数据时选择模板卡片
        let message_with_template = create_test_message();
        let template_msg = provider.choose_message_format(&message_with_template);
        assert_eq!(template_msg["msgtype"], "template_card");

        // 测试无模板数据但启用Markdown时选择Markdown
        let mut message_no_template = create_test_message();
        message_no_template.template_data.clear();
        let markdown_msg = provider.choose_message_format(&message_no_template);
        assert_eq!(markdown_msg["msgtype"], "markdown");

        // 测试禁用Markdown时选择文本
        let mut config_no_markdown = create_test_config();
        config_no_markdown.enable_markdown = false;
        let provider_no_markdown = WeChatProvider::new(config_no_markdown);
        message_no_template.template_data.clear();
        let text_msg = provider_no_markdown.choose_message_format(&message_no_template);
        assert_eq!(text_msg["msgtype"], "text");

        // 测试内容过长时选择文本
        let mut message_long_content = create_test_message();
        message_long_content.template_data.clear();
        message_long_content.content = "很长的内容".repeat(500); // 超过2000字符
        let long_text_msg = provider.choose_message_format(&message_long_content);
        assert_eq!(long_text_msg["msgtype"], "text");
    }

    #[test]
    fn test_wechat_config_default() {
        let config = WeChatConfig::default();

        assert!(config.webhook_url.is_empty());
        assert!(config.corp_id.is_none());
        assert!(config.corp_secret.is_none());
        assert!(config.agent_id.is_none());
        assert!(config.enable_markdown);
        assert!(config.enable_mentions);
        assert_eq!(config.timeout_seconds, 30);
        assert_eq!(config.max_content_length, 4096);
    }

    #[test]
    fn test_wechat_message_structures() {
        // 测试文本消息结构
        let text_msg = WeChatTextMessage {
            content: "test content".to_string(),
            mentioned_list: Some(vec!["@all".to_string()]),
            mentioned_mobile_list: Some(vec!["13800138000".to_string()]),
        };
        assert_eq!(text_msg.content, "test content");
        assert_eq!(text_msg.mentioned_list.as_ref().unwrap()[0], "@all");

        // 测试Markdown消息结构
        let markdown_msg = WeChatMarkdownMessage {
            content: "# Test Title\n\nTest content".to_string(),
        };
        assert!(markdown_msg.content.contains("# Test Title"));

        // 测试文章结构
        let article = WeChatArticle {
            title: "Test Article".to_string(),
            description: "Test Description".to_string(),
            url: "https://example.com".to_string(),
            picurl: Some("https://example.com/pic.jpg".to_string()),
        };
        assert_eq!(article.title, "Test Article");
        assert_eq!(article.url, "https://example.com");
    }

    #[test]
    fn test_message_without_score() {
        let config = create_test_config();
        let provider = WeChatProvider::new(config);

        let mut message = NotificationMessage::new(
            "无评分消息".to_string(),
            "这是一个没有评分的消息".to_string(),
            NotificationSeverity::Info,
            "/test/project".to_string(),
        );

        let text_msg = provider.build_text_message(&message);
        let content = text_msg["text"]["content"].as_str().unwrap();

        assert!(content.contains("📊")); // 无评分时的emoji
        assert!(!content.contains("评分:")); // 不应该包含评分信息
    }

    #[test]
    fn test_message_without_metadata() {
        let config = create_test_config();
        let provider = WeChatProvider::new(config);

        let message = NotificationMessage::new(
            "无元数据消息".to_string(),
            "这是一个没有元数据的消息".to_string(),
            NotificationSeverity::Info,
            "/test/project".to_string(),
        );

        let text_msg = provider.build_text_message(&message);
        let content = text_msg["text"]["content"].as_str().unwrap();

        assert!(!content.contains("详细信息:")); // 不应该包含元数据部分
    }

    #[test]
    fn test_mentions_disabled() {
        let mut config = create_test_config();
        config.enable_mentions = false;
        let provider = WeChatProvider::new(config);

        let message = create_message_with_mentions();
        let text_msg = provider.build_text_message(&message);

        // 当禁用mentions时，不应该有mentioned_list字段
        assert!(text_msg["text"]["mentioned_list"].is_null());
        assert!(text_msg["text"]["mentioned_mobile_list"].is_null());
    }
}