use std::sync::{Arc, OnceLock};
use std::sync::atomic::{AtomicBool, Ordering};
use tracing::{trace, warn};

use super::manager::ClickManager;

pub static GLOBAL_CLICK_MANAGER: OnceLock<Arc<ClickManager>> = OnceLock::new();

/// 全局标志：是否应该停止详细日志记录（因为超过行数限制）
static DETAILED_LOGGING_STOPPED: AtomicBool = AtomicBool::new(false);

/// 初始化全局点击管理器（只允许初始化一次）
/// 重复初始化会返回 Ok 但发出警告
pub fn set_global_click_manager(manager: Arc<ClickManager>) {
    if GLOBAL_CLICK_MANAGER.set(manager).is_err() {
        warn!("GLOBAL_CLICK_MANAGER has already been initialized, ignoring");
    }
}

/// 获取全局点击管理器
pub fn get_click_manager() -> Option<&'static Arc<ClickManager>> {
    match GLOBAL_CLICK_MANAGER.get() {
        Some(manager) => Some(manager),
        None => {
            trace!("GLOBAL_CLICK_MANAGER has not been initialized yet");
            None
        }
    }
}

/// 检查是否应该停止详细日志记录
#[inline]
pub fn is_detailed_logging_stopped() -> bool {
    DETAILED_LOGGING_STOPPED.load(Ordering::Relaxed)
}

/// 设置停止详细日志记录标志
pub fn set_detailed_logging_stopped(stopped: bool) {
    DETAILED_LOGGING_STOPPED.store(stopped, Ordering::Relaxed);
    if stopped {
        warn!("Detailed logging stopped due to max_log_rows limit");
    }
}
