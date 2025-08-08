use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// 重复检测结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicationResult {
    /// 项目路径
    pub project_path: String,
    /// 检测到的重复代码块
    pub duplications: Vec<CodeDuplication>,
    /// 重复统计摘要
    pub summary: DuplicationSummary,
    /// 重构建议
    pub refactoring_suggestions: Vec<RefactoringSuggestion>,
}

/// 代码重复块
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeDuplication {
    /// 重复ID
    pub id: String,
    /// 重复类型
    pub duplication_type: DuplicationType,
    /// 重复的代码块
    pub code_blocks: Vec<CodeBlock>,
    /// 重复内容
    pub content: String,
    /// 重复行数
    pub line_count: usize,
    /// 相似度分数 (0.0-1.0)
    pub similarity_score: f64,
    /// 风险等级
    pub risk_level: RiskLevel,
    /// 重构优先级
    pub refactoring_priority: RefactoringPriority,
}

/// 代码块位置信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeBlock {
    /// 文件路径
    pub file_path: String,
    /// 开始行号
    pub start_line: usize,
    /// 结束行号
    pub end_line: usize,
    /// 开始列号
    pub start_column: Option<usize>,
    /// 结束列号
    pub end_column: Option<usize>,
    /// 代码内容哈希
    pub content_hash: String,
}

/// 重复类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DuplicationType {
    /// 精确重复 - 完全相同的代码
    Exact,
    /// 结构相似 - AST结构相同但内容可能不同
    Structural,
    /// 跨文件重复 - 不同文件间的重复
    CrossFile,
    /// 近似重复 - 高度相似但不完全相同
    Similar,
}

/// 风险等级
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub enum RiskLevel {
    /// 低风险 - 小段重复代码
    Low,
    /// 中等风险 - 中等大小重复代码
    Medium,
    /// 高风险 - 大段重复代码
    High,
    /// 严重风险 - 大量重复代码
    Critical,
}

/// 重构优先级
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RefactoringPriority {
    /// 低优先级
    Low,
    /// 中等优先级
    Medium,
    /// 高优先级
    High,
    /// 紧急优先级
    Urgent,
}

/// 重复统计摘要
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicationSummary {
    /// 总文件数
    pub total_files: usize,
    /// 有重复的文件数
    pub files_with_duplications: usize,
    /// 总重复块数
    pub total_duplications: usize,
    /// 重复行数
    pub duplicated_lines: usize,
    /// 总行数
    pub total_lines: usize,
    /// 重复率 (0.0-1.0)
    pub duplication_ratio: f64,
    /// 按类型分组的统计
    pub by_type: HashMap<DuplicationType, TypeStatistics>,
    /// 按风险等级分组的统计
    pub by_risk_level: HashMap<RiskLevel, RiskStatistics>,
    /// 重复代码热点文件
    pub hotspot_files: Vec<HotspotFile>,
}

/// 按类型统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeStatistics {
    /// 该类型的重复块数量
    pub count: usize,
    /// 该类型的重复行数
    pub lines: usize,
    /// 该类型的重复率
    pub ratio: f64,
}

/// 按风险等级统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskStatistics {
    /// 该风险等级的重复块数量
    pub count: usize,
    /// 该风险等级的重复行数
    pub lines: usize,
    /// 该风险等级的重复率
    pub ratio: f64,
}

/// 重复代码热点文件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotspotFile {
    /// 文件路径
    pub file_path: String,
    /// 重复块数量
    pub duplication_count: usize,
    /// 重复行数
    pub duplicated_lines: usize,
    /// 文件总行数
    pub total_lines: usize,
    /// 重复率
    pub duplication_ratio: f64,
    /// 风险评分
    pub risk_score: f64,
    /// 热点等级
    pub hotspot_level: HotspotLevel,
}

/// 热点等级
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
pub enum HotspotLevel {
    /// 轻微热点
    Minor,
    /// 中等热点
    Moderate,
    /// 严重热点
    Severe,
    /// 极端热点
    Extreme,
}

