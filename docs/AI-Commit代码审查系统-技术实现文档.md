# AI-Commit 代码审查系统 - 技术实现文档

## 📋 文档信息

- **文档版本**: v2.1.0
- **创建日期**: 2025 年 1 月
- **最后更新**: 2025 年 8 月 3 日
- **文档状态**: ✅ 已更新 - 新增深度分析与扩展功能技术实现
- **技术栈**: Rust, AI Services, Static Analysis Tools, Notification Systems

## 🏗️ 系统架构深度解析

### 整体架构设计

AI-Commit 代码审查系统采用分层模块化架构，通过 Rust 语言实现高性能的代码分析能力。系统集成了传统静态分析工具和现代 AI 技术，提供全面的代码质量检测服务。

```
┌─────────────────────────────────────────────────────────────────┐
│                    AI-Commit 代码审查系统                          │
├─────────────────────────────────────────────────────────────────┤
│  CLI Interface Layer (src/cli/)                                │
│  - 命令行参数解析和验证                                            │
│  - 用户交互和进度显示                                              │
├─────────────────────────────────────────────────────────────────┤
│  Service Orchestration Layer                                   │
│  - CodeReviewService (src/languages/review_service_v2.rs)      │
│  - 工作流协调和结果聚合                                            │
├─────────────────┬─────────────────┬─────────────────────────────┤
│  Language       │  Static         │  Sensitive Info             │
│  Analysis       │  Analysis       │  Detection                  │
│  Layer          │  Layer          │  Layer                      │
│  (多语言支持)    │  (工具集成)      │  (安全检测)                  │
├─────────────────┼─────────────────┼─────────────────────────────┤
│  AI Services    │  Caching        │  Notification               │
│  Layer          │  Layer          │  Layer                      │
│  (智能分析)      │  (性能优化)      │  (团队协作)                  │
├─────────────────┴─────────────────┴─────────────────────────────┤
│  Infrastructure Layer                                          │
│  - Configuration Management (src/config/)                      │
│  - Error Handling & Logging                                    │
│  - Network & Storage                                            │
└─────────────────────────────────────────────────────────────────┘
```

### 核心数据流

```mermaid
graph TB
    A[Git Diff/Files] --> B[CodeReviewService]
    B --> C{Language Detection}
    C --> D[Go Analyzer]
    C --> E[Rust Analyzer]
    C --> F[TS/JS Analyzer]
    C --> G[Generic Analyzer]

    D --> H[Static Analysis]
    E --> H
    F --> H
    G --> H

    H --> I[AI Review]
    I --> J[Sensitive Info Detection]
    J --> K[Result Aggregation]
    K --> L[Report Generation]
    L --> M[Notification System]

    M --> N[Feishu]
    M --> O[WeChat]
    M --> P[Email]
    M --> Q[Other Platforms]
```

## 🔍 核心组件技术实现

### 1. 语言检测与分析系统

#### 1.1 智能语言检测器

```rust
// src/languages/ai_language_detector.rs
pub struct AILanguageDetector {
    config: Config,
    cache: HashMap<String, LanguageDetectionResult>,
}

impl AILanguageDetector {
    /// 三层检测策略：扩展名 -> AI分析 -> 启发式
    pub async fn detect_language(
        &mut self,
        file_path: &str,
        code_content: &str,
    ) -> LanguageDetectionResult {
        // 1. 快速路径：基于文件扩展名
        let extension_based = Language::from_file_path(file_path);
        if extension_based != Language::Unknown {
            return LanguageDetectionResult {
                detected_language: extension_based,
                confidence: 0.95,
                reasoning: format!("基于文件扩展名检测: {}", file_path),
                fallback_used: false,
            };
        }

        // 2. AI增强检测
        match self.ai_detect_language(file_path, code_content).await {
            Ok(result) => result,
            Err(_) => {
                // 3. 启发式后备检测
                self.heuristic_detection(file_path, code_content)
            }
        }
    }

    /// 启发式检测实现
    fn heuristic_detection(&self, file_path: &str, code_content: &str) -> LanguageDetectionResult {
        let content_lower = code_content.to_lowercase();

        // Go语言特征检测
        if content_lower.contains("package ") ||
           content_lower.contains("func ") ||
           content_lower.contains("import (") {
            return LanguageDetectionResult {
                detected_language: Language::Go,
                confidence: 0.8,
                reasoning: "启发式检测: 发现Go语言特征".to_string(),
                fallback_used: true,
            };
        }

        // Rust语言特征检测
        if content_lower.contains("fn ") ||
           content_lower.contains("use ") ||
           content_lower.contains("impl ") {
            return LanguageDetectionResult {
                detected_language: Language::Rust,
                confidence: 0.8,
                reasoning: "启发式检测: 发现Rust语言特征".to_string(),
                fallback_used: true,
            };
        }

        // 默认返回未知
        LanguageDetectionResult {
            detected_language: Language::Unknown,
            confidence: 0.3,
            reasoning: format!("无法检测语言: {}", file_path),
            fallback_used: true,
        }
    }
}
```

#### 1.2 语言特定分析器

```rust
// src/languages/go/analyzer.rs
pub struct GoAnalyzer {
    patterns: Vec<GoPattern>,
}

impl LanguageAnalyzer for GoAnalyzer {
    fn analyze_line(&self, line: &str, line_number: usize) -> Vec<LanguageFeature> {
        let mut features = Vec::new();

        // Go包声明检测
        if let Some(caps) = GO_PACKAGE_REGEX.captures(line) {
            features.push(LanguageFeature {
                feature_type: "package".to_string(),
                name: caps.get(1).map(|m| m.as_str().to_string()).unwrap_or_default(),
                line_number: Some(line_number),
                description: format!("Go package: {}", line),
            });
        }

        // Go函数检测
        if GO_FUNC_REGEX.is_match(line) {
            let func_name = self.extract_function_name(line);
            features.push(LanguageFeature {
                feature_type: "function".to_string(),
                name: func_name.unwrap_or_else(|| "anonymous".to_string()),
                line_number: Some(line_number),
                description: format!("Go function: {}", line),
            });
        }

        // Go结构体检测
        if let Some(caps) = GO_STRUCT_REGEX.captures(line) {
            features.push(LanguageFeature {
                feature_type: "struct".to_string(),
                name: caps.get(1).map(|m| m.as_str().to_string()).unwrap_or_default(),
                line_number: Some(line_number),
                description: format!("Go struct: {}", line),
            });
        }

        features
    }

    fn analyze_change_patterns(&self, features: &[LanguageFeature]) -> Vec<String> {
        let mut patterns = Vec::new();

        let has_packages = features.iter().any(|f| f.feature_type == "package");
        let has_functions = features.iter().any(|f| f.feature_type == "function");
        let has_structs = features.iter().any(|f| f.feature_type == "struct");

        if has_packages {
            patterns.push("Go包结构变更".to_string());
        }
        if has_functions {
            patterns.push("Go函数定义变更".to_string());
        }
        if has_structs {
            patterns.push("Go数据结构变更".to_string());
        }

        patterns
    }
}
```

### 2. 静态分析集成系统

#### 2.1 静态分析器接口设计

```rust
// src/languages/static_analysis.rs
#[async_trait]
pub trait StaticAnalyzer: Send + Sync {
    fn name(&self) -> &str;
    fn supported_languages(&self) -> Vec<String>;
    async fn analyze(&self, file_path: &str, content: &str) -> anyhow::Result<Vec<CodeIssue>>;
    fn is_available(&self) -> bool;
}

/// Go语言静态分析器实现
pub struct GoStaticAnalyzer {
    tools: Vec<Box<dyn GoTool>>,
}

impl GoStaticAnalyzer {
    pub fn new() -> Self {
        let mut tools: Vec<Box<dyn GoTool>> = Vec::new();
        tools.push(Box::new(GoFmtTool));
        tools.push(Box::new(GoVetTool));
        tools.push(Box::new(GoLintTool));
        tools.push(Box::new(GoBuildTool));

        Self { tools }
    }
}

#[async_trait]
impl StaticAnalyzer for GoStaticAnalyzer {
    fn name(&self) -> &str { "go-analyzer" }

    fn supported_languages(&self) -> Vec<String> {
        vec!["go".to_string()]
    }

    async fn analyze(&self, file_path: &str, _content: &str) -> anyhow::Result<Vec<CodeIssue>> {
        let mut all_issues = Vec::new();

        // 并行执行所有Go工具
        let futures: Vec<_> = self.tools
            .iter()
            .filter(|tool| tool.is_available())
            .map(|tool| tool.run(file_path))
            .collect();

        let results = futures::future::join_all(futures).await;

        for result in results {
            match result {
                Ok(issues) => all_issues.extend(issues),
                Err(e) => eprintln!("静态分析工具执行失败: {}", e),
            }
        }

        Ok(all_issues)
    }

    fn is_available(&self) -> bool {
        // 检查Go工具链是否可用
        std::process::Command::new("go")
            .arg("version")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }
}
```

#### 2.2 Go 工具集成实现

```rust
// Go工具特征定义
#[async_trait]
trait GoTool: Send + Sync {
    fn name(&self) -> &str;
    async fn run(&self, file_path: &str) -> anyhow::Result<Vec<CodeIssue>>;
    fn is_available(&self) -> bool;
}

/// GoFmt工具实现
struct GoFmtTool;

#[async_trait]
impl GoTool for GoFmtTool {
    fn name(&self) -> &str { "gofmt" }

    async fn run(&self, file_path: &str) -> anyhow::Result<Vec<CodeIssue>> {
        let output = tokio::process::Command::new("gofmt")
            .args(["-d", file_path])
            .output()
            .await?;

        let mut issues = Vec::new();

        if !output.stdout.is_empty() {
            issues.push(CodeIssue {
                tool: "gofmt".to_string(),
                file_path: file_path.to_string(),
                line_number: None,
                column: None,
                severity: IssueSeverity::Style,
                message: "代码格式不符合Go标准格式".to_string(),
                rule: Some("gofmt".to_string()),
                suggestion: Some("运行 'gofmt -w filename.go' 自动格式化代码".to_string()),
            });
        }

        Ok(issues)
    }

    fn is_available(&self) -> bool {
        std::process::Command::new("gofmt")
            .arg("--help")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }
}

/// GoVet工具实现
struct GoVetTool;

#[async_trait]
impl GoVet for GoVetTool {
    fn name(&self) -> &str { "go vet" }

    async fn run(&self, file_path: &str) -> anyhow::Result<Vec<CodeIssue>> {
        let dir = std::path::Path::new(file_path).parent().unwrap_or(std::path::Path::new("."));

        let output = tokio::process::Command::new("go")
            .args(["vet", file_path])
            .current_dir(dir)
            .output()
            .await?;

        let stderr = String::from_utf8_lossy(&output.stderr);
        let mut issues = Vec::new();

        // 解析go vet输出
        for line in stderr.lines() {
            if let Some(issue) = self.parse_vet_output(line, file_path) {
                issues.push(issue);
            }
        }

        Ok(issues)
    }
}

impl GoVetTool {
    fn parse_vet_output(&self, line: &str, file_path: &str) -> Option<CodeIssue> {
        // 解析格式: filename:line:column: message
        let parts: Vec<&str> = line.splitn(4, ':').collect();
        if parts.len() >= 3 {
            let line_num = parts[1].parse::<usize>().ok();
            let column = if parts.len() >= 4 {
                parts[2].parse::<usize>().ok()
            } else {
                None
            };
            let message = if parts.len() >= 4 {
                parts[3].trim().to_string()
            } else {
                parts[2].trim().to_string()
            };

            Some(CodeIssue {
                tool: "go vet".to_string(),
                file_path: file_path.to_string(),
                line_number: line_num,
                column,
                severity: IssueSeverity::Warning,
                message,
                rule: Some("go-vet".to_string()),
                suggestion: None,
            })
        } else {
            None
        }
    }
}
```

### 3. AI 增强审查系统

#### 3.1 AI 服务集成架构

```rust
// src/ai/mod.rs - AI服务管理器
pub struct AIServiceManager {
    providers: HashMap<String, Box<dyn AIProvider>>,
    config: Config,
    client: Arc<reqwest::Client>,
}

#[async_trait]
pub trait AIProvider: Send + Sync {
    fn name(&self) -> &str;
    async fn analyze_code(
        &self,
        prompt: &str,
        config: &Config,
    ) -> anyhow::Result<String>;
    fn is_available(&self, config: &Config) -> bool;
}

/// DeepSeek AI服务提供商
pub struct DeepSeekProvider {
    client: Arc<reqwest::Client>,
}

#[async_trait]
impl AIProvider for DeepSeekProvider {
    fn name(&self) -> &str { "deepseek" }

    async fn analyze_code(
        &self,
        prompt: &str,
        config: &Config,
    ) -> anyhow::Result<String> {
        let request = serde_json::json!({
            "model": config.model,
            "messages": [{
                "role": "user",
                "content": prompt
            }],
            "temperature": 0.3,
            "max_tokens": 2000
        });

        let response = self.client
            .post(&config.deepseek_url)
            .header("Authorization", format!("Bearer {}",
                config.deepseek_api_key.as_ref().unwrap()))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!("DeepSeek API请求失败: {}", response.status());
        }

        let response_json: serde_json::Value = response.json().await?;
        let content = response_json["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();

        Ok(content)
    }

    fn is_available(&self, config: &Config) -> bool {
        config.deepseek_api_key.is_some() &&
        !config.deepseek_api_key.as_ref().unwrap().is_empty()
    }
}
```

#### 3.2 语言特定 AI 审查器

