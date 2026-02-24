use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 敏感信息类型
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SensitiveInfoType {
    /// 密码相关
    Password,
    /// API密钥
    ApiKey,
    /// 证书和密钥
    Certificate,
    /// 数据库连接字符串
    DatabaseConnection,
    /// JWT Token
    JwtToken,
    /// 邮箱地址
    Email,
    /// 电话号码
    PhoneNumber,
    /// IP地址
    IpAddress,
    /// 信用卡号
    CreditCard,
    /// SSH密钥
    SshKey,
    /// 身份证号
    IdCard,
    /// 访问令牌
    AccessToken,
    /// 私钥
    PrivateKey,
    /// 服务账号密钥
    ServiceAccountKey,
    /// 其他敏感信息
    Other(String),
}

impl SensitiveInfoType {
    pub fn as_str(&self) -> &str {
        match self {
            SensitiveInfoType::Password => "密码",
            SensitiveInfoType::ApiKey => "API密钥",
            SensitiveInfoType::Certificate => "证书",
            SensitiveInfoType::DatabaseConnection => "数据库连接",
            SensitiveInfoType::JwtToken => "JWT令牌",
            SensitiveInfoType::Email => "邮箱地址",
            SensitiveInfoType::PhoneNumber => "电话号码",
            SensitiveInfoType::IpAddress => "IP地址",
            SensitiveInfoType::CreditCard => "信用卡号",
            SensitiveInfoType::SshKey => "SSH密钥",
            SensitiveInfoType::IdCard => "身份证号",
            SensitiveInfoType::AccessToken => "访问令牌",
            SensitiveInfoType::PrivateKey => "私钥",
            SensitiveInfoType::ServiceAccountKey => "服务账号密钥",
            SensitiveInfoType::Other(name) => name,
        }
    }

    pub fn risk_level(&self) -> SensitiveRiskLevel {
        match self {
            SensitiveInfoType::Password
            | SensitiveInfoType::ApiKey
            | SensitiveInfoType::PrivateKey
            | SensitiveInfoType::ServiceAccountKey
            | SensitiveInfoType::AccessToken => SensitiveRiskLevel::Critical,

            SensitiveInfoType::Certificate
            | SensitiveInfoType::DatabaseConnection
            | SensitiveInfoType::JwtToken
            | SensitiveInfoType::SshKey => SensitiveRiskLevel::High,

            SensitiveInfoType::Email
            | SensitiveInfoType::PhoneNumber
            | SensitiveInfoType::CreditCard
            | SensitiveInfoType::IdCard => SensitiveRiskLevel::Medium,

            SensitiveInfoType::IpAddress => SensitiveRiskLevel::Low,

            SensitiveInfoType::Other(_) => SensitiveRiskLevel::Medium,
        }
    }
}

/// 敏感信息风险等级
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SensitiveRiskLevel {
    Critical,
    High,
    Medium,
    Low,
}

impl SensitiveRiskLevel {
    pub fn as_str(&self) -> &str {
        match self {
            SensitiveRiskLevel::Critical => "严重",
            SensitiveRiskLevel::High => "高",
            SensitiveRiskLevel::Medium => "中等",
            SensitiveRiskLevel::Low => "低",
        }
    }

    pub fn emoji(&self) -> &str {
        match self {
            SensitiveRiskLevel::Critical => "🚨",
            SensitiveRiskLevel::High => "⚠️",
            SensitiveRiskLevel::Medium => "🟡",
            SensitiveRiskLevel::Low => "ℹ️",
        }
    }
}

/// 检测到的敏感信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensitiveInfoItem {
    pub info_type: SensitiveInfoType,
    pub line_number: usize,
    pub column_start: usize,
    pub column_end: usize,
    pub matched_text: String,
    pub masked_text: String,
    pub confidence: f32,
    pub description: String,
    pub recommendations: Vec<String>,
}

