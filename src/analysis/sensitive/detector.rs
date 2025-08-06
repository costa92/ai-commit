use std::collections::{HashMap, HashSet};
use std::time::Instant;

use super::patterns::{PatternEngine, SensitivePattern};
use super::result::{SensitiveInfoResult, SensitiveItem, SensitiveSummary};

/// 敏感信息检测器配置
#[derive(Debug, Clone)]
pub struct SensitiveDetectorConfig {
    pub max_file_size: usize,
    pub enable_whitelist: bool,
    pub enable_custom_patterns: bool,
    pub confidence_threshold: f32,
    pub max_matches_per_file: usize,
}

impl Default for SensitiveDetectorConfig {
    fn default() -> Self {
        Self {
            max_file_size: 10 * 1024 * 1024, // 10MB
            enable_whitelist: true,
            enable_custom_patterns: true,
            confidence_threshold: 0.5,
            max_matches_per_file: 1000,
        }
    }
}

/// 白名单条目
#[derive(Debug, Clone)]
pub struct WhitelistEntry {
    pub pattern: String,
    pub file_pattern: Option<String>,
    pub reason: String,
    pub enabled: bool,
}

impl WhitelistEntry {
    pub fn new(pattern: String, reason: String) -> Self {
        Self {
            pattern,
            file_pattern: None,
            reason,
            enabled: true,
        }
    }

    pub fn with_file_pattern(mut self, file_pattern: String) -> Self {
        self.file_pattern = Some(file_pattern);
        self
    }

    pub fn matches(&self, text: &str, file_path: &str) -> bool {
        if !self.enabled {
            return false;
        }

        // 检查文件路径匹配
        if let Some(ref file_pat) = self.file_pattern {
            if !file_path.contains(file_pat) {
                return false;
            }
        }

        // 检查文本匹配
        text.contains(&self.pattern)
    }
}

/// 敏感信息检测器
pub struct SensitiveInfoDetector {
    pattern_engine: PatternEngine,
    whitelist: Vec<WhitelistEntry>,
    config: SensitiveDetectorConfig,
    statistics: DetectionStatistics,
}

impl SensitiveInfoDetector {
    pub fn new(config: SensitiveDetectorConfig) -> Self {
        Self {
            pattern_engine: PatternEngine::new(),
            whitelist: Vec::new(),
            config,
            statistics: DetectionStatistics::new(),
        }
    }

    pub fn with_patterns(mut self, patterns: Vec<SensitivePattern>) -> anyhow::Result<Self> {
        self.pattern_engine.add_patterns(patterns)?;
        Ok(self)
    }

    pub fn with_whitelist(mut self, whitelist: Vec<WhitelistEntry>) -> Self {
        self.whitelist = whitelist;
        self
    }

    pub fn add_pattern(&mut self, pattern: SensitivePattern) -> anyhow::Result<()> {
        self.pattern_engine.add_pattern(pattern)
    }

    pub fn add_whitelist_entry(&mut self, entry: WhitelistEntry) {
        self.whitelist.push(entry);
    }

    pub fn detect(&mut self, file_path: &str, content: &str) -> anyhow::Result<SensitiveInfoResult> {
        let start_time = Instant::now();
        self.statistics.files_scanned += 1;

        // 检查文件大小限制
        if content.len() > self.config.max_file_size {
            self.statistics.files_skipped += 1;
            return Ok(SensitiveInfoResult::new(file_path.to_string())
                .with_duration(start_time.elapsed()));
        }

        // 执行模式匹配
        let pattern_matches = self.pattern_engine.match_text(content);
        let mut sensitive_items = Vec::new();
        let mut patterns_used = HashSet::new();

        for pattern_match in pattern_matches {
            if sensitive_items.len() >= self.config.max_matches_per_file {
                break;
            }

            // 获取模式信息
            if let Some(pattern) = self.pattern_engine.get_pattern(&pattern_match.pattern_name) {
                // 检查置信度阈值
                if pattern.confidence < self.config.confidence_threshold {
                    continue;
                }

                // 检查白名单
                if self.config.enable_whitelist && self.is_whitelisted(&pattern_match.matched_text, file_path) {
                    self.statistics.whitelisted_matches += 1;
                    continue;
                }

                // 计算行列位置
                let (line_number, column_start, column_end) = pattern_match.get_line_column(content);

                // 创建敏感信息项
                let sensitive_item = SensitiveItem::new(
                    pattern.info_type.clone(),
                    line_number,
                    column_start,
                    column_end,
                    pattern_match.matched_text.clone(),
                    pattern.confidence,
                    pattern.risk_level.clone(),
                    pattern.name.clone(),
                ).with_recommendations(pattern.recommendations.clone());

                sensitive_items.push(sensitive_item);
                patterns_used.insert(pattern.name.clone());
                self.statistics.total_matches += 1;
            }
        }

        let duration = start_time.elapsed();
        self.statistics.total_scan_time += duration;

        Ok(SensitiveInfoResult::new(file_path.to_string())
            .with_items(sensitive_items)
            .with_duration(duration)
            .with_patterns(patterns_used.into_iter().collect()))
    }

