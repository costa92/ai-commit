# AI-Commit 代码审查系统重构 - 设计文档

## 系统架构概览

AI-Commit 代码审查系统采用分层模块化架构，通过 Rust 语言实现高性能的代码分析能力。系统集成了传统静态分析工具和现代 AI 技术，提供全面的代码质量检测服务。

```
┌─────────────────────────────────────────────────────────────────┐
│                    AI-Commit 代码审查系统                          │
├─────────────────────────────────────────────────────────────────┤
│  CLI Interface Layer (src/cli/)                                │
│  - 命令行参数解析和验证                                            │
│  - 用户交互和进度显示                                              │
├─────────────────────────────────────────────────────────────────┤
│  Service Orchestration Layer (src/review/)                     │
│  - CodeReviewService - 工作流协调和结果聚合                       │
│  - ReviewOrchestrator - 审查流程编排                             │
├─────────────────┬─────────────────┬─────────────────────────────┤
│  Language       │  Static         │  Sensitive Info             │
│  Analysis       │  Analysis       │  Detection                  │
│  Layer          │  Layer          │  Layer                      │
│  (多语言支持)    │  (工具集成)      │  (安全检测)                  │
├─────────────────┼─────────────────┼─────────────────────────────┤
│  AI Services    │  Caching        │  Storage                    │
│  Layer          │  Layer          │  Middleware                 │
│  (智能分析)      │  (性能优化)      │  (数据持久化)                │
├─────────────────┼─────────────────┼─────────────────────────────┤
│  Messaging      │  Notification   │  Quality Analysis           │
│  Middleware     │  Layer          │  Layer                      │
│  (消息队列)      │  (团队协作)      │  (质量分析)                  │
├─────────────────┴─────────────────┴─────────────────────────────┤
│  Infrastructure Layer (src/infrastructure/)                    │
│  - Configuration Management                                     │
│  - Error Handling & Logging                                    │
│  - Network & Database Connections                              │
└─────────────────────────────────────────────────────────────────┘
```

## 核心组件设计

### 1. CLI 接口层 (src/cli/)

**职责**: 处理用户输入、参数解析、进度显示和结果输出

**组件结构**:
```rust
// src/cli/mod.rs
pub mod args;
pub mod progress;
pub mod output;

pub struct CliInterface {
    args_parser: ArgsParser,
    progress_reporter: ProgressReporter,
    output_formatter: OutputFormatter,
}

// src/cli/args.rs
#[derive(Parser)]
pub struct Args {
    // 基础审查参数
    #[arg(long)]
    pub code_review: bool,

    #[arg(long)]
    pub ai_review: bool,

    #[arg(long)]
    pub static_analysis: bool,

    // AI 配置参数
    #[arg(long)]
    pub ai_provider: Option<String>,

    #[arg(long)]
    pub ai_model: Option<String>,

    // 分析功能参数
    #[arg(long)]
    pub complexity_analysis: bool,

    #[arg(long)]
    pub duplication_scan: bool,

    #[arg(long)]
    pub dependency_scan: bool,

    #[arg(long)]
    pub coverage_analysis: bool,

    #[arg(long)]
    pub performance_analysis: bool,

    #[arg(long)]
    pub trend_analysis: bool,

    // 报告和输出参数
    #[arg(long)]
    pub report_format: Option<String>,

    #[arg(long)]
    pub output: Option<String>,

    // 通知参数
    #[arg(long)]
    pub enable_notifications: bool,

    #[arg(long)]
    pub notify: Option<String>,
}
```

### 2. 服务编排层 (src/review/)

**职责**: 协调各个分析组件，管理审查流程，聚合结果

