use crate::languages::Language;
use super::analyzer::{FunctionInfo, ComplexityConfig};

/// Calculator for cyclomatic complexity
pub struct CyclomaticComplexityCalculator;

impl CyclomaticComplexityCalculator {
    pub fn new() -> Self {
        Self
    }

    /// Calculate cyclomatic complexity for a function
    pub fn calculate(&self, function: &FunctionInfo, language: &Language) -> anyhow::Result<u32> {
        let content = &function.content;
        let mut complexity = 1; // Base complexity

        match language {
            Language::Rust => self.calculate_rust_complexity(content, &mut complexity),
            Language::Go => self.calculate_go_complexity(content, &mut complexity),
            Language::TypeScript | Language::JavaScript => self.calculate_ts_complexity(content, &mut complexity),
            _ => self.calculate_generic_complexity(content, &mut complexity),
        }

        Ok(complexity)
    }

    fn calculate_rust_complexity(&self, content: &str, complexity: &mut u32) {
        for line in content.lines() {
            let line = line.trim();

            // Skip comments
            if line.starts_with("//") || line.starts_with("/*") {
                continue;
            }

            // Count decision points - each adds 1 to complexity
            if line.contains(" if ") || line.starts_with("if ") {
                *complexity += 1;
            }
            if line.contains("else if") {
                *complexity += 1;
            }
            if line.contains(" while ") || line.starts_with("while ") {
                *complexity += 1;
            }
            if line.contains(" for ") || line.starts_with("for ") {
                *complexity += 1;
            }
            if line.contains(" loop") || line.starts_with("loop") {
                *complexity += 1;
            }
            if line.contains("match ") {
                *complexity += 1;
            }

            // Count logical operators
            *complexity += line.matches("&&").count() as u32;
            *complexity += line.matches("||").count() as u32;

            // Count ? operator (Option/Result handling)
            *complexity += line.matches('?').count() as u32;
        }
    }

    fn calculate_go_complexity(&self, content: &str, complexity: &mut u32) {
        for line in content.lines() {
            let line = line.trim();

            // Skip comments
            if line.starts_with("//") || line.starts_with("/*") {
                continue;
            }

            // Count decision points
            if line.contains(" if ") || line.starts_with("if ") {
                *complexity += 1;
            }
            if line.contains("else if") {
                *complexity += 1;
            }
            if line.contains(" for ") || line.starts_with("for ") {
                *complexity += 1;
            }
            if line.contains("switch ") {
                *complexity += 1;
            }
            if line.contains("case ") {
                *complexity += 1;
            }

            // Count logical operators
            *complexity += line.matches("&&").count() as u32;
            *complexity += line.matches("||").count() as u32;
        }
    }

    fn calculate_ts_complexity(&self, content: &str, complexity: &mut u32) {
        for line in content.lines() {
            let line = line.trim();

            // Skip comments
            if line.starts_with("//") || line.starts_with("/*") {
                continue;
            }

            // Count decision points
            if line.contains(" if ") || line.starts_with("if ") {
                *complexity += 1;
            }
            if line.contains("else if") {
                *complexity += 1;
            }
            if line.contains(" for ") || line.starts_with("for ") {
                *complexity += 1;
            }
            if line.contains(" while ") || line.starts_with("while ") {
                *complexity += 1;
            }
            if line.contains("switch ") {
                *complexity += 1;
            }
            if line.contains("case ") {
                *complexity += 1;
            }
            if line.contains(" try ") || line.starts_with("try ") {
                *complexity += 1;
            }
            if line.contains(" catch ") || line.starts_with("catch ") {
                *complexity += 1;
            }

            // Count logical operators
            *complexity += line.matches("&&").count() as u32;
            *complexity += line.matches("||").count() as u32;

            // Count ternary operators
            *complexity += line.matches('?').count() as u32;
        }
    }

    fn calculate_generic_complexity(&self, content: &str, complexity: &mut u32) {
        // Generic complexity calculation for unknown languages
        let decision_patterns = [
            "if", "else", "while", "for", "switch", "case",
            "&&", "||", "?", "catch", "try"
        ];

        for line in content.lines() {
            let line = line.trim().to_lowercase();

            for pattern in &decision_patterns {
                if line.contains(pattern) {
                    *complexity += 1;
                }
            }
        }
    }
}

/// Calculator for cognitive complexity
pub struct CognitiveComplexityCalculator;

impl CognitiveComplexityCalculator {
    pub fn new() -> Self {
        Self
    }