    pub fn detect_batch(&mut self, files: &[(String, String)]) -> anyhow::Result<Vec<SensitiveInfoResult>> {
        let mut results = Vec::new();

        for (file_path, content) in files {
            match self.detect(file_path, content) {
                Ok(result) => results.push(result),
                Err(e) => {
                    log::warn!("Failed to scan file {}: {}", file_path, e);
                    self.statistics.scan_errors += 1;
                }
            }
        }

        Ok(results)
    }

    pub fn aggregate_results(&self, results: &[SensitiveInfoResult]) -> AggregatedSensitiveResult {
        let mut total_items = 0;
        let mut critical_items = 0;
        let mut high_items = 0;
        let mut medium_items = 0;
        let mut low_items = 0;
        let mut all_types = HashSet::new();
        let mut file_results = HashMap::new();
        let mut total_risk_score = 0.0;

        for result in results {
            total_items += result.items.len();
            critical_items += result.summary.critical_items;
            high_items += result.summary.high_items;
            medium_items += result.summary.medium_items;
            low_items += result.summary.low_items;
            total_risk_score += result.summary.risk_score;

            for info_type in &result.summary.types_detected {
                all_types.insert(info_type.clone());
            }

            file_results.insert(result.file_path.clone(), result.clone());
        }

        let average_risk_score = if results.is_empty() { 0.0 } else { total_risk_score / results.len() as f32 };

        AggregatedSensitiveResult {
            total_files_scanned: results.len(),
            total_items,
            critical_items,
            high_items,
            medium_items,
            low_items,
            types_detected: all_types.into_iter().collect(),
            average_risk_score,
            file_results,
            scan_statistics: self.statistics.clone(),
        }
    }

    fn is_whitelisted(&self, text: &str, file_path: &str) -> bool {
        self.whitelist.iter().any(|entry| entry.matches(text, file_path))
    }

    pub fn get_statistics(&self) -> &DetectionStatistics {
        &self.statistics
    }

    pub fn reset_statistics(&mut self) {
        self.statistics = DetectionStatistics::new();
    }

    pub fn get_pattern_count(&self) -> usize {
        self.pattern_engine.get_pattern_count()
    }

    pub fn get_enabled_pattern_count(&self) -> usize {
        self.pattern_engine.get_enabled_pattern_count()
    }

    pub fn enable_pattern(&mut self, name: &str) -> bool {
        self.pattern_engine.enable_pattern(name)
    }

    pub fn disable_pattern(&mut self, name: &str) -> bool {
        self.pattern_engine.disable_pattern(name)
    }
}

impl Default for SensitiveInfoDetector {
    fn default() -> Self {
        Self::new(SensitiveDetectorConfig::default())
    }
}

/// 检测统计信息
#[derive(Debug, Clone)]
pub struct DetectionStatistics {
    pub files_scanned: usize,
    pub files_skipped: usize,
    pub total_matches: usize,
    pub whitelisted_matches: usize,
    pub scan_errors: usize,
    pub total_scan_time: std::time::Duration,
}

impl DetectionStatistics {
    pub fn new() -> Self {
        Self {
            files_scanned: 0,
            files_skipped: 0,
            total_matches: 0,
            whitelisted_matches: 0,
            scan_errors: 0,
            total_scan_time: std::time::Duration::from_secs(0),
        }
    }

    pub fn average_scan_time(&self) -> std::time::Duration {
        if self.files_scanned == 0 {
            std::time::Duration::from_secs(0)
        } else {
            self.total_scan_time / self.files_scanned as u32
        }
    }
}

impl Default for DetectionStatistics {
    fn default() -> Self {
        Self::new()
    }
}

/// 聚合的敏感信息检测结果
#[derive(Debug, Clone)]
pub struct AggregatedSensitiveResult {
    pub total_files_scanned: usize,
    pub total_items: usize,
    pub critical_items: usize,
    pub high_items: usize,
    pub medium_items: usize,
    pub low_items: usize,
    pub types_detected: Vec<super::result::SensitiveInfoType>,
    pub average_risk_score: f32,
    pub file_results: HashMap<String, SensitiveInfoResult>,
    pub scan_statistics: DetectionStatistics,
}

