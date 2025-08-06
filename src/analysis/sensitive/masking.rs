use super::result::{SensitiveInfoType, RiskLevel, SensitiveItem};
use std::collections::HashMap;

/// 脱敏算法类型
#[derive(Debug, Clone, PartialEq)]
pub enum MaskingAlgorithm {
    /// 保留前后字符，中间用星号替换
    KeepEnds,
    /// 完全替换为星号
    FullMask,
    /// 保留格式，替换内容
    FormatPreserving,
    /// 自定义脱敏
    Custom(String),
}

/// 脱敏配置
#[derive(Debug, Clone)]
pub struct MaskingConfig {
    pub algorithm: MaskingAlgorithm,
    pub mask_char: char,
    pub min_visible_chars: usize,
    pub max_visible_chars: usize,
    pub preserve_format: bool,
}

impl Default for MaskingConfig {
    fn default() -> Self {
        Self {
            algorithm: MaskingAlgorithm::KeepEnds,
            mask_char: '*',
            min_visible_chars: 2,
            max_visible_chars: 8,
            preserve_format: true,
        }
    }
}

/// 敏感信息脱敏器
pub struct SensitiveInfoMasker {
    type_configs: HashMap<SensitiveInfoType, MaskingConfig>,
    default_config: MaskingConfig,
}

impl SensitiveInfoMasker {
    pub fn new() -> Self {
        let mut masker = Self {
            type_configs: HashMap::new(),
            default_config: MaskingConfig::default(),
        };

        // 为不同类型设置默认脱敏配置
        masker.setup_default_configs();
        masker
    }

    fn setup_default_configs(&mut self) {
        // API 密钥 - 保留前后4个字符
        self.type_configs.insert(
            SensitiveInfoType::ApiKey,
            MaskingConfig {
                algorithm: MaskingAlgorithm::KeepEnds,
                mask_char: '*',
                min_visible_chars: 4,
                max_visible_chars: 8,
                preserve_format: false,
            },
        );

        // 密码 - 完全脱敏
        self.type_configs.insert(
            SensitiveInfoType::Password,
            MaskingConfig {
                algorithm: MaskingAlgorithm::FullMask,
                mask_char: '*',
                min_visible_chars: 0,
                max_visible_chars: 0,
                preserve_format: false,
            },
        );

        // 令牌 - 保留前后字符
        self.type_configs.insert(
            SensitiveInfoType::Token,
            MaskingConfig {
                algorithm: MaskingAlgorithm::KeepEnds,
                mask_char: '*',
                min_visible_chars: 4,
                max_visible_chars: 8,
                preserve_format: false,
            },
        );

        // 邮箱 - 保留格式
        self.type_configs.insert(
            SensitiveInfoType::Email,
            MaskingConfig {
                algorithm: MaskingAlgorithm::FormatPreserving,
                mask_char: '*',
                min_visible_chars: 2,
                max_visible_chars: 4,
                preserve_format: true,
            },
        );

        // 手机号 - 保留格式
        self.type_configs.insert(
            SensitiveInfoType::PhoneNumber,
            MaskingConfig {
                algorithm: MaskingAlgorithm::FormatPreserving,
                mask_char: '*',
                min_visible_chars: 3,
                max_visible_chars: 4,
                preserve_format: true,
            },
        );

        // 信用卡 - 保留后4位
        self.type_configs.insert(
            SensitiveInfoType::CreditCard,
            MaskingConfig {
                algorithm: MaskingAlgorithm::Custom("****-****-****-{last4}".to_string()),
                mask_char: '*',
                min_visible_chars: 4,
                max_visible_chars: 4,
                preserve_format: true,
            },
        );

        // 身份证号 - 保留前后字符
        self.type_configs.insert(
            SensitiveInfoType::SocialSecurityNumber,
            MaskingConfig {
                algorithm: MaskingAlgorithm::KeepEnds,
                mask_char: '*',
                min_visible_chars: 3,
                max_visible_chars: 6,
                preserve_format: false,
            },
        );

        // 私钥 - 完全脱敏
        self.type_configs.insert(
            SensitiveInfoType::PrivateKey,
            MaskingConfig {
                algorithm: MaskingAlgorithm::FullMask,
                mask_char: '*',
                min_visible_chars: 0,
                max_visible_chars: 0,
                preserve_format: false,
            },
        );
    }

