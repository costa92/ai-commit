use crate::diff_viewer::DiffViewer;
use crate::tui_unified::components::base::component::Component;
use crate::tui_unified::git::interface::GitRepositoryAPI;
use crate::tui_unified::Result;

/// 将 git interface 的 Commit 转换为 TUI state 的 Commit
fn convert_commits(
    commits_data: Vec<crate::tui_unified::git::models::Commit>,
) -> Vec<crate::tui_unified::state::git_state::Commit> {
    commits_data
        .into_iter()
        .map(|c| crate::tui_unified::state::git_state::Commit {
            hash: c.hash.clone(),
            short_hash: c.hash[..8.min(c.hash.len())].to_string(),
            author: c.author.clone(),
            author_email: format!("{}@example.com", c.author),
            committer: c.author.clone(),
            committer_email: format!("{}@example.com", c.author),
            date: chrono::DateTime::parse_from_str(
                &format!("{} 00:00:00 +0000", c.date),
                "%Y-%m-%d %H:%M:%S %z",
            )
            .unwrap_or_else(|_| chrono::Utc::now().into())
            .with_timezone(&chrono::Utc),
            message: c.message.clone(),
            subject: c.message,
            body: None,
            parents: Vec::new(),
            refs: Vec::new(),
            files_changed: c.files_changed as usize,
            insertions: 0,
            deletions: 0,
        })
        .collect()
}

/// 将 git interface 的 Branch 转换为 TUI state 的 Branch
fn convert_branches(
    branches_data: Vec<crate::tui_unified::git::models::Branch>,
) -> Vec<crate::tui_unified::state::git_state::Branch> {
    branches_data
        .into_iter()
        .map(|b| crate::tui_unified::state::git_state::Branch {
            name: b.name.clone(),
            full_name: format!("refs/heads/{}", b.name),
            is_current: b.is_current,
            is_remote: false,
            upstream: b.upstream,
            last_commit: None,
            ahead_count: 0,
            behind_count: 0,
            last_updated: chrono::Utc::now(),
        })
        .collect()
}

/// 将 git interface 的 Tag 转换为 TUI state 的 Tag
fn convert_tags(
    tags_data: Vec<crate::tui_unified::git::models::Tag>,
) -> Vec<crate::tui_unified::state::git_state::Tag> {
    tags_data
        .into_iter()
        .map(|t| crate::tui_unified::state::git_state::Tag {
            name: t.name,
            commit_hash: t.commit_hash,
            message: t.message,
            tagger: None,
            date: chrono::Utc::now(),
            is_annotated: true,
        })
        .collect()
}

/// 将 git interface 的 Remote 转换为 TUI state 的 Remote
fn convert_remotes(
    remotes_data: Vec<crate::tui_unified::git::models::Remote>,
) -> Vec<crate::tui_unified::state::git_state::Remote> {
    remotes_data
        .into_iter()
        .map(|r| crate::tui_unified::state::git_state::Remote {
            name: r.name.clone(),
            url: r.url,
            fetch_url: r.name.clone(),
            push_url: None,
            is_default: r.name == "origin",
        })
        .collect()
}

/// 将 git interface 的 Stash 转换为 TUI state 的 Stash
fn convert_stashes(
    stashes_data: Vec<crate::tui_unified::git::models::Stash>,
) -> Vec<crate::tui_unified::state::git_state::Stash> {
    stashes_data
        .into_iter()
        .map(|s| crate::tui_unified::state::git_state::Stash {
            index: s.index as usize,
            hash: format!("stash@{{{}}}", s.index),
            branch: s.branch,
            message: s.message,
            date: chrono::Utc::now(),
            files_changed: 0,
        })
        .collect()
}