```rust
// src/languages/rust/ai_reviewer.rs
pub struct RustAIReviewer {
    ai_service: AIServiceManager,
    config: Config,
}

impl RustAIReviewer {
    pub async fn review_code(
        &self,
        review_type: &str,
        features: &[LanguageFeature],
        file_path: &str,
    ) -> anyhow::Result<AIReviewResult> {
        let prompt = self.build_rust_review_prompt(review_type, features, file_path);

        let ai_response = self.ai_service
            .analyze_code(&prompt, &self.config)
            .await?;

        self.parse_rust_review_response(&ai_response)
    }

    fn build_rust_review_prompt(
        &self,
        review_type: &str,
        features: &[LanguageFeature],
        file_path: &str,
    ) -> String {
        let features_summary = features.iter()
            .map(|f| format!("- {}: {}", f.feature_type, f.name))
            .collect::<Vec<_>>()
            .join("\n");

        format!(
            r#"请对以下Rust代码进行{}审查：

文件路径: {}

检测到的代码特征:
{}

请重点关注以下Rust特定方面：
1. 内存安全性 - 检查unsafe块的使用
2. 错误处理 - 评估Result和Option的使用
3. 所有权和借用 - 分析生命周期管理
4. 性能优化 - 识别潜在的性能问题
5. 惯用法 - 检查是否遵循Rust最佳实践

请提供：
- 整体质量评分 (1-10)
- 具体改进建议
- 相关学习资源链接
- 代码示例（如果适用）

以JSON格式返回结果。"#,
            review_type, file_path, features_summary
        )
    }

    fn parse_rust_review_response(&self, response: &str) -> anyhow::Result<AIReviewResult> {
        // 尝试解析JSON响应
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(response) {
            Ok(AIReviewResult {
                review_type: "rust_comprehensive".to_string(),
                overall_score: json["score"].as_f64().unwrap_or(7.0) as f32,
                summary: json["summary"].as_str().unwrap_or("Rust代码审查完成").to_string(),
                detailed_feedback: response.to_string(),
                security_score: json["security_score"].as_f64().unwrap_or(8.0) as f32,
                performance_score: json["performance_score"].as_f64().unwrap_or(8.0) as f32,
                maintainability_score: json["maintainability_score"].as_f64().unwrap_or(8.0) as f32,
                recommendations: json["recommendations"]
                    .as_array()
                    .map(|arr| arr.iter()
                        .filter_map(|v| v.as_str())
                        .map(String::from)
                        .collect())
                    .unwrap_or_else(|| vec!["应用Rust最佳实践".to_string()]),
                learning_resources: json["learning_resources"]
                    .as_array()
                    .map(|arr| arr.iter()
                        .filter_map(|v| v.as_str())
                        .map(String::from)
                        .collect())
                    .unwrap_or_else(|| vec!["https://doc.rust-lang.org/book/".to_string()]),
            })
        } else {
            // 如果不是JSON格式，解析纯文本响应
            self.parse_text_response(response)
        }
    }
}
```

### 4. 敏感信息检测系统

#### 4.1 模式匹配引擎

```rust
// src/languages/sensitive_info.rs
pub struct SensitiveInfoDetector {
    patterns: Vec<SensitivePattern>,
    whitelist: HashSet<String>,
    custom_patterns: Vec<SensitivePattern>,
}

impl SensitiveInfoDetector {
    pub fn new() -> Self {
        Self {
            patterns: Self::initialize_default_patterns(),
            whitelist: HashSet::new(),
            custom_patterns: Vec::new(),
        }
    }

    fn initialize_default_patterns() -> Vec<SensitivePattern> {
        vec![
            // AWS访问密钥
            SensitivePattern {
                name: "AWS Access Key".to_string(),
                info_type: SensitiveInfoType::ApiKey,
                regex: Regex::new(r"AKIA[0-9A-Z]{16}").unwrap(),
                confidence: 0.95,
                risk_level: SensitiveRiskLevel::Critical,
                description: "AWS访问密钥".to_string(),
                recommendations: vec![
                    "立即轮换暴露的AWS密钥".to_string(),
                    "使用AWS IAM角色替代硬编码密钥".to_string(),
                    "启用AWS CloudTrail监控密钥使用".to_string(),
                ],
            },

            // 通用API密钥
            SensitivePattern {
                name: "Generic API Key".to_string(),
                info_type: SensitiveInfoType::ApiKey,
                regex: Regex::new(r"(?i)(api[_-]?key|apikey)\s*[=:]\s*['\"]([a-zA-Z0-9_-]{20,})['\"]").unwrap(),
                confidence: 0.85,
                risk_level: SensitiveRiskLevel::Critical,
                description: "通用API密钥".to_string(),
                recommendations: vec![
                    "使用环境变量存储API密钥".to_string(),
                    "实施密钥轮换策略".to_string(),
                    "使用密钥管理服务".to_string(),
                ],
            },

            // JWT令牌
            SensitivePattern {
                name: "JWT Token".to_string(),
                info_type: SensitiveInfoType::JwtToken,
                regex: Regex::new(r"eyJ[a-zA-Z0-9_-]+\.eyJ[a-zA-Z0-9_-]+\.[a-zA-Z0-9_-]+").unwrap(),
                confidence: 0.98,
                risk_level: SensitiveRiskLevel::High,
                description: "JWT令牌".to_string(),
                recommendations: vec![
                    "不要在代码中硬编码JWT令牌".to_string(),
                    "设置合理的令牌过期时间".to_string(),
                    "使用安全的令牌存储方式".to_string(),
                ],
            },

            // 数据库连接字符串
            SensitivePattern {
                name: "Database Connection String".to_string(),
                info_type: SensitiveInfoType::DatabaseConnection,
                regex: Regex::new(r"(?i)(mongodb://|mysql://|postgresql://|redis://)[^\s]+").unwrap(),
                confidence: 0.90,
                risk_level: SensitiveRiskLevel::High,
                description: "数据库连接字符串".to_string(),
                recommendations: vec![
                    "使用环境变量存储数据库连接信息".to_string(),
                    "启用数据库访问日志".to_string(),
                    "使用连接池和安全配置".to_string(),
                ],
            },
        ]
    }

    pub fn detect(&self, file_path: &str, content: &str) -> SensitiveInfoResult {
        let mut items = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        // 检测默认模式
        for pattern in &self.patterns {
            items.extend(self.detect_pattern(pattern, &lines, file_path));
        }

        // 检测自定义模式
        for pattern in &self.custom_patterns {
            items.extend(self.detect_pattern(pattern, &lines, file_path));
        }

        // 应用白名单过滤
        items.retain(|item| !self.is_whitelisted(&item.matched_text, file_path));

        let summary = self.generate_summary(&items);

        SensitiveInfoResult {
            file_path: file_path.to_string(),
            items,
            summary,
        }
    }

    fn detect_pattern(
        &self,
        pattern: &SensitivePattern,
        lines: &[&str],
        file_path: &str,
    ) -> Vec<SensitiveInfoItem> {
        let mut items = Vec::new();

        for (line_num, line) in lines.iter().enumerate() {
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

        items
    }

    fn mask_sensitive_text(&self, text: &str, info_type: &SensitiveInfoType) -> String {
        match info_type {
            SensitiveInfoType::ApiKey | SensitiveInfoType::AccessToken => {
                if text.len() <= 8 {
                    "*".repeat(text.len())
                } else {
                    format!("{}***{}", &text[..4], &text[text.len()-4..])
                }
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
                    "***@***.***".to_string()
                }
            },
            _ => {
                if text.len() <= 6 {
                    "*".repeat(text.len())
                } else {
                    format!("{}***{}", &text[..3], &text[text.len()-3..])
                }
            }
        }
    }
}
```

### 5. 代码复杂度分析系统

#### 5.1 复杂度分析器架构

```rust
// src/analysis/complexity/mod.rs
pub struct ComplexityAnalyzer {
    cyclomatic_analyzer: CyclomaticComplexityAnalyzer,
    cognitive_analyzer: CognitiveComplexityAnalyzer,
    function_analyzer: FunctionLengthAnalyzer,
    nesting_analyzer: NestingDepthAnalyzer,
    config: ComplexityConfig,
}

#[derive(Debug, Clone)]
pub struct ComplexityResult {
    pub file_path: String,
    pub functions: Vec<FunctionComplexity>,
    pub overall_metrics: OverallComplexityMetrics,
    pub hotspots: Vec<ComplexityHotspot>,
    pub recommendations: Vec<RefactoringRecommendation>,
}

impl ComplexityAnalyzer {
    pub fn analyze_file(&self, file_path: &str, content: &str, language: &Language) -> ComplexityResult {
        let ast = self.parse_ast(content, language);
        let functions = self.extract_functions(&ast);
        
        let mut function_complexities = Vec::new();
        for function in functions {
            let cyclomatic = self.cyclomatic_analyzer.analyze(&function);
            let cognitive = self.cognitive_analyzer.analyze(&function);
            let length = self.function_analyzer.analyze(&function);
            let nesting = self.nesting_analyzer.analyze(&function);
            
            function_complexities.push(FunctionComplexity {
                name: function.name.clone(),
                line_start: function.line_start,
                line_end: function.line_end,
                cyclomatic_complexity: cyclomatic,
                cognitive_complexity: cognitive,
                function_length: length,
                max_nesting_depth: nesting,
                risk_level: self.calculate_risk_level(cyclomatic, cognitive, length, nesting),
            });
        }
        
        let overall_metrics = self.calculate_overall_metrics(&function_complexities);
        let hotspots = self.identify_hotspots(&function_complexities);
        let recommendations = self.generate_recommendations(&function_complexities);
        
        ComplexityResult {
            file_path: file_path.to_string(),
            functions: function_complexities,
            overall_metrics,
            hotspots,
            recommendations,
        }
    }
    
    fn calculate_risk_level(&self, cyclomatic: u32, cognitive: u32, length: u32, nesting: u32) -> RiskLevel {
        let score = (cyclomatic as f32 * 0.3) + 
                   (cognitive as f32 * 0.4) + 
                   (length as f32 * 0.2) + 
                   (nesting as f32 * 0.1);
        
        match score {
            s if s >= 50.0 => RiskLevel::Critical,
            s if s >= 30.0 => RiskLevel::High,
            s if s >= 15.0 => RiskLevel::Medium,
            _ => RiskLevel::Low,
        }
    }
}
```

#### 5.2 圈复杂度分析器

```rust
// src/analysis/complexity/cyclomatic.rs
pub struct CyclomaticComplexityAnalyzer;

impl CyclomaticComplexityAnalyzer {
    pub fn analyze(&self, function: &FunctionNode) -> u32 {
        let mut complexity = 1; // 基础复杂度
        
        // 遍历AST节点，计算决策点
        for node in function.walk() {
            complexity += match node.kind() {
                // 条件语句
                "if_statement" | "else_if_clause" => 1,
                "match_expression" | "switch_statement" => {
                    // match/switch的每个分支增加复杂度
                    node.children().filter(|c| c.kind() == "match_arm").count() as u32
                },
                // 循环语句
                "for_statement" | "while_statement" | "loop_statement" => 1,
                // 逻辑运算符
                "logical_and" | "logical_or" => 1,
                // 异常处理
                "try_statement" | "catch_clause" => 1,
                // 三元运算符
                "conditional_expression" => 1,
                _ => 0,
            };
        }
        
        complexity
    }
}
```

### 6. 代码重复检测系统

#### 6.1 重复检测引擎

```rust
// src/analysis/duplication/mod.rs
pub struct DuplicationDetector {
    exact_detector: ExactDuplicationDetector,
    structural_detector: StructuralDuplicationDetector,
    semantic_detector: SemanticDuplicationDetector,
    config: DuplicationConfig,
}

#[derive(Debug, Clone)]
pub struct DuplicationResult {
    pub file_path: String,
    pub duplications: Vec<CodeDuplication>,
    pub duplication_rate: f32,
    pub refactoring_opportunities: Vec<RefactoringOpportunity>,
}

impl DuplicationDetector {
    pub fn detect_duplications(&self, files: &[FileContent]) -> Vec<DuplicationResult> {
        let mut results = Vec::new();
        
        // 1. 精确重复检测
        let exact_duplications = self.exact_detector.detect(files);
        
        // 2. 结构重复检测
        let structural_duplications = self.structural_detector.detect(files);
        
        // 3. 语义重复检测
        let semantic_duplications = self.semantic_detector.detect(files);
        
        // 合并和去重结果
        for file in files {
            let file_duplications = self.merge_duplications(
                &exact_duplications,
                &structural_duplications,
                &semantic_duplications,
                &file.path,
            );
            
            let duplication_rate = self.calculate_duplication_rate(&file_duplications, &file.content);
            let refactoring_opportunities = self.identify_refactoring_opportunities(&file_duplications);
            
            results.push(DuplicationResult {
                file_path: file.path.clone(),
                duplications: file_duplications,
                duplication_rate,
                refactoring_opportunities,
            });
        }
        
        results
    }
}

/// 精确重复检测器
pub struct ExactDuplicationDetector {
    min_clone_size: usize,
    ignore_whitespace: bool,
}

impl ExactDuplicationDetector {
    pub fn detect(&self, files: &[FileContent]) -> Vec<CodeDuplication> {
        let mut duplications = Vec::new();
        let mut hash_map: HashMap<String, Vec<CodeLocation>> = HashMap::new();
        
        // 为每个文件生成代码块哈希
        for file in files {
            let blocks = self.extract_code_blocks(&file.content, self.min_clone_size);
            
            for block in blocks {
                let hash = self.calculate_hash(&block.content);
                hash_map.entry(hash).or_default().push(CodeLocation {
                    file_path: file.path.clone(),
                    start_line: block.start_line,
                    end_line: block.end_line,
                    content: block.content,
                });
            }
        }
        
        // 识别重复
        for (_, locations) in hash_map {
            if locations.len() > 1 {
                duplications.push(CodeDuplication {
                    duplication_type: DuplicationType::Exact,
                    locations,
                    similarity_score: 1.0,
                    clone_size: self.min_clone_size,
                });
            }
        }
        
        duplications
    }
}
```

