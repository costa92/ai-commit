use super::*;
use crate::core::ai::provider::{AIProvider, ProviderConfig};
use async_trait::async_trait;
use std::sync::Arc;
use std::time::Instant;

/// 重构建议 Agent
pub struct RefactorAgent {
    name: String,
    description: String,
    provider: Option<Arc<dyn AIProvider>>,
    status: AgentStatus,
    config: AgentConfig,
}

impl Default for RefactorAgent {
    fn default() -> Self {
        Self::new()
    }
}

impl RefactorAgent {
    pub fn new() -> Self {
        Self {
            name: "RefactorAgent".to_string(),
            description: "提供代码重构建议和优化方案".to_string(),
            provider: None,
            status: AgentStatus::Uninitialized,
            config: AgentConfig::default(),
        }
    }

    async fn suggest_refactoring(&self, code: &str, context: &AgentContext) -> Result<String> {
        let provider = self
            .provider
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("AI provider not initialized"))?;

        let prompt = format!(
            "分析以下代码变更并提供重构建议，按以下结构输出：\n\n\
            ## 重构目标\n\
            概述当前代码的主要问题和重构方向。\n\n\
            ## 具体步骤\n\
            列出可执行的重构步骤，每步包含：\n\
            - 做什么（目标）\n\
            - 怎么做（方法）\n\
            - 为什么（理由）\n\n\
            ## 设计模式建议\n\
            如果适用，推荐可以应用的设计模式或架构改进。\n\n\
            ## 预期效果\n\
            重构后预期的改进（可读性、性能、可维护性等）。\n\n\
            代码变更：\n{}",
            code
        );

        let provider_config = ProviderConfig {
            model: context.config.model.clone(),
            api_key: context.env_vars.get("API_KEY").cloned(),
            api_url: context
                .env_vars
                .get("API_URL")
                .unwrap_or(&"http://localhost:11434".to_string())
                .clone(),
            timeout_secs: context.config.timeout_secs,
            max_retries: context.config.max_retries,
            stream: false,
        };

        provider.generate(&prompt, &provider_config).await
    }
}

#[async_trait]
impl Agent for RefactorAgent {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn capabilities(&self) -> Vec<AgentCapability> {
        vec![
            AgentCapability::RefactorSuggestion,
            AgentCapability::AnalyzeCode,
        ]
    }

    async fn initialize(&mut self, context: &AgentContext) -> Result<()> {
        use crate::core::ai::provider::ProviderFactory;

        let provider = ProviderFactory::create(&context.config.provider)?;
        self.provider = Some(Arc::from(provider));
        self.config = context.config.clone();
        self.status = AgentStatus::Ready;

        Ok(())
    }

    async fn execute(&self, task: AgentTask, context: &AgentContext) -> Result<AgentResult> {
        self.validate_task(&task)?;

        let start_time = Instant::now();

        let result = match task.task_type {
            TaskType::RefactorSuggestion => {
                let suggestions = self.suggest_refactoring(&task.input, context).await?;

                AgentResult {
                    success: true,
                    content: suggestions,
                    duration_ms: start_time.elapsed().as_millis() as u64,
                    tokens_used: None,
                    data: HashMap::new(),
                }
            }
            _ => {
                anyhow::bail!("Unsupported task type: {:?}", task.task_type);
            }
        };

        Ok(result)
    }

    fn status(&self) -> AgentStatus {
        self.status.clone()
    }
}
