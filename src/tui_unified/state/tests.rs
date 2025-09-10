#[cfg(test)]
mod state_tests {
    use super::*;
    use crate::tui_unified::config::AppConfig;
    use tempfile::TempDir;
    use chrono::Utc;
    use uuid::Uuid;
    use std::fs;

    // AppState 测试
    mod app_state_tests {
        use super::*;

        #[tokio::test]
        async fn test_app_state_creation() {
            let config = AppConfig::default();
            let app_state = AppState::new(&config).await.unwrap();
            
            assert_eq!(app_state.current_view, ViewType::GitLog);
            assert!(app_state.modal.is_none());
            assert!(app_state.notifications.is_empty());
            assert!(app_state.loading_tasks.is_empty());
            assert!(!app_state.is_loading());
        }

        #[tokio::test]
        async fn test_view_management() {
            let config = AppConfig::default();
            let mut app_state = AppState::new(&config).await.unwrap();
            
            app_state.set_current_view(ViewType::Branches);
            assert_eq!(app_state.get_current_view(), ViewType::Branches);
            
            app_state.set_current_view(ViewType::Tags);
            assert_eq!(app_state.current_view, ViewType::Tags);
        }

        #[tokio::test]
        async fn test_selection_management() {
            let config = AppConfig::default();
            let mut app_state = AppState::new(&config).await.unwrap();
            
            // 测试提交选择
            app_state.select_commit("abc123".to_string());
            assert_eq!(app_state.selected_items.selected_commit, Some("abc123".to_string()));
            
            // 测试分支选择
            app_state.select_branch("main".to_string());
            assert_eq!(app_state.selected_items.selected_branch, Some("main".to_string()));
            
            // 测试获取当前选择
            app_state.set_current_view(ViewType::GitLog);
            assert_eq!(app_state.get_current_selection(), Some("abc123".to_string()));
            
            app_state.set_current_view(ViewType::Branches);
            assert_eq!(app_state.get_current_selection(), Some("main".to_string()));
            
            // 测试清除选择
            app_state.clear_selection();
            assert!(app_state.get_current_selection().is_none());
        }

        #[tokio::test]
        async fn test_search_management() {
            let config = AppConfig::default();
            let mut app_state = AppState::new(&config).await.unwrap();
            
            // 测试搜索查询设置
            app_state.set_search_query("test query".to_string());
            assert_eq!(app_state.search_state.query, "test query");
            assert!(app_state.search_state.is_active);
            
            // 测试搜索历史
            app_state.add_search_to_history("query 1".to_string());
            app_state.add_search_to_history("query 2".to_string());
            assert_eq!(app_state.search_state.history.len(), 2);
            assert_eq!(app_state.search_state.history[0], "query 2");
            
            // 测试重复查询不会添加到历史
            app_state.add_search_to_history("query 1".to_string());
            assert_eq!(app_state.search_state.history.len(), 2);
            
            // 测试清除搜索
            app_state.clear_search();
            assert!(app_state.search_state.query.is_empty());
            assert!(!app_state.search_state.is_active);
            assert_eq!(app_state.search_state.results_count, 0);
        }

        #[tokio::test]
        async fn test_loading_task_management() {
            let config = AppConfig::default();
            let mut app_state = AppState::new(&config).await.unwrap();
            
            // 测试添加加载任务
            let task_id = app_state.add_loading_task("test_task".to_string(), "Loading...".to_string());
            assert!(app_state.is_loading());
            assert_eq!(app_state.loading_tasks.len(), 1);
            
            // 测试更新任务进度
            app_state.update_loading_progress("test_task", 0.5, "50% complete".to_string());
            let task = app_state.loading_tasks.get("test_task").unwrap();
            assert_eq!(task.progress, Some(0.5));
            assert_eq!(task.message, "50% complete");
            
            // 测试移除任务
            app_state.remove_loading_task("test_task");
            assert!(!app_state.is_loading());
            assert!(app_state.loading_tasks.is_empty());
        }

        #[tokio::test]
        async fn test_notification_management() {
            let config = AppConfig::default();
            let mut app_state = AppState::new(&config).await.unwrap();
            
            // 测试添加通知
            let id1 = app_state.add_notification("Info message".to_string(), NotificationLevel::Info);
            let id2 = app_state.add_notification("Error message".to_string(), NotificationLevel::Error);
            
            assert_eq!(app_state.notifications.len(), 2);
            
            let active_notifications = app_state.get_active_notifications();
            assert_eq!(active_notifications.len(), 2);
            
            // 测试消除通知
            app_state.dismiss_notification(id1);
            let active_notifications = app_state.get_active_notifications();
            assert_eq!(active_notifications.len(), 1);
            
            // 测试清理已消除的通知
            app_state.clean_dismissed_notifications();
            assert_eq!(app_state.notifications.len(), 1);
            assert_eq!(app_state.notifications[0].id, id2);
        }