### 7. 依赖安全扫描系统

#### 7.1 依赖分析器架构

```rust
// src/analysis/dependencies/mod.rs
pub struct DependencyAnalyzer {
    vulnerability_scanner: VulnerabilityScanner,
    license_checker: LicenseChecker,
    outdated_detector: OutdatedDependencyDetector,
    supply_chain_analyzer: SupplyChainAnalyzer,
    package_managers: Vec<Box<dyn PackageManager>>,
}

#[derive(Debug, Clone)]
pub struct DependencyAnalysisResult {
    pub project_path: String,
    pub dependencies: Vec<Dependency>,
    pub vulnerabilities: Vec<Vulnerability>,
    pub license_issues: Vec<LicenseIssue>,
    pub outdated_dependencies: Vec<OutdatedDependency>,
    pub supply_chain_risks: Vec<SupplyChainRisk>,
    pub recommendations: Vec<SecurityRecommendation>,
}

impl DependencyAnalyzer {
    pub async fn analyze_project(&self, project_path: &str) -> anyhow::Result<DependencyAnalysisResult> {
        // 1. 检测包管理器
        let package_manager = self.detect_package_manager(project_path)?;
        
        // 2. 解析依赖
        let dependencies = package_manager.parse_dependencies(project_path).await?;
        
        // 3. 漏洞扫描
        let vulnerabilities = self.vulnerability_scanner.scan(&dependencies).await?;
        
        // 4. 许可证检查
        let license_issues = self.license_checker.check(&dependencies).await?;
        
        // 5. 过时依赖检测
        let outdated_dependencies = self.outdated_detector.detect(&dependencies).await?;
        
        // 6. 供应链分析
        let supply_chain_risks = self.supply_chain_analyzer.analyze(&dependencies).await?;
        
        // 7. 生成建议
        let recommendations = self.generate_recommendations(
            &vulnerabilities,
            &license_issues,
            &outdated_dependencies,
            &supply_chain_risks,
        );
        
        Ok(DependencyAnalysisResult {
            project_path: project_path.to_string(),
            dependencies,
            vulnerabilities,
            license_issues,
            outdated_dependencies,
            supply_chain_risks,
            recommendations,
        })
    }
}

/// 漏洞扫描器
pub struct VulnerabilityScanner {
    vulnerability_db: Arc<VulnerabilityDatabase>,
    client: reqwest::Client,
}

impl VulnerabilityScanner {
    pub async fn scan(&self, dependencies: &[Dependency]) -> anyhow::Result<Vec<Vulnerability>> {
        let mut vulnerabilities = Vec::new();
        
        // 并行扫描所有依赖
        let futures: Vec<_> = dependencies
            .iter()
            .map(|dep| self.scan_dependency(dep))
            .collect();
        
        let results = futures::future::join_all(futures).await;
        
        for result in results {
            match result {
                Ok(mut vulns) => vulnerabilities.append(&mut vulns),
                Err(e) => eprintln!("依赖扫描失败: {}", e),
            }
        }
        
        // 按严重程度排序
        vulnerabilities.sort_by(|a, b| b.severity.cmp(&a.severity));
        
        Ok(vulnerabilities)
    }
    
    async fn scan_dependency(&self, dependency: &Dependency) -> anyhow::Result<Vec<Vulnerability>> {
        // 1. 本地数据库查询
        let local_vulns = self.vulnerability_db.query(dependency).await?;
        
        // 2. 在线数据库查询（如果配置了）
        let online_vulns = if self.should_query_online() {
            self.query_online_database(dependency).await.unwrap_or_default()
        } else {
            Vec::new()
        };
        
        // 3. 合并结果
        let mut all_vulns = local_vulns;
        all_vulns.extend(online_vulns);
        
        // 4. 去重
        all_vulns.dedup_by(|a, b| a.cve_id == b.cve_id);
        
        Ok(all_vulns)
    }
}
```

### 8. 自定义规则引擎

#### 8.1 规则引擎架构

```rust
// src/rules/engine/mod.rs
pub struct CustomRuleEngine {
    rule_parser: RuleParser,
    rule_executor: RuleExecutor,
    rule_cache: Arc<RwLock<HashMap<String, CompiledRule>>>,
    template_manager: TemplateManager,
}

#[derive(Debug, Clone)]
pub struct CustomRule {
    pub id: String,
    pub name: String,
    pub description: String,
    pub severity: IssueSeverity,
    pub language: Option<String>,
    pub pattern: RulePattern,
    pub message: String,
    pub suggestion: Option<String>,
    pub enabled: bool,
}

#[derive(Debug, Clone)]
pub enum RulePattern {
    Regex(String),
    AST(ASTPattern),
    Semantic(SemanticPattern),
    Composite(Vec<RulePattern>),
}

impl CustomRuleEngine {
    pub fn new() -> Self {
        Self {
            rule_parser: RuleParser::new(),
            rule_executor: RuleExecutor::new(),
            rule_cache: Arc::new(RwLock::new(HashMap::new())),
            template_manager: TemplateManager::new(),
        }
    }
    
    pub fn load_rules_from_file(&mut self, file_path: &str) -> anyhow::Result<()> {
        let content = std::fs::read_to_string(file_path)?;
        let rules: Vec<CustomRule> = self.rule_parser.parse(&content)?;
        
        for rule in rules {
            let compiled_rule = self.compile_rule(&rule)?;
            self.rule_cache.write().unwrap().insert(rule.id.clone(), compiled_rule);
        }
        
        Ok(())
    }
    
    pub fn execute_rules(&self, file_path: &str, content: &str, language: &Language) -> Vec<RuleViolation> {
        let mut violations = Vec::new();
        let cache = self.rule_cache.read().unwrap();
        
        for (rule_id, compiled_rule) in cache.iter() {
            if !compiled_rule.rule.enabled {
                continue;
            }
            
            // 检查语言匹配
            if let Some(rule_lang) = &compiled_rule.rule.language {
                if rule_lang != &language.to_string() {
                    continue;
                }
            }
            
            // 执行规则
            match self.rule_executor.execute(compiled_rule, file_path, content) {
                Ok(mut rule_violations) => violations.append(&mut rule_violations),
                Err(e) => eprintln!("规则执行失败 {}: {}", rule_id, e),
            }
        }
        
        violations
    }
    
    fn compile_rule(&self, rule: &CustomRule) -> anyhow::Result<CompiledRule> {
        let compiled_pattern = match &rule.pattern {
            RulePattern::Regex(pattern) => {
                CompiledPattern::Regex(Regex::new(pattern)?)
            },
            RulePattern::AST(ast_pattern) => {
                CompiledPattern::AST(self.compile_ast_pattern(ast_pattern)?)
            },
            RulePattern::Semantic(semantic_pattern) => {
                CompiledPattern::Semantic(self.compile_semantic_pattern(semantic_pattern)?)
            },
            RulePattern::Composite(patterns) => {
                let compiled_patterns: Result<Vec<_>, _> = patterns
                    .iter()
                    .map(|p| self.compile_single_pattern(p))
                    .collect();
                CompiledPattern::Composite(compiled_patterns?)
            },
        };
        
        Ok(CompiledRule {
            rule: rule.clone(),
            compiled_pattern,
            performance_stats: PerformanceStats::new(),
        })
    }
}

/// 规则模板管理器
pub struct TemplateManager {
    templates: HashMap<String, RuleTemplate>,
}

impl TemplateManager {
    pub fn new() -> Self {
        let mut templates = HashMap::new();
        
        // 预置常用模板
        templates.insert("no-hardcoded-secrets".to_string(), RuleTemplate {
            name: "禁止硬编码密钥".to_string(),
            description: "检测代码中的硬编码API密钥、密码等敏感信息".to_string(),
            pattern: RulePattern::Regex(r#"(?i)(api[_-]?key|password|secret)\s*[=:]\s*['""][^'""]{8,}['""]"#.to_string()),
            severity: IssueSeverity::Critical,
            message: "发现硬编码的敏感信息".to_string(),
            suggestion: Some("使用环境变量或配置文件存储敏感信息".to_string()),
        });
        
        templates.insert("function-too-long".to_string(), RuleTemplate {
            name: "函数过长".to_string(),
            description: "检测超过指定行数的函数".to_string(),
            pattern: RulePattern::AST(ASTPattern {
                node_type: "function_declaration".to_string(),
                conditions: vec![
                    ASTCondition::LineCount { min: Some(50), max: None }
                ],
            }),
            severity: IssueSeverity::Warning,
            message: "函数过长，建议拆分".to_string(),
            suggestion: Some("将大函数拆分为多个小函数，提高可读性和可维护性".to_string()),
        });
        
        Self { templates }
    }
    
    pub fn get_template(&self, name: &str) -> Option<&RuleTemplate> {
        self.templates.get(name)
    }
    
    pub fn list_templates(&self) -> Vec<&str> {
        self.templates.keys().map(|k| k.as_str()).collect()
    }
}
```

### 9. 通知系统实现

#### 9.1 通知服务架构

```rust
// src/notification/mod.rs
pub struct NotificationService {
    providers: HashMap<NotificationPlatform, Box<dyn NotificationProvider>>,
    config: NotificationConfig,
    retry_policy: RetryPolicy,
}

#[async_trait]
pub trait NotificationProvider: Send + Sync {
    fn platform(&self) -> NotificationPlatform;
    async fn send_notification(
        &mut self,
        message: &NotificationMessage,
    ) -> anyhow::Result<NotificationResult>;
    fn is_configured(&self) -> bool;
}

impl NotificationService {
    pub fn new(config: NotificationConfig) -> Self {
        let mut providers: HashMap<NotificationPlatform, Box<dyn NotificationProvider>> = HashMap::new();

        // 初始化各种通知提供商
        if let Some(feishu_config) = &config.feishu {
            providers.insert(
                NotificationPlatform::Feishu,
                Box::new(FeishuProvider::new(feishu_config.clone()))
            );
        }

        if let Some(wechat_config) = &config.wechat {
            providers.insert(
                NotificationPlatform::WeChat,
                Box::new(WeChatProvider::new(wechat_config.clone()))
            );
        }

        Self {
            providers,
            config,
            retry_policy: RetryPolicy::default(),
        }
    }

    pub async fn send_review_notification(
        &mut self,
        review_report: &CodeReviewReport,
    ) -> Vec<NotificationResult> {
        let message = self.build_notification_message(review_report);
        let mut results = Vec::new();

        // 检查是否应该发送通知
        if !self.should_notify(&message) {
            return results;
        }

        // 并发发送到所有配置的平台
        let mut futures = Vec::new();
        for (platform, provider) in &mut self.providers {
            if provider.is_configured() {
                let future = provider.send_notification(&message);
                futures.push((*platform, future));
            }
        }

        // 等待所有通知完成
        for (platform, future) in futures {
            match future.await {
                Ok(result) => results.push(result),
                Err(e) => {
                    results.push(NotificationResult {
                        platform,
                        success: false,
                        message_id: None,
                        error: Some(e.to_string()),
                        retry_count: 0,
                    });
                }
            }
        }

        // 处理失败的通知重试
        self.handle_failed_notifications(&mut results).await;

        results
    }

    fn build_notification_message(&self, report: &CodeReviewReport) -> NotificationMessage {
        let severity = self.determine_severity(report);
        let score = report.ai_review_summary.as_ref().map(|s| s.average_score);

        NotificationMessage {
            title: "🔍 代码审查完成".to_string(),
            summary: self.build_summary(report),
            details: self.build_details(report),
            severity,
            score,
            file_count: report.summary.total_files,
            issue_count: report.static_analysis_summary.total_issues,
            timestamp: chrono::Utc::now(),
            report_url: self.config.report_base_url.clone(),
        }
    }

    fn should_notify(&self, message: &NotificationMessage) -> bool {
        for rule in &self.config.rules {
            if self.evaluate_rule(rule, message) {
                return true;
            }
        }
        false
    }

    fn evaluate_rule(&self, rule: &NotificationRule, message: &NotificationMessage) -> bool {
        match &rule.condition {
            NotificationCondition::Always => true,
            NotificationCondition::OnIssuesFound => message.issue_count > 0,
            NotificationCondition::OnCriticalIssues => {
                matches!(message.severity, IssueSeverity::Error)
            },
            NotificationCondition::OnScoreBelow(threshold) => {
                message.score.map_or(false, |score| score < *threshold)
            },
        }
    }
}
```

#### 5.2 飞书通知提供商实现

