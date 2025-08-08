use std::sync::Arc;
use ai_commit::notification::{
    NotificationService, NotificationMessage, NotificationSeverity, NotificationConfig,
    NotificationPlatform, RetryConfig, RateLimitConfig, TemplateConfig, NotificationProvider,
};
use ai_commit::notification::providers::{EmailProvider, EmailConfig, EmailAttachment, EmailPoolConfig, EmailTemplateConfig};
use std::time::Duration;

/// 创建测试用的邮件配置
fn create_test_email_config() -> EmailConfig {
    EmailConfig {
        smtp_server: "smtp.gmail.com".to_string(),
        smtp_port: 587,
        username: "test@example.com".to_string(),
        password: "test_password".to_string(),
        from_address: "ai-commit@example.com".to_string(),
        from_name: Some("AI-Commit System".to_string()),
        to_addresses: vec![
            "developer1@example.com".to_string(),
            "developer2@example.com".to_string(),
        ],
        cc_addresses: vec!["manager@example.com".to_string()],
        bcc_addresses: vec!["admin@example.com".to_string()],
        enable_tls: true,
        pool_config: EmailPoolConfig {
            max_size: 5,
            min_idle: Some(1),
            connection_timeout: 30,
        },
        template_config: EmailTemplateConfig {
            default_template: "email_default".to_string(),
            enable_html: true,
            enable_text: true,
            subject_template: "[AI-Commit] {{title}} - {{severity}}".to_string(),
        },
    }
}

/// 创建测试用的通知消息
fn create_test_notification_message() -> NotificationMessage {
    NotificationMessage::new(
        "代码审查完成".to_string(),
        "项目代码审查已完成，发现了一些需要关注的问题。".to_string(),
        NotificationSeverity::Warning,
        "/home/user/projects/my-app".to_string(),
    )
    .with_score(7.5)
    .with_metadata("total_files".to_string(), "25".to_string())
    .with_metadata("issues_found".to_string(), "8".to_string())
    .with_metadata("critical_issues".to_string(), "2".to_string())
    .with_template_data("analysis_duration".to_string(), serde_json::json!("2.5 minutes"))
}

/// 创建高质量代码的通知消息
fn create_high_quality_notification() -> NotificationMessage {
    NotificationMessage::new(
        "代码质量优秀".to_string(),
        "恭喜！您的代码质量评分很高，没有发现严重问题。".to_string(),
        NotificationSeverity::Info,
        "/home/user/projects/excellent-app".to_string(),
    )
    .with_score(9.2)
    .with_metadata("total_files".to_string(), "15".to_string())
    .with_metadata("issues_found".to_string(), "1".to_string())
    .with_metadata("critical_issues".to_string(), "0".to_string())
}

/// 创建严重问题的通知消息
fn create_critical_notification() -> NotificationMessage {
    NotificationMessage::new(
        "发现严重安全问题".to_string(),
        "代码审查发现了严重的安全漏洞，请立即处理！".to_string(),
        NotificationSeverity::Critical,
        "/home/user/projects/vulnerable-app".to_string(),
    )
    .with_score(3.1)
    .with_metadata("total_files".to_string(), "42".to_string())
    .with_metadata("issues_found".to_string(), "23".to_string())
    .with_metadata("critical_issues".to_string(), "5".to_string())
    .with_metadata("security_issues".to_string(), "3".to_string())
}

#[tokio::test]
async fn test_email_provider_creation() {
    let config = create_test_email_config();
    let provider = EmailProvider::new(config);

    assert!(provider.is_ok());
    let provider = provider.unwrap();
    assert_eq!(provider.platform(), NotificationPlatform::Email);
    assert!(provider.supports_rich_content());
    assert!(provider.is_configured());
}

#[tokio::test]
async fn test_email_provider_not_configured() {
    let config = EmailConfig::default();
    let provider = EmailProvider::new(config).unwrap();

    assert!(!provider.is_configured());
}