        #[tokio::test]
        async fn test_modal_management() {
            let config = AppConfig::default();
            let mut app_state = AppState::new(&config).await.unwrap();
            
            assert!(!app_state.is_modal_active());
            
            let modal = ModalState {
                modal_type: ModalType::Info,
                title: "Test".to_string(),
                content: "Test content".to_string(),
                buttons: vec![],
                default_button: 0,
                can_cancel: true,
            };
            
            app_state.show_modal(modal);
            assert!(app_state.is_modal_active());
            
            app_state.hide_modal();
            assert!(!app_state.is_modal_active());
        }

        #[tokio::test]
        async fn test_focus_management() {
            let config = AppConfig::default();
            let mut app_state = AppState::new(&config).await.unwrap();
            
            use crate::tui_unified::focus::FocusPanel;
            
            // 测试初始焦点
            assert_eq!(*app_state.get_current_focus(), FocusPanel::Sidebar);
            
            // 测试设置焦点
            app_state.set_focus(FocusPanel::Content);
            assert_eq!(*app_state.get_current_focus(), FocusPanel::Content);
            assert_eq!(app_state.focus.panel_history.len(), 1);
            assert_eq!(app_state.focus.panel_history[0], FocusPanel::Sidebar);
            
            // 测试返回上一个焦点
            app_state.focus_previous();
            assert_eq!(*app_state.get_current_focus(), FocusPanel::Sidebar);
            assert!(app_state.focus.panel_history.is_empty());
        }
    }

    // GitRepoState 测试
    mod git_repo_state_tests {
        use super::*;
        use std::path::PathBuf;

        #[test]
        fn test_git_repo_state_creation() {
            let repo_path = PathBuf::from("/test/repo");
            let repo_state = GitRepoState::new(repo_path.clone());
            
            assert_eq!(repo_state.repo_path, repo_path);
            assert_eq!(repo_state.repo_name, "repo");
            assert!(repo_state.current_branch.is_empty());
            assert!(repo_state.commits.is_empty());
            assert!(repo_state.branches.is_empty());
        }

        #[test]
        fn test_git_repo_state_updates() {
            let mut repo_state = GitRepoState::default();
            
            // 测试更新分支
            repo_state.update_current_branch("main".to_string());
            assert_eq!(repo_state.current_branch, "main");
            
            // 测试更新状态
            let mut status = RepoStatus::default();
            status.is_clean = false;
            repo_state.update_status(status);
            assert!(!repo_state.status.is_clean);
            
            // 测试脏状态检查
            assert!(repo_state.is_dirty());
            assert!(!repo_state.has_conflicts());
        }

        #[test]
        fn test_git_data_queries() {
            let mut repo_state = GitRepoState::default();
            
            // 添加测试数据
            let commit = Commit {
                hash: "abc123def456".to_string(),
                short_hash: "abc123d".to_string(),
                author: "Test Author".to_string(),
                author_email: "test@example.com".to_string(),
                committer: "Test Author".to_string(),
                committer_email: "test@example.com".to_string(),
                date: Utc::now(),
                message: "Test commit".to_string(),
                subject: "Test commit".to_string(),
                body: None,
                parents: vec![],
                refs: vec![],
                files_changed: 1,
                insertions: 10,
                deletions: 5,
            };
            
            repo_state.update_commits(vec![commit]);
            
            // 测试查询提交
            let found_commit = repo_state.get_commit_by_hash("abc123");
            assert!(found_commit.is_some());
            assert_eq!(found_commit.unwrap().hash, "abc123def456");
            
            let found_by_short = repo_state.get_commit_by_hash("abc123d");
            assert!(found_by_short.is_some());
            
            let not_found = repo_state.get_commit_by_hash("xyz789");
            assert!(not_found.is_none());
        }

        #[test]
        fn test_repo_summary() {
            let mut repo_state = GitRepoState::default();
            repo_state.repo_name = "test-repo".to_string();
            repo_state.current_branch = "main".to_string();
            
            // 添加一些测试数据
            repo_state.commits = vec![Commit {
                hash: "abc123".to_string(),
                short_hash: "abc123".to_string(),
                author: "Author".to_string(),
                author_email: "author@example.com".to_string(),
                committer: "Author".to_string(),
                committer_email: "author@example.com".to_string(),
                date: Utc::now(),
                message: "Test".to_string(),
                subject: "Test".to_string(),
                body: None,
                parents: vec![],
                refs: vec![],
                files_changed: 0,
                insertions: 0,
                deletions: 0,
            }];
            
            let summary = repo_state.get_repo_summary();
            assert_eq!(summary.name, "test-repo");
            assert_eq!(summary.current_branch, "main");
            assert_eq!(summary.total_commits, 1);
            assert!(!summary.is_dirty);
            assert!(!summary.has_conflicts);
        }
    }

