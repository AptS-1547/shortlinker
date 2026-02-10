//! System operations for the TUI system panel
//!
//! Implements server status, config management, and password reset operations.
//! Follows IPC-first with direct DB fallback pattern for config operations,
//! IPC-only for server status, and direct DB only for password reset.

use super::state::{App, ConfigListItem};
use crate::config::definitions::{ALL_CONFIGS, get_def, keys};
use crate::config::schema::get_schema;
use crate::config::validators;
use crate::storage::ConfigStore;
use crate::system::ipc::{self, ConfigItemData, IpcCommand, IpcError, IpcResponse};

impl App {
    /// Fetch server status via IPC (IPC-only, no DB fallback)
    pub async fn fetch_server_status(&mut self) {
        self.system.status_error = None;

        match ipc::send_command(IpcCommand::GetStatus).await {
            Ok(IpcResponse::Status {
                version,
                uptime_secs,
                is_reloading,
                last_data_reload,
                last_config_reload,
                links_count,
            }) => {
                self.system.status_version = version;
                self.system.status_uptime_secs = uptime_secs;
                self.system.status_is_reloading = is_reloading;
                self.system.status_last_data_reload = last_data_reload;
                self.system.status_last_config_reload = last_config_reload;
                self.system.status_links_count = links_count;
            }
            Ok(IpcResponse::Error { message, .. }) => {
                self.system.status_error = Some(message);
            }
            Ok(other) => {
                self.system.status_error = Some(format!("Unexpected response: {:?}", other));
            }
            Err(e) => {
                self.system.status_error = Some(format!("{}", e));
            }
        }
    }

    /// Fetch all configs (IPC-first with direct DB fallback)
    pub async fn fetch_configs(&mut self) {
        // Try IPC first
        match ipc::config_list(None).await {
            Ok(IpcResponse::ConfigListResult { configs }) => {
                self.system.configs = configs;
                self.build_config_list_items();
                return;
            }
            Err(IpcError::ServerNotRunning) => {
                // Fall through to direct DB
            }
            Err(e) => {
                self.system.config_edit_error = Some(format!("Failed to fetch configs: {}", e));
                return;
            }
            Ok(IpcResponse::Error { message, .. }) => {
                self.system.config_edit_error =
                    Some(format!("Failed to fetch configs: {}", message));
                return;
            }
            Ok(_) => {
                self.system.config_edit_error = Some("Unexpected response from server".to_string());
                return;
            }
        }

        // Direct DB fallback
        self.fetch_configs_from_db().await;
    }

    /// Direct DB fallback for fetch_configs
    async fn fetch_configs_from_db(&mut self) {
        let store = ConfigStore::new(self.storage.get_db().clone());
        let stored = match store.get_all().await {
            Ok(map) => map,
            Err(e) => {
                self.system.config_edit_error =
                    Some(format!("Failed to load configs from database: {}", e));
                return;
            }
        };

        let mut configs = Vec::with_capacity(ALL_CONFIGS.len());
        for def in ALL_CONFIGS {
            let (value, updated_at) = match stored.get(def.key) {
                Some(item) => {
                    let val = (*item.value).clone();
                    (val, item.updated_at.to_rfc3339())
                }
                None => ((def.default_fn)(), String::new()),
            };

            // Mask sensitive values
            let display_value = if def.is_sensitive {
                "[REDACTED]".to_string()
            } else {
                value
            };

            // Get enum_options from schema
            let enum_options = get_schema(def.key).and_then(|schema| {
                schema
                    .enum_options
                    .map(|opts| opts.into_iter().map(|o| o.value).collect())
            });

            configs.push(ConfigItemData {
                key: def.key.to_string(),
                value: display_value,
                category: def.category.to_string(),
                value_type: def.value_type.to_string(),
                default_value: (def.default_fn)(),
                requires_restart: def.requires_restart,
                editable: def.editable,
                sensitive: def.is_sensitive,
                description: def.description.to_string(),
                enum_options,
                updated_at,
            });
        }

        self.system.configs = configs;
        self.build_config_list_items();
    }

    /// Build the flat config list items from configs, grouped by category
    pub fn build_config_list_items(&mut self) {
        self.system.config_list_items.clear();
        let mut last_category = String::new();

        for (index, cfg) in self.system.configs.iter().enumerate() {
            if cfg.category != last_category {
                last_category = cfg.category.clone();
                self.system
                    .config_list_items
                    .push(ConfigListItem::Header(cfg.category.clone()));
            }
            self.system.config_list_items.push(ConfigListItem::Config {
                index,
                key: cfg.key.clone(),
            });
        }

        // Reset selection to first Config item (skip initial Header)
        self.system.config_selected_index = 0;
        if !self.system.config_list_items.is_empty()
            && matches!(self.system.config_list_items[0], ConfigListItem::Header(_))
            && self.system.config_list_items.len() > 1
        {
            self.system.config_selected_index = 1;
        }
    }

