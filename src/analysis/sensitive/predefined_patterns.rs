use super::patterns::SensitivePattern;
use super::result::{SensitiveInfoType, RiskLevel};

/// 预置敏感信息模式集合
pub struct PredefinedPatterns;

impl PredefinedPatterns {
    /// 获取所有预置模式
    pub fn get_all_patterns() -> anyhow::Result<Vec<SensitivePattern>> {
        let mut patterns = Vec::new();

        // API 密钥模式
        patterns.extend(Self::get_api_key_patterns()?);

        // 密码模式
        patterns.extend(Self::get_password_patterns()?);

        // 令牌模式
        patterns.extend(Self::get_token_patterns()?);

        // 数据库连接字符串模式
        patterns.extend(Self::get_database_patterns()?);

        // 个人信息模式
        patterns.extend(Self::get_personal_info_patterns()?);

        // 证书和密钥模式
        patterns.extend(Self::get_certificate_patterns()?);

        Ok(patterns)
    }

    /// API 密钥检测模式
    fn get_api_key_patterns() -> anyhow::Result<Vec<SensitivePattern>> {
        let mut patterns = Vec::new();

        // AWS Access Key
        patterns.push(
            SensitivePattern::new(
                "aws_access_key".to_string(),
                SensitiveInfoType::ApiKey,
                r"AKIA[0-9A-Z]{16}".to_string(),
                0.95,
                RiskLevel::Critical,
                "AWS 访问密钥".to_string(),
            )?
            .with_recommendations(vec![
                "立即轮换暴露的 AWS 密钥".to_string(),
                "使用 AWS IAM 角色替代硬编码密钥".to_string(),
                "启用 AWS CloudTrail 监控密钥使用".to_string(),
                "考虑使用 AWS Secrets Manager 管理密钥".to_string(),
            ])
        );

        // AWS Secret Access Key
        patterns.push(
            SensitivePattern::new(
                "aws_secret_key".to_string(),
                SensitiveInfoType::ApiKey,
                r"[A-Za-z0-9/+=]{40}".to_string(),
                0.85,
                RiskLevel::Critical,
                "AWS 秘密访问密钥".to_string(),
            )?
            .with_recommendations(vec![
                "立即轮换暴露的 AWS 秘密密钥".to_string(),
                "使用环境变量或配置文件存储密钥".to_string(),
                "启用 MFA 增强账户安全性".to_string(),
            ])
        );

        // Google API Key
        patterns.push(
            SensitivePattern::new(
                "google_api_key".to_string(),
                SensitiveInfoType::ApiKey,
                r"AIza[0-9A-Za-z_-]{35}".to_string(),
                0.95,
                RiskLevel::High,
                "Google API 密钥".to_string(),
            )?
            .with_recommendations(vec![
                "在 Google Cloud Console 中轮换 API 密钥".to_string(),
                "限制 API 密钥的使用范围和权限".to_string(),
                "使用服务账户密钥替代 API 密钥".to_string(),
            ])
        );

        // GitHub Token
        patterns.push(
            SensitivePattern::new(
                "github_token".to_string(),
                SensitiveInfoType::ApiKey,
                r"gh[pousr]_[A-Za-z0-9_]{36,255}".to_string(),
                0.95,
                RiskLevel::High,
                "GitHub 访问令牌".to_string(),
            )?
            .with_recommendations(vec![
                "在 GitHub 设置中撤销暴露的令牌".to_string(),
                "生成新的个人访问令牌".to_string(),
                "使用最小权限原则配置令牌权限".to_string(),
                "考虑使用 GitHub Apps 替代个人令牌".to_string(),
            ])
        );

        // GitHub Classic Token
        patterns.push(
            SensitivePattern::new(
                "github_classic_token".to_string(),
                SensitiveInfoType::ApiKey,
                r"ghp_[A-Za-z0-9]{36}".to_string(),
                0.95,
                RiskLevel::High,
                "GitHub 经典个人访问令牌".to_string(),
            )?
            .with_recommendations(vec![
                "立即在 GitHub 设置中删除此令牌".to_string(),
                "使用细粒度个人访问令牌替代经典令牌".to_string(),
                "定期轮换访问令牌".to_string(),
            ])
        );

        // Slack Token
        patterns.push(
            SensitivePattern::new(
                "slack_token".to_string(),
                SensitiveInfoType::ApiKey,
                r"xox[baprs]-[0-9]{12}-[0-9]{12}-[0-9]{12}-[a-z0-9]{32}".to_string(),
                0.95,
                RiskLevel::High,
                "Slack API 令牌".to_string(),
            )?
            .with_recommendations(vec![
                "在 Slack App 管理页面中重新生成令牌".to_string(),
                "检查令牌权限范围是否合理".to_string(),
                "使用环境变量存储令牌".to_string(),
            ])
        );

        // Azure Subscription Key
        patterns.push(
            SensitivePattern::new(
                "azure_subscription_key".to_string(),
                SensitiveInfoType::ApiKey,
                r"[0-9a-f]{32}".to_string(),
                0.75,
                RiskLevel::High,
                "Azure 订阅密钥".to_string(),
            )?
            .with_recommendations(vec![
                "在 Azure 门户中重新生成订阅密钥".to_string(),
                "使用 Azure Key Vault 管理密钥".to_string(),
                "启用 Azure AD 身份验证".to_string(),
            ])
        );

        Ok(patterns)
    }