    pub fn mask_text(&self, text: &str, info_type: &SensitiveInfoType) -> String {
        let config = self.type_configs.get(info_type).unwrap_or(&self.default_config);

        match &config.algorithm {
            MaskingAlgorithm::KeepEnds => self.mask_keep_ends(text, config),
            MaskingAlgorithm::FullMask => self.mask_full(text, config),
            MaskingAlgorithm::FormatPreserving => self.mask_format_preserving(text, info_type, config),
            MaskingAlgorithm::Custom(pattern) => self.mask_custom(text, pattern, config),
        }
    }

    fn mask_keep_ends(&self, text: &str, config: &MaskingConfig) -> String {
        if text.len() <= config.min_visible_chars * 2 {
            return config.mask_char.to_string().repeat(text.len().min(12));
        }

        let visible_chars = (config.min_visible_chars).min(text.len() / 3);
        let start = &text[..visible_chars];
        let end = &text[text.len() - visible_chars..];
        let middle_len = text.len() - visible_chars * 2;
        let middle = config.mask_char.to_string().repeat(middle_len.min(20));

        format!("{}{}{}", start, middle, end)
    }

    fn mask_full(&self, text: &str, config: &MaskingConfig) -> String {
        config.mask_char.to_string().repeat(text.len().min(20))
    }

    fn mask_format_preserving(&self, text: &str, info_type: &SensitiveInfoType, config: &MaskingConfig) -> String {
        match info_type {
            SensitiveInfoType::Email => self.mask_email(text, config),
            SensitiveInfoType::PhoneNumber => self.mask_phone(text, config),
            _ => self.mask_keep_ends(text, config),
        }
    }

    fn mask_email(&self, text: &str, config: &MaskingConfig) -> String {
        if let Some(at_pos) = text.find('@') {
            let (local, domain) = text.split_at(at_pos);
            if local.len() <= config.min_visible_chars {
                format!("{}{}", config.mask_char.to_string().repeat(3), domain)
            } else {
                let visible = config.min_visible_chars.min(local.len() / 2);
                let start = &local[..visible];
                let masked_len = local.len() - visible;
                let masked = config.mask_char.to_string().repeat(masked_len.min(10));
                format!("{}{}{}", start, masked, domain)
            }
        } else {
            self.mask_keep_ends(text, config)
        }
    }

    fn mask_phone(&self, text: &str, config: &MaskingConfig) -> String {
        // 移除所有非数字字符来检测格式
        let digits: String = text.chars().filter(|c| c.is_ascii_digit()).collect();

        if digits.len() >= 10 {
            // 保留后4位数字
            let visible_digits = 4.min(digits.len());
            let last_digits = &digits[digits.len() - visible_digits..];

            // 根据原始格式重建
            if text.contains('-') {
                format!("***-***-{}", last_digits)
            } else if text.contains(' ') {
                format!("*** *** {}", last_digits)
            } else if text.contains('(') && text.contains(')') {
                format!("(***) ***-{}", last_digits)
            } else {
                format!("******{}", last_digits)
            }
        } else {
            self.mask_keep_ends(text, config)
        }
    }

    fn mask_custom(&self, text: &str, pattern: &str, _config: &MaskingConfig) -> String {
        if pattern.contains("{last4}") && text.len() >= 4 {
            let last4 = &text[text.len() - 4..];
            pattern.replace("{last4}", last4)
        } else if pattern.contains("{first4}") && text.len() >= 4 {
            let first4 = &text[..4];
            pattern.replace("{first4}", first4)
        } else {
            pattern.to_string()
        }
    }

    pub fn set_config(&mut self, info_type: SensitiveInfoType, config: MaskingConfig) {
        self.type_configs.insert(info_type, config);
    }

    pub fn get_config(&self, info_type: &SensitiveInfoType) -> &MaskingConfig {
        self.type_configs.get(info_type).unwrap_or(&self.default_config)
    }
}

impl Default for SensitiveInfoMasker {
    fn default() -> Self {
        Self::new()
    }
}

/// 风险评估器
pub struct RiskAssessor {
    type_weights: HashMap<SensitiveInfoType, f32>,
    risk_thresholds: RiskThresholds,
}

/// 风险阈值配置
#[derive(Debug, Clone)]
pub struct RiskThresholds {
    pub critical_threshold: f32,
    pub high_threshold: f32,
    pub medium_threshold: f32,
}

impl Default for RiskThresholds {
    fn default() -> Self {
        Self {
            critical_threshold: 8.0,
            high_threshold: 6.0,
            medium_threshold: 3.0,
        }
    }
}

impl RiskAssessor {
    pub fn new() -> Self {
        let mut assessor = Self {
            type_weights: HashMap::new(),
            risk_thresholds: RiskThresholds::default(),
        };

        assessor.setup_default_weights();
        assessor
    }