/// 重复代码分布图数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicationDistribution {
    /// 按文件大小分布
    pub by_file_size: Vec<FileSizeDistribution>,
    /// 按重复类型分布
    pub by_type_distribution: Vec<TypeDistribution>,
    /// 按风险等级分布
    pub by_risk_distribution: Vec<RiskDistribution>,
    /// 重复代码密度分布
    pub density_distribution: Vec<DensityDistribution>,
}

/// 按文件大小分布
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSizeDistribution {
    /// 文件大小范围（行数）
    pub size_range: String,
    /// 文件数量
    pub file_count: usize,
    /// 重复文件数量
    pub duplicated_file_count: usize,
    /// 重复率
    pub duplication_ratio: f64,
}

/// 按类型分布
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeDistribution {
    /// 重复类型
    pub duplication_type: DuplicationType,
    /// 重复块数量
    pub count: usize,
    /// 占总重复的百分比
    pub percentage: f64,
    /// 平均重复行数
    pub avg_lines: f64,
}

/// 按风险等级分布
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskDistribution {
    /// 风险等级
    pub risk_level: RiskLevel,
    /// 重复块数量
    pub count: usize,
    /// 占总重复的百分比
    pub percentage: f64,
    /// 涉及的文件数
    pub affected_files: usize,
}

/// 重复代码密度分布
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DensityDistribution {
    /// 密度范围（重复率）
    pub density_range: String,
    /// 文件数量
    pub file_count: usize,
    /// 平均重复行数
    pub avg_duplicated_lines: f64,
}

/// 详细统计报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetailedDuplicationReport {
    /// 项目概览
    pub project_overview: ProjectOverview,
    /// 重复代码分布
    pub distribution: DuplicationDistribution,
    /// 热点文件分析
    pub hotspot_analysis: HotspotAnalysis,
    /// 趋势分析
    pub trend_analysis: Option<TrendAnalysis>,
    /// 改进建议
    pub improvement_recommendations: Vec<ImprovementRecommendation>,
}

/// 项目概览
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectOverview {
    /// 项目路径
    pub project_path: String,
    /// 分析时间
    pub analysis_time: String,
    /// 总体健康评分 (0-100)
    pub health_score: f64,
    /// 重复代码等级
    pub duplication_grade: DuplicationGrade,
    /// 关键指标
    pub key_metrics: KeyMetrics,
}

/// 重复代码等级
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DuplicationGrade {
    /// A级 - 优秀 (重复率 < 5%)
    A,
    /// B级 - 良好 (重复率 5-10%)
    B,
    /// C级 - 一般 (重复率 10-20%)
    C,
    /// D级 - 较差 (重复率 20-30%)
    D,
    /// F级 - 很差 (重复率 > 30%)
    F,
}

/// 关键指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyMetrics {
    /// 总重复率
    pub total_duplication_ratio: f64,
    /// 平均重复块大小
    pub avg_duplication_size: f64,
    /// 最大重复块大小
    pub max_duplication_size: usize,
    /// 重复文件比例
    pub duplicated_files_ratio: f64,
    /// 高风险重复数量
    pub high_risk_duplications: usize,
}

/// 热点分析
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotspotAnalysis {
    /// 热点文件列表
    pub hotspot_files: Vec<HotspotFile>,
    /// 热点模式
    pub hotspot_patterns: Vec<HotspotPattern>,
    /// 热点建议
    pub hotspot_recommendations: Vec<String>,
}

/// 热点模式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotspotPattern {
    /// 模式名称
    pub pattern_name: String,
    /// 模式描述
    pub description: String,
    /// 涉及的文件
    pub affected_files: Vec<String>,
    /// 重复次数
    pub occurrence_count: usize,
}

