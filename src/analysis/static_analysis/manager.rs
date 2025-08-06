use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use async_trait::async_trait;
use futures_util::future::join_all;
use tokio::sync::Semaphore;
use tracing;

use crate::languages::Language;
use super::{StaticAnalysisTool, Issue, StaticAnalysisResult};

/// 静态分析配置
#[derive(Debug, Clone)]
pub struct StaticAnalysisConfig {
    /// 最大并发工具数量
    pub max_concurrent_tools: usize,
    /// 工具执行超时时间（秒）
    pub tool_timeout_seconds: u64,
    /// 是否启用并行执行
    pub enable_parallel_execution: bool,
    /// 是否在工具失败时继续执行其他工具
    pub continue_on_tool_failure: bool,
}

impl Default for StaticAnalysisConfig {
    fn default() -> Self {
        Self {
            max_concurrent_tools: 4,
            tool_timeout_seconds: 30,
            enable_parallel_execution: true,
            continue_on_tool_failure: true,
        }
    }
}

/// 静态分析管理器
pub struct StaticAnalysisManager {
    /// 按语言分组的工具
    tools: HashMap<Language, Vec<Arc<dyn StaticAnalysisTool>>>,
    /// 所有注册的工具
    all_tools: Vec<Arc<dyn StaticAnalysisTool>>,
    /// 配置
    config: StaticAnalysisConfig,
    /// 并发控制信号量
    semaphore: Arc<Semaphore>,
}

impl StaticAnalysisManager {
    /// 创建新的静态分析管理器
    pub fn new(config: StaticAnalysisConfig) -> Self {
        let semaphore = Arc::new(Semaphore::new(config.max_concurrent_tools));

        Self {
            tools: HashMap::new(),
            all_tools: Vec::new(),
            config,
            semaphore,
        }
    }

    /// 注册静态分析工具
    pub fn register_tool(&mut self, tool: Arc<dyn StaticAnalysisTool>) {
        // 为每种支持的语言注册工具
        for language in tool.supported_languages() {
            self.tools
                .entry(language)
                .or_insert_with(Vec::new)
                .push(tool.clone());
        }

        // 添加到所有工具列表
        self.all_tools.push(tool);
    }

    /// 发现并注册可用的工具
    pub fn discover_tools(&mut self) {
        // 这里可以实现自动发现系统中可用的静态分析工具
        // 目前先留空，由具体的工具实现来注册
    }

    /// 获取支持指定语言的工具
    pub fn get_tools_for_language(&self, language: &Language) -> Vec<Arc<dyn StaticAnalysisTool>> {
        self.tools
            .get(language)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter(|tool| tool.is_available())
            .collect()
    }

    /// 获取所有可用的工具
    pub fn get_available_tools(&self) -> Vec<Arc<dyn StaticAnalysisTool>> {
        self.all_tools
            .iter()
            .filter(|tool| tool.is_available())
            .cloned()
            .collect()
    }

    /// 分析单个文件
    pub async fn analyze_file(
        &self,
        file_path: &str,
        content: &str,
        language: &Language,
    ) -> Vec<StaticAnalysisResult> {
        let tools = self.get_tools_for_language(language);

        if tools.is_empty() {
            tracing::warn!("No available tools found for language: {:?}", language);
            return Vec::new();
        }

        if self.config.enable_parallel_execution {
            self.analyze_file_parallel(file_path, content, tools).await
        } else {
            self.analyze_file_sequential(file_path, content, tools).await
        }
    }

    /// 并行分析文件
    async fn analyze_file_parallel(
        &self,
        file_path: &str,
        content: &str,
        tools: Vec<Arc<dyn StaticAnalysisTool>>,
    ) -> Vec<StaticAnalysisResult> {
        let futures = tools.into_iter().map(|tool| {
            let semaphore = self.semaphore.clone();
            let file_path = file_path.to_string();
            let content = content.to_string();
            let timeout = tokio::time::Duration::from_secs(self.config.tool_timeout_seconds);

            async move {
                let _permit = semaphore.acquire().await.unwrap();

                let start_time = Instant::now();
                let result = tokio::time::timeout(
                    timeout,
                    tool.analyze(&file_path, &content)
                ).await;

                let execution_time = start_time.elapsed();

                match result {
                    Ok(Ok(issues)) => {
                        StaticAnalysisResult::new(tool.name().to_string(), file_path)
                            .with_issues(issues)
                            .with_execution_time(execution_time)
                    }
                    Ok(Err(e)) => {
                        tracing::error!("Tool {} failed to analyze {}: {}", tool.name(), file_path, e);
                        StaticAnalysisResult::new(tool.name().to_string(), file_path)
                            .with_execution_time(execution_time)
                            .with_error(e.to_string())
                    }
                    Err(_) => {
                        tracing::error!("Tool {} timed out analyzing {}", tool.name(), file_path);
                        StaticAnalysisResult::new(tool.name().to_string(), file_path)
                            .with_execution_time(execution_time)
                            .with_error("Tool execution timed out".to_string())
                    }
                }
            }
        });

        join_all(futures).await
    }

