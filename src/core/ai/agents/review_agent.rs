use super::*;
use crate::core::ai::provider::{AIProvider, ProviderConfig};
use async_trait::async_trait;
use std::sync::Arc;
use std::time::Instant;

/// 代码审查 Agent
pub struct ReviewAgent {
    name: String,
    description: String,
    provider: Option<Arc<dyn AIProvider>>,
    status: AgentStatus,
    config: AgentConfig,
}

impl Default for ReviewAgent {
    fn default() -> Self {
        Self::new()
    }
}

impl ReviewAgent {
    pub fn new() -> Self {
        Self {
            name: "ReviewAgent".to_string(),
            description: "智能代码审查和测试生成".to_string(),
            provider: None,
            status: AgentStatus::Uninitialized,
            config: AgentConfig::default(),
        }
    }

    async fn review_code(&self, code: &str, context: &AgentContext) -> Result<String> {
        let provider = self
            .provider
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("AI provider not initialized"))?;

        let prompt = format!(
            "请审查以下代码，提供改进建议：\n\
            1. 潜在的 bug\n\
            2. 性能问题\n\
            3. 代码风格\n\
            4. 安全问题\n\n\
            代码：\n{}",
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
impl Agent for ReviewAgent {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn capabilities(&self) -> Vec<AgentCapability> {
        vec![
            AgentCapability::CodeReview,
            AgentCapability::GenerateTest,
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
            TaskType::ReviewCode => {
                let review = self.review_code(&task.input, context).await?;

                AgentResult {
                    success: true,
                    content: review,
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
