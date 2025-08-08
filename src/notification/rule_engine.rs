use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Duration;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::notification::{
    NotificationMessage, NotificationPlatform, NotificationSeverity,
    NotificationCondition, ConditionOperator, AggregationConfig
};

/// 通知规则引擎
pub struct NotificationRuleEngine {
    rules: Arc<RwLock<Vec<NotificationRule>>>,
    aggregator: Arc<RwLock<NotificationAggregator>>,
    rate_limiter: Arc<RwLock<NotificationRateLimiter>>,
    deduplicator: Arc<RwLock<NotificationDeduplicator>>,
}

impl NotificationRuleEngine {
    pub fn new() -> Self {
        Self {
            rules: Arc::new(RwLock::new(Vec::new())),
            aggregator: Arc::new(RwLock::new(NotificationAggregator::new())),
            rate_limiter: Arc::new(RwLock::new(NotificationRateLimiter::new())),
            deduplicator: Arc::new(RwLock::new(NotificationDeduplicator::new())),
        }
    }

    /// 添加规则
    pub async fn add_rule(&self, rule: NotificationRule) -> anyhow::Result<()> {
        self.validate_rule(&rule)?;
        let mut rules = self.rules.write().await;
        rules.push(rule);
        Ok(())
    }

    /// 更新规则
    pub async fn update_rule(&self, rule_id: &str, rule: NotificationRule) -> anyhow::Result<()> {
        self.validate_rule(&rule)?;
        let mut rules = self.rules.write().await;

        if let Some(existing_rule) = rules.iter_mut().find(|r| r.id == rule_id) {
            *existing_rule = rule;
            Ok(())
        } else {
            Err(anyhow::anyhow!("Rule with id {} not found", rule_id))
        }
    }

    /// 删除规则
    pub async fn remove_rule(&self, rule_id: &str) -> anyhow::Result<()> {
        let mut rules = self.rules.write().await;
        let initial_len = rules.len();
        rules.retain(|rule| rule.id != rule_id);

        if rules.len() < initial_len {
            Ok(())
        } else {
            Err(anyhow::anyhow!("Rule with id {} not found", rule_id))
        }
    }

    /// 获取所有规则
    pub async fn get_rules(&self) -> Vec<NotificationRule> {
        let rules = self.rules.read().await;
        rules.clone()
    }

    /// 处理通知消息，应用规则引擎
    pub async fn process_notification(&self, message: NotificationMessage) -> anyhow::Result<Vec<ProcessedNotification>> {
        // 1. 检查去重
        if self.is_duplicate(&message).await? {
            log::debug!("Notification {} is duplicate, skipping", message.id);
            return Ok(vec![]);
        }

        // 2. 应用规则匹配
        let matching_rules = self.find_matching_rules(&message).await?;
        if matching_rules.is_empty() {
            log::debug!("No matching rules for notification {}", message.id);
            return Ok(vec![]);
        }

        // 3. 检查频率限制
        if !self.check_rate_limit(&message).await? {
            log::warn!("Rate limit exceeded for notification {}", message.id);
            return Err(anyhow::anyhow!("Rate limit exceeded"));
        }

        // 4. 处理聚合
        let processed_notifications = self.handle_aggregation(message, matching_rules).await?;

        // 5. 记录去重信息
        self.record_for_deduplication(&processed_notifications).await?;

        Ok(processed_notifications)
    }

    /// 验证规则
    fn validate_rule(&self, rule: &NotificationRule) -> anyhow::Result<()> {
        if rule.id.is_empty() {
            return Err(anyhow::anyhow!("Rule ID cannot be empty"));
        }

        if rule.name.is_empty() {
            return Err(anyhow::anyhow!("Rule name cannot be empty"));
        }

        if rule.platforms.is_empty() {
            return Err(anyhow::anyhow!("Rule must specify at least one platform"));
        }

        // 验证条件
        for condition in &rule.conditions {
            self.validate_condition(condition)?;
        }

        // 验证聚合配置
        if let Some(ref aggregation) = rule.aggregation {
            self.validate_aggregation_config(aggregation)?;
        }

        Ok(())
    }

