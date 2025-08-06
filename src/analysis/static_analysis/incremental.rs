use std::collections::HashMap;
use std::path::Path;
use std::time::SystemTime;
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use crate::analysis::static_analysis::{StaticAnalysisResult, StaticAnalysisManager};
use crate::languages::Language;

/// 文件变更检测器
pub struct FileChangeDetector {
    /// 文件哈希缓存
    file_hashes: HashMap<String, String>,
    /// 文件修改时间缓存
    file_timestamps: HashMap<String, SystemTime>,
}

impl FileChangeDetector {
    pub fn new() -> Self {
        Self {
            file_hashes: HashMap::new(),
            file_timestamps: HashMap::new(),
        }
    }

    /// 检测文件是否发生变更
    pub fn has_file_changed(&mut self, file_path: &str, content: &str) -> anyhow::Result<bool> {
        let current_hash = self.calculate_file_hash(content);

        // 检查哈希是否变更
        if let Some(cached_hash) = self.file_hashes.get(file_path) {
            if cached_hash == &current_hash {
                return Ok(false);
            }
        }

        // 更新缓存
        self.file_hashes.insert(file_path.to_string(), current_hash);

        // 更新时间戳
        if let Ok(metadata) = std::fs::metadata(file_path) {
            if let Ok(modified) = metadata.modified() {
                self.file_timestamps.insert(file_path.to_string(), modified);
            }
        }

        Ok(true)
    }

    /// 批量检测文件变更
    pub fn detect_changed_files(&mut self, files: &[(String, String)]) -> anyhow::Result<Vec<String>> {
        let mut changed_files = Vec::new();

        for (file_path, content) in files {
            if self.has_file_changed(file_path, content)? {
                changed_files.push(file_path.clone());
            }
        }

        Ok(changed_files)
    }

    /// 计算文件内容哈希
    fn calculate_file_hash(&self, content: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// 获取文件的最后修改时间
    pub fn get_file_timestamp(&self, file_path: &str) -> Option<SystemTime> {
        self.file_timestamps.get(file_path).copied()
    }

    /// 清除缓存
    pub fn clear_cache(&mut self) {
        self.file_hashes.clear();
        self.file_timestamps.clear();
    }
}

/// Git diff 分析器
pub struct GitDiffAnalyzer;

impl GitDiffAnalyzer {
    pub fn new() -> Self {
        Self
    }