    /// Calculate cognitive complexity for a function
    pub fn calculate(&self, function: &FunctionInfo, language: &Language) -> anyhow::Result<u32> {
        let content = &function.content;
        let mut complexity = 0;
        let mut nesting_level = 0;

        match language {
            Language::Rust => self.calculate_rust_cognitive(content, &mut complexity, &mut nesting_level),
            Language::Go => self.calculate_go_cognitive(content, &mut complexity, &mut nesting_level),
            Language::TypeScript | Language::JavaScript => self.calculate_ts_cognitive(content, &mut complexity, &mut nesting_level),
            _ => self.calculate_generic_cognitive(content, &mut complexity, &mut nesting_level),
        }

        Ok(complexity)
    }

    fn calculate_rust_cognitive(&self, content: &str, complexity: &mut u32, nesting_level: &mut u32) {
        for line in content.lines() {
            let line = line.trim();

            // Skip comments
            if line.starts_with("//") || line.starts_with("/*") {
                continue;
            }

            // Track nesting level
            if line.contains('{') {
                *nesting_level += 1;
            }
            if line.contains('}') && *nesting_level > 0 {
                *nesting_level -= 1;
            }

            // Cognitive complexity increments
            if line.contains("if ") || line.contains("else if") {
                *complexity += 1 + *nesting_level;
            }
            if line.contains("else") && !line.contains("else if") {
                *complexity += 1;
            }
            if line.contains("match ") {
                *complexity += 1;
            }
            if line.contains("while ") || line.contains("for ") || line.contains("loop") {
                *complexity += 1 + *nesting_level;
            }
            if line.contains("&&") || line.contains("||") {
                *complexity += 1;
            }
            if line.contains("break") || line.contains("continue") {
                *complexity += 1;
            }
            // Nested functions add cognitive load
            if line.contains("fn ") && *nesting_level > 0 {
                *complexity += *nesting_level;
            }
        }
    }

    fn calculate_go_cognitive(&self, content: &str, complexity: &mut u32, nesting_level: &mut u32) {
        for line in content.lines() {
            let line = line.trim();

            if line.starts_with("//") || line.starts_with("/*") {
                continue;
            }

            if line.contains('{') {
                *nesting_level += 1;
            }
            if line.contains('}') && *nesting_level > 0 {
                *nesting_level -= 1;
            }

            if line.contains("if ") {
                *complexity += 1 + *nesting_level;
            }
            if line.contains("else") {
                *complexity += 1;
            }
            if line.contains("switch ") {
                *complexity += 1;
            }
            if line.contains("for ") || line.contains("range ") {
                *complexity += 1 + *nesting_level;
            }
            if line.contains("&&") || line.contains("||") {
                *complexity += 1;
            }
            if line.contains("break") || line.contains("continue") {
                *complexity += 1;
            }
            if line.contains("go ") {
                *complexity += 1; // Goroutines add cognitive load
            }
        }
    }

    fn calculate_ts_cognitive(&self, content: &str, complexity: &mut u32, nesting_level: &mut u32) {
        for line in content.lines() {
            let line = line.trim();

            if line.starts_with("//") || line.starts_with("/*") {
                continue;
            }

            if line.contains('{') {
                *nesting_level += 1;
            }
            if line.contains('}') && *nesting_level > 0 {
                *nesting_level -= 1;
            }

            if line.contains("if ") {
                *complexity += 1 + *nesting_level;
            }
            if line.contains("else") {
                *complexity += 1;
            }
            if line.contains("switch ") {
                *complexity += 1;
            }
            if line.contains("for ") || line.contains("while ") || line.contains("do ") {
                *complexity += 1 + *nesting_level;
            }
            if line.contains("&&") || line.contains("||") {
                *complexity += 1;
            }
            if line.contains("?") && line.contains(":") {
                *complexity += 1; // Ternary operator
            }
            if line.contains("break") || line.contains("continue") {
                *complexity += 1;
            }
            if line.contains("async") || line.contains("await") {
                *complexity += 1; // Async operations add cognitive load
            }
        }
    }

    fn calculate_generic_cognitive(&self, content: &str, complexity: &mut u32, nesting_level: &mut u32) {
        for line in content.lines() {
            let line = line.trim().to_lowercase();

            if line.contains('{') {
                *nesting_level += 1;
            }
            if line.contains('}') && *nesting_level > 0 {
                *nesting_level -= 1;
            }

            if line.contains("if") {
                *complexity += 1 + *nesting_level;
            }
            if line.contains("else") {
                *complexity += 1;
            }
            if line.contains("while") || line.contains("for") {
                *complexity += 1 + *nesting_level;
            }
            if line.contains("&&") || line.contains("||") {
                *complexity += 1;
            }
        }
    }
}

/// Analyzer for function length
pub struct FunctionLengthAnalyzer;

impl FunctionLengthAnalyzer {
    pub fn new() -> Self {
        Self
    }

