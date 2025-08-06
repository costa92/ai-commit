use super::detector::WhitelistEntry;
use super::patterns::SensitivePattern;
use super::result::{SensitiveInfoType, RiskLevel};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// 敏感信息检测配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensitiveConfig {
    /// 检测器配置
    pub detector: DetectorConfig,
    /// 白名单配置
    pub whitelist: WhitelistConfig,
    /// 自定义规则配置
    pub custom_rules: CustomRulesConfig,
    /// 脱敏配置
    pub masking: MaskingConfigMap,
    /// 风险评估配置
    pub risk_assessment: RiskAssessmentConfig,
}

impl Default for SensitiveConfig {
    fn default() -> Self {
        Self {
            detector: DetectorConfig::default(),
            whitelist: WhitelistConfig::default(),
            custom_rules: CustomRulesConfig::default(),
            masking: MaskingConfigMap::default(),
            risk_assessment: RiskAssessmentConfig::default(),
        }
    }
}

/// 检测器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectorConfig {
    /// 最大文件大小 (字节)
    pub max_file_size: usize,
    /// 启用白名单
    pub enable_whitelist: bool,
    /// 启用自定义规则
    pub enable_custom_rules: bool,
    /// 置信度阈值
    pub confidence_threshold: f32,
    /// 每个文件最大匹配数
    pub max_matches_per_file: usize,
    /// 启用的预置规则类型
    pub enabled_predefined_types: Vec<SensitiveInfoType>,
    /// 禁用的预置规则名称
    pub disabled_predefined_rules: Vec<String>,
}

impl Default for DetectorConfig {
    fn default() -> Self {
        Self {
            max_file_size: 10 * 1024 * 1024, // 10MB
            enable_whitelist: true,
            enable_custom_rules: true,
            confidence_threshold: 0.5,
            max_matches_per_file: 1000,
            enabled_predefined_types: vec![
                SensitiveInfoType::ApiKey,
                SensitiveInfoType::Password,
                SensitiveInfoType::Token,
                SensitiveInfoType::DatabaseConnection,
                SensitiveInfoType::Email,
                SensitiveInfoType::PhoneNumber,
                SensitiveInfoType::CreditCard,
                SensitiveInfoType::SocialSecurityNumber,
                SensitiveInfoType::PrivateKey,
                SensitiveInfoType::Certificate,
            ],
            disabled_predefined_rules: Vec::new(),
        }
    }
}

/// 白名单配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhitelistConfig {
    /// 启用白名单
    pub enabled: bool,
    /// 白名单条目
    pub entries: Vec<WhitelistEntryConfig>,
    /// 白名单文件路径
    pub whitelist_files: Vec<String>,
}

impl Default for WhitelistConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            entries: Vec::new(),
            whitelist_files: vec![".sensitive-whitelist.json".to_string()],
        }
    }
}

/// 白名单条目配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhitelistEntryConfig {
    /// 匹配模式
    pub pattern: String,
    /// 文件路径模式 (可选)
    pub file_pattern: Option<String>,
    /// 敏感信息类型 (可选)
    pub info_type: Option<SensitiveInfoType>,
    /// 白名单原因
    pub reason: String,
    /// 是否启用
    pub enabled: bool,
    /// 创建时间
    pub created_at: Option<String>,
    /// 创建者
    pub created_by: Option<String>,
}

impl WhitelistEntryConfig {
    pub fn new(pattern: String, reason: String) -> Self {
        Self {
            pattern,
            file_pattern: None,
            info_type: None,
            reason,
            enabled: true,
            created_at: Some(chrono::Utc::now().to_rfc3339()),
            created_by: None,
        }
    }

    pub fn with_file_pattern(mut self, file_pattern: String) -> Self {
        self.file_pattern = Some(file_pattern);
        self
    }

    pub fn with_info_type(mut self, info_type: SensitiveInfoType) -> Self {
        self.info_type = Some(info_type);
        self
    }

    pub fn with_creator(mut self, creator: String) -> Self {
        self.created_by = Some(creator);
        self
    }

    pub fn to_whitelist_entry(&self) -> WhitelistEntry {
        let mut entry = WhitelistEntry::new(self.pattern.clone(), self.reason.clone());
        if let Some(ref file_pattern) = self.file_pattern {
            entry = entry.with_file_pattern(file_pattern.clone());
        }
        entry.enabled = self.enabled;
        entry
    }
}

