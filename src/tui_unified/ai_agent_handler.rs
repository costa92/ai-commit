use std::collections::HashMap;

use crate::config::Config;
use crate::core::ai::agents::manager::AgentManager;
use crate::core::ai::agents::{AgentConfig, AgentContext, AgentTask, TaskType};
use crate::tui_unified::state::app_state::NotificationLevel;
use crate::tui_unified::Result;

impl super::app::TuiUnifiedApp {
    /// 进入代码审查模式
    pub(crate) async fn enter_review_mode(&mut self) -> Result<()> {
        let code = match self.get_reviewable_code().await {
            Ok(code) => {
                if code.trim().is_empty() {
                    let mut state = self.state.write().await;
                    state.add_notification(
                        "No code changes to review".to_string(),
                        NotificationLevel::Warning,
                    );
                    return Ok(());
                }
                code
            }
            Err(e) => {
                let mut state = self.state.write().await;
                state.add_notification(
                    format!("Failed to get code for review: {}", e),
                    NotificationLevel::Error,
                );
                return Ok(());
            }
        };

        // 初始化 Agent Manager
        self.ensure_agent_manager();

        // 显示加载模态框
        {
            let mut state = self.state.write().await;
            state.show_ai_review_modal("Analyzing code...".to_string());
        }

        // 执行代码审查
        self.execute_review(code).await
    }

    /// 进入重构建议模式
    pub(crate) async fn enter_refactor_mode(&mut self) -> Result<()> {
        let code = match self.get_reviewable_code().await {
            Ok(code) => {
                if code.trim().is_empty() {
                    let mut state = self.state.write().await;
                    state.add_notification(
                        "No code changes to analyze".to_string(),
                        NotificationLevel::Warning,
                    );
                    return Ok(());
                }
                code
            }
            Err(e) => {
                let mut state = self.state.write().await;
                state.add_notification(
                    format!("Failed to get code for refactoring: {}", e),
                    NotificationLevel::Error,
                );
                return Ok(());
            }
        };

        // 初始化 Agent Manager
        self.ensure_agent_manager();

        // 显示加载模态框
        {
            let mut state = self.state.write().await;
            state.show_ai_refactor_modal(
                "Analyzing code for refactoring suggestions...".to_string(),
            );
        }

        // 执行重构分析
        self.execute_refactor(code).await
    }

    /// 获取可审查的代码（diff）
    async fn get_reviewable_code(&self) -> anyhow::Result<String> {
        let state = self.state.read().await;

        // 优先级 1: 如果在 GitLog 视图且有选中的 commit，获取该 commit 的 diff
        if state.current_view == crate::tui_unified::state::app_state::ViewType::GitLog {
            if let Some(ref commit_hash) = state.selected_items.selected_commit {
                let hash = commit_hash.clone();
                drop(state);
                let output = tokio::process::Command::new("git")
                    .args(["show", "--stat", "--patch", &hash])
                    .output()
                    .await?;
                if output.status.success() {
                    let diff = String::from_utf8_lossy(&output.stdout).to_string();
                    if !diff.trim().is_empty() {
                        return Ok(diff);
                    }
                }
                return Err(anyhow::anyhow!("Failed to get diff for commit {}", hash));
            }
        }
        drop(state);

        // 优先级 2: 获取已暂存变更
        let staged_output = tokio::process::Command::new("git")
            .args(["diff", "--cached"])
            .output()
            .await?;
        if staged_output.status.success() {
            let diff = String::from_utf8_lossy(&staged_output.stdout).to_string();
            if !diff.trim().is_empty() {
                return Ok(diff);
            }
        }

        // 优先级 3: 获取所有变更
        crate::git::get_all_changes_diff().await
    }

    /// 确保 AgentManager 已初始化
    fn ensure_agent_manager(&mut self) {
        if self.agent_manager.is_none() {
            self.agent_manager = Some(AgentManager::with_default_context());
        }
    }

    /// 构建 Agent 上下文
    fn build_agent_context() -> anyhow::Result<AgentContext> {
        let config = Config::new();
        let mut env_vars: HashMap<String, String> = std::env::vars().collect();

        if let Some(api_key) = config.get_api_key() {
            env_vars.insert("API_KEY".to_string(), api_key);
        }
        env_vars.insert("API_URL".to_string(), config.get_url());

        let agent_config = AgentConfig {
            provider: config.provider.clone(),
            model: config.model.clone(),
            temperature: 0.7,
            max_tokens: 4000,
            stream: false,
            max_retries: 3,
            timeout_secs: 120,
        };

        Ok(AgentContext {
            working_dir: std::env::current_dir()?,
            env_vars,
            config: agent_config,
            history: vec![],
        })
    }

    /// 执行代码审查
    async fn execute_review(&mut self, code: String) -> Result<()> {
        if let Some(ref mut agent_manager) = self.agent_manager {
            match Self::build_agent_context() {
                Ok(context) => {
                    agent_manager.update_context(context);
                }
                Err(e) => {
                    let mut state = self.state.write().await;
                    state.show_ai_review_modal(format!("Failed to build context: {}", e));
                    return Ok(());
                }
            }

            match agent_manager.get_or_create_agent("review").await {
                Ok(review_agent) => {
                    let task = AgentTask::new(TaskType::ReviewCode, code);
                    match review_agent.execute(task, agent_manager.context()).await {
                        Ok(result) => {
                            let mut state = self.state.write().await;
                            if result.success {
                                state.show_ai_review_modal(result.content);
                            } else {
                                state.show_ai_review_modal(
                                    "Code review failed: no result returned".to_string(),
                                );
                            }
                        }
                        Err(e) => {
                            let mut state = self.state.write().await;
                            state.show_ai_review_modal(format!("Review error: {}", e));
                        }
                    }
                }
                Err(e) => {
                    let mut state = self.state.write().await;
                    state.show_ai_review_modal(format!("Failed to create review agent: {}", e));
                }
            }
        }
        Ok(())
    }

    /// 执行重构分析
    async fn execute_refactor(&mut self, code: String) -> Result<()> {
        if let Some(ref mut agent_manager) = self.agent_manager {
            match Self::build_agent_context() {
                Ok(context) => {
                    agent_manager.update_context(context);
                }
                Err(e) => {
                    let mut state = self.state.write().await;
                    state.show_ai_refactor_modal(format!("Failed to build context: {}", e));
                    return Ok(());
                }
            }

            match agent_manager.get_or_create_agent("refactor").await {
                Ok(refactor_agent) => {
                    let task = AgentTask::new(TaskType::RefactorSuggestion, code);
                    match refactor_agent.execute(task, agent_manager.context()).await {
                        Ok(result) => {
                            let mut state = self.state.write().await;
                            if result.success {
                                state.show_ai_refactor_modal(result.content);
                            } else {
                                state.show_ai_refactor_modal(
                                    "Refactor analysis failed: no result returned".to_string(),
                                );
                            }
                        }
                        Err(e) => {
                            let mut state = self.state.write().await;
                            state.show_ai_refactor_modal(format!("Refactor error: {}", e));
                        }
                    }
                }
                Err(e) => {
                    let mut state = self.state.write().await;
                    state.show_ai_refactor_modal(format!("Failed to create refactor agent: {}", e));
                }
            }
        }
        Ok(())
    }
}
