use super::result::{
    CodeBlock, CodeDuplication, DuplicationResult, DuplicationType, RefactoringSuggestion,
    RiskLevel, RefactoringPriority, SuggestionType, ComplexityLevel
};
use crate::languages::{Language, LanguageDetector};
use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;
use sha2::{Digest, Sha256};
use std::cmp::Ordering;

/// 重复检测器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicationConfig {
    /// 最小重复行数阈值
    pub min_duplicate_lines: usize,
    /// 最小重复字符数阈值
    pub min_duplicate_chars: usize,
    /// 结构相似性阈值 (0.0-1.0)
    pub structural_similarity_threshold: f64,
    /// 是否启用精确重复检测
    pub enable_exact_detection: bool,
    /// 是否启用结构相似性检测
    pub enable_structural_detection: bool,
    /// 是否启用跨文件检测
    pub enable_cross_file_detection: bool,
    /// 忽略的文件模式
    pub ignore_patterns: Vec<String>,
    /// 忽略的代码模式
    pub ignore_code_patterns: Vec<String>,
    /// 白名单文件路径
    pub whitelist_files: Vec<String>,
    /// 是否忽略注释
    pub ignore_comments: bool,
    /// 是否忽略空行
    pub ignore_empty_lines: bool,
    /// 是否忽略导入语句
    pub ignore_imports: bool,
}

impl Default for DuplicationConfig {
    fn default() -> Self {
        Self {
            min_duplicate_lines: 5,
            min_duplicate_chars: 100,
            structural_similarity_threshold: 0.8,
            enable_exact_detection: true,
            enable_structural_detection: true,
            enable_cross_file_detection: true,
            ignore_patterns: vec![
                "*.test.*".to_string(),
                "*.spec.*".to_string(),
                "node_modules/**".to_string(),
                "target/**".to_string(),
                ".git/**".to_string(),
            ],
            ignore_code_patterns: vec![
                r"^\s*//.*$".to_string(),      // 单行注释
                r"^\s*/\*.*\*/\s*$".to_string(), // 单行块注释
                r"^\s*$".to_string(),          // 空行
            ],
            whitelist_files: Vec::new(),
            ignore_comments: true,
            ignore_empty_lines: true,
            ignore_imports: true,
        }
    }
}

/// 重复检测器主接口
pub struct DuplicationDetector {
    config: DuplicationConfig,
    language_detector: Arc<Mutex<LanguageDetector>>,
    exact_detector: ExactDuplicationDetector,
    structural_detector: StructuralDuplicationDetector,
    cross_file_detector: CrossFileDuplicationDetector,
    suggestion_generator: RefactoringSuggestionGenerator,
}

impl DuplicationDetector {
    /// 创建新的重复检测器
    pub fn new(config: DuplicationConfig, language_detector: Arc<Mutex<LanguageDetector>>) -> Self {
        Self {
            exact_detector: ExactDuplicationDetector::new(&config),
            structural_detector: StructuralDuplicationDetector::new(&config),
            cross_file_detector: CrossFileDuplicationDetector::new(&config),
            suggestion_generator: RefactoringSuggestionGenerator::new(),
            config,
            language_detector,
        }
    }

    /// 检测项目中的重复代码
    pub async fn detect_duplications(&mut self, project_path: &str, files: &[String]) -> Result<DuplicationResult> {
        let mut result = DuplicationResult::new(project_path.to_string());

        // 过滤文件
        let filtered_files = self.filter_files(files).await?;
        if filtered_files.is_empty() {
            return Ok(result);
        }

        // 读取文件内容
        let file_contents = self.read_file_contents(&filtered_files).await?;
        let total_lines = self.count_total_lines(&file_contents);

        // 预处理代码内容
        let processed_contents = self.preprocess_contents(&file_contents).await?;

        let mut all_duplications = Vec::new();

        // 1. 精确重复检测
        if self.config.enable_exact_detection {
            let exact_duplications = self.exact_detector.detect(&processed_contents).await?;
            all_duplications.extend(exact_duplications);
        }

        // 2. 结构相似性检测
        if self.config.enable_structural_detection {
            let structural_duplications = self.structural_detector.detect(&processed_contents).await?;
            all_duplications.extend(structural_duplications);
        }

        // 3. 跨文件重复检测
        if self.config.enable_cross_file_detection {
            let cross_file_duplications = self.cross_file_detector.detect(&processed_contents).await?;
            all_duplications.extend(cross_file_duplications);
        }

        // 去重和合并相似的重复块
        let merged_duplications = self.merge_similar_duplications(all_duplications);

        // 添加重复块到结果
        for duplication in merged_duplications {
            result.add_duplication(duplication);
        }

        // 生成重构建议
        let suggestions = self.suggestion_generator.generate_suggestions(&result.duplications).await?;
        for suggestion in suggestions {
            result.add_suggestion(suggestion);
        }

        // 计算统计摘要
        result.calculate_summary(filtered_files.len(), total_lines);

        Ok(result)
    }

    /// 过滤需要检测的文件
    async fn filter_files(&self, files: &[String]) -> Result<Vec<String>> {
        let mut filtered = Vec::new();

        for file_path in files {
            // 检查是否在白名单中
            if self.config.whitelist_files.contains(file_path) {
                continue;
            }

            // 检查是否匹配忽略模式
            let should_ignore = self.config.ignore_patterns.iter().any(|pattern| {
                glob::Pattern::new(pattern)
                    .map(|p| p.matches(file_path))
                    .unwrap_or(false)
            });

            if !should_ignore {
                filtered.push(file_path.clone());
            }
        }

        Ok(filtered)
    }

    /// 读取文件内容
    async fn read_file_contents(&self, files: &[String]) -> Result<HashMap<String, String>> {
        let mut contents = HashMap::new();

        for file_path in files {
            match tokio::fs::read_to_string(file_path).await {
                Ok(content) => {
                    contents.insert(file_path.clone(), content);
                }
                Err(e) => {
                    log::warn!("Failed to read file {}: {}", file_path, e);
                }
            }
        }

        Ok(contents)
    }

    /// 计算总行数
    fn count_total_lines(&self, file_contents: &HashMap<String, String>) -> usize {
        file_contents.values()
            .map(|content| content.lines().count())
            .sum()
    }

    /// 预处理代码内容
    async fn preprocess_contents(&self, file_contents: &HashMap<String, String>) -> Result<HashMap<String, ProcessedContent>> {
        let mut processed = HashMap::new();

        for (file_path, content) in file_contents {
            let language = {
                let mut detector = self.language_detector.lock().await;
                detector.detect_language(file_path, content).await.language
            };

            let processed_content = self.preprocess_single_file(content, &language)?;
            processed.insert(file_path.clone(), ProcessedContent {
                original: content.clone(),
                processed: processed_content,
                language,
                file_path: file_path.clone(),
            });
        }

        Ok(processed)
    }

    /// 预处理单个文件
    fn preprocess_single_file(&self, content: &str, language: &Language) -> Result<String> {
        let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();

        // 移除注释
        if self.config.ignore_comments {
            lines = self.remove_comments(lines, language);
        }

        // 移除空行
        if self.config.ignore_empty_lines {
            lines.retain(|line| !line.trim().is_empty());
        }

        // 移除导入语句
        if self.config.ignore_imports {
            lines = self.remove_imports(lines, language);
        }

        // 应用自定义忽略模式
        for pattern in &self.config.ignore_code_patterns {
            if let Ok(regex) = regex::Regex::new(pattern) {
                lines.retain(|line| !regex.is_match(line));
            }
        }

        Ok(lines.join("\n"))
    }

    /// 移除注释
    fn remove_comments(&self, lines: Vec<String>, language: &Language) -> Vec<String> {
        match language {
            Language::Rust | Language::Go | Language::TypeScript | Language::JavaScript => {
                self.remove_c_style_comments(lines)
            }
            Language::Python => self.remove_python_comments(lines),
            _ => lines, // 对于未知语言，保持原样
        }
    }

    /// 移除C风格注释
    fn remove_c_style_comments(&self, lines: Vec<String>) -> Vec<String> {
        let mut result = Vec::new();
        let mut in_block_comment = false;

        for line in lines {
            let mut processed_line = String::new();
            let mut chars = line.chars().peekable();

            while let Some(ch) = chars.next() {
                if in_block_comment {
                    if ch == '*' && chars.peek() == Some(&'/') {
                        chars.next(); // 消费 '/'
                        in_block_comment = false;
                    }
                } else {
                    if ch == '/' {
                        match chars.peek() {
                            Some('/') => break, // 单行注释，跳过剩余部分
                            Some('*') => {
                                chars.next(); // 消费 '*'
                                in_block_comment = true;
                            }
                            _ => processed_line.push(ch),
                        }
                    } else {
                        processed_line.push(ch);
                    }
                }
            }

            if !in_block_comment {
                result.push(processed_line);
            }
        }

        result
    }

    /// 移除Python注释
    fn remove_python_comments(&self, lines: Vec<String>) -> Vec<String> {
        lines.into_iter()
            .map(|line| {
                if let Some(pos) = line.find('#') {
                    line[..pos].to_string()
                } else {
                    line
                }
            })
            .collect()
    }

    /// 移除导入语句
    fn remove_imports(&self, lines: Vec<String>, language: &Language) -> Vec<String> {
        match language {
            Language::Rust => {
                lines.into_iter()
                    .filter(|line| !line.trim_start().starts_with("use "))
                    .collect()
            }
            Language::Go => {
                lines.into_iter()
                    .filter(|line| !line.trim_start().starts_with("import "))
                    .collect()
            }
            Language::TypeScript | Language::JavaScript => {
                lines.into_iter()
                    .filter(|line| {
                        let trimmed = line.trim_start();
                        !trimmed.starts_with("import ") && !trimmed.starts_with("export ")
                    })
                    .collect()
            }
            Language::Python => {
                lines.into_iter()
                    .filter(|line| {
                        let trimmed = line.trim_start();
                        !trimmed.starts_with("import ") && !trimmed.starts_with("from ")
                    })
                    .collect()
            }
            _ => lines,
        }
    }

    /// 合并相似的重复块
    fn merge_similar_duplications(&self, duplications: Vec<CodeDuplication>) -> Vec<CodeDuplication> {
        // TODO: 实现重复块合并逻辑
        // 这里可以根据相似度和位置信息合并重复的检测结果
        duplications
    }
}

/// 处理后的文件内容
#[derive(Debug, Clone)]
pub struct ProcessedContent {
    pub original: String,
    pub processed: String,
    pub language: Language,
    pub file_path: String,
}

/// 精确重复检测器
///
/// 使用基于哈希的算法检测完全相同的代码块。
/// 支持配置最小重复大小阈值，提供高效的精确匹配检测。
pub struct ExactDuplicationDetector {
    config: DuplicationConfig,
    /// 代码块哈希缓存，用于提高性能
    hash_cache: HashMap<String, String>,
}

impl ExactDuplicationDetector {
    /// 创建新的精确重复检测器
    pub fn new(config: &DuplicationConfig) -> Self {
        Self {
            config: config.clone(),
            hash_cache: HashMap::new(),
        }
    }

    /// 检测精确重复代码块
    ///
    /// 使用滑动窗口算法提取代码块，计算SHA-256哈希值进行精确匹配。
    /// 只有满足最小行数和字符数阈值的代码块才会被检测。
    pub async fn detect(&mut self, contents: &HashMap<String, ProcessedContent>) -> Result<Vec<CodeDuplication>> {
        let mut duplications = Vec::new();
        let mut hash_map: HashMap<String, Vec<CodeBlockLocation>> = HashMap::new();

        // 为每个文件生成代码块哈希
        for (file_path, content) in contents {
            let blocks = self.extract_code_blocks(&content.processed, file_path)?;

            for block_info in blocks {
                if self.meets_size_requirements(&block_info) {
                    let hash = self.calculate_hash_cached(&block_info.content);
                    hash_map.entry(hash.clone()).or_insert_with(Vec::new)
                        .push(CodeBlockLocation {
                            file_path: file_path.clone(),
                            start_line: block_info.start_line,
                            end_line: block_info.end_line,
                            content: block_info.content,
                            hash,
                        });
                }
            }
        }

        // 查找重复的哈希并创建重复对象
        for (hash, locations) in hash_map {
            if locations.len() > 1 {
                let duplication = self.create_duplication_from_locations(hash, locations)?;
                duplications.push(duplication);
            }
        }

        // 按风险等级和行数排序
        duplications.sort_by(|a, b| {
            b.risk_level.cmp(&a.risk_level)
                .then_with(|| b.line_count.cmp(&a.line_count))
        });

        Ok(duplications)
    }

    /// 提取代码块信息
    ///
    /// 使用滑动窗口算法提取不同大小的代码块，
    /// 从最小阈值开始到文件末尾的所有可能组合。
    fn extract_code_blocks(&self, content: &str, file_path: &str) -> Result<Vec<CodeBlockInfo>> {
        let lines: Vec<&str> = content.lines().collect();
        let mut blocks = Vec::new();
        let min_lines = self.config.min_duplicate_lines;

        if lines.len() < min_lines {
            return Ok(blocks);
        }

        // 使用滑动窗口提取代码块
        for window_size in min_lines..=lines.len() {
            for start_idx in 0..=(lines.len() - window_size) {
                let end_idx = start_idx + window_size - 1;
                let block_lines = &lines[start_idx..=end_idx];
                let block_content = block_lines.join("\n");

                // 跳过空白或只有注释的代码块
                if self.is_meaningful_block(&block_content) {
                    blocks.push(CodeBlockInfo {
                        start_line: start_idx + 1,
                        end_line: end_idx + 1,
                        content: block_content,
                        file_path: file_path.to_string(),
                    });
                }
            }
        }

        Ok(blocks)
    }

    /// 检查代码块是否满足大小要求
    fn meets_size_requirements(&self, block_info: &CodeBlockInfo) -> bool {
        let line_count = block_info.end_line - block_info.start_line + 1;
        line_count >= self.config.min_duplicate_lines &&
        block_info.content.len() >= self.config.min_duplicate_chars
    }

    /// 检查代码块是否有意义（不是空白或纯注释）
    fn is_meaningful_block(&self, content: &str) -> bool {
        let non_empty_lines: Vec<&str> = content
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .collect();

        // 至少要有一些非空行
        non_empty_lines.len() >= (self.config.min_duplicate_lines / 2).max(1)
    }