/// 趋势分析
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendAnalysis {
    /// 历史数据点
    pub historical_data: Vec<HistoricalDataPoint>,
    /// 趋势方向
    pub trend_direction: TrendDirection,
    /// 预测值
    pub predicted_values: Vec<PredictedValue>,
}

/// 历史数据点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalDataPoint {
    /// 时间戳
    pub timestamp: String,
    /// 重复率
    pub duplication_ratio: f64,
    /// 重复块数量
    pub duplication_count: usize,
}

/// 趋势方向
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrendDirection {
    /// 改善中
    Improving,
    /// 稳定
    Stable,
    /// 恶化中
    Deteriorating,
}

/// 预测值
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictedValue {
    /// 预测时间
    pub predicted_time: String,
    /// 预测重复率
    pub predicted_ratio: f64,
    /// 置信度
    pub confidence: f64,
}

/// 改进建议
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImprovementRecommendation {
    /// 建议标题
    pub title: String,
    /// 建议描述
    pub description: String,
    /// 优先级
    pub priority: RecommendationPriority,
    /// 预期影响
    pub expected_impact: String,
    /// 实施难度
    pub implementation_difficulty: ImplementationDifficulty,
    /// 相关文件
    pub related_files: Vec<String>,
}

/// 建议优先级
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
pub enum RecommendationPriority {
    /// 低优先级
    Low,
    /// 中等优先级
    Medium,
    /// 高优先级
    High,
    /// 紧急优先级
    Critical,
}

/// 实施难度
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImplementationDifficulty {
    /// 简单
    Easy,
    /// 中等
    Medium,
    /// 困难
    Hard,
    /// 非常困难
    VeryHard,
}

/// 重构建议
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefactoringSuggestion {
    /// 建议ID
    pub id: String,
    /// 相关的重复代码ID
    pub duplication_id: String,
    /// 建议类型
    pub suggestion_type: SuggestionType,
    /// 建议标题
    pub title: String,
    /// 建议描述
    pub description: String,
    /// 重构方案
    pub refactoring_approach: String,
    /// 预期收益
    pub expected_benefits: Vec<String>,
    /// 实施复杂度
    pub implementation_complexity: ComplexityLevel,
    /// 代码示例
    pub code_example: Option<String>,
    /// 相关资源链接
    pub resources: Vec<String>,
}

/// 建议类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SuggestionType {
    /// 提取方法
    ExtractMethod,
    /// 提取类
    ExtractClass,
    /// 提取常量
    ExtractConstant,
    /// 使用模板方法模式
    TemplateMethod,
    /// 使用策略模式
    StrategyPattern,
    /// 合并相似代码
    MergeSimilarCode,
    /// 创建工具类
    CreateUtilityClass,
}

/// 实施复杂度
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComplexityLevel {
    /// 简单 - 容易实施
    Simple,
    /// 中等 - 需要一些重构
    Moderate,
    /// 复杂 - 需要大量重构
    Complex,
    /// 非常复杂 - 需要架构级别的改动
    VeryComplex,
}

impl DuplicationResult {
    /// 创建新的重复检测结果
    pub fn new(project_path: String) -> Self {
        Self {
            project_path,
            duplications: Vec::new(),
            summary: DuplicationSummary::default(),
            refactoring_suggestions: Vec::new(),
        }
    }

    /// 添加重复代码块
    pub fn add_duplication(&mut self, duplication: CodeDuplication) {
        self.duplications.push(duplication);
    }

    /// 添加重构建议
    pub fn add_suggestion(&mut self, suggestion: RefactoringSuggestion) {
        self.refactoring_suggestions.push(suggestion);
    }

    /// 计算文件行数
    fn calculate_file_lines(file_path: &str) -> Result<usize, std::io::Error> {
        let content = std::fs::read_to_string(file_path)?;
        Ok(content.lines().count())
    }