    /// 顺序分析文件
    async fn analyze_file_sequential(
        &self,
        file_path: &str,
        content: &str,
        tools: Vec<Arc<dyn StaticAnalysisTool>>,
    ) -> Vec<StaticAnalysisResult> {
        let mut results = Vec::new();

        for tool in tools {
            let start_time = Instant::now();
            let timeout = tokio::time::Duration::from_secs(self.config.tool_timeout_seconds);

            let result = tokio::time::timeout(
                timeout,
                tool.analyze(file_path, content)
            ).await;

            let execution_time = start_time.elapsed();

            let analysis_result = match result {
                Ok(Ok(issues)) => {
                    StaticAnalysisResult::new(tool.name().to_string(), file_path.to_string())
                        .with_issues(issues)
                        .with_execution_time(execution_time)
                }
                Ok(Err(e)) => {
                    tracing::error!("Tool {} failed to analyze {}: {}", tool.name(), file_path, e);
                    let result = StaticAnalysisResult::new(tool.name().to_string(), file_path.to_string())
                        .with_execution_time(execution_time)
                        .with_error(e.to_string());

                    if !self.config.continue_on_tool_failure {
                        results.push(result);
                        break;
                    }
                    result
                }
                Err(_) => {
                    tracing::error!("Tool {} timed out analyzing {}", tool.name(), file_path);
                    StaticAnalysisResult::new(tool.name().to_string(), file_path.to_string())
                        .with_execution_time(execution_time)
                        .with_error("Tool execution timed out".to_string())
                }
            };

            results.push(analysis_result);
        }

        results
    }

    /// 分析多个文件
    pub async fn analyze_files(
        &self,
        files: &[(String, String, Language)], // (file_path, content, language)
    ) -> HashMap<String, Vec<StaticAnalysisResult>> {
        let mut results = HashMap::new();

        if self.config.enable_parallel_execution {
            // 并行分析所有文件
            let futures = files.iter().map(|(file_path, content, language)| {
                let file_path = file_path.clone();
                let content = content.clone();
                let language = language.clone();

                async move {
                    let file_results = self.analyze_file(&file_path, &content, &language).await;
                    (file_path, file_results)
                }
            });

            let all_results = join_all(futures).await;
            for (file_path, file_results) in all_results {
                results.insert(file_path, file_results);
            }
        } else {
            // 顺序分析文件
            for (file_path, content, language) in files {
                let file_results = self.analyze_file(file_path, content, language).await;
                results.insert(file_path.clone(), file_results);
            }
        }

        results
    }

    /// 获取所有问题的统计信息
    pub fn get_issue_statistics(results: &HashMap<String, Vec<StaticAnalysisResult>>) -> IssueStatistics {
        let mut stats = IssueStatistics::default();

        for file_results in results.values() {
            for result in file_results {
                if result.success {
                    stats.successful_analyses += 1;
                    stats.total_issues += result.issues.len();

                    for issue in &result.issues {
                        match issue.severity {
                            crate::analysis::static_analysis::result::Severity::Critical => stats.critical_issues += 1,
                            crate::analysis::static_analysis::result::Severity::High => stats.high_issues += 1,
                            crate::analysis::static_analysis::result::Severity::Medium => stats.medium_issues += 1,
                            crate::analysis::static_analysis::result::Severity::Low => stats.low_issues += 1,
                            crate::analysis::static_analysis::result::Severity::Info => stats.info_issues += 1,
                        }
                    }
                } else {
                    stats.failed_analyses += 1;
                }
            }
        }

        stats
    }
}

/// 问题统计信息
#[derive(Debug, Default, Clone)]
pub struct IssueStatistics {
    pub total_issues: usize,
    pub critical_issues: usize,
    pub high_issues: usize,
    pub medium_issues: usize,
    pub low_issues: usize,
    pub info_issues: usize,
    pub successful_analyses: usize,
    pub failed_analyses: usize,
}

impl IssueStatistics {
    pub fn has_critical_issues(&self) -> bool {
        self.critical_issues > 0
    }

    pub fn has_high_issues(&self) -> bool {
        self.high_issues > 0
    }