**组件结构**:
```rust
// src/review/mod.rs
pub mod orchestrator;
pub mod service;
pub mod result;

// src/review/orchestrator.rs
pub struct ReviewOrchestrator {
    language_detector: LanguageDetector,
    static_analyzers: HashMap<Language, Vec<Box<dyn StaticAnalyzer>>>,
    ai_service: AIServiceManager,
    sensitive_detector: SensitiveInfoDetector,
    complexity_analyzer: ComplexityAnalyzer,
    duplication_detector: DuplicationDetector,
    dependency_analyzer: DependencyAnalyzer,
    coverage_analyzer: CoverageAnalyzer,
    performance_analyzer: PerformanceAnalyzer,
    trend_analyzer: QualityTrendAnalyzer,
    cache_manager: CacheManager,
    storage_manager: StorageManager,
    messaging_manager: MessagingManager,
    notification_service: NotificationService,
}

impl ReviewOrchestrator {
    pub async fn conduct_review(&self, request: ReviewRequest) -> anyhow::Result<ReviewResult> {
        // 1. 语言检测
        let languages = self.detect_languages(&request.files).await?;

        // 2. 并行执行各种分析
        let mut analysis_futures = Vec::new();

        if request.options.static_analysis {
            analysis_futures.push(self.run_static_analysis(&request.files, &languages));
        }

        if request.options.ai_review {
            analysis_futures.push(self.run_ai_analysis(&request.files, &languages));
        }

        if request.options.sensitive_scan {
            analysis_futures.push(self.run_sensitive_scan(&request.files));
        }

        if request.options.complexity_analysis {
            analysis_futures.push(self.run_complexity_analysis(&request.files, &languages));
        }

        if request.options.duplication_scan {
            analysis_futures.push(self.run_duplication_scan(&request.files));
        }

        if request.options.dependency_scan {
            analysis_futures.push(self.run_dependency_scan(&request.project_path));
        }

        if request.options.coverage_analysis {
            analysis_futures.push(self.run_coverage_analysis(&request.project_path, &languages));
        }

        if request.options.performance_analysis {
            analysis_futures.push(self.run_performance_analysis(&request.files, &languages));
        }

        // 3. 等待所有分析完成
        let results = futures::future::join_all(analysis_futures).await;

        // 4. 聚合结果
        let aggregated_result = self.aggregate_results(results)?;

        // 5. 生成报告
        let report = self.generate_report(&aggregated_result, &request.options)?;

        // 6. 存储报告到数据库
        let report_id = if request.options.store_report {
            Some(self.storage_manager.store_report(&report).await?)
        } else {
            None
        };

        // 7. 发送报告事件到消息队列
        if request.options.enable_messaging {
            let event = ReportEvent {
                event_id: uuid::Uuid::new_v4().to_string(),
                event_type: ReportEventType::ReportGenerated,
                timestamp: Utc::now(),
                project_path: request.project_path.clone(),
                report_id: report_id.clone(),
                metadata: EventMetadata {
                    source: "ai-commit".to_string(),
                    version: env!("CARGO_PKG_VERSION").to_string(),
                    correlation_id: request.correlation_id.clone(),
                    user_id: request.user_id.clone(),
                    tags: request.tags.clone(),
                },
                payload: serde_json::to_value(&report)?,
            };

            self.messaging_manager.send_report_event(event).await?;
        }

        // 8. 发送通知
        if request.options.enable_notifications {
            self.send_notifications(&report).await?;
        }

        // 9. 记录质量快照
        if request.options.record_snapshot {
            self.record_quality_snapshot(&request.project_path, &report).await?;
        }

        Ok(ReviewResult {
            report,
            metadata: ReviewMetadata {
                duration: start_time.elapsed(),
                files_analyzed: request.files.len(),
                languages_detected: languages,
            },
        })
    }
}
```

### 3. 语言分析层 (src/languages/)

**职责**: 多语言支持、语言特定分析、AI 增强语言检测

**组件结构**:
```rust
// src/languages/mod.rs
pub mod detector;
pub mod go;
pub mod rust;
pub mod typescript;
pub mod generic;

// src/languages/detector.rs
pub struct LanguageDetector {
    ai_detector: Option<AILanguageDetector>,
    heuristic_detector: HeuristicDetector,
    cache: HashMap<String, LanguageDetectionResult>,
}

impl LanguageDetector {
    pub async fn detect_language(&mut self, file_path: &str, content: &str) -> LanguageDetectionResult {
        // 1. 快速路径：基于文件扩展名
        if let Some(lang) = Language::from_extension(file_path) {
            return LanguageDetectionResult::new(lang, 0.95, "extension-based");
        }

        // 2. AI 增强检测
        if let Some(ai_detector) = &self.ai_detector {
            if let Ok(result) = ai_detector.detect(file_path, content).await {
                return result;
            }
        }

        // 3. 启发式检测
        self.heuristic_detector.detect(file_path, content)
    }
}

// src/languages/go/mod.rs
pub mod analyzer;
pub mod patterns;

pub struct GoAnalyzer {
    patterns: Vec<GoPattern>,
}

impl LanguageAnalyzer for GoAnalyzer {
    fn analyze_features(&self, content: &str) -> Vec<LanguageFeature> {
        let mut features = Vec::new();

        // Go 包声明检测
        if let Some(package) = self.extract_package_declaration(content) {
            features.push(LanguageFeature::Package(package));
        }

        // Go 函数检测
        features.extend(self.extract_functions(content));

        // Go 结构体检测
        features.extend(self.extract_structs(content));

        // Go 接口检测
        features.extend(self.extract_interfaces(content));

        features
    }
}
```