    fn setup_default_weights(&mut self) {
        // 设置不同敏感信息类型的权重
        self.type_weights.insert(SensitiveInfoType::PrivateKey, 10.0);
        self.type_weights.insert(SensitiveInfoType::SocialSecurityNumber, 9.0);
        self.type_weights.insert(SensitiveInfoType::CreditCard, 9.0);
        self.type_weights.insert(SensitiveInfoType::ApiKey, 8.0);
        self.type_weights.insert(SensitiveInfoType::Password, 8.0);
        self.type_weights.insert(SensitiveInfoType::DatabaseConnection, 8.0);
        self.type_weights.insert(SensitiveInfoType::Token, 7.0);
        self.type_weights.insert(SensitiveInfoType::Certificate, 5.0);
        self.type_weights.insert(SensitiveInfoType::PhoneNumber, 4.0);
        self.type_weights.insert(SensitiveInfoType::Email, 3.0);
    }

    pub fn assess_risk(&self, items: &[SensitiveItem]) -> RiskAssessment {
        let mut total_score = 0.0;
        let mut type_scores = HashMap::new();
        let mut severity_counts = HashMap::new();

        for item in items {
            let base_weight = self.type_weights.get(&item.info_type).copied().unwrap_or(5.0);
            let confidence_factor = item.confidence;
            let item_score = base_weight * confidence_factor;

            total_score += item_score;
            *type_scores.entry(item.info_type.clone()).or_insert(0.0) += item_score;
            *severity_counts.entry(item.risk_level.clone()).or_insert(0) += 1;
        }

        let average_score = if items.is_empty() { 0.0 } else { total_score / items.len() as f32 };
        let overall_risk_level = self.calculate_risk_level(total_score);

        RiskAssessment {
            total_score,
            average_score,
            overall_risk_level,
            type_scores,
            severity_counts,
            recommendations: self.generate_recommendations(items, total_score),
        }
    }

    fn calculate_risk_level(&self, score: f32) -> RiskLevel {
        if score >= self.risk_thresholds.critical_threshold {
            RiskLevel::Critical
        } else if score >= self.risk_thresholds.high_threshold {
            RiskLevel::High
        } else if score >= self.risk_thresholds.medium_threshold {
            RiskLevel::Medium
        } else {
            RiskLevel::Low
        }
    }

    fn generate_recommendations(&self, items: &[SensitiveItem], total_score: f32) -> Vec<SecurityRecommendation> {
        let mut recommendations = Vec::new();

        // 基于总体风险评分的建议
        if total_score >= self.risk_thresholds.critical_threshold {
            recommendations.push(SecurityRecommendation {
                priority: RecommendationPriority::Critical,
                category: RecommendationCategory::Immediate,
                title: "立即处理关键安全风险".to_string(),
                description: "检测到关键级别的敏感信息泄露，需要立即采取行动".to_string(),
                actions: vec![
                    "立即从代码库中删除所有敏感信息".to_string(),
                    "轮换所有暴露的密钥和凭证".to_string(),
                    "审查代码提交历史".to_string(),
                    "通知安全团队".to_string(),
                ],
            });
        }

        // 基于敏感信息类型的建议
        let mut type_counts = HashMap::new();
        for item in items {
            *type_counts.entry(&item.info_type).or_insert(0) += 1;
        }

        for (info_type, count) in type_counts {
            match info_type {
                SensitiveInfoType::ApiKey => {
                    recommendations.push(SecurityRecommendation {
                        priority: RecommendationPriority::High,
                        category: RecommendationCategory::KeyManagement,
                        title: format!("API 密钥管理 ({} 个发现)", count),
                        description: "发现硬编码的 API 密钥".to_string(),
                        actions: vec![
                            "使用环境变量或密钥管理服务".to_string(),
                            "实施密钥轮换策略".to_string(),
                            "限制 API 密钥权限".to_string(),
                        ],
                    });
                },
                SensitiveInfoType::Password => {
                    recommendations.push(SecurityRecommendation {
                        priority: RecommendationPriority::Critical,
                        category: RecommendationCategory::Authentication,
                        title: format!("密码安全 ({} 个发现)", count),
                        description: "发现硬编码的密码".to_string(),
                        actions: vec![
                            "立即更换所有暴露的密码".to_string(),
                            "使用安全的密码存储方案".to_string(),
                            "实施多因素认证".to_string(),
                        ],
                    });
                },
                SensitiveInfoType::DatabaseConnection => {
                    recommendations.push(SecurityRecommendation {
                        priority: RecommendationPriority::Critical,
                        category: RecommendationCategory::DataProtection,
                        title: format!("数据库安全 ({} 个发现)", count),
                        description: "发现数据库连接字符串".to_string(),
                        actions: vec![
                            "使用连接池和环境变量".to_string(),
                            "启用数据库加密".to_string(),
                            "限制数据库访问权限".to_string(),
                        ],
                    });
                },
                SensitiveInfoType::PrivateKey => {
                    recommendations.push(SecurityRecommendation {
                        priority: RecommendationPriority::Critical,
                        category: RecommendationCategory::KeyManagement,
                        title: format!("私钥安全 ({} 个发现)", count),
                        description: "发现私钥文件".to_string(),
                        actions: vec![
                            "立即轮换所有暴露的私钥".to_string(),
                            "使用硬件安全模块".to_string(),
                            "实施私钥访问控制".to_string(),
                        ],
                    });
                },
                _ => {}
            }
        }

        // 通用安全建议
        if !items.is_empty() {
            recommendations.push(SecurityRecommendation {
                priority: RecommendationPriority::Medium,
                category: RecommendationCategory::Prevention,
                title: "预防措施".to_string(),
                description: "建立预防敏感信息泄露的机制".to_string(),
                actions: vec![
                    "配置 Git hooks 进行预提交检查".to_string(),
                    "使用代码扫描工具".to_string(),
                    "建立安全编码培训".to_string(),
                    "定期进行安全审计".to_string(),
                ],
            });
        }

        recommendations
    }

