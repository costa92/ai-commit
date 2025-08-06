use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

use super::{Language, LanguageFeature};

/// 语言检测结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageDetectionResult {
    pub language: Language,
    pub confidence: f32,
    pub detection_method: String,
    pub features: Vec<LanguageFeature>,
    pub detection_time_ms: u64,
    pub file_size: usize,
}

impl LanguageDetectionResult {
    pub fn new(language: Language, confidence: f32, detection_method: &str) -> Self {
        Self {
            language,
            confidence,
            detection_method: detection_method.to_string(),
            features: Vec::new(),
            detection_time_ms: 0,
            file_size: 0,
        }
    }

    pub fn with_features(mut self, features: Vec<LanguageFeature>) -> Self {
        self.features = features;
        self
    }

    pub fn with_timing(mut self, detection_time_ms: u64) -> Self {
        self.detection_time_ms = detection_time_ms;
        self
    }

    pub fn with_file_size(mut self, file_size: usize) -> Self {
        self.file_size = file_size;
        self
    }
}

/// 缓存条目
#[derive(Debug, Clone)]
struct CacheEntry {
    result: LanguageDetectionResult,
    created_at: Instant,
    access_count: u32,
}

impl CacheEntry {
    fn new(result: LanguageDetectionResult) -> Self {
        Self {
            result,
            created_at: Instant::now(),
            access_count: 1,
        }
    }

    fn is_expired(&self, ttl: Duration) -> bool {
        self.created_at.elapsed() > ttl
    }

    fn access(&mut self) -> LanguageDetectionResult {
        self.access_count += 1;
        self.result.clone()
    }
}

/// 检测统计信息
#[derive(Debug, Default)]
pub struct DetectionStats {
    pub total_detections: u64,
    pub cache_hits: u64,
    pub extension_based: u64,
    pub ai_enhanced: u64,
    pub heuristic: u64,
    pub fallback: u64,
    pub average_detection_time_ms: f64,
}

impl DetectionStats {
    pub fn cache_hit_rate(&self) -> f64 {
        if self.total_detections == 0 {
            0.0
        } else {
            self.cache_hits as f64 / self.total_detections as f64
        }
    }
}

/// 语言检测器
pub struct LanguageDetector {
    ai_detector: Option<AILanguageDetector>,
    heuristic_detector: HeuristicDetector,
    cache: HashMap<String, CacheEntry>,
    cache_ttl: Duration,
    max_cache_size: usize,
    stats: DetectionStats,
}

impl LanguageDetector {
    pub fn new() -> Self {
        Self {
            ai_detector: None,
            heuristic_detector: HeuristicDetector::new(),
            cache: HashMap::new(),
            cache_ttl: Duration::from_secs(300), // 5 minutes default TTL
            max_cache_size: 1000,
            stats: DetectionStats::default(),
        }
    }

    pub fn with_ai_detector(mut self, ai_detector: AILanguageDetector) -> Self {
        self.ai_detector = Some(ai_detector);
        self
    }

    pub fn with_cache_config(mut self, ttl: Duration, max_size: usize) -> Self {
        self.cache_ttl = ttl;
        self.max_cache_size = max_size;
        self
    }

    pub async fn detect_language(&mut self, file_path: &str, content: &str) -> LanguageDetectionResult {
        let start_time = Instant::now();
        let file_size = content.len();

        self.stats.total_detections += 1;

        // 检查缓存
        let cache_key = self.generate_cache_key(file_path, content);
        if let Some(cached_result) = self.get_from_cache(&cache_key) {
            self.stats.cache_hits += 1;
            return cached_result.with_timing(start_time.elapsed().as_millis() as u64);
        }

        // 1. 快速路径：基于文件扩展名
        if let Some(lang) = Language::from_extension(file_path) {
            self.stats.extension_based += 1;
            let result = LanguageDetectionResult::new(lang, 0.95, "extension-based")
                .with_timing(start_time.elapsed().as_millis() as u64)
                .with_file_size(file_size);
            self.set_cache(&cache_key, result.clone());
            return result;
        }

        // 2. AI 增强检测
        if let Some(ai_detector) = &self.ai_detector {
            if let Ok(result) = ai_detector.detect(file_path, content).await {
                self.stats.ai_enhanced += 1;
                let result = result
                    .with_timing(start_time.elapsed().as_millis() as u64)
                    .with_file_size(file_size);
                self.set_cache(&cache_key, result.clone());
                return result;
            }
        }

        // 3. 启发式检测
        let result = self.heuristic_detector.detect(file_path, content);
        if result.language != Language::Unknown {
            self.stats.heuristic += 1;
        } else {
            self.stats.fallback += 1;
        }

        let result = result
            .with_timing(start_time.elapsed().as_millis() as u64)
            .with_file_size(file_size);
        self.set_cache(&cache_key, result.clone());

        // 更新平均检测时间
        self.update_average_detection_time(start_time.elapsed().as_millis() as u64);

        result
    }

