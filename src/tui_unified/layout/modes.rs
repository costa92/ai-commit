#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutMode {
    Normal,           // 标准三栏布局
    SplitHorizontal,  // 水平分屏
    SplitVertical,    // 垂直分屏  
    FullScreen,       // 全屏diff模式
}

#[derive(Debug, Clone, PartialEq)]
pub enum PanelType {
    Sidebar,
    Content,
    Detail,
}