// 配置管理模块

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub cache_size: usize,
    pub theme_name: String,
    // TODO: 添加更多配置项
}

impl AppConfig {
    pub fn load() -> Option<Self> {
        // TODO: 从配置文件加载
        None
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            cache_size: 1000,
            theme_name: "default".to_string(),
        }
    }
}
