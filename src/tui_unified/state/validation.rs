use std::collections::HashSet;
use anyhow::{Result, Context, anyhow};
use chrono::{DateTime, Utc, Duration};

use super::{AppState, GitRepoState, SearchState, SelectionState};
use super::ui_state::LayoutState;
use super::persistence::PersistentState;

pub struct StateValidator {
    strict_mode: bool,
    max_age_days: i64,
    max_search_history: usize,
    max_notifications: usize,
}

impl StateValidator {
    pub fn new() -> Self {
        Self {
            strict_mode: false,
            max_age_days: 30,
            max_search_history: 100,
            max_notifications: 50,
        }
    }

    pub fn with_strict_mode(mut self, strict: bool) -> Self {
        self.strict_mode = strict;
        self
    }

    pub fn with_max_age_days(mut self, days: i64) -> Self {
        self.max_age_days = days;
        self
    }

    pub async fn validate_app_state(&self, state: &AppState) -> Result<ValidationResult> {
        let mut result = ValidationResult::new();
        
        // 验证布局状态
        self.validate_layout_state(&state.layout, &mut result);
        
        // 验证Git仓库状态
        self.validate_git_repo_state(&state.repo_state, &mut result);
        
        // 验证选择状态
        self.validate_selection_state(&state.selected_items, &mut result);
        
        // 验证搜索状态
        self.validate_search_state(&state.search_state, &mut result);
        
        // 验证通知状态
        self.validate_notifications(&state.notifications, &mut result);
        
        // 验证加载任务
        self.validate_loading_tasks(&state.loading_tasks, &mut result);
        
        Ok(result)
    }

    pub async fn validate_persistent_state(&self, state: &PersistentState) -> Result<ValidationResult> {
        let mut result = ValidationResult::new();
        
        // 验证版本兼容性
        self.validate_version(&state.version, &mut result);
        
        // 验证时间戳
        self.validate_timestamp(&state.last_saved, &mut result);
        
        // 验证布局偏好
        self.validate_layout_preferences(&state.layout_preferences, &mut result);
        
        // 验证搜索历史
        self.validate_search_history(&state.search_history, &mut result);
        
        // 验证窗口状态
        self.validate_window_state(&state.window_state, &mut result);
        
        // 验证用户偏好
        self.validate_user_preferences(&state.user_preferences, &mut result);
        
        // 验证会话数据
        self.validate_session_data(&state.session_data, &mut result);
        
        Ok(result)
    }

    fn validate_layout_state(&self, layout: &LayoutState, result: &mut ValidationResult) {
        // 验证面板宽度
        if layout.sidebar_width == 0 && layout.content_width == 0 && layout.detail_width == 0 {
            result.add_error("所有面板宽度都为0".to_string());
        }
        
        let total_width = layout.sidebar_width + layout.content_width + layout.detail_width;
        if total_width == 0 {
            result.add_error("面板总宽度为0".to_string());
        }
        
        // 验证比例
        let total_ratio = layout.panel_ratios.sidebar_ratio + 
                         layout.panel_ratios.content_ratio + 
                         layout.panel_ratios.detail_ratio;
        
        if (total_ratio - 1.0).abs() > 0.1 {
            result.add_warning(format!("面板比例总和不等于1.0: {}", total_ratio));
        }
        
        // 验证最小尺寸
        if layout.min_panel_sizes.sidebar_min > layout.sidebar_width {
            result.add_warning("侧边栏宽度小于最小值".to_string());
        }
        
        if layout.min_panel_sizes.content_min > layout.content_width {
            result.add_warning("内容面板宽度小于最小值".to_string());
        }
        
        if layout.min_panel_sizes.detail_min > layout.detail_width {
            result.add_warning("详情面板宽度小于最小值".to_string());
        }
    }