    /// 带缓存的哈希计算
    pub fn calculate_hash_cached(&mut self, content: &str) -> String {
        if let Some(cached_hash) = self.hash_cache.get(content) {
            return cached_hash.clone();
        }

        let hash = self.calculate_hash(content);
        self.hash_cache.insert(content.to_string(), hash.clone());
        hash
    }

    /// 计算内容的SHA-256哈希值
    fn calculate_hash(&self, content: &str) -> String {
        let mut hasher = Sha256::new();

        // 标准化内容：移除行尾空白，统一换行符
        let normalized_content = content
            .lines()
            .map(|line| line.trim_end())
            .collect::<Vec<_>>()
            .join("\n");

        hasher.update(normalized_content.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// 从位置信息创建重复对象
    fn create_duplication_from_locations(
        &self,
        hash: String,
        locations: Vec<CodeBlockLocation>
    ) -> Result<CodeDuplication> {
        if locations.is_empty() {
            anyhow::bail!("Cannot create duplication from empty locations");
        }

        let first_location = &locations[0];
        let line_count = first_location.end_line - first_location.start_line + 1;
        let risk_level = RiskLevel::assess(line_count, 1.0);
        let refactoring_priority = RefactoringPriority::assess(risk_level, locations.len());

        let code_blocks: Vec<CodeBlock> = locations
            .iter()
            .map(|loc| CodeBlock {
                file_path: loc.file_path.clone(),
                start_line: loc.start_line,
                end_line: loc.end_line,
                start_column: None,
                end_column: None,
                content_hash: hash.clone(),
            })
            .collect();

        Ok(CodeDuplication {
            id: uuid::Uuid::new_v4().to_string(),
            duplication_type: DuplicationType::Exact,
            code_blocks,
            content: first_location.content.clone(),
            line_count,
            similarity_score: 1.0, // 精确匹配总是100%相似
            risk_level,
            refactoring_priority,
        })
    }

    /// 清空哈希缓存（用于内存管理）
    pub fn clear_cache(&mut self) {
        self.hash_cache.clear();
    }

    /// 获取缓存统计信息
    pub fn get_cache_stats(&self) -> CacheStats {
        CacheStats {
            cache_size: self.hash_cache.len(),
            memory_usage_bytes: self.hash_cache
                .iter()
                .map(|(k, v)| k.len() + v.len())
                .sum(),
        }
    }
}

/// 代码块信息
#[derive(Debug, Clone)]
struct CodeBlockInfo {
    start_line: usize,
    end_line: usize,
    content: String,
    file_path: String,
}

/// 代码块位置信息
#[derive(Debug, Clone)]
struct CodeBlockLocation {
    file_path: String,
    start_line: usize,
    end_line: usize,
    content: String,
    hash: String,
}

/// 缓存统计信息
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub cache_size: usize,
    pub memory_usage_bytes: usize,
}

/// 结构相似性检测器
///
/// 检测结构相同但内容可能不同的代码块。
/// 使用基于模式的结构分析，识别具有相似控制流和语法结构的代码。
pub struct StructuralDuplicationDetector {
    config: DuplicationConfig,
    /// 结构模式缓存，用于提高性能
    pattern_cache: HashMap<String, StructuralPattern>,
}

impl StructuralDuplicationDetector {
    /// 创建新的结构相似性检测器
    pub fn new(config: &DuplicationConfig) -> Self {
        Self {
            config: config.clone(),
            pattern_cache: HashMap::new(),
        }
    }

    /// 检测结构相似的代码块
    ///
    /// 通过分析代码的结构模式（如控制流、函数调用模式、变量声明等）
    /// 来识别结构相同但具体内容可能不同的代码块。
    pub async fn detect(&mut self, contents: &HashMap<String, ProcessedContent>) -> Result<Vec<CodeDuplication>> {
        let mut duplications = Vec::new();
        let mut pattern_map: HashMap<String, Vec<StructuralMatch>> = HashMap::new();

        // 为每个文件提取结构模式
        for (file_path, content) in contents {
            let patterns = self.extract_structural_patterns(&content.processed, file_path, &content.language)?;

            for pattern_info in patterns {
                if self.meets_structural_requirements(&pattern_info) {
                    let pattern_signature = self.calculate_pattern_signature(&pattern_info);
                    pattern_map.entry(pattern_signature.clone()).or_insert_with(Vec::new)
                        .push(StructuralMatch {
                            file_path: file_path.clone(),
                            start_line: pattern_info.start_line,
                            end_line: pattern_info.end_line,
                            content: pattern_info.content,
                            pattern: pattern_info.pattern,
                            signature: pattern_signature,
                            similarity_features: pattern_info.similarity_features,
                        });
                }
            }
        }

        // 查找结构相似的模式
        for (signature, matches) in pattern_map {
            if matches.len() > 1 {
                // 计算匹配间的相似度
                let similar_groups = self.group_by_similarity(matches)?;

                for group in similar_groups {
                    if group.len() > 1 {
                        let duplication = self.create_structural_duplication(signature.clone(), group)?;
                        duplications.push(duplication);
                    }
                }
            }
        }

        // 按相似度和行数排序
        duplications.sort_by(|a, b| {
            b.similarity_score.partial_cmp(&a.similarity_score).unwrap()
                .then_with(|| b.line_count.cmp(&a.line_count))
        });

        Ok(duplications)
    }

    /// 提取代码的结构模式
    ///
    /// 分析代码的语法结构，提取控制流、函数调用、变量声明等模式。
    /// 这些模式用于后续的结构相似性比较。
    fn extract_structural_patterns(
        &mut self,
        content: &str,
        file_path: &str,
        language: &Language
    ) -> Result<Vec<StructuralPatternInfo>> {
        let lines: Vec<&str> = content.lines().collect();
        let mut patterns = Vec::new();
        let min_lines = self.config.min_duplicate_lines;

        if lines.len() < min_lines {
            return Ok(patterns);
        }

        // 使用滑动窗口提取结构模式
        for window_size in min_lines..=lines.len().min(100) { // 限制最大窗口大小
            for start_idx in 0..=(lines.len() - window_size) {
                let end_idx = start_idx + window_size - 1;
                let block_lines = &lines[start_idx..=end_idx];
                let block_content = block_lines.join("\n");

                if let Some(pattern) = self.analyze_structural_pattern(&block_content, language)? {
                    if self.is_meaningful_structural_pattern(&pattern) {
                        patterns.push(StructuralPatternInfo {
                            start_line: start_idx + 1,
                            end_line: end_idx + 1,
                            content: block_content.clone(),
                            pattern,
                            similarity_features: self.extract_similarity_features(&block_content, language)?,
                        });
                    }
                }
            }
        }

        Ok(patterns)
    }

    /// 分析代码块的结构模式
    ///
    /// 识别代码的结构特征，如控制流语句、函数调用、变量声明等。
    /// 返回标准化的结构模式，用于相似性比较。
    fn analyze_structural_pattern(&mut self, content: &str, language: &Language) -> Result<Option<StructuralPattern>> {
        // 检查缓存
        if let Some(cached_pattern) = self.pattern_cache.get(content) {
            return Ok(Some(cached_pattern.clone()));
        }

        let mut pattern = StructuralPattern::new();

        // 分析控制流结构
        pattern.control_flow = self.extract_control_flow_pattern(content, language)?;

        // 分析函数调用模式
        pattern.function_calls = self.extract_function_call_pattern(content, language)?;

        // 分析变量声明模式
        pattern.variable_declarations = self.extract_variable_declaration_pattern(content, language)?;

        // 分析代码块结构
        pattern.block_structure = self.extract_block_structure_pattern(content, language)?;

        // 分析操作符模式
        pattern.operator_pattern = self.extract_operator_pattern(content, language)?;

        // 计算结构复杂度
        pattern.complexity_score = self.calculate_structural_complexity(&pattern);

        // 如果模式有意义，缓存并返回
        if pattern.is_meaningful() {
            self.pattern_cache.insert(content.to_string(), pattern.clone());
            Ok(Some(pattern))
        } else {
            Ok(None)
        }
    }

    /// 提取控制流模式
    ///
    /// 识别if/else、for/while循环、switch/match等控制流语句的结构。
    fn extract_control_flow_pattern(&self, content: &str, language: &Language) -> Result<Vec<ControlFlowElement>> {
        let mut elements = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        for (line_num, line) in lines.iter().enumerate() {
            let trimmed = line.trim();

            match language {
                Language::Rust => {
                    if let Some(element) = self.parse_rust_control_flow(trimmed, line_num + 1)? {
                        elements.push(element);
                    }
                }
                Language::Go => {
                    if let Some(element) = self.parse_go_control_flow(trimmed, line_num + 1)? {
                        elements.push(element);
                    }
                }
                Language::TypeScript | Language::JavaScript => {
                    if let Some(element) = self.parse_typescript_control_flow(trimmed, line_num + 1)? {
                        elements.push(element);
                    }
                }
                _ => {
                    if let Some(element) = self.parse_generic_control_flow(trimmed, line_num + 1)? {
                        elements.push(element);
                    }
                }
            }
        }

        Ok(elements)
    }

    /// 解析Rust控制流
    fn parse_rust_control_flow(&self, line: &str, line_num: usize) -> Result<Option<ControlFlowElement>> {
        if line.starts_with("if ") || line.contains(" if ") {
            Ok(Some(ControlFlowElement::new(ControlFlowType::If, line_num)))
        } else if line.starts_with("else if ") {
            Ok(Some(ControlFlowElement::new(ControlFlowType::ElseIf, line_num)))
        } else if line.starts_with("else") {
            Ok(Some(ControlFlowElement::new(ControlFlowType::Else, line_num)))
        } else if line.starts_with("for ") {
            Ok(Some(ControlFlowElement::new(ControlFlowType::For, line_num)))
        } else if line.starts_with("while ") {
            Ok(Some(ControlFlowElement::new(ControlFlowType::While, line_num)))
        } else if line.starts_with("match ") {
            Ok(Some(ControlFlowElement::new(ControlFlowType::Match, line_num)))
        } else if line.starts_with("loop") {
            Ok(Some(ControlFlowElement::new(ControlFlowType::Loop, line_num)))
        } else {
            Ok(None)
        }
    }

    /// 解析Go控制流
    fn parse_go_control_flow(&self, line: &str, line_num: usize) -> Result<Option<ControlFlowElement>> {
        if line.starts_with("if ") {
            Ok(Some(ControlFlowElement::new(ControlFlowType::If, line_num)))
        } else if line.starts_with("else if ") {
            Ok(Some(ControlFlowElement::new(ControlFlowType::ElseIf, line_num)))
        } else if line.starts_with("else") {
            Ok(Some(ControlFlowElement::new(ControlFlowType::Else, line_num)))
        } else if line.starts_with("for ") {
            Ok(Some(ControlFlowElement::new(ControlFlowType::For, line_num)))
        } else if line.starts_with("switch ") {
            Ok(Some(ControlFlowElement::new(ControlFlowType::Switch, line_num)))
        } else if line.starts_with("select") {
            Ok(Some(ControlFlowElement::new(ControlFlowType::Select, line_num)))
        } else {
            Ok(None)
        }
    }

    /// 解析TypeScript/JavaScript控制流
    fn parse_typescript_control_flow(&self, line: &str, line_num: usize) -> Result<Option<ControlFlowElement>> {
        if line.starts_with("if ") || line.contains(" if ") {
            Ok(Some(ControlFlowElement::new(ControlFlowType::If, line_num)))
        } else if line.starts_with("else if ") {
            Ok(Some(ControlFlowElement::new(ControlFlowType::ElseIf, line_num)))
        } else if line.starts_with("else") {
            Ok(Some(ControlFlowElement::new(ControlFlowType::Else, line_num)))
        } else if line.starts_with("for ") {
            Ok(Some(ControlFlowElement::new(ControlFlowType::For, line_num)))
        } else if line.starts_with("while ") {
            Ok(Some(ControlFlowElement::new(ControlFlowType::While, line_num)))
        } else if line.starts_with("switch ") {
            Ok(Some(ControlFlowElement::new(ControlFlowType::Switch, line_num)))
        } else if line.starts_with("try") {
            Ok(Some(ControlFlowElement::new(ControlFlowType::Try, line_num)))
        } else if line.starts_with("catch") {
            Ok(Some(ControlFlowElement::new(ControlFlowType::Catch, line_num)))
        } else {
            Ok(None)
        }
    }

    /// 解析通用控制流
    fn parse_generic_control_flow(&self, line: &str, line_num: usize) -> Result<Option<ControlFlowElement>> {
        // 基于关键字的通用识别
        if line.contains("if") && (line.contains("(") || line.contains("{")) {
            Ok(Some(ControlFlowElement::new(ControlFlowType::If, line_num)))
        } else if line.contains("else") {
            Ok(Some(ControlFlowElement::new(ControlFlowType::Else, line_num)))
        } else if line.contains("for") && (line.contains("(") || line.contains("{")) {
            Ok(Some(ControlFlowElement::new(ControlFlowType::For, line_num)))
        } else if line.contains("while") && (line.contains("(") || line.contains("{")) {
            Ok(Some(ControlFlowElement::new(ControlFlowType::While, line_num)))
        } else {
            Ok(None)
        }
    }

    /// 提取函数调用模式
    fn extract_function_call_pattern(&self, content: &str, language: &Language) -> Result<Vec<FunctionCallElement>> {
        let mut calls = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        // 使用正则表达式匹配函数调用模式
        let call_pattern = match language {
            Language::Rust => regex::Regex::new(r"(\w+)\s*\(")?,
            Language::Go => regex::Regex::new(r"(\w+)\s*\(")?,
            Language::TypeScript | Language::JavaScript => regex::Regex::new(r"(\w+)\s*\(")?,
            _ => regex::Regex::new(r"(\w+)\s*\(")?,
        };

        for (line_num, line) in lines.iter().enumerate() {
            for cap in call_pattern.captures_iter(line) {
                if let Some(func_name) = cap.get(1) {
                    calls.push(FunctionCallElement {
                        function_name: func_name.as_str().to_string(),
                        line_number: line_num + 1,
                        call_type: self.classify_function_call(func_name.as_str(), language),
                    });
                }
            }
        }

        Ok(calls)
    }