    /// 密码检测模式
    fn get_password_patterns() -> anyhow::Result<Vec<SensitivePattern>> {
        let mut patterns = Vec::new();

        // 通用密码模式
        patterns.push(
            SensitivePattern::new(
                "generic_password".to_string(),
                SensitiveInfoType::Password,
                r"(?i)(password|passwd|pwd)\s*[:=]\s*['\"]([^'\"]{6,})['\"]".to_string(),
                0.85,
                RiskLevel::High,
                "通用密码字段".to_string(),
            )?
            .with_recommendations(vec![
                "使用环境变量或配置文件存储密码".to_string(),
                "考虑使用密码管理工具".to_string(),
                "启用密码加密存储".to_string(),
                "定期更换密码".to_string(),
            ])
            .with_case_sensitivity(false)
        );

        // 数据库密码
        patterns.push(
            SensitivePattern::new(
                "database_password".to_string(),
                SensitiveInfoType::Password,
                r"(?i)(db_password|database_password|mysql_password|postgres_password)\s*[:=]\s*['\"]([^'\"]{4,})['\"]".to_string(),
                0.90,
                RiskLevel::Critical,
                "数据库密码".to_string(),
            )?
            .with_recommendations(vec![
                "立即更换数据库密码".to_string(),
                "使用数据库连接池管理连接".to_string(),
                "启用数据库访问日志".to_string(),
                "考虑使用数据库身份验证服务".to_string(),
            ])
            .with_case_sensitivity(false)
        );

        // SSH 密码
        patterns.push(
            SensitivePattern::new(
                "ssh_password".to_string(),
                SensitiveInfoType::Password,
                r"(?i)(ssh_password|sshpass)\s*[:=]\s*['\"]([^'\"]{4,})['\"]".to_string(),
                0.90,
                RiskLevel::High,
                "SSH 密码".to_string(),
            )?
            .with_recommendations(vec![
                "使用 SSH 密钥替代密码认证".to_string(),
                "禁用 SSH 密码登录".to_string(),
                "启用 SSH 密钥对认证".to_string(),
                "配置 SSH 访问控制".to_string(),
            ])
            .with_case_sensitivity(false)
        );

        Ok(patterns)
    }

    /// 令牌检测模式
    fn get_token_patterns() -> anyhow::Result<Vec<SensitivePattern>> {
        let mut patterns = Vec::new();

        // JWT Token
        patterns.push(
            SensitivePattern::new(
                "jwt_token".to_string(),
                SensitiveInfoType::Token,
                r"eyJ[A-Za-z0-9_-]*\.eyJ[A-Za-z0-9_-]*\.[A-Za-z0-9_-]*".to_string(),
                0.95,
                RiskLevel::High,
                "JWT 令牌".to_string(),
            )?
            .with_recommendations(vec![
                "检查 JWT 令牌是否包含敏感信息".to_string(),
                "设置合理的令牌过期时间".to_string(),
                "使用 HTTPS 传输令牌".to_string(),
                "考虑使用刷新令牌机制".to_string(),
            ])
        );

        // Bearer Token
        patterns.push(
            SensitivePattern::new(
                "bearer_token".to_string(),
                SensitiveInfoType::Token,
                r"(?i)bearer\s+[A-Za-z0-9_-]{20,}".to_string(),
                0.85,
                RiskLevel::High,
                "Bearer 令牌".to_string(),
            )?
            .with_recommendations(vec![
                "确保令牌通过安全渠道传输".to_string(),
                "实施令牌过期和刷新机制".to_string(),
                "监控令牌使用情况".to_string(),
            ])
            .with_case_sensitivity(false)
        );

        // API Token
        patterns.push(
            SensitivePattern::new(
                "api_token".to_string(),
                SensitiveInfoType::Token,
                r"(?i)(api_token|access_token|auth_token)\s*[:=]\s*['\"]([A-Za-z0-9_-]{20,})['\"]".to_string(),
                0.85,
                RiskLevel::High,
                "API 访问令牌".to_string(),
            )?
            .with_recommendations(vec![
                "使用环境变量存储令牌".to_string(),
                "定期轮换访问令牌".to_string(),
                "限制令牌权限范围".to_string(),
                "监控令牌使用情况".to_string(),
            ])
            .with_case_sensitivity(false)
        );

        Ok(patterns)
    }