### 4. 静态分析层 (src/analysis/static/)

**职责**: 集成多种静态分析工具，提供统一的分析接口

**组件结构**:
```rust
// src/analysis/static/mod.rs
pub mod manager;
pub mod tools;
pub mod result;

// src/analysis/static/manager.rs
pub struct StaticAnalysisManager {
    tools: HashMap<Language, Vec<Box<dyn StaticAnalysisTool>>>,
    config: StaticAnalysisConfig,
}

#[async_trait]
pub trait StaticAnalysisTool: Send + Sync {
    fn name(&self) -> &str;
    fn supported_languages(&self) -> Vec<Language>;
    async fn analyze(&self, file_path: &str, content: &str) -> anyhow::Result<Vec<Issue>>;
    fn is_available(&self) -> bool;
}

// src/analysis/static/tools/go.rs
pub struct GoFmtTool;

#[async_trait]
impl StaticAnalysisTool for GoFmtTool {
    fn name(&self) -> &str { "gofmt" }

    fn supported_languages(&self) -> Vec<Language> {
        vec![Language::Go]
    }

    async fn analyze(&self, file_path: &str, _content: &str) -> anyhow::Result<Vec<Issue>> {
        let output = tokio::process::Command::new("gofmt")
            .args(["-d", file_path])
            .output()
            .await?;

        let mut issues = Vec::new();
        if !output.stdout.is_empty() {
            issues.push(Issue {
                tool: "gofmt".to_string(),
                file_path: file_path.to_string(),
                line_number: None,
                severity: Severity::Style,
                message: "代码格式不符合 Go 标准格式".to_string(),
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
```

### 5. AI 服务层 (src/ai/)

**职责**: AI 服务集成、智能代码分析、质量评分

**组件结构**:
```rust
// src/ai/mod.rs
pub mod manager;
pub mod providers;
pub mod reviewers;

// src/ai/manager.rs
pub struct AIServiceManager {
    providers: HashMap<String, Box<dyn AIProvider>>,
    config: AIConfig,
    client: Arc<reqwest::Client>,
}

#[async_trait]
pub trait AIProvider: Send + Sync {
    fn name(&self) -> &str;
    async fn analyze_code(&self, prompt: &str, config: &AIConfig) -> anyhow::Result<String>;
    fn is_available(&self, config: &AIConfig) -> bool;
}

// src/ai/providers/deepseek.rs
pub struct DeepSeekProvider {
    client: Arc<reqwest::Client>,
}

#[async_trait]
impl AIProvider for DeepSeekProvider {
    fn name(&self) -> &str { "deepseek" }

    async fn analyze_code(&self, prompt: &str, config: &AIConfig) -> anyhow::Result<String> {
        let request = serde_json::json!({
            "model": config.model,
            "messages": [{
                "role": "user",
                "content": prompt
            }],
            "temperature": config.temperature,
            "max_tokens": config.max_tokens
        });

        let response = self.client
            .post(&config.deepseek_url)
            .header("Authorization", format!("Bearer {}", config.deepseek_api_key.as_ref().unwrap()))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!("DeepSeek API 请求失败: {}", response.status());
        }

        let response_json: serde_json::Value = response.json().await?;
        let content = response_json["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();

        Ok(content)
    }

    fn is_available(&self, config: &AIConfig) -> bool {
        config.deepseek_api_key.is_some() && !config.deepseek_api_key.as_ref().unwrap().is_empty()
    }
}

// src/ai/reviewers/language_specific.rs
pub struct LanguageSpecificReviewer {
    ai_service: AIServiceManager,
}

impl LanguageSpecificReviewer {
    pub async fn review_go_code(&self, features: &[LanguageFeature], file_path: &str) -> anyhow::Result<AIReviewResult> {
        let prompt = self.build_go_review_prompt(features, file_path);
        let response = self.ai_service.analyze_code(&prompt).await?;
        self.parse_go_review_response(&response)
    }

    pub async fn review_rust_code(&self, features: &[LanguageFeature], file_path: &str) -> anyhow::Result<AIReviewResult> {
        let prompt = self.build_rust_review_prompt(features, file_path);
        let response = self.ai_service.analyze_code(&prompt).await?;
        self.parse_rust_review_response(&response)
    }
}
```

### 6. 敏感信息检测层 (src/analysis/sensitive/)

**职责**: 敏感信息模式匹配、风险评估、脱敏处理