impl AggregatedSensitiveResult {
    pub fn has_critical_issues(&self) -> bool {
        self.critical_items > 0
    }

    pub fn has_high_risk_issues(&self) -> bool {
        self.critical_items > 0 || self.high_items > 0
    }

    pub fn get_risk_level(&self) -> super::result::RiskLevel {
        if self.critical_items > 0 {
            super::result::RiskLevel::Critical
        } else if self.high_items > 0 {
            super::result::RiskLevel::High
        } else if self.medium_items > 0 {
            super::result::RiskLevel::Medium
        } else {
            super::result::RiskLevel::Low
        }
    }

    pub fn get_files_with_issues(&self) -> Vec<&String> {
        self.file_results.iter()
            .filter(|(_, result)| !result.items.is_empty())
            .map(|(path, _)| path)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::sensitive::result::{SensitiveInfoType, RiskLevel};

    fn create_test_pattern() -> SensitivePattern {
        SensitivePattern::new(
            "test_api_key".to_string(),
            SensitiveInfoType::ApiKey,
            r"api_key_[a-zA-Z0-9]{16}".to_string(),
            0.9,
            RiskLevel::High,
            "Test API key pattern".to_string(),
        ).unwrap()
    }

    #[test]
    fn test_detector_creation() {
        let config = SensitiveDetectorConfig::default();
        let detector = SensitiveInfoDetector::new(config);

        assert_eq!(detector.get_pattern_count(), 0);
        assert_eq!(detector.get_enabled_pattern_count(), 0);
    }

    #[test]
    fn test_detector_with_patterns() {
        let pattern = create_test_pattern();
        let detector = SensitiveInfoDetector::default()
            .with_patterns(vec![pattern])
            .unwrap();

        assert_eq!(detector.get_pattern_count(), 1);
        assert_eq!(detector.get_enabled_pattern_count(), 1);
    }

    #[test]
    fn test_detection() {
        let pattern = create_test_pattern();
        let mut detector = SensitiveInfoDetector::default()
            .with_patterns(vec![pattern])
            .unwrap();

        let test_content = "const key = 'api_key_1234567890abcdef';";
        let result = detector.detect("test.js", test_content).unwrap();

        assert_eq!(result.items.len(), 1);
        assert_eq!(result.items[0].info_type, SensitiveInfoType::ApiKey);
        assert_eq!(result.items[0].matched_text, "api_key_1234567890abcdef");
        assert_eq!(result.items[0].line_number, 1);
    }

    #[test]
    fn test_whitelist_filtering() {
        let pattern = create_test_pattern();
        let whitelist_entry = WhitelistEntry::new(
            "api_key_1234567890abcdef".to_string(),
            "Test key for development".to_string(),
        );

        let mut detector = SensitiveInfoDetector::default()
            .with_patterns(vec![pattern])
            .unwrap()
            .with_whitelist(vec![whitelist_entry]);

        let test_content = "const key = 'api_key_1234567890abcdef';";
        let result = detector.detect("test.js", test_content).unwrap();

        assert_eq!(result.items.len(), 0);
        assert_eq!(detector.get_statistics().whitelisted_matches, 1);
    }

    #[test]
    fn test_batch_detection() {
        let pattern = create_test_pattern();
        let mut detector = SensitiveInfoDetector::default()
            .with_patterns(vec![pattern])
            .unwrap();

        let files = vec![
            ("file1.js".to_string(), "const key1 = 'api_key_1111111111111111';".to_string()),
            ("file2.js".to_string(), "const key2 = 'api_key_2222222222222222';".to_string()),
            ("file3.js".to_string(), "const normal = 'just a string';".to_string()),
        ];

        let results = detector.detect_batch(&files).unwrap();
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].items.len(), 1);
        assert_eq!(results[1].items.len(), 1);
        assert_eq!(results[2].items.len(), 0);
    }

    #[test]
    fn test_aggregated_results() {
        let pattern = create_test_pattern();
        let mut detector = SensitiveInfoDetector::default()
            .with_patterns(vec![pattern])
            .unwrap();

        let files = vec![
            ("file1.js".to_string(), "const key1 = 'api_key_1111111111111111';".to_string()),
            ("file2.js".to_string(), "const key2 = 'api_key_2222222222222222';".to_string()),
        ];

        let results = detector.detect_batch(&files).unwrap();
        let aggregated = detector.aggregate_results(&results);

        assert_eq!(aggregated.total_files_scanned, 2);
        assert_eq!(aggregated.total_items, 2);
        assert_eq!(aggregated.high_items, 2);
        assert!(aggregated.has_high_risk_issues());
        assert_eq!(aggregated.get_risk_level(), RiskLevel::High);
    }
}