    /// 数据库连接字符串模式
    fn get_database_patterns() -> anyhow::Result<Vec<SensitivePattern>> {
        let mut patterns = Vec::new();

        // MySQL 连接字符串
        patterns.push(
            SensitivePattern::new(
                "mysql_connection".to_string(),
                SensitiveInfoType::DatabaseConnection,
                r"mysql://[^:]+:[^@]+@[^/]+/\w+".to_string(),
                0.95,
                RiskLevel::Critical,
                "MySQL 数据库连接字符串".to_string(),
            )?
            .with_recommendations(vec![
                "使用环境变量存储数据库连接信息".to_string(),
                "启用数据库 SSL 连接".to_string(),
                "限制数据库用户权限".to_string(),
                "定期更换数据库密码".to_string(),
            ])
        );

        // PostgreSQL 连接字符串
        patterns.push(
            SensitivePattern::new(
                "postgresql_connection".to_string(),
                SensitiveInfoType::DatabaseConnection,
                r"postgres(ql)?://[^:]+:[^@]+@[^/]+/\w+".to_string(),
                0.95,
                RiskLevel::Critical,
                "PostgreSQL 数据库连接字符串".to_string(),
            )?
            .with_recommendations(vec![
                "使用环境变量存储连接字符串".to_string(),
                "启用 PostgreSQL SSL 模式".to_string(),
                "配置数据库防火墙规则".to_string(),
                "使用连接池管理数据库连接".to_string(),
            ])
        );

        // MongoDB 连接字符串
        patterns.push(
            SensitivePattern::new(
                "mongodb_connection".to_string(),
                SensitiveInfoType::DatabaseConnection,
                r"mongodb(\+srv)?://[^:]+:[^@]+@[^/]+/\w+".to_string(),
                0.95,
                RiskLevel::Critical,
                "MongoDB 数据库连接字符串".to_string(),
            )?
            .with_recommendations(vec![
                "使用 MongoDB Atlas 连接字符串".to_string(),
                "启用 MongoDB 身份验证".to_string(),
                "配置网络访问控制".to_string(),
                "使用 MongoDB 加密功能".to_string(),
            ])
        );

        // Redis 连接字符串
        patterns.push(
            SensitivePattern::new(
                "redis_connection".to_string(),
                SensitiveInfoType::DatabaseConnection,
                r"redis://[^:]*:[^@]*@[^/]+/?\d*".to_string(),
                0.90,
                RiskLevel::High,
                "Redis 数据库连接字符串".to_string(),
            )?
            .with_recommendations(vec![
                "启用 Redis 密码认证".to_string(),
                "配置 Redis 网络安全".to_string(),
                "使用 Redis TLS 加密".to_string(),
                "限制 Redis 命令权限".to_string(),
            ])
        );

        Ok(patterns)
    }