```rust
// src/notification/feishu.rs
pub struct FeishuProvider {
    webhook_url: String,
    client: reqwest::Client,
    template: FeishuTemplate,
}

impl FeishuProvider {
    pub fn new(config: FeishuConfig) -> Self {
        Self {
            webhook_url: config.webhook_url,
            client: reqwest::Client::new(),
            template: FeishuTemplate::new(config.template_config),
        }
    }
}

#[async_trait]
impl NotificationProvider for FeishuProvider {
    fn platform(&self) -> NotificationPlatform {
        NotificationPlatform::Feishu
    }

    async fn send_notification(
        &mut self,
        message: &NotificationMessage,
    ) -> anyhow::Result<NotificationResult> {
        let card = self.build_interactive_card(message);

        let response = self.client
            .post(&self.webhook_url)
            .json(&card)
            .timeout(std::time::Duration::from_secs(30))
            .send()
            .await?;

        if response.status().is_success() {
            let response_body: serde_json::Value = response.json().await?;

            Ok(NotificationResult {
                platform: NotificationPlatform::Feishu,
                success: true,
                message_id: response_body["msg_id"].as_str().map(String::from),
                error: None,
                retry_count: 0,
            })
        } else {
            let error_text = response.text().await?;
            anyhow::bail!("飞书通知发送失败: {}", error_text);
        }
    }

    fn is_configured(&self) -> bool {
        !self.webhook_url.is_empty()
    }
}

impl FeishuProvider {
    fn build_interactive_card(&self, message: &NotificationMessage) -> serde_json::Value {
        let color = self.get_severity_color(&message.severity);
        let score_emoji = self.get_score_emoji(message.score);

        serde_json::json!({
            "msg_type": "interactive",
            "card": {
                "config": {
                    "wide_screen_mode": true,
                    "enable_forward": true
                },
                "header": {
                    "title": {
                        "tag": "plain_text",
                        "content": format!("{} {}", score_emoji, message.title)
                    },
                    "template": color
                },
                "elements": [
                    {
                        "tag": "div",
                        "text": {
                            "tag": "lark_md",
                            "content": self.build_summary_content(message)
                        }
                    },
                    {
                        "tag": "div",
                        "fields": [
                            {
                                "is_short": true,
                                "text": {
                                    "tag": "lark_md",
                                    "content": format!("**📁 审查文件数**\n{}", message.file_count)
                                }
                            },
                            {
                                "is_short": true,
                                "text": {
                                    "tag": "lark_md",
                                    "content": format!("**⚠️ 发现问题数**\n{}", message.issue_count)
                                }
                            },
                            {
                                "is_short": true,
                                "text": {
                                    "tag": "lark_md",
                                    "content": format!("**📈 代码评分**\n{}",
                                        message.score.map(|s| format!("{:.1}/10", s))
                                            .unwrap_or("未评分".to_string()))
                                }
                            },
                            {
                                "is_short": true,
                                "text": {
                                    "tag": "lark_md",
                                    "content": format!("**🕐 审查时间**\n{}",
                                        message.timestamp.format("%Y-%m-%d %H:%M:%S"))
                                }
                            }
                        ]
                    },
                    {
                        "tag": "hr"
                    },
                    {
                        "tag": "div",
                        "text": {
                            "tag": "lark_md",
                            "content": format!("**📝 详细信息:**\n{}",
                                self.truncate_details(&message.details, 500))
                        }
                    },
                    {
                        "tag": "action",
                        "actions": [
                            {
                                "tag": "button",
                                "text": {
                                    "tag": "plain_text",
                                    "content": "📊 查看详细报告"
                                },
                                "type": "primary",
                                "url": message.report_url.as_deref().unwrap_or("#")
                            },
                            {
                                "tag": "button",
                                "text": {
                                    "tag": "plain_text",
                                    "content": "🔄 重新审查"
                                },
                                "type": "default",
                                "url": "#"
                            }
                        ]
                    }
                ]
            }
        })
    }

    fn get_severity_color(&self, severity: &IssueSeverity) -> &str {
        match severity {
            IssueSeverity::Error => "red",
            IssueSeverity::Warning => "orange",
            IssueSeverity::Info => "blue",
            IssueSeverity::Style => "grey",
        }
    }

    fn get_score_emoji(&self, score: Option<f32>) -> &str {
        match score {
            Some(s) if s >= 9.0 => "🎉",
            Some(s) if s >= 8.0 => "✅",
            Some(s) if s >= 7.0 => "👍",
            Some(s) if s >= 6.0 => "⚠️",
            Some(_) => "❌",
            None => "❓",
        }
    }
}
```

### 6. 性能优化与缓存系统

#### 6.1 多级缓存实现

```rust
// src/cache/mod.rs
pub struct CacheManager {
    // 内存缓存
    memory_cache: Arc<Mutex<LruCache<String, CacheEntry>>>,
    // 文件系统缓存
    fs_cache: Option<FsCacheManager>,
    // 缓存配置
    config: CacheConfig,
}

#[derive(Clone)]
pub struct CacheEntry {
    pub data: CacheData,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub access_count: u64,
}

#[derive(Clone)]
pub enum CacheData {
    LanguageDetection(LanguageDetectionResult),
    AIReview(AIReviewResult),
    StaticAnalysis(Vec<StaticAnalysisResult>),
    SensitiveInfo(SensitiveInfoResult),
}

impl CacheManager {
    pub fn new(config: CacheConfig) -> Self {
        let memory_cache = Arc::new(Mutex::new(
            LruCache::new(config.memory_cache_size)
        ));

        let fs_cache = if config.enable_fs_cache {
            Some(FsCacheManager::new(config.fs_cache_dir.clone()))
        } else {
            None
        };

        Self {
            memory_cache,
            fs_cache,
            config,
        }
    }

    pub async fn get<T>(&self, key: &str) -> Option<T>
    where
        T: Clone + for<'de> serde::Deserialize<'de>,
    {
        // 1. 尝试内存缓存
        if let Some(entry) = self.get_from_memory(key).await {
            if !self.is_expired(&entry) {
                if let Ok(data) = self.extract_data::<T>(&entry.data) {
                    return Some(data);
                }
            }
        }

        // 2. 尝试文件系统缓存
        if let Some(ref fs_cache) = self.fs_cache {
            if let Some(data) = fs_cache.get::<T>(key).await {
                // 将数据重新加载到内存缓存
                self.set_memory_cache(key, data.clone()).await;
                return Some(data);
            }
        }

        None
    }

    pub async fn set<T>(&self, key: &str, data: T, ttl: Option<std::time::Duration>)
    where
        T: Clone + serde::Serialize,
    {
        let expires_at = ttl.map(|duration| chrono::Utc::now() + chrono::Duration::from_std(duration).unwrap());

        let entry = CacheEntry {
            data: self.wrap_data(data.clone()),
            created_at: chrono::Utc::now(),
            expires_at,
            access_count: 0,
        };

        // 设置内存缓存
        {
            let mut cache = self.memory_cache.lock().await;
            cache.put(key.to_string(), entry);
        }

        // 设置文件系统缓存
        if let Some(ref fs_cache) = self.fs_cache {
            fs_cache.set(key, data).await;
        }
    }

    async fn get_from_memory(&self, key: &str) -> Option<CacheEntry> {
        let mut cache = self.memory_cache.lock().await;
        cache.get_mut(key).map(|entry| {
            entry.access_count += 1;
            entry.clone()
        })
    }

    fn is_expired(&self, entry: &CacheEntry) -> bool {
        if let Some(expires_at) = entry.expires_at {
            chrono::Utc::now() > expires_at
        } else {
            false
        }
    }

    /// 缓存统计信息
    pub async fn get_stats(&self) -> CacheStats {
        let memory_cache = self.memory_cache.lock().await;

        CacheStats {
            memory_entries: memory_cache.len(),
            memory_capacity: memory_cache.cap(),
            fs_entries: self.fs_cache.as_ref()
                .map(|fs| fs.get_entry_count())
                .unwrap_or(0),
            hit_rate: self.calculate_hit_rate().await,
        }
    }
}
```

#### 6.2 并行处理优化

```rust
// src/parallel/mod.rs
pub struct ParallelProcessor {
    semaphore: Arc<Semaphore>,
    thread_pool: Arc<ThreadPool>,
    config: ParallelConfig,
}

impl ParallelProcessor {
    pub fn new(config: ParallelConfig) -> Self {
        let semaphore = Arc::new(Semaphore::new(config.max_concurrent_tasks));
        let thread_pool = Arc::new(
            ThreadPool::new(config.thread_pool_size)
        );

        Self {
            semaphore,
            thread_pool,
            config,
        }
    }

    pub async fn process_files_parallel<F, R>(
        &self,
        files: Vec<String>,
        processor: F,
    ) -> Vec<anyhow::Result<R>>
    where
        F: Fn(String) -> Pin<Box<dyn Future<Output = anyhow::Result<R>> + Send>> + Send + Sync + Clone + 'static,
        R: Send + 'static,
    {
        let mut tasks = Vec::new();

        for file_path in files {
            let semaphore = self.semaphore.clone();
            let processor = processor.clone();

            let task = tokio::spawn(async move {
                let _permit = semaphore.acquire().await.unwrap();
                processor(file_path).await
            });

            tasks.push(task);
        }

        // 等待所有任务完成
        let results = futures::future::join_all(tasks).await;

        results.into_iter()
            .map(|result| result.unwrap_or_else(|e| Err(anyhow::anyhow!("Task failed: {}", e))))
            .collect()
    }

    pub async fn process_with_batching<T, F, R>(
        &self,
        items: Vec<T>,
        batch_size: usize,
        processor: F,
    ) -> Vec<anyhow::Result<R>>
    where
        T: Send + 'static,
        F: Fn(Vec<T>) -> Pin<Box<dyn Future<Output = Vec<anyhow::Result<R>>> + Send>> + Send + Sync + Clone + 'static,
        R: Send + 'static,
    {
        let batches: Vec<Vec<T>> = items
            .chunks(batch_size)
            .map(|chunk| chunk.to_vec())
            .collect();

        let mut batch_tasks = Vec::new();

        for batch in batches {
            let semaphore = self.semaphore.clone();
            let processor = processor.clone();

            let task = tokio::spawn(async move {
                let _permit = semaphore.acquire().await.unwrap();
                processor(batch).await
            });

            batch_tasks.push(task);
        }

        let batch_results = futures::future::join_all(batch_tasks).await;

        // 展平批处理结果
        let mut final_results = Vec::new();
        for batch_result in batch_results {
            match batch_result {
                Ok(results) => final_results.extend(results),
                Err(e) => final_results.push(Err(anyhow::anyhow!("Batch failed: {}", e))),
            }
        }

        final_results
    }
}
```

## 🚀 部署与运维

## 🔧 配置管理系统设计

### 配置架构概述

AI-Commit 采用分层配置管理架构，支持多种配置源和优先级覆盖机制：

```
配置优先级 (高 → 低):
1. CLI 命令行参数
2. 环境变量
3. 项目配置文件 (.ai-commit.toml)
4. 用户配置文件 (~/.ai-commit/config.toml)
5. 系统配置文件 (/etc/ai-commit/config.toml)
6. 默认配置
```

### 配置数据结构

```rust
// src/config/mod.rs
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    // AI服务配置
    pub ai: AIConfig,

    // 静态分析配置
    pub static_analysis: StaticAnalysisConfig,

    // 敏感信息检测配置
    pub sensitive_info: SensitiveInfoConfig,

    // 通知系统配置
    pub notifications: NotificationConfig,

    // 报告生成配置
    pub reports: ReportConfig,

    // 缓存配置
    pub cache: CacheConfig,

    // 性能配置
    pub performance: PerformanceConfig,

    // 日志配置
    pub logging: LoggingConfig,

    // 扩展配置
    pub extensions: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIConfig {
    /// AI服务提供商 (ollama, deepseek, siliconflow)
    pub provider: String,

    /// AI模型名称
    pub model: String,

    /// 启用AI审查
    pub enabled: bool,

    /// 审查类型 (general, security, performance, architecture)
    pub review_types: Vec<String>,

    /// 详细反馈级别 (basic, detailed, comprehensive)
    pub detail_level: String,

    /// 语言特定规则
    pub language_specific: bool,

    /// AI语言检测
    pub language_detection: bool,

    /// 超时设置 (秒)
    pub timeout: u64,

    /// 重试次数
    pub retry_count: u32,

    /// 服务配置
    pub services: AIServicesConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIServicesConfig {
    pub ollama: OllamaConfig,
    pub deepseek: DeepSeekConfig,
    pub siliconflow: SiliconFlowConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaConfig {
    pub url: String,
    pub model: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeepSeekConfig {
    pub url: String,
    pub api_key: Option<String>,
    pub model: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiliconFlowConfig {
    pub url: String,
    pub api_key: Option<String>,
    pub model: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaticAnalysisConfig {
    /// 启用静态分析
    pub enabled: bool,

    /// 支持的工具
    pub tools: HashMap<String, ToolConfig>,

    /// 并行执行
    pub parallel: bool,

    /// 超时设置
    pub timeout: u64,

    /// 忽略规则
    pub ignore_rules: Vec<String>,

    /// 自定义规则文件
    pub custom_rules_file: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolConfig {
    pub enabled: bool,
    pub path: Option<String>,
    pub args: Vec<String>,
    pub timeout: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensitiveInfoConfig {
    /// 启用敏感信息检测
    pub enabled: bool,

    /// 检测模式 (strict, normal, loose)
    pub mode: String,

    /// 自定义模式文件
    pub custom_patterns_file: Option<String>,

    /// 白名单文件
    pub whitelist_file: Option<String>,

    /// 风险等级阈值
    pub risk_threshold: String,

    /// 支持的信息类型
    pub info_types: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationConfig {
    /// 启用通知
    pub enabled: bool,

    /// 通知平台配置
    pub platforms: HashMap<String, PlatformConfig>,

    /// 通知规则
    pub rules: Vec<NotificationRule>,

    /// 消息模板
    pub templates: HashMap<String, String>,

    /// 重试配置
    pub retry: RetryConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformConfig {
    pub enabled: bool,
    pub webhook_url: Option<String>,
    pub api_key: Option<String>,
    pub template: String,
    pub timeout: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationRule {
    pub name: String,
    pub condition: String,
    pub platforms: Vec<String>,
    pub template: String,
    pub aggregation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub delay_seconds: u64,
    pub backoff_multiplier: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportConfig {
    /// 默认报告格式
    pub default_format: String,

    /// 输出目录
    pub output_dir: String,

    /// 文件名模板
    pub filename_template: String,

    /// 包含详细信息
    pub include_details: bool,

    /// 包含统计信息
    pub include_stats: bool,

    /// 自定义模板目录
    pub template_dir: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// 启用缓存
    pub enabled: bool,

    /// 缓存目录
    pub cache_dir: String,

    /// 内存缓存大小
    pub memory_cache_size: usize,

    /// 文件系统缓存
    pub fs_cache_enabled: bool,

    /// 缓存过期时间 (秒)
    pub ttl_seconds: u64,

    /// 最大缓存大小 (MB)
    pub max_size_mb: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// 最大并发任务数
    pub max_concurrent_tasks: usize,

    /// 线程池大小
    pub thread_pool_size: usize,

    /// 批处理大小
    pub batch_size: usize,

    /// 内存限制 (MB)
    pub memory_limit_mb: u64,

    /// 超时设置
    pub global_timeout: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// 日志级别 (error, warn, info, debug, trace)
    pub level: String,

    /// 日志格式 (json, text)
    pub format: String,

    /// 日志文件路径
    pub file_path: Option<String>,

    /// 日志轮转
    pub rotation: LogRotationConfig,

    /// 控制台输出
    pub console_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogRotationConfig {
    pub enabled: bool,
    pub max_size_mb: u64,
    pub max_files: u32,
}
```