    /// 验证条件
    fn validate_condition(&self, condition: &NotificationCondition) -> anyhow::Result<()> {
        if condition.field.is_empty() {
            return Err(anyhow::anyhow!("Condition field cannot be empty"));
        }

        // 验证字段名是否有效
        let valid_fields = [
            "severity", "score", "project_path", "title", "content",
            "timestamp", "metadata"
        ];

        if !valid_fields.contains(&condition.field.as_str()) && !condition.field.starts_with("metadata.") {
            return Err(anyhow::anyhow!("Invalid condition field: {}", condition.field));
        }

        // 验证正则表达式
        if matches!(condition.operator, ConditionOperator::Regex) {
            if let Some(pattern) = condition.value.as_str() {
                regex::Regex::new(pattern)
                    .map_err(|e| anyhow::anyhow!("Invalid regex pattern: {}", e))?;
            } else {
                return Err(anyhow::anyhow!("Regex condition requires string value"));
            }
        }

        Ok(())
    }

    /// 验证聚合配置
    fn validate_aggregation_config(&self, config: &AggregationConfig) -> anyhow::Result<()> {
        if config.window_duration.as_secs() == 0 {
            return Err(anyhow::anyhow!("Aggregation window duration must be greater than 0"));
        }

        if config.max_messages == 0 {
            return Err(anyhow::anyhow!("Max messages must be greater than 0"));
        }

        if config.group_by.is_empty() {
            return Err(anyhow::anyhow!("Aggregation must specify at least one group_by field"));
        }

        Ok(())
    }

    /// 查找匹配的规则
    async fn find_matching_rules(&self, message: &NotificationMessage) -> anyhow::Result<Vec<NotificationRule>> {
        let rules = self.rules.read().await;
        let mut matching_rules = Vec::new();

        for rule in rules.iter() {
            if !rule.enabled {
                continue;
            }

            if self.evaluate_rule_conditions(rule, message).await? {
                matching_rules.push(rule.clone());
            }
        }

        Ok(matching_rules)
    }