impl super::app::TuiUnifiedApp {
    /// 加载初始Git数据
    ///
    /// 使用 Load → Transform → Update 模式：
    /// 1. 无锁加载所有 git 数据到局部变量
    /// 2. 短暂写锁更新 state
    /// 3. 读锁通知组件
    pub(crate) async fn load_initial_git_data(&mut self) -> Result<()> {
        let repo_path = std::env::current_dir()?;
        let git = crate::tui_unified::git::interface::AsyncGitImpl::new(repo_path.clone());

        // Step 1: 无锁加载所有数据到局部变量
        let current_branch = git.get_current_branch().await.ok();
        let commits = git.get_commits(Some(100)).await.ok().map(convert_commits);
        let branches = git.get_branches().await.ok().map(convert_branches);
        let status = git.get_status().await.ok();
        let tags = git.get_tags().await.ok().map(convert_tags);
        let remotes = git.get_remotes().await.ok().map(convert_remotes);
        let stashes = git.get_stashes().await.ok().map(convert_stashes);

        // Step 2: 短暂写锁更新 state
        {
            let mut state = self.state.write().await;

            if let Some(branch) = current_branch {
                state.repo_state.update_current_branch(branch);
            }
            if let Some(ref commits) = commits {
                state.repo_state.update_commits(commits.clone());
            }
            if let Some(branches) = branches {
                state.repo_state.update_branches(branches);
            }
            if let Some(status_text) = status {
                let is_clean = status_text.trim() == "Working tree clean";
                state
                    .repo_state
                    .update_status(crate::tui_unified::state::git_state::RepoStatus {
                        staged_files: Vec::new(),
                        unstaged_files: Vec::new(),
                        untracked_files: Vec::new(),
                        conflicts: Vec::new(),
                        ahead_count: 0,
                        behind_count: 0,
                        is_clean,
                        is_detached: false,
                    });
            }
            if let Some(tags) = tags {
                state.repo_state.update_tags(tags);
            }
            if let Some(remotes) = remotes {
                state.repo_state.update_remotes(remotes);
            }
            if let Some(stashes) = stashes {
                state.repo_state.update_stashes(stashes);
            }
        }
        // 写锁在此自动释放

        // Step 3: 读锁通知组件
        let state_ref = &*self.state.read().await;
        self.remotes_view.load_remotes(state_ref).await;
        self.stash_view.load_stashes(state_ref).await;
        self.query_history_view.load_history().await;

        // 更新GitLogView的commit数据
        if let Some(commits) = commits {
            let has_commits = !commits.is_empty();
            self.git_log_view.update_commits(commits);
            if has_commits {
                self.git_log_view.set_focus(true);
            }
        }

        Ok(())
    }

    /// 处理pending diff请求
    pub(crate) async fn handle_pending_diff_request(&mut self) -> Result<()> {
        // 获取并清除pending diff请求（只需读锁，Mutex提供内部可变性）
        let commit_hash = {
            let state = self.state.read().await;
            state.get_pending_diff_commit()
        };

        if let Some(hash) = commit_hash {
            // 创建DiffViewer实例
            match DiffViewer::new(&hash).await {
                Ok(diff_viewer) => {
                    // 保存diff_viewer实例
                    self.diff_viewer = Some(diff_viewer);

                    // 显示diff弹窗（传入空的内容，因为DiffViewer自己管理内容）
                    let mut state = self.state.write().await;
                    state.show_diff_modal(hash, String::new());
                    state.add_notification(
                        "Diff viewer created successfully".to_string(),
                        crate::tui_unified::state::app_state::NotificationLevel::Info,
                    );
                }
                Err(e) => {
                    // 显示详细错误通知
                    let mut state = self.state.write().await;
                    state.add_notification(
                        format!(
                            "Failed to create diff viewer for commit {}: {}",
                            &hash[..8.min(hash.len())],
                            e
                        ),
                        crate::tui_unified::state::app_state::NotificationLevel::Error,
                    );
                    // 添加调试建议
                    state.add_notification(
                        "Try checking if the commit exists: git log --oneline".to_string(),
                        crate::tui_unified::state::app_state::NotificationLevel::Warning,
                    );
                }
            }
        }

        Ok(())
    }

