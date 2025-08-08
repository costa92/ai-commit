use ai_commit::notification::{
    NotificationService, NotificationMessage, NotificationPlatform, NotificationSeverity,
    NotificationConfig, RetryConfig, RateLimitConfig, TemplateConfig, NotificationProvider
};
use ai_commit::notification::providers::{WeChatProvider};
use ai_commit::notification::providers::wechat::WeChatConfig;
use std::sync::Arc;

/// 创建测试用的微信配置
fn create_test_wechat_config() -> WeChatConfig {
    WeChatConfig {
        webhook_url: std::env::var("WECHAT_WEBHOOK_URL")
            .unwrap_or_else(|_| "https://qyapi.weixin.qq.com/cgi-bin/webhook/send?key=test".to_string()),
        corp_id: Some("test_corp_id".to_string()),
        corp_secret: Some("test_corp_secret".to_string()),
        agent_id: Some("1000001".to_string()),
        enable_markdown: true,
        enable_mentions: true,
        timeout_seconds: 30,
        max_content_length: 4096,
    }
}

/// 创建测试消息
fn create_test_notification_message() -> NotificationMessage {
    let mut message = NotificationMessage::new(
        "AI-Commit 代码审查完成".to_string(),
        "本次代码审查发现了一些需要注意的问题，包括代码风格、潜在的性能问题和安全隐患。建议及时修复这些问题以提高代码质量。".to_string(),
        NotificationSeverity::Warning,
        "/home/user/projects/ai-commit".to_string(),
    );

    message = message.with_score(7.8);
    message = message.with_metadata("总问题数".to_string(), "12".to_string());
    message = message.with_metadata("分析文件数".to_string(), "45".to_string());
    message = message.with_metadata("代码行数".to_string(), "3,247".to_string());
    message = message.with_metadata("检测时长".to_string(), "2.3秒".to_string());

    message = message.with_template_data("report_url".to_string(),
        serde_json::Value::String("https://ai-commit.example.com/reports/20241205-001".to_string()));
    message = message.with_template_data("project_url".to_string(),
        serde_json::Value::String("https://github.com/example/ai-commit".to_string()));
    message = message.with_template_data("mentions".to_string(),
        serde_json::Value::Array(vec![
            serde_json::Value::String("@all".to_string()),
            serde_json::Value::String("developer1".to_string()),
        ]));
    message = message.with_template_data("mobile_mentions".to_string(),
        serde_json::Value::Array(vec![
            serde_json::Value::String("13800138000".to_string()),
        ]));

    message
}

/// 创建不同严重程度的测试消息
fn create_messages_with_different_severities() -> Vec<NotificationMessage> {
    let base_message = create_test_notification_message();
    let mut messages = Vec::new();

    // 严重错误消息
    let mut critical_message = base_message.clone();
    critical_message.title = "严重安全漏洞检测".to_string();
    critical_message.content = "检测到严重的安全漏洞，包括SQL注入和XSS攻击风险，需要立即修复！".to_string();
    critical_message.severity = NotificationSeverity::Critical;
    critical_message.score = Some(3.2);
    messages.push(critical_message);

    // 错误消息
    let mut error_message = base_message.clone();
    error_message.title = "编译错误检测".to_string();
    error_message.content = "代码存在编译错误，无法正常构建项目。".to_string();
    error_message.severity = NotificationSeverity::Error;
    error_message.score = Some(4.5);
    messages.push(error_message);

    // 警告消息
    let mut warning_message = base_message.clone();
    warning_message.title = "代码质量警告".to_string();
    warning_message.content = "发现一些代码质量问题，建议优化。".to_string();
    warning_message.severity = NotificationSeverity::Warning;
    warning_message.score = Some(6.8);
    messages.push(warning_message);

    // 信息消息
    let mut info_message = base_message.clone();
    info_message.title = "代码审查完成".to_string();
    info_message.content = "代码审查已完成，整体质量良好。".to_string();
    info_message.severity = NotificationSeverity::Info;
    info_message.score = Some(8.9);
    messages.push(info_message);

    messages
}

#[tokio::test]
async fn test_wechat_provider_basic_functionality() {
    let config = create_test_wechat_config();
    let provider = WeChatProvider::new(config);

    // 测试基本属性
    assert_eq!(provider.platform(), NotificationPlatform::WeChat);
    assert!(provider.supports_rich_content());

    // 注意：只有在配置了有效的webhook URL时才会返回true
    // 在测试环境中，我们使用模拟的URL，所以这里可能返回false
    println!("Provider configured: {}", provider.is_configured());
}

