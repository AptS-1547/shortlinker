//! 系统面板状态管理
//!
//! 管理 Status/Config/PasswordReset 等系统运维面板的状态

use crate::system::ipc::ConfigItemData;

/// 密码重置表单当前编辑字段
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PasswordField {
    Password,
    Confirm,
}

/// 配置列表项（分类标题或配置项）
#[derive(Debug, Clone)]
pub enum ConfigListItem {
    /// 分类标题，导航时跳过
    Header(String),
    /// 配置项，index 指向 SystemState.configs[index]
    Config {
        index: usize,
        #[allow(dead_code)]
        key: String,
    },
}

/// 系统面板状态
#[derive(Debug, Default)]
pub struct SystemState {
    // ========== Server Status ==========
    pub status_version: String,
    pub status_uptime_secs: u64,
    pub status_is_reloading: bool,
    pub status_last_data_reload: Option<String>,
    pub status_last_config_reload: Option<String>,
    pub status_links_count: usize,
    pub status_error: Option<String>,

    // ========== Config List ==========
    pub configs: Vec<ConfigItemData>,
    pub config_selected_index: usize,
    pub config_list_items: Vec<ConfigListItem>,

    // ========== Config Edit ==========
    pub config_edit_key: String,
    pub config_edit_value: String,
    pub config_edit_error: Option<String>,

    // ========== Password Reset ==========
    pub password_input: String,
    pub password_confirm: String,
    pub password_field: Option<PasswordField>,
    pub password_error: Option<String>,

    // ========== Import Mode ==========
    pub import_overwrite: bool,
}
