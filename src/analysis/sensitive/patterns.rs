use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::result::{SensitiveInfoType, RiskLevel};

/// 敏感信息模式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensitivePattern {
    pub name: String,
    pub info_type: SensitiveInfoType,
    pub regex: String,
    #[serde(skip)]
    pub compiled_regex: Option<Regex>,
    pub confidence: f32,
    pub risk_level: RiskLevel,
    pub description: String,
    pub recommendations: Vec<String>,
    pub enabled: bool,
    pub case_sensitive: bool,
}

impl SensitivePattern {
    pub fn new(
        name: String,
        info_type: SensitiveInfoType,
        regex: String,
        confidence: f32,
        risk_level: RiskLevel,
        description: String,
    ) -> anyhow::Result<Self> {
        let compiled_regex = if regex.is_empty() {
            None
        } else {
            Some(Regex::new(&regex)?)
        };

        Ok(Self {
            name,
            info_type,
            regex,
            compiled_regex,
            confidence,
            risk_level,
            description,
            recommendations: Vec::new(),
            enabled: true,
            case_sensitive: true,
        })
    }

    pub fn with_recommendations(mut self, recommendations: Vec<String>) -> Self {
        self.recommendations = recommendations;
        self
    }

    pub fn with_case_sensitivity(mut self, case_sensitive: bool) -> Self {
        self.case_sensitive = case_sensitive;
        self
    }

    pub fn compile_regex(&mut self) -> anyhow::Result<()> {
        if self.regex.is_empty() {
            return Ok(());
        }

        let regex_str = if self.case_sensitive {
            self.regex.clone()
        } else {
            format!("(?i){}", self.regex)
        };

        self.compiled_regex = Some(Regex::new(&regex_str)?);
        Ok(())
    }

    pub fn is_match(&self, text: &str) -> bool {
        if let Some(ref regex) = self.compiled_regex {
            regex.is_match(text)
        } else {
            false
        }
    }

    pub fn find_matches(&self, text: &str) -> Vec<regex::Match> {
        if let Some(ref regex) = self.compiled_regex {
            regex.find_iter(text).collect()
        } else {
            Vec::new()
        }
    }
}

/// 模式匹配引擎
#[derive(Debug)]
pub struct PatternEngine {
    patterns: Vec<SensitivePattern>,
    pattern_index: HashMap<String, usize>,
    enabled_patterns: Vec<usize>,
}

impl PatternEngine {
    pub fn new() -> Self {
        Self {
            patterns: Vec::new(),
            pattern_index: HashMap::new(),
            enabled_patterns: Vec::new(),
        }
    }

    pub fn add_pattern(&mut self, mut pattern: SensitivePattern) -> anyhow::Result<()> {
        // 编译正则表达式
        pattern.compile_regex()?;

        let index = self.patterns.len();
        self.pattern_index.insert(pattern.name.clone(), index);

        if pattern.enabled {
            self.enabled_patterns.push(index);
        }

        self.patterns.push(pattern);
        Ok(())
    }

    pub fn add_patterns(&mut self, patterns: Vec<SensitivePattern>) -> anyhow::Result<()> {
        for pattern in patterns {
            self.add_pattern(pattern)?;
        }
        Ok(())
    }

    pub fn get_pattern(&self, name: &str) -> Option<&SensitivePattern> {
        self.pattern_index.get(name)
            .and_then(|&index| self.patterns.get(index))
    }

    pub fn enable_pattern(&mut self, name: &str) -> bool {
        if let Some(&index) = self.pattern_index.get(name) {
            if let Some(pattern) = self.patterns.get_mut(index) {
                pattern.enabled = true;
                if !self.enabled_patterns.contains(&index) {
                    self.enabled_patterns.push(index);
                }
                return true;
            }
        }
        false
    }

    pub fn disable_pattern(&mut self, name: &str) -> bool {
        if let Some(&index) = self.pattern_index.get(name) {
            if let Some(pattern) = self.patterns.get_mut(index) {
                pattern.enabled = false;
                self.enabled_patterns.retain(|&i| i != index);
                return true;
            }
        }
        false
    }

    pub fn get_enabled_patterns(&self) -> Vec<&SensitivePattern> {
        self.enabled_patterns.iter()
            .filter_map(|&index| self.patterns.get(index))
            .collect()
    }

