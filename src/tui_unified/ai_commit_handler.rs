use std::collections::HashMap;

use crate::config::Config;
use crate::core::ai::agents::manager::AgentManager;
use crate::core::ai::agents::{AgentConfig, AgentContext, AgentTask, TaskType};
use crate::tui_unified::Result;

use super::app::AppMode;

impl super::app::TuiUnifiedApp {
    pub(crate) async fn enter_ai_commit_mode(&mut self) -> Result<()> {
        // 使用新的函数获取所有变更（包括未暂存的）
        let diff = match crate::git::get_all_changes_diff().await {
            Ok(diff) => {
                if diff.trim().is_empty() {
                    let mut state = self.state.write().await;
                    state.add_notification(
                        "No changes to commit".to_string(),
                        crate::tui_unified::state::app_state::NotificationLevel::Warning,
                    );
                    return Ok(());
                }
                diff
            }
            Err(e) => {
                let mut state = self.state.write().await;
                state.add_notification(
                    format!("Failed to get changes: {}", e),
                    crate::tui_unified::state::app_state::NotificationLevel::Error,
                );
                return Ok(());
            }
        };

        // 初始化 Agent Manager（如果还没有）
        if self.agent_manager.is_none() {
            let agent_manager = AgentManager::with_default_context();
            self.agent_manager = Some(agent_manager);
        }

        // 设置状态
        self.ai_commit_mode = true;
        self.ai_commit_status = Some("Generating commit message...".to_string());
        self.current_mode = AppMode::AICommit;

        // 显示 AI commit 模态框
        {
            let mut state = self.state.write().await;
            state.show_ai_commit_modal("".to_string(), "Generating commit message...".to_string());
        }

        // 生成 commit message
        self.generate_commit_message(diff).await
    }

    /// 生成 AI commit message
    async fn generate_commit_message(&mut self, diff: String) -> Result<()> {
        if let Some(ref mut agent_manager) = self.agent_manager {
            // 创建配置
            let config = Config::new();

            // 更新 Agent 配置
            let mut env_vars = std::env::vars().collect::<HashMap<String, String>>();

            // 添加 API Key 配置
            if let Some(api_key) = config.get_api_key() {
                env_vars.insert("API_KEY".to_string(), api_key);
            }

            // 设置 API URL
            let api_url = config.get_url();
            env_vars.insert("API_URL".to_string(), api_url);

            let agent_config = AgentConfig {
                provider: config.provider.clone(),
                model: config.model.clone(),
                temperature: 0.7,
                max_tokens: 2000,
                stream: true,
                max_retries: 3,
                timeout_secs: 60,
            };

            let context = AgentContext {
                working_dir: std::env::current_dir()?,
                env_vars,
                config: agent_config,
                history: vec![],
            };

            // 更新管理器上下文
            agent_manager.update_context(context);

            // 获取或创建 Commit Agent
            match agent_manager.get_or_create_agent("commit").await {
                Ok(commit_agent) => {
                    // 创建任务
                    let task = AgentTask::new(TaskType::GenerateCommit, diff);

                    // 执行任务
                    match commit_agent.execute(task, agent_manager.context()).await {
                        Ok(result) => {
                            if result.success {
                                self.ai_commit_message = Some(result.content.clone());
                                self.ai_commit_status =
                                    Some("Commit message generated successfully".to_string());

                                // 更新模态框内容
                                let mut state = self.state.write().await;
                                state.show_ai_commit_modal(
                                    result.content,
                                    "Commit message generated successfully".to_string(),
                                );
                            } else {
                                self.ai_commit_status =
                                    Some("Failed to generate commit message".to_string());

                                // 更新模态框内容
                                let mut state = self.state.write().await;
                                state.show_ai_commit_modal(
                                    "".to_string(),
                                    "Failed to generate commit message".to_string(),
                                );
                            }
                        }
                        Err(e) => {
                            self.ai_commit_status = Some(format!("Error: {}", e));

                            // 更新模态框内容
                            let mut state = self.state.write().await;
                            state.show_ai_commit_modal("".to_string(), format!("Error: {}", e));
                        }
                    }
                }
                Err(e) => {
                    self.ai_commit_status = Some(format!("Failed to create agent: {}", e));

                    // 更新模态框内容
                    let mut state = self.state.write().await;
                    state.show_ai_commit_modal(
                        "".to_string(),
                        format!("Failed to create agent: {}", e),
                    );
                }
            }
        }

        Ok(())
    }