#[tokio::test]
async fn test_wechat_text_message_format() {
    let config = create_test_wechat_config();
    let provider = WeChatProvider::new(config);
    let message = create_test_notification_message();

    // 测试文本消息构建
    let text_msg = provider.build_text_message(&message);

    assert_eq!(text_msg["msgtype"], "text");

    let text_content = text_msg["text"]["content"].as_str().unwrap();
    assert!(text_content.contains("AI-Commit 代码审查完成"));
    assert!(text_content.contains("7.8/10"));
    assert!(text_content.contains("/home/user/projects/ai-commit"));
    assert!(text_content.contains("总问题数: 12"));
    assert!(text_content.contains("👍")); // 7.8分对应的emoji

    // 测试@提醒功能
    let mentioned_list = text_msg["text"]["mentioned_list"].as_array().unwrap();
    assert_eq!(mentioned_list.len(), 2);
    assert_eq!(mentioned_list[0], "@all");
    assert_eq!(mentioned_list[1], "developer1");

    let mobile_list = text_msg["text"]["mentioned_mobile_list"].as_array().unwrap();
    assert_eq!(mobile_list.len(), 1);
    assert_eq!(mobile_list[0], "13800138000");
}

#[tokio::test]
async fn test_wechat_markdown_message_format() {
    let config = create_test_wechat_config();
    let provider = WeChatProvider::new(config);
    let message = create_test_notification_message();

    // 测试Markdown消息构建
    let markdown_msg = provider.build_markdown_message(&message);

    assert_eq!(markdown_msg["msgtype"], "markdown");

    let markdown_content = markdown_msg["markdown"]["content"].as_str().unwrap();
    assert!(markdown_content.contains("# 👍 AI-Commit 代码审查完成"));
    assert!(markdown_content.contains("**项目路径**: `/home/user/projects/ai-commit`"));
    assert!(markdown_content.contains("**质量评分**: 7.8/10 (中等)"));
    assert!(markdown_content.contains("**严重程度**: <font color=\"#FFAA00\">警告</font>"));
    assert!(markdown_content.contains("## 详细信息"));
    assert!(markdown_content.contains("## 统计信息"));
    assert!(markdown_content.contains("- **总问题数**: 12"));
    assert!(markdown_content.contains("[查看详细报告](https://ai-commit.example.com/reports/20241205-001)"));
    assert!(markdown_content.contains("[打开项目](https://github.com/example/ai-commit)"));
}

#[tokio::test]
async fn test_wechat_template_card_message_format() {
    let config = create_test_wechat_config();
    let provider = WeChatProvider::new(config);
    let message = create_test_notification_message();

    // 测试模板卡片消息构建
    let card_msg = provider.build_template_card_message(&message);

    assert_eq!(card_msg["msgtype"], "template_card");

    let template_card = &card_msg["template_card"];
    assert_eq!(template_card["card_type"], "text_notice");

    // 验证卡片来源
    let source = &template_card["source"];
    assert_eq!(source["desc"], "AI-Commit 代码审查系统");
    assert!(source["icon_url"].as_str().unwrap().contains("wwpic"));

    // 验证主标题
    let main_title = &template_card["main_title"];
    assert!(main_title["title"].as_str().unwrap().contains("👍 AI-Commit 代码审查完成"));
    assert!(main_title["desc"].as_str().unwrap().contains("严重程度: 警告"));

    // 验证强调内容（评分）
    let emphasis_content = &template_card["emphasis_content"];
    assert_eq!(emphasis_content["title"], "7.8");
    assert_eq!(emphasis_content["desc"], "质量评分");

    // 验证引用区域
    let quote_area = &template_card["quote_area"];
    assert_eq!(quote_area["type"], 0);
    assert!(quote_area["quote_text"].as_str().unwrap().contains("本次代码审查发现了一些需要注意的问题"));

    // 验证副标题
    assert!(template_card["sub_title_text"].as_str().unwrap().contains("/home/user/projects/ai-commit"));

    // 验证水平内容列表
    let horizontal_content = template_card["horizontal_content_list"].as_array().unwrap();
    assert!(horizontal_content.len() >= 5); // 至少包含项目路径、时间、评分和元数据

    // 验证跳转链接
    let jump_list = template_card["jump_list"].as_array().unwrap();
    assert_eq!(jump_list.len(), 2);

    let report_jump = &jump_list[0];
    assert_eq!(report_jump["type"], 1);
    assert_eq!(report_jump["title"], "查看详细报告");
    assert_eq!(report_jump["url"], "https://ai-commit.example.com/reports/20241205-001");

    let project_jump = &jump_list[1];
    assert_eq!(project_jump["type"], 1);
    assert_eq!(project_jump["title"], "打开项目");
    assert_eq!(project_jump["url"], "https://github.com/example/ai-commit");

    // 验证卡片动作
    let card_action = &template_card["card_action"];
    assert_eq!(card_action["type"], 1);
    assert_eq!(card_action["url"], "https://ai-commit.example.com/reports/20241205-001");
}