**组件结构**:
```rust
// src/analysis/sensitive/mod.rs
pub mod detector;
pub mod patterns;
pub mod result;

// src/analysis/sensitive/detector.rs
pub struct SensitiveInfoDetector {
    patterns: Vec<SensitivePattern>,
    whitelist: HashSet<String>,
    custom_patterns: Vec<SensitivePattern>,
}

impl SensitiveInfoDetector {
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
}

// src/analysis/sensitive/patterns.rs
pub struct SensitivePattern {
    pub name: String,
    pub info_type: SensitiveInfoType,
    pub regex: Regex,
    pub confidence: f32,
    pub risk_level: RiskLevel,
    pub description: String,
    pub recommendations: Vec<String>,
}

impl SensitivePattern {
    pub fn aws_access_key() -> Self {
        Self {
            name: "AWS Access Key".to_string(),
            info_type: SensitiveInfoType::ApiKey,
            regex: Regex::new(r"AKIA[0-9A-Z]{16}").unwrap(),
            confidence: 0.95,
            risk_level: RiskLevel::Critical,
            description: "AWS 访问密钥".to_string(),
            recommendations: vec![
                "立即轮换暴露的 AWS 密钥".to_string(),
                "使用 AWS IAM 角色替代硬编码密钥".to_string(),
                "启用 AWS CloudTrail 监控密钥使用".to_string(),
            ],
        }
    }
}
```

### 7. 复杂度分析层 (src/analysis/complexity/)

**职责**: 代码复杂度计算、热点识别、重构建议

**组件结构**:
```rust
// src/analysis/complexity/mod.rs
pub mod analyzer;
pub mod metrics;
pub mod calculator;

// src/analysis/complexity/analyzer.rs
pub struct ComplexityAnalyzer {
    cyclomatic_calculator: CyclomaticComplexityCalculator,
    cognitive_calculator: CognitiveComplexityCalculator,
    function_analyzer: FunctionLengthAnalyzer,
    nesting_analyzer: NestingDepthAnalyzer,
    config: ComplexityConfig,
}

impl ComplexityAnalyzer {
    pub fn analyze_file(&self, file_path: &str, content: &str, language: &Language) -> ComplexityResult {
        let ast = self.parse_ast(content, language);
        let functions = self.extract_functions(&ast);

        let mut function_complexities = Vec::new();
        for function in functions {
            let cyclomatic = self.cyclomatic_calculator.calculate(&function);
            let cognitive = self.cognitive_calculator.calculate(&function);
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
}
```

### 8. 缓存管理层 (src/cache/)

**职责**: 多级缓存、性能优化、缓存策略

**组件结构**:
```rust
// src/cache/mod.rs
pub mod manager;
pub mod storage;
pub mod strategy;

// src/cache/manager.rs
pub struct CacheManager {
    memory_cache: Arc<Mutex<LruCache<String, CacheEntry>>>,
    fs_cache: Option<FsCacheManager>,
    config: CacheConfig,
}

impl CacheManager {
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
                self.set_memory_cache(key, data.clone()).await;
                return Some(data);
            }
        }

        None
    }

    pub async fn set<T>(&self, key: &str, data: T, ttl: Option<Duration>)
    where
        T: Clone + serde::Serialize,
    {
        let expires_at = ttl.map(|duration| Utc::now() + chrono::Duration::from_std(duration).unwrap());

        let entry = CacheEntry {
            data: self.wrap_data(data.clone()),
            created_at: Utc::now(),
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
}
```

### 9. 存储中间件层 (src/storage/)

**职责**: 报告数据持久化、多数据库支持、数据访问抽象

