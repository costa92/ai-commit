use crate::diff_viewer::DiffViewer;
use crate::tui_unified::components::base::component::Component;
use crate::tui_unified::git::interface::GitRepositoryAPI;
use crate::tui_unified::Result;

impl super::app::TuiUnifiedApp {
    /// 加载初始Git数据
    pub(crate) async fn load_initial_git_data(&mut self) -> Result<()> {
        // 获取当前目录作为Git仓库路径
        let repo_path = std::env::current_dir()?;

        // 创建AsyncGitImpl实例
        let git = crate::tui_unified::git::interface::AsyncGitImpl::new(repo_path.clone());

        // 获取写锁访问状态
        let mut state = self.state.write().await;

        // 加载基础Git数据
        match git.get_current_branch().await {
            Ok(branch) => {
                state.repo_state.update_current_branch(branch);
            }
            Err(e) => {
                // 如果获取分支失败，可能不是Git仓库，记录但继续
                eprintln!("Warning: Failed to get current branch: {}", e);
            }
        }

        // 加载提交历史
        match git.get_commits(Some(100)).await {
            Ok(commits_data) => {
                // 转换为内部数据结构
                let commits: Vec<crate::tui_unified::state::git_state::Commit> = commits_data
                    .into_iter()
                    .map(|c| crate::tui_unified::state::git_state::Commit {
                        hash: c.hash.clone(),
                        short_hash: c.hash[..8.min(c.hash.len())].to_string(),
                        author: c.author.clone(),
                        author_email: format!("{}@example.com", c.author), // Git interface doesn't provide email yet
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
                    .collect();

                state.repo_state.update_commits(commits);
            }
            Err(e) => {
                eprintln!("Warning: Failed to load commits: {}", e);
            }
        }

        // 加载分支信息
        match git.get_branches().await {
            Ok(branches_data) => {
                let branches: Vec<crate::tui_unified::state::git_state::Branch> = branches_data
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
                    .collect();

                state.repo_state.update_branches(branches);
            }
            Err(e) => {
                eprintln!("Warning: Failed to load branches: {}", e);
            }
        }

        // 加载仓库状态
        match git.get_status().await {
            Ok(status_text) => {
                // 简单的状态解析 - 如果状态文本包含文件变更信息则认为不干净
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
            Err(e) => {
                eprintln!("Warning: Failed to get repository status: {}", e);
            }
        }

        // 加载标签信息
        match git.get_tags().await {
            Ok(tags_data) => {
                let tags: Vec<crate::tui_unified::state::git_state::Tag> = tags_data
                    .into_iter()
                    .map(|t| crate::tui_unified::state::git_state::Tag {
                        name: t.name,
                        commit_hash: t.commit_hash,
                        message: t.message,
                        tagger: None,
                        date: chrono::Utc::now(), // TODO: Parse actual date from Git
                        is_annotated: true,       // TODO: Detect if annotated
                    })
                    .collect();

                state.repo_state.update_tags(tags);
            }
            Err(e) => {
                eprintln!("Warning: Failed to load tags: {}", e);
            }
        }

        // 加载远程仓库信息
        match git.get_remotes().await {
            Ok(remotes_data) => {
                let remotes: Vec<crate::tui_unified::state::git_state::Remote> = remotes_data
                    .into_iter()
                    .map(|r| crate::tui_unified::state::git_state::Remote {
                        name: r.name.clone(),
                        url: r.url,
                        fetch_url: r.name.clone(), // TODO: Get actual fetch URL
                        push_url: None,
                        is_default: r.name == "origin",
                    })
                    .collect();

                state.repo_state.update_remotes(remotes);
            }
            Err(e) => {
                eprintln!("Warning: Failed to load remotes: {}", e);
            }
        }

        // 加载储藏信息
        match git.get_stashes().await {
            Ok(stashes_data) => {
                let stashes: Vec<crate::tui_unified::state::git_state::Stash> = stashes_data
                    .into_iter()
                    .map(|s| crate::tui_unified::state::git_state::Stash {
                        index: s.index as usize,
                        hash: format!("stash@{{{}}}", s.index), // Use stash reference as hash
                        branch: s.branch,
                        message: s.message,
                        date: chrono::Utc::now(), // TODO: Parse actual date from Git
                        files_changed: 0,         // TODO: Get actual file count
                    })
                    .collect();

                state.repo_state.update_stashes(stashes);
            }
            Err(e) => {
                eprintln!("Warning: Failed to load stashes: {}", e);
            }
        }

        // 释放状态锁
        drop(state);

        // 加载各组件的数据
        let state_ref = &*self.state.read().await;
        self.remotes_view.load_remotes(state_ref).await;
        self.stash_view.load_stashes(state_ref).await;
        self.query_history_view.load_history().await;

        // 更新GitLogView的commit数据
        let commits = state_ref.repo_state.commits.clone();
        let has_commits = !commits.is_empty();
        self.git_log_view.update_commits(commits);

        // 确保GitLogView获得焦点（因为它是默认视图）
        if has_commits {
            self.git_log_view.set_focus(true);
        }

        Ok(())
    }

    /// 处理pending diff请求
    pub(crate) async fn handle_pending_diff_request(&mut self) -> Result<()> {
        // 获取并清除pending diff请求
        let commit_hash = {
            let mut state = self.state.write().await;
            state.get_pending_diff_commit()
        };

        if let Some(hash) = commit_hash {
            // 获取当前目录作为Git仓库路径
            let repo_path = std::env::current_dir()?;
            let _git = crate::tui_unified::git::interface::AsyncGitImpl::new(repo_path);

            // 添加调试信息
            {
                let mut state = self.state.write().await;
                state.add_notification(
                    format!(
                        "Creating diff viewer for commit: {}",
                        &hash[..8.min(hash.len())]
                    ),
                    crate::tui_unified::state::app_state::NotificationLevel::Info,
                );
            }

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
                        committer: author.clone(), // 简化处理，使用author作为committer
                        committer_email: author_email.clone(),
                        date: date.with_timezone(&Utc),
                        message: message.clone(),
                        subject: message.lines().next().unwrap_or(&message).to_string(),
                        body: if message.lines().count() > 1 {
                            Some(message.lines().skip(1).collect::<Vec<_>>().join("\n"))
                        } else {
                            None
                        },
                        parents: Vec::new(), // 简化处理
                        refs: Vec::new(),    // 简化处理
                        files_changed: 0,    // 简化处理
                        insertions: 0,       // 简化处理
                        deletions: 0,        // 简化处理
                    });
                }
            }
        }

