use std::time::Duration;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

use crate::cache::manager::{CacheConfig, CacheEntry};

/// Cache eviction strategies
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvictionStrategy {
    /// Least Recently Used
    LRU,
    /// Least Frequently Used
    LFU,
    /// Time-based expiration
    TTL,
    /// First In, First Out
    FIFO,
}

/// Cache key generation strategies
#[derive(Debug, Clone)]
pub enum KeyStrategy {
    /// Simple string key
    Simple(String),
    /// Hierarchical key with namespace
    Hierarchical { namespace: String, key: String },
    /// Hash-based key for content
    ContentHash(String),
    /// Composite key with multiple components
    Composite(Vec<String>),
}

impl KeyStrategy {
    pub fn to_string(&self) -> String {
        match self {
            KeyStrategy::Simple(key) => key.clone(),
            KeyStrategy::Hierarchical { namespace, key } => {
                format!("{}:{}", namespace, key)
            }
            KeyStrategy::ContentHash(hash) => format!("hash:{}", hash),
            KeyStrategy::Composite(components) => components.join(":"),
        }
    }

    /// Create a key for file analysis results
    pub fn file_analysis(file_path: &str, analysis_type: &str) -> Self {
        KeyStrategy::Hierarchical {
            namespace: "analysis".to_string(),
            key: format!("{}:{}", analysis_type, file_path),
        }
    }

    /// Create a key for language detection results
    pub fn language_detection(file_path: &str) -> Self {
        KeyStrategy::Hierarchical {
            namespace: "language".to_string(),
            key: file_path.to_string(),
        }
    }

    /// Create a key for AI review results
    pub fn ai_review(file_path: &str, provider: &str, model: &str) -> Self {
        KeyStrategy::Composite(vec![
            "ai_review".to_string(),
            provider.to_string(),
            model.to_string(),
            file_path.to_string(),
        ])
    }

    /// Create a key for static analysis results
    pub fn static_analysis(file_path: &str, tool: &str) -> Self {
        KeyStrategy::Hierarchical {
            namespace: "static".to_string(),
            key: format!("{}:{}", tool, file_path),
        }
    }

    /// Create a key for sensitive info detection results
    pub fn sensitive_info(file_path: &str) -> Self {
        KeyStrategy::Hierarchical {
            namespace: "sensitive".to_string(),
            key: file_path.to_string(),
        }
    }
}

/// Cache strategy configuration and utilities
pub struct CacheStrategy {
    eviction_strategy: EvictionStrategy,
    default_ttl: Duration,
    max_entry_size: usize,
    compression_threshold: usize,
}

impl CacheStrategy {
    pub fn new(config: &CacheConfig) -> Self {
        Self {
            eviction_strategy: EvictionStrategy::LRU,
            default_ttl: config.default_ttl,
            max_entry_size: 10 * 1024 * 1024, // 10MB
            compression_threshold: 1024, // 1KB
        }
    }

    /// Determine if an entry should be cached based on strategy
    pub fn should_cache(&self, key: &str, data_size: usize) -> bool {
        // Don't cache if data is too large
        if data_size > self.max_entry_size {
            return false;
        }

        // Don't cache temporary or one-time use data
        if key.contains("temp") || key.contains("tmp") {
            return false;
        }

        true
    }

    /// Calculate TTL for a specific key based on strategy
    pub fn calculate_ttl(&self, key: &str, data_size: usize) -> Duration {
        // Longer TTL for smaller, frequently accessed data
        let base_ttl = self.default_ttl;

        // Adjust based on key type
        if key.starts_with("language:") {
            // Language detection results are stable
            return base_ttl * 4;
        } else if key.starts_with("static:") {
            // Static analysis results change with code
            return base_ttl / 2;
        } else if key.starts_with("ai_review:") {
            // AI reviews are expensive to generate
            return base_ttl * 2;
        } else if key.starts_with("sensitive:") {
            // Sensitive info detection is relatively stable
            return base_ttl;
        }

        // Adjust based on data size (smaller data cached longer)
        if data_size < 1024 {
            base_ttl * 2
        } else if data_size > 100 * 1024 {
            base_ttl / 2
        } else {
            base_ttl
        }
    }

    /// Determine cache priority for eviction decisions
    pub fn calculate_priority(&self, entry: &CacheEntry) -> f64 {
        let now = Utc::now();
        let age = (now - entry.created_at).num_seconds() as f64;
        let last_access_age = (now - entry.last_accessed).num_seconds() as f64;

        match self.eviction_strategy {
            EvictionStrategy::LRU => {
                // Lower score = higher priority for eviction
                -last_access_age
            }
            EvictionStrategy::LFU => {
                // Lower score = higher priority for eviction
                -(entry.access_count as f64)
            }
            EvictionStrategy::TTL => {
                // Prioritize by expiration time
                if let Some(expires_at) = entry.expires_at {
                    -(expires_at.timestamp() as f64)
                } else {
                    -age
                }
            }
            EvictionStrategy::FIFO => {
                // Older entries have higher priority for eviction
                -age
            }
        }
    }

