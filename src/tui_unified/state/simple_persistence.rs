use std::path::PathBuf;
use std::fs;
use serde::{Deserialize, Serialize};
use anyhow::Result;
use chrono::{DateTime, Utc};

use super::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimplePersistentState {
    pub version: String,
    pub sidebar_width: u16,
    pub content_width: u16,
    pub detail_width: u16,
    pub search_history: Vec<String>,
    pub last_view: String,
    pub last_saved: DateTime<Utc>,
}

pub struct SimpleStatePersistence {
    state_file: PathBuf,
}

impl SimpleStatePersistence {
    pub fn new() -> Result<Self> {
        let config_dir = dirs::home_dir()
            .unwrap_or_default()
            .join(".ai-commit")
            .join("tui");
        
        fs::create_dir_all(&config_dir)?;
        let state_file = config_dir.join("simple_state.json");
        
        Ok(Self { state_file })
    }

    pub async fn save_state(&self, app_state: &AppState) -> Result<()> {
        let simple_state = SimplePersistentState {
            version: env!("CARGO_PKG_VERSION").to_string(),
            sidebar_width: app_state.layout.sidebar_width,
            content_width: app_state.layout.content_width,
            detail_width: app_state.layout.detail_width,
            search_history: app_state.search_state.history.clone(),
            last_view: format!("{:?}", app_state.current_view),
            last_saved: Utc::now(),
        };
        
        let json_data = serde_json::to_string_pretty(&simple_state)?;
        fs::write(&self.state_file, json_data)?;
        
        Ok(())
    }

    pub async fn load_state(&self) -> Result<Option<SimplePersistentState>> {
        if !self.state_file.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&self.state_file)?;
        let state: SimplePersistentState = serde_json::from_str(&content)?;
        
        Ok(Some(state))
    }

    pub async fn apply_state(&self, app_state: &mut AppState, persistent_state: &SimplePersistentState) -> Result<()> {
        // 应用布局设置
        app_state.layout.sidebar_width = persistent_state.sidebar_width;
        app_state.layout.content_width = persistent_state.content_width;
        app_state.layout.detail_width = persistent_state.detail_width;
        
        // 应用搜索历史
        app_state.search_state.history = persistent_state.search_history.clone();
        
        // 应用视图设置
        match persistent_state.last_view.as_str() {
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
}

impl Default for SimplePersistentState {
    fn default() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            sidebar_width: 20,
            content_width: 50,
            detail_width: 30,
            search_history: Vec::new(),
            last_view: "GitLog".to_string(),
            last_saved: Utc::now(),
        }
    }
}