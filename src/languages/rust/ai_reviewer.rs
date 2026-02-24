use crate::ai;
use crate::config::Config;
use crate::languages::LanguageFeature;

/// Rust 专用的 AI 代码审查器
pub struct RustAIReviewer {
    config: Config,
}

impl Default for RustAIReviewer {
    fn default() -> Self {
        Self::new(Config::default())
    }
}

impl RustAIReviewer {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub fn with_config(config: Config) -> Self {
        Self::new(config)
    }

    /// 生成 Rust 特定的审查提示词
    pub fn generate_review_prompt(
        &self,
        review_type: &str,
        features: &[LanguageFeature],
        file_path: &str,
    ) -> String {
        let features_summary = if features.is_empty() {
            "未检测到特定代码特征".to_string()
        } else {
            features
                .iter()
                .map(|f| {
                    format!(
                        "{}:{} (第{}行)",
                        f.feature_type,
                        f.name,
                        f.line_number.unwrap_or(0)
                    )
                })
                .collect::<Vec<_>>()
                .join(", ")
        };

        match review_type {
            "comprehensive" => format!(
                "请对以下Rust代码进行全面的Rust代码审查，包括安全性分析、性能评估、内存管理和最佳实践：\n\n\
                文件路径: {}\n\
                检测到的特征: {}\n\n\
                请提供JSON格式的响应，包含overall_score、summary、detailed_feedback、security_score、performance_score、maintainability_score、recommendations和learning_resources字段。",
                file_path, features_summary
            ),
            "security" => format!(
                "请对以下Rust代码进行Rust安全性审查，重点关注：\n\
                1. 内存安全检查\n\
                2. 类型安全验证\n\
                3. 并发安全分析\n\
                4. unsafe代码块审查\n\
                5. 借用检查器相关问题\n\n\
                文件路径: {}\n\
                检测到的特征: {}\n\n\
                请提供JSON格式的响应。",
                file_path, features_summary
            ),
            "performance" => format!(
                "请对以下Rust代码进行Rust性能审查，重点关注：\n\
                1. 零成本抽象应用\n\
                2. 内存分配优化\n\
                3. 算法复杂度分析\n\
                4. 编译时优化机会\n\
                5. 运行时性能瓶颈\n\n\
                文件路径: {}\n\
                检测到的特征: {}\n\n\
                请提供JSON格式的响应。",
                file_path, features_summary
            ),
            "architecture" => format!(
                "请对以下Rust代码进行Rust架构审查，重点关注：\n\
                1. 模块设计和组织\n\
                2. trait设计模式\n\
                3. 错误处理策略\n\
                4. API设计原则\n\
                5. 生命周期管理\n\n\
                文件路径: {}\n\
                检测到的特征: {}\n\n\
                请提供JSON格式的响应。",
                file_path, features_summary
            ),
            _ => format!(
                "请对以下Rust代码进行全面的Rust代码审查：\n\n\
                文件路径: {}\n\
                检测到的特征: {}\n\n\
                请提供JSON格式的响应。",
                file_path, features_summary
            ),
        }
    }

    /// 解析AI响应
    pub fn parse_ai_response(
        &self,
        review_type: &str,
        response: &str,
    ) -> anyhow::Result<crate::languages::review_service_v2::AIReviewResult> {
        // 尝试解析JSON响应
        if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(response) {
            let overall_score = json_value
                .get("overall_score")
                .and_then(|v| v.as_f64())
                .unwrap_or(7.0) as f32;
            let summary = json_value
                .get("summary")
                .and_then(|v| v.as_str())
                .unwrap_or("Rust代码审查完成")
                .to_string();
            let detailed_feedback = json_value
                .get("detailed_feedback")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let security_score = json_value
                .get("security_score")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0) as f32;
            let performance_score = json_value
                .get("performance_score")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0) as f32;
            let maintainability_score = json_value
                .get("maintainability_score")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0) as f32;

            let recommendations = json_value
                .get("recommendations")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str())
                        .map(|s| s.to_string())
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();

            let learning_resources = json_value
                .get("learning_resources")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str())
                        .map(|s| s.to_string())
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();

            Ok(crate::languages::review_service_v2::AIReviewResult {
                review_type: format!("rust_{}", review_type),
                overall_score,
                summary,
                detailed_feedback,
                security_score,
                performance_score,
                maintainability_score,
                recommendations,
                learning_resources,
            })
        } else {
            // 如果JSON解析失败，创建基础响应
            Ok(crate::languages::review_service_v2::AIReviewResult {
                review_type: format!("rust_{}", review_type),
                overall_score: 5.0,
                summary: "AI响应解析失败".to_string(),
                detailed_feedback: if response.is_empty() {
                    "响应为空".to_string()
                } else {
                    format!("原始响应: {}", response)
                },
                security_score: 0.0,
                performance_score: 0.0,
                maintainability_score: 0.0,
                recommendations: vec![],
                learning_resources: vec![],
            })
        }
    }

    /// 执行代码审查
    pub async fn review_code(
        &self,
        review_type: &str,
        features: &[LanguageFeature],
        file_path: &str,
    ) -> anyhow::Result<crate::languages::review_service_v2::AIReviewResult> {
        let prompt = self.generate_review_prompt(review_type, features, file_path);

        // 这里应该调用AI服务，但为了测试目的，我们返回一个模拟响应
        // 在真实环境中，这会调用实际的AI API
        let ai_response = ai::generate_commit_message("", &self.config, &prompt).await?;

        self.parse_ai_response(review_type, &ai_response)
    }
}

#[cfg(test)]
mod tests {
    include!("ai_reviewer_tests.rs");
}