    /// 个人信息检测模式
    fn get_personal_info_patterns() -> anyhow::Result<Vec<SensitivePattern>> {
        let mut patterns = Vec::new();

        // 邮箱地址
        patterns.push(
            SensitivePattern::new(
                "email_address".to_string(),
                SensitiveInfoType::Email,
                r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b".to_string(),
                0.90,
                RiskLevel::Medium,
                "邮箱地址".to_string(),
            )?
            .with_recommendations(vec![
                "避免在代码中硬编码邮箱地址".to_string(),
                "使用配置文件存储邮箱信息".to_string(),
                "考虑邮箱地址的隐私保护".to_string(),
            ])
        );

        // 中国手机号
        patterns.push(
            SensitivePattern::new(
                "chinese_phone".to_string(),
                SensitiveInfoType::PhoneNumber,
                r"1[3-9]\d{9}".to_string(),
                0.85,
                RiskLevel::Medium,
                "中国手机号码".to_string(),
            )?
            .with_recommendations(vec![
                "避免在代码中存储真实手机号".to_string(),
                "使用测试手机号进行开发".to_string(),
                "实施手机号脱敏显示".to_string(),
            ])
        );

        // 美国手机号
        patterns.push(
            SensitivePattern::new(
                "us_phone".to_string(),
                SensitiveInfoType::PhoneNumber,
                r"\+?1?[-.\s]?\(?[0-9]{3}\)?[-.\s]?[0-9]{3}[-.\s]?[0-9]{4}".to_string(),
                0.80,
                RiskLevel::Medium,
                "美国手机号码".to_string(),
            )?
            .with_recommendations(vec![
                "使用格式化显示手机号".to_string(),
                "避免存储完整手机号".to_string(),
                "实施数据脱敏处理".to_string(),
            ])
        );

        // 中国身份证号
        patterns.push(
            SensitivePattern::new(
                "chinese_id_card".to_string(),
                SensitiveInfoType::SocialSecurityNumber,
                r"[1-9]\d{5}(18|19|20)\d{2}((0[1-9])|(1[0-2]))(([0-2][1-9])|10|20|30|31)\d{3}[0-9Xx]".to_string(),
                0.95,
                RiskLevel::Critical,
                "中国身份证号码".to_string(),
            )?
            .with_recommendations(vec![
                "立即删除代码中的身份证号".to_string(),
                "使用脱敏身份证号进行测试".to_string(),
                "实施严格的数据访问控制".to_string(),
                "遵守个人信息保护法规".to_string(),
            ])
        );

        // 美国社会安全号
        patterns.push(
            SensitivePattern::new(
                "us_ssn".to_string(),
                SensitiveInfoType::SocialSecurityNumber,
                r"\b\d{3}-?\d{2}-?\d{4}\b".to_string(),
                0.90,
                RiskLevel::Critical,
                "美国社会安全号码".to_string(),
            )?
            .with_recommendations(vec![
                "立即删除代码中的 SSN".to_string(),
                "使用虚假 SSN 进行测试".to_string(),
                "遵守 GDPR 和隐私法规".to_string(),
                "实施数据加密存储".to_string(),
            ])
        );

        // 信用卡号
        patterns.push(
            SensitivePattern::new(
                "credit_card".to_string(),
                SensitiveInfoType::CreditCard,
                r"\b(?:4[0-9]{12}(?:[0-9]{3})?|5[1-5][0-9]{14}|3[47][0-9]{13}|3[0-9]{13}|6(?:011|5[0-9]{2})[0-9]{12})\b".to_string(),
                0.95,
                RiskLevel::Critical,
                "信用卡号码".to_string(),
            )?
            .with_recommendations(vec![
                "立即删除代码中的信用卡号".to_string(),
                "使用测试信用卡号进行开发".to_string(),
                "实施 PCI DSS 合规措施".to_string(),
                "启用信用卡数据加密".to_string(),
            ])
        );

        Ok(patterns)
    }