        Ok(commits)
    }

    pub(crate) async fn handle_direct_branch_switch_request(&mut self) -> Result<()> {
        // 获取并清除直接分支切换请求
        let branch_name = {
            let mut state = self.state.write().await;
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
        // 直接调用现有的加载逻辑
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
        }
    }

    /// 刷新Git Log视图
    async fn refresh_git_log(&mut self) -> Result<()> {
        let repo_path = std::env::current_dir()?;
        let git = crate::tui_unified::git::interface::AsyncGitImpl::new(repo_path);

        match git.get_commits(Some(100)).await {
            Ok(commits_data) => {
                // 转换为内部数据结构
                let commits: Vec<crate::tui_unified::state::git_state::Commit> = commits_data
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
                    .collect();

                // 更新状态
                let mut state = self.state.write().await;
                state.repo_state.update_commits(commits.clone());
                drop(state);

                // 更新GitLogView
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
                let branches: Vec<crate::tui_unified::state::git_state::Branch> = branches_data
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
                    .collect();

                // 更新状态
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
                let tags: Vec<crate::tui_unified::state::git_state::Tag> = tags_data
                    .into_iter()
                    .map(|t| crate::tui_unified::state::git_state::Tag {
                        name: t.name,
                        commit_hash: t.commit_hash,
                        message: t.message,
                        tagger: None,
                        date: chrono::Utc::now(),
                        is_annotated: true,
                    })
                    .collect();

                // 更新状态
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
                let remotes: Vec<crate::tui_unified::state::git_state::Remote> = remotes_data
                    .into_iter()
                    .map(|r| crate::tui_unified::state::git_state::Remote {
                        name: r.name.clone(),
                        url: r.url,
                        fetch_url: r.name.clone(),
                        push_url: None,
                        is_default: r.name == "origin",
                    })
                    .collect();

                // 更新状态并通知视图
                let mut state = self.state.write().await;
                state.repo_state.update_remotes(remotes);
                drop(state);

                // 通知RemotesView重新加载数据
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
                let stashes: Vec<crate::tui_unified::state::git_state::Stash> = stashes_data
                    .into_iter()
                    .map(|s| crate::tui_unified::state::git_state::Stash {
                        index: s.index as usize,
                        hash: format!("stash@{{{}}}", s.index),
                        branch: s.branch,
                        message: s.message,
                        date: chrono::Utc::now(),
                        files_changed: 0,
                    })
                    .collect();

                // 更新状态并通知视图
                let mut state = self.state.write().await;
                state.repo_state.update_stashes(stashes);
                drop(state);

                // 通知StashView重新加载数据
                let state_ref = &*self.state.read().await;
                self.stash_view.load_stashes(state_ref).await;
                Ok(())
            }
            Err(e) => Err(anyhow::anyhow!("Git operation failed: {}", e).into()),
        }
    }

    /// 刷新Query History视图
    async fn refresh_query_history(&mut self) -> Result<()> {
        // 重新加载查询历史
        self.query_history_view.load_history().await;
        Ok(())
    }
}