**组件结构**:
```rust
// src/storage/mod.rs
pub mod manager;
pub mod providers;
pub mod models;

// src/storage/manager.rs
pub struct StorageManager {
    providers: HashMap<StorageType, Box<dyn StorageProvider>>,
    config: StorageConfig,
    connection_pool: Arc<ConnectionPool>,
}

#[async_trait]
pub trait StorageProvider: Send + Sync {
    fn storage_type(&self) -> StorageType;
    async fn store_report(&mut self, report: &CodeReviewReport) -> anyhow::Result<String>;
    async fn retrieve_report(&self, report_id: &str) -> anyhow::Result<Option<CodeReviewReport>>;
    async fn list_reports(&self, filter: &ReportFilter) -> anyhow::Result<Vec<ReportSummary>>;
    async fn delete_report(&mut self, report_id: &str) -> anyhow::Result<()>;
    fn is_available(&self) -> bool;
}

// src/storage/providers/mongodb.rs
pub struct MongoDBProvider {
    client: mongodb::Client,
    database: mongodb::Database,
    collection: mongodb::Collection<Document>,
}

#[async_trait]
impl StorageProvider for MongoDBProvider {
    fn storage_type(&self) -> StorageType { StorageType::MongoDB }

    async fn store_report(&mut self, report: &CodeReviewReport) -> anyhow::Result<String> {
        let document = self.serialize_report(report)?;
        let result = self.collection.insert_one(document, None).await?;

        Ok(result.inserted_id.as_object_id()
            .unwrap()
            .to_hex())
    }

    async fn retrieve_report(&self, report_id: &str) -> anyhow::Result<Option<CodeReviewReport>> {
        let object_id = mongodb::bson::oid::ObjectId::parse_str(report_id)?;
        let filter = doc! { "_id": object_id };

        if let Some(document) = self.collection.find_one(filter, None).await? {
            let report = self.deserialize_report(&document)?;
            Ok(Some(report))
        } else {
            Ok(None)
        }
    }

    async fn list_reports(&self, filter: &ReportFilter) -> anyhow::Result<Vec<ReportSummary>> {
        let mongo_filter = self.build_mongo_filter(filter);
        let mut cursor = self.collection.find(mongo_filter, None).await?;

        let mut summaries = Vec::new();
        while let Some(document) = cursor.try_next().await? {
            let summary = self.extract_summary(&document)?;
            summaries.push(summary);
        }

        Ok(summaries)
    }
}

// src/storage/providers/mysql.rs
pub struct MySQLProvider {
    pool: sqlx::MySqlPool,
    table_name: String,
}

#[async_trait]
impl StorageProvider for MySQLProvider {
    fn storage_type(&self) -> StorageType { StorageType::MySQL }

    async fn store_report(&mut self, report: &CodeReviewReport) -> anyhow::Result<String> {
        let report_json = serde_json::to_string(report)?;
        let report_id = uuid::Uuid::new_v4().to_string();

        sqlx::query(&format!(
            "INSERT INTO {} (id, project_path, report_data, created_at, overall_score) VALUES (?, ?, ?, NOW(), ?)",
            self.table_name
        ))
        .bind(&report_id)
        .bind(&report.summary.project_path)
        .bind(&report_json)
        .bind(report.overall_score)
        .execute(&self.pool)
        .await?;

        Ok(report_id)
    }

    async fn retrieve_report(&self, report_id: &str) -> anyhow::Result<Option<CodeReviewReport>> {
        let row: Option<(String,)> = sqlx::query_as(&format!(
            "SELECT report_data FROM {} WHERE id = ?",
            self.table_name
        ))
        .bind(report_id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some((report_json,)) = row {
            let report: CodeReviewReport = serde_json::from_str(&report_json)?;
            Ok(Some(report))
        } else {
            Ok(None)
        }
    }
}
```

### 10. 消息队列中间件层 (src/messaging/)

**职责**: 异步报告处理、消息队列集成、事件驱动架构