    /// 处理 hunk 级暂存请求
    pub(crate) async fn handle_pending_hunk_stage(&mut self) -> Result<()> {
        let hunk_request = {
            let state = self.state.read().await;
            let value = state
                .selected_items
                .pending_hunk_stage
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .take();
            value
        };

        if let Some((_file_path, patch)) = hunk_request {
            // 使用 git apply --cached 暂存单个 hunk
            let mut child = tokio::process::Command::new("git")
                .args(["apply", "--cached", "--allow-empty"])
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .spawn()
                .map_err(|e| anyhow::anyhow!("Failed to spawn git apply: {}", e))?;

            if let Some(mut stdin) = child.stdin.take() {
                use tokio::io::AsyncWriteExt;
                stdin.write_all(patch.as_bytes()).await.map_err(|e| {
                    anyhow::anyhow!("Failed to write patch to stdin: {}", e)
                })?;
                drop(stdin);
            }

            let output = child.wait_with_output().await.map_err(|e| {
                anyhow::anyhow!("Failed to wait for git apply: {}", e)
            })?;

            let mut state = self.state.write().await;
            if output.status.success() {
                state.add_notification(
                    "Hunk staged successfully".to_string(),
                    crate::tui_unified::state::app_state::NotificationLevel::Info,
                );
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                state.add_notification(
                    format!("Failed to stage hunk: {}", stderr.trim()),
                    crate::tui_unified::state::app_state::NotificationLevel::Error,
                );
            }
        }

        Ok(())
    }

    /// 同步获取特定分支的提交历史
    pub(crate) fn get_branch_commits_sync(
        &self,
        branch_name: &str,
    ) -> anyhow::Result<Vec<crate::tui_unified::state::git_state::Commit>> {
        use chrono::{DateTime, Utc};
        use std::process::Command;

        // 执行 git log 命令获取分支的提交历史
        let output = Command::new("git")
            .args([
                "log",
                branch_name,
                "--pretty=format:%H╬%an╬%ae╬%ai╬%s",
                "--max-count=100", // 限制提交数量
            ])
            .output()?;

        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "Failed to get branch commits: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        let mut commits = Vec::new();
        let log_output = String::from_utf8_lossy(&output.stdout);

        for line in log_output.lines() {
            if line.is_empty() {
                continue;
            }

            let parts: Vec<&str> = line.split('╬').collect();
            if parts.len() >= 5 {
                let hash = parts[0].to_string();
                let author = parts[1].to_string();
                let author_email = parts[2].to_string();
                let date_str = parts[3];
                let message = parts[4].to_string();

                // 解析日期
                if let Ok(date) = DateTime::parse_from_str(date_str, "%Y-%m-%d %H:%M:%S %z") {
                    let short_hash = hash[..8.min(hash.len())].to_string();
                    commits.push(crate::tui_unified::state::git_state::Commit {
                        hash: hash.clone(),
                        short_hash,
                        author: author.clone(),
                        author_email: author_email.clone(),
                        committer: author.clone(),
                        committer_email: author_email.clone(),
                        date: date.with_timezone(&Utc),
                        message: message.clone(),
                        subject: message.lines().next().unwrap_or(&message).to_string(),
                        body: if message.lines().count() > 1 {
                            Some(message.lines().skip(1).collect::<Vec<_>>().join("\n"))
                        } else {
                            None
                        },
                        parents: Vec::new(),
                        refs: Vec::new(),
                        files_changed: 0,
                        insertions: 0,
                        deletions: 0,
                    });
                }
            }
        }

        Ok(commits)
    }

    pub(crate) async fn handle_direct_branch_switch_request(&mut self) -> Result<()> {
        // 获取并清除直接分支切换请求（只需读锁，Mutex提供内部可变性）
        let branch_name = {
            let state = self.state.read().await;
            state.get_direct_branch_switch()
        };

        if let Some(branch_name) = branch_name {
            // 直接切换分支
            self.checkout_branch_directly(&branch_name).await?;
        }

        Ok(())
    }

    /// 重新加载 Git 数据（在提交后刷新）
    pub(crate) async fn reload_git_data(&mut self) -> Result<()> {
        self.load_initial_git_data().await
    }