    /// 生成重复代码分布图数据
    pub fn generate_distribution(&self) -> DuplicationDistribution {
        let by_file_size = self.generate_file_size_distribution();
        let by_type_distribution = self.generate_type_distribution();
        let by_risk_distribution = self.generate_risk_distribution();
        let density_distribution = self.generate_density_distribution();

        DuplicationDistribution {
            by_file_size,
            by_type_distribution,
            by_risk_distribution,
            density_distribution,
        }
    }

    /// 生成按文件大小分布
    fn generate_file_size_distribution(&self) -> Vec<FileSizeDistribution> {
        let mut size_buckets: HashMap<String, (usize, usize)> = HashMap::new(); // (total_files, duplicated_files)
        let mut file_sizes: HashMap<String, usize> = HashMap::new();

        // 收集文件大小信息
        for hotspot in &self.summary.hotspot_files {
            file_sizes.insert(hotspot.file_path.clone(), hotspot.total_lines);
        }

        // 分类到大小桶中
        for (file_path, total_lines) in &file_sizes {
            let size_range = match *total_lines {
                0..=50 => "0-50行",
                51..=100 => "51-100行",
                101..=200 => "101-200行",
                201..=500 => "201-500行",
                501..=1000 => "501-1000行",
                _ => "1000行以上",
            };

            let (total_count, duplicated_count) = size_buckets.entry(size_range.to_string()).or_insert((0, 0));
            *total_count += 1;

            // 检查是否有重复
            if self.summary.hotspot_files.iter().any(|h| &h.file_path == file_path && h.duplication_count > 0) {
                *duplicated_count += 1;
            }
        }

        size_buckets
            .into_iter()
            .map(|(size_range, (file_count, duplicated_file_count))| {
                let duplication_ratio = if file_count > 0 {
                    duplicated_file_count as f64 / file_count as f64
                } else {
                    0.0
                };

                FileSizeDistribution {
                    size_range,
                    file_count,
                    duplicated_file_count,
                    duplication_ratio,
                }
            })
            .collect()
    }

    /// 生成按类型分布
    fn generate_type_distribution(&self) -> Vec<TypeDistribution> {
        let total_duplications = self.duplications.len();

        self.summary.by_type
            .iter()
            .map(|(duplication_type, stats)| {
                let percentage = if total_duplications > 0 {
                    stats.count as f64 / total_duplications as f64 * 100.0
                } else {
                    0.0
                };

                let avg_lines = if stats.count > 0 {
                    stats.lines as f64 / stats.count as f64
                } else {
                    0.0
                };

                TypeDistribution {
                    duplication_type: *duplication_type,
                    count: stats.count,
                    percentage,
                    avg_lines,
                }
            })
            .collect()
    }

    /// 生成按风险等级分布
    fn generate_risk_distribution(&self) -> Vec<RiskDistribution> {
        let total_duplications = self.duplications.len();

        self.summary.by_risk_level
            .iter()
            .map(|(risk_level, stats)| {
                let percentage = if total_duplications > 0 {
                    stats.count as f64 / total_duplications as f64 * 100.0
                } else {
                    0.0
                };

                // 计算涉及的文件数
                let affected_files = self.duplications
                    .iter()
                    .filter(|d| d.risk_level == *risk_level)
                    .flat_map(|d| d.code_blocks.iter().map(|b| &b.file_path))
                    .collect::<std::collections::HashSet<_>>()
                    .len();

                RiskDistribution {
                    risk_level: *risk_level,
                    count: stats.count,
                    percentage,
                    affected_files,
                }
            })
            .collect()
    }