/// 自定义规则配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomRulesConfig {
    /// 启用自定义规则
    pub enabled: bool,
    /// 自定义规则
    pub rules: Vec<CustomRuleConfig>,
    /// 自定义规则文件路径
    pub rules_files: Vec<String>,
}

impl Default for CustomRulesConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            rules: Vec::new(),
            rules_files: vec![".sensitive-rules.json".to_string()],
        }
    }
}

/// 自定义规则配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomRuleConfig {
    /// 规则名称
    pub name: String,
    /// 敏感信息类型
    pub info_type: SensitiveInfoType,
    /// 正则表达式
    pub regex: String,
    /// 置信度
    pub confidence: f32,
    /// 风险等级
    pub risk_level: RiskLevel,
    /// 描述
    pub description: String,
    /// 建议
    pub recommendations: Vec<String>,
    /// 是否启用
    pub enabled: bool,
    /// 是否区分大小写
    pub case_sensitive: bool,
    /// 创建时间
    pub created_at: Option<String>,
    /// 创建者
    pub created_by: Option<String>,
    /// 标签
    pub tags: Vec<String>,
}

impl CustomRuleConfig {
    pub fn new(
        name: String,
        info_type: SensitiveInfoType,
        regex: String,
        confidence: f32,
        risk_level: RiskLevel,
        description: String,
    ) -> Self {
        Self {
            name,
            info_type,
            regex,
            confidence,
            risk_level,
            description,
            recommendations: Vec::new(),
            enabled: true,
            case_sensitive: true,
            created_at: Some(chrono::Utc::now().to_rfc3339()),
            created_by: None,
            tags: Vec::new(),
        }
    }

    pub fn with_recommendations(mut self, recommendations: Vec<String>) -> Self {
        self.recommendations = recommendations;
        self
    }

    pub fn with_case_sensitivity(mut self, case_sensitive: bool) -> Self {
        self.case_sensitive = case_sensitive;
        self
    }

    pub fn with_creator(mut self, creator: String) -> Self {
        self.created_by = Some(creator);
        self
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    pub fn to_sensitive_pattern(&self) -> anyhow::Result<SensitivePattern> {
        let mut pattern = SensitivePattern::new(
            self.name.clone(),
            self.info_type.clone(),
            self.regex.clone(),
            self.confidence,
            self.risk_level.clone(),
            self.description.clone(),
        )?;

        pattern = pattern
            .with_recommendations(self.recommendations.clone())
            .with_case_sensitivity(self.case_sensitive);

        pattern.enabled = self.enabled;
        Ok(pattern)
    }
}

/// 脱敏配置映射
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaskingConfigMap {
    /// 类型特定的脱敏配置
    pub type_configs: HashMap<SensitiveInfoType, MaskingConfigEntry>,
    /// 默认脱敏配置
    pub default_config: MaskingConfigEntry,
}

impl Default for MaskingConfigMap {
    fn default() -> Self {
        Self {
            type_configs: HashMap::new(),
            default_config: MaskingConfigEntry::default(),
        }
    }
}

/// 脱敏配置条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaskingConfigEntry {
    /// 脱敏算法
    pub algorithm: String,
    /// 脱敏字符
    pub mask_char: char,
    /// 最小可见字符数
    pub min_visible_chars: usize,
    /// 最大可见字符数
    pub max_visible_chars: usize,
    /// 保留格式
    pub preserve_format: bool,
}

impl Default for MaskingConfigEntry {
    fn default() -> Self {
        Self {
            algorithm: "keep_ends".to_string(),
            mask_char: '*',
            min_visible_chars: 2,
            max_visible_chars: 8,
            preserve_format: true,
        }
    }
}

/// 风险评估配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessmentConfig {
    /// 启用风险评估
    pub enabled: bool,
    /// 类型权重
    pub type_weights: HashMap<SensitiveInfoType, f32>,
    /// 风险阈值
    pub risk_thresholds: RiskThresholdsConfig,
}

impl Default for RiskAssessmentConfig {
    fn default() -> Self {
        let mut type_weights = HashMap::new();
        type_weights.insert(SensitiveInfoType::PrivateKey, 10.0);
        type_weights.insert(SensitiveInfoType::SocialSecurityNumber, 9.0);
        type_weights.insert(SensitiveInfoType::CreditCard, 9.0);
        type_weights.insert(SensitiveInfoType::ApiKey, 8.0);
        type_weights.insert(SensitiveInfoType::Password, 8.0);
        type_weights.insert(SensitiveInfoType::DatabaseConnection, 8.0);
        type_weights.insert(SensitiveInfoType::Token, 7.0);
        type_weights.insert(SensitiveInfoType::Certificate, 5.0);
        type_weights.insert(SensitiveInfoType::PhoneNumber, 4.0);
        type_weights.insert(SensitiveInfoType::Email, 3.0);

        Self {
            enabled: true,
            type_weights,
            risk_thresholds: RiskThresholdsConfig::default(),
        }
    }
}