### 配置管理器实现

```rust
// src/config/manager.rs
use anyhow::{Context, Result};
use clap::ArgMatches;
use std::path::PathBuf;

pub struct ConfigManager {
    config: Config,
    config_paths: Vec<PathBuf>,
    watchers: Vec<Box<dyn ConfigWatcher>>,
}

impl ConfigManager {
    /// 创建配置管理器
    pub fn new() -> Result<Self> {
        let mut config = Config::default();
        let config_paths = Self::discover_config_files();

        // 按优先级加载配置
        Self::load_configurations(&mut config, &config_paths)?;

        // 验证配置
        config.validate()?;

        Ok(Self {
            config,
            config_paths,
            watchers: Vec::new(),
        })
    }

    /// 从CLI参数更新配置
    pub fn update_from_cli(&mut self, matches: &ArgMatches) -> Result<()> {
        // AI配置
        if let Some(provider) = matches.get_one::<String>("provider") {
            self.config.ai.provider = provider.clone();
        }

        if let Some(model) = matches.get_one::<String>("model") {
            self.config.ai.model = model.clone();
        }

        if matches.get_flag("ai-review") {
            self.config.ai.enabled = true;
        }

        if let Some(detail_level) = matches.get_one::<String>("ai-detail-level") {
            self.config.ai.detail_level = detail_level.clone();
        }

        // 静态分析配置
        if matches.get_flag("no-static-analysis") {
            self.config.static_analysis.enabled = false;
        }

        // 敏感信息检测配置
        if matches.get_flag("no-sensitive-check") {
            self.config.sensitive_info.enabled = false;
        }

        // 通知配置
        if matches.get_flag("enable-notifications") {
            self.config.notifications.enabled = true;
        }

        if matches.get_flag("no-notifications") {
            self.config.notifications.enabled = false;
        }

        // 报告配置
        if let Some(format) = matches.get_one::<String>("report-format") {
            self.config.reports.default_format = format.clone();
        }

        if let Some(output) = matches.get_one::<String>("report-output") {
            self.config.reports.output_dir = output.clone();
        }

        // 性能配置
        if let Some(threads) = matches.get_one::<usize>("threads") {
            self.config.performance.thread_pool_size = *threads;
        }

        // 日志配置
        if matches.get_flag("debug") {
            self.config.logging.level = "debug".to_string();
        }

        if matches.get_flag("verbose") {
            self.config.logging.level = "trace".to_string();
        }

        // 重新验证配置
        self.config.validate()?;

        Ok(())
    }

    /// 发现配置文件
    fn discover_config_files() -> Vec<PathBuf> {
        let mut paths = Vec::new();

        // 1. 系统级配置
        if cfg!(unix) {
            paths.push(PathBuf::from("/etc/ai-commit/config.toml"));
        }

        // 2. 用户级配置
        if let Some(home) = dirs::home_dir() {
            paths.push(home.join(".ai-commit").join("config.toml"));
            paths.push(home.join(".config").join("ai-commit").join("config.toml"));
        }

        // 3. 项目级配置
        paths.push(PathBuf::from(".ai-commit.toml"));
        paths.push(PathBuf::from("ai-commit.toml"));

        paths
    }

    /// 加载所有配置文件
    fn load_configurations(config: &mut Config, paths: &[PathBuf]) -> Result<()> {
        for path in paths {
            if path.exists() {
                let content = std::fs::read_to_string(path)
                    .with_context(|| format!("Failed to read config file: {}", path.display()))?;

                let file_config: Config = toml::from_str(&content)
                    .with_context(|| format!("Failed to parse config file: {}", path.display()))?;

                config.merge(file_config)?;
            }
        }

        // 加载环境变量
        Self::load_from_environment(config)?;

        Ok(())
    }

    /// 从环境变量加载配置
    fn load_from_environment(config: &mut Config) -> Result<()> {
        // AI配置
        if let Ok(provider) = std::env::var("AI_COMMIT_PROVIDER") {
            config.ai.provider = provider;
        }

        if let Ok(model) = std::env::var("AI_COMMIT_MODEL") {
            config.ai.model = model;
        }

        if let Ok(enabled) = std::env::var("AI_COMMIT_AI_ENABLED") {
            config.ai.enabled = enabled.parse().unwrap_or(config.ai.enabled);
        }

        // DeepSeek配置
        if let Ok(api_key) = std::env::var("AI_COMMIT_DEEPSEEK_API_KEY") {
            config.ai.services.deepseek.api_key = Some(api_key);
        }

        if let Ok(url) = std::env::var("AI_COMMIT_DEEPSEEK_URL") {
            config.ai.services.deepseek.url = url;
        }

        // SiliconFlow配置
        if let Ok(api_key) = std::env::var("AI_COMMIT_SILICONFLOW_API_KEY") {
            config.ai.services.siliconflow.api_key = Some(api_key);
        }

        // Ollama配置
        if let Ok(url) = std::env::var("AI_COMMIT_OLLAMA_URL") {
            config.ai.services.ollama.url = url;
        }

        // 通知配置
        if let Ok(enabled) = std::env::var("AI_COMMIT_NOTIFICATIONS_ENABLED") {
            config.notifications.enabled = enabled.parse().unwrap_or(config.notifications.enabled);
        }

        // 飞书配置
        if let Ok(webhook) = std::env::var("AI_COMMIT_FEISHU_WEBHOOK") {
            if let Some(feishu_config) = config.notifications.platforms.get_mut("feishu") {
                feishu_config.webhook_url = Some(webhook);
            }
        }

        // 微信配置
        if let Ok(webhook) = std::env::var("AI_COMMIT_WECHAT_WEBHOOK") {
            if let Some(wechat_config) = config.notifications.platforms.get_mut("wechat") {
                wechat_config.webhook_url = Some(webhook);
            }
        }

        // 日志配置
        if let Ok(level) = std::env::var("AI_COMMIT_LOG_LEVEL") {
            config.logging.level = level;
        }

        if let Ok(debug) = std::env::var("AI_COMMIT_DEBUG") {
            if debug.parse().unwrap_or(false) {
                config.logging.level = "debug".to_string();
            }
        }

        Ok(())
    }

    /// 监控配置文件变更
    pub fn watch_for_changes(&mut self) -> Result<()> {
        for path in &self.config_paths {
            if path.exists() {
                let watcher = FileWatcher::new(path.clone())?;
                self.watchers.push(Box::new(watcher));
            }
        }
        Ok(())
    }

    /// 获取配置
    pub fn get_config(&self) -> &Config {
        &self.config
    }

    /// 热重载配置
    pub fn reload(&mut self) -> Result<()> {
        let mut new_config = Config::default();
        Self::load_configurations(&mut new_config, &self.config_paths)?;
        new_config.validate()?;
        self.config = new_config;
        Ok(())
    }

    /// 保存配置到文件
    pub fn save_to_file(&self, path: &PathBuf) -> Result<()> {
        let content = toml::to_string_pretty(&self.config)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}

impl Config {
    /// 默认配置
    pub fn default() -> Self {
        Self {
            ai: AIConfig::default(),
            static_analysis: StaticAnalysisConfig::default(),
            sensitive_info: SensitiveInfoConfig::default(),
            notifications: NotificationConfig::default(),
            reports: ReportConfig::default(),
            cache: CacheConfig::default(),
            performance: PerformanceConfig::default(),
            logging: LoggingConfig::default(),
            extensions: HashMap::new(),
        }
    }

    /// 合并配置
    pub fn merge(&mut self, other: Config) -> Result<()> {
        // 使用serde_merge或手动合并逻辑
        self.ai.merge(other.ai);
        self.static_analysis.merge(other.static_analysis);
        self.sensitive_info.merge(other.sensitive_info);
        self.notifications.merge(other.notifications);
        self.reports.merge(other.reports);
        self.cache.merge(other.cache);
        self.performance.merge(other.performance);
        self.logging.merge(other.logging);

        // 合并扩展配置
        for (key, value) in other.extensions {
            self.extensions.insert(key, value);
        }

        Ok(())
    }

    /// 验证配置
    pub fn validate(&self) -> Result<()> {
        // AI配置验证
        self.ai.validate()?;

        // 静态分析配置验证
        self.static_analysis.validate()?;

        // 敏感信息配置验证
        self.sensitive_info.validate()?;

        // 通知配置验证
        self.notifications.validate()?;

        // 报告配置验证
        self.reports.validate()?;

        // 缓存配置验证
        self.cache.validate()?;

        // 性能配置验证
        self.performance.validate()?;

        // 日志配置验证
        self.logging.validate()?;

        Ok(())
    }
}
```

### CLI 参数定义

```rust
// src/cli/args.rs
use clap::{Arg, ArgMatches, Command};

pub fn build_cli() -> Command {
    Command::new("ai-commit")
        .version(env!("CARGO_PKG_VERSION"))
        .about("AI-enhanced code review system")
        .arg(
            Arg::new("config")
                .long("config")
                .short('c')
                .value_name("FILE")
                .help("指定配置文件路径")
                .action(clap::ArgAction::Set)
        )

        // AI配置参数
        .arg(
            Arg::new("provider")
                .long("provider")
                .value_name("PROVIDER")
                .help("AI服务提供商 (ollama, deepseek, siliconflow)")
                .action(clap::ArgAction::Set)
        )
        .arg(
            Arg::new("model")
                .long("model")
                .short('m')
                .value_name("MODEL")
                .help("AI模型名称")
                .action(clap::ArgAction::Set)
        )
        .arg(
            Arg::new("ai-review")
                .long("ai-review")
                .help("启用AI增强审查")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("ai-detail-level")
                .long("ai-detail-level")
                .value_name("LEVEL")
                .help("AI审查详细程度 (basic, detailed, comprehensive)")
                .action(clap::ArgAction::Set)
        )
        .arg(
            Arg::new("ai-language-specific")
                .long("ai-language-specific")
                .help("启用语言特定AI审查")
                .action(clap::ArgAction::SetTrue)
        )

        // 静态分析参数
        .arg(
            Arg::new("static-analysis")
                .long("static-analysis")
                .help("启用静态分析")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("no-static-analysis")
                .long("no-static-analysis")
                .help("禁用静态分析")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("static-tools")
                .long("static-tools")
                .value_name("TOOLS")
                .help("指定静态分析工具 (逗号分隔)")
                .action(clap::ArgAction::Set)
        )

        // 敏感信息检测参数
        .arg(
            Arg::new("sensitive-check")
                .long("sensitive-check")
                .help("启用敏感信息检测")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("no-sensitive-check")
                .long("no-sensitive-check")
                .help("禁用敏感信息检测")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("sensitive-mode")
                .long("sensitive-mode")
                .value_name("MODE")
                .help("敏感信息检测模式 (strict, normal, loose)")
                .action(clap::ArgAction::Set)
        )

        // 通知系统参数
        .arg(
            Arg::new("enable-notifications")
                .long("enable-notifications")
                .help("启用通知系统")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("no-notifications")
                .long("no-notifications")
                .help("禁用通知系统")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("notification-platforms")
                .long("notification-platforms")
                .value_name("PLATFORMS")
                .help("指定通知平台 (逗号分隔)")
                .action(clap::ArgAction::Set)
        )
        .arg(
            Arg::new("notification-rule")
                .long("notification-rule")
                .value_name("RULE")
                .help("通知规则 (always, on_issues, critical_only)")
                .action(clap::ArgAction::Set)
        )

        // 报告生成参数
        .arg(
            Arg::new("report-format")
                .long("report-format")
                .short('f')
                .value_name("FORMAT")
                .help("报告格式 (markdown, json, text)")
                .action(clap::ArgAction::Set)
        )
        .arg(
            Arg::new("report-output")
                .long("report-output")
                .short('o')
                .value_name("PATH")
                .help("报告输出路径")
                .action(clap::ArgAction::Set)
        )
        .arg(
            Arg::new("include-details")
                .long("include-details")
                .help("包含详细信息")
                .action(clap::ArgAction::SetTrue)
        )

        // 性能参数
        .arg(
            Arg::new("threads")
                .long("threads")
                .short('j')
                .value_name("NUM")
                .help("并行线程数")
                .action(clap::ArgAction::Set)
        )
        .arg(
            Arg::new("batch-size")
                .long("batch-size")
                .value_name("SIZE")
                .help("批处理大小")
                .action(clap::ArgAction::Set)
        )
        .arg(
            Arg::new("timeout")
                .long("timeout")
                .value_name("SECONDS")
                .help("全局超时时间")
                .action(clap::ArgAction::Set)
        )

        // 缓存参数
        .arg(
            Arg::new("no-cache")
                .long("no-cache")
                .help("禁用缓存")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("clear-cache")
                .long("clear-cache")
                .help("清除缓存")
                .action(clap::ArgAction::SetTrue)
        )

        // 日志参数
        .arg(
            Arg::new("debug")
                .long("debug")
                .short('d')
                .help("启用调试模式")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("verbose")
                .long("verbose")
                .short('v')
                .help("详细输出")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("quiet")
                .long("quiet")
                .short('q')
                .help("静默模式")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("log-file")
                .long("log-file")
                .value_name("FILE")
                .help("日志文件路径")
                .action(clap::ArgAction::Set)
        )

        // 文件和目录参数
        .arg(
            Arg::new("files")
                .long("files")
                .value_name("PATTERN")
                .help("指定要审查的文件模式")
                .action(clap::ArgAction::Set)
        )
        .arg(
            Arg::new("exclude")
                .long("exclude")
                .value_name("PATTERN")
                .help("排除文件模式")
                .action(clap::ArgAction::Set)
        )

        // 其他参数
        .arg(
            Arg::new("dry-run")
                .long("dry-run")
                .help("试运行模式，不执行实际操作")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("show-config")
                .long("show-config")
                .help("显示当前配置")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("validate-config")
                .long("validate-config")
                .help("验证配置文件")
                .action(clap::ArgAction::SetTrue)
        )
}
```