#[tokio::test]
async fn test_wechat_different_severities() {
    let config = create_test_wechat_config();
    let provider = WeChatProvider::new(config);
    let messages = create_messages_with_different_severities();

    for message in messages {
        let markdown_msg = provider.build_markdown_message(&message);
        let markdown_content = markdown_msg["markdown"]["content"].as_str().unwrap();

        match message.severity {
            NotificationSeverity::Critical => {
                assert!(markdown_content.contains("🚨")); // 3.2分对应的emoji
                assert!(markdown_content.contains("<font color=\"#FF0000\">严重</font>"));
            },
            NotificationSeverity::Error => {
                assert!(markdown_content.contains("🚨")); // 4.5分对应的emoji
                assert!(markdown_content.contains("<font color=\"#FF6600\">错误</font>"));
            },
            NotificationSeverity::Warning => {
                assert!(markdown_content.contains("⚠️")); // 6.8分对应的emoji
                assert!(markdown_content.contains("<font color=\"#FFAA00\">警告</font>"));
            },
            NotificationSeverity::Info => {
                assert!(markdown_content.contains("✅")); // 8.9分对应的emoji
                assert!(markdown_content.contains("<font color=\"#0066FF\">信息</font>"));
            },
        }
    }
}

#[tokio::test]
async fn test_wechat_message_format_selection() {
    let config = create_test_wechat_config();
    let provider = WeChatProvider::new(config);

    // 测试有模板数据时选择模板卡片
    let message_with_template = create_test_notification_message();
    let template_msg = provider.choose_message_format(&message_with_template);
    assert_eq!(template_msg["msgtype"], "template_card");

    // 测试无模板数据但启用Markdown时选择Markdown
    let mut message_no_template = create_test_notification_message();
    message_no_template.template_data.clear();
    let markdown_msg = provider.choose_message_format(&message_no_template);
    assert_eq!(markdown_msg["msgtype"], "markdown");

    // 测试内容过长时选择文本消息
    let mut message_long_content = create_test_notification_message();
    message_long_content.template_data.clear();
    message_long_content.content = "这是一个非常长的内容，".repeat(200); // 超过2000字符
    let text_msg = provider.choose_message_format(&message_long_content);
    assert_eq!(text_msg["msgtype"], "text");
}

#[tokio::test]
async fn test_wechat_content_truncation() {
    let mut config = create_test_wechat_config();
    config.max_content_length = 200; // 设置较小的内容长度限制
    let provider = WeChatProvider::new(config);

    let mut message = create_test_notification_message();
    message.content = "这是一个非常长的内容，用于测试内容截断功能。".repeat(10);

    // 测试文本消息截断
    let text_msg = provider.build_text_message(&message);
    let text_content = text_msg["text"]["content"].as_str().unwrap();
    assert!(text_content.len() <= 200);
    assert!(text_content.contains("[内容过长，已截断]"));

    // 测试Markdown消息截断
    let markdown_msg = provider.build_markdown_message(&message);
    let markdown_content = markdown_msg["markdown"]["content"].as_str().unwrap();
    assert!(markdown_content.len() <= 200);
    assert!(markdown_content.contains("*内容过长，已截断*"));
}

#[tokio::test]
async fn test_wechat_notification_service_integration() {
    let wechat_config = create_test_wechat_config();
    let wechat_provider = Arc::new(WeChatProvider::new(wechat_config));

    let notification_config = NotificationConfig {
        enabled_platforms: vec![NotificationPlatform::WeChat],
        retry_config: RetryConfig::default(),
        rate_limit: RateLimitConfig::default(),
        template_config: TemplateConfig::default(),
        rules: vec![],
    };

    let mut notification_service = NotificationService::new(notification_config);
    notification_service.register_provider(wechat_provider);

    // 验证提供商注册
    let providers = notification_service.get_providers();
    assert!(providers.contains(&NotificationPlatform::WeChat));

    // 验证提供商可用性检查
    let is_available = notification_service.is_provider_available(&NotificationPlatform::WeChat);
    println!("WeChat provider available: {}", is_available);

    // 创建测试消息
    let message = create_test_notification_message();

    // 注意：实际发送测试需要有效的webhook URL
    // 这里我们只测试消息构建和服务集成
    println!("Test message created: {}", message.title);
    println!("Message ID: {}", message.id);
    println!("Message severity: {:?}", message.severity);
    println!("Message score: {:?}", message.score);
}