    pub fn set_weight(&mut self, info_type: SensitiveInfoType, weight: f32) {
        self.type_weights.insert(info_type, weight);
    }

    pub fn set_thresholds(&mut self, thresholds: RiskThresholds) {
        self.risk_thresholds = thresholds;
    }
}

impl Default for RiskAssessor {
    fn default() -> Self {
        Self::new()
    }
}

/// 风险评估结果
#[derive(Debug, Clone)]
pub struct RiskAssessment {
    pub total_score: f32,
    pub average_score: f32,
    pub overall_risk_level: RiskLevel,
    pub type_scores: HashMap<SensitiveInfoType, f32>,
    pub severity_counts: HashMap<RiskLevel, usize>,
    pub recommendations: Vec<SecurityRecommendation>,
}

impl RiskAssessment {
    pub fn is_high_risk(&self) -> bool {
        matches!(self.overall_risk_level, RiskLevel::Critical | RiskLevel::High)
    }

    pub fn get_critical_recommendations(&self) -> Vec<&SecurityRecommendation> {
        self.recommendations.iter()
            .filter(|r| r.priority == RecommendationPriority::Critical)
            .collect()
    }

    pub fn get_summary(&self) -> String {
        format!(
            "风险评分: {:.1}/10.0, 风险等级: {}, 发现 {} 条安全建议",
            self.total_score,
            self.overall_risk_level,
            self.recommendations.len()
        )
    }
}

/// 安全建议
#[derive(Debug, Clone)]
pub struct SecurityRecommendation {
    pub priority: RecommendationPriority,
    pub category: RecommendationCategory,
    pub title: String,
    pub description: String,
    pub actions: Vec<String>,
}

/// 建议优先级
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecommendationPriority {
    Critical,
    High,
    Medium,
    Low,
}

