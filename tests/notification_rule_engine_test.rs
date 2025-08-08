use ai_commit::notification::{
    NotificationMessage, NotificationSeverity, NotificationPlatform,
    NotificationCondition, ConditionOperator, AggregationConfig
};
use ai_commit::notification::rule_engine::{NotificationRuleEngine, NotificationRule};
use chrono::Utc;
use std::time::Duration;

#[tokio::test]
async fn test_rule_engine_basic_functionality() {
    let engine = NotificationRuleEngine::new();

    // 创建一个简单的规则
    let rule = NotificationRule::new("test-rule".to_string(), "Test Rule".to_string())
        .with_condition(NotificationCondition {
            field: "severity".to_string(),
            operator: ConditionOperator::Equals,
            value: serde_json::json!("Warning"),
        })
        .with_platform(NotificationPlatform::Email);

    // 添加规则
    engine.add_rule(rule).await.unwrap();

    // 创建匹配的消息
    let message = NotificationMessage::new(
        "Test Warning".to_string(),
        "This is a test warning message".to_string(),
        NotificationSeverity::Warning,
        "/test/project".to_string(),
    );

    // 处理通知
    let processed = engine.process_notification(message).await.unwrap();

    // 验证结果
    assert_eq!(processed.len(), 1);
    assert_eq!(processed[0].platforms, vec![NotificationPlatform::Email]);
    assert_eq!(processed[0].aggregated_messages.len(), 1);
}

#[tokio::test]
async fn test_rule_engine_condition_evaluation() {
    let engine = NotificationRuleEngine::new();

    // 创建分数条件规则
    let rule = NotificationRule::new("score-rule".to_string(), "Score Rule".to_string())
        .with_condition(NotificationCondition {
            field: "score".to_string(),
            operator: ConditionOperator::GreaterThan,
            value: serde_json::json!(8.0),
        })
        .with_platform(NotificationPlatform::Feishu);

    engine.add_rule(rule).await.unwrap();

    // 创建高分消息
    let high_score_message = NotificationMessage::new(
        "High Score".to_string(),
        "This is a high score message".to_string(),
        NotificationSeverity::Info,
        "/test/project".to_string(),
    ).with_score(9.5);

    // 创建低分消息
    let low_score_message = NotificationMessage::new(
        "Low Score".to_string(),
        "This is a low score message".to_string(),
        NotificationSeverity::Info,
        "/test/project".to_string(),
    ).with_score(6.0);

    // 处理高分消息
    let high_score_processed = engine.process_notification(high_score_message).await.unwrap();
    assert_eq!(high_score_processed.len(), 1);

    // 处理低分消息
    let low_score_processed = engine.process_notification(low_score_message).await.unwrap();
    assert_eq!(low_score_processed.len(), 0); // 不应该匹配
}

#[tokio::test]
async fn test_rule_engine_aggregation() {
    let engine = NotificationRuleEngine::new();

    // 创建聚合规则
    let aggregation_config = AggregationConfig {
        window_duration: Duration::from_secs(60),
        max_messages: 3,
        group_by: vec!["project_path".to_string()],
    };

    let rule = NotificationRule::new("aggregation-rule".to_string(), "Aggregation Rule".to_string())
        .with_condition(NotificationCondition {
            field: "severity".to_string(),
            operator: ConditionOperator::Equals,
            value: serde_json::json!("Info"),
        })
        .with_platform(NotificationPlatform::WeChat)
        .with_aggregation(aggregation_config);

    engine.add_rule(rule).await.unwrap();

    // 发送多个消息
    for i in 1..=3 {
        let message = NotificationMessage::new(
            format!("Info Message {}", i),
            format!("This is info message number {}", i),
            NotificationSeverity::Info,
            "/test/project".to_string(),
        );

        let processed = engine.process_notification(message).await.unwrap();

        if i < 3 {
            // 前两个消息应该被聚合，不立即发送
            assert_eq!(processed.len(), 0);
        } else {
            // 第三个消息应该触发聚合
            assert_eq!(processed.len(), 1);
            assert_eq!(processed[0].aggregated_messages.len(), 3);
        }
    }
}

#[tokio::test]
async fn test_rule_engine_deduplication() {
    let engine = NotificationRuleEngine::new();

    // 创建规则
    let rule = NotificationRule::new("dedup-rule".to_string(), "Deduplication Rule".to_string())
        .with_platform(NotificationPlatform::DingTalk);

    engine.add_rule(rule).await.unwrap();

    // 创建相同的消息
    let message1 = NotificationMessage::new(
        "Duplicate Message".to_string(),
        "This is a duplicate message".to_string(),
        NotificationSeverity::Error,
        "/test/project".to_string(),
    );

    let message2 = NotificationMessage::new(
        "Duplicate Message".to_string(),
        "This is a duplicate message".to_string(),
        NotificationSeverity::Error,
        "/test/project".to_string(),
    );

    // 处理第一个消息
    let processed1 = engine.process_notification(message1).await.unwrap();
    assert_eq!(processed1.len(), 1);

    // 处理第二个消息（应该被去重）
    let processed2 = engine.process_notification(message2).await.unwrap();
    assert_eq!(processed2.len(), 0); // 应该被去重
}

