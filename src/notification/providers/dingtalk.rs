use std::sync::Arc;
use std::collections::HashMap;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use base64::Engine;

use crate::notification::{NotificationProvider, NotificationMessage, NotificationResult, NotificationPlatform, NotificationSeverity};

/// 钉钉配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DingTalkConfig {
    pub webhook_url: String,
    pub secret: Option<String>,
    pub at_mobiles: Vec<String>,
    pub at_user_ids: Vec<String>,
    pub is_at_all: bool,
    pub enable_markdown: bool,
}

impl Default for DingTalkConfig {
    fn default() -> Self {
        Self {
            webhook_url: String::new(),
            secret: None,
            at_mobiles: Vec::new(),
            at_user_ids: Vec::new(),
            is_at_all: false,
            enable_markdown: true,
        }
    }
}

/// 钉钉消息类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "msgtype")]
pub enum DingTalkMessageType {
    #[serde(rename = "text")]
    Text { text: DingTalkText, at: DingTalkAt },
    #[serde(rename = "markdown")]
    Markdown { markdown: DingTalkMarkdown, at: DingTalkAt },
    #[serde(rename = "link")]
    Link { link: DingTalkLink },
    #[serde(rename = "actionCard")]
    ActionCard { actionCard: DingTalkActionCard },
}

/// 钉钉文本消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DingTalkText {
    pub content: String,
}

/// 钉钉Markdown消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DingTalkMarkdown {
    pub title: String,
    pub text: String,
}

/// 钉钉链接消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DingTalkLink {
    pub title: String,
    pub text: String,
    #[serde(rename = "messageUrl")]
    pub message_url: String,
    #[serde(rename = "picUrl")]
    pub pic_url: Option<String>,
}

/// 钉钉ActionCard消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DingTalkActionCard {
    pub title: String,
    pub text: String,
    #[serde(rename = "singleTitle")]
    pub single_title: Option<String>,
    #[serde(rename = "singleURL")]
    pub single_url: Option<String>,
    #[serde(rename = "btnOrientation")]
    pub btn_orientation: Option<String>,
    pub btns: Option<Vec<DingTalkButton>>,
}

/// 钉钉按钮
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DingTalkButton {
    pub title: String,
    #[serde(rename = "actionURL")]
    pub action_url: String,
}

/// 钉钉@功能
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DingTalkAt {
    #[serde(rename = "atMobiles")]
    pub at_mobiles: Vec<String>,
    #[serde(rename = "atUserIds")]
    pub at_user_ids: Vec<String>,
    #[serde(rename = "isAtAll")]
    pub is_at_all: bool,
}

impl Default for DingTalkAt {
    fn default() -> Self {
        Self {
            at_mobiles: Vec::new(),
            at_user_ids: Vec::new(),
            is_at_all: false,
        }
    }
}

/// 钉钉API响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DingTalkResponse {
    pub errcode: i32,
    pub errmsg: String,
}

/// 钉钉通知提供商
pub struct DingTalkProvider {
    config: DingTalkConfig,
    client: Arc<reqwest::Client>,
}

impl DingTalkProvider {
    pub fn new(config: DingTalkConfig) -> Self {
        Self {
            config,
            client: Arc::new(reqwest::Client::new()),
        }
    }

    /// 生成签名
    fn generate_signature(&self, timestamp: i64) -> anyhow::Result<String> {
        if let Some(secret) = &self.config.secret {
            let string_to_sign = format!("{}\n{}", timestamp, secret);

            type HmacSha256 = Hmac<Sha256>;
            let mut mac = HmacSha256::new_from_slice(secret.as_bytes())?;
            mac.update(string_to_sign.as_bytes());
            let result = mac.finalize();

            let signature = base64::engine::general_purpose::STANDARD.encode(result.into_bytes());
            Ok(urlencoding::encode(&signature).to_string())
        } else {
            Ok(String::new())
        }
    }

    /// 构建请求URL
    fn build_request_url(&self) -> anyhow::Result<String> {
        let mut url = self.config.webhook_url.clone();

        if self.config.secret.is_some() {
            let timestamp = Utc::now().timestamp_millis();
            let signature = self.generate_signature(timestamp)?;

            let separator = if url.contains('?') { "&" } else { "?" };
            url = format!("{}{}timestamp={}&sign={}", url, separator, timestamp, signature);
        }

        Ok(url)
    }