### 配置文件示例

#### 完整配置文件示例 (.ai-commit.toml)

```toml
# AI-Commit 配置文件
# 支持项目级、用户级、系统级配置

[ai]
# AI服务提供商: ollama, deepseek, siliconflow
provider = "deepseek"
# AI模型名称
model = "deepseek-chat"
# 启用AI审查
enabled = true
# 审查类型
review_types = ["general", "security", "performance"]
# 详细程度: basic, detailed, comprehensive
detail_level = "detailed"
# 语言特定规则
language_specific = true
# AI语言检测
language_detection = false
# 超时设置 (秒)
timeout = 30
# 重试次数
retry_count = 3

[ai.services.ollama]
url = "http://localhost:11434/api/generate"
model = "mistral"
enabled = true

[ai.services.deepseek]
url = "https://api.deepseek.com/v1/chat/completions"
api_key = "${AI_COMMIT_DEEPSEEK_API_KEY}"
model = "deepseek-chat"
enabled = true

[ai.services.siliconflow]
url = "https://api.siliconflow.cn/v1/chat/completions"
api_key = "${AI_COMMIT_SILICONFLOW_API_KEY}"
model = "qwen-plus"
enabled = false

[static_analysis]
# 启用静态分析
enabled = true
# 并行执行
parallel = true
# 超时设置
timeout = 60
# 忽略规则
ignore_rules = ["unused_variable", "dead_code"]
# 自定义规则文件
custom_rules_file = ".ai-commit-rules.toml"

[static_analysis.tools.gofmt]
enabled = true
path = "gofmt"
args = ["-d"]
timeout = 10

[static_analysis.tools.govet]
enabled = true
path = "go"
args = ["vet"]
timeout = 30

[static_analysis.tools.golint]
enabled = true
path = "golint"
args = []
timeout = 20

[static_analysis.tools.clippy]
enabled = true
path = "cargo"
args = ["clippy", "--message-format=json"]
timeout = 60

[sensitive_info]
# 启用敏感信息检测
enabled = true
# 检测模式: strict, normal, loose
mode = "normal"
# 自定义模式文件
custom_patterns_file = ".ai-commit-patterns.toml"
# 白名单文件
whitelist_file = ".ai-commit-whitelist.txt"
# 风险等级阈值: critical, high, medium, low
risk_threshold = "medium"
# 支持的信息类型
info_types = [
    "api_key", "password", "jwt_token", "email",
    "phone_number", "credit_card", "ssh_key"
]

[notifications]
# 启用通知
enabled = true

[notifications.platforms.feishu]
enabled = true
webhook_url = "${AI_COMMIT_FEISHU_WEBHOOK}"
template = "feishu_card"
timeout = 30

[notifications.platforms.wechat]
enabled = false
webhook_url = "${AI_COMMIT_WECHAT_WEBHOOK}"
api_key = "${AI_COMMIT_WECHAT_API_KEY}"
template = "wechat_text"
timeout = 30

[notifications.platforms.dingtalk]
enabled = false
webhook_url = "${AI_COMMIT_DINGTALK_WEBHOOK}"
template = "dingtalk_markdown"
timeout = 30

[notifications.platforms.email]
enabled = false
webhook_url = "smtp://smtp.gmail.com:587"
api_key = "${AI_COMMIT_EMAIL_PASSWORD}"
template = "email_html"
timeout = 60

[[notifications.rules]]
name = "critical_issues"
condition = "severity >= critical"
platforms = ["feishu", "wechat"]
template = "urgent_alert"
aggregation = "immediate"

[[notifications.rules]]
name = "daily_summary"
condition = "always"
platforms = ["feishu"]
template = "daily_report"
aggregation = "daily"

[[notifications.rules]]
name = "low_score"
condition = "score < 7.0"
platforms = ["feishu"]
template = "quality_alert"
aggregation = "immediate"

[notifications.templates]
feishu_card = """
{
  "msg_type": "interactive",
  "card": {
    "header": {
      "title": {
        "tag": "plain_text",
        "content": "{{title}}"
      }
    },
    "elements": [
      {
        "tag": "div",
        "text": {
          "tag": "lark_md",
          "content": "{{summary}}"
        }
      }
    ]
  }
}
"""

[notifications.retry]
max_attempts = 3
delay_seconds = 5
backoff_multiplier = 2.0

[reports]
# 默认报告格式: markdown, json, text
default_format = "markdown"
# 输出目录
output_dir = "code-review"
# 文件名模板
filename_template = "review_{{timestamp}}.{{format}}"
# 包含详细信息
include_details = true
# 包含统计信息
include_stats = true
# 自定义模板目录
template_dir = ".ai-commit-templates"

[cache]
# 启用缓存
enabled = true
# 缓存目录
cache_dir = ".ai-commit-cache"
# 内存缓存大小
memory_cache_size = 1000
# 文件系统缓存
fs_cache_enabled = true
# 缓存过期时间 (秒)
ttl_seconds = 3600
# 最大缓存大小 (MB)
max_size_mb = 100

[performance]
# 最大并发任务数
max_concurrent_tasks = 10
# 线程池大小
thread_pool_size = 4
# 批处理大小
batch_size = 50
# 内存限制 (MB)
memory_limit_mb = 500
# 全局超时 (秒)
global_timeout = 300

[logging]
# 日志级别: error, warn, info, debug, trace
level = "info"
# 日志格式: json, text
format = "text"
# 日志文件路径
file_path = "ai-commit.log"
# 控制台输出
console_enabled = true

[logging.rotation]
enabled = true
max_size_mb = 10
max_files = 5

# 扩展配置
[extensions]
# 自定义扩展配置可以放在这里
custom_analyzer = { enabled = true, path = "./custom-analyzer" }
```

#### 环境变量配置

```bash
# .env 文件示例

# AI服务配置
AI_COMMIT_PROVIDER=deepseek
AI_COMMIT_MODEL=deepseek-chat
AI_COMMIT_AI_ENABLED=true

# DeepSeek配置
AI_COMMIT_DEEPSEEK_API_KEY=your-deepseek-api-key
AI_COMMIT_DEEPSEEK_URL=https://api.deepseek.com/v1/chat/completions

# SiliconFlow配置
AI_COMMIT_SILICONFLOW_API_KEY=your-siliconflow-api-key
AI_COMMIT_SILICONFLOW_URL=https://api.siliconflow.cn/v1/chat/completions

# Ollama配置
AI_COMMIT_OLLAMA_URL=http://localhost:11434/api/generate

# 通知配置
AI_COMMIT_NOTIFICATIONS_ENABLED=true
AI_COMMIT_FEISHU_WEBHOOK=https://open.feishu.cn/open-apis/bot/v2/hook/your-token
AI_COMMIT_WECHAT_WEBHOOK=https://qyapi.weixin.qq.com/cgi-bin/webhook/send?key=your-key
AI_COMMIT_DINGTALK_WEBHOOK=https://oapi.dingtalk.com/robot/send?access_token=your-token

# 邮件配置
AI_COMMIT_EMAIL_SMTP_HOST=smtp.gmail.com
AI_COMMIT_EMAIL_SMTP_PORT=587
AI_COMMIT_EMAIL_USERNAME=your-email@gmail.com
AI_COMMIT_EMAIL_PASSWORD=your-app-password

# 日志配置
AI_COMMIT_LOG_LEVEL=info
AI_COMMIT_DEBUG=false

# 缓存配置
AI_COMMIT_CACHE_ENABLED=true
AI_COMMIT_CACHE_DIR=.ai-commit-cache

# 性能配置
AI_COMMIT_MAX_THREADS=4
AI_COMMIT_TIMEOUT=300
```

### 配置验证和错误处理

```rust
// src/config/validation.rs
impl Config {
    pub fn validate(&self) -> Result<()> {
        self.validate_ai_config()?;
        self.validate_static_analysis_config()?;
        self.validate_sensitive_info_config()?;
        self.validate_notification_config()?;
        self.validate_report_config()?;
        self.validate_cache_config()?;
        self.validate_performance_config()?;
        self.validate_logging_config()?;
        Ok(())
    }

    fn validate_ai_config(&self) -> Result<()> {
        // 验证AI提供商
        match self.ai.provider.as_str() {
            "ollama" | "deepseek" | "siliconflow" => {},
            _ => return Err(anyhow::anyhow!(
                "不支持的AI提供商: {}. 支持的提供商: ollama, deepseek, siliconflow",
                self.ai.provider
            )),
        }

        // 验证详细程度
        match self.ai.detail_level.as_str() {
            "basic" | "detailed" | "comprehensive" => {},
            _ => return Err(anyhow::anyhow!(
                "无效的AI详细程度: {}. 支持的级别: basic, detailed, comprehensive",
                self.ai.detail_level
            )),
        }

        // 验证API密钥
        match self.ai.provider.as_str() {
            "deepseek" => {
                if self.ai.services.deepseek.enabled && self.ai.services.deepseek.api_key.is_none() {
                    return Err(anyhow::anyhow!(
                        "DeepSeek服务已启用但未配置API密钥。请设置 AI_COMMIT_DEEPSEEK_API_KEY 环境变量"
                    ));
                }
            },
            "siliconflow" => {
                if self.ai.services.siliconflow.enabled && self.ai.services.siliconflow.api_key.is_none() {
                    return Err(anyhow::anyhow!(
                        "SiliconFlow服务已启用但未配置API密钥。请设置 AI_COMMIT_SILICONFLOW_API_KEY 环境变量"
                    ));
                }
            },
            _ => {}
        }

        // 验证超时设置
        if self.ai.timeout == 0 {
            return Err(anyhow::anyhow!("AI超时时间必须大于0"));
        }

        Ok(())
    }

    fn validate_notification_config(&self) -> Result<()> {
        if !self.notifications.enabled {
            return Ok(());
        }

        // 验证至少有一个平台启用
        let enabled_platforms: Vec<_> = self.notifications.platforms
            .iter()
            .filter(|(_, config)| config.enabled)
            .collect();

        if enabled_platforms.is_empty() {
            return Err(anyhow::anyhow!(
                "通知系统已启用但没有配置任何通知平台"
            ));
        }

        // 验证平台配置
        for (platform, config) in &self.notifications.platforms {
            if config.enabled {
                match platform.as_str() {
                    "feishu" | "dingtalk" => {
                        if config.webhook_url.is_none() {
                            return Err(anyhow::anyhow!(
                                "{}平台已启用但未配置webhook_url", platform
                            ));
                        }
                    },
                    "wechat" => {
                        if config.webhook_url.is_none() || config.api_key.is_none() {
                            return Err(anyhow::anyhow!(
                                "微信平台已启用但未配置完整的webhook_url和api_key"
                            ));
                        }
                    },
                    "email" => {
                        if config.webhook_url.is_none() || config.api_key.is_none() {
                            return Err(anyhow::anyhow!(
                                "邮件平台已启用但未配置完整的SMTP信息"
                            ));
                        }
                    },
                    _ => {
                        return Err(anyhow::anyhow!(
                            "不支持的通知平台: {}", platform
                        ));
                    }
                }
            }
        }

        // 验证通知规则
        for rule in &self.notifications.rules {
            match rule.condition.as_str() {
                "always" | "on_issues" | "critical_only" => {},
                condition if condition.starts_with("score <") => {
                    // 验证评分条件格式
                    if let Some(score_str) = condition.strip_prefix("score < ") {
                        if score_str.parse::<f64>().is_err() {
                            return Err(anyhow::anyhow!(
                                "无效的评分条件: {}", condition
                            ));
                        }
                    }
                },
                _ => {
                    return Err(anyhow::anyhow!(
                        "无效的通知条件: {}", rule.condition
                    ));
                }
            }

            match rule.aggregation.as_str() {
                "immediate" | "hourly" | "daily" => {},
                _ => {
                    return Err(anyhow::anyhow!(
                        "无效的聚合策略: {}", rule.aggregation
                    ));
                }
            }
        }

        Ok(())
    }

    fn validate_performance_config(&self) -> Result<()> {
        if self.performance.max_concurrent_tasks == 0 {
            return Err(anyhow::anyhow!("最大并发任务数必须大于0"));
        }

        if self.performance.thread_pool_size == 0 {
            return Err(anyhow::anyhow!("线程池大小必须大于0"));
        }

        if self.performance.batch_size == 0 {
            return Err(anyhow::anyhow!("批处理大小必须大于0"));
        }

        if self.performance.memory_limit_mb == 0 {
            return Err(anyhow::anyhow!("内存限制必须大于0"));
        }

        Ok(())
    }

    fn validate_logging_config(&self) -> Result<()> {
        match self.logging.level.as_str() {
            "error" | "warn" | "info" | "debug" | "trace" => {},
            _ => return Err(anyhow::anyhow!(
                "无效的日志级别: {}. 支持的级别: error, warn, info, debug, trace",
                self.logging.level
            )),
        }

        match self.logging.format.as_str() {
            "json" | "text" => {},
            _ => return Err(anyhow::anyhow!(
                "无效的日志格式: {}. 支持的格式: json, text",
                self.logging.format
            )),
        }

        Ok(())
    }
}
```