    /// 证书和密钥检测模式
    fn get_certificate_patterns() -> anyhow::Result<Vec<SensitivePattern>> {
        let mut patterns = Vec::new();

        // RSA 私钥
        patterns.push(
            SensitivePattern::new(
                "rsa_private_key".to_string(),
                SensitiveInfoType::PrivateKey,
                r"-----BEGIN RSA PRIVATE KEY-----[\s\S]*?-----END RSA PRIVATE KEY-----".to_string(),
                0.99,
                RiskLevel::Critical,
                "RSA 私钥".to_string(),
            )?
            .with_recommendations(vec![
                "立即轮换暴露的私钥".to_string(),
                "使用密钥管理服务存储私钥".to_string(),
                "启用私钥密码保护".to_string(),
                "限制私钥文件访问权限".to_string(),
            ])
        );

        // EC 私钥
        patterns.push(
            SensitivePattern::new(
                "ec_private_key".to_string(),
                SensitiveInfoType::PrivateKey,
                r"-----BEGIN EC PRIVATE KEY-----[\s\S]*?-----END EC PRIVATE KEY-----".to_string(),
                0.99,
                RiskLevel::Critical,
                "椭圆曲线私钥".to_string(),
            )?
            .with_recommendations(vec![
                "立即轮换暴露的 EC 私钥".to_string(),
                "使用硬件安全模块存储私钥".to_string(),
                "实施私钥访问审计".to_string(),
            ])
        );

        // OpenSSH 私钥
        patterns.push(
            SensitivePattern::new(
                "openssh_private_key".to_string(),
                SensitiveInfoType::PrivateKey,
                r"-----BEGIN OPENSSH PRIVATE KEY-----[\s\S]*?-----END OPENSSH PRIVATE KEY-----".to_string(),
                0.99,
                RiskLevel::Critical,
                "OpenSSH 私钥".to_string(),
            )?
            .with_recommendations(vec![
                "立即更换 SSH 密钥对".to_string(),
                "使用 SSH 代理管理密钥".to_string(),
                "启用 SSH 密钥密码保护".to_string(),
                "配置 SSH 访问控制".to_string(),
            ])
        );

        // X.509 证书
        patterns.push(
            SensitivePattern::new(
                "x509_certificate".to_string(),
                SensitiveInfoType::Certificate,
                r"-----BEGIN CERTIFICATE-----[\s\S]*?-----END CERTIFICATE-----".to_string(),
                0.85,
                RiskLevel::Medium,
                "X.509 证书".to_string(),
            )?
            .with_recommendations(vec![
                "检查证书是否包含敏感信息".to_string(),
                "确保证书未过期".to_string(),
                "使用证书管理服务".to_string(),
                "定期更新证书".to_string(),
            ])
        );

        // PGP 私钥
        patterns.push(
            SensitivePattern::new(
                "pgp_private_key".to_string(),
                SensitiveInfoType::PrivateKey,
                r"-----BEGIN PGP PRIVATE KEY BLOCK-----[\s\S]*?-----END PGP PRIVATE KEY BLOCK-----".to_string(),
                0.99,
                RiskLevel::Critical,
                "PGP 私钥".to_string(),
            )?
            .with_recommendations(vec![
                "立即撤销暴露的 PGP 私钥".to_string(),
                "生成新的 PGP 密钥对".to_string(),
                "使用强密码保护私钥".to_string(),
                "更新公钥服务器信息".to_string(),
            ])
        );

        Ok(patterns)
    }

    /// 获取特定类型的模式
    pub fn get_patterns_by_type(info_type: &SensitiveInfoType) -> anyhow::Result<Vec<SensitivePattern>> {
        let all_patterns = Self::get_all_patterns()?;
        Ok(all_patterns.into_iter()
            .filter(|pattern| &pattern.info_type == info_type)
            .collect())
    }

    /// 获取特定风险等级的模式
    pub fn get_patterns_by_risk_level(risk_level: &RiskLevel) -> anyhow::Result<Vec<SensitivePattern>> {
        let all_patterns = Self::get_all_patterns()?;
        Ok(all_patterns.into_iter()
            .filter(|pattern| &pattern.risk_level == risk_level)
            .collect())
    }

    /// 获取模式统计信息
    pub fn get_pattern_statistics() -> anyhow::Result<PatternStatistics> {
        let patterns = Self::get_all_patterns()?;
        let mut stats = PatternStatistics::new();

        for pattern in &patterns {
            stats.total_patterns += 1;

            match pattern.risk_level {
                RiskLevel::Critical => stats.critical_patterns += 1,
                RiskLevel::High => stats.high_patterns += 1,
                RiskLevel::Medium => stats.medium_patterns += 1,
                RiskLevel::Low => stats.low_patterns += 1,
            }

            *stats.type_counts.entry(pattern.info_type.clone()).or_insert(0) += 1;
        }

        Ok(stats)
    }
}

/// 模式统计信息
#[derive(Debug, Clone)]
pub struct PatternStatistics {
    pub total_patterns: usize,
    pub critical_patterns: usize,
    pub high_patterns: usize,
    pub medium_patterns: usize,
    pub low_patterns: usize,
    pub type_counts: std::collections::HashMap<SensitiveInfoType, usize>,
}

impl PatternStatistics {
    pub fn new() -> Self {
        Self {
            total_patterns: 0,
            critical_patterns: 0,
            high_patterns: 0,
            medium_patterns: 0,
            low_patterns: 0,
            type_counts: std::collections::HashMap::new(),
        }
    }
}

impl Default for PatternStatistics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_all_patterns() {
        let patterns = PredefinedPatterns::get_all_patterns().unwrap();
        assert!(!patterns.is_empty());