    pub fn detect_languages_batch(&mut self, files: &[(String, String)]) -> Vec<(String, LanguageDetectionResult)> {
        let mut results = Vec::new();

        for (file_path, content) in files {
            // 对于批量处理，我们只使用同步方法
            let result = self.detect_language_sync(file_path, content);
            results.push((file_path.clone(), result));
        }

        results
    }

    fn detect_language_sync(&mut self, file_path: &str, content: &str) -> LanguageDetectionResult {
        let start_time = Instant::now();
        let file_size = content.len();

        self.stats.total_detections += 1;

        // 检查缓存
        let cache_key = self.generate_cache_key(file_path, content);
        if let Some(cached_result) = self.get_from_cache(&cache_key) {
            self.stats.cache_hits += 1;
            return cached_result.with_timing(start_time.elapsed().as_millis() as u64);
        }

        // 基于文件扩展名检测
        if let Some(lang) = Language::from_extension(file_path) {
            self.stats.extension_based += 1;
            let result = LanguageDetectionResult::new(lang, 0.95, "extension-based")
                .with_timing(start_time.elapsed().as_millis() as u64)
                .with_file_size(file_size);
            self.set_cache(&cache_key, result.clone());
            return result;
        }

        // 启发式检测
        let result = self.heuristic_detector.detect(file_path, content);
        if result.language != Language::Unknown {
            self.stats.heuristic += 1;
        } else {
            self.stats.fallback += 1;
        }

        let result = result
            .with_timing(start_time.elapsed().as_millis() as u64)
            .with_file_size(file_size);
        self.set_cache(&cache_key, result.clone());

        // 更新平均检测时间
        self.update_average_detection_time(start_time.elapsed().as_millis() as u64);

        result
    }

    fn generate_cache_key(&self, file_path: &str, content: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        file_path.hash(&mut hasher);
        content.len().hash(&mut hasher);
        // 使用内容的前1000个字符的哈希来避免完整内容哈希的性能开销
        let content_sample = if content.len() > 1000 {
            &content[..1000]
        } else {
            content
        };
        content_sample.hash(&mut hasher);

        format!("{}:{:x}", file_path, hasher.finish())
    }

    fn get_from_cache(&mut self, cache_key: &str) -> Option<LanguageDetectionResult> {
        if let Some(entry) = self.cache.get_mut(cache_key) {
            if !entry.is_expired(self.cache_ttl) {
                return Some(entry.access());
            } else {
                // 缓存过期，移除条目
                self.cache.remove(cache_key);
            }
        }
        None
    }

    fn set_cache(&mut self, cache_key: &str, result: LanguageDetectionResult) {
        // 如果缓存已满，移除最旧的条目
        if self.cache.len() >= self.max_cache_size {
            self.evict_oldest_cache_entry();
        }

        self.cache.insert(cache_key.to_string(), CacheEntry::new(result));
    }

    fn evict_oldest_cache_entry(&mut self) {
        if let Some((oldest_key, _)) = self.cache.iter()
            .min_by_key(|(_, entry)| entry.created_at)
            .map(|(k, v)| (k.clone(), v.created_at)) {
            self.cache.remove(&oldest_key);
        }
    }

    fn update_average_detection_time(&mut self, detection_time_ms: u64) {
        let total_time = self.stats.average_detection_time_ms * (self.stats.total_detections - 1) as f64;
        self.stats.average_detection_time_ms = (total_time + detection_time_ms as f64) / self.stats.total_detections as f64;
    }

    pub fn get_stats(&self) -> &DetectionStats {
        &self.stats
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    pub fn cache_size(&self) -> usize {
        self.cache.len()
    }
}

/// AI 语言检测配置
#[derive(Debug, Clone)]
pub struct AIDetectionConfig {
    pub provider: String,
    pub model: String,
    pub api_key: Option<String>,
    pub base_url: String,
    pub timeout_seconds: u64,
    pub max_content_length: usize,
}

impl Default for AIDetectionConfig {
    fn default() -> Self {
        Self {
            provider: "ollama".to_string(),
            model: "llama2".to_string(),
            api_key: None,
            base_url: "http://localhost:11434".to_string(),
            timeout_seconds: 30,
            max_content_length: 2000,
        }
    }
}

/// AI 语言检测器
pub struct AILanguageDetector {
    config: AIDetectionConfig,
    client: reqwest::Client,
    prompt_templates: LanguageDetectionPrompts,
}

impl AILanguageDetector {
    pub fn new(config: AIDetectionConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .build()
            .expect("Failed to create HTTP client for AI language detection");

        Self {
            config,
            client,
            prompt_templates: LanguageDetectionPrompts::new(),
        }
    }

