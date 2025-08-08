use std::sync::Arc;
use std::path::Path;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use lettre::{
    AsyncSmtpTransport, AsyncTransport, Tokio1Executor,
    message::{header::ContentType, SinglePart},
    transport::smtp::{authentication::Credentials, PoolConfig},
    Message,
};
use mime::Mime;

use crate::notification::{
    NotificationProvider, NotificationMessage, NotificationResult, NotificationPlatform,
};
use crate::notification::templates::TemplateEngine;

/// 邮件配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailConfig {
    /// SMTP 服务器地址
    pub smtp_server: String,
    /// SMTP 端口
    pub smtp_port: u16,
    /// 用户名
    pub username: String,
    /// 密码
    pub password: String,
    /// 发件人地址
    pub from_address: String,
    /// 发件人名称
    pub from_name: Option<String>,
    /// 收件人地址列表
    pub to_addresses: Vec<String>,
    /// 抄送地址列表
    pub cc_addresses: Vec<String>,
    /// 密送地址列表
    pub bcc_addresses: Vec<String>,
    /// 是否启用 TLS
    pub enable_tls: bool,
    /// 连接池配置
    pub pool_config: EmailPoolConfig,
    /// 邮件模板配置
    pub template_config: EmailTemplateConfig,
}

impl Default for EmailConfig {
    fn default() -> Self {
        Self {
            smtp_server: String::new(),
            smtp_port: 587,
            username: String::new(),
            password: String::new(),
            from_address: String::new(),
            from_name: None,
            to_addresses: Vec::new(),
            cc_addresses: Vec::new(),
            bcc_addresses: Vec::new(),
            enable_tls: true,
            pool_config: EmailPoolConfig::default(),
            template_config: EmailTemplateConfig::default(),
        }
    }
}

/// 邮件连接池配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailPoolConfig {
    /// 最大连接数
    pub max_size: u32,
    /// 最小空闲连接数
    pub min_idle: Option<u32>,
    /// 连接超时时间（秒）
    pub connection_timeout: u64,
}

impl Default for EmailPoolConfig {
    fn default() -> Self {
        Self {
            max_size: 10,
            min_idle: Some(1),
            connection_timeout: 30,
        }
    }
}

/// 邮件模板配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailTemplateConfig {
    /// 默认模板名称
    pub default_template: String,
    /// 是否启用 HTML 格式
    pub enable_html: bool,
    /// 是否启用纯文本格式
    pub enable_text: bool,
    /// 邮件主题模板
    pub subject_template: String,
}

impl Default for EmailTemplateConfig {
    fn default() -> Self {
        Self {
            default_template: "email_default".to_string(),
            enable_html: true,
            enable_text: true,
            subject_template: "[AI-Commit] {{title}}".to_string(),
        }
    }
}

/// 邮件附件
#[derive(Debug, Clone)]
pub struct EmailAttachment {
    /// 文件名
    pub filename: String,
    /// 内容类型
    pub content_type: Mime,
    /// 文件内容
    pub content: Vec<u8>,
    /// 是否为内嵌图片
    pub inline: bool,
    /// 内容 ID（用于内嵌图片）
    pub content_id: Option<String>,
}

impl EmailAttachment {
    /// 创建文件附件
    pub fn from_file<P: AsRef<Path>>(file_path: P) -> anyhow::Result<Self> {
        let path = file_path.as_ref();
        let filename = path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("attachment")
            .to_string();

        let content = std::fs::read(path)?;
        let content_type = mime_guess::from_path(path)
            .first_or_octet_stream();

        Ok(Self {
            filename,
            content_type,
            content,
            inline: false,
            content_id: None,
        })
    }

    /// 创建内嵌图片
    pub fn inline_image<P: AsRef<Path>>(file_path: P, content_id: String) -> anyhow::Result<Self> {
        let mut attachment = Self::from_file(file_path)?;
        attachment.inline = true;
        attachment.content_id = Some(content_id);
        Ok(attachment)
    }

    /// 创建内存附件
    pub fn from_bytes(filename: String, content_type: Mime, content: Vec<u8>) -> Self {
        Self {
            filename,
            content_type,
            content,
            inline: false,
            content_id: None,
        }
    }
}

/// 邮件提供商
pub struct EmailProvider {
    config: EmailConfig,
    transport: AsyncSmtpTransport<Tokio1Executor>,
    template_engine: Arc<TemplateEngine>,
}

impl EmailProvider {
    /// 创建新的邮件提供商
    pub fn new(config: EmailConfig) -> anyhow::Result<Self> {
        let transport = Self::create_transport(&config)?;
        let template_engine = Arc::new(TemplateEngine::new());

        Ok(Self {
            config,
            transport,
            template_engine,
        })
    }