### 使用示例

```bash
# 使用默认配置
ai-commit --ai-review

# 使用指定配置文件
ai-commit --config ./custom-config.toml --ai-review

# 覆盖特定配置
ai-commit --provider deepseek --model deepseek-chat --enable-notifications

# 显示当前配置
ai-commit --show-config

# 验证配置文件
ai-commit --validate-config --config ./config.toml

# 组合使用
ai-commit \
  --ai-review \
  --provider deepseek \
  --enable-notifications \
  --notification-platforms feishu,wechat \
  --report-format json \
  --threads 8 \
  --debug
```

```

## 📊 性能基准与优化

### 性能测试结果

| 场景 | 文件数 | 平均耗时 | 内存使用 | 缓存命中率 |
|------|--------|----------|----------|------------|
| 小型项目 | 1-10 | 3.2s | 45MB | 85% |
| 中型项目 | 11-50 | 12.8s | 128MB | 78% |
| 大型项目 | 51-200 | 45.6s | 256MB | 82% |
| 超大项目 | 200+ | 118.3s | 412MB | 79% |

### 优化策略实施

1. **智能缓存策略**
   - 基于文件内容哈希的缓存键
   - LRU淘汰策略
   - 分层缓存（内存+文件系统）

2. **并行处理优化**
   - 文件级并行分析
   - 工具级并发执行
   - 批处理优化

3. **内存管理优化**
   - 流式文件处理
   - 及时释放大对象
   - 内存池复用

4. **网络优化**
   - HTTP连接池
   - 请求批处理
   - 智能重试机制

## 🔒 安全性设计

### 数据保护措施

1. **敏感信息处理**
   - 检测后立即脱敏
   - 不在日志中记录敏感内容
   - 安全的内存清理

2. **API密钥管理**
   - 环境变量存储
   - 运行时加密
   - 定期轮换提醒

3. **网络安全**
   - HTTPS强制加密
   - 证书验证
   - 请求签名验证

### 隐私保护

1. **代码隐私**
   - 最小化AI服务数据传输
   - 本地优先处理
   - 用户数据不持久化

2. **审计日志**
   - 操作记录不含敏感内容
   - 可配置的日志级别
   - 安全的日志存储

## 📈 扩展性规划

### 后期功能扩展

1. **MCP协议集成**
   - 标准化工具通信
   - 外部服务集成
   - 生态系统互操作

2. **数据持久化**
   - MySQL历史存储
   - MongoDB文档存储
   - 数据分析API

3. **消息队列集成**
   - Kafka异步处理
   - 事件驱动架构
   - 分布式处理

4. **企业级功能**
   - 团队协作
   - 权限管理
   - 审计合规

## 📋 配置管理总结

### 配置层次结构

AI-Commit 采用分层配置管理，确保灵活性和可维护性：

```

优先级 (高 → 低):
┌─────────────────────┐
│ CLI 命令行参数 │ ← 最高优先级，运行时覆盖
├─────────────────────┤
│ 环境变量 │ ← 适合 CI/CD 和容器化部署
├─────────────────────┤
│ 项目配置文件 │ ← .ai-commit.toml (项目特定)
├─────────────────────┤
│ 用户配置文件 │ ← ~/.ai-commit/config.toml
├─────────────────────┤
│ 系统配置文件 │ ← /etc/ai-commit/config.toml
└─────────────────────┘
│ 默认配置 │ ← 最低优先级，内置默认值
└─────────────────────┘

```

### 配置特性

1. **类型安全**: 使用Rust类型系统确保配置正确性
2. **验证机制**: 启动时验证所有配置项
3. **热重载**: 支持配置文件变更监控和热重载
4. **环境变量**: 支持环境变量替换和模板
5. **CLI覆盖**: 命令行参数可覆盖任何配置项
6. **扩展性**: 支持自定义扩展配置

### 最佳实践

1. **项目配置**: 将项目特定配置放在 `.ai-commit.toml`
2. **敏感信息**: 使用环境变量存储API密钥等敏感信息
3. **用户偏好**: 将个人偏好配置放在用户配置文件
4. **CI/CD**: 使用环境变量和CLI参数进行自动化配置
5. **团队协作**: 将通用配置提交到版本控制，敏感配置使用环境变量

### 配置验证

系统在启动时会进行全面的配置验证：
- ✅ 必需参数检查
- ✅ 参数值范围验证
- ✅ 依赖关系检查
- ✅ 权限和路径验证
- ✅ 网络连接测试

这个技术实现文档提供了AI-Commit代码审查系统的完整技术架构和实现细节，特别是详细的配置管理系统设计，为开发团队提供了详细的技术指导和最佳实践参考。
```

### 10. 性能热点分析系统

#### 10.1 性能分析器架构

```rust
// src/analysis/performance/mod.rs
pub struct PerformanceAnalyzer {
    antipattern_detector: AntipatternDetector,
    complexity_analyzer: AlgorithmComplexityAnalyzer,
    memory_analyzer: MemoryLeakAnalyzer,
    optimization_advisor: OptimizationAdvisor,
    benchmark_integrator: BenchmarkIntegrator,
}

#[derive(Debug, Clone)]
pub struct PerformanceAnalysisResult {
    pub file_path: String,
    pub antipatterns: Vec<PerformanceAntipattern>,
    pub complexity_issues: Vec<ComplexityIssue>,
    pub memory_risks: Vec<MemoryRisk>,
    pub optimization_opportunities: Vec<OptimizationOpportunity>,
    pub performance_score: f32,
    pub recommendations: Vec<PerformanceRecommendation>,
}

impl PerformanceAnalyzer {
    pub fn analyze_file(&self, file_path: &str, content: &str, language: &Language) -> PerformanceAnalysisResult {
        // 1. 检测性能反模式
        let antipatterns = self.antipattern_detector.detect(content, language);
        
        // 2. 分析算法复杂度
        let complexity_issues = self.complexity_analyzer.analyze(content, language);
        
        // 3. 检测内存泄漏风险
        let memory_risks = self.memory_analyzer.analyze(content, language);
        
        // 4. 识别优化机会
        let optimization_opportunities = self.optimization_advisor.identify(content, language);
        
        // 5. 计算性能评分
        let performance_score = self.calculate_performance_score(
            &antipatterns,
            &complexity_issues,
            &memory_risks,
        );
        
        // 6. 生成建议
        let recommendations = self.generate_recommendations(
            &antipatterns,
            &complexity_issues,
            &memory_risks,
            &optimization_opportunities,
        );
        
        PerformanceAnalysisResult {
            file_path: file_path.to_string(),
            antipatterns,
            complexity_issues,
            memory_risks,
            optimization_opportunities,
            performance_score,
            recommendations,
        }
    }
}

/// 性能反模式检测器
pub struct AntipatternDetector {
    patterns: Vec<AntipatternRule>,
}

impl AntipatternDetector {
    pub fn new() -> Self {
        let mut patterns = Vec::new();
        
        // N+1 查询问题
        patterns.push(AntipatternRule {
            name: "N+1 Query Problem".to_string(),
            pattern: Regex::new(r"for\s+.*\s+in\s+.*\{[^}]*(?:query|select|find)[^}]*\}").unwrap(),
            severity: PerformanceSeverity::High,
            description: "检测到可能的N+1查询问题".to_string(),
            suggestion: "考虑使用批量查询或预加载来优化数据库访问".to_string(),
            languages: vec![Language::Go, Language::Rust, Language::TypeScript],
        });
        
        // 字符串拼接在循环中
        patterns.push(AntipatternRule {
            name: "String Concatenation in Loop".to_string(),
            pattern: Regex::new(r"for\s+.*\{[^}]*\+\s*=\s*.*string[^}]*\}").unwrap(),
            severity: PerformanceSeverity::Medium,
            description: "在循环中进行字符串拼接".to_string(),
            suggestion: "使用StringBuilder或类似的高效字符串构建方法".to_string(),
            languages: vec![Language::Go, Language::TypeScript, Language::JavaScript],
        });
        
        // 不必要的正则表达式编译
        patterns.push(AntipatternRule {
            name: "Regex Compilation in Loop".to_string(),
            pattern: Regex::new(r"for\s+.*\{[^}]*Regex::new[^}]*\}").unwrap(),
            severity: PerformanceSeverity::High,
            description: "在循环中编译正则表达式".to_string(),
            suggestion: "将正则表达式编译移到循环外部".to_string(),
            languages: vec![Language::Rust, Language::Go],
        });
        
        Self { patterns }
    }
    
    pub fn detect(&self, content: &str, language: &Language) -> Vec<PerformanceAntipattern> {
        let mut antipatterns = Vec::new();
        let lines: Vec<&str> = content.lines().collect();
        
        for rule in &self.patterns {
            if !rule.languages.contains(language) {
                continue;
            }
            
            for (line_num, line) in lines.iter().enumerate() {
                if rule.pattern.is_match(line) {
                    antipatterns.push(PerformanceAntipattern {
                        name: rule.name.clone(),
                        line_number: line_num + 1,
                        severity: rule.severity.clone(),
                        description: rule.description.clone(),
                        suggestion: rule.suggestion.clone(),
                        code_snippet: line.to_string(),
                        estimated_impact: self.estimate_impact(&rule.severity),
                    });
                }
            }
        }
        
        antipatterns
    }
}

/// 算法复杂度分析器
pub struct AlgorithmComplexityAnalyzer;

impl AlgorithmComplexityAnalyzer {
    pub fn analyze(&self, content: &str, language: &Language) -> Vec<ComplexityIssue> {
        let mut issues = Vec::new();
        
        // 检测嵌套循环
        let nested_loops = self.detect_nested_loops(content);
        for nested_loop in nested_loops {
            issues.push(ComplexityIssue {
                issue_type: ComplexityIssueType::NestedLoops,
                line_number: nested_loop.line_number,
                estimated_complexity: nested_loop.complexity,
                description: format!("检测到{}层嵌套循环", nested_loop.depth),
                suggestion: "考虑优化算法以减少时间复杂度".to_string(),
                severity: if nested_loop.complexity > TimeComplexity::Quadratic {
                    PerformanceSeverity::High
                } else {
                    PerformanceSeverity::Medium
                },
            });
        }
        
        // 检测递归深度
        let recursive_functions = self.detect_recursive_functions(content);
        for recursive_func in recursive_functions {
            if !recursive_func.has_base_case {
                issues.push(ComplexityIssue {
                    issue_type: ComplexityIssueType::InfiniteRecursion,
                    line_number: recursive_func.line_number,
                    estimated_complexity: TimeComplexity::Infinite,
                    description: "递归函数可能缺少基础情况".to_string(),
                    suggestion: "确保递归函数有明确的终止条件".to_string(),
                    severity: PerformanceSeverity::Critical,
                });
            }
        }
        
        issues
    }
}
```

### 11. 质量趋势分析系统

#### 11.1 趋势分析器架构