    pub async fn detect(&self, file_path: &str, content: &str) -> anyhow::Result<LanguageDetectionResult> {
        // 限制内容长度以避免过长的 AI 请求
        let truncated_content = if content.len() > self.config.max_content_length {
            &content[..self.config.max_content_length]
        } else {
            content
        };

        let prompt = self.prompt_templates.create_detection_prompt(file_path, truncated_content);

        match self.config.provider.as_str() {
            "ollama" => self.detect_with_ollama(&prompt).await,
            "deepseek" => self.detect_with_deepseek(&prompt).await,
            "siliconflow" => self.detect_with_siliconflow(&prompt).await,
            _ => Err(anyhow::anyhow!("Unsupported AI provider: {}", self.config.provider)),
        }
    }

    async fn detect_with_ollama(&self, prompt: &str) -> anyhow::Result<LanguageDetectionResult> {
        let request = serde_json::json!({
            "model": self.config.model,
            "prompt": prompt,
            "stream": false,
            "options": {
                "temperature": 0.1,
                "top_p": 0.9,
                "num_predict": 100
            }
        });

        let url = format!("{}/api/generate", self.config.base_url);
        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("Ollama API request failed: {}", response.status()));
        }

        let response_json: serde_json::Value = response.json().await?;
        let ai_response = response_json["response"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid Ollama response format"))?;

        self.parse_ai_response(ai_response)
    }

    async fn detect_with_deepseek(&self, prompt: &str) -> anyhow::Result<LanguageDetectionResult> {
        let request = serde_json::json!({
            "model": self.config.model,
            "messages": [
                {
                    "role": "user",
                    "content": prompt
                }
            ],
            "temperature": 0.1,
            "max_tokens": 100,
            "stream": false
        });

        let response = self.client
            .post(&self.config.base_url)
            .bearer_auth(self.config.api_key.as_ref().ok_or_else(|| {
                anyhow::anyhow!("DeepSeek API key is required")
            })?)
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("DeepSeek API request failed: {}", response.status()));
        }

        let response_json: serde_json::Value = response.json().await?;
        let ai_response = response_json["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid DeepSeek response format"))?;

        self.parse_ai_response(ai_response)
    }

    async fn detect_with_siliconflow(&self, prompt: &str) -> anyhow::Result<LanguageDetectionResult> {
        let request = serde_json::json!({
            "model": self.config.model,
            "messages": [
                {
                    "role": "user",
                    "content": prompt
                }
            ],
            "temperature": 0.1,
            "max_tokens": 100,
            "stream": false
        });

        let response = self.client
            .post(&self.config.base_url)
            .bearer_auth(self.config.api_key.as_ref().ok_or_else(|| {
                anyhow::anyhow!("SiliconFlow API key is required")
            })?)
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("SiliconFlow API request failed: {}", response.status()));
        }

        let response_json: serde_json::Value = response.json().await?;
        let ai_response = response_json["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid SiliconFlow response format"))?;

        self.parse_ai_response(ai_response)
    }

    fn parse_ai_response(&self, response: &str) -> anyhow::Result<LanguageDetectionResult> {
        // 尝试解析 JSON 格式的响应
        if let Ok(json_response) = serde_json::from_str::<serde_json::Value>(response) {
            if let Some(language_str) = json_response["language"].as_str() {
                if let Some(confidence) = json_response["confidence"].as_f64() {
                    let language = self.parse_language_string(language_str)?;
                    return Ok(LanguageDetectionResult::new(
                        language,
                        confidence as f32,
                        "ai-enhanced",
                    ));
                }
            }
        }

        // 如果不是 JSON 格式，尝试解析纯文本响应
        let response_lower = response.to_lowercase();

        // 查找语言关键词
        let (language, confidence) = if response_lower.contains("go") || response_lower.contains("golang") {
            (Language::Go, 0.95)
        } else if response_lower.contains("rust") {
            (Language::Rust, 0.95)
        } else if response_lower.contains("typescript") || response_lower.contains("ts") {
            (Language::TypeScript, 0.95)
        } else if response_lower.contains("javascript") || response_lower.contains("js") {
            (Language::JavaScript, 0.90)
        } else if response_lower.contains("python") || response_lower.contains("py") {
            (Language::Python, 0.90)
        } else if response_lower.contains("java") {
            (Language::Java, 0.90)
        } else if response_lower.contains("c++") || response_lower.contains("cpp") {
            (Language::Cpp, 0.90)
        } else if response_lower.contains(" c ") || response_lower.contains("c language") {
            (Language::C, 0.90)
        } else {
            return Err(anyhow::anyhow!("Could not parse language from AI response: {}", response));
        };

        Ok(LanguageDetectionResult::new(language, confidence, "ai-enhanced"))
    }

    fn parse_language_string(&self, language_str: &str) -> anyhow::Result<Language> {
        match language_str.to_lowercase().as_str() {
            "go" | "golang" => Ok(Language::Go),
            "rust" => Ok(Language::Rust),
            "typescript" | "ts" => Ok(Language::TypeScript),
            "javascript" | "js" => Ok(Language::JavaScript),
            "python" | "py" => Ok(Language::Python),
            "java" => Ok(Language::Java),
            "c" => Ok(Language::C),
            "c++" | "cpp" => Ok(Language::Cpp),
            _ => Err(anyhow::anyhow!("Unknown language: {}", language_str)),
        }
    }

    pub fn is_available(&self) -> bool {
        match self.config.provider.as_str() {
            "ollama" => true, // Ollama doesn't require API key
            "deepseek" | "siliconflow" => self.config.api_key.is_some(),
            _ => false,
        }
    }
}