    /// 创建 SMTP 传输
    fn create_transport(config: &EmailConfig) -> anyhow::Result<AsyncSmtpTransport<Tokio1Executor>> {
        let mut builder = AsyncSmtpTransport::<Tokio1Executor>::relay(&config.smtp_server)?;

        // 设置端口
        builder = builder.port(config.smtp_port);

        // 设置认证
        if !config.username.is_empty() && !config.password.is_empty() {
            let credentials = Credentials::new(config.username.clone(), config.password.clone());
            builder = builder.credentials(credentials);
        }

        // 设置连接池
        let pool_config = PoolConfig::new()
            .max_size(config.pool_config.max_size);

        let pool_config = if let Some(min_idle) = config.pool_config.min_idle {
            pool_config.min_idle(min_idle)
        } else {
            pool_config
        };

        builder = builder.pool_config(pool_config);

        Ok(builder.build())
    }

    /// 发送带附件的邮件
    pub async fn send_notification_with_attachments(
        &self,
        message: &NotificationMessage,
        attachments: Vec<EmailAttachment>,
    ) -> anyhow::Result<NotificationResult> {
        let email_message = self.build_email_message(message, attachments).await?;

        match self.transport.send(email_message).await {
            Ok(response) => {
                log::info!("Email sent successfully: {:?}", response);
                Ok(NotificationResult::success(
                    message.id.clone(),
                    NotificationPlatform::Email,
                ))
            }
            Err(e) => {
                log::error!("Failed to send email: {}", e);
                Err(anyhow::anyhow!("Failed to send email: {}", e))
            }
        }
    }

    /// 构建邮件消息
    async fn build_email_message(
        &self,
        message: &NotificationMessage,
        attachments: Vec<EmailAttachment>,
    ) -> anyhow::Result<Message> {
        // 解析发件人地址
        let from = if let Some(ref name) = self.config.from_name {
            format!("{} <{}>", name, self.config.from_address).parse()?
        } else {
            self.config.from_address.parse()?
        };

        // 构建邮件主题
        let subject = self.render_subject_template(message)?;

        // 开始构建邮件
        let mut email_builder = Message::builder()
            .from(from)
            .subject(subject);

        // 添加收件人
        for to_addr in &self.config.to_addresses {
            email_builder = email_builder.to(to_addr.parse()?);
        }

        // 添加抄送
        for cc_addr in &self.config.cc_addresses {
            email_builder = email_builder.cc(cc_addr.parse()?);
        }

        // 添加密送
        for bcc_addr in &self.config.bcc_addresses {
            email_builder = email_builder.bcc(bcc_addr.parse()?);
        }

        // 构建邮件内容 - 简化版本，只支持 HTML 或纯文本
        if self.config.template_config.enable_html {
            let html_content = self.render_html_content(message)?;
            Ok(email_builder.body(html_content)?)
        } else {
            let text_content = self.render_text_content(message)?;
            Ok(email_builder.body(text_content)?)
        }
    }

    /// 构建附件部分
    fn build_attachment_part(&self, attachment: &EmailAttachment) -> anyhow::Result<SinglePart> {
        let part_builder = SinglePart::builder()
            .header(ContentType::parse(&attachment.content_type.to_string())?)
            .body(attachment.content.clone());

        // Note: lettre 0.11 doesn't support adding custom headers to SinglePart in the same way
        // For now, we'll create a basic attachment without custom disposition headers
        // In a production environment, you might need to use a different approach or upgrade lettre

        Ok(part_builder)
    }

    /// 渲染邮件主题模板
    pub fn render_subject_template(&self, message: &NotificationMessage) -> anyhow::Result<String> {
        let template = &self.config.template_config.subject_template;
        let context = serde_json::json!({
            "title": message.title,
            "severity": message.severity,
            "project_path": message.project_path,
            "score": message.score,
        });

        let mut handlebars = handlebars::Handlebars::new();
        handlebars.register_template_string("subject", template)?;
        Ok(handlebars.render("subject", &context)?)
    }

    /// 渲染 HTML 内容
    pub fn render_html_content(&self, message: &NotificationMessage) -> anyhow::Result<String> {
        let template_name = &self.config.template_config.default_template;
        self.template_engine.render(template_name, message)
    }

    /// 渲染纯文本内容
    pub fn render_text_content(&self, message: &NotificationMessage) -> anyhow::Result<String> {
        // 简单的纯文本版本
        let mut content = String::new();
        content.push_str(&format!("标题: {}\n", message.title));
        content.push_str(&format!("项目路径: {}\n", message.project_path));
        content.push_str(&format!("时间: {}\n", message.timestamp.format("%Y-%m-%d %H:%M:%S")));

        if let Some(score) = message.score {
            content.push_str(&format!("质量评分: {}/10\n", score));
        }

        content.push_str("\n内容:\n");
        content.push_str(&message.content);

        if !message.metadata.is_empty() {
            content.push_str("\n\n详细信息:\n");
            for (key, value) in &message.metadata {
                content.push_str(&format!("- {}: {}\n", key, value));
            }
        }

        Ok(content)
    }