    /// Analyze function length
    pub fn analyze(&self, function: &FunctionInfo, config: &ComplexityConfig) -> anyhow::Result<u32> {
        let lines: Vec<&str> = function.content.lines().collect();
        let mut length = 0;

        for line in lines {
            let trimmed = line.trim();

            // Skip empty lines if configured
            if !config.include_blank_lines_in_length && trimmed.is_empty() {
                continue;
            }

            // Skip comments if configured
            if !config.include_comments_in_length {
                if trimmed.starts_with("//") ||
                   trimmed.starts_with("/*") ||
                   trimmed.starts_with("*") ||
                   trimmed.starts_with("#") {
                    continue;
                }
            }

            length += 1;
        }

        Ok(length)
    }
}

/// Analyzer for nesting depth
pub struct NestingDepthAnalyzer;

impl NestingDepthAnalyzer {
    pub fn new() -> Self {
        Self
    }

    /// Analyze maximum nesting depth
    pub fn analyze(&self, function: &FunctionInfo, language: &Language) -> anyhow::Result<u32> {
        let content = &function.content;
        let mut max_depth = 0;
        let mut current_depth = 0;

        match language {
            Language::Rust => self.analyze_rust_nesting(content, &mut max_depth, &mut current_depth),
            Language::Go => self.analyze_go_nesting(content, &mut max_depth, &mut current_depth),
            Language::TypeScript | Language::JavaScript => self.analyze_ts_nesting(content, &mut max_depth, &mut current_depth),
            _ => self.analyze_generic_nesting(content, &mut max_depth, &mut current_depth),
        }

        Ok(max_depth)
    }

    fn analyze_rust_nesting(&self, content: &str, max_depth: &mut u32, current_depth: &mut u32) {
        for line in content.lines() {
            let line = line.trim();

            // Skip comments
            if line.starts_with("//") || line.starts_with("/*") {
                continue;
            }

            // Count nesting-inducing constructs
            if line.contains("if ") || line.contains("else if") || line.contains("else") ||
               line.contains("match ") || line.contains("while ") || line.contains("for ") ||
               line.contains("loop") || line.contains("impl ") || line.contains("mod ") {
                if line.contains('{') {
                    *current_depth += 1;
                    *max_depth = (*max_depth).max(*current_depth);
                }
            }

            // Handle opening braces
            let open_braces = line.matches('{').count() as u32;
            let close_braces = line.matches('}').count() as u32;

            *current_depth += open_braces;
            *max_depth = (*max_depth).max(*current_depth);

            if *current_depth >= close_braces {
                *current_depth -= close_braces;
            } else {
                *current_depth = 0;
            }
        }
    }

    fn analyze_go_nesting(&self, content: &str, max_depth: &mut u32, current_depth: &mut u32) {
        for line in content.lines() {
            let line = line.trim();

            if line.starts_with("//") || line.starts_with("/*") {
                continue;
            }

            if line.contains("if ") || line.contains("else") ||
               line.contains("switch ") || line.contains("for ") ||
               line.contains("select ") {
                if line.contains('{') {
                    *current_depth += 1;
                    *max_depth = (*max_depth).max(*current_depth);
                }
            }

            let open_braces = line.matches('{').count() as u32;
            let close_braces = line.matches('}').count() as u32;

            *current_depth += open_braces;
            *max_depth = (*max_depth).max(*current_depth);

            if *current_depth >= close_braces {
                *current_depth -= close_braces;
            } else {
                *current_depth = 0;
            }
        }
    }

    fn analyze_ts_nesting(&self, content: &str, max_depth: &mut u32, current_depth: &mut u32) {
        for line in content.lines() {
            let line = line.trim();

            if line.starts_with("//") || line.starts_with("/*") {
                continue;
            }

            if line.contains("if ") || line.contains("else") ||
               line.contains("switch ") || line.contains("for ") ||
               line.contains("while ") || line.contains("do ") ||
               line.contains("try ") || line.contains("catch ") {
                if line.contains('{') {
                    *current_depth += 1;
                    *max_depth = (*max_depth).max(*current_depth);
                }
            }

            let open_braces = line.matches('{').count() as u32;
            let close_braces = line.matches('}').count() as u32;

            *current_depth += open_braces;
            *max_depth = (*max_depth).max(*current_depth);

            if *current_depth >= close_braces {
                *current_depth -= close_braces;
            } else {
                *current_depth = 0;
            }
        }
    }

    fn analyze_generic_nesting(&self, content: &str, max_depth: &mut u32, current_depth: &mut u32) {
        for line in content.lines() {
            let line = line.trim();

            let open_braces = line.matches('{').count() as u32;
            let close_braces = line.matches('}').count() as u32;

            *current_depth += open_braces;
            *max_depth = (*max_depth).max(*current_depth);

            if *current_depth >= close_braces {
                *current_depth -= close_braces;
            } else {
                *current_depth = 0;
            }
        }
    }
}

impl Default for CyclomaticComplexityCalculator {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for CognitiveComplexityCalculator {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for FunctionLengthAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for NestingDepthAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}