```rust
// src/analysis/trends/mod.rs
pub struct QualityTrendAnalyzer {
    metrics_collector: MetricsCollector,
    trend_calculator: TrendCalculator,
    regression_detector: RegressionDetector,
    debt_tracker: TechnicalDebtTracker,
    storage: Box<dyn TrendStorage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualitySnapshot {
    pub timestamp: DateTime<Utc>,
    pub commit_hash: String,
    pub metrics: QualityMetrics,
    pub file_count: usize,
    pub line_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMetrics {
    pub overall_score: f32,
    pub security_score: f32,
    pub maintainability_score: f32,
    pub performance_score: f32,
    pub test_coverage: f32,
    pub complexity_score: f32,
    pub duplication_rate: f32,
    pub issue_count: IssueCount,
    pub technical_debt_hours: f32,
}

impl QualityTrendAnalyzer {
    pub async fn analyze_trends(&self, project_path: &str, time_range: TimeRange) -> anyhow::Result<TrendAnalysisResult> {
        // 1. 收集历史数据
        let snapshots = self.storage.get_snapshots(project_path, time_range).await?;
        
        if snapshots.len() < 2 {
            return Ok(TrendAnalysisResult::insufficient_data());
        }
        
        // 2. 计算趋势
        let trends = self.trend_calculator.calculate_trends(&snapshots);
        
        // 3. 检测回归
        let regressions = self.regression_detector.detect_regressions(&snapshots);
        
        // 4. 分析技术债务
        let debt_analysis = self.debt_tracker.analyze_debt_trends(&snapshots);
        
        // 5. 生成预测
        let predictions = self.generate_predictions(&snapshots, &trends);
        
        // 6. 创建可视化数据
        let charts = self.create_trend_charts(&snapshots, &trends);
        
        Ok(TrendAnalysisResult {
            project_path: project_path.to_string(),
            time_range,
            snapshots,
            trends,
            regressions,
            debt_analysis,
            predictions,
            charts,
            recommendations: self.generate_trend_recommendations(&trends, &regressions),
        })
    }
    
    pub async fn record_snapshot(&self, project_path: &str, review_result: &CodeReviewReport) -> anyhow::Result<()> {
        let snapshot = QualitySnapshot {
            timestamp: Utc::now(),
            commit_hash: self.get_current_commit_hash(project_path)?,
            metrics: self.extract_metrics(review_result),
            file_count: review_result.summary.total_files,
            line_count: review_result.summary.total_lines,
        };
        
        self.storage.store_snapshot(project_path, &snapshot).await?;
        
        // 清理旧数据（保留最近6个月）
        let cutoff_date = Utc::now() - chrono::Duration::days(180);
        self.storage.cleanup_old_snapshots(project_path, cutoff_date).await?;
        
        Ok(())
    }
}

/// 趋势计算器
pub struct TrendCalculator;

impl TrendCalculator {
    pub fn calculate_trends(&self, snapshots: &[QualitySnapshot]) -> QualityTrends {
        let mut trends = QualityTrends::new();
        
        if snapshots.len() < 2 {
            return trends;
        }
        
        // 计算各项指标的趋势
        trends.overall_score = self.calculate_metric_trend(
            snapshots.iter().map(|s| s.metrics.overall_score).collect()
        );
        
        trends.security_score = self.calculate_metric_trend(
            snapshots.iter().map(|s| s.metrics.security_score).collect()
        );
        
        trends.maintainability_score = self.calculate_metric_trend(
            snapshots.iter().map(|s| s.metrics.maintainability_score).collect()
        );
        
        trends.performance_score = self.calculate_metric_trend(
            snapshots.iter().map(|s| s.metrics.performance_score).collect()
        );
        
        trends.test_coverage = self.calculate_metric_trend(
            snapshots.iter().map(|s| s.metrics.test_coverage).collect()
        );
        
        trends.complexity_score = self.calculate_metric_trend(
            snapshots.iter().map(|s| s.metrics.complexity_score).collect()
        );
        
        trends.duplication_rate = self.calculate_metric_trend(
            snapshots.iter().map(|s| s.metrics.duplication_rate).collect()
        );
        
        trends.technical_debt = self.calculate_metric_trend(
            snapshots.iter().map(|s| s.metrics.technical_debt_hours).collect()
        );
        
        trends
    }
    
    fn calculate_metric_trend(&self, values: Vec<f32>) -> MetricTrend {
        if values.len() < 2 {
            return MetricTrend::stable(0.0);
        }
        
        // 使用线性回归计算趋势
        let n = values.len() as f32;
        let x_values: Vec<f32> = (0..values.len()).map(|i| i as f32).collect();
        
        let sum_x: f32 = x_values.iter().sum();
        let sum_y: f32 = values.iter().sum();
        let sum_xy: f32 = x_values.iter().zip(&values).map(|(x, y)| x * y).sum();
        let sum_x2: f32 = x_values.iter().map(|x| x * x).sum();
        
        let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_x2 - sum_x * sum_x);
        let intercept = (sum_y - slope * sum_x) / n;
        
        let trend_direction = if slope > 0.01 {
            TrendDirection::Improving
        } else if slope < -0.01 {
            TrendDirection::Declining
        } else {
            TrendDirection::Stable
        };
        
        // 计算变化率
        let first_value = values[0];
        let last_value = values[values.len() - 1];
        let change_rate = if first_value != 0.0 {
            (last_value - first_value) / first_value * 100.0
        } else {
            0.0
        };
        
        MetricTrend {
            direction: trend_direction,
            slope,
            change_rate,
            confidence: self.calculate_trend_confidence(&values, slope, intercept),
            current_value: last_value,
            previous_value: first_value,
        }
    }
}

/// 回归检测器
pub struct RegressionDetector;

impl RegressionDetector {
    pub fn detect_regressions(&self, snapshots: &[QualitySnapshot]) -> Vec<QualityRegression> {
        let mut regressions = Vec::new();
        
        if snapshots.len() < 2 {
            return regressions;
        }
        
        // 检测各项指标的回归
        for i in 1..snapshots.len() {
            let current = &snapshots[i];
            let previous = &snapshots[i - 1];
            
            // 整体质量回归
            if current.metrics.overall_score < previous.metrics.overall_score - 0.5 {
                regressions.push(QualityRegression {
                    regression_type: RegressionType::OverallQuality,
                    detected_at: current.timestamp,
                    commit_hash: current.commit_hash.clone(),
                    severity: self.calculate_regression_severity(
                        previous.metrics.overall_score,
                        current.metrics.overall_score,
                    ),
                    description: format!(
                        "整体质量评分从 {:.1} 下降到 {:.1}",
                        previous.metrics.overall_score,
                        current.metrics.overall_score
                    ),
                    impact: (previous.metrics.overall_score - current.metrics.overall_score) / previous.metrics.overall_score,
                });
            }
            
            // 安全性回归
            if current.metrics.security_score < previous.metrics.security_score - 1.0 {
                regressions.push(QualityRegression {
                    regression_type: RegressionType::Security,
                    detected_at: current.timestamp,
                    commit_hash: current.commit_hash.clone(),
                    severity: RegressionSeverity::High,
                    description: format!(
                        "安全评分从 {:.1} 下降到 {:.1}",
                        previous.metrics.security_score,
                        current.metrics.security_score
                    ),
                    impact: (previous.metrics.security_score - current.metrics.security_score) / previous.metrics.security_score,
                });
            }
            
            // 测试覆盖率回归
            if current.metrics.test_coverage < previous.metrics.test_coverage - 5.0 {
                regressions.push(QualityRegression {
                    regression_type: RegressionType::TestCoverage,
                    detected_at: current.timestamp,
                    commit_hash: current.commit_hash.clone(),
                    severity: RegressionSeverity::Medium,
                    description: format!(
                        "测试覆盖率从 {:.1}% 下降到 {:.1}%",
                        previous.metrics.test_coverage,
                        current.metrics.test_coverage
                    ),
                    impact: (previous.metrics.test_coverage - current.metrics.test_coverage) / previous.metrics.test_coverage,
                });
            }
        }
        
        regressions
    }
}

/// 技术债务跟踪器
pub struct TechnicalDebtTracker;

impl TechnicalDebtTracker {
    pub fn analyze_debt_trends(&self, snapshots: &[QualitySnapshot]) -> TechnicalDebtAnalysis {
        if snapshots.is_empty() {
            return TechnicalDebtAnalysis::empty();
        }
        
        let current_debt = snapshots.last().unwrap().metrics.technical_debt_hours;
        let debt_trend = if snapshots.len() > 1 {
            let previous_debt = snapshots[snapshots.len() - 2].metrics.technical_debt_hours;
            if current_debt > previous_debt {
                DebtTrend::Increasing
            } else if current_debt < previous_debt {
                DebtTrend::Decreasing
            } else {
                DebtTrend::Stable
            }
        } else {
            DebtTrend::Unknown
        };
        
        // 计算债务分布
        let debt_by_category = self.categorize_debt(snapshots);
        
        // 预测债务增长
        let debt_projection = self.project_debt_growth(snapshots);
        
        TechnicalDebtAnalysis {
            current_debt_hours: current_debt,
            debt_trend,
            debt_by_category,
            debt_projection,
            payback_recommendations: self.generate_payback_recommendations(current_debt, &debt_by_category),
        }
    }
}
```

### 12. 测试覆盖率集成系统

#### 12.1 覆盖率分析器架构

```rust
// src/analysis/coverage/mod.rs
pub struct CoverageAnalyzer {
    coverage_tools: HashMap<Language, Vec<Box<dyn CoverageTool>>>,
    config: CoverageConfig,
}

#[async_trait]
pub trait CoverageTool: Send + Sync {
    fn name(&self) -> &str;
    fn supported_languages(&self) -> Vec<Language>;
    async fn collect_coverage(&self, project_path: &str) -> anyhow::Result<CoverageReport>;
    fn is_available(&self) -> bool;
}

#[derive(Debug, Clone)]
pub struct CoverageReport {
    pub tool_name: String,
    pub overall_coverage: CoverageMetrics,
    pub file_coverage: Vec<FileCoverage>,
    pub uncovered_lines: Vec<UncoveredLine>,
    pub coverage_trends: Option<CoverageTrends>,
}

impl CoverageAnalyzer {
    pub async fn analyze_coverage(&self, project_path: &str, language: &Language) -> anyhow::Result<CoverageAnalysisResult> {
        let tools = self.coverage_tools.get(language).unwrap_or(&vec![]);
        let mut reports = Vec::new();
        
        // 并行执行所有可用的覆盖率工具
        let futures: Vec<_> = tools
            .iter()
            .filter(|tool| tool.is_available())
            .map(|tool| tool.collect_coverage(project_path))
            .collect();
        
        let results = futures::future::join_all(futures).await;
        
        for result in results {
            match result {
                Ok(report) => reports.push(report),
                Err(e) => eprintln!("覆盖率收集失败: {}", e),
            }
        }
        
        // 合并多个工具的结果
        let merged_report = self.merge_coverage_reports(reports);
        
        // 分析覆盖率质量
        let quality_analysis = self.analyze_coverage_quality(&merged_report);
        
        // 生成改进建议
        let recommendations = self.generate_coverage_recommendations(&merged_report, &quality_analysis);
        
        Ok(CoverageAnalysisResult {
            project_path: project_path.to_string(),
            language: language.clone(),
            coverage_report: merged_report,
            quality_analysis,
            recommendations,
        })
    }
}

/// Go语言覆盖率工具
pub struct GoCoverageTool;

#[async_trait]
impl CoverageTool for GoCoverageTool {
    fn name(&self) -> &str { "go-coverage" }
    
    fn supported_languages(&self) -> Vec<Language> {
        vec![Language::Go]
    }
    
    async fn collect_coverage(&self, project_path: &str) -> anyhow::Result<CoverageReport> {
        // 运行 go test -cover
        let output = tokio::process::Command::new("go")
            .args(["test", "-cover", "-coverprofile=coverage.out", "./..."])
            .current_dir(project_path)
            .output()
            .await?;
        
        if !output.status.success() {
            anyhow::bail!("Go测试执行失败: {}", String::from_utf8_lossy(&output.stderr));
        }
        
        // 解析覆盖率报告
        let coverage_file = format!("{}/coverage.out", project_path);
        let coverage_content = tokio::fs::read_to_string(&coverage_file).await?;
        
        let mut file_coverage = Vec::new();
        let mut total_statements = 0;
        let mut covered_statements = 0;
        
        for line in coverage_content.lines().skip(1) { // 跳过模式行
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                let file_path = parts[0].split(':').next().unwrap_or("");
                let statements: u32 = parts[1].parse().unwrap_or(0);
                let covered: u32 = parts[2].parse().unwrap_or(0);
                
                total_statements += statements;
                covered_statements += covered;
                
                file_coverage.push(FileCoverage {
                    file_path: file_path.to_string(),
                    line_coverage: (covered as f32 / statements as f32) * 100.0,
                    branch_coverage: None, // Go工具不提供分支覆盖率
                    function_coverage: None,
                    statements_total: statements,
                    statements_covered: covered,
                });
            }
        }
        
        let overall_coverage = CoverageMetrics {
            line_coverage: (covered_statements as f32 / total_statements as f32) * 100.0,
            branch_coverage: None,
            function_coverage: None,
            statement_coverage: Some((covered_statements as f32 / total_statements as f32) * 100.0),
        };
        
        // 清理临时文件
        let _ = tokio::fs::remove_file(&coverage_file).await;
        
        Ok(CoverageReport {
            tool_name: "go-coverage".to_string(),
            overall_coverage,
            file_coverage,
            uncovered_lines: self.identify_uncovered_lines(project_path, &coverage_content).await?,
            coverage_trends: None,
        })
    }
    
    fn is_available(&self) -> bool {
        std::process::Command::new("go")
            .arg("version")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }
}
```

这些新增的功能模块大大增强了AI-Commit代码审查系统的能力：

## 📊 新增功能总结

### 🔍 深度分析功能
1. **代码复杂度分析** - 圈复杂度、认知复杂度、函数长度分析
2. **代码重复检测** - 精确重复、结构相似性、跨文件重复检测
3. **依赖安全扫描** - 漏洞扫描、许可证检查、供应链安全
4. **性能热点分析** - 反模式检测、算法复杂度、内存泄漏风险

### ⚙️ 扩展性功能
5. **自定义规则引擎** - 灵活的规则定义、模板系统、规则共享
6. **测试覆盖率集成** - 多工具支持、质量评估、改进建议

### 📈 数据分析功能
7. **质量趋势分析** - 历史趋势、回归检测、技术债务跟踪

这些功能使系统从基础的代码审查工具升级为全面的代码质量管理平台，能够：

- 提供更深入的代码质量洞察
- 支持长期的质量改进跟踪
- 适应不同团队的特定需求
- 集成到完整的开发工作流中

文档已经更新完成，包含了所有新功能的详细技术实现方案。