**组件结构**:
```rust
// src/messaging/mod.rs
pub mod manager;
pub mod producers;
pub mod consumers;
pub mod events;

// src/messaging/manager.rs
pub struct MessagingManager {
    producers: HashMap<QueueType, Box<dyn MessageProducer>>,
    consumers: HashMap<QueueType, Box<dyn MessageConsumer>>,
    config: MessagingConfig,
}

#[async_trait]
pub trait MessageProducer: Send + Sync {
    fn queue_type(&self) -> QueueType;
    async fn send_message(&mut self, topic: &str, message: &[u8]) -> anyhow::Result<MessageId>;
    async fn send_report_event(&mut self, event: ReportEvent) -> anyhow::Result<MessageId>;
    fn is_available(&self) -> bool;
}

#[async_trait]
pub trait MessageConsumer: Send + Sync {
    fn queue_type(&self) -> QueueType;
    async fn consume_messages<F>(&mut self, topic: &str, handler: F) -> anyhow::Result<()>
    where
        F: Fn(Message) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>> + Send + Sync + 'static;
    async fn subscribe_to_events<F>(&mut self, handler: F) -> anyhow::Result<()>
    where
        F: Fn(ReportEvent) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>> + Send + Sync + 'static;
}

// src/messaging/producers/kafka.rs
pub struct KafkaProducer {
    producer: rdkafka::producer::FutureProducer,
    config: KafkaConfig,
}

#[async_trait]
impl MessageProducer for KafkaProducer {
    fn queue_type(&self) -> QueueType { QueueType::Kafka }

    async fn send_message(&mut self, topic: &str, message: &[u8]) -> anyhow::Result<MessageId> {
        let record = FutureRecord::to(topic)
            .payload(message)
            .key(&format!("report-{}", uuid::Uuid::new_v4()));

        let delivery_status = self.producer.send(record, Duration::from_secs(10)).await;

        match delivery_status {
            Ok((partition, offset)) => {
                Ok(MessageId::Kafka {
                    topic: topic.to_string(),
                    partition,
                    offset
                })
            },
            Err((kafka_error, _)) => {
                anyhow::bail!("Failed to send message to Kafka: {}", kafka_error)
            }
        }
    }

    async fn send_report_event(&mut self, event: ReportEvent) -> anyhow::Result<MessageId> {
        let message = serde_json::to_vec(&event)?;
        let topic = match event.event_type {
            ReportEventType::ReportGenerated => &self.config.report_generated_topic,
            ReportEventType::AnalysisCompleted => &self.config.analysis_completed_topic,
            ReportEventType::QualityThresholdExceeded => &self.config.quality_alert_topic,
        };

        self.send_message(topic, &message).await
    }
}

// src/messaging/events.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportEvent {
    pub event_id: String,
    pub event_type: ReportEventType,
    pub timestamp: DateTime<Utc>,
    pub project_path: String,
    pub report_id: Option<String>,
    pub metadata: EventMetadata,
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReportEventType {
    ReportGenerated,
    AnalysisCompleted,
    QualityThresholdExceeded,
    SecurityIssueDetected,
    PerformanceRegressionDetected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMetadata {
    pub source: String,
    pub version: String,
    pub correlation_id: Option<String>,
    pub user_id: Option<String>,
    pub tags: HashMap<String, String>,
}

// src/messaging/consumers/kafka.rs
pub struct KafkaConsumer {
    consumer: rdkafka::consumer::StreamConsumer,
    config: KafkaConfig,
}

#[async_trait]
impl MessageConsumer for KafkaConsumer {
    fn queue_type(&self) -> QueueType { QueueType::Kafka }

    async fn consume_messages<F>(&mut self, topic: &str, handler: F) -> anyhow::Result<()>
    where
        F: Fn(Message) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>> + Send + Sync + 'static,
    {
        self.consumer.subscribe(&[topic])?;

        loop {
            match self.consumer.recv().await {
                Ok(message) => {
                    let msg = Message {
                        topic: message.topic().to_string(),
                        partition: message.partition(),
                        offset: message.offset(),
                        key: message.key().map(|k| k.to_vec()),
                        payload: message.payload().unwrap_or(&[]).to_vec(),
                        timestamp: message.timestamp().to_millis(),
                    };

                    if let Err(e) = handler(msg).await {
                        log::error!("Error processing message: {}", e);
                    }

                    self.consumer.commit_message(&message, CommitMode::Async)?;
                },
                Err(e) => {
                    log::error!("Error receiving message: {}", e);
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
            }
        }
    }
}
```

### 11. 通知系统层 (src/notification/)

**职责**: 多平台通知、消息模板、重试机制

**组件结构**:
```rust
// src/notification/mod.rs
pub mod service;
pub mod providers;
pub mod templates;

// src/notification/service.rs
pub struct NotificationService {
    providers: HashMap<NotificationPlatform, Box<dyn NotificationProvider>>,
    config: NotificationConfig,
    retry_policy: RetryPolicy,
}

#[async_trait]
pub trait NotificationProvider: Send + Sync {
    fn platform(&self) -> NotificationPlatform;
    async fn send_notification(&mut self, message: &NotificationMessage) -> anyhow::Result<NotificationResult>;
    fn is_configured(&self) -> bool;
}

// src/notification/providers/feishu.rs
pub struct FeishuProvider {
    webhook_url: String,
    client: reqwest::Client,
    template: FeishuTemplate,
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
                    }
                ]
            }
        })
    }
}
```

### 10. 配置管理层 (src/config/)

**职责**: 多层次配置、验证、热重载

**组件结构**:
```rust
// src/config/mod.rs
pub mod manager;
pub mod validation;
pub mod loader;

// src/config/manager.rs
pub struct ConfigManager {
    config: Config,
    config_paths: Vec<PathBuf>,
    watchers: Vec<Box<dyn ConfigWatcher>>,
}

impl ConfigManager {
    pub fn new() -> anyhow::Result<Self> {
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

    pub fn update_from_cli(&mut self, matches: &ArgMatches) -> anyhow::Result<()> {
        // 从 CLI 参数更新配置
        if let Some(provider) = matches.get_one::<String>("ai-provider") {
            self.config.ai.provider = provider.clone();
        }

        if let Some(model) = matches.get_one::<String>("ai-model") {
            self.config.ai.model = model.clone();
        }

        // 重新验证配置
        self.config.validate()?;

        Ok(())
    }
}
```

## 数据模型设计

### 核心数据结构