/// 建议类别
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecommendationCategory {
    Immediate,
    KeyManagement,
    Authentication,
    DataProtection,
    Prevention,
    Compliance,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_masker_creation() {
        let masker = SensitiveInfoMasker::new();
        assert!(!masker.type_configs.is_empty());
    }

    #[test]
    fn test_api_key_masking() {
        let masker = SensitiveInfoMasker::new();
        let api_key = "AKIA1234567890ABCDEF";
        let masked = masker.mask_text(api_key, &SensitiveInfoType::ApiKey);

        assert!(masked.starts_with("AKIA"));
        assert!(masked.ends_with("CDEF"));
        assert!(masked.contains("*"));
        assert_eq!(masked.len(), api_key.len());
    }

    #[test]
    fn test_password_masking() {
        let masker = SensitiveInfoMasker::new();
        let password = "mySecretPassword123";
        let masked = masker.mask_text(password, &SensitiveInfoType::Password);

        assert_eq!(masked, "*".repeat(password.len().min(20)));
        assert!(!masked.contains("Secret"));
    }

    #[test]
    fn test_email_masking() {
        let masker = SensitiveInfoMasker::new();
        let email = "user@example.com";
        let masked = masker.mask_text(email, &SensitiveInfoType::Email);

        assert!(masked.contains("@example.com"));
        assert!(masked.starts_with("us") || masked.starts_with("u*"));
        assert!(masked.contains("*"));
    }

    #[test]
    fn test_phone_masking() {
        let masker = SensitiveInfoMasker::new();
        let phone = "123-456-7890";
        let masked = masker.mask_text(phone, &SensitiveInfoType::PhoneNumber);

        assert!(masked.ends_with("7890"));
        assert!(masked.contains("***"));
        assert!(masked.contains("-"));
    }

    #[test]
    fn test_credit_card_masking() {
        let masker = SensitiveInfoMasker::new();
        let card = "4111111111111111";
        let masked = masker.mask_text(card, &SensitiveInfoType::CreditCard);

        assert!(masked.ends_with("1111"));
        assert!(masked.contains("****"));
    }

    #[test]
    fn test_risk_assessor_creation() {
        let assessor = RiskAssessor::new();
        assert!(!assessor.type_weights.is_empty());
    }

    #[test]
    fn test_risk_assessment() {
        let assessor = RiskAssessor::new();
        let items = vec![
            SensitiveItem::new(
                SensitiveInfoType::ApiKey,
                1, 10, 30,
                "AKIA1234567890ABCDEF".to_string(),
                0.95,
                RiskLevel::Critical,
                "aws_access_key".to_string(),
            ),
            SensitiveItem::new(
                SensitiveInfoType::Password,
                2, 15, 25,
                "password123".to_string(),
                0.90,
                RiskLevel::High,
                "generic_password".to_string(),
            ),
        ];

        let assessment = assessor.assess_risk(&items);

        assert!(assessment.total_score > 0.0);
        assert!(assessment.is_high_risk());
        assert!(!assessment.recommendations.is_empty());
        assert!(assessment.type_scores.contains_key(&SensitiveInfoType::ApiKey));
        assert!(assessment.type_scores.contains_key(&SensitiveInfoType::Password));
    }

    #[test]
    fn test_risk_level_calculation() {
        let assessor = RiskAssessor::new();

        // 测试不同风险等级
        assert_eq!(assessor.calculate_risk_level(10.0), RiskLevel::Critical);
        assert_eq!(assessor.calculate_risk_level(7.0), RiskLevel::High);
        assert_eq!(assessor.calculate_risk_level(4.0), RiskLevel::Medium);
        assert_eq!(assessor.calculate_risk_level(1.0), RiskLevel::Low);
    }

    #[test]
    fn test_recommendations_generation() {
        let assessor = RiskAssessor::new();
        let items = vec![
            SensitiveItem::new(
                SensitiveInfoType::PrivateKey,
                1, 1, 100,
                "-----BEGIN RSA PRIVATE KEY-----".to_string(),
                0.99,
                RiskLevel::Critical,
                "rsa_private_key".to_string(),
            ),
        ];

        let assessment = assessor.assess_risk(&items);
        let critical_recs = assessment.get_critical_recommendations();

        assert!(!critical_recs.is_empty());
        assert!(critical_recs.iter().any(|r| r.title.contains("立即处理")));
    }

    #[test]
    fn test_custom_masking_config() {
        let mut masker = SensitiveInfoMasker::new();

        let custom_config = MaskingConfig {
            algorithm: MaskingAlgorithm::Custom("XXXX-XXXX-XXXX-{last4}".to_string()),
            mask_char: 'X',
            min_visible_chars: 4,
            max_visible_chars: 4,
            preserve_format: true,
        };

        masker.set_config(SensitiveInfoType::CreditCard, custom_config);

        let card = "1234567890123456";
        let masked = masker.mask_text(card, &SensitiveInfoType::CreditCard);

        assert_eq!(masked, "XXXX-XXXX-XXXX-3456");
    }

    #[test]
    fn test_risk_assessment_summary() {
        let assessor = RiskAssessor::new();
        let items = vec![
            SensitiveItem::new(
                SensitiveInfoType::Email,
                1, 1, 20,
                "test@example.com".to_string(),
                0.90,
                RiskLevel::Medium,
                "email_address".to_string(),
            ),
        ];

        let assessment = assessor.assess_risk(&items);
        let summary = assessment.get_summary();

        assert!(summary.contains("风险评分"));
        assert!(summary.contains("风险等级"));
        assert!(summary.contains("安全建议"));
    }
}