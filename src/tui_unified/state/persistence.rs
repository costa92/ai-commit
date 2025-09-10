use std::path::{Path, PathBuf};
use std::fs;
use std::io::{Read, Write};
use serde::{Deserialize, Serialize};
use anyhow::{Result, Context};
use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::{AppState, SearchState, LayoutState};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistentState {
    pub version: String,
    pub layout_preferences: LayoutPreferences,
    pub search_history: Vec<String>,
    pub window_state: WindowState,
    pub user_preferences: UserPreferences,
    pub session_data: SessionData,
    pub last_saved: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutPreferences {
    pub sidebar_ratio: f32,
    pub content_ratio: f32,
    pub detail_ratio: f32,
    pub layout_mode: String, // 序列化友好的字符串形式
    pub panel_sizes: PanelSizesData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanelSizesData {
    pub sidebar_width: u16,
    pub content_width: u16,
    pub detail_width: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowState {
    pub last_focus_panel: String,
    pub focus_history: Vec<String>,
    pub last_view: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferences {
    pub auto_refresh_interval: u64,
    pub max_commits_to_load: usize,
    pub show_file_changes: bool,
    pub enable_syntax_highlighting: bool,
    pub theme_name: String,
    pub key_bindings: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionData {
    pub repo_path: PathBuf,
    pub last_selected_commit: Option<String>,
    pub last_selected_branch: Option<String>,
    pub open_tabs: Vec<String>,
    pub cached_data_keys: Vec<String>,
}

pub struct StatePersistence {
    config_dir: PathBuf,
    state_file: PathBuf,
    backup_dir: PathBuf,
    max_backups: usize,
}

impl StatePersistence {
    pub fn new() -> Result<Self> {
        let config_dir = Self::get_config_dir()?;
        let state_file = config_dir.join("tui_state.json");
        let backup_dir = config_dir.join("backups");
        
        // 确保目录存在
        fs::create_dir_all(&config_dir)?;
        fs::create_dir_all(&backup_dir)?;
        
        Ok(Self {
            config_dir,
            state_file,
            backup_dir,
            max_backups: 10,
        })
    }

    fn get_config_dir() -> Result<PathBuf> {
        let home = dirs::home_dir()
            .context("无法获取用户主目录")?;
        Ok(home.join(".ai-commit").join("tui"))
    }

    pub async fn save_state(&self, app_state: &AppState) -> Result<()> {
        let persistent_state = self.extract_persistent_data(app_state)?;
        
        // 创建备份
        self.create_backup().await?;
        
        // 保存新状态
        let json_data = serde_json::to_string_pretty(&persistent_state)
            .context("状态序列化失败")?;
        
        // 原子写入
        let temp_file = self.state_file.with_extension("tmp");
        fs::write(&temp_file, json_data)
            .context("写入临时状态文件失败")?;
        
        fs::rename(&temp_file, &self.state_file)
            .context("状态文件重命名失败")?;
        
        Ok(())
    }

    pub async fn load_state(&self) -> Result<Option<PersistentState>> {
        if !self.state_file.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&self.state_file)
            .context("读取状态文件失败")?;
        
        match serde_json::from_str::<PersistentState>(&content) {
            Ok(state) => {
                // 验证状态版本兼容性
                if self.is_compatible_version(&state.version) {
                    Ok(Some(state))
                } else {
                    // 版本不兼容，尝试迁移或使用默认值
                    self.handle_version_migration(&state).await
                }
            }
            Err(e) => {
                // 解析失败，尝试从备份恢复
                eprintln!("状态文件解析失败: {}, 尝试从备份恢复", e);
                self.restore_from_backup().await
            }
        }
    }

    pub async fn apply_state(&self, app_state: &mut AppState, persistent_state: &PersistentState) -> Result<()> {
        // 应用布局偏好
        self.apply_layout_preferences(app_state, &persistent_state.layout_preferences);
        
        // 应用搜索历史
        app_state.search_state.history = persistent_state.search_history.clone();
        
        // 应用窗口状态
        self.apply_window_state(app_state, &persistent_state.window_state)?;
        
        // 应用会话数据
        self.apply_session_data(app_state, &persistent_state.session_data);
        
        Ok(())
    }

    fn extract_persistent_data(&self, app_state: &AppState) -> Result<PersistentState> {
        Ok(PersistentState {
            version: env!("CARGO_PKG_VERSION").to_string(),
            layout_preferences: LayoutPreferences {
                sidebar_ratio: app_state.layout.panel_ratios.sidebar_ratio,
                content_ratio: app_state.layout.panel_ratios.content_ratio,
                detail_ratio: app_state.layout.panel_ratios.detail_ratio,
                layout_mode: format!("{:?}", app_state.layout.layout_mode),
                panel_sizes: PanelSizesData {
                    sidebar_width: app_state.layout.sidebar_width,
                    content_width: app_state.layout.content_width,
                    detail_width: app_state.layout.detail_width,
                },
            },
            search_history: app_state.search_state.history.clone(),
            window_state: WindowState {
                last_focus_panel: format!("{:?}", app_state.focus.current_panel),
                focus_history: app_state.focus.panel_history
                    .iter()
                    .map(|p| format!("{:?}", p))
                    .collect(),
                last_view: format!("{:?}", app_state.current_view),
            },
            user_preferences: UserPreferences {
                auto_refresh_interval: 5,
                max_commits_to_load: 1000,
                show_file_changes: true,
                enable_syntax_highlighting: true,
                theme_name: "default".to_string(),
                key_bindings: std::collections::HashMap::new(),
            },
            session_data: SessionData {
                repo_path: app_state.repo_state.repo_path.clone(),
                last_selected_commit: app_state.selected_items.selected_commit.clone(),
                last_selected_branch: app_state.selected_items.selected_branch.clone(),
                open_tabs: Vec::new(),
                cached_data_keys: Vec::new(),
            },
            last_saved: Utc::now(),
        })
    }

    fn apply_layout_preferences(&self, app_state: &mut AppState, layout_prefs: &LayoutPreferences) {
        app_state.layout.panel_ratios.sidebar_ratio = layout_prefs.sidebar_ratio;
        app_state.layout.panel_ratios.content_ratio = layout_prefs.content_ratio;
        app_state.layout.panel_ratios.detail_ratio = layout_prefs.detail_ratio;
        
        app_state.layout.sidebar_width = layout_prefs.panel_sizes.sidebar_width;
        app_state.layout.content_width = layout_prefs.panel_sizes.content_width;
        app_state.layout.detail_width = layout_prefs.panel_sizes.detail_width;
    }

    fn apply_window_state(&self, app_state: &mut AppState, window_state: &WindowState) -> Result<()> {
        // 解析并应用最后的视图
        match window_state.last_view.as_str() {
            "GitLog" => app_state.current_view = super::ViewType::GitLog,
            "Branches" => app_state.current_view = super::ViewType::Branches,
            "Tags" => app_state.current_view = super::ViewType::Tags,
            "Remotes" => app_state.current_view = super::ViewType::Remotes,
            "Stash" => app_state.current_view = super::ViewType::Stash,
            "QueryHistory" => app_state.current_view = super::ViewType::QueryHistory,
            _ => {} // 保持默认值
        }
        
        Ok(())
    }

    fn apply_session_data(&self, app_state: &mut AppState, session_data: &SessionData) {
        // 应用最后选择的项
        app_state.selected_items.selected_commit = session_data.last_selected_commit.clone();
        app_state.selected_items.selected_branch = session_data.last_selected_branch.clone();
    }

    async fn create_backup(&self) -> Result<()> {
        if !self.state_file.exists() {
            return Ok(());
        }

        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let backup_file = self.backup_dir.join(format!("state_backup_{}.json", timestamp));
        
        fs::copy(&self.state_file, &backup_file)
            .context("创建状态备份失败")?;
        
        // 清理旧备份
        self.cleanup_old_backups().await?;
        
        Ok(())
    }

    async fn cleanup_old_backups(&self) -> Result<()> {
        let mut backup_files: Vec<_> = fs::read_dir(&self.backup_dir)?
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                entry.path().extension()
                    .map(|ext| ext == "json")
                    .unwrap_or(false)
            })
            .collect();
        
        if backup_files.len() <= self.max_backups {
            return Ok(());
        }

        // 按修改时间排序
        backup_files.sort_by_key(|entry| {
            entry.metadata()
                .and_then(|m| m.modified())
                .unwrap_or(std::time::UNIX_EPOCH)
        });

        // 删除最旧的备份
        let to_remove = backup_files.len() - self.max_backups;
        for entry in backup_files.iter().take(to_remove) {
            let _ = fs::remove_file(entry.path());
        }

        Ok(())
    }

    async fn restore_from_backup(&self) -> Result<Option<PersistentState>> {
        let backup_files: Vec<_> = fs::read_dir(&self.backup_dir)?
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                entry.path().extension()
                    .map(|ext| ext == "json")
                    .unwrap_or(false)
            })
            .collect();

        // 尝试从最新的备份恢复
        for entry in backup_files.iter().rev() {
            if let Ok(content) = fs::read_to_string(entry.path()) {
                if let Ok(state) = serde_json::from_str::<PersistentState>(&content) {
                    // 恢复成功，复制到主状态文件
                    fs::copy(entry.path(), &self.state_file)?;
                    return Ok(Some(state));
                }
            }
        }

        Ok(None)
    }

    fn is_compatible_version(&self, version: &str) -> bool {
        // 简单的版本兼容性检查
        let current_version = env!("CARGO_PKG_VERSION");
        version == current_version
    }

    async fn handle_version_migration(&self, _old_state: &PersistentState) -> Result<Option<PersistentState>> {
        // 这里可以实现版本迁移逻辑
        // 目前简单返回 None，使用默认状态
        Ok(None)
    }

    pub async fn clear_state(&self) -> Result<()> {
        if self.state_file.exists() {
            fs::remove_file(&self.state_file)
                .context("删除状态文件失败")?;
        }
        Ok(())
    }

    pub async fn get_state_info(&self) -> Result<StateInfo> {
        let mut info = StateInfo {
            config_dir: self.config_dir.clone(),
            state_file_exists: self.state_file.exists(),
            state_file_size: 0,
            last_modified: None,
            backup_count: 0,
            total_backup_size: 0,
        };

        if info.state_file_exists {
            let metadata = fs::metadata(&self.state_file)?;
            info.state_file_size = metadata.len();
            info.last_modified = metadata.modified().ok()
                .map(|t| DateTime::<Utc>::from(t));
        }

        // 统计备份信息
        if self.backup_dir.exists() {
            for entry in fs::read_dir(&self.backup_dir)? {
                if let Ok(entry) = entry {
                    if let Ok(metadata) = entry.metadata() {
                        info.backup_count += 1;
                        info.total_backup_size += metadata.len();
                    }
                }
            }
        }

        Ok(info)
    }
}