    /// 生成密度分布
    fn generate_density_distribution(&self) -> Vec<DensityDistribution> {
        let mut density_buckets: HashMap<String, Vec<usize>> = HashMap::new();

        for hotspot in &self.summary.hotspot_files {
            let density_range = match hotspot.duplication_ratio {
                r if r < 0.1 => "0-10%",
                r if r < 0.2 => "10-20%",
                r if r < 0.3 => "20-30%",
                r if r < 0.5 => "30-50%",
                r if r < 0.7 => "50-70%",
                _ => "70%以上",
            };

            density_buckets
                .entry(density_range.to_string())
                .or_insert_with(Vec::new)
                .push(hotspot.duplicated_lines);
        }

        density_buckets
            .into_iter()
            .map(|(density_range, duplicated_lines_list)| {
                let file_count = duplicated_lines_list.len();
                let avg_duplicated_lines = if file_count > 0 {
                    duplicated_lines_list.iter().sum::<usize>() as f64 / file_count as f64
                } else {
                    0.0
                };

                DensityDistribution {
                    density_range,
                    file_count,
                    avg_duplicated_lines,
                }
            })
            .collect()
    }

    /// 生成详细统计报告
    pub fn generate_detailed_report(&self) -> DetailedDuplicationReport {
        let project_overview = self.generate_project_overview();
        let distribution = self.generate_distribution();
        let hotspot_analysis = self.generate_hotspot_analysis();
        let improvement_recommendations = self.generate_improvement_recommendations();

        DetailedDuplicationReport {
            project_overview,
            distribution,
            hotspot_analysis,
            trend_analysis: None, // 趋势分析需要历史数据，暂时为空
            improvement_recommendations,
        }
    }

    /// 生成项目概览
    fn generate_project_overview(&self) -> ProjectOverview {
        let health_score = self.calculate_health_score();
        let duplication_grade = DuplicationGrade::from_ratio(self.summary.duplication_ratio);
        let key_metrics = self.calculate_key_metrics();

        ProjectOverview {
            project_path: self.project_path.clone(),
            analysis_time: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string(),
            health_score,
            duplication_grade,
            key_metrics,
        }
    }

    /// 计算健康评分
    fn calculate_health_score(&self) -> f64 {
        let base_score = 100.0;

        // 根据重复率扣分
        let duplication_penalty = self.summary.duplication_ratio * 100.0;

        // 根据高风险重复数量扣分
        let high_risk_count = self.duplications
            .iter()
            .filter(|d| matches!(d.risk_level, RiskLevel::High | RiskLevel::Critical))
            .count();
        let high_risk_penalty = high_risk_count as f64 * 5.0;

        // 根据热点文件数量扣分
        let hotspot_penalty = self.summary.hotspot_files
            .iter()
            .filter(|h| matches!(h.hotspot_level, HotspotLevel::Severe | HotspotLevel::Extreme))
            .count() as f64 * 3.0;

        (base_score - duplication_penalty - high_risk_penalty - hotspot_penalty).max(0.0)
    }

    /// 计算关键指标
    fn calculate_key_metrics(&self) -> KeyMetrics {
        let avg_duplication_size = if self.duplications.is_empty() {
            0.0
        } else {
            self.duplications.iter().map(|d| d.line_count).sum::<usize>() as f64 / self.duplications.len() as f64
        };

        let max_duplication_size = self.duplications
            .iter()
            .map(|d| d.line_count)
            .max()
            .unwrap_or(0);

        let duplicated_files_ratio = if self.summary.total_files > 0 {
            self.summary.files_with_duplications as f64 / self.summary.total_files as f64
        } else {
            0.0
        };

        let high_risk_duplications = self.duplications
            .iter()
            .filter(|d| matches!(d.risk_level, RiskLevel::High | RiskLevel::Critical))
            .count();

        KeyMetrics {
            total_duplication_ratio: self.summary.duplication_ratio,
            avg_duplication_size,
            max_duplication_size,
            duplicated_files_ratio,
            high_risk_duplications,
        }
    }

    /// 生成热点分析
    fn generate_hotspot_analysis(&self) -> HotspotAnalysis {
        let hotspot_files = self.summary.hotspot_files.clone();
        let hotspot_patterns = self.identify_hotspot_patterns();
        let hotspot_recommendations = self.generate_hotspot_recommendations();

        HotspotAnalysis {
            hotspot_files,
            hotspot_patterns,
            hotspot_recommendations,
        }
    }