    /// 评估规则条件
    async fn evaluate_rule_conditions(&self, rule: &NotificationRule, message: &NotificationMessage) -> anyhow::Result<bool> {
        // 所有条件都必须满足（AND 逻辑）
        for condition in &rule.conditions {
            if !self.evaluate_condition(condition, message).await? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// 评估单个条件
    async fn evaluate_condition(&self, condition: &NotificationCondition, message: &NotificationMessage) -> anyhow::Result<bool> {
        let field_value = self.extract_field_value(&condition.field, message)?;

        match condition.operator {
            ConditionOperator::Equals => Ok(field_value == condition.value),
            ConditionOperator::NotEquals => Ok(field_value != condition.value),
            ConditionOperator::GreaterThan => {
                if let (Some(field_num), Some(condition_num)) = (field_value.as_f64(), condition.value.as_f64()) {
                    Ok(field_num > condition_num)
                } else {
                    Ok(false)
                }
            },
            ConditionOperator::LessThan => {
                if let (Some(field_num), Some(condition_num)) = (field_value.as_f64(), condition.value.as_f64()) {
                    Ok(field_num < condition_num)
                } else {
                    Ok(false)
                }
            },
            ConditionOperator::Contains => {
                if let (Some(field_str), Some(condition_str)) = (field_value.as_str(), condition.value.as_str()) {
                    Ok(field_str.contains(condition_str))
                } else {
                    Ok(false)
                }
            },
            ConditionOperator::NotContains => {
                if let (Some(field_str), Some(condition_str)) = (field_value.as_str(), condition.value.as_str()) {
                    Ok(!field_str.contains(condition_str))
                } else {
                    Ok(true)
                }
            },
            ConditionOperator::Regex => {
                if let (Some(field_str), Some(pattern)) = (field_value.as_str(), condition.value.as_str()) {
                    match regex::Regex::new(pattern) {
                        Ok(re) => Ok(re.is_match(field_str)),
                        Err(_) => Ok(false),
                    }
                } else {
                    Ok(false)
                }
            },
        }
    }

    /// 提取字段值
    fn extract_field_value(&self, field: &str, message: &NotificationMessage) -> anyhow::Result<serde_json::Value> {
        match field {
            "severity" => Ok(serde_json::to_value(&message.severity)?),
            "score" => Ok(serde_json::to_value(&message.score)?),
            "project_path" => Ok(serde_json::to_value(&message.project_path)?),
            "title" => Ok(serde_json::to_value(&message.title)?),
            "content" => Ok(serde_json::to_value(&message.content)?),
            "timestamp" => Ok(serde_json::to_value(&message.timestamp)?),
            field if field.starts_with("metadata.") => {
                let key = &field[9..]; // Remove "metadata." prefix
                Ok(message.metadata.get(key)
                    .map(|v| serde_json::to_value(v))
                    .transpose()?
                    .unwrap_or(serde_json::Value::Null))
            },
            _ => Ok(serde_json::Value::Null),
        }
    }

    /// 检查是否重复
    async fn is_duplicate(&self, message: &NotificationMessage) -> anyhow::Result<bool> {
        let deduplicator = self.deduplicator.read().await;
        Ok(deduplicator.is_duplicate(message))
    }

    /// 检查频率限制
    async fn check_rate_limit(&self, message: &NotificationMessage) -> anyhow::Result<bool> {
        let mut rate_limiter = self.rate_limiter.write().await;
        Ok(rate_limiter.allow_notification(message))
    }

    /// 处理聚合
    async fn handle_aggregation(&self, message: NotificationMessage, rules: Vec<NotificationRule>) -> anyhow::Result<Vec<ProcessedNotification>> {
        let mut processed_notifications = Vec::new();

        for rule in rules {
            if let Some(ref aggregation_config) = rule.aggregation {
                // 聚合处理
                let mut aggregator = self.aggregator.write().await;
                if let Some(aggregated) = aggregator.add_message(message.clone(), rule.clone(), aggregation_config).await? {
                    processed_notifications.push(aggregated);
                }
            } else {
                // 直接处理
                processed_notifications.push(ProcessedNotification {
                    id: Uuid::new_v4().to_string(),
                    original_message: message.clone(),
                    rule: rule.clone(),
                    platforms: rule.platforms.clone(),
                    aggregated_messages: vec![message.clone()],
                    processed_at: Utc::now(),
                });
            }
        }

        Ok(processed_notifications)
    }

    /// 记录去重信息
    async fn record_for_deduplication(&self, notifications: &[ProcessedNotification]) -> anyhow::Result<()> {
        let mut deduplicator = self.deduplicator.write().await;
        for notification in notifications {
            deduplicator.record_notification(&notification.original_message);
        }
        Ok(())
    }

    /// 获取规则统计信息
    pub async fn get_rule_statistics(&self) -> RuleEngineStatistics {
        let rules = self.rules.read().await;
        let rate_limiter = self.rate_limiter.read().await;
        let deduplicator = self.deduplicator.read().await;
        let aggregator = self.aggregator.read().await;

        RuleEngineStatistics {
            total_rules: rules.len(),
            enabled_rules: rules.iter().filter(|r| r.enabled).count(),
            disabled_rules: rules.iter().filter(|r| !r.enabled).count(),
            total_processed: rate_limiter.get_total_processed(),
            rate_limited: rate_limiter.get_rate_limited_count(),
            duplicates_filtered: deduplicator.get_duplicates_count(),
            aggregated_messages: aggregator.get_aggregated_count(),
        }
    }
}

/// 通知规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationRule {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub enabled: bool,
    pub priority: u32,
    pub conditions: Vec<NotificationCondition>,
    pub platforms: Vec<NotificationPlatform>,
    pub template: Option<String>,
    pub aggregation: Option<AggregationConfig>,
    pub rate_limit: Option<RuleRateLimit>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl NotificationRule {
    pub fn new(id: String, name: String) -> Self {
        let now = Utc::now();
        Self {
            id,
            name,
            description: None,
            enabled: true,
            priority: 100,
            conditions: Vec::new(),
            platforms: Vec::new(),
            template: None,
            aggregation: None,
            rate_limit: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn with_condition(mut self, condition: NotificationCondition) -> Self {
        self.conditions.push(condition);
        self
    }

    pub fn with_platform(mut self, platform: NotificationPlatform) -> Self {
        self.platforms.push(platform);
        self
    }

    pub fn with_aggregation(mut self, aggregation: AggregationConfig) -> Self {
        self.aggregation = Some(aggregation);
        self
    }
}

/// 规则级别的频率限制
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleRateLimit {
    pub max_per_minute: u32,
    pub max_per_hour: u32,
    pub max_per_day: u32,
}

/// 处理后的通知
#[derive(Debug, Clone)]
pub struct ProcessedNotification {
    pub id: String,
    pub original_message: NotificationMessage,
    pub rule: NotificationRule,
    pub platforms: Vec<NotificationPlatform>,
    pub aggregated_messages: Vec<NotificationMessage>,
    pub processed_at: DateTime<Utc>,
}

/// 规则引擎统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleEngineStatistics {
    pub total_rules: usize,
    pub enabled_rules: usize,
    pub disabled_rules: usize,
    pub total_processed: u64,
    pub rate_limited: u64,
    pub duplicates_filtered: u64,
    pub aggregated_messages: u64,
}

/// 通知聚合器
struct NotificationAggregator {
    pending_aggregations: HashMap<String, AggregationGroup>,
}

impl NotificationAggregator {
    fn new() -> Self {
        Self {
            pending_aggregations: HashMap::new(),
        }
    }

    async fn add_message(&mut self, message: NotificationMessage, rule: NotificationRule, config: &AggregationConfig) -> anyhow::Result<Option<ProcessedNotification>> {
        let group_key = self.generate_group_key(&message, &config.group_by)?;

        // 检查是否需要创建新组
        let should_create_group = !self.pending_aggregations.contains_key(&group_key);

        if should_create_group {
            self.pending_aggregations.insert(group_key.clone(), AggregationGroup {
                key: group_key.clone(),
                messages: Vec::new(),
                rule: rule.clone(),
                created_at: Utc::now(),
                config: config.clone(),
            });
        }

        // 获取组的可变引用并添加消息
        {
            let group = self.pending_aggregations.get_mut(&group_key).unwrap();
            group.messages.push(message);
        }

        // 检查是否应该触发聚合
        let should_trigger = {
            let group = self.pending_aggregations.get(&group_key).unwrap();
            self.should_trigger_aggregation(group)
        };

        if should_trigger {
            // 移除组并创建聚合通知
            let group = self.pending_aggregations.remove(&group_key).unwrap();
            let aggregated = self.create_aggregated_notification(&group)?;
            Ok(Some(aggregated))
        } else {
            Ok(None)
        }
    }

    fn generate_group_key(&self, message: &NotificationMessage, group_by: &[String]) -> anyhow::Result<String> {
        let mut key_parts = Vec::new();

        for field in group_by {
            let value = match field.as_str() {
                "project_path" => message.project_path.clone(),
                "severity" => format!("{:?}", message.severity),
                field if field.starts_with("metadata.") => {
                    let key = &field[9..];
                    message.metadata.get(key).cloned().unwrap_or_default()
                },
                _ => "unknown".to_string(),
            };
            key_parts.push(value);
        }

        Ok(key_parts.join(":"))
    }

    fn should_trigger_aggregation(&self, group: &AggregationGroup) -> bool {
        // 检查消息数量
        if group.messages.len() >= group.config.max_messages as usize {
            return true;
        }

        // 检查时间窗口
        let elapsed = Utc::now().signed_duration_since(group.created_at);
        if elapsed.to_std().unwrap_or(Duration::from_secs(0)) >= group.config.window_duration {
            return true;
        }

        false
    }

    fn create_aggregated_notification(&self, group: &AggregationGroup) -> anyhow::Result<ProcessedNotification> {
        let first_message = group.messages.first()
            .ok_or_else(|| anyhow::anyhow!("Empty aggregation group"))?;

        Ok(ProcessedNotification {
            id: Uuid::new_v4().to_string(),
            original_message: first_message.clone(),
            rule: group.rule.clone(),
            platforms: group.rule.platforms.clone(),
            aggregated_messages: group.messages.clone(),
            processed_at: Utc::now(),
        })
    }

    fn get_aggregated_count(&self) -> u64 {
        self.pending_aggregations.values()
            .map(|group| group.messages.len() as u64)
            .sum()
    }
}

/// 聚合组
struct AggregationGroup {
    key: String,
    messages: Vec<NotificationMessage>,
    rule: NotificationRule,
    created_at: DateTime<Utc>,
    config: AggregationConfig,
}

/// 通知频率限制器
struct NotificationRateLimiter {
    request_history: HashMap<String, Vec<DateTime<Utc>>>,
    total_processed: u64,
    rate_limited_count: u64,
}

impl NotificationRateLimiter {
    fn new() -> Self {
        Self {
            request_history: HashMap::new(),
            total_processed: 0,
            rate_limited_count: 0,
        }
    }

    fn allow_notification(&mut self, message: &NotificationMessage) -> bool {
        self.total_processed += 1;

        let key = format!("{}:{:?}", message.project_path, message.severity);
        let now = Utc::now();

        // 清理过期记录
        self.cleanup_expired_records(&now);

        let history = self.request_history.entry(key).or_insert_with(Vec::new);

        // 检查每分钟限制
        let minute_ago = now - chrono::Duration::minutes(1);
        let recent_count = history.iter().filter(|&&timestamp| timestamp > minute_ago).count();

        if recent_count >= 10 { // 每分钟最多10个通知
            self.rate_limited_count += 1;
            return false;
        }

        // 检查每小时限制
        let hour_ago = now - chrono::Duration::hours(1);
        let hourly_count = history.iter().filter(|&&timestamp| timestamp > hour_ago).count();

        if hourly_count >= 100 { // 每小时最多100个通知
            self.rate_limited_count += 1;
            return false;
        }

        history.push(now);
        true
    }

    fn cleanup_expired_records(&mut self, now: &DateTime<Utc>) {
        let hour_ago = *now - chrono::Duration::hours(1);

        for history in self.request_history.values_mut() {
            history.retain(|&timestamp| timestamp > hour_ago);
        }

        // 移除空的历史记录
        self.request_history.retain(|_, history| !history.is_empty());
    }

    fn get_total_processed(&self) -> u64 {
        self.total_processed
    }

    fn get_rate_limited_count(&self) -> u64 {
        self.rate_limited_count
    }
}

/// 通知去重器
struct NotificationDeduplicator {
    message_hashes: HashMap<String, DateTime<Utc>>,
    duplicates_count: u64,
}

impl NotificationDeduplicator {
    fn new() -> Self {
        Self {
            message_hashes: HashMap::new(),
            duplicates_count: 0,
        }
    }

    fn is_duplicate(&self, message: &NotificationMessage) -> bool {
        let hash = self.generate_message_hash(message);

        if let Some(&timestamp) = self.message_hashes.get(&hash) {
            // 检查是否在去重时间窗口内（5分钟）
            let elapsed = Utc::now().signed_duration_since(timestamp);
            elapsed.num_minutes() < 5
        } else {
            false
        }
    }

    fn record_notification(&mut self, message: &NotificationMessage) {
        let hash = self.generate_message_hash(message);

        if self.message_hashes.contains_key(&hash) {
            self.duplicates_count += 1;
        }

        self.message_hashes.insert(hash, Utc::now());

        // 清理过期记录
        self.cleanup_expired_hashes();
    }

    fn generate_message_hash(&self, message: &NotificationMessage) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        message.title.hash(&mut hasher);
        message.content.hash(&mut hasher);
        message.project_path.hash(&mut hasher);
        format!("{:?}", message.severity).hash(&mut hasher);

        format!("{:x}", hasher.finish())
    }

    fn cleanup_expired_hashes(&mut self) {
        let now = Utc::now();
        let cutoff = now - chrono::Duration::minutes(5);

        self.message_hashes.retain(|_, &mut timestamp| timestamp > cutoff);
    }

    fn get_duplicates_count(&self) -> u64 {
        self.duplicates_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rule_engine_creation() {
        let engine = NotificationRuleEngine::new();
        let rules = engine.get_rules().await;
        assert!(rules.is_empty());
    }

    #[tokio::test]
    async fn test_add_rule() {
        let engine = NotificationRuleEngine::new();
        let rule = NotificationRule::new("test-rule".to_string(), "Test Rule".to_string())
            .with_platform(NotificationPlatform::Email);

        engine.add_rule(rule).await.unwrap();
        let rules = engine.get_rules().await;
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].id, "test-rule");
    }

    #[tokio::test]
    async fn test_rule_validation() {
        let engine = NotificationRuleEngine::new();

        // 测试空ID
        let invalid_rule = NotificationRule {
            id: "".to_string(),
            name: "Test".to_string(),
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

        assert!(engine.add_rule(invalid_rule).await.is_err());
    }

    #[tokio::test]
    async fn test_condition_evaluation() {
        let engine = NotificationRuleEngine::new();
        let message = NotificationMessage::new(
            "Test Title".to_string(),
            "Test Content".to_string(),
            NotificationSeverity::Warning,
            "/test/project".to_string(),
        ).with_score(7.5);

        let condition = NotificationCondition {
            field: "score".to_string(),
            operator: ConditionOperator::GreaterThan,
            value: serde_json::json!(7.0),
        };

        let result = engine.evaluate_condition(&condition, &message).await.unwrap();
        assert!(result);
    }

    #[tokio::test]
    async fn test_deduplication() {
        let engine = NotificationRuleEngine::new();
        let message = NotificationMessage::new(
            "Test Title".to_string(),
            "Test Content".to_string(),
            NotificationSeverity::Info,
            "/test/project".to_string(),
        );

        // 第一次应该不是重复
        assert!(!engine.is_duplicate(&message).await.unwrap());

        // 记录消息
        engine.record_for_deduplication(&[ProcessedNotification {
            id: "test".to_string(),
            original_message: message.clone(),
            rule: NotificationRule::new("test".to_string(), "test".to_string()),
            platforms: vec![],
            aggregated_messages: vec![],
            processed_at: Utc::now(),
        }]).await.unwrap();

        // 现在应该是重复
        assert!(engine.is_duplicate(&message).await.unwrap());
    }
}