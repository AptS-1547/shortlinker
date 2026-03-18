//! System operations for the TUI system panel
//!
//! Implements server status, config management, and password reset operations.
//! Uses ConfigClient/SystemClient for IPC-first with service-fallback,
//! except password reset which stays as direct DB for security.

use super::state::{App, ConfigListItem};
use crate::config::definitions::{get_def, keys};
use crate::config::schema::get_schema;
use crate::config::validators;
use crate::services::ConfigItemView;
use crate::storage::ConfigStore;
use crate::system::ipc::ConfigItemData;

/// Convert a ConfigItemView (from client layer) to ConfigItemData (for TUI state).
///
/// Enriches the view with definition metadata (category, description, default_value,
/// editable, enum_options) that the client layer doesn't carry.
fn config_view_to_item_data(view: ConfigItemView) -> ConfigItemData {
    let def = get_def(&view.key);
    let schema = get_schema(&view.key);
    let enum_options = schema.and_then(|s| {
        s.enum_options
            .map(|opts| opts.into_iter().map(|o| o.value).collect())
    });
    ConfigItemData {
        key: view.key.clone(),
        value: view.value,
        category: def.map(|d| d.category.to_string()).unwrap_or_default(),
        value_type: def
            .map(|d| d.value_type.to_string())
            .unwrap_or_else(|| format!("{}", view.value_type)),
        default_value: def.map(|d| (d.default_fn)()).unwrap_or_default(),
        requires_restart: view.requires_restart,
        editable: def.map(|d| d.editable).unwrap_or(false),
        sensitive: view.is_sensitive,
        description: def.map(|d| d.description.to_string()).unwrap_or_default(),
        enum_options,
        updated_at: view.updated_at.to_rfc3339(),
    }
}

impl App {
    /// Fetch server status via SystemClient (IPC-only, no DB fallback)
    pub async fn fetch_server_status(&mut self) {
        self.system.status_error = None;

        match self.system_client.get_status().await {
            Ok(status) => {
                self.system.status_version = status.version;
                self.system.status_uptime_secs = status.uptime_secs;
                self.system.status_is_reloading = status.is_reloading;
                self.system.status_last_data_reload = status.last_data_reload;
                self.system.status_last_config_reload = status.last_config_reload;
                self.system.status_links_count = status.links_count;
            }
            Err(e) => {
                self.system.status_error = Some(format!("{}", e));
            }
        }
    }

    /// Fetch all configs via ConfigClient (IPC-first with service-fallback)
    pub async fn fetch_configs(&mut self) {
        match self.config_client.get_all(None).await {
            Ok(views) => {
                self.system.configs = views.into_iter().map(config_view_to_item_data).collect();
                self.build_config_list_items();
            }
            Err(e) => {
                self.system.config_edit_error = Some(format!("Failed to fetch configs: {}", e));
            }
        }
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

    /// Update a config value via ConfigClient (IPC-first with service-fallback)
    pub async fn update_config(&mut self) -> Result<(), String> {
        let key = self.system.config_edit_key.clone();
        let value = self.system.config_edit_value.clone();

        // Validate the value (client-side for immediate feedback)
        validators::validate_config_value(&key, &value)?;

        self.config_client
            .set(key, value)
            .await
            .map(|_| ())
            .map_err(|e| format!("{}", e))
    }

    /// Reset a config to its default value via ConfigClient (IPC-first with service-fallback)
    pub async fn reset_config(&mut self) -> Result<(), String> {
        let key = self.system.config_edit_key.clone();

        self.config_client
            .reset(key)
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