    /// 识别热点模式
    fn identify_hotspot_patterns(&self) -> Vec<HotspotPattern> {
        let mut patterns = Vec::new();

        // 识别跨文件重复模式
        let cross_file_duplications: Vec<_> = self.duplications
            .iter()
            .filter(|d| d.duplication_type == DuplicationType::CrossFile)
            .collect();

        if !cross_file_duplications.is_empty() {
            let affected_files: Vec<String> = cross_file_duplications
                .iter()
                .flat_map(|d| d.code_blocks.iter().map(|b| b.file_path.clone()))
                .collect::<std::collections::HashSet<_>>()
                .into_iter()
                .collect();

            patterns.push(HotspotPattern {
                pattern_name: "跨文件重复模式".to_string(),
                description: "存在大量跨文件的重复代码，可能表明缺乏代码复用机制".to_string(),
                affected_files,
                occurrence_count: cross_file_duplications.len(),
            });
        }

        // 识别高风险集中模式
        let high_risk_files: Vec<String> = self.summary.hotspot_files
            .iter()
            .filter(|h| matches!(h.hotspot_level, HotspotLevel::Severe | HotspotLevel::Extreme))
            .map(|h| h.file_path.clone())
            .collect();

        if !high_risk_files.is_empty() {
            patterns.push(HotspotPattern {
                pattern_name: "高风险文件集中".to_string(),
                description: "部分文件包含大量重复代码，需要优先重构".to_string(),
                affected_files: high_risk_files.clone(),
                occurrence_count: high_risk_files.len(),
            });
        }

        patterns
    }

    /// 生成热点建议
    fn generate_hotspot_recommendations(&self) -> Vec<String> {
        let mut recommendations = Vec::new();

        // 基于热点文件数量的建议
        let extreme_hotspots = self.summary.hotspot_files
            .iter()
            .filter(|h| h.hotspot_level == HotspotLevel::Extreme)
            .count();

        if extreme_hotspots > 0 {
            recommendations.push(format!(
                "发现 {} 个极端热点文件，建议立即进行重构以降低维护成本",
                extreme_hotspots
            ));
        }

        // 基于重复率的建议
        if self.summary.duplication_ratio > 0.3 {
            recommendations.push("项目整体重复率过高，建议制定系统性的代码重构计划".to_string());
        } else if self.summary.duplication_ratio > 0.2 {
            recommendations.push("项目重复率较高，建议优先处理高风险重复代码".to_string());
        }

        // 基于跨文件重复的建议
        let cross_file_count = self.duplications
            .iter()
            .filter(|d| d.duplication_type == DuplicationType::CrossFile)
            .count();

        if cross_file_count > 5 {
            recommendations.push("存在大量跨文件重复，建议建立公共代码库或工具类".to_string());
        }

        if recommendations.is_empty() {
            recommendations.push("代码重复情况良好，继续保持良好的编码实践".to_string());
        }

        recommendations
    }