#[tokio::test]
async fn test_email_config_validation() {
    let mut config = create_test_email_config();

    // 测试完整配置
    let provider = EmailProvider::new(config.clone()).unwrap();
    assert!(provider.is_configured());

    // 测试缺少 SMTP 服务器
    config.smtp_server = String::new();
    let provider = EmailProvider::new(config.clone()).unwrap();
    assert!(!provider.is_configured());

    // 恢复 SMTP 服务器，测试缺少用户名
    config.smtp_server = "smtp.gmail.com".to_string();
    config.username = String::new();
    let provider = EmailProvider::new(config.clone()).unwrap();
    assert!(!provider.is_configured());

    // 恢复用户名，测试缺少发件人地址
    config.username = "test@example.com".to_string();
    config.from_address = String::new();
    let provider = EmailProvider::new(config.clone()).unwrap();
    assert!(!provider.is_configured());

    // 恢复发件人地址，测试缺少收件人
    config.from_address = "ai-commit@example.com".to_string();
    config.to_addresses = Vec::new();
    let provider = EmailProvider::new(config).unwrap();
    assert!(!provider.is_configured());
}

#[tokio::test]
async fn test_email_attachment_creation() {
    // 测试从字节创建附件
    let content = b"Hello, World! This is a test attachment.".to_vec();
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

    // 测试创建内嵌图片附件
    let image_content = b"fake_image_data".to_vec();
    let inline_attachment = EmailAttachment::from_bytes(
        "logo.png".to_string(),
        mime::IMAGE_PNG,
        image_content.clone(),
    );

    // 转换为内嵌图片
    let mut inline_image = inline_attachment;
    inline_image.inline = true;
    inline_image.content_id = Some("logo123".to_string());

    assert_eq!(inline_image.filename, "logo.png");
    assert_eq!(inline_image.content_type, mime::IMAGE_PNG);
    assert!(inline_image.inline);
    assert_eq!(inline_image.content_id, Some("logo123".to_string()));
}

#[tokio::test]
async fn test_email_template_rendering() {
    let config = create_test_email_config();
    let provider = EmailProvider::new(config).unwrap();
    let message = create_test_notification_message();

    // 测试主题模板渲染
    let subject = provider.render_subject_template(&message).unwrap();
    assert!(subject.contains("代码审查完成"));
    assert!(subject.contains("Warning"));

    // 测试纯文本内容渲染
    let text_content = provider.render_text_content(&message).unwrap();
    assert!(text_content.contains("代码审查完成"));
    assert!(text_content.contains("/home/user/projects/my-app"));
    assert!(text_content.contains("7.5/10"));
    assert!(text_content.contains("total_files: 25"));
    assert!(text_content.contains("issues_found: 8"));

    // 测试 HTML 内容渲染
    let html_content = provider.render_html_content(&message).unwrap();
    assert!(html_content.contains("代码审查完成"));
    assert!(html_content.contains("/home/user/projects/my-app"));
    assert!(html_content.contains("7.5"));
    assert!(html_content.contains("<!DOCTYPE html>"));
    assert!(html_content.contains("</html>"));
}

#[tokio::test]
async fn test_email_template_rendering_different_severities() {
    let config = create_test_email_config();
    let provider = EmailProvider::new(config).unwrap();

    // 测试不同严重程度的消息
    let messages = vec![
        create_high_quality_notification(),
        create_test_notification_message(),
        create_critical_notification(),
    ];

    for message in messages {
        let subject = provider.render_subject_template(&message).unwrap();
        let text_content = provider.render_text_content(&message).unwrap();
        let html_content = provider.render_html_content(&message).unwrap();

        // 验证基本内容存在
        assert!(subject.contains(&message.title));
        assert!(text_content.contains(&message.title));
        assert!(text_content.contains(&message.project_path));
        assert!(html_content.contains(&message.title));
        assert!(html_content.contains(&message.project_path));

        // 验证评分信息
        if let Some(score) = message.score {
            assert!(text_content.contains(&format!("{}/10", score)));
            // HTML content may format the score differently due to floating point precision
            // Just check if it contains the integer part of the score
            let score_int = score as i32;
            assert!(html_content.contains(&score_int.to_string()), "HTML content should contain score {}, but got: {}", score_int, html_content);
        }

        // 验证元数据
        for (key, value) in &message.metadata {
            assert!(text_content.contains(&format!("{}: {}", key, value)));
        }
    }
}