```rust
// src/models/mod.rs
pub mod review;
pub mod language;
pub mod issue;
pub mod result;

// src/models/review.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewRequest {
    pub project_path: String,
    pub files: Vec<String>,
    pub options: ReviewOptions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewOptions {
    pub static_analysis: bool,
    pub ai_review: bool,
    pub sensitive_scan: bool,
    pub complexity_analysis: bool,
    pub duplication_scan: bool,
    pub dependency_scan: bool,
    pub coverage_analysis: bool,
    pub performance_analysis: bool,
    pub trend_analysis: bool,
    pub enable_notifications: bool,
    pub record_snapshot: bool,
    pub report_format: ReportFormat,
    pub output_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewResult {
    pub report: CodeReviewReport,
    pub metadata: ReviewMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeReviewReport {
    pub summary: ReviewSummary,
    pub static_analysis_results: Vec<StaticAnalysisResult>,
    pub ai_review_results: Vec<AIReviewResult>,
    pub sensitive_info_results: Vec<SensitiveInfoResult>,
    pub complexity_results: Vec<ComplexityResult>,
    pub duplication_results: Vec<DuplicationResult>,
    pub dependency_results: Option<DependencyAnalysisResult>,
    pub coverage_results: Option<CoverageAnalysisResult>,
    pub performance_results: Vec<PerformanceAnalysisResult>,
    pub trend_results: Option<TrendAnalysisResult>,
    pub overall_score: f32,
    pub recommendations: Vec<String>,
}
```

## 错误处理策略

### 错误类型定义

```rust
// src/error/mod.rs
#[derive(Debug, thiserror::Error)]
pub enum ReviewError {
    #[error("配置错误: {0}")]
    ConfigError(String),

    #[error("语言检测失败: {0}")]
    LanguageDetectionError(String),

    #[error("静态分析失败: {tool} - {message}")]
    StaticAnalysisError { tool: String, message: String },

    #[error("AI 服务错误: {provider} - {message}")]
    AIServiceError { provider: String, message: String },

    #[error("缓存错误: {0}")]
    CacheError(String),

    #[error("通知发送失败: {platform} - {message}")]
    NotificationError { platform: String, message: String },

    #[error("IO 错误: {0}")]
    IoError(#[from] std::io::Error),

    #[error("网络错误: {0}")]
    NetworkError(#[from] reqwest::Error),

    #[error("序列化错误: {0}")]
    SerializationError(#[from] serde_json::Error),
}

pub type ReviewResult<T> = Result<T, ReviewError>;
```

### 错误恢复机制

```rust
// src/error/recovery.rs
pub struct ErrorRecoveryManager {
    retry_policies: HashMap<String, RetryPolicy>,
    fallback_strategies: HashMap<String, Box<dyn FallbackStrategy>>,
}

impl ErrorRecoveryManager {
    pub async fn handle_error<T>(&self, error: ReviewError, context: &str) -> ReviewResult<Option<T>> {
        match error {
            ReviewError::AIServiceError { provider, .. } => {
                // AI 服务错误：尝试其他提供商或降级到传统分析
                self.handle_ai_service_error(&provider).await
            },
            ReviewError::StaticAnalysisError { tool, .. } => {
                // 静态分析工具错误：跳过该工具，继续其他分析
                self.handle_static_analysis_error(&tool).await
            },
            ReviewError::NotificationError { platform, .. } => {
                // 通知错误：尝试其他平台或记录失败
                self.handle_notification_error(&platform).await
            },
            _ => {
                // 其他错误：记录日志并返回
                log::error!("Unhandled error in {}: {}", context, error);
                Err(error)
            }
        }
    }
}
```

## 测试策略

### 单元测试

```rust
// tests/unit/language_detector_test.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_go_language_detection() {
        let detector = LanguageDetector::new();
        let go_code = r#"
            package main

            import "fmt"

            func main() {
                fmt.Println("Hello, World!")
            }
        "#;

        let result = detector.detect_language("main.go", go_code).await;
        assert_eq!(result.detected_language, Language::Go);
        assert!(result.confidence > 0.9);
    }

    #[tokio::test]
    async fn test_rust_language_detection() {
        let detector = LanguageDetector::new();
        let rust_code = r#"
            fn main() {
                println!("Hello, World!");
            }
        "#;

        let result = detector.detect_language("main.rs", rust_code).await;
        assert_eq!(result.detected_language, Language::Rust);
        assert!(result.confidence > 0.9);
    }
}
```

### 集成测试