    /// 生成改进建议
    fn generate_improvement_recommendations(&self) -> Vec<ImprovementRecommendation> {
        let mut recommendations = Vec::new();

        // 高优先级建议：处理极端热点文件
        let extreme_hotspots: Vec<_> = self.summary.hotspot_files
            .iter()
            .filter(|h| h.hotspot_level == HotspotLevel::Extreme)
            .collect();

        if !extreme_hotspots.is_empty() {
            recommendations.push(ImprovementRecommendation {
                title: "重构极端热点文件".to_string(),
                description: "这些文件包含大量重复代码，严重影响代码质量和维护性".to_string(),
                priority: RecommendationPriority::Critical,
                expected_impact: format!("预计可减少 {}% 的重复代码",
                    extreme_hotspots.iter().map(|h| h.duplicated_lines).sum::<usize>() as f64 / self.summary.duplicated_lines as f64 * 100.0),
                implementation_difficulty: ImplementationDifficulty::Hard,
                related_files: extreme_hotspots.iter().map(|h| h.file_path.clone()).collect(),
            });
        }

        // 中优先级建议：处理跨文件重复
        let cross_file_duplications: Vec<_> = self.duplications
            .iter()
            .filter(|d| d.duplication_type == DuplicationType::CrossFile)
            .collect();

        if cross_file_duplications.len() > 3 {
            let related_files: Vec<String> = cross_file_duplications
                .iter()
                .flat_map(|d| d.code_blocks.iter().map(|b| b.file_path.clone()))
                .collect::<std::collections::HashSet<_>>()
                .into_iter()
                .collect();

            recommendations.push(ImprovementRecommendation {
                title: "建立公共代码库".to_string(),
                description: "提取跨文件重复的代码到公共模块中，提高代码复用性".to_string(),
                priority: RecommendationPriority::High,
                expected_impact: "提高代码复用性，减少维护成本".to_string(),
                implementation_difficulty: ImplementationDifficulty::Medium,
                related_files,
            });
        }

        // 低优先级建议：代码规范
        if self.summary.duplication_ratio > 0.1 {
            recommendations.push(ImprovementRecommendation {
                title: "建立代码审查机制".to_string(),
                description: "通过代码审查流程防止新的重复代码引入".to_string(),
                priority: RecommendationPriority::Medium,
                expected_impact: "预防新的重复代码产生".to_string(),
                implementation_difficulty: ImplementationDifficulty::Easy,
                related_files: Vec::new(),
            });
        }

        recommendations
    }

    /// 计算统计摘要
    pub fn calculate_summary(&mut self, total_files: usize, total_lines: usize) {
        let mut files_with_duplications = std::collections::HashSet::new();
        let mut duplicated_lines = 0;
        let mut by_type: HashMap<DuplicationType, TypeStatistics> = HashMap::new();
        let mut by_risk_level: HashMap<RiskLevel, RiskStatistics> = HashMap::new();
        let mut file_stats: HashMap<String, (usize, usize)> = HashMap::new(); // (duplication_count, duplicated_lines)

        for duplication in &self.duplications {
            // 收集有重复的文件
            for block in &duplication.code_blocks {
                files_with_duplications.insert(block.file_path.clone());
                let (count, lines) = file_stats.entry(block.file_path.clone()).or_insert((0, 0));
                *count += 1;
                *lines += duplication.line_count;
            }

            duplicated_lines += duplication.line_count;

            // 按类型统计
            let type_stat = by_type.entry(duplication.duplication_type).or_insert(TypeStatistics {
                count: 0,
                lines: 0,
                ratio: 0.0,
            });
            type_stat.count += 1;
            type_stat.lines += duplication.line_count;

            // 按风险等级统计
            let risk_stat = by_risk_level.entry(duplication.risk_level).or_insert(RiskStatistics {
                count: 0,
                lines: 0,
                ratio: 0.0,
            });
            risk_stat.count += 1;
            risk_stat.lines += duplication.line_count;
        }

        // 计算比率
        let duplication_ratio = if total_lines > 0 {
            duplicated_lines as f64 / total_lines as f64
        } else {
            0.0
        };

        for stat in by_type.values_mut() {
            stat.ratio = if total_lines > 0 {
                stat.lines as f64 / total_lines as f64
            } else {
                0.0
            };
        }

        for stat in by_risk_level.values_mut() {
            stat.ratio = if total_lines > 0 {
                stat.lines as f64 / total_lines as f64
            } else {
                0.0
            };
        }

        // 生成热点文件列表
        let mut hotspot_files: Vec<HotspotFile> = file_stats
            .into_iter()
            .map(|(file_path, (duplication_count, duplicated_lines))| {
                let file_total_lines = Self::calculate_file_lines(&file_path).unwrap_or(100);
                let duplication_ratio = if file_total_lines > 0 {
                    duplicated_lines as f64 / file_total_lines as f64
                } else {
                    0.0
                };
                let risk_score = duplication_count as f64 * duplication_ratio;
                let hotspot_level = HotspotLevel::assess(duplication_ratio, duplication_count);

                HotspotFile {
                    file_path,
                    duplication_count,
                    duplicated_lines,
                    total_lines: file_total_lines,
                    duplication_ratio,
                    risk_score,
                    hotspot_level,
                }
            })
            .collect();

        // 按风险评分排序
        hotspot_files.sort_by(|a, b| b.risk_score.partial_cmp(&a.risk_score).unwrap());

        self.summary = DuplicationSummary {
            total_files,
            files_with_duplications: files_with_duplications.len(),
            total_duplications: self.duplications.len(),
            duplicated_lines,
            total_lines,
            duplication_ratio,
            by_type,
            by_risk_level,
            hotspot_files,
        };
    }
}

