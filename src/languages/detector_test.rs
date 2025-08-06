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
        let ai_detector = AILanguageDetector::new("test".to_string(), "test-model".to_string());

        // Test Go detection
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(ai_detector.detect("test.go", "package main\nfunc main() {}"));
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.language, Language::Go);
        assert_eq!(result.detection_method, "ai-enhanced");
        assert_eq!(result.confidence, 0.98);

        // Test Rust detection
        let result = rt.block_on(ai_detector.detect("test.rs", "fn main() {}\nuse std::collections::HashMap;"));
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.language, Language::Rust);

        // Test TypeScript detection
        let result = rt.block_on(ai_detector.detect("test.ts", "interface Test {}\ntype MyType = string;"));
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.language, Language::TypeScript);

        // Test unknown content
        let result = rt.block_on(ai_detector.detect("test.xyz", "random content"));
        assert!(result.is_err());
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
        let ai_detector = AILanguageDetector::new("test".to_string(), "test-model".to_string());
        let mut detector = LanguageDetector::new().with_ai_detector(ai_detector);

        // Test that AI detection is attempted for unknown extensions
        let result = detector.detect_language("unknown_file", "package main\nfunc main() {}").await;
        assert_eq!(result.language, Language::Go);
        assert_eq!(result.detection_method, "ai-enhanced");

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