    fn validate_git_repo_state(&self, repo_state: &GitRepoState, result: &mut ValidationResult) {
        // 验证仓库路径
        if !repo_state.repo_path.exists() {
            result.add_error(format!("仓库路径不存在: {:?}", repo_state.repo_path));
        }
        
        // 验证分支名称
        if repo_state.current_branch.is_empty() {
            result.add_warning("当前分支名称为空".to_string());
        }
        
        if repo_state.current_branch.contains(char::is_whitespace) {
            result.add_warning("分支名称包含空白字符".to_string());
        }
        
        // 验证刷新时间
        let now = Utc::now();
        let age = now.signed_duration_since(repo_state.last_refresh);
        if age > Duration::days(1) {
            result.add_info("仓库状态可能需要刷新".to_string());
        }
        
        // 验证提交数据完整性
        for commit in &repo_state.commits {
            if commit.hash.len() < 7 || commit.hash.len() > 40 {
                result.add_warning(format!("无效的提交哈希长度: {}", commit.hash));
            }
            
            if commit.short_hash.len() < 7 || commit.short_hash.len() > 8 {
                result.add_warning(format!("无效的短哈希长度: {}", commit.short_hash));
            }
            
            if commit.author.is_empty() {
                result.add_warning(format!("提交 {} 作者为空", commit.hash));
            }
        }
        
        // 验证分支数据
        let mut branch_names = HashSet::new();
        for branch in &repo_state.branches {
            if !branch_names.insert(&branch.name) {
                result.add_error(format!("重复的分支名称: {}", branch.name));
            }
            
            if branch.name.is_empty() {
                result.add_error("分支名称为空".to_string());
            }
        }
        
        // 验证标签数据
        let mut tag_names = HashSet::new();
        for tag in &repo_state.tags {
            if !tag_names.insert(&tag.name) {
                result.add_error(format!("重复的标签名称: {}", tag.name));
            }
            
            if tag.name.is_empty() {
                result.add_error("标签名称为空".to_string());
            }
            
            if tag.commit_hash.len() < 7 || tag.commit_hash.len() > 40 {
                result.add_warning(format!("标签 {} 的提交哈希无效", tag.name));
            }
        }
        
        // 验证远程仓库数据
        let mut remote_names = HashSet::new();
        for remote in &repo_state.remotes {
            if !remote_names.insert(&remote.name) {
                result.add_error(format!("重复的远程仓库名称: {}", remote.name));
            }
            
            if remote.name.is_empty() {
                result.add_error("远程仓库名称为空".to_string());
            }
            
            if remote.url.is_empty() {
                result.add_error(format!("远程仓库 {} URL为空", remote.name));
            }
        }
    }

    fn validate_selection_state(&self, selection: &SelectionState, result: &mut ValidationResult) {
        // 验证多选状态一致性
        if selection.selection_mode == super::SelectionMode::Single && selection.multi_selection.len() > 1 {
            result.add_error("单选模式下有多个选中项".to_string());
        }
        
        // 验证选中项的有效性
        if let Some(ref commit) = selection.selected_commit {
            if commit.len() < 7 || commit.len() > 40 {
                result.add_warning(format!("选中的提交哈希无效: {}", commit));
            }
        }
        
        if let Some(ref branch) = selection.selected_branch {
            if branch.is_empty() {
                result.add_warning("选中的分支名称为空".to_string());
            }
        }
        
        if let Some(ref tag) = selection.selected_tag {
            if tag.is_empty() {
                result.add_warning("选中的标签名称为空".to_string());
            }
        }
        
        // 验证多选项的唯一性
        let mut unique_items = HashSet::new();
        for item in &selection.multi_selection {
            if !unique_items.insert(item) {
                result.add_warning(format!("多选中有重复项: {}", item));
            }
        }
    }

    fn validate_search_state(&self, search: &SearchState, result: &mut ValidationResult) {
        // 验证搜索历史大小
        if search.history.len() > self.max_search_history {
            result.add_warning(format!("搜索历史过长: {} > {}", search.history.len(), self.max_search_history));
        }
        
        // 验证搜索结果一致性
        if search.current_match > search.results_count && search.results_count > 0 {
            result.add_error(format!("当前匹配索引超出范围: {} > {}", search.current_match, search.results_count));
        }
        
        // 验证活动状态一致性
        if search.is_active && search.query.is_empty() {
            result.add_warning("搜索处于活动状态但查询为空".to_string());
        }
        
        // 验证搜索历史中的重复项
        let mut unique_queries = HashSet::new();
        for query in &search.history {
            if !unique_queries.insert(query) {
                result.add_info(format!("搜索历史中有重复查询: {}", query));
            }
        }
        
        // 验证搜索过滤器
        if let Some(ref date_from) = search.filters.date_from {
            if let Some(ref date_to) = search.filters.date_to {
                if date_from > date_to {
                    result.add_error("搜索开始日期晚于结束日期".to_string());
                }
            }
        }
    }