/// 敏感信息检测结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensitiveInfoResult {
    pub file_path: String,
    pub items: Vec<SensitiveInfoItem>,
    pub summary: SensitiveInfoSummary,
}

/// 敏感信息摘要
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensitiveInfoSummary {
    pub total_count: usize,
    pub critical_count: usize,
    pub high_count: usize,
    pub medium_count: usize,
    pub low_count: usize,
    pub types_detected: HashMap<SensitiveInfoType, usize>,
    pub overall_risk: SensitiveRiskLevel,
}

/// 敏感信息检测器
pub struct SensitiveInfoDetector {
    patterns: Vec<SensitivePattern>,
}

/// 敏感信息检测模式
struct SensitivePattern {
    info_type: SensitiveInfoType,
    regex: Regex,
    confidence: f32,
    description: String,
    recommendations: Vec<String>,
}

impl SensitiveInfoDetector {
    pub fn new() -> Self {
        let patterns = Self::initialize_patterns();
        Self { patterns }
    }

    /// 初始化所有检测模式
    fn initialize_patterns() -> Vec<SensitivePattern> {
        vec![
            // API密钥模式
            SensitivePattern {
                info_type: SensitiveInfoType::ApiKey,
                regex: Regex::new(
                    r#"(?i)(api[_-]?key|apikey|access[_-]?key)\s*[:=]\s*['"]([a-zA-Z0-9-]{20,})['"]"#,
                )
                .unwrap(),
                confidence: 0.9,
                description: "检测到API密钥".to_string(),
                recommendations: vec![
                    "使用环境变量存储API密钥".to_string(),
                    "不要在代码中硬编码API密钥".to_string(),
                    "考虑使用密钥管理服务".to_string(),
                ],
            },
            // 密码模式
            SensitivePattern {
                info_type: SensitiveInfoType::Password,
                regex: Regex::new(r#"(?i)(password|passwd|pwd)\s*[:=]\s*['"]([^'"]{6,})['"]"#)
                    .unwrap(),
                confidence: 0.8,
                description: "检测到明文密码".to_string(),
                recommendations: vec![
                    "使用密码哈希存储".to_string(),
                    "不要在代码中硬编码密码".to_string(),
                    "使用环境变量或密钥管理".to_string(),
                ],
            },
            // JWT Token模式
            SensitivePattern {
                info_type: SensitiveInfoType::JwtToken,
                regex: Regex::new(r"eyJ[a-zA-Z0-9_-]+\.eyJ[a-zA-Z0-9_-]+\.[a-zA-Z0-9_-]+").unwrap(),
                confidence: 0.95,
                description: "检测到JWT令牌".to_string(),
                recommendations: vec![
                    "不要在代码中硬编码JWT令牌".to_string(),
                    "使用安全的存储方式".to_string(),
                    "设置合理的过期时间".to_string(),
                ],
            },
            // 数据库连接字符串
            SensitivePattern {
                info_type: SensitiveInfoType::DatabaseConnection,
                regex: Regex::new(r"(?i)(mongodb://|mysql://|postgresql://|redis://|sqlite://)[^\s]+")
                    .unwrap(),
                confidence: 0.85,
                description: "检测到数据库连接字符串".to_string(),
                recommendations: vec![
                    "使用环境变量存储数据库连接信息".to_string(),
                    "避免在代码中暴露数据库凭证".to_string(),
                    "使用连接池和安全配置".to_string(),
                ],
            },
            // 私钥模式
            SensitivePattern {
                info_type: SensitiveInfoType::PrivateKey,
                regex: Regex::new(r"-----BEGIN\s+(RSA\s+)?PRIVATE\s+KEY-----").unwrap(),
                confidence: 0.99,
                description: "检测到私钥".to_string(),
                recommendations: vec![
                    "不要在代码中包含私钥".to_string(),
                    "使用密钥管理服务".to_string(),
                    "立即轮换暴露的密钥".to_string(),
                ],
            },
            // SSH密钥
            SensitivePattern {
                info_type: SensitiveInfoType::SshKey,
                regex: Regex::new(r"ssh-rsa\s+[A-Za-z0-9+/=]+").unwrap(),
                confidence: 0.9,
                description: "检测到SSH公钥".to_string(),
                recommendations: vec![
                    "确认SSH密钥是否应该在代码中".to_string(),
                    "考虑使用专门的密钥管理".to_string(),
                ],
            },
            // 邮箱地址
            SensitivePattern {
                info_type: SensitiveInfoType::Email,
                regex: Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b").unwrap(),
                confidence: 0.7,
                description: "检测到邮箱地址".to_string(),
                recommendations: vec![
                    "确认邮箱地址是否需要保护".to_string(),
                    "考虑使用配置文件".to_string(),
                ],
            },
            // IP地址
            SensitivePattern {
                info_type: SensitiveInfoType::IpAddress,
                regex: Regex::new(r"\b(?:[0-9]{1,3}\.){3}[0-9]{1,3}\b").unwrap(),
                confidence: 0.6,
                description: "检测到IP地址".to_string(),
                recommendations: vec![
                    "检查IP地址是否为内网地址".to_string(),
                    "避免暴露生产环境IP".to_string(),
                ],
            },
            // 信用卡号
            SensitivePattern {
                info_type: SensitiveInfoType::CreditCard,
                regex: Regex::new(r"\b(?:4[0-9]{12}(?:[0-9]{3})?|5[1-5][0-9]{14}|3[47][0-9]{13}|3[0-9]{13}|6(?:011|5[0-9]{2})[0-9]{12})\b").unwrap(),
                confidence: 0.8,
                description: "检测到可能的信用卡号".to_string(),
                recommendations: vec![
                    "不要在代码中存储信用卡信息".to_string(),
                    "使用PCI DSS合规的处理方式".to_string(),
                    "立即删除暴露的信用卡信息".to_string(),
                ],
            },
            // 中国身份证号
            SensitivePattern {
                info_type: SensitiveInfoType::IdCard,
                regex: Regex::new(r"\b[1-9]\d{5}(18|19|20)\d{2}((0[1-9])|(1[0-2]))(([0-2][1-9])|10|20|30|31)\d{3}[0-9Xx]\b").unwrap(),
                confidence: 0.85,
                description: "检测到身份证号".to_string(),
                recommendations: vec![
                    "不要在代码中存储身份证信息".to_string(),
                    "使用脱敏处理".to_string(),
                    "遵守数据保护法规".to_string(),
                ],
            },
            // 电话号码
            SensitivePattern {
                info_type: SensitiveInfoType::PhoneNumber,
                regex: Regex::new(r"\b1[3-9]\d{9}\b").unwrap(),
                confidence: 0.7,
                description: "检测到手机号码".to_string(),
                recommendations: vec![
                    "检查手机号是否需要保护".to_string(),
                    "考虑使用脱敏处理".to_string(),
                ],
            },
            // 访问令牌
            SensitivePattern {
                info_type: SensitiveInfoType::AccessToken,
                regex: Regex::new(
                    r#"(?i)(access[_-]?token|bearer[_-]?token)\s*[:=]\s*['"]([a-zA-Z0-9-]{20,})['"]"#,
                )
                .unwrap(),
                confidence: 0.85,
                description: "检测到访问令牌".to_string(),
                recommendations: vec![
                    "不要在代码中硬编码访问令牌".to_string(),
                    "使用安全的令牌存储方式".to_string(),
                    "设置合理的令牌过期时间".to_string(),
                ],
            },
        ]
    }

    /// 检测代码中的敏感信息
    pub fn detect(&self, file_path: &str, content: &str) -> SensitiveInfoResult {
        let mut items = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        for (line_num, line) in lines.iter().enumerate() {
            for pattern in &self.patterns {
                for mat in pattern.regex.find_iter(line) {
                    let matched_text = mat.as_str().to_string();
                    let masked_text = self.mask_sensitive_text(&matched_text, &pattern.info_type);

                    items.push(SensitiveInfoItem {
                        info_type: pattern.info_type.clone(),
                        line_number: line_num + 1,
                        column_start: mat.start(),
                        column_end: mat.end(),
                        matched_text,
                        masked_text,
                        confidence: pattern.confidence,
                        description: pattern.description.clone(),
                        recommendations: pattern.recommendations.clone(),
                    });
                }
            }
        }

        let summary = self.generate_summary(&items);

        SensitiveInfoResult {
            file_path: file_path.to_string(),
            items,
            summary,
        }
    }

    /// 检测多个文件的敏感信息
    pub fn detect_files(&self, files: &[(String, String)]) -> Vec<SensitiveInfoResult> {
        files
            .iter()
            .map(|(path, content)| self.detect(path, content))
            .collect()
    }

    /// 掩码敏感文本
    fn mask_sensitive_text(&self, text: &str, info_type: &SensitiveInfoType) -> String {
        match info_type {
            SensitiveInfoType::Password
            | SensitiveInfoType::ApiKey
            | SensitiveInfoType::AccessToken => {
                if text.len() <= 4 {
                    "*".repeat(text.len())
                } else {
                    format!("{}***{}", &text[..2], &text[text.len() - 2..])
                }
            }
            SensitiveInfoType::Email => {
                if let Some(at_pos) = text.find('@') {
                    let (local, domain) = text.split_at(at_pos);
                    if local.len() <= 2 {
                        format!("***{}", domain)
                    } else {
                        format!("{}***{}", &local[..2], domain)
                    }
                } else {
                    "***@***.***".to_string()
                }
            }
            SensitiveInfoType::PhoneNumber => {
                if text.len() >= 7 {
                    format!("{}****{}", &text[..3], &text[text.len() - 4..])
                } else {
                    "*".repeat(text.len())
                }
            }
            SensitiveInfoType::CreditCard => {
                if text.len() >= 8 {
                    format!("****-****-****-{}", &text[text.len() - 4..])
                } else {
                    "*".repeat(text.len())
                }
            }
            SensitiveInfoType::IdCard => {
                if text.len() >= 8 {
                    format!("{}************{}", &text[..2], &text[text.len() - 2..])
                } else {
                    "*".repeat(text.len())
                }
            }
            _ => {
                if text.len() <= 6 {
                    "*".repeat(text.len())
                } else {
                    format!("{}***{}", &text[..3], &text[text.len() - 3..])
                }
            }
        }
    }

    /// 生成敏感信息摘要
    fn generate_summary(&self, items: &[SensitiveInfoItem]) -> SensitiveInfoSummary {
        let mut types_detected = HashMap::new();
        let mut critical_count = 0;
        let mut high_count = 0;
        let mut medium_count = 0;
        let mut low_count = 0;

        for item in items {
            *types_detected.entry(item.info_type.clone()).or_insert(0) += 1;

            match item.info_type.risk_level() {
                SensitiveRiskLevel::Critical => critical_count += 1,
                SensitiveRiskLevel::High => high_count += 1,
                SensitiveRiskLevel::Medium => medium_count += 1,
                SensitiveRiskLevel::Low => low_count += 1,
            }
        }

        let overall_risk = if critical_count > 0 {
            SensitiveRiskLevel::Critical
        } else if high_count > 0 {
            SensitiveRiskLevel::High
        } else if medium_count > 0 {
            SensitiveRiskLevel::Medium
        } else {
            SensitiveRiskLevel::Low
        };

        SensitiveInfoSummary {
            total_count: items.len(),
            critical_count,
            high_count,
            medium_count,
            low_count,
            types_detected,
            overall_risk,
        }
    }
}

impl Default for SensitiveInfoDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_key_detection() {
        let detector = SensitiveInfoDetector::new();
        let content = r#"
        const config = {
            api_key: "sk-1234567890abcdef1234567890abcdef"
        };
        "#;