    pub fn match_text(&self, text: &str) -> Vec<PatternMatch> {
        let mut matches = Vec::new();

        for &index in &self.enabled_patterns {
            if let Some(pattern) = self.patterns.get(index) {
                for regex_match in pattern.find_matches(text) {
                    matches.push(PatternMatch {
                        pattern_name: pattern.name.clone(),
                        pattern_index: index,
                        start: regex_match.start(),
                        end: regex_match.end(),
                        matched_text: regex_match.as_str().to_string(),
                    });
                }
            }
        }

        // 按位置排序
        matches.sort_by_key(|m| m.start);
        matches
    }

    pub fn get_pattern_count(&self) -> usize {
        self.patterns.len()
    }

    pub fn get_enabled_pattern_count(&self) -> usize {
        self.enabled_patterns.len()
    }

    pub fn clear_patterns(&mut self) {
        self.patterns.clear();
        self.pattern_index.clear();
        self.enabled_patterns.clear();
    }
}

impl Default for PatternEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// 模式匹配结果
#[derive(Debug, Clone)]
pub struct PatternMatch {
    pub pattern_name: String,
    pub pattern_index: usize,
    pub start: usize,
    pub end: usize,
    pub matched_text: String,
}

impl PatternMatch {
    pub fn get_line_column(&self, text: &str) -> (usize, usize, usize) {
        let mut line = 1;
        let mut column_start = 1;
        let mut column_end = 1;

        for (i, ch) in text.char_indices() {
            if i == self.start {
                column_start = column_end;
            }
            if i == self.end {
                break;
            }
            if ch == '\n' {
                line += 1;
                column_end = 1;
            } else {
                column_end += 1;
            }
        }

        (line, column_start, column_start + self.matched_text.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sensitive_pattern_creation() {
        let pattern = SensitivePattern::new(
            "test_pattern".to_string(),
            SensitiveInfoType::ApiKey,
            r"AKIA[0-9A-Z]{16}".to_string(),
            0.95,
            RiskLevel::Critical,
            "Test AWS API key pattern".to_string(),
        ).unwrap();

        assert_eq!(pattern.name, "test_pattern");
        assert_eq!(pattern.confidence, 0.95);
        assert_eq!(pattern.risk_level, RiskLevel::Critical);
        assert!(pattern.compiled_regex.is_some());
    }

    #[test]
    fn test_pattern_matching() {
        let mut pattern = SensitivePattern::new(
            "aws_key".to_string(),
            SensitiveInfoType::ApiKey,
            r"AKIA[0-9A-Z]{16}".to_string(),
            0.95,
            RiskLevel::Critical,
            "AWS API key".to_string(),
        ).unwrap();

        let test_text = "const key = 'AKIA1234567890ABCDEF';";
        assert!(pattern.is_match(test_text));

        let matches = pattern.find_matches(test_text);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].as_str(), "AKIA1234567890ABCDEF");
    }

    #[test]
    fn test_pattern_engine() {
        let mut engine = PatternEngine::new();

        let pattern = SensitivePattern::new(
            "test_key".to_string(),
            SensitiveInfoType::ApiKey,
            r"test_[0-9]+".to_string(),
            0.8,
            RiskLevel::Medium,
            "Test pattern".to_string(),
        ).unwrap();

        engine.add_pattern(pattern).unwrap();

        let matches = engine.match_text("This is test_123 and test_456");
        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0].matched_text, "test_123");
        assert_eq!(matches[1].matched_text, "test_456");
    }

    #[test]
    fn test_pattern_enable_disable() {
        let mut engine = PatternEngine::new();

        let pattern = SensitivePattern::new(
            "test_pattern".to_string(),
            SensitiveInfoType::Token,
            r"token_\w+".to_string(),
            0.9,
            RiskLevel::High,
            "Test token pattern".to_string(),
        ).unwrap();

        engine.add_pattern(pattern).unwrap();
        assert_eq!(engine.get_enabled_pattern_count(), 1);

        engine.disable_pattern("test_pattern");
        assert_eq!(engine.get_enabled_pattern_count(), 0);

        engine.enable_pattern("test_pattern");
        assert_eq!(engine.get_enabled_pattern_count(), 1);
    }

    #[test]
    fn test_pattern_match_line_column() {
        let text = "line 1\nline 2 with test_123\nline 3";
        let pattern_match = PatternMatch {
            pattern_name: "test".to_string(),
            pattern_index: 0,
            start: 15, // position of "test_123"
            end: 23,
            matched_text: "test_123".to_string(),
        };

        let (line, col_start, col_end) = pattern_match.get_line_column(text);
        assert_eq!(line, 2);
        assert_eq!(col_start, 8);
        assert_eq!(col_end, 16);
    }
}