    /// 分类函数调用类型
    fn classify_function_call(&self, func_name: &str, language: &Language) -> FunctionCallType {
        match language {
            Language::Rust => {
                if func_name.starts_with("std::") || func_name.contains("::") {
                    FunctionCallType::StandardLibrary
                } else if func_name.chars().next().unwrap_or('a').is_uppercase() {
                    FunctionCallType::Constructor
                } else {
                    FunctionCallType::UserDefined
                }
            }
            Language::Go => {
                if func_name.starts_with("fmt.") || func_name.contains(".") {
                    FunctionCallType::StandardLibrary
                } else if func_name.chars().next().unwrap_or('a').is_uppercase() {
                    FunctionCallType::Constructor
                } else {
                    FunctionCallType::UserDefined
                }
            }
            _ => FunctionCallType::UserDefined,
        }
    }

    /// 提取变量声明模式
    fn extract_variable_declaration_pattern(&self, content: &str, language: &Language) -> Result<Vec<VariableDeclarationElement>> {
        let mut declarations = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        let declaration_pattern = match language {
            Language::Rust => regex::Regex::new(r"let\s+(\w+)")?,
            Language::Go => regex::Regex::new(r"(var\s+\w+|:\s*=)")?,
            Language::TypeScript => regex::Regex::new(r"(let|const|var)\s+(\w+)")?,
            Language::JavaScript => regex::Regex::new(r"(let|const|var)\s+(\w+)")?,
            _ => regex::Regex::new(r"(\w+)\s*=")?,
        };

        for (line_num, line) in lines.iter().enumerate() {
            if declaration_pattern.is_match(line) {
                declarations.push(VariableDeclarationElement {
                    line_number: line_num + 1,
                    declaration_type: self.classify_variable_declaration(line, language),
                });
            }
        }

        Ok(declarations)
    }

    /// 分类变量声明类型
    fn classify_variable_declaration(&self, line: &str, language: &Language) -> VariableDeclarationType {
        match language {
            Language::Rust => {
                if line.contains("let mut") {
                    VariableDeclarationType::Mutable
                } else if line.contains("let") {
                    VariableDeclarationType::Immutable
                } else {
                    VariableDeclarationType::Unknown
                }
            }
            Language::TypeScript | Language::JavaScript => {
                if line.contains("const") {
                    VariableDeclarationType::Constant
                } else if line.contains("let") {
                    VariableDeclarationType::BlockScoped
                } else if line.contains("var") {
                    VariableDeclarationType::FunctionScoped
                } else {
                    VariableDeclarationType::Unknown
                }
            }
            _ => VariableDeclarationType::Unknown,
        }
    }

    /// 提取代码块结构模式
    fn extract_block_structure_pattern(&self, content: &str, _language: &Language) -> Result<BlockStructure> {
        let mut structure = BlockStructure::new();
        let mut brace_depth = 0;
        let mut paren_depth = 0;
        let mut bracket_depth = 0;

        for ch in content.chars() {
            match ch {
                '{' => {
                    brace_depth += 1;
                    structure.max_brace_depth = structure.max_brace_depth.max(brace_depth);
                }
                '}' => brace_depth = brace_depth.saturating_sub(1),
                '(' => {
                    paren_depth += 1;
                    structure.max_paren_depth = structure.max_paren_depth.max(paren_depth);
                }
                ')' => paren_depth = paren_depth.saturating_sub(1),
                '[' => {
                    bracket_depth += 1;
                    structure.max_bracket_depth = structure.max_bracket_depth.max(bracket_depth);
                }
                ']' => bracket_depth = bracket_depth.saturating_sub(1),
                _ => {}
            }
        }

        structure.total_braces = content.matches('{').count() + content.matches('}').count();
        structure.total_parens = content.matches('(').count() + content.matches(')').count();
        structure.total_brackets = content.matches('[').count() + content.matches(']').count();

        Ok(structure)
    }

    /// 提取操作符模式
    fn extract_operator_pattern(&self, content: &str, _language: &Language) -> Result<OperatorPattern> {
        let mut pattern = OperatorPattern::new();

        // 统计各种操作符
        pattern.arithmetic_ops = content.matches('+').count() + content.matches('-').count() +
                                content.matches('*').count() + content.matches('/').count();
        pattern.comparison_ops = content.matches("==").count() + content.matches("!=").count() +
                               content.matches("<=").count() + content.matches(">=").count() +
                               content.matches('<').count() + content.matches('>').count();
        pattern.logical_ops = content.matches("&&").count() + content.matches("||").count() +
                            content.matches('!').count();
        pattern.assignment_ops = content.matches('=').count() - content.matches("==").count() * 2 -
                               content.matches("!=").count() - content.matches("<=").count() -
                               content.matches(">=").count();

        Ok(pattern)
    }

    /// 计算结构复杂度
    fn calculate_structural_complexity(&self, pattern: &StructuralPattern) -> f64 {
        let control_flow_complexity = pattern.control_flow.len() as f64 * 2.0;
        let function_call_complexity = pattern.function_calls.len() as f64 * 1.5;
        let variable_complexity = pattern.variable_declarations.len() as f64 * 1.0;
        let block_complexity = (pattern.block_structure.max_brace_depth +
                              pattern.block_structure.max_paren_depth) as f64 * 3.0;
        let operator_complexity = (pattern.operator_pattern.arithmetic_ops +
                                 pattern.operator_pattern.comparison_ops +
                                 pattern.operator_pattern.logical_ops) as f64 * 0.5;

        control_flow_complexity + function_call_complexity + variable_complexity +
        block_complexity + operator_complexity
    }

    /// 提取相似性特征
    fn extract_similarity_features(&self, content: &str, language: &Language) -> Result<SimilarityFeatures> {
        let lines: Vec<&str> = content.lines().collect();

        Ok(SimilarityFeatures {
            line_count: lines.len(),
            non_empty_line_count: lines.iter().filter(|line| !line.trim().is_empty()).count(),
            keyword_count: self.count_language_keywords(content, language),
            identifier_pattern: self.extract_identifier_pattern(content),
            indentation_pattern: self.extract_indentation_pattern(&lines),
            comment_ratio: self.calculate_comment_ratio(content, language),
        })
    }

    /// 统计语言关键字
    fn count_language_keywords(&self, content: &str, language: &Language) -> usize {
        let keywords = match language {
            Language::Rust => vec!["fn", "let", "mut", "if", "else", "match", "for", "while", "loop", "impl", "struct", "enum"],
            Language::Go => vec!["func", "var", "if", "else", "for", "switch", "case", "struct", "interface", "type"],
            Language::TypeScript | Language::JavaScript => vec!["function", "let", "const", "var", "if", "else", "for", "while", "class", "interface"],
            _ => vec!["if", "else", "for", "while", "function", "class"],
        };

        keywords.iter()
            .map(|keyword| content.matches(keyword).count())
            .sum()
    }

    /// 提取标识符模式
    fn extract_identifier_pattern(&self, content: &str) -> String {
        // 简化的标识符模式提取
        let identifier_regex = regex::Regex::new(r"\b[a-zA-Z_][a-zA-Z0-9_]*\b").unwrap();
        let identifiers: Vec<&str> = identifier_regex.find_iter(content)
            .map(|m| m.as_str())
            .collect();

        // 生成标识符长度和首字母的模式
        let mut pattern = String::new();
        for identifier in identifiers.iter().take(10) { // 限制数量避免过长
            pattern.push_str(&format!("{}:{},", identifier.len(), identifier.chars().next().unwrap_or('_')));
        }

        pattern
    }

    /// 提取缩进模式
    fn extract_indentation_pattern(&self, lines: &[&str]) -> String {
        let mut pattern = String::new();
        for line in lines.iter().take(20) { // 限制行数
            let indent_level = line.len() - line.trim_start().len();
            pattern.push_str(&format!("{},", indent_level));
        }
        pattern
    }

    /// 计算注释比例
    fn calculate_comment_ratio(&self, content: &str, language: &Language) -> f64 {
        let total_lines = content.lines().count();
        if total_lines == 0 {
            return 0.0;
        }

        let comment_lines = match language {
            Language::Rust | Language::Go | Language::TypeScript | Language::JavaScript => {
                content.lines().filter(|line| {
                    let trimmed = line.trim();
                    trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with("*")
                }).count()
            }
            Language::Python => {
                content.lines().filter(|line| line.trim().starts_with("#")).count()
            }
            _ => 0,
        };

        comment_lines as f64 / total_lines as f64
    }

    /// 检查结构模式是否满足要求
    fn meets_structural_requirements(&self, pattern_info: &StructuralPatternInfo) -> bool {
        let line_count = pattern_info.end_line - pattern_info.start_line + 1;
        line_count >= self.config.min_duplicate_lines &&
        pattern_info.content.len() >= self.config.min_duplicate_chars &&
        pattern_info.pattern.complexity_score >= 5.0 // 最小复杂度阈值
    }

    /// 检查结构模式是否有意义
    fn is_meaningful_structural_pattern(&self, pattern: &StructuralPattern) -> bool {
        // 至少要有一些结构元素
        !pattern.control_flow.is_empty() ||
        !pattern.function_calls.is_empty() ||
        pattern.block_structure.max_brace_depth > 0 ||
        pattern.complexity_score >= 3.0
    }