    fn validate_notifications(&self, notifications: &[super::Notification], result: &mut ValidationResult) {
        if notifications.len() > self.max_notifications {
            result.add_warning(format!("通知数量过多: {} > {}", notifications.len(), self.max_notifications));
        }
        
        // 验证通知ID唯一性
        let mut unique_ids = HashSet::new();
        for notification in notifications {
            if !unique_ids.insert(&notification.id) {
                result.add_error(format!("重复的通知ID: {}", notification.id));
            }
            
            if notification.message.is_empty() {
                result.add_warning("通知消息为空".to_string());
            }
        }
        
        // 检查过期的通知
        let now = Utc::now();
        for notification in notifications {
            let age = now.signed_duration_since(notification.timestamp);
            if age > Duration::hours(24) {
                result.add_info(format!("通知已过期: {}", notification.message));
            }
        }
    }

    fn validate_loading_tasks(&self, tasks: &std::collections::HashMap<String, super::LoadingTask>, result: &mut ValidationResult) {
        // 验证任务ID唯一性
        let mut unique_ids = HashSet::new();
        for task in tasks.values() {
            if !unique_ids.insert(&task.id) {
                result.add_error(format!("重复的任务ID: {}", task.id));
            }
            
            if task.name.is_empty() {
                result.add_warning("任务名称为空".to_string());
            }
            
            if let Some(progress) = task.progress {
                if !(0.0..=1.0).contains(&progress) {
                    result.add_error(format!("任务进度超出范围: {}", progress));
                }
            }
        }
        
        // 检查长时间运行的任务
        let now = Utc::now();
        for task in tasks.values() {
            let age = now.signed_duration_since(task.started_at);
            if age > Duration::minutes(30) {
                result.add_warning(format!("任务运行时间过长: {}", task.name));
            }
        }
    }

    fn validate_version(&self, version: &str, result: &mut ValidationResult) {
        if version.is_empty() {
            result.add_error("版本信息为空".to_string());
            return;
        }
        
        let current_version = env!("CARGO_PKG_VERSION");
        if version != current_version {
            result.add_warning(format!("版本不匹配: 状态版本 {} vs 当前版本 {}", version, current_version));
        }
    }

    fn validate_timestamp(&self, timestamp: &DateTime<Utc>, result: &mut ValidationResult) {
        let now = Utc::now();
        let age = now.signed_duration_since(*timestamp);
        
        if age > Duration::days(self.max_age_days) {
            result.add_warning(format!("状态过旧: {} 天前", age.num_days()));
        }
        
        if *timestamp > now {
            result.add_error("状态时间戳在未来".to_string());
        }
    }

    fn validate_layout_preferences(&self, layout_prefs: &super::persistence::LayoutPreferences, result: &mut ValidationResult) {
        let total_ratio = layout_prefs.sidebar_ratio + layout_prefs.content_ratio + layout_prefs.detail_ratio;
        if (total_ratio - 1.0).abs() > 0.1 {
            result.add_error(format!("布局比例总和不等于1.0: {}", total_ratio));
        }
        
        if layout_prefs.sidebar_ratio < 0.0 || layout_prefs.sidebar_ratio > 1.0 {
            result.add_error(format!("侧边栏比例超出范围: {}", layout_prefs.sidebar_ratio));
        }
        
        if layout_prefs.content_ratio < 0.0 || layout_prefs.content_ratio > 1.0 {
            result.add_error(format!("内容面板比例超出范围: {}", layout_prefs.content_ratio));
        }
        
        if layout_prefs.detail_ratio < 0.0 || layout_prefs.detail_ratio > 1.0 {
            result.add_error(format!("详情面板比例超出范围: {}", layout_prefs.detail_ratio));
        }
    }