    /// 获取 Git diff 中的变更文件
    pub async fn get_changed_files_from_git(&self, base_ref: Option<&str>) -> anyhow::Result<Vec<String>> {
        let base = base_ref.unwrap_or("HEAD");

        let output = tokio::process::Command::new("git")
            .args(["diff", "--name-only", base])
            .output()
            .await?;

        if !output.status.success() {
            anyhow::bail!("Git diff command failed: {}", String::from_utf8_lossy(&output.stderr));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let changed_files: Vec<String> = stdout
            .lines()
            .filter(|line| !line.is_empty())
            .map(|line| line.to_string())
            .collect();

        Ok(changed_files)
    }

    /// 获取 Git diff 中的具体变更内容
    pub async fn get_file_diff(&self, file_path: &str, base_ref: Option<&str>) -> anyhow::Result<String> {
        let base = base_ref.unwrap_or("HEAD");

        let output = tokio::process::Command::new("git")
            .args(["diff", base, "--", file_path])
            .output()
            .await?;

        if !output.status.success() {
            anyhow::bail!("Git diff command failed for file {}: {}", file_path, String::from_utf8_lossy(&output.stderr));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// 解析 diff 获取变更的行号
    pub fn parse_changed_lines(&self, diff_content: &str) -> Vec<(usize, usize)> {
        let mut changed_ranges = Vec::new();

        for line in diff_content.lines() {
            if line.starts_with("@@") {
                // 解析 @@ -old_start,old_count +new_start,new_count @@ 格式
                if let Some(range_part) = line.split("@@").nth(1) {
                    if let Some(new_part) = range_part.split('+').nth(1) {
                        if let Some(range_str) = new_part.split(' ').next() {
                            if let Some((start_str, count_str)) = range_str.split_once(',') {
                                if let (Ok(start), Ok(count)) = (start_str.parse::<usize>(), count_str.parse::<usize>()) {
                                    changed_ranges.push((start, start + count - 1));
                                }
                            } else if let Ok(start) = range_str.parse::<usize>() {
                                changed_ranges.push((start, start));
                            }
                        }
                    }
                }
            }
        }

        changed_ranges
    }
}

/// 分析结果缓存
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisCache {
    /// 缓存的分析结果
    results: HashMap<String, CachedAnalysisResult>,
    /// 缓存创建时间
    created_at: SystemTime,
    /// 缓存版本
    version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedAnalysisResult {
    /// 文件哈希
    file_hash: String,
    /// 分析结果
    results: Vec<StaticAnalysisResult>,
    /// 缓存时间
    cached_at: SystemTime,
    /// 语言类型
    language: Language,
}

impl AnalysisCache {
    pub fn new() -> Self {
        Self {
            results: HashMap::new(),
            created_at: SystemTime::now(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }

    /// 获取缓存的分析结果
    pub fn get_cached_result(&self, file_path: &str, file_hash: &str) -> Option<&Vec<StaticAnalysisResult>> {
        if let Some(cached) = self.results.get(file_path) {
            if cached.file_hash == file_hash {
                return Some(&cached.results);
            }
        }
        None
    }

    /// 缓存分析结果
    pub fn cache_result(&mut self, file_path: &str, file_hash: String, results: Vec<StaticAnalysisResult>, language: Language) {
        let cached_result = CachedAnalysisResult {
            file_hash,
            results,
            cached_at: SystemTime::now(),
            language,
        };

        self.results.insert(file_path.to_string(), cached_result);
    }

    /// 清除过期缓存
    pub fn cleanup_expired(&mut self, max_age: std::time::Duration) {
        let now = SystemTime::now();
        self.results.retain(|_, cached| {
            if let Ok(age) = now.duration_since(cached.cached_at) {
                age <= max_age
            } else {
                false
            }
        });
    }

    /// 获取缓存统计信息
    pub fn get_cache_stats(&self) -> CacheStats {
        CacheStats {
            total_entries: self.results.len(),
            cache_age: SystemTime::now().duration_since(self.created_at).unwrap_or_default(),
            version: self.version.clone(),
        }
    }

    /// 保存缓存到文件
    pub fn save_to_file(&self, cache_file: &str) -> anyhow::Result<()> {
        let cache_data = serde_json::to_string_pretty(self)?;
        std::fs::write(cache_file, cache_data)?;
        Ok(())
    }

    /// 从文件加载缓存
    pub fn load_from_file(cache_file: &str) -> anyhow::Result<Self> {
        if !Path::new(cache_file).exists() {
            return Ok(Self::new());
        }

        let cache_data = std::fs::read_to_string(cache_file)?;
        let cache: Self = serde_json::from_str(&cache_data)?;

        // 检查版本兼容性
        if cache.version != env!("CARGO_PKG_VERSION") {
            tracing::warn!("Cache version mismatch, creating new cache");
            return Ok(Self::new());
        }

        Ok(cache)
    }
}

#[derive(Debug, Clone)]
pub struct CacheStats {
    pub total_entries: usize,
    pub cache_age: std::time::Duration,
    pub version: String,
}

/// 增量静态分析管理器
pub struct IncrementalAnalysisManager {
    /// 静态分析管理器
    analysis_manager: StaticAnalysisManager,
    /// 文件变更检测器
    change_detector: FileChangeDetector,
    /// Git diff 分析器
    git_analyzer: GitDiffAnalyzer,
    /// 分析结果缓存
    cache: AnalysisCache,
    /// 缓存文件路径
    cache_file_path: String,
}

impl IncrementalAnalysisManager {
    pub fn new(analysis_manager: StaticAnalysisManager, cache_file_path: String) -> anyhow::Result<Self> {
        let cache = AnalysisCache::load_from_file(&cache_file_path)?;

        Ok(Self {
            analysis_manager,
            change_detector: FileChangeDetector::new(),
            git_analyzer: GitDiffAnalyzer::new(),
            cache,
            cache_file_path,
        })
    }

    /// 执行增量分析
    pub async fn analyze_incremental(
        &mut self,
        files: &[(String, String, Language)], // (file_path, content, language)
        use_git_diff: bool,
    ) -> anyhow::Result<HashMap<String, Vec<StaticAnalysisResult>>> {
        let mut results = HashMap::new();
        let mut files_to_analyze = Vec::new();

        // 确定需要分析的文件
        if use_git_diff {
            // 使用 Git diff 确定变更文件
            let changed_files = self.git_analyzer.get_changed_files_from_git(None).await?;

            for (file_path, content, language) in files {
                if changed_files.iter().any(|changed| changed == file_path || file_path.ends_with(changed)) {
                    files_to_analyze.push((file_path.clone(), content.clone(), language.clone()));
                } else {
                    // 尝试从缓存获取结果
                    let file_hash = self.calculate_file_hash(content);
                    if let Some(cached_results) = self.cache.get_cached_result(file_path, &file_hash) {
                        results.insert(file_path.clone(), cached_results.clone());
                        tracing::debug!("Using cached results for unchanged file: {}", file_path);
                    } else {
                        // 缓存未命中，需要分析
                        files_to_analyze.push((file_path.clone(), content.clone(), language.clone()));
                    }
                }
            }
        } else {
            // 使用文件变更检测
            for (file_path, content, language) in files {
                if self.change_detector.has_file_changed(file_path, content)? {
                    files_to_analyze.push((file_path.clone(), content.clone(), language.clone()));
                } else {
                    // 尝试从缓存获取结果
                    let file_hash = self.calculate_file_hash(content);
                    if let Some(cached_results) = self.cache.get_cached_result(file_path, &file_hash) {
                        results.insert(file_path.clone(), cached_results.clone());
                        tracing::debug!("Using cached results for unchanged file: {}", file_path);
                    } else {
                        // 缓存未命中，需要分析
                        files_to_analyze.push((file_path.clone(), content.clone(), language.clone()));
                    }
                }
            }
        }

        // 分析需要更新的文件
        if !files_to_analyze.is_empty() {
            tracing::info!("Analyzing {} changed files", files_to_analyze.len());
            let analysis_results = self.analysis_manager.analyze_files(&files_to_analyze).await;

            // 更新缓存并合并结果
            for (file_path, file_results) in analysis_results {
                // 缓存结果
                if let Some((_, content, language)) = files_to_analyze.iter().find(|(path, _, _)| path == &file_path) {
                    let file_hash = self.calculate_file_hash(content);
                    self.cache.cache_result(&file_path, file_hash, file_results.clone(), language.clone());
                }

                results.insert(file_path, file_results);
            }
        }

        // 保存缓存
        self.save_cache()?;

        Ok(results)
    }

    /// 获取缓存统计信息
    pub fn get_cache_stats(&self) -> CacheStats {
        self.cache.get_cache_stats()
    }

    /// 清理过期缓存
    pub fn cleanup_cache(&mut self, max_age: std::time::Duration) -> anyhow::Result<()> {
        self.cache.cleanup_expired(max_age);
        self.save_cache()
    }

    /// 清除所有缓存
    pub fn clear_cache(&mut self) -> anyhow::Result<()> {
        self.cache = AnalysisCache::new();
        self.change_detector.clear_cache();
        self.save_cache()
    }

    /// 计算文件哈希
    fn calculate_file_hash(&self, content: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// 保存缓存到文件
    fn save_cache(&self) -> anyhow::Result<()> {
        self.cache.save_to_file(&self.cache_file_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::static_analysis::{StaticAnalysisConfig, Issue, Severity, IssueCategory};

    #[test]
    fn test_file_change_detector() {
        let mut detector = FileChangeDetector::new();

        // 第一次检测应该返回 true（文件是新的）
        assert!(detector.has_file_changed("test.rs", "fn main() {}").unwrap());

        // 相同内容应该返回 false
        assert!(!detector.has_file_changed("test.rs", "fn main() {}").unwrap());

        // 不同内容应该返回 true
        assert!(detector.has_file_changed("test.rs", "fn main() { println!(\"hello\"); }").unwrap());
    }

    #[test]
    fn test_git_diff_line_parsing() {
        let analyzer = GitDiffAnalyzer::new();
        let diff_content = r#"
@@ -10,5 +10,8 @@ fn main() {
     println!("hello");
+    println!("world");
+    let x = 5;
 }
"#;

        let changed_lines = analyzer.parse_changed_lines(diff_content);
        assert_eq!(changed_lines.len(), 1);
        assert_eq!(changed_lines[0], (10, 17)); // 从第10行开始，共8行
    }

    #[test]
    fn test_analysis_cache() {
        let mut cache = AnalysisCache::new();

        // 创建测试结果
        let issue = Issue::new(
            "test".to_string(),
            "test.rs".to_string(),
            Severity::Low,
            IssueCategory::Style,
            "Test issue".to_string(),
        );
        let result = crate::analysis::static_analysis::StaticAnalysisResult::new("test".to_string(), "test.rs".to_string())
            .with_issues(vec![issue]);

        let file_hash = "test_hash".to_string();

        // 缓存结果
        cache.cache_result("test.rs", file_hash.clone(), vec![result], Language::Rust);

        // 获取缓存结果
        let cached = cache.get_cached_result("test.rs", &file_hash);
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().len(), 1);

        // 不同哈希应该返回 None
        let cached_wrong_hash = cache.get_cached_result("test.rs", "wrong_hash");
        assert!(cached_wrong_hash.is_none());
    }

    #[test]
    fn test_cache_cleanup() {
        let mut cache = AnalysisCache::new();

        // 添加一些测试数据
        cache.cache_result("test1.rs", "hash1".to_string(), vec![], Language::Rust);
        cache.cache_result("test2.rs", "hash2".to_string(), vec![], Language::Rust);

        assert_eq!(cache.results.len(), 2);

        // 清理过期缓存（使用很短的时间，所有缓存都应该被清理）
        cache.cleanup_expired(std::time::Duration::from_nanos(1));
        assert_eq!(cache.results.len(), 0);
    }

    #[tokio::test]
    async fn test_incremental_analysis_manager_creation() {
        let config = StaticAnalysisConfig::default();
        let analysis_manager = StaticAnalysisManager::new(config);
        let cache_file = "/tmp/test_cache.json".to_string();

        let result = IncrementalAnalysisManager::new(analysis_manager, cache_file);
        assert!(result.is_ok());
    }
}