    /// 刷新当前视图的数据
    pub(crate) async fn refresh_current_view(
        &mut self,
        view_type: crate::tui_unified::state::app_state::ViewType,
    ) -> Result<()> {
        match view_type {
            crate::tui_unified::state::app_state::ViewType::GitLog => self.refresh_git_log().await,
            crate::tui_unified::state::app_state::ViewType::Branches => {
                self.refresh_branches().await
            }
            crate::tui_unified::state::app_state::ViewType::Tags => self.refresh_tags().await,
            crate::tui_unified::state::app_state::ViewType::Remotes => self.refresh_remotes().await,
            crate::tui_unified::state::app_state::ViewType::Stash => self.refresh_stash().await,
            crate::tui_unified::state::app_state::ViewType::QueryHistory => {
                self.refresh_query_history().await
            }
            crate::tui_unified::state::app_state::ViewType::Staging => self.refresh_staging().await,
        }
    }

    /// 刷新Git Log视图
    async fn refresh_git_log(&mut self) -> Result<()> {
        let repo_path = std::env::current_dir()?;
        let git = crate::tui_unified::git::interface::AsyncGitImpl::new(repo_path);

        match git.get_commits(Some(100)).await {
            Ok(commits_data) => {
                let commits = convert_commits(commits_data);

                let mut state = self.state.write().await;
                state.repo_state.update_commits(commits.clone());
                drop(state);

                self.git_log_view.update_commits(commits);
                Ok(())
            }
            Err(e) => Err(anyhow::anyhow!("Git operation failed: {}", e).into()),
        }
    }

    /// 刷新Branches视图
    async fn refresh_branches(&mut self) -> Result<()> {
        let repo_path = std::env::current_dir()?;
        let git = crate::tui_unified::git::interface::AsyncGitImpl::new(repo_path);

        match git.get_branches().await {
            Ok(branches_data) => {
                let branches = convert_branches(branches_data);

                let mut state = self.state.write().await;
                state.repo_state.update_branches(branches);
                Ok(())
            }
            Err(e) => Err(anyhow::anyhow!("Git operation failed: {}", e).into()),
        }
    }

    /// 刷新Tags视图
    async fn refresh_tags(&mut self) -> Result<()> {
        let repo_path = std::env::current_dir()?;
        let git = crate::tui_unified::git::interface::AsyncGitImpl::new(repo_path);

        match git.get_tags().await {
            Ok(tags_data) => {
                let tags = convert_tags(tags_data);

                let mut state = self.state.write().await;
                state.repo_state.update_tags(tags);
                Ok(())
            }
            Err(e) => Err(anyhow::anyhow!("Git operation failed: {}", e).into()),
        }
    }

    /// 刷新Remotes视图
    async fn refresh_remotes(&mut self) -> Result<()> {
        let repo_path = std::env::current_dir()?;
        let git = crate::tui_unified::git::interface::AsyncGitImpl::new(repo_path);

        match git.get_remotes().await {
            Ok(remotes_data) => {
                let remotes = convert_remotes(remotes_data);

                let mut state = self.state.write().await;
                state.repo_state.update_remotes(remotes);
                drop(state);

                let state_ref = &*self.state.read().await;
                self.remotes_view.load_remotes(state_ref).await;
                Ok(())
            }
            Err(e) => Err(anyhow::anyhow!("Git operation failed: {}", e).into()),
        }
    }

    /// 刷新Stash视图
    async fn refresh_stash(&mut self) -> Result<()> {
        let repo_path = std::env::current_dir()?;
        let git = crate::tui_unified::git::interface::AsyncGitImpl::new(repo_path);

        match git.get_stashes().await {
            Ok(stashes_data) => {
                let stashes = convert_stashes(stashes_data);

                let mut state = self.state.write().await;
                state.repo_state.update_stashes(stashes);
                drop(state);

                let state_ref = &*self.state.read().await;
                self.stash_view.load_stashes(state_ref).await;
                Ok(())
            }
            Err(e) => Err(anyhow::anyhow!("Git operation failed: {}", e).into()),
        }
    }

    /// 刷新Query History视图
    async fn refresh_query_history(&mut self) -> Result<()> {
        self.query_history_view.load_history().await;
        Ok(())
    }

    /// 刷新Staging视图
    async fn refresh_staging(&mut self) -> Result<()> {
        let state = self.state.read().await;
        self.staging_view.refresh_file_list(&state);
        Ok(())
    }
}
