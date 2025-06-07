use std::sync::{Arc, OnceLock};

use super::manager::ClickManager;

pub static GLOBAL_CLICK_MANAGER: OnceLock<Arc<ClickManager>> = OnceLock::new();

/// 初始化全局点击管理器（只允许初始化一次）
pub fn set_global_click_manager(manager: Arc<ClickManager>) {
    if GLOBAL_CLICK_MANAGER.set(manager).is_err() {
        panic!("GLOBAL_CLICK_MANAGER has already been set");
    }
}

/// 获取全局点击管理器
pub fn get_click_manager() -> Arc<ClickManager> {
    GLOBAL_CLICK_MANAGER
        .get()
        .expect("GLOBAL_CLICK_MANAGER is not initialized")
        .clone()
}
