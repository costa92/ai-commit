// 基础组件trait
use super::events::{EventResult, StateChange};
use crate::tui_unified::state::AppState;
use crossterm::event::KeyEvent;
use ratatui::{layout::Rect, Frame};

/// 基础组件trait，所有TUI组件都应该实现这个trait
pub trait Component {
    /// 组件名称，用于调试和识别
    fn name(&self) -> &str;

    /// 渲染组件到指定区域
    fn render(&mut self, frame: &mut Frame, area: Rect, state: &AppState);

    /// 处理键盘事件
    fn handle_key_event(&mut self, key: KeyEvent, state: &mut AppState) -> EventResult {
        _ = (key, state);
        EventResult::NotHandled
    }

    /// 组件是否获得焦点
    fn is_focused(&self) -> bool {
        false
    }

    /// 设置组件焦点状态
    fn set_focus(&mut self, focused: bool) {
        _ = focused;
    }

    /// 组件是否可以获得焦点
    fn can_focus(&self) -> bool {
        false
    }

    /// 更新组件状态（每帧调用）
    fn update(&mut self, state: &AppState) -> Vec<StateChange> {
        _ = state;
        Vec::new()
    }

    /// 组件是否需要重绘
    fn needs_redraw(&self) -> bool {
        true // 默认总是需要重绘，子组件可以优化这个逻辑
    }

    /// 重置组件状态
    fn reset(&mut self) {}

    /// 组件的最小尺寸要求
    fn min_size(&self) -> (u16, u16) {
        (1, 1) // 默认最小尺寸
    }
}

/// 面板组件trait，用于侧边栏、内容区、详情区等面板
pub trait PanelComponent: Component {
    /// 面板类型标识
    fn panel_type(&self) -> PanelType;

    /// 面板是否支持滚动
    fn supports_scroll(&self) -> bool {
        false
    }

    /// 获取当前滚动位置
    fn scroll_position(&self) -> usize {
        0
    }

    /// 设置滚动位置
    fn set_scroll_position(&mut self, position: usize) {
        _ = position;
    }

    /// 滚动到顶部
    fn scroll_to_top(&mut self) {
        self.set_scroll_position(0);
    }

    /// 滚动到底部
    fn scroll_to_bottom(&mut self) {
        // 默认实现，子组件应该重写
    }
}

/// 视图组件trait，用于不同的Git操作视图
pub trait ViewComponent: Component {
    /// 视图类型标识
    fn view_type(&self) -> ViewType;

    /// 视图标题
    fn title(&self) -> String;

    /// 视图是否支持搜索
    fn supports_search(&self) -> bool {
        false
    }

    /// 执行搜索
    fn search(&mut self, query: &str) -> EventResult {
        _ = query;
        EventResult::NotHandled
    }

    /// 清空搜索
    fn clear_search(&mut self) -> EventResult {
        EventResult::NotHandled
    }

    /// 获取当前选择的项目索引
    fn selected_index(&self) -> Option<usize> {
        None
    }

    /// 设置选择的项目索引
    fn set_selected_index(&mut self, index: Option<usize>) {
        _ = index;
    }

    /// 向上移动选择
    fn select_previous(&mut self) -> EventResult {
        EventResult::NotHandled
    }

    /// 向下移动选择
    fn select_next(&mut self) -> EventResult {
        EventResult::NotHandled
    }

    /// 激活当前选择的项目
    fn activate_selected(&mut self, state: &mut AppState) -> EventResult {
        _ = state;
        EventResult::NotHandled
    }
}

/// 面板类型枚举
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PanelType {
    Sidebar,
    Content,
    Detail,
    StatusBar,
    SearchBox,
    HelpPanel,
}

/// 视图类型枚举
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ViewType {
    GitLog,
    Branches,
    Tags,
    Remotes,
    Stash,
    QueryHistory,
    DiffViewer,
    Staging,
}

/// 组件工厂，用于创建各种组件实例
pub struct ComponentFactory;

impl ComponentFactory {
    /// 创建面板组件 (暂时返回空的实现，等待具体组件实现)
    pub fn create_panel(_panel_type: PanelType) -> Box<dyn PanelComponent> {
        // TODO: 实现具体的组件创建逻辑
        // 暂时返回一个占位符，等待具体组件实现
        unimplemented!("Component creation will be implemented in Task 0.3")
    }

    /// 创建视图组件 (暂时返回空的实现，等待具体组件实现)
    pub fn create_view(_view_type: ViewType) -> Box<dyn ViewComponent> {
        // TODO: 实现具体的组件创建逻辑
        // 暂时返回一个占位符，等待具体组件实现
        unimplemented!("Component creation will be implemented in Task 0.3")
    }
}

/// 组件注册表，用于管理组件生命周期
pub struct ComponentRegistry {
    panels: Vec<Box<dyn PanelComponent>>,
    views: Vec<Box<dyn ViewComponent>>,
}

impl ComponentRegistry {
    /// 创建新的组件注册表
    pub fn new() -> Self {
        Self {
            panels: Vec::new(),
            views: Vec::new(),
        }
    }

    /// 注册面板组件
    pub fn register_panel(&mut self, panel: Box<dyn PanelComponent>) {
        self.panels.push(panel);
    }

    /// 注册视图组件
    pub fn register_view(&mut self, view: Box<dyn ViewComponent>) {
        self.views.push(view);
    }

    /// 获取指定类型的面板组件
    pub fn get_panel_mut(&mut self, panel_type: PanelType) -> Option<&mut Box<dyn PanelComponent>> {
        self.panels
            .iter_mut()
            .find(|panel| panel.panel_type() == panel_type)
    }

    /// 获取指定类型的视图组件
    pub fn get_view_mut(&mut self, view_type: ViewType) -> Option<&mut Box<dyn ViewComponent>> {
        self.views
            .iter_mut()
            .find(|view| view.view_type() == view_type)
    }

    /// 重置所有组件
    pub fn reset_all(&mut self) {
        for panel in &mut self.panels {
            panel.reset();
        }
        for view in &mut self.views {
            view.reset();
        }
    }
}

impl Default for ComponentRegistry {
    fn default() -> Self {
        Self::new()
    }
}