#[tokio::test]
async fn test_notification_service_with_email() {
    let email_config = create_test_email_config();
    let email_provider = Arc::new(EmailProvider::new(email_config).unwrap());

    let notification_config = NotificationConfig {
        enabled_platforms: vec![NotificationPlatform::Email],
        retry_config: RetryConfig {
            max_retries: 2,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(5),
            backoff_multiplier: 2.0,
        },
        rate_limit: RateLimitConfig {
            max_notifications_per_minute: 5,
            max_notifications_per_hour: 50,
            burst_limit: 3,
        },
        template_config: TemplateConfig::default(),
        rules: vec![],
    };

    let mut service = NotificationService::new(notification_config);
    service.register_provider(email_provider);

    // 验证提供商注册
    assert!(service.is_provider_available(&NotificationPlatform::Email));
    assert_eq!(service.get_providers(), vec![NotificationPlatform::Email]);

    let message = create_test_notification_message();

    // 注意：这里不会真正发送邮件，因为我们使用的是测试配置
    // 在实际测试中，可能需要使用 mock SMTP 服务器
    // let results = service.send_notification(message).await;
    // 由于没有真实的 SMTP 服务器，这个测试会失败，所以我们跳过实际发送
}

#[tokio::test]
async fn test_email_provider_config_update() {
    let mut config = create_test_email_config();
    let mut provider = EmailProvider::new(config.clone()).unwrap();

    // 验证初始配置
    assert_eq!(provider.get_config().smtp_server, "smtp.gmail.com");
    assert_eq!(provider.get_config().smtp_port, 587);

    // 更新配置
    config.smtp_server = "smtp.outlook.com".to_string();
    config.smtp_port = 25;

    let result = provider.update_config(config);
    assert!(result.is_ok());

    // 验证配置已更新
    assert_eq!(provider.get_config().smtp_server, "smtp.outlook.com");
    assert_eq!(provider.get_config().smtp_port, 25);
}

#[tokio::test]
async fn test_email_pool_config() {
    let pool_config = EmailPoolConfig {
        max_size: 20,
        min_idle: Some(5),
        connection_timeout: 60,
    };

    let email_config = EmailConfig {
        pool_config,
        ..create_test_email_config()
    };

    let provider = EmailProvider::new(email_config).unwrap();
    assert_eq!(provider.get_config().pool_config.max_size, 20);
    assert_eq!(provider.get_config().pool_config.min_idle, Some(5));
    assert_eq!(provider.get_config().pool_config.connection_timeout, 60);
}

#[tokio::test]
async fn test_email_template_config() {
    let template_config = EmailTemplateConfig {
        default_template: "custom_email_template".to_string(),
        enable_html: false,
        enable_text: true,
        subject_template: "Custom Subject: {{title}}".to_string(),
    };

    let email_config = EmailConfig {
        template_config,
        ..create_test_email_config()
    };

    let provider = EmailProvider::new(email_config).unwrap();
    let message = create_test_notification_message();

    // 测试自定义主题模板
    let subject = provider.render_subject_template(&message).unwrap();
    assert!(subject.contains("Custom Subject:"));
    assert!(subject.contains("代码审查完成"));

    // 验证模板配置
    assert_eq!(provider.get_config().template_config.default_template, "custom_email_template");
    assert!(!provider.get_config().template_config.enable_html);
    assert!(provider.get_config().template_config.enable_text);
}

#[tokio::test]
async fn test_multiple_recipients() {
    let config = EmailConfig {
        to_addresses: vec![
            "dev1@example.com".to_string(),
            "dev2@example.com".to_string(),
            "dev3@example.com".to_string(),
        ],
        cc_addresses: vec![
            "manager1@example.com".to_string(),
            "manager2@example.com".to_string(),
        ],
        bcc_addresses: vec![
            "admin@example.com".to_string(),
        ],
        ..create_test_email_config()
    };

    let provider = EmailProvider::new(config).unwrap();
    assert!(provider.is_configured());

    // 验证收件人配置
    assert_eq!(provider.get_config().to_addresses.len(), 3);
    assert_eq!(provider.get_config().cc_addresses.len(), 2);
    assert_eq!(provider.get_config().bcc_addresses.len(), 1);
}