#[derive(Debug)]
pub struct StateInfo {
    pub config_dir: PathBuf,
    pub state_file_exists: bool,
    pub state_file_size: u64,
    pub last_modified: Option<DateTime<Utc>>,
    pub backup_count: usize,
    pub total_backup_size: u64,
}

impl Default for PersistentState {
    fn default() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            layout_preferences: LayoutPreferences {
                sidebar_ratio: 0.2,
                content_ratio: 0.5,
                detail_ratio: 0.3,
                layout_mode: "Normal".to_string(),
                panel_sizes: PanelSizesData {
                    sidebar_width: 20,
                    content_width: 50,
                    detail_width: 30,
                },
            },
            search_history: Vec::new(),
            window_state: WindowState {
                last_focus_panel: "Sidebar".to_string(),
                focus_history: Vec::new(),
                last_view: "GitLog".to_string(),
            },
            user_preferences: UserPreferences {
                auto_refresh_interval: 5,
                max_commits_to_load: 1000,
                show_file_changes: true,
                enable_syntax_highlighting: true,
                theme_name: "default".to_string(),
                key_bindings: std::collections::HashMap::new(),
            },
            session_data: SessionData {
                repo_path: PathBuf::from("."),
                last_selected_commit: None,
                last_selected_branch: None,
                open_tabs: Vec::new(),
                cached_data_keys: Vec::new(),
            },
            last_saved: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_state_persistence_basic() {
        let temp_dir = TempDir::new().unwrap();
        let config_dir = temp_dir.path().join("test_config");
        
        let persistence = StatePersistence {
            config_dir: config_dir.clone(),
            state_file: config_dir.join("tui_state.json"),
            backup_dir: config_dir.join("backups"),
            max_backups: 5,
        };

        // 创建测试状态
        let test_state = PersistentState::default();
        
        // 保存状态
        fs::create_dir_all(&config_dir).unwrap();
        fs::create_dir_all(&persistence.backup_dir).unwrap();
        
        let json_data = serde_json::to_string_pretty(&test_state).unwrap();
        fs::write(&persistence.state_file, json_data).unwrap();

        // 加载状态
        let loaded_state = persistence.load_state().await.unwrap();
        assert!(loaded_state.is_some());
        
        let loaded = loaded_state.unwrap();
        assert_eq!(loaded.version, test_state.version);
        assert_eq!(loaded.layout_preferences.sidebar_ratio, test_state.layout_preferences.sidebar_ratio);
    }

    #[tokio::test]
    async fn test_backup_creation_and_cleanup() {
        let temp_dir = TempDir::new().unwrap();
        let config_dir = temp_dir.path().join("test_config");
        
        let persistence = StatePersistence {
            config_dir: config_dir.clone(),
            state_file: config_dir.join("tui_state.json"),
            backup_dir: config_dir.join("backups"),
            max_backups: 3,
        };

        fs::create_dir_all(&persistence.backup_dir).unwrap();
        
        // 创建初始状态文件
        let test_state = PersistentState::default();
        let json_data = serde_json::to_string_pretty(&test_state).unwrap();
        fs::write(&persistence.state_file, json_data).unwrap();

        // 创建多个备份
        for i in 0..5 {
            persistence.create_backup().await.unwrap();
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }

        // 检查备份数量被限制
        let backup_count = fs::read_dir(&persistence.backup_dir).unwrap().count();
        assert!(backup_count <= persistence.max_backups);
    }
}