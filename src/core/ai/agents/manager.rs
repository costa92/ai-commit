use super::*;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

// 日志宏定义在开始
macro_rules! info {
    ($($arg:tt)*) => {
        eprintln!("[INFO] {}", format!($($arg)*));
    };
}

macro_rules! error {
    ($($arg:tt)*) => {
        eprintln!("[ERROR] {}", format!($($arg)*));
    };
}

macro_rules! debug {
    ($($arg:tt)*) => {
        #[cfg(debug_assertions)]
        eprintln!("[DEBUG] {}", format!($($arg)*));
    };
}

/// Agent 管理器 - 负责管理和协调多个 Agent
pub struct AgentManager {
    agents: Arc<RwLock<HashMap<String, Arc<dyn Agent>>>>,
    context: AgentContext,
    task_queue: mpsc::UnboundedSender<AgentTask>,
    task_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<AgentTask>>>>,
    worker_handles: Vec<JoinHandle<()>>,
    worker_count: usize,
}

impl AgentManager {
    /// 创建新的 Agent 管理器
    pub fn new(context: AgentContext) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();

        Self {
            agents: Arc::new(RwLock::new(HashMap::new())),
            context,
            task_queue: tx,
            task_receiver: Arc::new(RwLock::new(Some(rx))),
            worker_handles: Vec::new(),
            worker_count: 4, // 默认 4 个工作线程
        }
    }

    /// 创建带默认上下文的 Agent 管理器
    pub fn with_default_context() -> Self {
        let context = AgentContext {
            working_dir: std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from(".")),
            env_vars: std::env::vars().collect(),
            config: AgentConfig::default(),
            history: vec![],
        };
        Self::new(context)
    }

    /// 获取或创建 Agent（简化接口）
    pub async fn get_or_create_agent(&mut self, agent_type: &str) -> Result<Arc<dyn Agent>> {
        let agent_name = match agent_type {
            "commit" => "CommitAgent",
            "tag" => "TagAgent",
            "review" => "ReviewAgent",
            "refactor" => "RefactorAgent",
            _ => agent_type,
        };

        // 首先尝试获取已存在的 Agent
        {
            let agents = self.agents.read().unwrap();
            if let Some(agent) = agents.get(agent_name) {
                return Ok(agent.clone());
            }
        }

        // Agent 不存在，创建新的
        self.register_agent(agent_type).await?;
        let agents = self.agents.read().unwrap();
        agents
            .get(agent_name)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Failed to create agent: {}", agent_type))
    }

    /// 更新 Agent 上下文
    pub fn update_context(&mut self, context: AgentContext) {
        self.context = context;
    }

    /// 获取当前上下文的引用
    pub fn context(&self) -> &AgentContext {
        &self.context
    }

    /// 设置工作线程数量
    pub fn with_worker_count(mut self, count: usize) -> Self {
        self.worker_count = count.max(1);
        self
    }

    /// 注册 Agent
    pub async fn register_agent(&mut self, agent_type: &str) -> Result<()> {
        let mut agent = AgentFactory::create(agent_type)?;
        agent.initialize(&self.context).await?;

        let agent_name = agent.name().to_string();
        let mut agents = self.agents.write().unwrap();
        agents.insert(agent_name.clone(), Arc::from(agent));

        info!("Registered agent: {}", agent_name);
        Ok(())
    }

    /// 注册自定义 Agent
    pub async fn register_custom_agent(&mut self, mut agent: Box<dyn Agent>) -> Result<()> {
        agent.initialize(&self.context).await?;

        let agent_name = agent.name().to_string();
        let mut agents = self.agents.write().unwrap();
        agents.insert(agent_name.clone(), Arc::from(agent));

        info!("Registered custom agent: {}", agent_name);
        Ok(())
    }

    /// 获取 Agent
    pub fn get_agent(&self, name: &str) -> Option<Arc<dyn Agent>> {
        let agents = self.agents.read().unwrap();
        agents.get(name).cloned()
    }

    /// 列出所有已注册的 Agent
    pub fn list_agents(&self) -> Vec<String> {
        let agents = self.agents.read().unwrap();
        agents.keys().cloned().collect()
    }

    /// 提交任务到队列
    pub fn submit_task(&self, task: AgentTask) -> Result<()> {
        self.task_queue
            .send(task)
            .map_err(|e| anyhow::anyhow!("Failed to submit task: {}", e))
    }

    /// 执行单个任务（同步等待）
    pub async fn execute_task(&self, agent_name: &str, task: AgentTask) -> Result<AgentResult> {
        let agent = self
            .get_agent(agent_name)
            .ok_or_else(|| anyhow::anyhow!("Agent not found: {}", agent_name))?;

        agent.execute(task, &self.context).await
    }

    /// 启动工作线程池
    pub fn start_workers(&mut self) {
        // 只能启动一次
        if !self.worker_handles.is_empty() {
            return;
        }

        // 获取接收器
        let rx = self.task_receiver.write().unwrap().take();
        if rx.is_none() {
            error!("Task receiver already taken");
            return;
        }

        let rx = Arc::new(tokio::sync::Mutex::new(rx.unwrap()));

        // 启动工作线程
        for i in 0..self.worker_count {
            let agents = self.agents.clone();
            let context = self.context.clone();
            let rx = rx.clone();

            let handle = tokio::spawn(async move {
                info!("Worker {} started", i);

                loop {
                    // 从队列获取任务
                    let task = {
                        let mut receiver = rx.lock().await;
                        receiver.recv().await
                    };

                    if let Some(task) = task {
                        debug!("Worker {} processing task: {}", i, task.id);

                        // 根据任务类型选择 Agent
                        let agent_name = Self::select_agent_for_task(&task);

                        // 获取 Agent
                        let agent = {
                            let agents_guard = agents.read().unwrap();
                            agents_guard.get(&agent_name).cloned()
                        };

                        if let Some(agent) = agent {
                            // 执行任务
                            match agent.execute(task.clone(), &context).await {
                                Ok(_result) => {
                                    info!("Task {} completed successfully", task.id);
                                }
                                Err(e) => {
                                    error!("Task {} failed: {}", task.id, e);
                                }
                            }
                        } else {
                            error!("Agent {} not found for task {}", agent_name, task.id);
                        }
                    } else {
                        // 通道关闭，退出工作线程
                        info!("Worker {} shutting down", i);
                        break;
                    }
                }
            });

            self.worker_handles.push(handle);
        }

        info!("Started {} workers", self.worker_count);
    }

    /// 停止所有工作线程
    pub async fn stop_workers(&mut self) {
        // 关闭任务队列
        // drop(self.task_queue); // 发送端会自动关闭

        // 等待所有工作线程完成
        for handle in self.worker_handles.drain(..) {
            let _ = handle.await;
        }

        info!("All workers stopped");
    }

    /// 根据任务类型选择合适的 Agent
    fn select_agent_for_task(task: &AgentTask) -> String {
        match task.task_type {
            TaskType::GenerateCommit => "CommitAgent".to_string(),
            TaskType::GenerateTag => "TagAgent".to_string(),
            TaskType::ReviewCode => "ReviewAgent".to_string(),
            TaskType::RefactorSuggestion => "RefactorAgent".to_string(),
            TaskType::GenerateDocumentation => "TagAgent".to_string(), // TagAgent 也处理文档
            TaskType::GenerateTests => "ReviewAgent".to_string(),      // ReviewAgent 也生成测试
            TaskType::Custom(ref name) => name.clone(),
        }
    }

    /// 执行协作任务（多个 Agent 协同工作）
    pub async fn execute_collaborative_task(
        &self,
        agents: Vec<&str>,
        initial_task: AgentTask,
    ) -> Result<Vec<AgentResult>> {
        let mut results = Vec::new();
        let mut current_input = initial_task.input.clone();

        for agent_name in agents {
            let agent = self
                .get_agent(agent_name)
                .ok_or_else(|| anyhow::anyhow!("Agent not found: {}", agent_name))?;

            // 创建任务，使用上一个 Agent 的输出作为输入
            let mut task = initial_task.clone();
            task.input = current_input.clone();

            // 执行任务
            let result = agent.execute(task, &self.context).await?;

            // 使用输出作为下一个 Agent 的输入
            current_input = result.content.clone();
            results.push(result);
        }

        Ok(results)
    }

    /// 获取 Agent 状态报告
    pub fn get_status_report(&self) -> StatusReport {
        let agents = self.agents.read().unwrap();

        let agent_statuses: HashMap<String, AgentStatus> = agents
            .iter()
            .map(|(name, agent)| (name.clone(), agent.status()))
            .collect();

        StatusReport {
            total_agents: agents.len(),
            agent_statuses,
            worker_count: self.worker_count,
            queue_size: 0, // TODO: 实现队列大小统计
        }
    }
}