    /// 测试 SMTP 连接
    pub async fn test_connection(&self) -> anyhow::Result<()> {
        self.transport.test_connection().await?;
        Ok(())
    }

    /// 获取配置
    pub fn get_config(&self) -> &EmailConfig {
        &self.config
    }

    /// 更新配置
    pub fn update_config(&mut self, config: EmailConfig) -> anyhow::Result<()> {
        self.transport = Self::create_transport(&config)?;
        self.config = config;
        Ok(())
    }
}

#[async_trait]
impl NotificationProvider for EmailProvider {
    fn platform(&self) -> NotificationPlatform {
        NotificationPlatform::Email
    }

    async fn send_notification(&self, message: &NotificationMessage) -> anyhow::Result<NotificationResult> {
        // 发送不带附件的邮件
        self.send_notification_with_attachments(message, vec![]).await
    }

    fn is_configured(&self) -> bool {
        !self.config.smtp_server.is_empty()
            && !self.config.username.is_empty()
            && !self.config.from_address.is_empty()
            && !self.config.to_addresses.is_empty()
    }

    fn supports_rich_content(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::notification::NotificationSeverity;
    use chrono::Utc;

    fn create_test_config() -> EmailConfig {
        EmailConfig {
            smtp_server: "smtp.example.com".to_string(),
            smtp_port: 587,
            username: "test@example.com".to_string(),
            password: "password".to_string(),
            from_address: "test@example.com".to_string(),
            from_name: Some("AI-Commit".to_string()),
            to_addresses: vec!["recipient@example.com".to_string()],
            cc_addresses: vec![],
            bcc_addresses: vec![],
            enable_tls: true,
            pool_config: EmailPoolConfig::default(),
            template_config: EmailTemplateConfig::default(),
        }
    }

    fn create_test_message() -> NotificationMessage {
        NotificationMessage::new(
            "Test Notification".to_string(),
            "This is a test notification content.".to_string(),
            NotificationSeverity::Info,
            "/test/project".to_string(),
        ).with_score(8.5)
    }

    #[test]
    fn test_email_config_default() {
        let config = EmailConfig::default();
        assert_eq!(config.smtp_port, 587);
        assert!(config.enable_tls);
        assert_eq!(config.template_config.default_template, "email_default");
    }

    #[test]
    fn test_email_attachment_from_bytes() {
        let content = b"Hello, World!".to_vec();
        let attachment = EmailAttachment::from_bytes(
            "test.txt".to_string(),
            mime::TEXT_PLAIN,
            content.clone(),
        );

        assert_eq!(attachment.filename, "test.txt");
        assert_eq!(attachment.content_type, mime::TEXT_PLAIN);
        assert_eq!(attachment.content, content);
        assert!(!attachment.inline);
        assert!(attachment.content_id.is_none());
    }

    #[tokio::test]
    async fn test_email_provider_creation() {
        let config = create_test_config();
        let provider = EmailProvider::new(config);
        assert!(provider.is_ok());
    }

    #[tokio::test]
    async fn test_email_provider_is_configured() {
        let config = create_test_config();
        let provider = EmailProvider::new(config).unwrap();
        assert!(provider.is_configured());
    }

    #[tokio::test]
    async fn test_email_provider_not_configured() {
        let config = EmailConfig::default();
        let provider = EmailProvider::new(config).unwrap();
        assert!(!provider.is_configured());
    }

    #[tokio::test]
    async fn test_render_subject_template() {
        let config = create_test_config();
        let provider = EmailProvider::new(config).unwrap();
        let message = create_test_message();

        let subject = provider.render_subject_template(&message).unwrap();
        assert!(subject.contains("Test Notification"));
    }

    #[tokio::test]
    async fn test_render_text_content() {
        let config = create_test_config();
        let provider = EmailProvider::new(config).unwrap();
        let message = create_test_message();

        let content = provider.render_text_content(&message).unwrap();
        assert!(content.contains("Test Notification"));
        assert!(content.contains("/test/project"));
        assert!(content.contains("8.5/10"));
    }

    #[tokio::test]
    async fn test_render_html_content() {
        let config = create_test_config();
        let provider = EmailProvider::new(config).unwrap();
        let message = create_test_message();

        let content = provider.render_html_content(&message).unwrap();
        assert!(content.contains("Test Notification"));
        assert!(content.contains("/test/project"));
        assert!(content.contains("8.5"));
    }

    #[test]
    fn test_supports_rich_content() {
        let config = create_test_config();
        let provider = EmailProvider::new(config).unwrap();
        assert!(provider.supports_rich_content());
    }

    #[test]
    fn test_platform() {
        let config = create_test_config();
        let provider = EmailProvider::new(config).unwrap();
        assert_eq!(provider.platform(), NotificationPlatform::Email);
    }
}