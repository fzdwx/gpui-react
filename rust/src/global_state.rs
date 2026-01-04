use std::sync::atomic::{AtomicBool, Ordering};

/// 全局状态结构体 - 只包含真正全局的状态
pub struct GlobalState {
    /// GPUI 是否已初始化
    gpui_initialized: AtomicBool,

    /// GPUI 线程是否已启动
    gpui_thread_started: AtomicBool,
}

impl GlobalState {
    pub fn new() -> Self {
        Self {
            gpui_initialized: AtomicBool::new(false),
            gpui_thread_started: AtomicBool::new(false),
        }
    }

    /// 检查 GPUI 是否已初始化
    pub fn is_initialized(&self) -> bool {
        self.gpui_initialized.load(Ordering::SeqCst)
    }

    /// 设置 GPUI 初始化状态
    pub fn set_initialized(&self, value: bool) {
        self.gpui_initialized.store(value, Ordering::SeqCst);
    }

    /// 检查 GPUI 线程是否已启动
    pub fn is_thread_started(&self) -> bool {
        self.gpui_thread_started.load(Ordering::SeqCst)
    }

    /// 设置 GPUI 线程启动状态
    pub fn set_thread_started(&self, value: bool) {
        self.gpui_thread_started.store(value, Ordering::SeqCst);
    }
}

impl Default for GlobalState {
    fn default() -> Self {
        Self::new()
    }
}

lazy_static::lazy_static! {
    /// 全局状态实例
    pub static ref GLOBAL_STATE: GlobalState = GlobalState::new();
}