        // 验证每个模式都有名称和描述
        for pattern in &patterns {
            assert!(!pattern.name.is_empty());
            assert!(!pattern.description.is_empty());
            assert!(pattern.confidence > 0.0 && pattern.confidence <= 1.0);
        }
    }

    #[test]
    fn test_api_key_patterns() {
        let patterns = PredefinedPatterns::get_api_key_patterns().unwrap();
        assert!(!patterns.is_empty());

        // 测试 AWS Access Key 模式
        let aws_pattern = patterns.iter()
            .find(|p| p.name == "aws_access_key")
            .expect("AWS access key pattern should exist");

        assert_eq!(aws_pattern.info_type, SensitiveInfoType::ApiKey);
        assert_eq!(aws_pattern.risk_level, RiskLevel::Critical);
        assert!(aws_pattern.is_match("AKIA1234567890ABCDEF"));
        assert!(!aws_pattern.is_match("invalid_key"));
    }

    #[test]
    fn test_password_patterns() {
        let patterns = PredefinedPatterns::get_password_patterns().unwrap();
        assert!(!patterns.is_empty());

        // 测试通用密码模式
        let password_pattern = patterns.iter()
            .find(|p| p.name == "generic_password")
            .expect("Generic password pattern should exist");

        assert!(password_pattern.is_match("password = \"secret123\""));
        assert!(password_pattern.is_match("PASSWORD: 'mypassword'"));
        assert!(!password_pattern.is_match("password = \"\""));
    }

    #[test]
    fn test_token_patterns() {
        let patterns = PredefinedPatterns::get_token_patterns().unwrap();
        assert!(!patterns.is_empty());

        // 测试 JWT 模式
        let jwt_pattern = patterns.iter()
            .find(|p| p.name == "jwt_token")
            .expect("JWT token pattern should exist");

        let jwt_token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c";
        assert!(jwt_pattern.is_match(jwt_token));
    }

    #[test]
    fn test_database_patterns() {
        let patterns = PredefinedPatterns::get_database_patterns().unwrap();
        assert!(!patterns.is_empty());

        // 测试 MySQL 连接字符串
        let mysql_pattern = patterns.iter()
            .find(|p| p.name == "mysql_connection")
            .expect("MySQL connection pattern should exist");

        assert!(mysql_pattern.is_match("mysql://user:password@localhost/database"));
        assert!(!mysql_pattern.is_match("invalid_connection_string"));
    }

    #[test]
    fn test_personal_info_patterns() {
        let patterns = PredefinedPatterns::get_personal_info_patterns().unwrap();
        assert!(!patterns.is_empty());

        // 测试邮箱模式
        let email_pattern = patterns.iter()
            .find(|p| p.name == "email_address")
            .expect("Email pattern should exist");

        assert!(email_pattern.is_match("user@example.com"));
        assert!(email_pattern.is_match("test.email+tag@domain.co.uk"));
        assert!(!email_pattern.is_match("invalid_email"));
    }

    #[test]
    fn test_certificate_patterns() {
        let patterns = PredefinedPatterns::get_certificate_patterns().unwrap();
        assert!(!patterns.is_empty());

        // 测试 RSA 私钥模式
        let rsa_pattern = patterns.iter()
            .find(|p| p.name == "rsa_private_key")
            .expect("RSA private key pattern should exist");

        let rsa_key = "-----BEGIN RSA PRIVATE KEY-----\nMIIEpAIBAAKCAQEA...\n-----END RSA PRIVATE KEY-----";
        assert!(rsa_pattern.is_match(rsa_key));
    }

    #[test]
    fn test_patterns_by_type() {
        let api_patterns = PredefinedPatterns::get_patterns_by_type(&SensitiveInfoType::ApiKey).unwrap();
        assert!(!api_patterns.is_empty());

        for pattern in &api_patterns {
            assert_eq!(pattern.info_type, SensitiveInfoType::ApiKey);
        }
    }

    #[test]
    fn test_patterns_by_risk_level() {
        let critical_patterns = PredefinedPatterns::get_patterns_by_risk_level(&RiskLevel::Critical).unwrap();
        assert!(!critical_patterns.is_empty());

        for pattern in &critical_patterns {
            assert_eq!(pattern.risk_level, RiskLevel::Critical);
        }
    }

    #[test]
    fn test_pattern_statistics() {
        let stats = PredefinedPatterns::get_pattern_statistics().unwrap();
        assert!(stats.total_patterns > 0);
        assert!(stats.critical_patterns > 0);
        assert!(stats.high_patterns > 0);
        assert!(!stats.type_counts.is_empty());

        // 验证统计数据一致性
        let sum = stats.critical_patterns + stats.high_patterns + stats.medium_patterns + stats.low_patterns;
        assert_eq!(sum, stats.total_patterns);
    }
}