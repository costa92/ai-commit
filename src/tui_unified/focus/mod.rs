pub mod manager;
pub mod ring;

pub use manager::FocusManager;
pub use ring::{FocusRing, NavigationDirection};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusPanel {
    Sidebar, // 侧边栏 (焦点0)
    Content, // 主内容 (焦点1)
    Detail,  // 详情区 (焦点2)
}