    /// 创建@配置
    fn create_at_config(&self, message: &NotificationMessage) -> DingTalkAt {
        let mut at_config = DingTalkAt {
            at_mobiles: self.config.at_mobiles.clone(),
            at_user_ids: self.config.at_user_ids.clone(),
            is_at_all: self.config.is_at_all,
        };

        // 从消息元数据中获取额外的@配置
        if let Some(at_mobiles) = message.metadata.get("at_mobiles") {
            if let Ok(mobiles) = serde_json::from_str::<Vec<String>>(at_mobiles) {
                at_config.at_mobiles.extend(mobiles);
            }
        }

        if let Some(at_user_ids) = message.metadata.get("at_user_ids") {
            if let Ok(user_ids) = serde_json::from_str::<Vec<String>>(at_user_ids) {
                at_config.at_user_ids.extend(user_ids);
            }
        }

        if let Some(is_at_all) = message.metadata.get("is_at_all") {
            if let Ok(at_all) = is_at_all.parse::<bool>() {
                at_config.is_at_all = at_all;
            }
        }

        at_config
    }

    /// 创建Markdown消息
    fn create_markdown_message(&self, message: &NotificationMessage) -> DingTalkMessageType {
        let at_config = self.create_at_config(message);

        let title = format!("{} {}", self.get_severity_emoji(&message.severity), message.title);
        let mut content = String::new();

        // 标题
        content.push_str(&format!("# {}\n\n", title));

        // 基本信息
        content.push_str(&format!("**项目路径**: {}\n", message.project_path));
        content.push_str(&format!("**时间**: {}\n", message.timestamp.format("%Y-%m-%d %H:%M:%S")));

        if let Some(score) = message.score {
            content.push_str(&format!("**质量评分**: {}/10\n", score));
        }

        content.push_str("\n---\n\n");

        // 详细内容
        content.push_str("## 详细内容\n\n");
        content.push_str(&message.content);

        // 元数据
        if !message.metadata.is_empty() {
            content.push_str("\n\n## 详细信息\n\n");
            for (key, value) in &message.metadata {
                if !key.starts_with("at_") && key != "is_at_all" {
                    content.push_str(&format!("- **{}**: {}\n", key, value));
                }
            }
        }

        // 添加@提醒文本
        if !at_config.at_mobiles.is_empty() || !at_config.at_user_ids.is_empty() || at_config.is_at_all {
            content.push_str("\n\n---\n");
            if at_config.is_at_all {
                content.push_str("@所有人 ");
            } else {
                for mobile in &at_config.at_mobiles {
                    content.push_str(&format!("@{} ", mobile));
                }
                for user_id in &at_config.at_user_ids {
                    content.push_str(&format!("@{} ", user_id));
                }
            }
        }

        DingTalkMessageType::Markdown {
            markdown: DingTalkMarkdown {
                title: title.clone(),
                text: content,
            },
            at: at_config,
        }
    }

    /// 创建文本消息
    fn create_text_message(&self, message: &NotificationMessage) -> DingTalkMessageType {
        let at_config = self.create_at_config(message);

        let mut content = String::new();

        // 标题
        content.push_str(&format!("{} {}\n\n", self.get_severity_emoji(&message.severity), message.title));

        // 基本信息
        content.push_str(&format!("项目路径: {}\n", message.project_path));
        content.push_str(&format!("时间: {}\n", message.timestamp.format("%Y-%m-%d %H:%M:%S")));

        if let Some(score) = message.score {
            content.push_str(&format!("质量评分: {}/10\n", score));
        }

        content.push_str("\n");

        // 详细内容
        content.push_str("详细内容:\n");
        content.push_str(&message.content);

        // 元数据
        if !message.metadata.is_empty() {
            content.push_str("\n\n详细信息:\n");
            for (key, value) in &message.metadata {
                if !key.starts_with("at_") && key != "is_at_all" {
                    content.push_str(&format!("• {}: {}\n", key, value));
                }
            }
        }

        DingTalkMessageType::Text {
            text: DingTalkText { content },
            at: at_config,
        }
    }