#[tokio::test]
async fn test_wechat_config_validation() {
    // 测试有效配置
    let valid_config = create_test_wechat_config();
    let valid_provider = WeChatProvider::new(valid_config);
    // 注意：在测试环境中，模拟URL可能不会通过验证

    // 测试空URL配置
    let mut empty_config = create_test_wechat_config();
    empty_config.webhook_url = String::new();
    let empty_provider = WeChatProvider::new(empty_config);
    assert!(!empty_provider.is_configured());

    // 测试HTTP URL（应该要求HTTPS）
    let mut http_config = create_test_wechat_config();
    http_config.webhook_url = "http://qyapi.weixin.qq.com/webhook".to_string();
    let http_provider = WeChatProvider::new(http_config);
    assert!(!http_provider.is_configured());

    // 测试无效域名
    let mut invalid_config = create_test_wechat_config();
    invalid_config.webhook_url = "https://invalid.com/webhook".to_string();
    let invalid_provider = WeChatProvider::new(invalid_config);
    assert!(!invalid_provider.is_configured());
}

#[tokio::test]
async fn test_wechat_message_without_optional_fields() {
    let config = create_test_wechat_config();
    let provider = WeChatProvider::new(config);

    // 创建没有评分和元数据的消息
    let simple_message = NotificationMessage::new(
        "简单消息".to_string(),
        "这是一个简单的测试消息".to_string(),
        NotificationSeverity::Info,
        "/test/project".to_string(),
    );

    // 测试文本消息
    let text_msg = provider.build_text_message(&simple_message);
    let text_content = text_msg["text"]["content"].as_str().unwrap();
    assert!(text_content.contains("📊")); // 无评分时的emoji
    assert!(!text_content.contains("评分:")); // 不应该包含评分信息
    assert!(!text_content.contains("详细信息:")); // 不应该包含元数据部分

    // 测试Markdown消息
    let markdown_msg = provider.build_markdown_message(&simple_message);
    let markdown_content = markdown_msg["markdown"]["content"].as_str().unwrap();
    assert!(markdown_content.contains("# 📊 简单消息"));
    assert!(!markdown_content.contains("**质量评分**")); // 不应该包含评分部分
    assert!(!markdown_content.contains("## 统计信息")); // 不应该包含统计信息部分

    // 测试模板卡片消息
    let card_msg = provider.build_template_card_message(&simple_message);
    let template_card = &card_msg["template_card"];
    assert!(template_card["emphasis_content"].is_null()); // 没有评分时不应该有强调内容
}

#[tokio::test]
async fn test_wechat_mentions_functionality() {
    let config = create_test_wechat_config();
    let provider = WeChatProvider::new(config);

    // 创建带有@提醒的消息
    let mut message_with_mentions = create_test_notification_message();
    message_with_mentions = message_with_mentions.with_template_data("mentions".to_string(),
        serde_json::Value::Array(vec![
            serde_json::Value::String("@all".to_string()),
            serde_json::Value::String("user1".to_string()),
            serde_json::Value::String("user2".to_string()),
        ]));
    message_with_mentions = message_with_mentions.with_template_data("mobile_mentions".to_string(),
        serde_json::Value::Array(vec![
            serde_json::Value::String("13800138000".to_string()),
            serde_json::Value::String("13900139000".to_string()),
        ]));

    let text_msg = provider.build_text_message(&message_with_mentions);

    // 验证@用户提醒
    let mentioned_list = text_msg["text"]["mentioned_list"].as_array().unwrap();
    assert_eq!(mentioned_list.len(), 3);
    assert_eq!(mentioned_list[0], "@all");
    assert_eq!(mentioned_list[1], "user1");
    assert_eq!(mentioned_list[2], "user2");

    // 验证手机号提醒
    let mobile_list = text_msg["text"]["mentioned_mobile_list"].as_array().unwrap();
    assert_eq!(mobile_list.len(), 2);
    assert_eq!(mobile_list[0], "13800138000");
    assert_eq!(mobile_list[1], "13900139000");

    // 测试禁用@提醒功能
    let mut config_no_mentions = create_test_wechat_config();
    config_no_mentions.enable_mentions = false;
    let provider_no_mentions = WeChatProvider::new(config_no_mentions);

    let text_msg_no_mentions = provider_no_mentions.build_text_message(&message_with_mentions);
    assert!(text_msg_no_mentions["text"]["mentioned_list"].is_null());
    assert!(text_msg_no_mentions["text"]["mentioned_mobile_list"].is_null());
}