    // LayoutState 测试
    mod layout_state_tests {
        use super::*;
        use ratatui::layout::Rect;

        #[test]
        fn test_layout_state_creation() {
            let terminal_size = Rect::new(0, 0, 100, 50);
            let layout_state = LayoutState::new(terminal_size);
            
            assert_eq!(layout_state.terminal_size, terminal_size);
            assert!(layout_state.sidebar_width > 0);
            assert!(layout_state.content_width > 0);
            assert!(layout_state.detail_width > 0);
        }

        #[test]
        fn test_layout_calculation() {
            let mut layout_state = LayoutState::default();
            layout_state.terminal_size = Rect::new(0, 0, 100, 50);
            
            layout_state.calculate_panel_sizes();
            
            let total = layout_state.sidebar_width + layout_state.content_width + layout_state.detail_width;
            assert!(total <= 98); // 100 - 2 for borders
        }

        #[test]
        fn test_layout_mode_changes() {
            let mut layout_state = LayoutState::default();
            layout_state.terminal_size = Rect::new(0, 0, 100, 50);
            
            // 测试全屏模式
            use crate::tui_unified::focus::FocusPanel;
            layout_state.set_layout_mode(LayoutMode::FullScreen(FocusPanel::Content));
            
            assert_eq!(layout_state.layout_mode, LayoutMode::FullScreen(FocusPanel::Content));
            assert_eq!(layout_state.sidebar_width, 0);
            assert!(layout_state.content_width > 0);
            assert_eq!(layout_state.detail_width, 0);
        }

        #[test]
        fn test_panel_ratio_adjustment() {
            let mut layout_state = LayoutState::default();
            layout_state.terminal_size = Rect::new(0, 0, 100, 50);
            
            let original_sidebar_ratio = layout_state.panel_ratios.sidebar_ratio;
            
            layout_state.adjust_panel_ratios(0.1, -0.05);
            
            assert!(layout_state.panel_ratios.sidebar_ratio > original_sidebar_ratio);
            
            // 验证比例总和仍然合理
            let total_ratio = layout_state.panel_ratios.sidebar_ratio + 
                             layout_state.panel_ratios.content_ratio + 
                             layout_state.panel_ratios.detail_ratio;
            assert!((total_ratio - 1.0).abs() < 0.1);
        }

        #[test]
        fn test_panel_fit_check() {
            let mut layout_state = LayoutState::default();
            
            // 测试正常尺寸
            layout_state.terminal_size = Rect::new(0, 0, 100, 50);
            assert!(layout_state.can_fit_panels());
            
            // 测试过小尺寸
            layout_state.terminal_size = Rect::new(0, 0, 20, 10);
            assert!(!layout_state.can_fit_panels());
        }
    }

    // FocusRing 测试
    mod focus_ring_tests {
        use super::*;
        use crate::tui_unified::focus::FocusPanel;

        #[test]
        fn test_focus_ring_navigation() {
            let mut focus_ring = FocusRing::default();
            
            // 测试初始状态
            assert_eq!(focus_ring.current(), FocusPanel::Sidebar);
            
            // 测试下一个焦点
            let next = focus_ring.next();
            assert_eq!(next, FocusPanel::Content);
            assert_eq!(focus_ring.current(), FocusPanel::Content);
            
            // 继续下一个
            let next = focus_ring.next();
            assert_eq!(next, FocusPanel::Detail);
            
            // 测试环绕
            let next = focus_ring.next();
            assert_eq!(next, FocusPanel::Sidebar);
        }

        #[test]
        fn test_focus_ring_previous() {
            let mut focus_ring = FocusRing::default();
            
            // 从第一个位置向前
            let prev = focus_ring.previous();
            assert_eq!(prev, FocusPanel::Detail); // 应该环绕到最后一个
            
            let prev = focus_ring.previous();
            assert_eq!(prev, FocusPanel::Content);
            
            let prev = focus_ring.previous();
            assert_eq!(prev, FocusPanel::Sidebar);
        }

        #[test]
        fn test_focus_ring_set_current() {
            let mut focus_ring = FocusRing::default();
            
            focus_ring.set_current(FocusPanel::Detail);
            assert_eq!(focus_ring.current(), FocusPanel::Detail);
            assert_eq!(focus_ring.current_index, 2);
        }