    /// 确认并提交 AI 生成的 commit message
    pub(crate) async fn confirm_ai_commit(&mut self) -> Result<()> {
        if let Some(ref message) = self.ai_commit_message {
            // 首先检查是否有暂存的变更
            let staged_diff = match crate::git::get_git_diff().await {
                Ok(diff) => diff,
                Err(e) => {
                    let mut state = self.state.write().await;
                    state.add_notification(
                        format!("Failed to check staged changes: {}", e),
                        crate::tui_unified::state::app_state::NotificationLevel::Error,
                    );
                    return Ok(());
                }
            };

            // 如果没有暂存变更，先自动添加所有变更
            if staged_diff.trim().is_empty() {
                match crate::git::git_add_all().await {
                    Ok(_) => {
                        let mut state = self.state.write().await;
                        state.add_notification(
                            "Changes staged automatically".to_string(),
                            crate::tui_unified::state::app_state::NotificationLevel::Info,
                        );
                        drop(state);
                    }
                    Err(e) => {
                        let mut state = self.state.write().await;
                        state.add_notification(
                            format!("Failed to stage changes: {}", e),
                            crate::tui_unified::state::app_state::NotificationLevel::Error,
                        );
                        return Ok(());
                    }
                }
            }

            // 现在执行提交
            match crate::git::git_commit(message).await {
                Ok(_) => {
                    let mut state = self.state.write().await;
                    state.add_notification(
                        "Commit successful!".to_string(),
                        crate::tui_unified::state::app_state::NotificationLevel::Info,
                    );
                    drop(state);

                    // 重新加载 Git 数据以显示新的提交
                    if let Err(e) = self.reload_git_data().await {
                        let mut state = self.state.write().await;
                        state.add_notification(
                            format!("Failed to reload git data: {}", e),
                            crate::tui_unified::state::app_state::NotificationLevel::Warning,
                        );
                        drop(state);
                    }

                    // 显示推送提示而不是立即退出
                    self.ai_commit_push_prompt = true;
                    self.ai_commit_status = Some("Commit successful! Push to remote?".to_string());

                    // 显示推送提示模态框
                    let mut state = self.state.write().await;
                    state.show_ai_commit_push_modal("Commit successful!".to_string());
                }
                Err(e) => {
                    let mut state = self.state.write().await;
                    state.add_notification(
                        format!("Commit failed: {}", e),
                        crate::tui_unified::state::app_state::NotificationLevel::Error,
                    );
                }
            }
        }

        Ok(())
    }

    /// 退出 AI commit 模式
    pub(crate) fn exit_ai_commit_mode(&mut self) {
        self.ai_commit_mode = false;
        self.ai_commit_editing = false;
        self.ai_commit_message = None;
        self.ai_commit_status = None;
        self.ai_commit_push_prompt = false;
        self.current_mode = AppMode::Normal;

        // 重置编辑器状态
        self.commit_editor.set_focused(false);
        self.commit_editor.set_content("");
    }

    /// 确认推送到远程
    pub(crate) async fn confirm_push(&mut self) -> Result<()> {
        // 执行 git push
        match crate::git::git_push().await {
            Ok(_) => {
                let mut state = self.state.write().await;
                state.add_notification(
                    "Push successful!".to_string(),
                    crate::tui_unified::state::app_state::NotificationLevel::Success,
                );
                state.hide_modal();
                drop(state);

                // 完成推送后退出AI commit模式
                self.exit_ai_commit_mode();
            }
            Err(e) => {
                let mut state = self.state.write().await;
                state.add_notification(
                    format!("Push failed: {}", e),
                    crate::tui_unified::state::app_state::NotificationLevel::Error,
                );
                // 推送失败时不退出AI commit模式，让用户可以重试
                self.ai_commit_status = Some(format!("Push failed: {}", e));
                state.show_ai_commit_push_modal(format!("Push failed: {}. Try again?", e));
            }
        }

        Ok(())
    }