#[tokio::test]
async fn test_rule_engine_rate_limiting() {
    let engine = NotificationRuleEngine::new();

    // 创建规则
    let rule = NotificationRule::new("rate-limit-rule".to_string(), "Rate Limit Rule".to_string())
        .with_platform(NotificationPlatform::Email);

    engine.add_rule(rule).await.unwrap();

    // 快速发送多个消息
    let mut successful_count = 0;
    let mut rate_limited_count = 0;

    for i in 1..=15 {
        let message = NotificationMessage::new(
            format!("Rate Limit Test {}", i),
            format!("This is rate limit test message {}", i),
            NotificationSeverity::Info,
            "/test/project".to_string(),
        );

        match engine.process_notification(message).await {
            Ok(processed) => {
                if !processed.is_empty() {
                    successful_count += 1;
                }
            },
            Err(_) => {
                rate_limited_count += 1;
            }
        }
    }

    // 验证频率限制生效
    assert!(rate_limited_count > 0, "Rate limiting should have occurred");
    assert!(successful_count <= 10, "Should not exceed rate limit");
}

#[tokio::test]
async fn test_rule_engine_regex_condition() {
    let engine = NotificationRuleEngine::new();

    // 创建正则表达式规则
    let rule = NotificationRule::new("regex-rule".to_string(), "Regex Rule".to_string())
        .with_condition(NotificationCondition {
            field: "title".to_string(),
            operator: ConditionOperator::Regex,
            value: serde_json::json!(r"^ERROR.*database"),
        })
        .with_platform(NotificationPlatform::Feishu);

    engine.add_rule(rule).await.unwrap();

    // 创建匹配的消息
    let matching_message = NotificationMessage::new(
        "ERROR: database connection failed".to_string(),
        "Database connection error occurred".to_string(),
        NotificationSeverity::Error,
        "/test/project".to_string(),
    );

    // 创建不匹配的消息
    let non_matching_message = NotificationMessage::new(
        "WARNING: network timeout".to_string(),
        "Network timeout occurred".to_string(),
        NotificationSeverity::Warning,
        "/test/project".to_string(),
    );

    // 处理匹配的消息
    let matching_processed = engine.process_notification(matching_message).await.unwrap();
    assert_eq!(matching_processed.len(), 1);

    // 处理不匹配的消息
    let non_matching_processed = engine.process_notification(non_matching_message).await.unwrap();
    assert_eq!(non_matching_processed.len(), 0);
}

#[tokio::test]
async fn test_rule_engine_metadata_condition() {
    let engine = NotificationRuleEngine::new();

    // 创建元数据条件规则
    let rule = NotificationRule::new("metadata-rule".to_string(), "Metadata Rule".to_string())
        .with_condition(NotificationCondition {
            field: "metadata.environment".to_string(),
            operator: ConditionOperator::Equals,
            value: serde_json::json!("production"),
        })
        .with_platform(NotificationPlatform::Email);

    engine.add_rule(rule).await.unwrap();

    // 创建带有匹配元数据的消息
    let message = NotificationMessage::new(
        "Production Alert".to_string(),
        "This is a production alert".to_string(),
        NotificationSeverity::Critical,
        "/prod/project".to_string(),
    ).with_metadata("environment".to_string(), "production".to_string());

    // 处理消息
    let processed = engine.process_notification(message).await.unwrap();
    assert_eq!(processed.len(), 1);
}

#[tokio::test]
async fn test_rule_engine_statistics() {
    let engine = NotificationRuleEngine::new();

    // 添加一些规则
    let rule1 = NotificationRule::new("rule1".to_string(), "Rule 1".to_string())
        .with_platform(NotificationPlatform::Email);

    let mut rule2 = NotificationRule::new("rule2".to_string(), "Rule 2".to_string())
        .with_platform(NotificationPlatform::Feishu);
    rule2.enabled = false;

    engine.add_rule(rule1).await.unwrap();
    engine.add_rule(rule2).await.unwrap();

    // 获取统计信息
    let stats = engine.get_rule_statistics().await;

    assert_eq!(stats.total_rules, 2);
    assert_eq!(stats.enabled_rules, 1);
    assert_eq!(stats.disabled_rules, 1);
}

#[tokio::test]
async fn test_rule_validation() {
    let engine = NotificationRuleEngine::new();

    // 测试空ID规则
    let invalid_rule = NotificationRule {
        id: "".to_string(),
        name: "Invalid Rule".to_string(),
        description: None,
        enabled: true,
        priority: 100,
        conditions: vec![],
        platforms: vec![NotificationPlatform::Email],
        template: None,
        aggregation: None,
        rate_limit: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    // 应该验证失败
    assert!(engine.add_rule(invalid_rule).await.is_err());

    // 测试无效正则表达式
    let regex_rule = NotificationRule::new("regex-rule".to_string(), "Regex Rule".to_string())
        .with_condition(NotificationCondition {
            field: "title".to_string(),
            operator: ConditionOperator::Regex,
            value: serde_json::json!("[invalid regex"),
        })
        .with_platform(NotificationPlatform::Email);

    // 应该验证失败
    assert!(engine.add_rule(regex_rule).await.is_err());
}