        #[test]
        fn test_focus_ring_no_wrap() {
            let mut focus_ring = FocusRing::default();
            focus_ring.wrap_around = false;
            
            // 测试到达边界时不环绕
            let next = focus_ring.next();
            assert_eq!(next, FocusPanel::Content);
            
            let next = focus_ring.next();
            assert_eq!(next, FocusPanel::Detail);
            
            // 应该停在最后一个
            let next = focus_ring.next();
            assert_eq!(next, FocusPanel::Detail);
        }
    }

    // Modal 测试
    mod modal_tests {
        use super::*;

        #[test]
        fn test_modal_creation() {
            let info_modal = UIModalState::new_info(
                "信息".to_string(),
                "这是一个信息对话框".to_string()
            );
            
            assert_eq!(info_modal.modal_type, ModalType::Info);
            assert_eq!(info_modal.title, "信息");
            assert_eq!(info_modal.buttons.len(), 1);
            assert_eq!(info_modal.buttons[0].action, ModalAction::Ok);
        }

        #[test]
        fn test_confirm_modal() {
            let confirm_modal = UIModalState::new_confirm(
                "确认".to_string(),
                "确定要继续吗？".to_string()
            );
            
            assert_eq!(confirm_modal.modal_type, ModalType::Confirm);
            assert_eq!(confirm_modal.buttons.len(), 2);
            
            let yes_button = confirm_modal.buttons.iter().find(|b| b.action == ModalAction::Yes);
            let no_button = confirm_modal.buttons.iter().find(|b| b.action == ModalAction::No);
            
            assert!(yes_button.is_some());
            assert!(no_button.is_some());
            assert!(yes_button.unwrap().is_default);
            assert!(!no_button.unwrap().is_default);
        }

        #[test]
        fn test_error_modal() {
            let error_modal = UIModalState::new_error(
                "错误".to_string(),
                "发生了一个错误".to_string()
            );
            
            assert_eq!(error_modal.modal_type, ModalType::Error);
            assert_eq!(error_modal.title, "错误");
            assert_eq!(error_modal.buttons.len(), 1);
        }
    }

    // 集成测试
    mod integration_tests {
        use super::*;

        #[tokio::test]
        async fn test_complete_state_workflow() {
            let config = AppConfig::default();
            let mut app_state = AppState::new(&config).await.unwrap();
            
            // 模拟完整的用户工作流程
            
            // 1. 切换到分支视图
            app_state.set_current_view(ViewType::Branches);
            
            // 2. 选择一个分支
            app_state.select_branch("feature/new-feature".to_string());
            
            // 3. 执行搜索
            app_state.set_search_query("bug fix".to_string());
            app_state.add_search_to_history("bug fix".to_string());
            
            // 4. 添加一个加载任务
            let task_id = app_state.add_loading_task("loading_commits".to_string(), "加载提交历史...".to_string());
            
            // 5. 更新进度
            app_state.update_loading_progress("loading_commits", 0.5, "50% 完成".to_string());
            
            // 6. 添加通知
            let notification_id = app_state.add_notification("分支切换成功".to_string(), NotificationLevel::Success);
            
            // 验证状态
            assert_eq!(app_state.current_view, ViewType::Branches);
            assert_eq!(app_state.get_current_selection(), Some("feature/new-feature".to_string()));
            assert_eq!(app_state.search_state.query, "bug fix");
            assert!(app_state.is_loading());
            assert_eq!(app_state.get_active_notifications().len(), 1);
            
            // 7. 完成任务
            app_state.remove_loading_task("loading_commits");
            
            // 8. 清理
            app_state.dismiss_notification(notification_id);
            app_state.clean_dismissed_notifications();
            app_state.clear_search();
            
            // 验证清理后的状态
            assert!(!app_state.is_loading());
            assert!(app_state.get_active_notifications().is_empty());
            assert!(!app_state.search_state.is_active);
        }

        #[tokio::test]
        async fn test_state_consistency() {
            let config = AppConfig::default();
            let mut app_state = AppState::new(&config).await.unwrap();
            
            // 测试状态一致性
            app_state.selected_items.selection_mode = SelectionMode::Single;
            app_state.selected_items.multi_selection = vec!["item1".to_string(), "item2".to_string()];
            
            // 这种状态是不一致的，应该被验证器捕获
            let validator = StateValidator::new();
            let result = validator.validate_app_state(&app_state).await.unwrap();
            
            assert!(!result.is_valid);
            assert!(result.errors.iter().any(|e| e.contains("单选模式下有多个选中项")));
        }
    }
}