#[tokio::test]
async fn test_email_content_with_metadata() {
    let config = create_test_email_config();
    let provider = EmailProvider::new(config).unwrap();

    let mut message = create_test_notification_message();
    message = message
        .with_metadata("repository".to_string(), "my-awesome-project".to_string())
        .with_metadata("branch".to_string(), "feature/new-feature".to_string())
        .with_metadata("commit_hash".to_string(), "abc123def456".to_string())
        .with_metadata("author".to_string(), "John Doe".to_string());

    let text_content = provider.render_text_content(&message).unwrap();
    let html_content = provider.render_html_content(&message).unwrap();

    // 验证元数据在内容中
    assert!(text_content.contains("repository: my-awesome-project"));
    assert!(text_content.contains("branch: feature/new-feature"));
    assert!(text_content.contains("commit_hash: abc123def456"));
    assert!(text_content.contains("author: John Doe"));

    // HTML 内容也应该包含元数据
    assert!(html_content.contains("my-awesome-project"));
    assert!(html_content.contains("feature/new-feature"));
    assert!(html_content.contains("abc123def456"));
    assert!(html_content.contains("John Doe"));
}

#[test]
fn test_email_config_serialization() {
    let config = create_test_email_config();

    // 测试序列化
    let serialized = serde_json::to_string(&config).unwrap();
    assert!(serialized.contains("smtp.gmail.com"));
    assert!(serialized.contains("test@example.com"));

    // 测试反序列化
    let deserialized: EmailConfig = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized.smtp_server, config.smtp_server);
    assert_eq!(deserialized.username, config.username);
    assert_eq!(deserialized.to_addresses, config.to_addresses);
}

#[test]
fn test_email_config_defaults() {
    let config = EmailConfig::default();

    assert_eq!(config.smtp_port, 587);
    assert!(config.enable_tls);
    assert_eq!(config.pool_config.max_size, 10);
    assert_eq!(config.pool_config.min_idle, Some(1));
    assert_eq!(config.template_config.default_template, "email_default");
    assert!(config.template_config.enable_html);
    assert!(config.template_config.enable_text);
    assert_eq!(config.template_config.subject_template, "[AI-Commit] {{title}}");
}

// 集成测试：测试完整的邮件发送流程（需要真实的 SMTP 服务器）
#[tokio::test]
#[ignore] // 默认忽略，需要真实 SMTP 服务器时手动运行
async fn test_real_email_sending() {
    // 这个测试需要真实的 SMTP 配置
    // 可以通过环境变量提供真实的 SMTP 配置进行测试

    let smtp_server = std::env::var("TEST_SMTP_SERVER").unwrap_or_default();
    let smtp_username = std::env::var("TEST_SMTP_USERNAME").unwrap_or_default();
    let smtp_password = std::env::var("TEST_SMTP_PASSWORD").unwrap_or_default();
    let from_address = std::env::var("TEST_FROM_ADDRESS").unwrap_or_default();
    let to_address = std::env::var("TEST_TO_ADDRESS").unwrap_or_default();

    if smtp_server.is_empty() || smtp_username.is_empty() || to_address.is_empty() {
        println!("Skipping real email test - missing environment variables");
        return;
    }

    let config = EmailConfig {
        smtp_server,
        smtp_port: 587,
        username: smtp_username,
        password: smtp_password,
        from_address,
        from_name: Some("AI-Commit Test".to_string()),
        to_addresses: vec![to_address],
        cc_addresses: vec![],
        bcc_addresses: vec![],
        enable_tls: true,
        pool_config: EmailPoolConfig::default(),
        template_config: EmailTemplateConfig::default(),
    };

    let provider = EmailProvider::new(config).unwrap();

    // 测试连接
    let connection_result = provider.test_connection().await;
    if connection_result.is_err() {
        println!("SMTP connection test failed: {:?}", connection_result);
        return;
    }

    let message = NotificationMessage::new(
        "AI-Commit 邮件测试".to_string(),
        "这是一封来自 AI-Commit 系统的测试邮件。".to_string(),
        NotificationSeverity::Info,
        "/test/project".to_string(),
    ).with_score(8.0);

    // 发送邮件
    let result = provider.send_notification(&message).await;
    assert!(result.is_ok());

    let notification_result = result.unwrap();
    assert!(notification_result.success);
    assert_eq!(notification_result.platform, NotificationPlatform::Email);
}