    pub fn severity_score(&self) -> f64 {
        // 计算严重程度评分 (0-100)
        let total = self.total_issues as f64;
        if total == 0.0 {
            return 100.0;
        }

        let weighted_score = (self.critical_issues as f64 * 0.0) +
                           (self.high_issues as f64 * 25.0) +
                           (self.medium_issues as f64 * 50.0) +
                           (self.low_issues as f64 * 75.0) +
                           (self.info_issues as f64 * 90.0);

        weighted_score / total
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::static_analysis::result::{Issue, Severity, IssueCategory};

    struct MockTool {
        name: String,
        languages: Vec<Language>,
        available: bool,
        issues: Vec<Issue>,
    }

    impl MockTool {
        fn new(name: &str, languages: Vec<Language>) -> Self {
            Self {
                name: name.to_string(),
                languages,
                available: true,
                issues: Vec::new(),
            }
        }

        fn with_issues(mut self, issues: Vec<Issue>) -> Self {
            self.issues = issues;
            self
        }

        fn unavailable(mut self) -> Self {
            self.available = false;
            self
        }
    }

    #[async_trait]
    impl StaticAnalysisTool for MockTool {
        fn name(&self) -> &str {
            &self.name
        }

        fn supported_languages(&self) -> Vec<Language> {
            self.languages.clone()
        }

        async fn analyze(&self, _file_path: &str, _content: &str) -> anyhow::Result<Vec<Issue>> {
            Ok(self.issues.clone())
        }

        fn is_available(&self) -> bool {
            self.available
        }
    }

    #[tokio::test]
    async fn test_manager_creation() {
        let config = StaticAnalysisConfig::default();
        let manager = StaticAnalysisManager::new(config);

        assert_eq!(manager.all_tools.len(), 0);
        assert_eq!(manager.tools.len(), 0);
    }

    #[tokio::test]
    async fn test_tool_registration() {
        let mut manager = StaticAnalysisManager::new(StaticAnalysisConfig::default());
        let tool = Arc::new(MockTool::new("test-tool", vec![Language::Go, Language::Rust]));

        manager.register_tool(tool);

        assert_eq!(manager.all_tools.len(), 1);
        assert_eq!(manager.tools.len(), 2); // Go and Rust
        assert_eq!(manager.get_tools_for_language(&Language::Go).len(), 1);
        assert_eq!(manager.get_tools_for_language(&Language::Rust).len(), 1);
        assert_eq!(manager.get_tools_for_language(&Language::TypeScript).len(), 0);
    }

    #[tokio::test]
    async fn test_unavailable_tool_filtering() {
        let mut manager = StaticAnalysisManager::new(StaticAnalysisConfig::default());
        let available_tool = Arc::new(MockTool::new("available", vec![Language::Go]));
        let unavailable_tool = Arc::new(MockTool::new("unavailable", vec![Language::Go]).unavailable());

        manager.register_tool(available_tool);
        manager.register_tool(unavailable_tool);

        assert_eq!(manager.all_tools.len(), 2);
        assert_eq!(manager.get_available_tools().len(), 1);
        assert_eq!(manager.get_tools_for_language(&Language::Go).len(), 1);
    }

    #[tokio::test]
    async fn test_file_analysis() {
        let mut manager = StaticAnalysisManager::new(StaticAnalysisConfig::default());
        let issue = Issue::new(
            "test-tool".to_string(),
            "test.go".to_string(),
            Severity::High,
            IssueCategory::Bug,
            "Test issue".to_string(),
        );
        let tool = Arc::new(MockTool::new("test-tool", vec![Language::Go]).with_issues(vec![issue]));

        manager.register_tool(tool);

        let results = manager.analyze_file("test.go", "package main", &Language::Go).await;

        assert_eq!(results.len(), 1);
        assert!(results[0].success);
        assert_eq!(results[0].issues.len(), 1);
        assert_eq!(results[0].issues[0].message, "Test issue");
    }

    #[tokio::test]
    async fn test_issue_statistics() {
        let mut results = HashMap::new();
        let mut file_results = Vec::new();

        let issues = vec![
            Issue::new("tool1".to_string(), "test.go".to_string(), Severity::Critical, IssueCategory::Bug, "Critical issue".to_string()),
            Issue::new("tool1".to_string(), "test.go".to_string(), Severity::High, IssueCategory::Security, "High issue".to_string()),
            Issue::new("tool1".to_string(), "test.go".to_string(), Severity::Medium, IssueCategory::Style, "Medium issue".to_string()),
        ];

        file_results.push(
            StaticAnalysisResult::new("tool1".to_string(), "test.go".to_string())
                .with_issues(issues)
        );

        results.insert("test.go".to_string(), file_results);

        let stats = StaticAnalysisManager::get_issue_statistics(&results);

        assert_eq!(stats.total_issues, 3);
        assert_eq!(stats.critical_issues, 1);
        assert_eq!(stats.high_issues, 1);
        assert_eq!(stats.medium_issues, 1);
        assert_eq!(stats.successful_analyses, 1);
        assert_eq!(stats.failed_analyses, 0);
        assert!(stats.has_critical_issues());
        assert!(stats.has_high_issues());
    }
}