/// 状态报告
#[derive(Debug)]
pub struct StatusReport {
    pub total_agents: usize,
    pub agent_statuses: HashMap<String, AgentStatus>,
    pub worker_count: usize,
    pub queue_size: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn create_test_context() -> AgentContext {
        AgentContext {
            working_dir: PathBuf::from("/tmp"),
            env_vars: HashMap::new(),
            config: AgentConfig::default(),
            history: Vec::new(),
        }
    }

    #[tokio::test]
    async fn test_manager_creation() {
        let context = create_test_context();
        let manager = AgentManager::new(context);

        assert_eq!(manager.worker_count, 4);
        assert_eq!(manager.list_agents().len(), 0);
    }

    #[tokio::test]
    async fn test_register_agent() {
        let context = create_test_context();
        let mut manager = AgentManager::new(context);

        manager.register_agent("commit").await.unwrap();
        let agents = manager.list_agents();

        assert_eq!(agents.len(), 1);
        assert!(agents.contains(&"CommitAgent".to_string()));
    }

    #[tokio::test]
    async fn test_get_agent() {
        let context = create_test_context();
        let mut manager = AgentManager::new(context);

        manager.register_agent("commit").await.unwrap();

        assert!(manager.get_agent("CommitAgent").is_some());
        assert!(manager.get_agent("NonExistent").is_none());
    }

    #[tokio::test]
    async fn test_submit_task() {
        let context = create_test_context();
        let manager = AgentManager::new(context);

        let task = AgentTask::new(TaskType::GenerateCommit, "test diff");
        assert!(manager.submit_task(task).is_ok());
    }

    #[test]
    fn test_select_agent_for_task() {
        let task = AgentTask::new(TaskType::GenerateCommit, "test");
        assert_eq!(AgentManager::select_agent_for_task(&task), "CommitAgent");

        let task = AgentTask::new(TaskType::GenerateTag, "test");
        assert_eq!(AgentManager::select_agent_for_task(&task), "TagAgent");
    }

    #[tokio::test]
    async fn test_status_report() {
        let context = create_test_context();
        let mut manager = AgentManager::new(context);

        manager.register_agent("commit").await.unwrap();
        manager.register_agent("tag").await.unwrap();

        let report = manager.get_status_report();
        assert_eq!(report.total_agents, 2);
        assert_eq!(report.worker_count, 4);
    }
}
