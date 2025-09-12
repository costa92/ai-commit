use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod commit_agent;
pub mod manager;
pub mod refactor_agent;
pub mod review_agent;
pub mod tag_agent;

pub use commit_agent::CommitAgent;
pub use manager::AgentManager;
pub use refactor_agent::RefactorAgent;
pub use review_agent::ReviewAgent;
pub use tag_agent::TagAgent;

/// Agent 执行上下文
#[derive(Debug, Clone)]
pub struct AgentContext {
    /// 当前工作目录
    pub working_dir: std::path::PathBuf,
    /// 环境变量
    pub env_vars: HashMap<String, String>,
    /// 配置参数
    pub config: AgentConfig,
    /// 会话历史
    pub history: Vec<AgentMessage>,
}

/// Agent 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// AI 提供商
    pub provider: String,
    /// 模型名称
    pub model: String,
    /// 温度参数
    pub temperature: f32,
    /// 最大 token 数
    pub max_tokens: u32,
    /// 是否启用流式输出
    pub stream: bool,
    /// 重试次数
    pub max_retries: u32,
    /// 超时时间（秒）
    pub timeout_secs: u64,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            provider: "ollama".to_string(),
            model: "mistral".to_string(),
            temperature: 0.7,
            max_tokens: 2000,
            stream: true,
            max_retries: 3,
            timeout_secs: 60,
        }
    }
}

/// Agent 消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMessage {
    /// 角色（system/user/assistant）
    pub role: MessageRole,
    /// 消息内容
    pub content: String,
    /// 时间戳
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// 元数据
    pub metadata: HashMap<String, String>,
}

/// 消息角色
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageRole {
    System,
    User,
    Assistant,
}

/// Agent 执行结果
#[derive(Debug, Clone)]
pub struct AgentResult {
    /// 是否成功
    pub success: bool,
    /// 结果内容
    pub content: String,
    /// 执行耗时（毫秒）
    pub duration_ms: u64,
    /// 使用的 token 数
    pub tokens_used: Option<u32>,
    /// 附加数据
    pub data: HashMap<String, serde_json::Value>,
}

/// Agent 能力
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AgentCapability {
    /// 生成提交消息
    GenerateCommit,
    /// 生成标签说明
    GenerateTag,
    /// 代码审查
    CodeReview,
    /// 代码重构建议
    RefactorSuggestion,
    /// 生成文档
    GenerateDoc,
    /// 生成测试
    GenerateTest,
    /// 分析代码
    AnalyzeCode,
    /// 问答
    QuestionAnswer,
}

/// Agent 基础 trait
#[async_trait]
pub trait Agent: Send + Sync {
    /// 获取 Agent 名称
    fn name(&self) -> &str;

    /// 获取 Agent 描述
    fn description(&self) -> &str;

    /// 获取 Agent 能力列表
    fn capabilities(&self) -> Vec<AgentCapability>;

    /// 检查是否支持某个能力
    fn has_capability(&self, capability: &AgentCapability) -> bool {
        self.capabilities().contains(capability)
    }

    /// 初始化 Agent
    async fn initialize(&mut self, context: &AgentContext) -> Result<()>;

    /// 执行任务
    async fn execute(&self, task: AgentTask, context: &AgentContext) -> Result<AgentResult>;

    /// 验证任务是否可执行
    fn validate_task(&self, task: &AgentTask) -> Result<()> {
        // 默认验证：检查必需参数
        if task.input.is_empty() {
            anyhow::bail!("Task input cannot be empty");
        }
        Ok(())
    }

    /// 获取 Agent 状态
    fn status(&self) -> AgentStatus {
        AgentStatus::Ready
    }
}

/// Agent 状态
#[derive(Debug, Clone, PartialEq)]
pub enum AgentStatus {
    /// 就绪
    Ready,
    /// 执行中
    Running,
    /// 暂停
    Paused,
    /// 错误
    Error(String),
    /// 未初始化
    Uninitialized,
}

/// Agent 任务
#[derive(Debug, Clone)]
pub struct AgentTask {
    /// 任务 ID
    pub id: String,
    /// 任务类型
    pub task_type: TaskType,
    /// 输入内容
    pub input: String,
    /// 任务参数
    pub params: HashMap<String, String>,
    /// 优先级（0-10，10 最高）
    pub priority: u8,
    /// 创建时间
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl AgentTask {
    /// 创建新任务
    pub fn new(task_type: TaskType, input: impl Into<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            task_type,
            input: input.into(),
            params: HashMap::new(),
            priority: 5,
            created_at: chrono::Utc::now(),
        }
    }

    /// 设置参数
    pub fn with_param(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.params.insert(key.into(), value.into());
        self
    }

    /// 设置优先级
    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority.min(10);
        self
    }
}

/// 任务类型
#[derive(Debug, Clone, PartialEq)]
pub enum TaskType {
    /// 生成提交消息
    GenerateCommit,
    /// 生成标签
    GenerateTag,
    /// 审查代码
    ReviewCode,
    /// 重构建议
    RefactorSuggestion,
    /// 生成文档
    GenerateDocumentation,
    /// 生成测试
    GenerateTests,
    /// 自定义任务
    Custom(String),
}

/// Agent 工厂
pub struct AgentFactory;

impl AgentFactory {
    /// 创建 Agent
    pub fn create(agent_type: &str) -> Result<Box<dyn Agent>> {
        match agent_type.to_lowercase().as_str() {
            "commit" => Ok(Box::new(CommitAgent::new())),
            "tag" => Ok(Box::new(TagAgent::new())),
            "review" => Ok(Box::new(ReviewAgent::new())),
            "refactor" => Ok(Box::new(RefactorAgent::new())),
            _ => anyhow::bail!("Unknown agent type: {}", agent_type),
        }
    }

    /// 获取所有可用的 Agent 类型
    pub fn available_agents() -> Vec<&'static str> {
        vec!["commit", "tag", "review", "refactor"]
    }
}

// 使用 uuid crate 生成唯一 ID
use uuid;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_config_default() {
        let config = AgentConfig::default();
        assert_eq!(config.provider, "ollama");
        assert_eq!(config.model, "mistral");
        assert_eq!(config.temperature, 0.7);
    }

    #[test]
    fn test_agent_task_creation() {
        let task = AgentTask::new(TaskType::GenerateCommit, "test input")
            .with_param("key", "value")
            .with_priority(8);

        assert_eq!(task.task_type, TaskType::GenerateCommit);
        assert_eq!(task.input, "test input");
        assert_eq!(task.params.get("key"), Some(&"value".to_string()));
        assert_eq!(task.priority, 8);
    }

    #[test]
    fn test_agent_factory() {
        assert!(AgentFactory::create("commit").is_ok());
        assert!(AgentFactory::create("tag").is_ok());
        assert!(AgentFactory::create("unknown").is_err());
    }

    #[test]
    fn test_agent_capability() {
        let cap1 = AgentCapability::GenerateCommit;
        let cap2 = AgentCapability::GenerateCommit;
        assert_eq!(cap1, cap2);
    }
}
