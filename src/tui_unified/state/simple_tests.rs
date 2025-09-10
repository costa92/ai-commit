#[cfg(test)]
mod simple_state_tests {
    use crate::tui_unified::config::AppConfig;
    use crate::tui_unified::state::{AppState, ViewType, LayoutState, SimpleStatePersistence, NotificationLevel, GitRepoState};
    use std::path::PathBuf;

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
    fn test_layout_state_creation() {
        use ratatui::layout::Rect;
        
        let terminal_size = Rect::new(0, 0, 100, 50);
        let layout_state = LayoutState::new(terminal_size);
        
        assert_eq!(layout_state.terminal_size, terminal_size);
        assert!(layout_state.sidebar_width > 0);
        assert!(layout_state.content_width > 0);
        assert!(layout_state.detail_width > 0);
    }

    #[tokio::test]
    async fn test_simple_persistence() {
        use tempfile::TempDir;
        
        let temp_dir = TempDir::new().unwrap();
        let config_dir = temp_dir.path().join("test_config");
        std::fs::create_dir_all(&config_dir).unwrap();
        
        // 创建持久化实例
        let persistence = SimpleStatePersistence::new().unwrap();
        
        // 创建测试状态
        let config = AppConfig::default();
        let app_state = AppState::new(&config).await.unwrap();
        
        // 保存状态
        let save_result = persistence.save_state(&app_state).await;
        assert!(save_result.is_ok());
        
        // 加载状态
        let loaded_state = persistence.load_state().await.unwrap();
        assert!(loaded_state.is_some());
        
        let state = loaded_state.unwrap();
        assert_eq!(state.version, env!("CARGO_PKG_VERSION"));
    }
}