    pub(crate) async fn confirm_git_pull(&mut self) -> Result<()> {
        // 隐藏模态框
        {
            let mut state = self.state.write().await;
            state.hide_modal();
        }

        // 执行 git pull
        let result = tokio::process::Command::new("git")
            .arg("pull")
            .output()
            .await;

        match result {
            Ok(output) => {
                let mut state = self.state.write().await;
                if output.status.success() {
                    let pull_output = String::from_utf8_lossy(&output.stdout);
                    let notification_msg = if pull_output.contains("Already up to date") {
                        "Already up to date".to_string()
                    } else {
                        "Pull completed successfully!".to_string()
                    };

                    state.add_notification(
                        notification_msg,
                        crate::tui_unified::state::app_state::NotificationLevel::Success,
                    );
                    drop(state);

                    // 拉取成功后刷新git log
                    if let Err(e) = self
                        .refresh_current_view(
                            crate::tui_unified::state::app_state::ViewType::GitLog,
                        )
                        .await
                    {
                        let mut state = self.state.write().await;
                        state.add_notification(
                            format!("Failed to refresh git log: {}", e),
                            crate::tui_unified::state::app_state::NotificationLevel::Warning,
                        );
                    }
                } else {
                    let error_output = String::from_utf8_lossy(&output.stderr);
                    state.add_notification(
                        format!("Pull failed: {}", error_output),
                        crate::tui_unified::state::app_state::NotificationLevel::Error,
                    );
                }
            }
            Err(e) => {
                let mut state = self.state.write().await;
                state.add_notification(
                    format!("Failed to execute git pull: {}", e),
                    crate::tui_unified::state::app_state::NotificationLevel::Error,
                );
            }
        }

        Ok(())
    }

    /// 直接切换分支（仿照 tui_enhanced 的实现）
    pub(crate) async fn checkout_branch_directly(&mut self, branch_name: &str) -> Result<()> {
        let output = tokio::process::Command::new("git")
            .args(["checkout", branch_name])
            .output()
            .await?;

        let mut state = self.state.write().await;
        if output.status.success() {
            state.add_notification(
                format!("Switched to branch '{}'", branch_name),
                crate::tui_unified::state::app_state::NotificationLevel::Success,
            );
            drop(state);

            // 重新加载分支列表和提交记录
            let _ = self
                .refresh_current_view(crate::tui_unified::state::app_state::ViewType::Branches)
                .await;
            let _ = self
                .refresh_current_view(crate::tui_unified::state::app_state::ViewType::GitLog)
                .await;
        } else {
            let error = String::from_utf8_lossy(&output.stderr);
            state.add_notification(
                format!("Failed to switch branch: {}", error),
                crate::tui_unified::state::app_state::NotificationLevel::Error,
            );
        }

        Ok(())
    }

    pub(crate) async fn confirm_branch_switch(&mut self) -> Result<()> {
        // 获取待切换的分支名
        let branch_name = {
            let mut state = self.state.write().await;
            state.hide_modal();
            state.get_pending_branch_switch()
        };

        let branch_name = match branch_name {
            Some(name) => name,
            None => {
                let mut state = self.state.write().await;
                state.add_notification(
                    "No branch selected for switching".to_string(),
                    crate::tui_unified::state::app_state::NotificationLevel::Error,
                );
                return Ok(());
            }
        };

        // 执行分支切换
        let switch_result = tokio::process::Command::new("git")
            .arg("checkout")
            .arg(&branch_name)
            .output()
            .await;

        match switch_result {
            Ok(output) => {
                let mut state = self.state.write().await;
                if output.status.success() {
                    state.add_notification(
                        format!("Switched to branch '{}'", branch_name),
                        crate::tui_unified::state::app_state::NotificationLevel::Success,
                    );
                    drop(state);

                    // 分支切换成功后刷新相关视图
                    let _ = self
                        .refresh_current_view(
                            crate::tui_unified::state::app_state::ViewType::Branches,
                        )
                        .await;
                    let _ = self
                        .refresh_current_view(
                            crate::tui_unified::state::app_state::ViewType::GitLog,
                        )
                        .await;
                } else {
                    let error_output = String::from_utf8_lossy(&output.stderr);
                    state.add_notification(
                        format!("Failed to switch branch: {}", error_output),
                        crate::tui_unified::state::app_state::NotificationLevel::Error,
                    );
                }
            }
            Err(e) => {
                let mut state = self.state.write().await;
                state.add_notification(
                    format!("Failed to execute git checkout: {}", e),
                    crate::tui_unified::state::app_state::NotificationLevel::Error,
                );
            }
        }

        Ok(())
    }

    /// 跳过推送
    pub(crate) fn skip_push(&mut self) {
        // 关闭模态框并退出AI commit模式
        self.exit_ai_commit_mode();
    }
}