    /// Check if data should be compressed before caching
    pub fn should_compress(&self, data_size: usize) -> bool {
        data_size >= self.compression_threshold
    }

    /// Get cache warming strategies for common access patterns
    pub fn get_warming_keys(&self) -> Vec<KeyStrategy> {
        vec![
            // Common language detection patterns
            KeyStrategy::Hierarchical {
                namespace: "language".to_string(),
                key: "*.rs".to_string(),
            },
            KeyStrategy::Hierarchical {
                namespace: "language".to_string(),
                key: "*.go".to_string(),
            },
            KeyStrategy::Hierarchical {
                namespace: "language".to_string(),
                key: "*.ts".to_string(),
            },
            // Common static analysis patterns
            KeyStrategy::Hierarchical {
                namespace: "static".to_string(),
                key: "rustfmt:*".to_string(),
            },
            KeyStrategy::Hierarchical {
                namespace: "static".to_string(),
                key: "gofmt:*".to_string(),
            },
        ]
    }
}

/// Cache invalidation strategies
#[derive(Debug, Clone)]
pub struct InvalidationStrategy {
    /// File modification time-based invalidation
    pub file_mtime_based: bool,
    /// Git commit-based invalidation
    pub git_commit_based: bool,
    /// Manual invalidation patterns
    pub manual_patterns: Vec<String>,
}

impl Default for InvalidationStrategy {
    fn default() -> Self {
        Self {
            file_mtime_based: true,
            git_commit_based: true,
            manual_patterns: vec![
                "config:*".to_string(),
                "rules:*".to_string(),
            ],
        }
    }
}

impl InvalidationStrategy {
    /// Check if a cache entry should be invalidated
    pub fn should_invalidate(&self, key: &str, entry: &CacheEntry, file_mtime: Option<DateTime<Utc>>) -> bool {
        // Check file modification time
        if self.file_mtime_based {
            if let Some(mtime) = file_mtime {
                if mtime > entry.created_at {
                    return true;
                }
            }
        }

        // Check manual patterns
        for pattern in &self.manual_patterns {
            if self.matches_pattern(key, pattern) {
                return true;
            }
        }

        false
    }

    fn matches_pattern(&self, key: &str, pattern: &str) -> bool {
        if pattern.ends_with('*') {
            let prefix = &pattern[..pattern.len() - 1];
            key.starts_with(prefix)
        } else if pattern.starts_with('*') {
            let suffix = &pattern[1..];
            key.ends_with(suffix)
        } else {
            key == pattern
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_strategy_generation() {
        let key = KeyStrategy::file_analysis("src/main.rs", "complexity");
        assert_eq!(key.to_string(), "analysis:complexity:src/main.rs");

        let key = KeyStrategy::ai_review("src/lib.rs", "deepseek", "coder");
        assert_eq!(key.to_string(), "ai_review:deepseek:coder:src/lib.rs");

        let key = KeyStrategy::language_detection("test.go");
        assert_eq!(key.to_string(), "language:test.go");
    }

    #[test]
    fn test_cache_strategy_should_cache() {
        let config = CacheConfig::default();
        let strategy = CacheStrategy::new(&config);

        assert!(strategy.should_cache("analysis:test", 1024));
        assert!(!strategy.should_cache("temp:test", 1024));
        assert!(!strategy.should_cache("analysis:test", 20 * 1024 * 1024)); // Too large
    }

    #[test]
    fn test_ttl_calculation() {
        let config = CacheConfig::default();
        let strategy = CacheStrategy::new(&config);

        let language_ttl = strategy.calculate_ttl("language:test.rs", 100);
        let static_ttl = strategy.calculate_ttl("static:rustfmt:test.rs", 100);
        let ai_ttl = strategy.calculate_ttl("ai_review:deepseek:test.rs", 100);

        assert!(language_ttl > static_ttl);
        assert!(ai_ttl > static_ttl);
    }

    #[test]
    fn test_invalidation_strategy() {
        let strategy = InvalidationStrategy::default();
        let entry = CacheEntry::new(vec![1, 2, 3], None);

        // Should invalidate if file is newer
        let newer_time = Utc::now() + chrono::Duration::seconds(10);
        assert!(strategy.should_invalidate("test:key", &entry, Some(newer_time)));

        // Should not invalidate if file is older
        let older_time = Utc::now() - chrono::Duration::seconds(10);
        assert!(!strategy.should_invalidate("test:key", &entry, Some(older_time)));

        // Should invalidate config keys
        assert!(strategy.should_invalidate("config:test", &entry, None));
    }
}