impl Default for DuplicationSummary {
    fn default() -> Self {
        Self {
            total_files: 0,
            files_with_duplications: 0,
            total_duplications: 0,
            duplicated_lines: 0,
            total_lines: 0,
            duplication_ratio: 0.0,
            by_type: HashMap::new(),
            by_risk_level: HashMap::new(),
            hotspot_files: Vec::new(),
        }
    }
}

impl RiskLevel {
    /// 根据重复行数和相似度评估风险等级
    pub fn assess(line_count: usize, similarity_score: f64) -> Self {
        // 对于精确匹配（similarity_score = 1.0），主要基于行数评估
        if similarity_score >= 0.99 {
            match line_count {
                lines if lines >= 100 => RiskLevel::Critical,
                lines if lines >= 50 => RiskLevel::High,
                lines if lines >= 20 => RiskLevel::Medium,
                _ => RiskLevel::Low,
            }
        } else {
            // 对于非精确匹配，综合考虑行数和相似度
            match (line_count, similarity_score) {
                (lines, score) if lines >= 100 || score >= 0.95 => RiskLevel::Critical,
                (lines, score) if lines >= 50 || score >= 0.85 => RiskLevel::High,
                (lines, score) if lines >= 20 || score >= 0.75 => RiskLevel::Medium,
                _ => RiskLevel::Low,
            }
        }
    }
}

impl RefactoringPriority {
    /// 根据风险等级和重复块数量评估重构优先级
    pub fn assess(risk_level: RiskLevel, duplication_count: usize) -> Self {
        match (risk_level, duplication_count) {
            (RiskLevel::Critical, _) => RefactoringPriority::Urgent,
            (RiskLevel::High, count) if count >= 5 => RefactoringPriority::Urgent,
            (RiskLevel::High, _) => RefactoringPriority::High,
            (RiskLevel::Medium, count) if count >= 10 => RefactoringPriority::High,
            (RiskLevel::Medium, _) => RefactoringPriority::Medium,
            _ => RefactoringPriority::Low,
        }
    }
}

impl HotspotLevel {
    /// 根据重复率和重复数量评估热点等级
    pub fn assess(duplication_ratio: f64, duplication_count: usize) -> Self {
        match (duplication_ratio, duplication_count) {
            (ratio, count) if ratio >= 0.5 || count >= 20 => HotspotLevel::Extreme,
            (ratio, count) if ratio >= 0.3 || count >= 10 => HotspotLevel::Severe,
            (ratio, count) if ratio >= 0.15 || count >= 5 => HotspotLevel::Moderate,
            _ => HotspotLevel::Minor,
        }
    }
}

impl DuplicationGrade {
    /// 根据重复率评估等级
    pub fn from_ratio(ratio: f64) -> Self {
        match ratio {
            r if r < 0.05 => DuplicationGrade::A,
            r if r < 0.10 => DuplicationGrade::B,
            r if r < 0.20 => DuplicationGrade::C,
            r if r < 0.30 => DuplicationGrade::D,
            _ => DuplicationGrade::F,
        }
    }
}