        let result = detector.detect("config.js", content);
        assert_eq!(result.items.len(), 1);
        assert_eq!(result.items[0].info_type, SensitiveInfoType::ApiKey);
        assert_eq!(result.summary.critical_count, 1);
    }

    #[test]
    fn test_password_detection() {
        let detector = SensitiveInfoDetector::new();
        let content = r#"
        password = "secretpassword123"
        "#;

        let result = detector.detect("config.py", content);
        assert_eq!(result.items.len(), 1);
        assert_eq!(result.items[0].info_type, SensitiveInfoType::Password);
    }

    #[test]
    fn test_jwt_token_detection() {
        let detector = SensitiveInfoDetector::new();
        let content = "token = eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c";

        let result = detector.detect("app.js", content);
        assert_eq!(result.items.len(), 1);
        assert_eq!(result.items[0].info_type, SensitiveInfoType::JwtToken);
    }

    #[test]
    fn test_email_detection() {
        let detector = SensitiveInfoDetector::new();
        let content = "email = user@example.com";

        let result = detector.detect("user.js", content);
        assert_eq!(result.items.len(), 1);
        assert_eq!(result.items[0].info_type, SensitiveInfoType::Email);
    }

    #[test]
    fn test_phone_number_detection() {
        let detector = SensitiveInfoDetector::new();
        let content = "phone = 13812345678";

        let result = detector.detect("contact.js", content);
        assert_eq!(result.items.len(), 1);
        assert_eq!(result.items[0].info_type, SensitiveInfoType::PhoneNumber);
    }