/// 风险阈值配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskThresholdsConfig {
    pub critical_threshold: f32,
    pub high_threshold: f32,
    pub medium_threshold: f32,
}

impl Default for RiskThresholdsConfig {
    fn default() -> Self {
        Self {
            critical_threshold: 8.0,
            high_threshold: 6.0,
            medium_threshold: 3.0,
        }
    }
}

/// 配置管理器
pub struct ConfigManager {
    config: SensitiveConfig,
    config_path: Option<String>,
}

impl ConfigManager {
    pub fn new() -> Self {
        Self {
            config: SensitiveConfig::default(),
            config_path: None,
        }
    }

    pub fn with_config(config: SensitiveConfig) -> Self {
        Self {
            config,
            config_path: None,
        }
    }

    pub fn load_from_file<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let path_str = path.as_ref().to_string_lossy().to_string();
        let content = fs::read_to_string(&path)?;
        let config: SensitiveConfig = serde_json::from_str(&content)?;

        Ok(Self {
            config,
            config_path: Some(path_str),
        })
    }

    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> anyhow::Result<()> {
        let content = serde_json::to_string_pretty(&self.config)?;
        fs::write(path, content)?;
        Ok(())
    }

    pub fn save(&self) -> anyhow::Result<()> {
        if let Some(ref path) = self.config_path {
            self.save_to_file(path)
        } else {
            anyhow::bail!("No config path specified")
        }
    }

    pub fn get_config(&self) -> &SensitiveConfig {
        &self.config
    }

    pub fn get_config_mut(&mut self) -> &mut SensitiveConfig {
        &mut self.config
    }

    pub fn add_whitelist_entry(&mut self, entry: WhitelistEntryConfig) {
        self.config.whitelist.entries.push(entry);
    }

    pub fn remove_whitelist_entry(&mut self, pattern: &str) -> bool {
        let initial_len = self.config.whitelist.entries.len();
        self.config.whitelist.entries.retain(|entry| entry.pattern != pattern);
        self.config.whitelist.entries.len() != initial_len
    }

    pub fn add_custom_rule(&mut self, rule: CustomRuleConfig) {
        self.config.custom_rules.rules.push(rule);
    }

    pub fn remove_custom_rule(&mut self, name: &str) -> bool {
        let initial_len = self.config.custom_rules.rules.len();
        self.config.custom_rules.rules.retain(|rule| rule.name != name);
        self.config.custom_rules.rules.len() != initial_len
    }

    pub fn enable_rule(&mut self, name: &str) -> bool {
        if let Some(rule) = self.config.custom_rules.rules.iter_mut().find(|r| r.name == name) {
            rule.enabled = true;
            return true;
        }
        false
    }

    pub fn disable_rule(&mut self, name: &str) -> bool {
        if let Some(rule) = self.config.custom_rules.rules.iter_mut().find(|r| r.name == name) {
            rule.enabled = false;
            return true;
        }
        false
    }

    pub fn get_whitelist_entries(&self) -> Vec<WhitelistEntry> {
        self.config.whitelist.entries.iter()
            .filter(|entry| entry.enabled)
            .map(|entry| entry.to_whitelist_entry())
            .collect()
    }

    pub fn get_custom_patterns(&self) -> anyhow::Result<Vec<SensitivePattern>> {
        let mut patterns = Vec::new();
        for rule in &self.config.custom_rules.rules {
            if rule.enabled {
                patterns.push(rule.to_sensitive_pattern()?);
            }
        }
        Ok(patterns)
    }

    pub fn load_external_whitelist(&mut self) -> anyhow::Result<()> {
        for file_path in &self.config.whitelist.whitelist_files {
            if Path::new(file_path).exists() {
                let content = fs::read_to_string(file_path)?;
                let entries: Vec<WhitelistEntryConfig> = serde_json::from_str(&content)?;
                self.config.whitelist.entries.extend(entries);
            }
        }
        Ok(())
    }

    pub fn load_external_rules(&mut self) -> anyhow::Result<()> {
        for file_path in &self.config.custom_rules.rules_files {
            if Path::new(file_path).exists() {
                let content = fs::read_to_string(file_path)?;
                let rules: Vec<CustomRuleConfig> = serde_json::from_str(&content)?;
                self.config.custom_rules.rules.extend(rules);
            }
        }
        Ok(())
    }

    pub fn validate_config(&self) -> anyhow::Result<Vec<ConfigValidationError>> {
        let mut errors = Vec::new();

        // 验证检测器配置
        if self.config.detector.confidence_threshold < 0.0 || self.config.detector.confidence_threshold > 1.0 {
            errors.push(ConfigValidationError {
                field: "detector.confidence_threshold".to_string(),
                message: "置信度阈值必须在 0.0 到 1.0 之间".to_string(),
            });
        }

        if self.config.detector.max_file_size == 0 {
            errors.push(ConfigValidationError {
                field: "detector.max_file_size".to_string(),
                message: "最大文件大小必须大于 0".to_string(),
            });
        }

        // 验证自定义规则
        for (i, rule) in self.config.custom_rules.rules.iter().enumerate() {
            if rule.name.is_empty() {
                errors.push(ConfigValidationError {
                    field: format!("custom_rules.rules[{}].name", i),
                    message: "规则名称不能为空".to_string(),
                });
            }

            if rule.regex.is_empty() {
                errors.push(ConfigValidationError {
                    field: format!("custom_rules.rules[{}].regex", i),
                    message: "正则表达式不能为空".to_string(),
                });
            }

            // 验证正则表达式语法
            if let Err(e) = regex::Regex::new(&rule.regex) {
                errors.push(ConfigValidationError {
                    field: format!("custom_rules.rules[{}].regex", i),
                    message: format!("正则表达式语法错误: {}", e),
                });
            }

            if rule.confidence < 0.0 || rule.confidence > 1.0 {
                errors.push(ConfigValidationError {
                    field: format!("custom_rules.rules[{}].confidence", i),
                    message: "置信度必须在 0.0 到 1.0 之间".to_string(),
                });
            }
        }

        // 验证白名单条目
        for (i, entry) in self.config.whitelist.entries.iter().enumerate() {
            if entry.pattern.is_empty() {
                errors.push(ConfigValidationError {
                    field: format!("whitelist.entries[{}].pattern", i),
                    message: "白名单模式不能为空".to_string(),
                });
            }

            if entry.reason.is_empty() {
                errors.push(ConfigValidationError {
                    field: format!("whitelist.entries[{}].reason", i),
                    message: "白名单原因不能为空".to_string(),
                });
            }
        }

        Ok(errors)
    }

    pub fn generate_example_config() -> SensitiveConfig {
        let mut config = SensitiveConfig::default();

        // 添加示例白名单条目
        config.whitelist.entries.push(
            WhitelistEntryConfig::new(
                "test_api_key_12345".to_string(),
                "测试用的 API 密钥".to_string(),
            )
            .with_file_pattern("test/".to_string())
            .with_info_type(SensitiveInfoType::ApiKey)
        );

        // 添加示例自定义规则
        config.custom_rules.rules.push(
            CustomRuleConfig::new(
                "company_internal_token".to_string(),
                SensitiveInfoType::Token,
                r"COMP_[A-Z0-9]{32}".to_string(),
                0.95,
                RiskLevel::High,
                "公司内部令牌".to_string(),
            )
            .with_recommendations(vec![
                "使用环境变量存储令牌".to_string(),
                "定期轮换令牌".to_string(),
            ])
            .with_tags(vec!["internal".to_string(), "token".to_string()])
        );

        config
    }
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 配置验证错误
#[derive(Debug, Clone)]
pub struct ConfigValidationError {
    pub field: String,
    pub message: String,
}