    fn validate_search_history(&self, history: &[String], result: &mut ValidationResult) {
        if history.len() > self.max_search_history {
            result.add_warning(format!("搜索历史过长: {} > {}", history.len(), self.max_search_history));
        }
        
        for query in history {
            if query.is_empty() {
                result.add_warning("搜索历史中有空查询".to_string());
            }
            
            if query.len() > 1000 {
                result.add_warning(format!("搜索查询过长: {} 字符", query.len()));
            }
        }
    }

    fn validate_window_state(&self, window_state: &super::persistence::WindowState, result: &mut ValidationResult) {
        let valid_panels = ["Sidebar", "Content", "Detail"];
        if !valid_panels.contains(&window_state.last_focus_panel.as_str()) {
            result.add_error(format!("无效的焦点面板: {}", window_state.last_focus_panel));
        }
        
        let valid_views = ["GitLog", "Branches", "Tags", "Remotes", "Stash", "QueryHistory"];
        if !valid_views.contains(&window_state.last_view.as_str()) {
            result.add_error(format!("无效的视图类型: {}", window_state.last_view));
        }
        
        for panel in &window_state.focus_history {
            if !valid_panels.contains(&panel.as_str()) {
                result.add_warning(format!("焦点历史中有无效面板: {}", panel));
            }
        }
    }

    fn validate_user_preferences(&self, prefs: &super::persistence::UserPreferences, result: &mut ValidationResult) {
        if prefs.auto_refresh_interval == 0 {
            result.add_warning("自动刷新间隔为0".to_string());
        }
        
        if prefs.auto_refresh_interval > 3600 {
            result.add_warning(format!("自动刷新间隔过长: {} 秒", prefs.auto_refresh_interval));
        }
        
        if prefs.max_commits_to_load == 0 {
            result.add_warning("最大提交加载数为0".to_string());
        }
        
        if prefs.max_commits_to_load > 100000 {
            result.add_warning(format!("最大提交加载数过大: {}", prefs.max_commits_to_load));
        }
        
        if prefs.theme_name.is_empty() {
            result.add_warning("主题名称为空".to_string());
        }
    }

    fn validate_session_data(&self, session_data: &super::persistence::SessionData, result: &mut ValidationResult) {
        if !session_data.repo_path.exists() {
            result.add_error(format!("会话仓库路径不存在: {:?}", session_data.repo_path));
        }
        
        if let Some(ref commit) = session_data.last_selected_commit {
            if commit.len() < 7 || commit.len() > 40 {
                result.add_warning(format!("会话中选中的提交哈希无效: {}", commit));
            }
        }
        
        if let Some(ref branch) = session_data.last_selected_branch {
            if branch.is_empty() {
                result.add_warning("会话中选中的分支名称为空".to_string());
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub info: Vec<String>,
    pub is_valid: bool,
}

impl ValidationResult {
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
            warnings: Vec::new(),
            info: Vec::new(),
            is_valid: true,
        }
    }

    pub fn add_error(&mut self, message: String) {
        self.errors.push(message);
        self.is_valid = false;
    }

    pub fn add_warning(&mut self, message: String) {
        self.warnings.push(message);
    }

    pub fn add_info(&mut self, message: String) {
        self.info.push(message);
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }

    pub fn total_issues(&self) -> usize {
        self.errors.len() + self.warnings.len()
    }

    pub fn merge(&mut self, other: ValidationResult) {
        self.errors.extend(other.errors);
        self.warnings.extend(other.warnings);
        self.info.extend(other.info);
        self.is_valid = self.is_valid && other.is_valid;
    }

    pub fn summary(&self) -> String {
        format!(
            "验证结果: {} 错误, {} 警告, {} 信息",
            self.errors.len(),
            self.warnings.len(),
            self.info.len()
        )
    }
}

pub struct StateRecovery {
    validator: StateValidator,
}

impl StateRecovery {
    pub fn new() -> Self {
        Self {
            validator: StateValidator::new(),
        }
    }

    pub async fn recover_app_state(&self, state: &mut AppState) -> Result<ValidationResult> {
        let validation_result = self.validator.validate_app_state(state).await?;
        
        if validation_result.has_errors() {
            self.apply_recovery_strategies(state, &validation_result).await?;
        }
        
        Ok(validation_result)
    }

    async fn apply_recovery_strategies(&self, state: &mut AppState, result: &ValidationResult) -> Result<()> {
        for error in &result.errors {
            self.apply_specific_recovery(state, error).await?;
        }
        Ok(())
    }