    /// Get the currently selected config item
    pub fn get_selected_config(&self) -> Option<&ConfigItemData> {
        let item = self
            .system
            .config_list_items
            .get(self.system.config_selected_index)?;
        match item {
            ConfigListItem::Config { index, .. } => self.system.configs.get(*index),
            ConfigListItem::Header(_) => None,
        }
    }

    /// Update a config value (IPC-first with direct DB fallback)
    pub async fn update_config(&mut self) -> Result<(), String> {
        let key = self.system.config_edit_key.clone();
        let value = self.system.config_edit_value.clone();

        // Validate the value
        validators::validate_config_value(&key, &value)?;

        // Try IPC first
        match ipc::config_set(key.clone(), value.clone()).await {
            Ok(IpcResponse::ConfigSetResult { .. }) => return Ok(()),
            Ok(IpcResponse::Error { message, .. }) => return Err(message),
            Err(IpcError::ServerNotRunning) => {
                // Fall through to direct DB
            }
            Err(e) => return Err(format!("{}", e)),
            Ok(_) => return Err("Unexpected response from server".to_string()),
        }

        // Direct DB fallback
        let store = ConfigStore::new(self.storage.get_db().clone());
        store
            .set(&key, &value)
            .await
            .map(|_| ())
            .map_err(|e| format!("{}", e))
    }

    /// Reset a config to its default value (IPC-first with direct DB fallback)
    pub async fn reset_config(&mut self) -> Result<(), String> {
        let key = self.system.config_edit_key.clone();

        // Get default value from definitions
        let def = get_def(&key).ok_or_else(|| format!("Unknown config key: {}", key))?;
        let default_value = (def.default_fn)();

        // Try IPC first
        match ipc::config_reset(key.clone()).await {
            Ok(IpcResponse::ConfigResetResult { .. }) => return Ok(()),
            Ok(IpcResponse::Error { message, .. }) => return Err(message),
            Err(IpcError::ServerNotRunning) => {
                // Fall through to direct DB
            }
            Err(e) => return Err(format!("{}", e)),
            Ok(_) => return Err("Unexpected response from server".to_string()),
        }

        // Direct DB fallback
        let store = ConfigStore::new(self.storage.get_db().clone());
        store
            .set(&key, &default_value)
            .await
            .map(|_| ())
            .map_err(|e| format!("{}", e))
    }

    /// Reset admin password (direct DB only, for security)
    pub async fn reset_admin_password(&mut self) -> Result<(), String> {
        // Validate password length
        if self.system.password_input.len() < 8 {
            return Err("Password must be at least 8 characters".to_string());
        }

        // Validate passwords match
        if self.system.password_input != self.system.password_confirm {
            return Err("Passwords do not match".to_string());
        }

        // Hash the password
        let hash = crate::utils::password::process_new_password(Some(&self.system.password_input))
            .map_err(|e| format!("{}", e))?
            .ok_or_else(|| "Password cannot be empty".to_string())?;

        // Write directly to database
        let store = ConfigStore::new(self.storage.get_db().clone());
        store
            .set(keys::API_ADMIN_TOKEN, &hash)
            .await
            .map_err(|e| format!("{}", e))?;

        // Clear password fields on success
        self.system.password_input.clear();
        self.system.password_confirm.clear();

        Ok(())
    }

    /// Move config selection up, skipping Header items
    pub fn config_move_up(&mut self) {
        if self.system.config_list_items.is_empty() {
            return;
        }

        let mut idx = self.system.config_selected_index;
        loop {
            if idx == 0 {
                break;
            }
            idx -= 1;
            if matches!(
                self.system.config_list_items[idx],
                ConfigListItem::Config { .. }
            ) {
                self.system.config_selected_index = idx;
                break;
            }
        }
    }

    /// Move config selection down, skipping Header items
    pub fn config_move_down(&mut self) {
        if self.system.config_list_items.is_empty() {
            return;
        }

        let len = self.system.config_list_items.len();
        let mut idx = self.system.config_selected_index;
        loop {
            idx += 1;
            if idx >= len {
                break;
            }
            if matches!(
                self.system.config_list_items[idx],
                ConfigListItem::Config { .. }
            ) {
                self.system.config_selected_index = idx;
                break;
            }
        }
    }
}