    /// 计算模式签名
    fn calculate_pattern_signature(&self, pattern_info: &StructuralPatternInfo) -> String {
        let mut hasher = sha2::Sha256::new();

        // 基于结构特征生成签名
        let signature_data = format!(
            "cf:{:?}|fc:{}|vd:{}|bs:{}|op:{}|comp:{:.1}",
            pattern_info.pattern.control_flow.iter().map(|cf| cf.flow_type).collect::<Vec<_>>(),
            pattern_info.pattern.function_calls.len(),
            pattern_info.pattern.variable_declarations.len(),
            pattern_info.pattern.block_structure.max_brace_depth,
            pattern_info.pattern.operator_pattern.arithmetic_ops + pattern_info.pattern.operator_pattern.comparison_ops,
            pattern_info.pattern.complexity_score
        );

        hasher.update(signature_data.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// 按相似度分组匹配
    fn group_by_similarity(&self, matches: Vec<StructuralMatch>) -> Result<Vec<Vec<StructuralMatch>>> {
        let mut groups = Vec::new();
        let mut remaining_matches = matches;

        while !remaining_matches.is_empty() {
            let base_match = remaining_matches.remove(0);
            let mut current_group = vec![base_match.clone()];

            // 查找与基准匹配相似的其他匹配
            let mut i = 0;
            while i < remaining_matches.len() {
                let similarity = self.calculate_similarity_score(&base_match, &remaining_matches[i])?;
                if similarity >= self.config.structural_similarity_threshold {
                    current_group.push(remaining_matches.remove(i));
                } else {
                    i += 1;
                }
            }

            if current_group.len() > 1 {
                groups.push(current_group);
            }
        }

        Ok(groups)
    }

    /// 计算两个结构匹配的相似度
    fn calculate_similarity_score(&self, match1: &StructuralMatch, match2: &StructuralMatch) -> Result<f64> {
        let features1 = &match1.similarity_features;
        let features2 = &match2.similarity_features;

        // 计算各个特征的相似度
        let line_count_similarity = 1.0 - (features1.line_count as f64 - features2.line_count as f64).abs() /
                                   (features1.line_count.max(features2.line_count) as f64);

        let keyword_similarity = 1.0 - (features1.keyword_count as f64 - features2.keyword_count as f64).abs() /
                               (features1.keyword_count.max(features2.keyword_count) as f64).max(1.0);

        let comment_similarity = 1.0 - (features1.comment_ratio - features2.comment_ratio).abs();

        // 计算标识符模式相似度
        let identifier_similarity = self.calculate_string_similarity(&features1.identifier_pattern, &features2.identifier_pattern);

        // 计算缩进模式相似度
        let indentation_similarity = self.calculate_string_similarity(&features1.indentation_pattern, &features2.indentation_pattern);

        // 加权平均
        let total_similarity = (line_count_similarity * 0.2 +
                              keyword_similarity * 0.3 +
                              comment_similarity * 0.1 +
                              identifier_similarity * 0.2 +
                              indentation_similarity * 0.2);

        Ok(total_similarity)
    }

    /// 计算字符串相似度（简化的编辑距离）
    fn calculate_string_similarity(&self, s1: &str, s2: &str) -> f64 {
        if s1.is_empty() && s2.is_empty() {
            return 1.0;
        }
        if s1.is_empty() || s2.is_empty() {
            return 0.0;
        }

        let len1 = s1.len();
        let len2 = s2.len();
        let max_len = len1.max(len2);

        // 简化的相似度计算：基于公共子串
        let common_chars = s1.chars().filter(|c| s2.contains(*c)).count();
        common_chars as f64 / max_len as f64
    }

    /// 创建结构相似性重复对象
    fn create_structural_duplication(
        &self,
        signature: String,
        matches: Vec<StructuralMatch>
    ) -> Result<CodeDuplication> {
        if matches.is_empty() {
            anyhow::bail!("Cannot create duplication from empty matches");
        }

        let first_match = &matches[0];
        let line_count = first_match.end_line - first_match.start_line + 1;

        // 计算平均相似度
        let mut total_similarity = 0.0;
        let mut comparison_count = 0;

        for i in 0..matches.len() {
            for j in (i + 1)..matches.len() {
                if let Ok(similarity) = self.calculate_similarity_score(&matches[i], &matches[j]) {
                    total_similarity += similarity;
                    comparison_count += 1;
                }
            }
        }

        let average_similarity = if comparison_count > 0 {
            total_similarity / comparison_count as f64
        } else {
            self.config.structural_similarity_threshold
        };

        let risk_level = RiskLevel::assess(line_count, average_similarity);
        let refactoring_priority = RefactoringPriority::assess(risk_level, matches.len());

        let code_blocks: Vec<CodeBlock> = matches
            .iter()
            .map(|m| CodeBlock {
                file_path: m.file_path.clone(),
                start_line: m.start_line,
                end_line: m.end_line,
                start_column: None,
                end_column: None,
                content_hash: signature.clone(),
            })
            .collect();

        Ok(CodeDuplication {
            id: uuid::Uuid::new_v4().to_string(),
            duplication_type: DuplicationType::Structural,
            code_blocks,
            content: first_match.content.clone(),
            line_count,
            similarity_score: average_similarity,
            risk_level,
            refactoring_priority,
        })
    }

    /// 清空模式缓存
    pub fn clear_cache(&mut self) {
        self.pattern_cache.clear();
    }

    /// 获取缓存统计信息
    pub fn get_cache_stats(&self) -> CacheStats {
        CacheStats {
            cache_size: self.pattern_cache.len(),
            memory_usage_bytes: self.pattern_cache
                .iter()
                .map(|(k, _)| k.len() + std::mem::size_of::<StructuralPattern>())
                .sum(),
        }
    }
}

/// 跨文件重复检测器
///
/// 专门检测不同文件之间的重复代码，包括精确重复和结构相似的重复。
/// 使用优化的算法处理大型项目，支持增量检测和缓存机制。
pub struct CrossFileDuplicationDetector {
    config: DuplicationConfig,
    /// 文件间代码块哈希缓存
    cross_file_hash_cache: HashMap<String, Vec<CrossFileCodeBlock>>,
    /// 文件间结构模式缓存
    cross_file_pattern_cache: HashMap<String, Vec<CrossFileStructuralPattern>>,
    /// 性能统计
    performance_stats: CrossFilePerformanceStats,
}

impl CrossFileDuplicationDetector {
    /// 创建新的跨文件重复检测器
    pub fn new(config: &DuplicationConfig) -> Self {
        Self {
            config: config.clone(),
            cross_file_hash_cache: HashMap::new(),
            cross_file_pattern_cache: HashMap::new(),
            performance_stats: CrossFilePerformanceStats::new(),
        }
    }

    /// 检测跨文件重复代码
    ///
    /// 使用多阶段检测策略：
    /// 1. 文件级预过滤 - 快速排除不可能有重复的文件对
    /// 2. 代码块提取 - 从每个文件提取候选代码块
    /// 3. 跨文件匹配 - 在不同文件间查找重复
    /// 4. 相似度计算 - 计算精确匹配和结构相似度
    /// 5. 结果优化 - 合并和排序检测结果
    pub async fn detect(&mut self, contents: &HashMap<String, ProcessedContent>) -> Result<Vec<CodeDuplication>> {
        let start_time = std::time::Instant::now();
        let mut duplications = Vec::new();

        if contents.len() < 2 {
            return Ok(duplications);
        }

        // 1. 文件级预过滤
        let file_pairs = self.prefilter_file_pairs(contents).await?;
        self.performance_stats.file_pairs_analyzed = file_pairs.len();

        if file_pairs.is_empty() {
            return Ok(duplications);
        }

        // 2. 提取所有文件的代码块
        let file_code_blocks = self.extract_cross_file_code_blocks(contents).await?;
        self.performance_stats.code_blocks_extracted = file_code_blocks.values()
            .map(|blocks| blocks.len())
            .sum();

        // 3. 执行跨文件精确重复检测
        let exact_duplications = self.detect_cross_file_exact_duplications(&file_code_blocks).await?;
        log::debug!("Found {} exact cross-file duplications", exact_duplications.len());
        duplications.extend(exact_duplications);

        // 4. 执行跨文件结构相似性检测
        if self.config.enable_structural_detection {
            let structural_duplications = self.detect_cross_file_structural_duplications(&file_code_blocks, contents).await?;
            duplications.extend(structural_duplications);
        }

        // 5. 去重和优化结果
        let optimized_duplications = self.optimize_cross_file_results(duplications).await?;

        // 6. 更新性能统计
        self.performance_stats.total_duplications_found = optimized_duplications.len();
        self.performance_stats.detection_time = start_time.elapsed();

        Ok(optimized_duplications)
    }

    /// 文件级预过滤
    ///
    /// 基于文件大小、语言类型、修改时间等因素快速排除不可能有重复的文件对。
    /// 这个步骤可以显著减少需要详细比较的文件对数量。
    async fn prefilter_file_pairs(&self, contents: &HashMap<String, ProcessedContent>) -> Result<Vec<(String, String)>> {
        let mut file_pairs = Vec::new();
        let files: Vec<&String> = contents.keys().collect();

        // 按语言分组文件
        let mut files_by_language: HashMap<Language, Vec<&String>> = HashMap::new();
        for (file_path, content) in contents {
            files_by_language.entry(content.language.clone()).or_insert_with(Vec::new).push(file_path);
        }

        // 只在相同语言的文件间查找重复
        for (language, language_files) in files_by_language {
            if language_files.len() < 2 {
                continue;
            }

            // 按文件大小分组，相似大小的文件更可能有重复
            let mut size_groups: HashMap<usize, Vec<&String>> = HashMap::new();
            for file_path in &language_files {
                let content = &contents[*file_path];
                let size_bucket = self.calculate_size_bucket(content.processed.len());
                size_groups.entry(size_bucket).or_insert_with(Vec::new).push(file_path);
            }

            // 在每个大小组内生成文件对
            for (_, group_files) in size_groups {
                if group_files.len() < 2 {
                    continue;
                }

                for i in 0..group_files.len() {
                    for j in (i + 1)..group_files.len() {
                        let file1 = group_files[i];
                        let file2 = group_files[j];

                        // 额外的预过滤条件
                        if self.should_compare_files(file1, file2, &contents[file1], &contents[file2]) {
                            file_pairs.push((file1.clone(), file2.clone()));
                        }
                    }
                }
            }
        }

        Ok(file_pairs)
    }

    /// 计算文件大小桶
    fn calculate_size_bucket(&self, size: usize) -> usize {
        // 将文件大小分组到桶中，相似大小的文件在同一桶
        match size {
            0..=1000 => 0,
            1001..=5000 => 1,
            5001..=20000 => 2,
            20001..=100000 => 3,
            _ => 4,
        }
    }

    /// 判断是否应该比较两个文件
    fn should_compare_files(&self, file1: &str, file2: &str, content1: &ProcessedContent, content2: &ProcessedContent) -> bool {
        // 跳过相同文件
        if file1 == file2 {
            return false;
        }

        // 跳过大小差异过大的文件
        let size_ratio = if content1.processed.len() > content2.processed.len() {
            content1.processed.len() as f64 / content2.processed.len() as f64
        } else {
            content2.processed.len() as f64 / content1.processed.len() as f64
        };

        if size_ratio > 3.0 {
            return false;
        }

        // 跳过太小的文件
        if content1.processed.len() < self.config.min_duplicate_chars ||
           content2.processed.len() < self.config.min_duplicate_chars {
            return false;
        }

        true
    }

    /// 提取跨文件代码块
    async fn extract_cross_file_code_blocks(&mut self, contents: &HashMap<String, ProcessedContent>) -> Result<HashMap<String, Vec<CrossFileCodeBlock>>> {
        let mut file_code_blocks = HashMap::new();

        for (file_path, content) in contents {
            // 检查缓存
            if let Some(cached_blocks) = self.cross_file_hash_cache.get(file_path) {
                file_code_blocks.insert(file_path.clone(), cached_blocks.clone());
                continue;
            }

            let blocks = self.extract_code_blocks_from_file(file_path, content).await?;

            // 更新缓存
            self.cross_file_hash_cache.insert(file_path.clone(), blocks.clone());
            file_code_blocks.insert(file_path.clone(), blocks);
        }

        Ok(file_code_blocks)
    }

    /// 从单个文件提取代码块
    async fn extract_code_blocks_from_file(&self, file_path: &str, content: &ProcessedContent) -> Result<Vec<CrossFileCodeBlock>> {
        let lines: Vec<&str> = content.processed.lines().collect();
        let mut blocks = Vec::new();
        let min_lines = self.config.min_duplicate_lines;

        if lines.len() < min_lines {
            return Ok(blocks);
        }

        // 使用滑动窗口提取代码块
        for window_size in min_lines..=lines.len().min(50) { // 限制最大窗口大小以提高性能
            for start_idx in 0..=(lines.len() - window_size) {
                let end_idx = start_idx + window_size - 1;
                let block_lines = &lines[start_idx..=end_idx];
                let block_content = block_lines.join("\n");

                // 跳过不符合要求的代码块
                if !self.is_valid_cross_file_block(&block_content) {
                    continue;
                }

                let hash = self.calculate_normalized_hash(&block_content);
                let structural_signature = self.calculate_structural_signature(&block_content, &content.language)?;

                blocks.push(CrossFileCodeBlock {
                    file_path: file_path.to_string(),
                    start_line: start_idx + 1,
                    end_line: end_idx + 1,
                    content: block_content.clone(),
                    content_hash: hash,
                    structural_signature,
                    language: content.language.clone(),
                    line_count: window_size,
                    char_count: block_content.len(),
                });
            }
        }

        Ok(blocks)
    }

    /// 检查代码块是否适合跨文件检测
    fn is_valid_cross_file_block(&self, content: &str) -> bool {
        // 检查最小大小要求
        if content.len() < self.config.min_duplicate_chars {
            return false;
        }

        // 检查是否有足够的非空行
        let non_empty_lines: Vec<&str> = content
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .collect();

        if non_empty_lines.len() < self.config.min_duplicate_lines {
            return false;
        }

        // 跳过只有注释或导入语句的代码块
        let meaningful_lines = non_empty_lines.iter()
            .filter(|line| {
                !line.starts_with("//") &&
                !line.starts_with("/*") &&
                !line.starts_with("*") &&
                !line.starts_with("#") &&
                !line.starts_with("import ") &&
                !line.starts_with("use ") &&
                !line.starts_with("from ")
            })
            .count();

        meaningful_lines >= (self.config.min_duplicate_lines / 2).max(1)
    }

    /// 计算标准化哈希
    fn calculate_normalized_hash(&self, content: &str) -> String {
        let mut hasher = Sha256::new();

        // 标准化内容：移除多余空白，统一换行符
        let normalized_content = content
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join("\n");

        hasher.update(normalized_content.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// 计算结构签名
    fn calculate_structural_signature(&self, content: &str, language: &Language) -> Result<String> {
        let mut signature_parts = Vec::new();

        // 提取控制流结构
        let control_flow = self.extract_control_flow_signature(content, language)?;
        if !control_flow.is_empty() {
            signature_parts.push(format!("cf:{}", control_flow));
        }

        // 提取函数调用模式
        let function_calls = self.extract_function_call_signature(content, language)?;
        if !function_calls.is_empty() {
            signature_parts.push(format!("fc:{}", function_calls));
        }

        // 提取代码块结构
        let block_structure = self.extract_block_structure_signature(content)?;
        signature_parts.push(format!("bs:{}", block_structure));

        Ok(signature_parts.join("|"))
    }

    /// 提取控制流签名
    fn extract_control_flow_signature(&self, content: &str, language: &Language) -> Result<String> {
        let mut flow_elements = Vec::new();

        for line in content.lines() {
            let trimmed = line.trim();

            match language {
                Language::Rust => {
                    if trimmed.starts_with("if ") || trimmed.contains(" if ") {
                        flow_elements.push("if");
                    } else if trimmed.starts_with("else") {
                        flow_elements.push("else");
                    } else if trimmed.starts_with("for ") {
                        flow_elements.push("for");
                    } else if trimmed.starts_with("while ") {
                        flow_elements.push("while");
                    } else if trimmed.starts_with("match ") {
                        flow_elements.push("match");
                    } else if trimmed.starts_with("loop") {
                        flow_elements.push("loop");
                    }
                }
                Language::Go => {
                    if trimmed.starts_with("if ") {
                        flow_elements.push("if");
                    } else if trimmed.starts_with("else") {
                        flow_elements.push("else");
                    } else if trimmed.starts_with("for ") {
                        flow_elements.push("for");
                    } else if trimmed.starts_with("switch ") {
                        flow_elements.push("switch");
                    } else if trimmed.starts_with("select") {
                        flow_elements.push("select");
                    }
                }
                Language::TypeScript | Language::JavaScript => {
                    if trimmed.starts_with("if ") || trimmed.contains(" if ") {
                        flow_elements.push("if");
                    } else if trimmed.starts_with("else") {
                        flow_elements.push("else");
                    } else if trimmed.starts_with("for ") {
                        flow_elements.push("for");
                    } else if trimmed.starts_with("while ") {
                        flow_elements.push("while");
                    } else if trimmed.starts_with("switch ") {
                        flow_elements.push("switch");
                    } else if trimmed.starts_with("try") {
                        flow_elements.push("try");
                    } else if trimmed.starts_with("catch") {
                        flow_elements.push("catch");
                    }
                }
                _ => {
                    // 通用控制流检测
                    if trimmed.contains("if") && (trimmed.contains("(") || trimmed.contains("{")) {
                        flow_elements.push("if");
                    } else if trimmed.contains("for") && (trimmed.contains("(") || trimmed.contains("{")) {
                        flow_elements.push("for");
                    } else if trimmed.contains("while") && (trimmed.contains("(") || trimmed.contains("{")) {
                        flow_elements.push("while");
                    }
                }
            }
        }

        Ok(flow_elements.join(","))
    }

    /// 提取函数调用签名
    fn extract_function_call_signature(&self, content: &str, language: &Language) -> Result<String> {
        let call_pattern = regex::Regex::new(r"(\w+)\s*\(")?;
        let mut function_calls = Vec::new();

        for line in content.lines() {
            for cap in call_pattern.captures_iter(line) {
                if let Some(func_name) = cap.get(1) {
                    let name = func_name.as_str();
                    // 过滤掉常见的关键字
                    if !matches!(name, "if" | "for" | "while" | "match" | "switch" | "try" | "catch") {
                        function_calls.push(name);
                    }
                }
            }
        }

        // 去重并排序
        function_calls.sort();
        function_calls.dedup();

        Ok(function_calls.join(","))
    }

    /// 提取代码块结构签名
    fn extract_block_structure_signature(&self, content: &str) -> Result<String> {
        let mut brace_depth: i32 = 0;
        let mut max_brace_depth: i32 = 0;
        let mut paren_depth: i32 = 0;
        let mut max_paren_depth: i32 = 0;
        let mut bracket_depth: i32 = 0;
        let mut max_bracket_depth: i32 = 0;

        for ch in content.chars() {
            match ch {
                '{' => {
                    brace_depth += 1;
                    max_brace_depth = max_brace_depth.max(brace_depth);
                }
                '}' => {
                    brace_depth = brace_depth.saturating_sub(1);
                }
                '(' => {
                    paren_depth += 1;
                    max_paren_depth = max_paren_depth.max(paren_depth);
                }
                ')' => {
                    paren_depth = paren_depth.saturating_sub(1);
                }
                '[' => {
                    bracket_depth += 1;
                    max_bracket_depth = max_bracket_depth.max(bracket_depth);
                }
                ']' => {
                    bracket_depth = bracket_depth.saturating_sub(1);
                }
                _ => {}
            }
        }

        Ok(format!("{}:{}:{}", max_brace_depth.max(0), max_paren_depth.max(0), max_bracket_depth.max(0)))
    }

    /// 检测跨文件精确重复
    async fn detect_cross_file_exact_duplications(&self, file_code_blocks: &HashMap<String, Vec<CrossFileCodeBlock>>) -> Result<Vec<CodeDuplication>> {
        let mut duplications = Vec::new();
        let mut hash_to_blocks: HashMap<String, Vec<&CrossFileCodeBlock>> = HashMap::new();

        // 收集所有代码块按哈希分组
        for blocks in file_code_blocks.values() {
            for block in blocks {
                hash_to_blocks.entry(block.content_hash.clone()).or_insert_with(Vec::new).push(block);
            }
        }

        // 查找跨文件的重复哈希
        for (hash, blocks) in hash_to_blocks {
            if blocks.len() < 2 {
                continue;
            }

            // 检查是否真的是跨文件重复
            let unique_files: std::collections::HashSet<&String> = blocks.iter()
                .map(|block| &block.file_path)
                .collect();

            if unique_files.len() < 2 {
                continue; // 不是跨文件重复
            }

            // 创建跨文件重复对象
            let duplication = self.create_cross_file_duplication(hash, blocks, DuplicationType::CrossFile)?;
            duplications.push(duplication);
        }

        Ok(duplications)
    }

    /// 检测跨文件结构相似性重复
    async fn detect_cross_file_structural_duplications(
        &self,
        file_code_blocks: &HashMap<String, Vec<CrossFileCodeBlock>>,
        contents: &HashMap<String, ProcessedContent>
    ) -> Result<Vec<CodeDuplication>> {
        let mut duplications = Vec::new();
        let mut signature_to_blocks: HashMap<String, Vec<&CrossFileCodeBlock>> = HashMap::new();

        // 收集所有代码块按结构签名分组
        for blocks in file_code_blocks.values() {
            for block in blocks {
                if !block.structural_signature.is_empty() {
                    signature_to_blocks.entry(block.structural_signature.clone()).or_insert_with(Vec::new).push(block);
                }
            }
        }

        // 查找跨文件的结构相似重复
        for (signature, blocks) in signature_to_blocks {
            if blocks.len() < 2 {
                continue;
            }

            // 检查是否真的是跨文件重复
            let unique_files: std::collections::HashSet<&String> = blocks.iter()
                .map(|block| &block.file_path)
                .collect();

            if unique_files.len() < 2 {
                continue; // 不是跨文件重复
            }

            // 计算详细的相似度
            let similar_groups = self.group_blocks_by_detailed_similarity(blocks, contents).await?;

            for group in similar_groups {
                if group.len() >= 2 {
                    let avg_similarity = self.calculate_average_similarity(&group, contents).await?;
                    if avg_similarity >= self.config.structural_similarity_threshold {
                        let duplication = self.create_cross_file_structural_duplication(signature.clone(), group, avg_similarity)?;
                        duplications.push(duplication);
                    }
                }
            }
        }

        Ok(duplications)
    }

    /// 按详细相似度分组代码块
    async fn group_blocks_by_detailed_similarity<'a>(
        &self,
        blocks: Vec<&'a CrossFileCodeBlock>,
        contents: &HashMap<String, ProcessedContent>
    ) -> Result<Vec<Vec<&'a CrossFileCodeBlock>>> {
        let mut groups = Vec::new();
        let mut remaining_blocks = blocks;

        while !remaining_blocks.is_empty() {
            let seed_block = remaining_blocks.remove(0);
            let mut current_group = vec![seed_block];

            // 查找与种子块相似的其他块
            let mut i = 0;
            while i < remaining_blocks.len() {
                let candidate_block = remaining_blocks[i];
                let similarity = self.calculate_detailed_similarity(seed_block, candidate_block, contents).await?;

                if similarity >= self.config.structural_similarity_threshold {
                    current_group.push(candidate_block);
                    remaining_blocks.remove(i);
                } else {
                    i += 1;
                }
            }

            if current_group.len() >= 2 {
                groups.push(current_group);
            }
        }

        Ok(groups)
    }

    /// 计算详细相似度
    async fn calculate_detailed_similarity(
        &self,
        block1: &CrossFileCodeBlock,
        block2: &CrossFileCodeBlock,
        contents: &HashMap<String, ProcessedContent>
    ) -> Result<f64> {
        // 如果哈希相同，则完全相似
        if block1.content_hash == block2.content_hash {
            return Ok(1.0);
        }

        // 如果结构签名不同，相似度较低
        if block1.structural_signature != block2.structural_signature {
            return Ok(0.0);
        }

        // 计算基于内容的相似度
        let content_similarity = self.calculate_content_similarity(&block1.content, &block2.content)?;

        // 计算基于语言特性的相似度
        let language_similarity = if block1.language == block2.language {
            self.calculate_language_specific_similarity(block1, block2, contents).await?
        } else {
            0.5 // 不同语言的相似度降低
        };

        // 综合相似度
        let combined_similarity = (content_similarity * 0.6) + (language_similarity * 0.4);

        Ok(combined_similarity)
    }

    /// 计算内容相似度
    fn calculate_content_similarity(&self, content1: &str, content2: &str) -> Result<f64> {
        let lines1: Vec<&str> = content1.lines().map(|l| l.trim()).filter(|l| !l.is_empty()).collect();
        let lines2: Vec<&str> = content2.lines().map(|l| l.trim()).filter(|l| !l.is_empty()).collect();

        if lines1.is_empty() || lines2.is_empty() {
            return Ok(0.0);
        }

        // 使用最长公共子序列算法计算相似度
        let lcs_length = self.longest_common_subsequence(&lines1, &lines2);
        let max_length = lines1.len().max(lines2.len());

        Ok(lcs_length as f64 / max_length as f64)
    }

    /// 最长公共子序列算法
    fn longest_common_subsequence(&self, seq1: &[&str], seq2: &[&str]) -> usize {
        let m = seq1.len();
        let n = seq2.len();
        let mut dp = vec![vec![0; n + 1]; m + 1];

        for i in 1..=m {
            for j in 1..=n {
                if seq1[i - 1] == seq2[j - 1] {
                    dp[i][j] = dp[i - 1][j - 1] + 1;
                } else {
                    dp[i][j] = dp[i - 1][j].max(dp[i][j - 1]);
                }
            }
        }

        dp[m][n]
    }

    /// 计算语言特定相似度
    async fn calculate_language_specific_similarity(
        &self,
        block1: &CrossFileCodeBlock,
        block2: &CrossFileCodeBlock,
        _contents: &HashMap<String, ProcessedContent>
    ) -> Result<f64> {
        // 基于语言特性的相似度计算
        match block1.language {
            Language::Rust => self.calculate_rust_similarity(block1, block2),
            Language::Go => self.calculate_go_similarity(block1, block2),
            Language::TypeScript | Language::JavaScript => self.calculate_typescript_similarity(block1, block2),
            _ => Ok(0.7), // 默认相似度
        }
    }

    /// 计算Rust特定相似度
    fn calculate_rust_similarity(&self, block1: &CrossFileCodeBlock, block2: &CrossFileCodeBlock) -> Result<f64> {
        let mut similarity_score = 0.0;
        let mut total_features = 0;

        // 检查函数定义模式
        let fn_pattern = regex::Regex::new(r"fn\s+\w+")?;
        let fn_count1 = fn_pattern.find_iter(&block1.content).count();
        let fn_count2 = fn_pattern.find_iter(&block2.content).count();
        if fn_count1 > 0 || fn_count2 > 0 {
            similarity_score += if fn_count1 == fn_count2 { 1.0 } else { 0.5 };
            total_features += 1;
        }

        // 检查结构体定义
        let struct_pattern = regex::Regex::new(r"struct\s+\w+")?;
        let struct_count1 = struct_pattern.find_iter(&block1.content).count();
        let struct_count2 = struct_pattern.find_iter(&block2.content).count();
        if struct_count1 > 0 || struct_count2 > 0 {
            similarity_score += if struct_count1 == struct_count2 { 1.0 } else { 0.5 };
            total_features += 1;
        }

        // 检查match表达式
        let match_count1 = block1.content.matches("match ").count();
        let match_count2 = block2.content.matches("match ").count();
        if match_count1 > 0 || match_count2 > 0 {
            similarity_score += if match_count1 == match_count2 { 1.0 } else { 0.5 };
            total_features += 1;
        }

        if total_features == 0 {
            return Ok(0.7);
        }

        Ok(similarity_score / total_features as f64)
    }

    /// 计算Go特定相似度
    fn calculate_go_similarity(&self, block1: &CrossFileCodeBlock, block2: &CrossFileCodeBlock) -> Result<f64> {
        let mut similarity_score = 0.0;
        let mut total_features = 0;

        // 检查函数定义模式
        let func_pattern = regex::Regex::new(r"func\s+\w+")?;
        let func_count1 = func_pattern.find_iter(&block1.content).count();
        let func_count2 = func_pattern.find_iter(&block2.content).count();
        if func_count1 > 0 || func_count2 > 0 {
            similarity_score += if func_count1 == func_count2 { 1.0 } else { 0.5 };
            total_features += 1;
        }

        // 检查接口定义
        let interface_pattern = regex::Regex::new(r"type\s+\w+\s+interface")?;
        let interface_count1 = interface_pattern.find_iter(&block1.content).count();
        let interface_count2 = interface_pattern.find_iter(&block2.content).count();
        if interface_count1 > 0 || interface_count2 > 0 {
            similarity_score += if interface_count1 == interface_count2 { 1.0 } else { 0.5 };
            total_features += 1;
        }

        // 检查goroutine使用
        let go_count1 = block1.content.matches("go ").count();
        let go_count2 = block2.content.matches("go ").count();
        if go_count1 > 0 || go_count2 > 0 {
            similarity_score += if go_count1 == go_count2 { 1.0 } else { 0.5 };
            total_features += 1;
        }

        if total_features == 0 {
            return Ok(0.7);
        }

        Ok(similarity_score / total_features as f64)
    }

    /// 计算TypeScript特定相似度
    fn calculate_typescript_similarity(&self, block1: &CrossFileCodeBlock, block2: &CrossFileCodeBlock) -> Result<f64> {
        let mut similarity_score = 0.0;
        let mut total_features = 0;

        // 检查函数定义模式
        let func_pattern = regex::Regex::new(r"function\s+\w+|const\s+\w+\s*=\s*\(|async\s+function")?;
        let func_count1 = func_pattern.find_iter(&block1.content).count();
        let func_count2 = func_pattern.find_iter(&block2.content).count();
        if func_count1 > 0 || func_count2 > 0 {
            similarity_score += if func_count1 == func_count2 { 1.0 } else { 0.5 };
            total_features += 1;
        }

        // 检查类定义
        let class_pattern = regex::Regex::new(r"class\s+\w+")?;
        let class_count1 = class_pattern.find_iter(&block1.content).count();
        let class_count2 = class_pattern.find_iter(&block2.content).count();
        if class_count1 > 0 || class_count2 > 0 {
            similarity_score += if class_count1 == class_count2 { 1.0 } else { 0.5 };
            total_features += 1;
        }

        // 检查异步模式
        let async_count1 = block1.content.matches("await ").count() + block1.content.matches("async ").count();
        let async_count2 = block2.content.matches("await ").count() + block2.content.matches("async ").count();
        if async_count1 > 0 || async_count2 > 0 {
            similarity_score += if async_count1 == async_count2 { 1.0 } else { 0.5 };
            total_features += 1;
        }

        if total_features == 0 {
            return Ok(0.7);
        }

        Ok(similarity_score / total_features as f64)
    }

    /// 计算平均相似度
    async fn calculate_average_similarity(
        &self,
        blocks: &[&CrossFileCodeBlock],
        contents: &HashMap<String, ProcessedContent>
    ) -> Result<f64> {
        if blocks.len() < 2 {
            return Ok(0.0);
        }

        let mut total_similarity = 0.0;
        let mut comparison_count = 0;

        for i in 0..blocks.len() {
            for j in (i + 1)..blocks.len() {
                let similarity = self.calculate_detailed_similarity(blocks[i], blocks[j], contents).await?;
                total_similarity += similarity;
                comparison_count += 1;
            }
        }

        if comparison_count == 0 {
            return Ok(0.0);
        }

        Ok(total_similarity / comparison_count as f64)
    }

    /// 创建跨文件重复对象
    fn create_cross_file_duplication(
        &self,
        hash: String,
        blocks: Vec<&CrossFileCodeBlock>,
        duplication_type: DuplicationType
    ) -> Result<CodeDuplication> {
        if blocks.is_empty() {
            anyhow::bail!("Cannot create duplication from empty blocks");
        }

        let first_block = blocks[0];
        let line_count = first_block.line_count;
        let similarity_score = if duplication_type == DuplicationType::CrossFile { 1.0 } else { 0.8 };
        let risk_level = RiskLevel::assess(line_count, similarity_score);
        let refactoring_priority = RefactoringPriority::assess(risk_level, blocks.len());

        let code_blocks: Vec<CodeBlock> = blocks
            .iter()
            .map(|block| CodeBlock {
                file_path: block.file_path.clone(),
                start_line: block.start_line,
                end_line: block.end_line,
                start_column: None,
                end_column: None,
                content_hash: block.content_hash.clone(),
            })
            .collect();

        Ok(CodeDuplication {
            id: uuid::Uuid::new_v4().to_string(),
            duplication_type,
            code_blocks,
            content: first_block.content.clone(),
            line_count,
            similarity_score,
            risk_level,
            refactoring_priority,
        })
    }

    /// 创建跨文件结构相似重复对象
    fn create_cross_file_structural_duplication(
        &self,
        signature: String,
        blocks: Vec<&CrossFileCodeBlock>,
        similarity_score: f64
    ) -> Result<CodeDuplication> {
        if blocks.is_empty() {
            anyhow::bail!("Cannot create structural duplication from empty blocks");
        }

        let first_block = blocks[0];
        let line_count = first_block.line_count;
        let risk_level = RiskLevel::assess(line_count, similarity_score);
        let refactoring_priority = RefactoringPriority::assess(risk_level, blocks.len());

        let code_blocks: Vec<CodeBlock> = blocks
            .iter()
            .map(|block| CodeBlock {
                file_path: block.file_path.clone(),
                start_line: block.start_line,
                end_line: block.end_line,
                start_column: None,
                end_column: None,
                content_hash: format!("structural:{}", signature),
            })
            .collect();

        Ok(CodeDuplication {
            id: uuid::Uuid::new_v4().to_string(),
            duplication_type: DuplicationType::CrossFile,
            code_blocks,
            content: first_block.content.clone(),
            line_count,
            similarity_score,
            risk_level,
            refactoring_priority,
        })
    }

    /// 优化跨文件检测结果
    async fn optimize_cross_file_results(&self, duplications: Vec<CodeDuplication>) -> Result<Vec<CodeDuplication>> {
        let mut optimized = duplications;

        // 去重相似的重复块
        optimized = self.deduplicate_similar_results(optimized)?;

        // 按风险等级和相似度排序
        optimized.sort_by(|a, b| {
            b.risk_level.cmp(&a.risk_level)
                .then_with(|| b.similarity_score.partial_cmp(&a.similarity_score).unwrap_or(Ordering::Equal))
                .then_with(|| b.line_count.cmp(&a.line_count))
        });

        Ok(optimized)
    }

    /// 去重相似的检测结果
    fn deduplicate_similar_results(&self, duplications: Vec<CodeDuplication>) -> Result<Vec<CodeDuplication>> {
        let mut deduplicated = Vec::new();

        for duplication in duplications {
            let mut is_duplicate = false;

            for existing in &deduplicated {
                if self.are_duplications_similar(&duplication, existing) {
                    is_duplicate = true;
                    break;
                }
            }

            if !is_duplicate {
                deduplicated.push(duplication);
            }
        }

        Ok(deduplicated)
    }

    /// 判断两个重复检测结果是否相似
    fn are_duplications_similar(&self, dup1: &CodeDuplication, dup2: &CodeDuplication) -> bool {
        // 检查是否有重叠的代码块
        for block1 in &dup1.code_blocks {
            for block2 in &dup2.code_blocks {
                if block1.file_path == block2.file_path {
                    // 检查行号是否重叠
                    let overlap = !(block1.end_line < block2.start_line || block2.end_line < block1.start_line);
                    if overlap {
                        // 计算重叠程度
                        let overlap_start = block1.start_line.max(block2.start_line);
                        let overlap_end = block1.end_line.min(block2.end_line);
                        let overlap_lines = overlap_end.saturating_sub(overlap_start) + 1;
                        let total_lines = (block1.end_line - block1.start_line + 1).max(block2.end_line - block2.start_line + 1);

                        if overlap_lines as f64 / total_lines as f64 > 0.8 {
                            return true;
                        }
                    }
                }
            }
        }

        false
    }

    /// 清空缓存
    pub fn clear_cache(&mut self) {
        self.cross_file_hash_cache.clear();
        self.cross_file_pattern_cache.clear();
        self.performance_stats = CrossFilePerformanceStats::new();
    }

    /// 获取性能统计
    pub fn get_performance_stats(&self) -> &CrossFilePerformanceStats {
        &self.performance_stats
    }

    /// 获取缓存统计
    pub fn get_cache_stats(&self) -> CrossFileCacheStats {
        CrossFileCacheStats {
            hash_cache_size: self.cross_file_hash_cache.len(),
            pattern_cache_size: self.cross_file_pattern_cache.len(),
            hash_cache_memory: self.cross_file_hash_cache
                .iter()
                .map(|(k, v)| k.len() + v.iter().map(|b| b.content.len()).sum::<usize>())
                .sum(),
            pattern_cache_memory: self.cross_file_pattern_cache
                .iter()
                .map(|(k, v)| k.len() + v.len() * 100) // 估算模式大小
                .sum(),
        }
    }
}

/// 跨文件代码块
#[derive(Debug, Clone)]
struct CrossFileCodeBlock {
    file_path: String,
    start_line: usize,
    end_line: usize,
    content: String,
    content_hash: String,
    structural_signature: String,
    language: Language,
    line_count: usize,
    char_count: usize,
}

/// 跨文件结构模式
#[derive(Debug, Clone)]
struct CrossFileStructuralPattern {
    signature: String,
    file_path: String,
    start_line: usize,
    end_line: usize,
    complexity_score: f64,
}

/// 跨文件性能统计
#[derive(Debug, Clone)]
pub struct CrossFilePerformanceStats {
    pub file_pairs_analyzed: usize,
    pub code_blocks_extracted: usize,
    pub total_duplications_found: usize,
    pub detection_time: std::time::Duration,
}

impl CrossFilePerformanceStats {
    fn new() -> Self {
        Self {
            file_pairs_analyzed: 0,
            code_blocks_extracted: 0,
            total_duplications_found: 0,
            detection_time: std::time::Duration::from_secs(0),
        }
    }
}

/// 跨文件缓存统计
#[derive(Debug, Clone)]
pub struct CrossFileCacheStats {
    pub hash_cache_size: usize,
    pub pattern_cache_size: usize,
    pub hash_cache_memory: usize,
    pub pattern_cache_memory: usize,
}

/// 重构建议生成器
///
/// 基于重复检测结果生成具体的重构建议，包括：
/// - 重构方案分析和推荐
/// - 优先级评估和排序
/// - 代码示例和实施指导
/// - 预期收益评估
pub struct RefactoringSuggestionGenerator {
    /// 语言特定的重构模式
    language_patterns: HashMap<Language, Vec<RefactoringPattern>>,
}

/// 重构模式定义
#[derive(Debug, Clone)]
struct RefactoringPattern {
    /// 模式名称
    name: String,
    /// 适用的重复类型
    applicable_types: Vec<DuplicationType>,
    /// 最小行数阈值
    min_lines: usize,
    /// 最大行数阈值
    max_lines: usize,
    /// 建议类型
    suggestion_type: SuggestionType,
    /// 复杂度评估函数
    complexity_evaluator: fn(usize, f64, usize) -> ComplexityLevel,
}

impl RefactoringSuggestionGenerator {
    /// 创建新的重构建议生成器
    pub fn new() -> Self {
        let mut generator = Self {
            language_patterns: HashMap::new(),
        };
        generator.initialize_patterns();
        generator
    }

    /// 初始化语言特定的重构模式
    fn initialize_patterns(&mut self) {
        // Rust 语言重构模式
        let rust_patterns = vec![
            RefactoringPattern {
                name: "提取常量".to_string(),
                applicable_types: vec![DuplicationType::Exact],
                min_lines: 1,
                max_lines: 8,
                suggestion_type: SuggestionType::ExtractConstant,
                complexity_evaluator: |lines, _, _| if lines <= 3 { ComplexityLevel::Simple } else { ComplexityLevel::Moderate },
            },
            RefactoringPattern {
                name: "提取函数".to_string(),
                applicable_types: vec![DuplicationType::Exact, DuplicationType::Structural],
                min_lines: 8,
                max_lines: 30,
                suggestion_type: SuggestionType::ExtractMethod,
                complexity_evaluator: |lines, similarity, blocks| {
                    match (lines, similarity, blocks) {
                        (l, _, _) if l <= 10 => ComplexityLevel::Simple,
                        (l, s, _) if l <= 20 && s > 0.9 => ComplexityLevel::Moderate,
                        _ => ComplexityLevel::Complex,
                    }
                },
            },
            RefactoringPattern {
                name: "提取模块".to_string(),
                applicable_types: vec![DuplicationType::CrossFile, DuplicationType::Structural],
                min_lines: 20,
                max_lines: 100,
                suggestion_type: SuggestionType::ExtractClass,
                complexity_evaluator: |lines, _, blocks| {
                    match (lines, blocks) {
                        (l, b) if l <= 50 && b <= 3 => ComplexityLevel::Moderate,
                        (l, b) if l <= 80 && b <= 5 => ComplexityLevel::Complex,
                        _ => ComplexityLevel::VeryComplex,
                    }
                },
            },
            RefactoringPattern {
                name: "使用 trait 抽象".to_string(),
                applicable_types: vec![DuplicationType::Structural],
                min_lines: 30,
                max_lines: 200,
                suggestion_type: SuggestionType::TemplateMethod,
                complexity_evaluator: |_, _, _| ComplexityLevel::Complex,
            },
            RefactoringPattern {
                name: "创建工具模块".to_string(),
                applicable_types: vec![DuplicationType::CrossFile],
                min_lines: 10,
                max_lines: 50,
                suggestion_type: SuggestionType::CreateUtilityClass,
                complexity_evaluator: |lines, _, blocks| {
                    if blocks >= 3 && lines >= 15 { ComplexityLevel::Moderate } else { ComplexityLevel::Simple }
                },
            },
        ];
        self.language_patterns.insert(Language::Rust, rust_patterns);

        // Go 语言重构模式
        let go_patterns = vec![
            RefactoringPattern {
                name: "提取常量".to_string(),
                applicable_types: vec![DuplicationType::Exact],
                min_lines: 1,
                max_lines: 8,
                suggestion_type: SuggestionType::ExtractConstant,
                complexity_evaluator: |lines, _, _| if lines <= 3 { ComplexityLevel::Simple } else { ComplexityLevel::Moderate },
            },
            RefactoringPattern {
                name: "提取函数".to_string(),
                applicable_types: vec![DuplicationType::Exact, DuplicationType::Structural],
                min_lines: 8,
                max_lines: 40,
                suggestion_type: SuggestionType::ExtractMethod,
                complexity_evaluator: |lines, similarity, _| {
                    match (lines, similarity) {
                        (l, _) if l <= 15 => ComplexityLevel::Simple,
                        (l, s) if l <= 25 && s > 0.85 => ComplexityLevel::Moderate,
                        _ => ComplexityLevel::Complex,
                    }
                },
            },
            RefactoringPattern {
                name: "提取包".to_string(),
                applicable_types: vec![DuplicationType::CrossFile],
                min_lines: 25,
                max_lines: 150,
                suggestion_type: SuggestionType::ExtractClass,
                complexity_evaluator: |lines, _, _| {
                    if lines <= 60 { ComplexityLevel::Moderate } else { ComplexityLevel::Complex }
                },
            },
            RefactoringPattern {
                name: "使用接口抽象".to_string(),
                applicable_types: vec![DuplicationType::Structural],
                min_lines: 20,
                max_lines: 100,
                suggestion_type: SuggestionType::StrategyPattern,
                complexity_evaluator: |_, _, _| ComplexityLevel::Complex,
            },
        ];
        self.language_patterns.insert(Language::Go, go_patterns);

        // TypeScript 语言重构模式
        let typescript_patterns = vec![
            RefactoringPattern {
                name: "提取常量".to_string(),
                applicable_types: vec![DuplicationType::Exact],
                min_lines: 1,
                max_lines: 5,
                suggestion_type: SuggestionType::ExtractConstant,
                complexity_evaluator: |lines, _, _| if lines <= 3 { ComplexityLevel::Simple } else { ComplexityLevel::Moderate },
            },
            RefactoringPattern {
                name: "提取函数".to_string(),
                applicable_types: vec![DuplicationType::Exact, DuplicationType::Structural],
                min_lines: 5,
                max_lines: 35,
                suggestion_type: SuggestionType::ExtractMethod,
                complexity_evaluator: |lines, similarity, _| {
                    match (lines, similarity) {
                        (l, _) if l <= 12 => ComplexityLevel::Simple,
                        (l, s) if l <= 25 && s > 0.9 => ComplexityLevel::Moderate,
                        _ => ComplexityLevel::Complex,
                    }
                },
            },
            RefactoringPattern {
                name: "提取类".to_string(),
                applicable_types: vec![DuplicationType::CrossFile, DuplicationType::Structural],
                min_lines: 20,
                max_lines: 120,
                suggestion_type: SuggestionType::ExtractClass,
                complexity_evaluator: |lines, _, blocks| {
                    match (lines, blocks) {
                        (l, b) if l <= 40 && b <= 3 => ComplexityLevel::Moderate,
                        _ => ComplexityLevel::Complex,
                    }
                },
            },
            RefactoringPattern {
                name: "使用策略模式".to_string(),
                applicable_types: vec![DuplicationType::Structural],
                min_lines: 25,
                max_lines: 100,
                suggestion_type: SuggestionType::StrategyPattern,
                complexity_evaluator: |_, _, _| ComplexityLevel::Complex,
            },
        ];
        self.language_patterns.insert(Language::TypeScript, typescript_patterns);
    }

    /// 生成重构建议
    pub async fn generate_suggestions(&self, duplications: &[CodeDuplication]) -> Result<Vec<RefactoringSuggestion>> {
        let mut suggestions = Vec::new();

        // 按优先级排序重复代码
        let mut sorted_duplications = duplications.to_vec();
        sorted_duplications.sort_by(|a, b| {
            self.calculate_refactoring_priority(a)
                .cmp(&self.calculate_refactoring_priority(b))
                .reverse()
        });

        for duplication in &sorted_duplications {
            if let Some(suggestion) = self.generate_suggestion_for_duplication(duplication).await? {
                suggestions.push(suggestion);
            }
        }

        // 生成组合重构建议
        let combined_suggestions = self.generate_combined_suggestions(&sorted_duplications).await?;
        suggestions.extend(combined_suggestions);

        Ok(suggestions)
    }

    /// 为单个重复代码生成建议
    async fn generate_suggestion_for_duplication(&self, duplication: &CodeDuplication) -> Result<Option<RefactoringSuggestion>> {
        // 检测主要语言
        let primary_language = self.detect_primary_language(duplication);

        // 获取适用的重构模式
        let applicable_patterns = self.find_applicable_patterns(&primary_language, duplication);

        if applicable_patterns.is_empty() {
            return Ok(None);
        }

        // 选择最佳模式
        let best_pattern = self.select_best_pattern(&applicable_patterns, duplication);

        // 生成具体建议
        let suggestion = self.create_suggestion_from_pattern(duplication, &best_pattern, &primary_language).await?;

        Ok(Some(suggestion))
    }

    /// 检测重复代码的主要语言
    fn detect_primary_language(&self, duplication: &CodeDuplication) -> Language {
        // 统计各语言的文件数量
        let mut language_counts: HashMap<Language, usize> = HashMap::new();

        for block in &duplication.code_blocks {
            let language = Language::from_extension(&block.file_path).unwrap_or(Language::Unknown);
            *language_counts.entry(language).or_insert(0) += 1;
        }

        // 返回出现最多的语言
        language_counts
            .into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(lang, _)| lang)
            .unwrap_or(Language::Unknown)
    }

    /// 查找适用的重构模式
    fn find_applicable_patterns(&self, language: &Language, duplication: &CodeDuplication) -> Vec<RefactoringPattern> {
        let empty_patterns = vec![];
        let patterns = self.language_patterns.get(language).unwrap_or(&empty_patterns);

        patterns
            .iter()
            .filter(|pattern| {
                pattern.applicable_types.contains(&duplication.duplication_type) &&
                duplication.line_count >= pattern.min_lines &&
                duplication.line_count <= pattern.max_lines
            })
            .cloned()
            .collect()
    }

    /// 选择最佳重构模式
    fn select_best_pattern(&self, patterns: &[RefactoringPattern], duplication: &CodeDuplication) -> RefactoringPattern {
        // 根据重复代码特征选择最合适的模式
        patterns
            .iter()
            .min_by_key(|pattern| {
                // 计算模式适配度分数（越小越好）
                let size_diff = if duplication.line_count < pattern.min_lines {
                    pattern.min_lines - duplication.line_count
                } else if duplication.line_count > pattern.max_lines {
                    duplication.line_count - pattern.max_lines
                } else {
                    0
                };

                let type_match_score = if pattern.applicable_types.contains(&duplication.duplication_type) { 0 } else { 100 };

                size_diff + type_match_score
            })
            .cloned()
            .unwrap_or_else(|| patterns[0].clone())
    }

    /// 从模式创建具体建议
    async fn create_suggestion_from_pattern(
        &self,
        duplication: &CodeDuplication,
        pattern: &RefactoringPattern,
        language: &Language,
    ) -> Result<RefactoringSuggestion> {
        let complexity = (pattern.complexity_evaluator)(
            duplication.line_count,
            duplication.similarity_score,
            duplication.code_blocks.len(),
        );

        let (title, description, approach) = self.generate_suggestion_content(
            &pattern.suggestion_type,
            duplication,
            language,
        );

        let expected_benefits = self.calculate_expected_benefits(duplication, &pattern.suggestion_type);
        let code_example = self.generate_code_example(duplication, &pattern.suggestion_type, language).await?;
        let resources = self.get_relevant_resources(&pattern.suggestion_type, language);

        Ok(RefactoringSuggestion {
            id: uuid::Uuid::new_v4().to_string(),
            duplication_id: duplication.id.clone(),
            suggestion_type: pattern.suggestion_type,
            title,
            description,
            refactoring_approach: approach,
            expected_benefits,
            implementation_complexity: complexity,
            code_example,
            resources,
        })
    }

    /// 生成建议内容
    fn generate_suggestion_content(
        &self,
        suggestion_type: &SuggestionType,
        duplication: &CodeDuplication,
        language: &Language,
    ) -> (String, String, String) {
        match (suggestion_type, language) {
            (SuggestionType::ExtractConstant, Language::Rust) => (
                "提取常量到 const 声明".to_string(),
                format!("发现 {} 处重复的常量值，建议提取为 const 常量", duplication.code_blocks.len()),
                "使用 `const CONSTANT_NAME: Type = value;` 定义常量，替换所有重复的字面值".to_string(),
            ),
            (SuggestionType::ExtractMethod, Language::Rust) => (
                "提取函数".to_string(),
                format!("发现 {} 行重复代码，建议提取为独立函数", duplication.line_count),
                "创建新函数封装重复逻辑，使用适当的参数和返回值，在原位置调用该函数".to_string(),
            ),
            (SuggestionType::ExtractClass, Language::Rust) => (
                "提取模块或结构体".to_string(),
                format!("发现大段重复代码（{} 行），建议提取为独立模块", duplication.line_count),
                "创建新的模块或结构体来封装相关功能，使用 pub 关键字暴露必要的接口".to_string(),
            ),
            (SuggestionType::TemplateMethod, Language::Rust) => (
                "使用 trait 定义通用行为".to_string(),
                "发现结构相似的代码，建议使用 trait 抽象共同行为".to_string(),
                "定义 trait 描述通用行为，为不同类型实现具体逻辑，消除结构重复".to_string(),
            ),
            (SuggestionType::CreateUtilityClass, Language::Rust) => (
                "创建工具模块".to_string(),
                format!("发现跨文件重复的工具函数，建议创建专用工具模块"),
                "创建 utils 模块，将通用函数集中管理，通过 use 语句导入使用".to_string(),
            ),
            (SuggestionType::ExtractConstant, Language::Go) => (
                "提取常量到 const 块".to_string(),
                format!("发现 {} 处重复的常量值，建议提取为 const 常量", duplication.code_blocks.len()),
                "使用 `const ConstantName = value` 或 const 块定义常量".to_string(),
            ),
            (SuggestionType::ExtractMethod, Language::Go) => (
                "提取函数".to_string(),
                format!("发现 {} 行重复代码，建议提取为独立函数", duplication.line_count),
                "创建新函数封装重复逻辑，使用适当的参数和返回值".to_string(),
            ),
            (SuggestionType::ExtractClass, Language::Go) => (
                "提取包".to_string(),
                format!("发现大段重复代码（{} 行），建议提取为独立包", duplication.line_count),
                "创建新的包来封装相关功能，使用大写字母开头的标识符暴露公共接口".to_string(),
            ),
            (SuggestionType::StrategyPattern, Language::Go) => (
                "使用接口抽象".to_string(),
                "发现结构相似的代码，建议使用接口抽象共同行为".to_string(),
                "定义接口描述通用行为，为不同类型实现接口方法".to_string(),
            ),
            (SuggestionType::ExtractConstant, Language::TypeScript) => (
                "提取常量".to_string(),
                format!("发现 {} 处重复的常量值，建议提取为 const 常量", duplication.code_blocks.len()),
                "使用 `const CONSTANT_NAME = value` 定义常量，考虑使用 enum 或 readonly 对象".to_string(),
            ),
            (SuggestionType::ExtractMethod, Language::TypeScript) => (
                "提取函数".to_string(),
                format!("发现 {} 行重复代码，建议提取为独立函数", duplication.line_count),
                "创建新函数封装重复逻辑，使用 TypeScript 类型注解确保类型安全".to_string(),
            ),
            (SuggestionType::ExtractClass, Language::TypeScript) => (
                "提取类".to_string(),
                format!("发现大段重复代码（{} 行），建议提取为独立类", duplication.line_count),
                "创建新类封装相关功能，使用访问修饰符控制可见性".to_string(),
            ),
            (SuggestionType::StrategyPattern, Language::TypeScript) => (
                "使用策略模式".to_string(),
                "发现结构相似的代码，建议使用策略模式抽象变化".to_string(),
                "定义策略接口，实现具体策略类，在上下文中使用策略对象".to_string(),
            ),
            _ => (
                "通用重构建议".to_string(),
                "发现重复代码，建议进行重构".to_string(),
                "分析重复代码的共同点和差异，选择合适的重构手法".to_string(),
            ),
        }
    }

    /// 计算预期收益
    fn calculate_expected_benefits(&self, duplication: &CodeDuplication, suggestion_type: &SuggestionType) -> Vec<String> {
        let mut benefits = Vec::new();

        // 基础收益
        benefits.push(format!("减少 {} 行重复代码", duplication.line_count * (duplication.code_blocks.len() - 1)));
        benefits.push("提高代码可维护性".to_string());
        benefits.push("降低 bug 修复成本".to_string());

        // 根据建议类型添加特定收益
        match suggestion_type {
            SuggestionType::ExtractConstant => {
                benefits.push("避免魔法数字，提高代码可读性".to_string());
                benefits.push("集中管理常量，便于后续修改".to_string());
            },
            SuggestionType::ExtractMethod => {
                benefits.push("提高代码复用性".to_string());
                benefits.push("简化单元测试".to_string());
                benefits.push("增强代码可读性".to_string());
            },
            SuggestionType::ExtractClass => {
                benefits.push("改善代码组织结构".to_string());
                benefits.push("提高模块化程度".to_string());
                benefits.push("便于功能扩展".to_string());
            },
            SuggestionType::TemplateMethod | SuggestionType::StrategyPattern => {
                benefits.push("提高代码灵活性".to_string());
                benefits.push("便于添加新的变体".to_string());
                benefits.push("符合开闭原则".to_string());
            },
            SuggestionType::CreateUtilityClass => {
                benefits.push("集中管理工具函数".to_string());
                benefits.push("提高跨项目复用性".to_string());
            },
            _ => {},
        }

        // 根据风险等级添加收益
        match duplication.risk_level {
            RiskLevel::Critical => {
                benefits.push("显著降低维护风险".to_string());
                benefits.push("大幅提升开发效率".to_string());
            },
            RiskLevel::High => {
                benefits.push("明显改善代码质量".to_string());
            },
            _ => {},
        }

        benefits
    }

    /// 生成代码示例
    async fn generate_code_example(
        &self,
        duplication: &CodeDuplication,
        suggestion_type: &SuggestionType,
        language: &Language,
    ) -> Result<Option<String>> {
        // 获取第一个代码块作为示例基础
        let first_block = duplication.code_blocks.first();
        if first_block.is_none() {
            return Ok(None);
        }

        let example = match (suggestion_type, language) {
            (SuggestionType::ExtractConstant, Language::Rust) => {
                self.generate_rust_constant_example(duplication).await?
            },
            (SuggestionType::ExtractMethod, Language::Rust) => {
                self.generate_rust_method_example(duplication).await?
            },
            (SuggestionType::ExtractClass, Language::Rust) => {
                self.generate_rust_module_example(duplication).await?
            },
            (SuggestionType::ExtractConstant, Language::Go) => {
                self.generate_go_constant_example(duplication).await?
            },
            (SuggestionType::ExtractMethod, Language::Go) => {
                self.generate_go_function_example(duplication).await?
            },
            (SuggestionType::ExtractConstant, Language::TypeScript) => {
                self.generate_typescript_constant_example(duplication).await?
            },
            (SuggestionType::ExtractMethod, Language::TypeScript) => {
                self.generate_typescript_function_example(duplication).await?
            },
            _ => None,
        };

        Ok(example)
    }

    /// 生成 Rust 常量提取示例
    async fn generate_rust_constant_example(&self, duplication: &CodeDuplication) -> Result<Option<String>> {
        let example = format!(
            r#"// 重构前：
{}

// 重构后：
const DEFAULT_TIMEOUT: u64 = 30;
const MAX_RETRIES: usize = 3;

// 在使用处：
// 使用 DEFAULT_TIMEOUT 和 MAX_RETRIES 替代重复的字面值"#,
            self.get_sample_content(duplication, 3)
        );
        Ok(Some(example))
    }

    /// 生成 Rust 方法提取示例
    async fn generate_rust_method_example(&self, duplication: &CodeDuplication) -> Result<Option<String>> {
        let example = format!(
            r#"// 重构前：
{}

// 重构后：
fn validate_and_process_data(data: &str) -> Result<ProcessedData, Error> {{
    // 提取的重复逻辑
    if data.is_empty() {{
        return Err(Error::EmptyData);
    }}

    let processed = data.trim().to_lowercase();
    Ok(ProcessedData::new(processed))
}}

// 在使用处：
let result = validate_and_process_data(&input)?;"#,
            self.get_sample_content(duplication, 10)
        );
        Ok(Some(example))
    }

    /// 生成 Rust 模块提取示例
    async fn generate_rust_module_example(&self, duplication: &CodeDuplication) -> Result<Option<String>> {
        let example = format!(
            r#"// 重构前：
{}

// 重构后：
// 新建 src/data_processor.rs
pub struct DataProcessor {{
    config: ProcessorConfig,
}}

impl DataProcessor {{
    pub fn new(config: ProcessorConfig) -> Self {{
        Self {{ config }}
    }}

    pub fn process(&self, data: &[u8]) -> Result<ProcessedData, ProcessorError> {{
        // 提取的重复逻辑
    }}
}}

// 在使用处：
use crate::data_processor::DataProcessor;
let processor = DataProcessor::new(config);
let result = processor.process(&data)?;"#,
            self.get_sample_content(duplication, 15)
        );
        Ok(Some(example))
    }

    /// 生成 Go 常量提取示例
    async fn generate_go_constant_example(&self, duplication: &CodeDuplication) -> Result<Option<String>> {
        let example = format!(
            r#"// 重构前：
{}

// 重构后：
const (
    DefaultTimeout = 30 * time.Second
    MaxRetries     = 3
    BufferSize     = 1024
)

// 在使用处：
// 使用 DefaultTimeout 和 MaxRetries 替代重复的字面值"#,
            self.get_sample_content(duplication, 3)
        );
        Ok(Some(example))
    }

    /// 生成 Go 函数提取示例
    async fn generate_go_function_example(&self, duplication: &CodeDuplication) -> Result<Option<String>> {
        let example = format!(
            r#"// 重构前：
{}

// 重构后：
func validateAndProcessData(data string) (*ProcessedData, error) {{
    // 提取的重复逻辑
    if data == "" {{
        return nil, errors.New("empty data")
    }}

    processed := strings.TrimSpace(strings.ToLower(data))
    return &ProcessedData{{Value: processed}}, nil
}}

// 在使用处：
result, err := validateAndProcessData(input)
if err != nil {{
    return err
}}"#,
            self.get_sample_content(duplication, 10)
        );
        Ok(Some(example))
    }

    /// 生成 TypeScript 常量提取示例
    async fn generate_typescript_constant_example(&self, duplication: &CodeDuplication) -> Result<Option<String>> {
        let example = format!(
            r#"// 重构前：
{}

// 重构后：
export const CONFIG = {{
    DEFAULT_TIMEOUT: 30000,
    MAX_RETRIES: 3,
    BUFFER_SIZE: 1024,
}} as const;

// 或使用枚举：
export enum Timeouts {{
    DEFAULT = 30000,
    LONG = 60000,
}}

// 在使用处：
// 使用 CONFIG.DEFAULT_TIMEOUT 替代重复的字面值"#,
            self.get_sample_content(duplication, 3)
        );
        Ok(Some(example))
    }

    /// 生成 TypeScript 函数提取示例
    async fn generate_typescript_function_example(&self, duplication: &CodeDuplication) -> Result<Option<String>> {
        let example = format!(
            r#"// 重构前：
{}

// 重构后：
function validateAndProcessData(data: string): ProcessedData {{
    // 提取的重复逻辑
    if (!data || data.trim() === '') {{
        throw new Error('Empty data');
    }}

    const processed = data.trim().toLowerCase();
    return new ProcessedData(processed);
}}

// 在使用处：
const result = validateAndProcessData(input);"#,
            self.get_sample_content(duplication, 10)
        );
        Ok(Some(example))
    }

    /// 获取示例内容
    fn get_sample_content(&self, duplication: &CodeDuplication, max_lines: usize) -> String {
        let content = &duplication.content;
        let lines: Vec<&str> = content.lines().take(max_lines).collect();
        if lines.len() < content.lines().count() {
            format!("{}\n// ... (省略更多行)", lines.join("\n"))
        } else {
            lines.join("\n")
        }
    }

    /// 获取相关资源链接
    fn get_relevant_resources(&self, suggestion_type: &SuggestionType, language: &Language) -> Vec<String> {
        let mut resources = vec![
            "https://refactoring.guru/refactoring".to_string(),
            "https://martinfowler.com/books/refactoring.html".to_string(),
        ];

        match suggestion_type {
            SuggestionType::ExtractConstant => {
                resources.push("https://refactoring.guru/extract-variable".to_string());
            },
            SuggestionType::ExtractMethod => {
                resources.push("https://refactoring.guru/extract-method".to_string());
            },
            SuggestionType::ExtractClass => {
                resources.push("https://refactoring.guru/extract-class".to_string());
            },
            SuggestionType::TemplateMethod => {
                resources.push("https://refactoring.guru/design-patterns/template-method".to_string());
            },
            SuggestionType::StrategyPattern => {
                resources.push("https://refactoring.guru/design-patterns/strategy".to_string());
            },
            _ => {},
        }

        match language {
            Language::Rust => {
                resources.push("https://doc.rust-lang.org/book/".to_string());
                resources.push("https://rust-unofficial.github.io/patterns/".to_string());
            },
            Language::Go => {
                resources.push("https://golang.org/doc/effective_go.html".to_string());
                resources.push("https://github.com/golang/go/wiki/CodeReviewComments".to_string());
            },
            Language::TypeScript => {
                resources.push("https://www.typescriptlang.org/docs/".to_string());
                resources.push("https://github.com/microsoft/TypeScript/wiki/Coding-guidelines".to_string());
            },
            _ => {},
        }

        resources
    }

    /// 计算重构优先级
    fn calculate_refactoring_priority(&self, duplication: &CodeDuplication) -> u32 {
        let mut priority_score = 0u32;

        // 基于风险等级的优先级
        priority_score += match duplication.risk_level {
            RiskLevel::Critical => 1000,
            RiskLevel::High => 500,
            RiskLevel::Medium => 200,
            RiskLevel::Low => 50,
        };

        // 基于重复块数量的优先级
        priority_score += (duplication.code_blocks.len() as u32) * 50;

        // 基于行数的优先级
        priority_score += (duplication.line_count as u32) * 2;

        // 基于相似度的优先级
        priority_score += (duplication.similarity_score * 100.0) as u32;

        // 跨文件重复获得额外优先级
        if duplication.duplication_type == DuplicationType::CrossFile {
            priority_score += 200;
        }

        priority_score
    }

    /// 生成组合重构建议
    async fn generate_combined_suggestions(&self, duplications: &[CodeDuplication]) -> Result<Vec<RefactoringSuggestion>> {
        let mut suggestions = Vec::new();

        // 分析跨文件重复模式
        let cross_file_duplications: Vec<_> = duplications
            .iter()
            .filter(|d| d.duplication_type == DuplicationType::CrossFile)
            .collect();

        if cross_file_duplications.len() >= 3 {
            let suggestion = self.create_utility_module_suggestion(&cross_file_duplications).await?;
            suggestions.push(suggestion);
        }

        // 分析高风险重复集中的文件
        let high_risk_files = self.identify_high_risk_files(duplications);
        if high_risk_files.len() >= 2 {
            let suggestion = self.create_file_refactoring_suggestion(&high_risk_files).await?;
            suggestions.push(suggestion);
        }

        Ok(suggestions)
    }

    /// 创建工具模块建议
    async fn create_utility_module_suggestion(&self, duplications: &[&CodeDuplication]) -> Result<RefactoringSuggestion> {
        let total_lines: usize = duplications.iter().map(|d| d.line_count).sum();
        let affected_files: std::collections::HashSet<String> = duplications
            .iter()
            .flat_map(|d| d.code_blocks.iter().map(|b| b.file_path.clone()))
            .collect();

        Ok(RefactoringSuggestion {
            id: uuid::Uuid::new_v4().to_string(),
            duplication_id: "combined_cross_file".to_string(),
            suggestion_type: SuggestionType::CreateUtilityClass,
            title: "创建公共工具模块".to_string(),
            description: format!(
                "发现 {} 个跨文件重复模式，涉及 {} 个文件，共 {} 行代码。建议创建公共工具模块统一管理。",
                duplications.len(),
                affected_files.len(),
                total_lines
            ),
            refactoring_approach: "创建 utils 或 common 模块，将跨文件重复的功能提取到公共模块中，通过导入语句在各处使用。".to_string(),
            expected_benefits: vec![
                format!("减少 {} 行重复代码", total_lines * (duplications.len() - 1)),
                "建立项目级别的代码复用机制".to_string(),
                "提高代码一致性".to_string(),
                "便于统一维护和测试".to_string(),
            ],
            implementation_complexity: ComplexityLevel::Complex,
            code_example: Some(self.generate_utility_module_example().await?),
            resources: vec![
                "https://refactoring.guru/extract-class".to_string(),
                "https://en.wikipedia.org/wiki/Don%27t_repeat_yourself".to_string(),
            ],
        })
    }

    /// 生成工具模块示例
    async fn generate_utility_module_example(&self) -> Result<String> {
        Ok(r#"// 新建 src/utils/common.rs
pub fn validate_input(input: &str) -> Result<String, ValidationError> {
    if input.is_empty() {
        return Err(ValidationError::Empty);
    }
    Ok(input.trim().to_string())
}

pub fn format_response<T: Serialize>(data: T) -> String {
    serde_json::to_string(&data).unwrap_or_default()
}

// 在各个文件中使用：
use crate::utils::common::{validate_input, format_response};

let validated = validate_input(&user_input)?;
let response = format_response(&result);"#.to_string())
    }

    /// 识别高风险文件
    fn identify_high_risk_files(&self, duplications: &[CodeDuplication]) -> Vec<String> {
        let mut file_risk_scores: HashMap<String, f64> = HashMap::new();

        for duplication in duplications {
            if matches!(duplication.risk_level, RiskLevel::High | RiskLevel::Critical) {
                for block in &duplication.code_blocks {
                    let score = file_risk_scores.entry(block.file_path.clone()).or_insert(0.0);
                    *score += duplication.line_count as f64 * duplication.similarity_score;
                }
            }
        }

        let mut high_risk_files: Vec<(String, f64)> = file_risk_scores.into_iter().collect();
        high_risk_files.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        high_risk_files
            .into_iter()
            .take(5) // 取前5个高风险文件
            .map(|(file, _)| file)
            .collect()
    }

    /// 创建文件重构建议
    async fn create_file_refactoring_suggestion(&self, files: &[String]) -> Result<RefactoringSuggestion> {
        Ok(RefactoringSuggestion {
            id: uuid::Uuid::new_v4().to_string(),
            duplication_id: "combined_high_risk_files".to_string(),
            suggestion_type: SuggestionType::ExtractClass,
            title: "重构高风险文件".to_string(),
            description: format!(
                "发现 {} 个文件包含大量重复代码，建议优先进行重构。",
                files.len()
            ),
            refactoring_approach: "分析这些文件的共同模式，提取公共功能到独立模块，简化原文件结构。".to_string(),
            expected_benefits: vec![
                "显著降低维护成本".to_string(),
                "提高代码质量".to_string(),
                "减少 bug 风险".to_string(),
                "改善开发体验".to_string(),
            ],
            implementation_complexity: ComplexityLevel::VeryComplex,
            code_example: None,
            resources: vec![
                "https://refactoring.guru/refactoring/smells/large-class".to_string(),
                "https://refactoring.guru/extract-class".to_string(),
            ],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_duplication_config_default() {
        let config = DuplicationConfig::default();
        assert_eq!(config.min_duplicate_lines, 5);
        assert_eq!(config.min_duplicate_chars, 100);
        assert_eq!(config.structural_similarity_threshold, 0.8);
        assert!(config.enable_exact_detection);
        assert!(config.enable_structural_detection);
        assert!(config.enable_cross_file_detection);
    }

    #[test]
    fn test_risk_level_assessment() {
        assert_eq!(RiskLevel::assess(5, 0.6), RiskLevel::Low);
        assert_eq!(RiskLevel::assess(25, 0.8), RiskLevel::Medium);
        assert_eq!(RiskLevel::assess(60, 0.9), RiskLevel::High);
        assert_eq!(RiskLevel::assess(150, 0.98), RiskLevel::Critical);
    }

    #[test]
    fn test_refactoring_priority_assessment() {
        assert_eq!(RefactoringPriority::assess(RiskLevel::Low, 2), RefactoringPriority::Low);
        assert_eq!(RefactoringPriority::assess(RiskLevel::Medium, 5), RefactoringPriority::Medium);
        assert_eq!(RefactoringPriority::assess(RiskLevel::High, 3), RefactoringPriority::High);
        assert_eq!(RefactoringPriority::assess(RiskLevel::Critical, 1), RefactoringPriority::Urgent);
    }
}

/// 结构模式信息
#[derive(Debug, Clone)]
struct StructuralPatternInfo {
    start_line: usize,
    end_line: usize,
    content: String,
    pattern: StructuralPattern,
    similarity_features: SimilarityFeatures,
}

/// 结构模式
#[derive(Debug, Clone)]
struct StructuralPattern {
    /// 控制流元素
    control_flow: Vec<ControlFlowElement>,
    /// 函数调用元素
    function_calls: Vec<FunctionCallElement>,
    /// 变量声明元素
    variable_declarations: Vec<VariableDeclarationElement>,
    /// 代码块结构
    block_structure: BlockStructure,
    /// 操作符模式
    operator_pattern: OperatorPattern,
    /// 结构复杂度分数
    complexity_score: f64,
}

impl StructuralPattern {
    fn new() -> Self {
        Self {
            control_flow: Vec::new(),
            function_calls: Vec::new(),
            variable_declarations: Vec::new(),
            block_structure: BlockStructure::new(),
            operator_pattern: OperatorPattern::new(),
            complexity_score: 0.0,
        }
    }

    fn is_meaningful(&self) -> bool {
        !self.control_flow.is_empty() ||
        !self.function_calls.is_empty() ||
        self.block_structure.max_brace_depth > 0 ||
        self.complexity_score >= 3.0
    }
}

/// 结构匹配
#[derive(Debug, Clone)]
struct StructuralMatch {
    file_path: String,
    start_line: usize,
    end_line: usize,
    content: String,
    pattern: StructuralPattern,
    signature: String,
    similarity_features: SimilarityFeatures,
}

/// 控制流元素
#[derive(Debug, Clone)]
struct ControlFlowElement {
    flow_type: ControlFlowType,
    line_number: usize,
}

impl ControlFlowElement {
    fn new(flow_type: ControlFlowType, line_number: usize) -> Self {
        Self { flow_type, line_number }
    }
}

/// 控制流类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ControlFlowType {
    If,
    ElseIf,
    Else,
    For,
    While,
    Loop,
    Match,
    Switch,
    Select,
    Try,
    Catch,
}

/// 函数调用元素
#[derive(Debug, Clone)]
struct FunctionCallElement {
    function_name: String,
    line_number: usize,
    call_type: FunctionCallType,
}

/// 函数调用类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FunctionCallType {
    StandardLibrary,
    Constructor,
    UserDefined,
}

/// 变量声明元素
#[derive(Debug, Clone)]
struct VariableDeclarationElement {
    line_number: usize,
    declaration_type: VariableDeclarationType,
}

/// 变量声明类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VariableDeclarationType {
    Mutable,
    Immutable,
    Constant,
    BlockScoped,
    FunctionScoped,
    Unknown,
}

/// 代码块结构
#[derive(Debug, Clone)]
struct BlockStructure {
    max_brace_depth: usize,
    max_paren_depth: usize,
    max_bracket_depth: usize,
    total_braces: usize,
    total_parens: usize,
    total_brackets: usize,
}

impl BlockStructure {
    fn new() -> Self {
        Self {
            max_brace_depth: 0,
            max_paren_depth: 0,
            max_bracket_depth: 0,
            total_braces: 0,
            total_parens: 0,
            total_brackets: 0,
        }
    }
}

/// 操作符模式
#[derive(Debug, Clone)]
struct OperatorPattern {
    arithmetic_ops: usize,
    comparison_ops: usize,
    logical_ops: usize,
    assignment_ops: usize,
}

impl OperatorPattern {
    fn new() -> Self {
        Self {
            arithmetic_ops: 0,
            comparison_ops: 0,
            logical_ops: 0,
            assignment_ops: 0,
        }
    }
}

/// 相似性特征
#[derive(Debug, Clone)]
struct SimilarityFeatures {
    line_count: usize,
    non_empty_line_count: usize,
    keyword_count: usize,
    identifier_pattern: String,
    indentation_pattern: String,
    comment_ratio: f64,
}