    async fn apply_specific_recovery(&self, state: &mut AppState, error: &str) -> Result<()> {
        match error {
            e if e.contains("所有面板宽度都为0") => {
                state.layout = LayoutState::default();
            }
            e if e.contains("仓库路径不存在") => {
                state.repo_state.repo_path = std::env::current_dir().unwrap_or_default();
            }
            e if e.contains("单选模式下有多个选中项") => {
                state.selected_items.multi_selection.clear();
                state.selected_items.selection_mode = super::SelectionMode::Single;
            }
            e if e.contains("当前匹配索引超出范围") => {
                state.search_state.current_match = 0;
            }
            e if e.contains("重复的通知ID") => {
                self.fix_duplicate_notification_ids(state);
            }
            e if e.contains("重复的任务ID") => {
                self.fix_duplicate_task_ids(state);
            }
            _ => {
                // 对于未知错误，记录日志但不进行恢复
                eprintln!("未知的状态错误，无法自动恢复: {}", error);
            }
        }
        Ok(())
    }

    fn fix_duplicate_notification_ids(&self, state: &mut AppState) {
        let mut seen_ids = HashSet::new();
        let mut to_regenerate = Vec::new();
        
        for (index, notification) in state.notifications.iter().enumerate() {
            if !seen_ids.insert(notification.id) {
                to_regenerate.push(index);
            }
        }
        
        for index in to_regenerate {
            state.notifications[index].id = uuid::Uuid::new_v4();
        }
    }

    fn fix_duplicate_task_ids(&self, state: &mut AppState) {
        let mut seen_ids = HashSet::new();
        let mut to_fix = Vec::new();
        
        for (name, task) in state.loading_tasks.iter() {
            if !seen_ids.insert(task.id) {
                to_fix.push(name.clone());
            }
        }
        
        for name in to_fix {
            if let Some(task) = state.loading_tasks.get_mut(&name) {
                task.id = uuid::Uuid::new_v4();
            }
        }
    }
}

impl Default for StateValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for StateRecovery {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui_unified::config::AppConfig;

    #[tokio::test]
    async fn test_layout_state_validation() {
        let validator = StateValidator::new();
        let config = AppConfig::default();
        let mut app_state = AppState::new(&config).await.unwrap();
        
        // 测试有效状态
        let result = validator.validate_app_state(&app_state).await.unwrap();
        assert!(result.is_valid);
        
        // 测试无效状态 - 所有面板宽度为0
        app_state.layout.sidebar_width = 0;
        app_state.layout.content_width = 0;
        app_state.layout.detail_width = 0;
        
        let result = validator.validate_app_state(&app_state).await.unwrap();
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| e.contains("所有面板宽度都为0")));
    }

    #[tokio::test]
    async fn test_state_recovery() {
        let recovery = StateRecovery::new();
        let config = AppConfig::default();
        let mut app_state = AppState::new(&config).await.unwrap();
        
        // 创建无效状态
        app_state.layout.sidebar_width = 0;
        app_state.layout.content_width = 0;
        app_state.layout.detail_width = 0;
        
        // 执行恢复
        let result = recovery.recover_app_state(&mut app_state).await.unwrap();
        
        // 验证恢复后的状态
        assert!(app_state.layout.sidebar_width > 0);
        assert!(app_state.layout.content_width > 0);
        assert!(app_state.layout.detail_width > 0);
    }

    #[tokio::test]
    async fn test_search_state_validation() {
        let validator = StateValidator::new().with_max_age_days(7);
        let config = AppConfig::default();
        let mut app_state = AppState::new(&config).await.unwrap();
        
        // 测试搜索历史过长
        app_state.search_state.history = (0..200).map(|i| format!("query_{}", i)).collect();
        
        let result = validator.validate_app_state(&app_state).await.unwrap();
        assert!(result.warnings.iter().any(|w| w.contains("搜索历史过长")));
        
        // 测试匹配索引超出范围
        app_state.search_state.results_count = 10;
        app_state.search_state.current_match = 15;
        
        let result = validator.validate_app_state(&app_state).await.unwrap();
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| e.contains("当前匹配索引超出范围")));
    }
}