impl std::fmt::Display for ConfigValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.field, self.message)
    }
}

impl std::error::Error for ConfigValidationError {}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_config_creation() {
        let config = SensitiveConfig::default();
        assert!(config.detector.enable_whitelist);
        assert!(config.detector.enable_custom_rules);
        assert!(!config.detector.enabled_predefined_types.is_empty());
    }

    #[test]
    fn test_whitelist_entry_config() {
        let entry = WhitelistEntryConfig::new(
            "test_pattern".to_string(),
            "test reason".to_string(),
        )
        .with_file_pattern("test/".to_string())
        .with_info_type(SensitiveInfoType::ApiKey);

        assert_eq!(entry.pattern, "test_pattern");
        assert_eq!(entry.file_pattern, Some("test/".to_string()));
        assert_eq!(entry.info_type, Some(SensitiveInfoType::ApiKey));
        assert!(entry.enabled);
    }

    #[test]
    fn test_custom_rule_config() {
        let rule = CustomRuleConfig::new(
            "test_rule".to_string(),
            SensitiveInfoType::Token,
            r"TEST_[0-9]+".to_string(),
            0.9,
            RiskLevel::High,
            "Test rule".to_string(),
        )
        .with_recommendations(vec!["Use env vars".to_string()])
        .with_tags(vec!["test".to_string()]);

        assert_eq!(rule.name, "test_rule");
        assert_eq!(rule.confidence, 0.9);
        assert_eq!(rule.recommendations.len(), 1);
        assert_eq!(rule.tags.len(), 1);

        let pattern = rule.to_sensitive_pattern().unwrap();
        assert_eq!(pattern.name, "test_rule");
        assert_eq!(pattern.confidence, 0.9);
    }

    #[test]
    fn test_config_manager() {
        let mut manager = ConfigManager::new();

        // 添加白名单条目
        let whitelist_entry = WhitelistEntryConfig::new(
            "test_key".to_string(),
            "Test key".to_string(),
        );
        manager.add_whitelist_entry(whitelist_entry);

        // 添加自定义规则
        let custom_rule = CustomRuleConfig::new(
            "test_rule".to_string(),
            SensitiveInfoType::ApiKey,
            r"TEST_[A-Z0-9]{16}".to_string(),
            0.95,
            RiskLevel::High,
            "Test API key".to_string(),
        );
        manager.add_custom_rule(custom_rule);

        assert_eq!(manager.config.whitelist.entries.len(), 1);
        assert_eq!(manager.config.custom_rules.rules.len(), 1);

        // 测试删除
        assert!(manager.remove_whitelist_entry("test_key"));
        assert!(manager.remove_custom_rule("test_rule"));
        assert_eq!(manager.config.whitelist.entries.len(), 0);
        assert_eq!(manager.config.custom_rules.rules.len(), 0);
    }

    #[test]
    fn test_config_serialization() {
        let config = SensitiveConfig::default();
        let json = serde_json::to_string_pretty(&config).unwrap();
        assert!(json.contains("detector"));
        assert!(json.contains("whitelist"));
        assert!(json.contains("custom_rules"));

        let deserialized: SensitiveConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.detector.max_file_size, config.detector.max_file_size);
    }

    #[test]
    fn test_config_file_operations() {
        let config = SensitiveConfig::default();
        let temp_file = NamedTempFile::new().unwrap();
        let temp_path = temp_file.path();

        // 保存配置
        let manager = ConfigManager::with_config(config);
        manager.save_to_file(temp_path).unwrap();

        // 加载配置
        let loaded_manager = ConfigManager::load_from_file(temp_path).unwrap();
        assert_eq!(
            loaded_manager.config.detector.max_file_size,
            manager.config.detector.max_file_size
        );
    }

    #[test]
    fn test_config_validation() {
        let mut config = SensitiveConfig::default();

        // 添加无效配置
        config.detector.confidence_threshold = 1.5; // 无效值
        config.custom_rules.rules.push(CustomRuleConfig {
            name: "".to_string(), // 空名称
            info_type: SensitiveInfoType::ApiKey,
            regex: "[".to_string(), // 无效正则表达式
            confidence: -0.1, // 无效置信度
            risk_level: RiskLevel::High,
            description: "Test".to_string(),
            recommendations: Vec::new(),
            enabled: true,
            case_sensitive: true,
            created_at: None,
            created_by: None,
            tags: Vec::new(),
        });

        let manager = ConfigManager::with_config(config);
        let errors = manager.validate_config().unwrap();

        assert!(!errors.is_empty());
        assert!(errors.iter().any(|e| e.field.contains("confidence_threshold")));
        assert!(errors.iter().any(|e| e.field.contains("name")));
        assert!(errors.iter().any(|e| e.field.contains("regex")));
    }

    #[test]
    fn test_example_config_generation() {
        let config = ConfigManager::generate_example_config();
        assert!(!config.whitelist.entries.is_empty());
        assert!(!config.custom_rules.rules.is_empty());

        let whitelist_entry = &config.whitelist.entries[0];
        assert_eq!(whitelist_entry.pattern, "test_api_key_12345");
        assert!(whitelist_entry.file_pattern.is_some());

        let custom_rule = &config.custom_rules.rules[0];
        assert_eq!(custom_rule.name, "company_internal_token");
        assert!(!custom_rule.recommendations.is_empty());
        assert!(!custom_rule.tags.is_empty());
    }
}