```rust
// tests/integration/review_orchestrator_test.rs
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_full_review_workflow() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path().to_str().unwrap();

        // 创建测试文件
        let go_file = temp_dir.path().join("main.go");
        std::fs::write(&go_file, r#"
            package main

            import "fmt"

            func main() {
                fmt.Println("Hello, World!")
            }
        "#).unwrap();

        let orchestrator = ReviewOrchestrator::new_for_test();
        let request = ReviewRequest {
            project_path: project_path.to_string(),
            files: vec![go_file.to_str().unwrap().to_string()],
            options: ReviewOptions {
                static_analysis: true,
                ai_review: false, // 测试中禁用 AI
                sensitive_scan: true,
                complexity_analysis: true,
                duplication_scan: false,
                dependency_scan: false,
                coverage_analysis: false,
                performance_analysis: false,
                trend_analysis: false,
                enable_notifications: false,
                record_snapshot: false,
                report_format: ReportFormat::Json,
                output_path: None,
            },
        };

        let result = orchestrator.conduct_review(request).await.unwrap();

        assert!(!result.report.static_analysis_results.is_empty());
        assert!(result.report.overall_score > 0.0);
        assert_eq!(result.metadata.files_analyzed, 1);
    }
}
```

## 性能优化设计

### 并行处理架构

```rust
// src/performance/parallel.rs
pub struct ParallelProcessor {
    semaphore: Arc<Semaphore>,
    thread_pool: Arc<ThreadPool>,
    config: ParallelConfig,
}

impl ParallelProcessor {
    pub async fn process_files_parallel<F, R>(&self, files: Vec<String>, processor: F) -> Vec<anyhow::Result<R>>
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

        let results = futures::future::join_all(tasks).await;
        results.into_iter()
            .map(|result| result.unwrap_or_else(|e| Err(anyhow::anyhow!("Task failed: {}", e))))
            .collect()
    }
}
```

### 内存管理优化

```rust
// src/performance/memory.rs
pub struct MemoryManager {
    memory_limit: usize,
    current_usage: Arc<AtomicUsize>,
    cleanup_threshold: usize,
}

impl MemoryManager {
    pub fn check_memory_usage(&self) -> bool {
        let current = self.current_usage.load(Ordering::Relaxed);
        current < self.memory_limit
    }

    pub async fn cleanup_if_needed(&self) {
        let current = self.current_usage.load(Ordering::Relaxed);
        if current > self.cleanup_threshold {
            self.perform_cleanup().await;
        }
    }

    async fn perform_cleanup(&self) {
        // 清理缓存
        // 释放不必要的内存
        // 触发垃圾回收
    }
}
```

## 安全性设计

### 敏感信息保护

```rust
// src/security/protection.rs
pub struct SensitiveDataProtector {
    encryption_key: [u8; 32],
    masking_rules: Vec<MaskingRule>,
}

impl SensitiveDataProtector {
    pub fn mask_sensitive_data(&self, data: &str, info_type: &SensitiveInfoType) -> String {
        match info_type {
            SensitiveInfoType::ApiKey => self.mask_api_key(data),
            SensitiveInfoType::Password => self.mask_password(data),
            SensitiveInfoType::Email => self.mask_email(data),
            _ => self.generic_mask(data),
        }
    }

    pub fn secure_log(&self, message: &str) -> String {
        let mut secure_message = message.to_string();
        for rule in &self.masking_rules {
            secure_message = rule.apply(&secure_message);
        }
        secure_message
    }
}
```

### 网络安全

```rust
// src/security/network.rs
pub struct SecureHttpClient {
    client: reqwest::Client,
    certificate_validator: CertificateValidator,
}

impl SecureHttpClient {
    pub fn new() -> anyhow::Result<Self> {
        let client = reqwest::Client::builder()
            .use_rustls_tls()
            .https_only(true)
            .timeout(Duration::from_secs(30))
            .build()?;

        Ok(Self {
            client,
            certificate_validator: CertificateValidator::new(),
        })
    }

    pub async fn secure_request(&self, request: RequestBuilder) -> anyhow::Result<Response> {
        let response = request.send().await?;

        // 验证响应安全性
        self.validate_response(&response)?;

        Ok(response)
    }
}
```

这个设计文档提供了 AI-Commit 代码审查系统重构的完整架构设计，包括：

1. **分层模块化架构** - 清晰的职责分离和组件边界
2. **核心组件设计** - 详细的接口定义和实现策略
3. **数据模型设计** - 完整的数据结构和序列化支持
4. **错误处理策略** - 全面的错误类型和恢复机制
5. **测试策略** - 单元测试和集成测试框架
6. **性能优化设计** - 并行处理和内存管理
7. **安全性设计** - 敏感信息保护和网络安全

该设计支持所有 16 个功能需求，提供了可扩展、高性能、安全的代码审查系统架构。