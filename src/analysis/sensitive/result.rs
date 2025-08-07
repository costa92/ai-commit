use serde::{Deserialize, Serialize};

/// 敏感信息类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum SensitiveInfoType {
    ApiKey,
    Password,
    Token,
    DatabaseConnection,
    Email,
    PhoneNumber,
    CreditCard,
    SocialSecurityNumber,
    PrivateKey,
    Certificate,
    Custom(String),
}

impl std::fmt::Display for SensitiveInfoType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SensitiveInfoType::ApiKey => write!(f, "API Key"),
            SensitiveInfoType::Password => write!(f, "Password"),
            SensitiveInfoType::Token => write!(f, "Token"),
            SensitiveInfoType::DatabaseConnection => write!(f, "Database Connection"),
            SensitiveInfoType::Email => write!(f, "Email"),
            SensitiveInfoType::PhoneNumber => write!(f, "Phone Number"),
            SensitiveInfoType::CreditCard => write!(f, "Credit Card"),
            SensitiveInfoType::SocialSecurityNumber => write!(f, "Social Security Number"),
            SensitiveInfoType::PrivateKey => write!(f, "Private Key"),
            SensitiveInfoType::Certificate => write!(f, "Certificate"),
            SensitiveInfoType::Custom(name) => write!(f, "{}", name),
        }
    }
}

/// 风险等级
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RiskLevel {
    Critical,
    High,
    Medium,
    Low,
}

impl std::fmt::Display for RiskLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RiskLevel::Critical => write!(f, "Critical"),
            RiskLevel::High => write!(f, "High"),
            RiskLevel::Medium => write!(f, "Medium"),
            RiskLevel::Low => write!(f, "Low"),
        }
    }
}

/// 敏感信息项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensitiveItem {
    pub info_type: SensitiveInfoType,
    pub line_number: usize,
    pub column_start: usize,
    pub column_end: usize,
    pub matched_text: String,
    pub masked_text: String,
    pub confidence: f32,
    pub risk_level: RiskLevel,
    pub recommendations: Vec<String>,
    pub pattern_name: String,
}

impl SensitiveItem {
    pub fn new(
        info_type: SensitiveInfoType,
        line_number: usize,
        column_start: usize,
        column_end: usize,
        matched_text: String,
        confidence: f32,
        risk_level: RiskLevel,
        pattern_name: String,
    ) -> Self {
        let masked_text = Self::mask_text(&matched_text, &info_type);

        Self {
            info_type,
            line_number,
            column_start,
            column_end,
            matched_text,
            masked_text,
            confidence,
            risk_level,
            recommendations: Vec::new(),
            pattern_name,
        }
    }

    pub fn with_recommendations(mut self, recommendations: Vec<String>) -> Self {
        self.recommendations = recommendations;
        self
    }

    fn mask_text(text: &str, info_type: &SensitiveInfoType) -> String {
        match info_type {
            SensitiveInfoType::ApiKey | SensitiveInfoType::Token | SensitiveInfoType::PrivateKey => {
                if text.len() <= 8 {
                    "*".repeat(text.len())
                } else {
                    format!("{}***{}", &text[..4], &text[text.len()-4..])
                }
            },
            SensitiveInfoType::Password => {
                "*".repeat(text.len().min(12))
            },
            SensitiveInfoType::Email => {
                if let Some(at_pos) = text.find('@') {
                    let (local, domain) = text.split_at(at_pos);
                    if local.len() <= 2 {
                        format!("***{}", domain)
                    } else {
                        format!("{}***{}", &local[..2], domain)
                    }
                } else {
                    "*".repeat(text.len())
                }
            },
            SensitiveInfoType::PhoneNumber => {
                if text.len() >= 4 {
                    format!("***-***-{}", &text[text.len()-4..])
                } else {
                    "*".repeat(text.len())
                }
            },
            SensitiveInfoType::CreditCard => {
                if text.len() >= 4 {
                    format!("****-****-****-{}", &text[text.len()-4..])
                } else {
                    "*".repeat(text.len())
                }
            },
            _ => {
                if text.len() <= 4 {
                    "*".repeat(text.len())
                } else {
                    format!("{}***", &text[..2])
                }
            }
        }
    }
}

/// 敏感信息摘要
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensitiveSummary {
    pub total_items: usize,
    pub critical_items: usize,
    pub high_items: usize,
    pub medium_items: usize,
    pub low_items: usize,
    pub types_detected: Vec<SensitiveInfoType>,
    pub risk_score: f32,
}

impl SensitiveSummary {
    pub fn new() -> Self {
        Self {
            total_items: 0,
            critical_items: 0,
            high_items: 0,
            medium_items: 0,
            low_items: 0,
            types_detected: Vec::new(),
            risk_score: 0.0,
        }
    }

    pub fn from_items(items: &[SensitiveItem]) -> Self {
        let mut summary = Self::new();
        summary.total_items = items.len();

        let mut type_set = std::collections::HashSet::new();
        let mut total_risk = 0.0;

        for item in items {
            match item.risk_level {
                RiskLevel::Critical => summary.critical_items += 1,
                RiskLevel::High => summary.high_items += 1,
                RiskLevel::Medium => summary.medium_items += 1,
                RiskLevel::Low => summary.low_items += 1,
            }

            type_set.insert(item.info_type.clone());

            // 计算风险分数
            let risk_weight = match item.risk_level {
                RiskLevel::Critical => 4.0,
                RiskLevel::High => 3.0,
                RiskLevel::Medium => 2.0,
                RiskLevel::Low => 1.0,
            };
            total_risk += risk_weight * item.confidence;
        }

        summary.types_detected = type_set.into_iter().collect();
        summary.risk_score = if items.is_empty() { 0.0 } else { total_risk / items.len() as f32 };

        summary
    }
}

/// 敏感信息检测结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensitiveInfoResult {
    pub file_path: String,
    pub items: Vec<SensitiveItem>,
    pub summary: SensitiveSummary,
    pub scan_duration: std::time::Duration,
    pub patterns_used: Vec<String>,
}

impl SensitiveInfoResult {
    pub fn new(file_path: String) -> Self {
        Self {
            file_path,
            items: Vec::new(),
            summary: SensitiveSummary::new(),
            scan_duration: std::time::Duration::from_secs(0),
            patterns_used: Vec::new(),
        }
    }

    pub fn with_items(mut self, items: Vec<SensitiveItem>) -> Self {
        self.summary = SensitiveSummary::from_items(&items);
        self.items = items;
        self
    }

    pub fn with_duration(mut self, duration: std::time::Duration) -> Self {
        self.scan_duration = duration;
        self
    }

    pub fn with_patterns(mut self, patterns: Vec<String>) -> Self {
        self.patterns_used = patterns;
        self
    }

    pub fn has_critical_issues(&self) -> bool {
        self.summary.critical_items > 0
    }

    pub fn has_high_risk_issues(&self) -> bool {
        self.summary.critical_items > 0 || self.summary.high_items > 0
    }
}