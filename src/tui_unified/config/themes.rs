use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub name: String,
    pub colors: ColorScheme,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorScheme {
    pub background: String,
    pub foreground: String,
    pub primary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyleScheme {
    // TODO: 实现样式方案
}

impl Default for ColorScheme {
    fn default() -> Self {
        Self {
            background: "black".to_string(),
            foreground: "white".to_string(),
            primary: "blue".to_string(),
        }
    }
}