/// 语言检测提示词模板
pub struct LanguageDetectionPrompts;

impl LanguageDetectionPrompts {
    pub fn new() -> Self {
        Self
    }

    pub fn create_detection_prompt(&self, file_path: &str, content: &str) -> String {
        format!(
            r#"You are a programming language detection expert. Analyze the following code snippet and determine the programming language.

File path: {}

Code content:
```
{}
```

Please respond with ONLY a JSON object in this exact format:
{{
    "language": "language_name",
    "confidence": 0.95,
    "reasoning": "brief explanation"
}}

Supported languages: go, rust, typescript, javascript, python, java, c, cpp

Rules:
1. Use lowercase language names
2. Confidence should be between 0.0 and 1.0
3. Consider file extension, syntax patterns, and keywords
4. If uncertain, use lower confidence score
5. Respond ONLY with the JSON object, no additional text"#,
            file_path, content
        )
    }

    pub fn create_fallback_prompt(&self, file_path: &str, content: &str) -> String {
        format!(
            r#"Detect programming language for file: {}

Code:
{}

Language (one word only):"#,
            file_path, content
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_language_detection_result_creation() {
        let result = LanguageDetectionResult::new(Language::Go, 0.95, "extension-based");

        assert_eq!(result.language, Language::Go);
        assert_eq!(result.confidence, 0.95);
        assert_eq!(result.detection_method, "extension-based");
        assert!(result.features.is_empty());
        assert_eq!(result.detection_time_ms, 0);
        assert_eq!(result.file_size, 0);
    }

    #[test]
    fn test_language_detection_result_with_features() {
        let features = vec![
            LanguageFeature::Package("main".to_string()),
            LanguageFeature::Function("main".to_string()),
        ];

        let result = LanguageDetectionResult::new(Language::Go, 0.95, "extension-based")
            .with_features(features.clone())
            .with_timing(100)
            .with_file_size(1024);

        assert_eq!(result.features, features);
        assert_eq!(result.detection_time_ms, 100);
        assert_eq!(result.file_size, 1024);
    }

    #[test]
    fn test_detection_stats() {
        let mut stats = DetectionStats::default();

        assert_eq!(stats.cache_hit_rate(), 0.0);

        stats.total_detections = 100;
        stats.cache_hits = 80;

        assert_eq!(stats.cache_hit_rate(), 0.8);
    }

    #[test]
    fn test_language_detector_creation() {
        let detector = LanguageDetector::new();

        assert!(detector.ai_detector.is_none());
        assert_eq!(detector.cache_ttl, Duration::from_secs(300));
        assert_eq!(detector.max_cache_size, 1000);
        assert_eq!(detector.cache_size(), 0);
    }

    #[test]
    fn test_language_detector_with_cache_config() {
        let detector = LanguageDetector::new()
            .with_cache_config(Duration::from_secs(600), 2000);

        assert_eq!(detector.cache_ttl, Duration::from_secs(600));
        assert_eq!(detector.max_cache_size, 2000);
    }

    #[test]
    fn test_extension_based_detection() {
        let mut detector = LanguageDetector::new();

        let result = detector.detect_language_sync("test.go", "package main\nfunc main() {}");
        assert_eq!(result.language, Language::Go);
        assert_eq!(result.detection_method, "extension-based");
        assert_eq!(result.confidence, 0.95);

        let result = detector.detect_language_sync("test.rs", "fn main() {}");
        assert_eq!(result.language, Language::Rust);
        assert_eq!(result.detection_method, "extension-based");

        let result = detector.detect_language_sync("test.ts", "interface Test {}");
        assert_eq!(result.language, Language::TypeScript);
        assert_eq!(result.detection_method, "extension-based");
    }

    #[test]
    fn test_heuristic_detection_go() {
        let mut detector = LanguageDetector::new();

        let go_code = r#"
package main

import "fmt"

func main() {
    fmt.Println("Hello, World!")
    go func() {
        defer fmt.Println("Deferred")
    }()
}
"#;

        let result = detector.detect_language_sync("unknown_file", go_code);
        assert_eq!(result.language, Language::Go);
        assert_eq!(result.detection_method, "heuristic");
        assert!(result.confidence > 0.3);
    }

    #[test]
    fn test_heuristic_detection_rust() {
        let mut detector = LanguageDetector::new();

        let rust_code = r#"
use std::collections::HashMap;

fn main() {
    let mut map = HashMap::new();
    map.insert("key", "value");

    match map.get("key") {
        Some(value) => println!("{}", value),
        None => println!("Not found"),
    }
}

impl MyStruct {
    pub fn new() -> Self {
        Self {}
    }
}
"#;

        let result = detector.detect_language_sync("unknown_file", rust_code);
        assert_eq!(result.language, Language::Rust);
        assert_eq!(result.detection_method, "heuristic");
        assert!(result.confidence > 0.3);
    }

    #[test]
    fn test_heuristic_detection_typescript() {
        let mut detector = LanguageDetector::new();

        let ts_code = r#"
interface User {
    name: string;
    age: number;
    isActive: boolean;
}

export class UserService {
    async getUser(id: string): Promise<User> {
        const response = await fetch(`/api/users/${id}`);
        return response.json();
    }
}

type UserKeys = keyof User;
"#;

        let result = detector.detect_language_sync("unknown_file", ts_code);
        assert_eq!(result.language, Language::TypeScript);
        assert_eq!(result.detection_method, "heuristic");
        assert!(result.confidence > 0.3);
    }

    #[test]
    fn test_filename_pattern_detection() {
        let detector = HeuristicDetector::new();

        let result = detector.detect("Makefile", "CC=gcc\nall:\n\tgcc -o main main.c");
        assert_eq!(result.language, Language::C);
        assert_eq!(result.detection_method, "filename-pattern");

        let result = detector.detect("Cargo.toml", "[package]\nname = \"test\"");
        assert_eq!(result.language, Language::Rust);
        assert_eq!(result.detection_method, "filename-pattern");

        let result = detector.detect("go.mod", "module test\ngo 1.19");
        assert_eq!(result.language, Language::Go);
        assert_eq!(result.detection_method, "filename-pattern");
    }

    #[test]
    fn test_cache_functionality() {
        let mut detector = LanguageDetector::new();

        // First detection should not be cached
        let result1 = detector.detect_language_sync("test.go", "package main");
        assert_eq!(detector.get_stats().cache_hits, 0);
        assert_eq!(detector.cache_size(), 1);

        // Second detection of same file should be cached
        let result2 = detector.detect_language_sync("test.go", "package main");
        assert_eq!(detector.get_stats().cache_hits, 1);
        assert_eq!(result1.language, result2.language);

        // Clear cache
        detector.clear_cache();
        assert_eq!(detector.cache_size(), 0);
    }

    #[test]
    fn test_batch_detection() {
        let mut detector = LanguageDetector::new();

        let files = vec![
            ("test.go".to_string(), "package main\nfunc main() {}".to_string()),
            ("test.rs".to_string(), "fn main() {}".to_string()),
            ("test.ts".to_string(), "interface Test {}".to_string()),
        ];

        let results = detector.detect_languages_batch(&files);

        assert_eq!(results.len(), 3);
        assert_eq!(results[0].1.language, Language::Go);
        assert_eq!(results[1].1.language, Language::Rust);
        assert_eq!(results[2].1.language, Language::TypeScript);
    }

    #[test]
    fn test_unknown_language_fallback() {
        let mut detector = LanguageDetector::new();

        let result = detector.detect_language_sync("unknown.xyz", "some random content");
        assert_eq!(result.language, Language::Unknown);
        assert_eq!(result.detection_method, "heuristic-fallback");
        assert!(result.confidence < 0.3);
    }

    #[test]
    fn test_ai_language_detector() {
        // Test with ollama provider (should be available without API key)
        let config = AIDetectionConfig {
            provider: "ollama".to_string(),
            model: "test-model".to_string(),
            api_key: None,
            base_url: "http://localhost:11434".to_string(),
            timeout_seconds: 30,
            max_content_length: 2000,
        };
        let ai_detector = AILanguageDetector::new(config);

        // Test availability check for ollama
        assert!(ai_detector.is_available());

        // Test with deepseek provider (should require API key)
        let config_deepseek = AIDetectionConfig {
            provider: "deepseek".to_string(),
            model: "test-model".to_string(),
            api_key: None,
            base_url: "https://api.deepseek.com".to_string(),
            timeout_seconds: 30,
            max_content_length: 2000,
        };
        let ai_detector_deepseek = AILanguageDetector::new(config_deepseek);
        assert!(!ai_detector_deepseek.is_available()); // Should be false without API key

        // Test with API key
        let config_with_key = AIDetectionConfig {
            provider: "deepseek".to_string(),
            model: "test-model".to_string(),
            api_key: Some("test-key".to_string()),
            base_url: "https://api.deepseek.com".to_string(),
            timeout_seconds: 30,
            max_content_length: 2000,
        };
        let ai_detector_with_key = AILanguageDetector::new(config_with_key);
        assert!(ai_detector_with_key.is_available()); // Should be true with API key

        // Test prompt template creation
        let prompt = ai_detector.prompt_templates.create_detection_prompt("test.go", "package main");
        assert!(prompt.contains("test.go"));
        assert!(prompt.contains("package main"));
        assert!(prompt.contains("JSON object"));

        // Test fallback prompt
        let fallback_prompt = ai_detector.prompt_templates.create_fallback_prompt("test.rs", "fn main() {}");
        assert!(fallback_prompt.contains("test.rs"));
        assert!(fallback_prompt.contains("fn main() {}"));

        // Test language string parsing
        assert_eq!(ai_detector.parse_language_string("go").unwrap(), Language::Go);
        assert_eq!(ai_detector.parse_language_string("rust").unwrap(), Language::Rust);
        assert_eq!(ai_detector.parse_language_string("typescript").unwrap(), Language::TypeScript);
        assert!(ai_detector.parse_language_string("unknown").is_err());

        // Test AI response parsing
        let json_response = r#"{"language": "go", "confidence": 0.95, "reasoning": "test"}"#;
        let result = ai_detector.parse_ai_response(json_response).unwrap();
        assert_eq!(result.language, Language::Go);
        assert_eq!(result.confidence, 0.95);

        // Test text response parsing
        let text_response = "This code is written in Rust programming language.";
        let result = ai_detector.parse_ai_response(text_response).unwrap();
        assert_eq!(result.language, Language::Rust);
        assert_eq!(result.confidence, 0.95);
    }

    #[test]
    fn test_language_pattern_scoring() {
        let detector = HeuristicDetector::new();

        // Test strong indicators vs weak indicators
        let strong_go_code = "package main\nfunc main()\nimport (\ngo func()";
        let weak_go_code = "func test()\nvar x int";

        let strong_result = detector.detect("test", strong_go_code);
        let weak_result = detector.detect("test", weak_go_code);

        assert_eq!(strong_result.language, Language::Go);
        assert_eq!(weak_result.language, Language::Go);
        assert!(strong_result.confidence > weak_result.confidence);
    }

    #[tokio::test]
    async fn test_async_detection_with_ai() {
        let config = AIDetectionConfig {
            provider: "test".to_string(),
            model: "test-model".to_string(),
            api_key: None,
            base_url: "http://localhost:11434".to_string(),
            timeout_seconds: 30,
            max_content_length: 2000,
        };
        let ai_detector = AILanguageDetector::new(config);
        let mut detector = LanguageDetector::new().with_ai_detector(ai_detector);

        // Test that AI detection would be attempted for unknown extensions
        // Since we can't actually make AI calls in tests, we test the fallback behavior
        let result = detector.detect_language("unknown_file", "package main\nfunc main() {}").await;
        // Should fall back to heuristic since AI call will fail
        assert_eq!(result.language, Language::Go);
        assert_eq!(result.detection_method, "heuristic");

        // Test fallback to heuristic when AI fails
        let result = detector.detect_language("unknown_file", "random content").await;
        assert_eq!(result.language, Language::Unknown);
        assert_eq!(result.detection_method, "heuristic-fallback");
    }

    #[test]
    fn test_cache_expiration() {
        let mut detector = LanguageDetector::new()
            .with_cache_config(Duration::from_millis(1), 1000); // Very short TTL

        let result1 = detector.detect_language_sync("test.go", "package main");
        assert_eq!(detector.cache_size(), 1);

        // Wait for cache to expire
        std::thread::sleep(Duration::from_millis(2));

        let result2 = detector.detect_language_sync("test.go", "package main");
        // Cache should have been cleared due to expiration
        assert_eq!(detector.get_stats().cache_hits, 0); // Second call shouldn't be a cache hit
    }

    #[test]
    fn test_cache_size_limit() {
        let mut detector = LanguageDetector::new()
            .with_cache_config(Duration::from_secs(300), 2); // Very small cache

        // Fill cache to capacity
        detector.detect_language_sync("test1.go", "package main");
        detector.detect_language_sync("test2.rs", "fn main() {}");
        assert_eq!(detector.cache_size(), 2);

        // Add one more - should evict oldest
        detector.detect_language_sync("test3.ts", "interface Test {}");
        assert_eq!(detector.cache_size(), 2); // Should still be 2 due to eviction
    }
}

/// 语言模式定义
#[derive(Debug, Clone)]
struct LanguagePattern {
    pattern: &'static str,
    weight: f32,
    is_strong_indicator: bool,
}

impl LanguagePattern {
    fn new(pattern: &'static str, weight: f32, is_strong_indicator: bool) -> Self {
        Self {
            pattern,
            weight,
            is_strong_indicator,
        }
    }
}

/// 启发式检测器
pub struct HeuristicDetector {
    patterns: HashMap<Language, Vec<LanguagePattern>>,
}

impl HeuristicDetector {
    pub fn new() -> Self {
        let mut patterns = HashMap::new();

        // Go 语言模式 - 更精确的模式匹配
        patterns.insert(Language::Go, vec![
            LanguagePattern::new("package main", 3.0, true),
            LanguagePattern::new("package ", 2.0, true),
            LanguagePattern::new("func main()", 3.0, true),
            LanguagePattern::new("func ", 1.5, false),
            LanguagePattern::new("import (", 2.0, true),
            LanguagePattern::new("type ", 1.0, false),
            LanguagePattern::new("var ", 1.0, false),
            LanguagePattern::new("const ", 1.0, false),
            LanguagePattern::new("go func", 2.5, true),
            LanguagePattern::new("defer ", 2.0, true),
            LanguagePattern::new("chan ", 2.0, true),
            LanguagePattern::new("select {", 2.5, true),
            LanguagePattern::new("range ", 1.5, false),
            LanguagePattern::new("make(", 1.5, false),
            LanguagePattern::new("fmt.", 1.5, false),
        ]);

        // Rust 语言模式
        patterns.insert(Language::Rust, vec![
            LanguagePattern::new("fn main()", 3.0, true),
            LanguagePattern::new("fn ", 1.5, false),
            LanguagePattern::new("use std::", 2.5, true),
            LanguagePattern::new("use ", 1.0, false),
            LanguagePattern::new("mod ", 1.5, false),
            LanguagePattern::new("struct ", 1.5, false),
            LanguagePattern::new("enum ", 1.5, false),
            LanguagePattern::new("impl ", 2.0, true),
            LanguagePattern::new("trait ", 2.0, true),
            LanguagePattern::new("pub fn", 2.0, true),
            LanguagePattern::new("pub struct", 2.0, true),
            LanguagePattern::new("let mut", 2.0, true),
            LanguagePattern::new("let ", 1.0, false),
            LanguagePattern::new("match ", 2.0, true),
            LanguagePattern::new("unsafe ", 2.5, true),
            LanguagePattern::new("&mut ", 2.0, true),
            LanguagePattern::new("Box<", 2.0, true),
            LanguagePattern::new("Vec<", 2.0, true),
            LanguagePattern::new("Result<", 2.0, true),
            LanguagePattern::new("Option<", 2.0, true),
        ]);

        // TypeScript 语言模式
        patterns.insert(Language::TypeScript, vec![
            LanguagePattern::new("interface ", 3.0, true),
            LanguagePattern::new("type ", 2.0, true),
            LanguagePattern::new(": string", 2.0, true),
            LanguagePattern::new(": number", 2.0, true),
            LanguagePattern::new(": boolean", 2.0, true),
            LanguagePattern::new("Promise<", 2.5, true),
            LanguagePattern::new("async ", 1.5, false),
            LanguagePattern::new("await ", 1.5, false),
            LanguagePattern::new("export interface", 3.0, true),
            LanguagePattern::new("export type", 3.0, true),
            LanguagePattern::new("export class", 2.5, true),
            LanguagePattern::new("import type", 2.5, true),
            LanguagePattern::new("as const", 2.0, true),
            LanguagePattern::new("readonly ", 2.0, true),
            LanguagePattern::new("keyof ", 2.5, true),
            LanguagePattern::new("typeof ", 2.0, true),
            LanguagePattern::new("extends ", 1.5, false),
            LanguagePattern::new("implements ", 2.0, true),
        ]);

        // JavaScript 语言模式
        patterns.insert(Language::JavaScript, vec![
            LanguagePattern::new("function ", 1.5, false),
            LanguagePattern::new("const ", 1.0, false),
            LanguagePattern::new("let ", 1.0, false),
            LanguagePattern::new("var ", 1.0, false),
            LanguagePattern::new("class ", 1.5, false),
            LanguagePattern::new("export ", 1.0, false),
            LanguagePattern::new("import ", 1.0, false),
            LanguagePattern::new("async ", 1.5, false),
            LanguagePattern::new("await ", 1.5, false),
            LanguagePattern::new("Promise", 1.5, false),
            LanguagePattern::new("console.", 1.5, false),
            LanguagePattern::new("require(", 2.0, true),
            LanguagePattern::new("module.exports", 2.5, true),
            LanguagePattern::new("exports.", 2.0, true),
            LanguagePattern::new("process.", 2.0, true),
        ]);

        // Python 语言模式
        patterns.insert(Language::Python, vec![
            LanguagePattern::new("def ", 2.0, true),
            LanguagePattern::new("class ", 1.5, false),
            LanguagePattern::new("import ", 1.0, false),
            LanguagePattern::new("from ", 1.5, false),
            LanguagePattern::new("if __name__", 3.0, true),
            LanguagePattern::new("print(", 1.5, false),
            LanguagePattern::new("self.", 2.0, true),
            LanguagePattern::new("elif ", 2.0, true),
            LanguagePattern::new("lambda ", 2.0, true),
            LanguagePattern::new("yield ", 2.0, true),
        ]);

        // Java 语言模式
        patterns.insert(Language::Java, vec![
            LanguagePattern::new("public class", 3.0, true),
            LanguagePattern::new("private ", 1.5, false),
            LanguagePattern::new("protected ", 1.5, false),
            LanguagePattern::new("public static void main", 3.0, true),
            LanguagePattern::new("import java.", 2.5, true),
            LanguagePattern::new("package ", 2.0, true),
            LanguagePattern::new("extends ", 2.0, true),
            LanguagePattern::new("implements ", 2.0, true),
            LanguagePattern::new("@Override", 2.5, true),
            LanguagePattern::new("System.out", 2.0, true),
        ]);

        // C 语言模式
        patterns.insert(Language::C, vec![
            LanguagePattern::new("#include <", 2.5, true),
            LanguagePattern::new("int main(", 3.0, true),
            LanguagePattern::new("printf(", 2.0, true),
            LanguagePattern::new("malloc(", 2.0, true),
            LanguagePattern::new("free(", 2.0, true),
            LanguagePattern::new("struct ", 1.5, false),
            LanguagePattern::new("typedef ", 2.0, true),
        ]);

        // C++ 语言模式
        patterns.insert(Language::Cpp, vec![
            LanguagePattern::new("#include <iostream>", 3.0, true),
            LanguagePattern::new("std::", 2.5, true),
            LanguagePattern::new("using namespace", 2.5, true),
            LanguagePattern::new("class ", 2.0, true),
            LanguagePattern::new("public:", 2.0, true),
            LanguagePattern::new("private:", 2.0, true),
            LanguagePattern::new("protected:", 2.0, true),
            LanguagePattern::new("cout <<", 2.5, true),
            LanguagePattern::new("cin >>", 2.5, true),
            LanguagePattern::new("new ", 1.5, false),
            LanguagePattern::new("delete ", 2.0, true),
        ]);

        Self { patterns }
    }

    pub fn detect(&self, file_path: &str, content: &str) -> LanguageDetectionResult {
        let mut scores = HashMap::new();

        // 计算每种语言的加权分数
        for (language, patterns) in &self.patterns {
            let mut total_score = 0.0;
            let mut strong_indicators = 0;
            let mut pattern_matches = 0;

            for pattern in patterns {
                if content.contains(pattern.pattern) {
                    total_score += pattern.weight;
                    pattern_matches += 1;
                    if pattern.is_strong_indicator {
                        strong_indicators += 1;
                    }
                }
            }

            // 如果有强指示符，给予额外加分
            if strong_indicators > 0 {
                total_score *= 1.0 + (strong_indicators as f32 * 0.2);
            }

            scores.insert(language.clone(), (total_score, pattern_matches, strong_indicators));
        }

        // 找到最高分的语言
        let best_match = scores.iter()
            .max_by(|(_, (score1, matches1, strong1)), (_, (score2, matches2, strong2))| {
                // 首先比较强指示符数量
                match strong1.cmp(strong2) {
                    std::cmp::Ordering::Equal => {
                        // 然后比较总分数
                        match score1.partial_cmp(score2).unwrap_or(std::cmp::Ordering::Equal) {
                            std::cmp::Ordering::Equal => matches1.cmp(matches2),
                            other => other,
                        }
                    }
                    other => other,
                }
            })
            .map(|(lang, (score, matches, strong))| (lang.clone(), *score, *matches, *strong));

        if let Some((language, score, matches, strong_indicators)) = best_match {
            if score > 0.0 {
                // 计算置信度：基于分数、匹配数量和强指示符
                let base_confidence = (score / 10.0).min(1.0);
                let strong_bonus = if strong_indicators > 0 { 0.2 } else { 0.0 };
                let match_bonus = (matches as f32 / 20.0).min(0.1);

                let confidence = (base_confidence + strong_bonus + match_bonus).min(1.0);

                if confidence > 0.3 {
                    return LanguageDetectionResult::new(language, confidence, "heuristic");
                }
            }
        }

        // 如果没有足够的置信度，尝试基于文件名模式的检测
        if let Some(lang) = self.detect_by_filename_patterns(file_path) {
            return LanguageDetectionResult::new(lang, 0.6, "filename-pattern");
        }

        // 默认返回未知语言
        LanguageDetectionResult::new(Language::Unknown, 0.1, "heuristic-fallback")
    }

    fn detect_by_filename_patterns(&self, file_path: &str) -> Option<Language> {
        let filename = std::path::Path::new(file_path)
            .file_name()?
            .to_str()?
            .to_lowercase();

        // 特殊文件名模式
        match filename.as_str() {
            "makefile" | "makefile.am" | "makefile.in" => Some(Language::C),
            "cargo.toml" | "cargo.lock" => Some(Language::Rust),
            "go.mod" | "go.sum" => Some(Language::Go),
            "package.json" | "tsconfig.json" => Some(Language::TypeScript),
            "requirements.txt" | "setup.py" => Some(Language::Python),
            "pom.xml" | "build.gradle" => Some(Language::Java),
            _ => {
                // 检查文件名中的关键词
                if filename.contains("test") || filename.contains("spec") {
                    // 测试文件，根据扩展名推断
                    return None;
                }
                None
            }
        }
    }
}