    #[test]
    fn test_masking() {
        let detector = SensitiveInfoDetector::new();

        // 测试API密钥掩码
        let masked =
            detector.mask_sensitive_text("sk-1234567890abcdef", &SensitiveInfoType::ApiKey);
        assert_eq!(masked, "sk***ef");

        // 测试邮箱掩码
        let masked = detector.mask_sensitive_text("user@example.com", &SensitiveInfoType::Email);
        assert_eq!(masked, "us***@example.com");

        // 测试手机号掩码
        let masked = detector.mask_sensitive_text("13812345678", &SensitiveInfoType::PhoneNumber);
        assert_eq!(masked, "138****5678");
    }

    #[test]
    fn test_risk_levels() {
        assert_eq!(
            SensitiveInfoType::ApiKey.risk_level(),
            SensitiveRiskLevel::Critical
        );
        assert_eq!(
            SensitiveInfoType::Email.risk_level(),
            SensitiveRiskLevel::Medium
        );
        assert_eq!(
            SensitiveInfoType::IpAddress.risk_level(),
            SensitiveRiskLevel::Low
        );
    }

    #[test]
    fn test_no_false_positives() {
        let detector = SensitiveInfoDetector::new();
        let content = r#"
        // This is just a comment with password mentioned
        function calculateSum(a, b) {
            return a + b;
        }
        "#;

        let result = detector.detect("safe.js", content);
        assert_eq!(result.items.len(), 0);
    }

    #[test]
    fn test_multiple_detections() {
        let detector = SensitiveInfoDetector::new();
        let content = r#"
        const config = {
            api_key: "sk-1234567890abcdef1234567890abcdef",
            password: "secretpassword123",
            email: "admin@company.com"
        };
        "#;

        let result = detector.detect("config.js", content);
        assert_eq!(result.items.len(), 3);

        // 检查摘要统计
        assert_eq!(result.summary.total_count, 3);
        assert_eq!(result.summary.critical_count, 2); // api_key + password
        assert_eq!(result.summary.medium_count, 1); // email
    }
}