    /// 创建ActionCard消息
    fn create_action_card_message(&self, message: &NotificationMessage) -> DingTalkMessageType {
        let title = format!("{} {}", self.get_severity_emoji(&message.severity), message.title);
        let mut content = String::new();

        // 基本信息
        content.push_str(&format!("**项目路径**: {}\n\n", message.project_path));
        content.push_str(&format!("**时间**: {}\n\n", message.timestamp.format("%Y-%m-%d %H:%M:%S")));

        if let Some(score) = message.score {
            content.push_str(&format!("**质量评分**: {}/10\n\n", score));
        }

        // 详细内容
        content.push_str("### 详细内容\n\n");
        content.push_str(&message.content);

        // 元数据
        if !message.metadata.is_empty() {
            content.push_str("\n\n### 详细信息\n\n");
            for (key, value) in &message.metadata {
                if !key.starts_with("at_") && key != "is_at_all" {
                    content.push_str(&format!("- **{}**: {}\n", key, value));
                }
            }
        }

        // 创建按钮
        let mut buttons = Vec::new();

        // 从模板数据中获取按钮配置
        if let Some(btns_data) = message.template_data.get("buttons") {
            if let Ok(btns) = serde_json::from_value::<Vec<DingTalkButton>>(btns_data.clone()) {
                buttons = btns;
            }
        }

        // 默认按钮
        if buttons.is_empty() {
            if let Some(report_url) = message.metadata.get("report_url") {
                buttons.push(DingTalkButton {
                    title: "查看详细报告".to_string(),
                    action_url: report_url.clone(),
                });
            }
        }

        DingTalkMessageType::ActionCard {
            actionCard: DingTalkActionCard {
                title: title.clone(),
                text: content,
                single_title: None,
                single_url: None,
                btn_orientation: Some("0".to_string()), // 竖直排列
                btns: if buttons.is_empty() { None } else { Some(buttons) },
            },
        }
    }

    /// 获取严重程度对应的表情符号
    fn get_severity_emoji(&self, severity: &NotificationSeverity) -> &'static str {
        match severity {
            NotificationSeverity::Critical => "🚨",
            NotificationSeverity::Error => "❌",
            NotificationSeverity::Warning => "⚠️",
            NotificationSeverity::Info => "ℹ️",
        }
    }

    /// 发送消息到钉钉
    async fn send_dingtalk_message(&self, message_type: DingTalkMessageType) -> anyhow::Result<DingTalkResponse> {
        let url = self.build_request_url()?;

        let response = self.client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&message_type)
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!("钉钉API请求失败: HTTP {}", response.status());
        }

        let dingtalk_response: DingTalkResponse = response.json().await?;

        if dingtalk_response.errcode != 0 {
            anyhow::bail!("钉钉API返回错误: {} - {}", dingtalk_response.errcode, dingtalk_response.errmsg);
        }

        Ok(dingtalk_response)
    }
}

#[async_trait]
impl NotificationProvider for DingTalkProvider {
    fn platform(&self) -> NotificationPlatform {
        NotificationPlatform::DingTalk
    }

    async fn send_notification(&self, message: &NotificationMessage) -> anyhow::Result<NotificationResult> {
        // 根据配置选择消息类型
        let message_type = if let Some(msg_type) = message.metadata.get("message_type") {
            match msg_type.as_str() {
                "text" => self.create_text_message(message),
                "actionCard" => self.create_action_card_message(message),
                _ => {
                    if self.config.enable_markdown {
                        self.create_markdown_message(message)
                    } else {
                        self.create_text_message(message)
                    }
                }
            }
        } else if self.config.enable_markdown {
            self.create_markdown_message(message)
        } else {
            self.create_text_message(message)
        };

        match self.send_dingtalk_message(message_type).await {
            Ok(_) => {
                log::info!("钉钉通知发送成功: {}", message.id);
                Ok(NotificationResult::success(
                    message.id.clone(),
                    NotificationPlatform::DingTalk,
                ))
            }
            Err(e) => {
                log::error!("钉钉通知发送失败: {}", e);
                Err(e)
            }
        }
    }

    fn is_configured(&self) -> bool {
        !self.config.webhook_url.is_empty()
    }

    fn supports_rich_content(&self) -> bool {
        true
    }
}