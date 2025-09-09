// 事件处理占位符
#[derive(Debug, Clone)]
pub enum EventResult {
    Handled,
    NotHandled,
}

#[derive(Debug, Clone)]
pub enum Navigation {
    NextPanel,
    PrevPanel,
}

#[derive(Debug, Clone)]
pub enum StateChange {
    // TODO: 实现状态变化类型
}

#[derive(Debug, Clone)]
pub enum AsyncTask {
    // TODO: 实现异步任务类型
}

#[derive(Debug, Clone)]
pub enum CustomEvent {
    // TODO: 实现自定义事件类型
}