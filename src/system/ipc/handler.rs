//! IPC command handler
//!
//! Processes incoming IPC commands and returns appropriate responses.
//! Uses LinkService for link management operations.

use std::sync::{Arc, OnceLock};
use std::time::Instant;
use tracing::{debug, info, warn};

use super::types::{
    ConfigItemData, ImportErrorData, ImportLinkData, IpcCommand, IpcResponse, ShortLinkData,
};
use crate::errors::ShortlinkerError;
use crate::services::{
    ConfigService, CreateLinkRequest, ImportLinkItem, ImportMode, LinkService, UpdateLinkRequest,
};
use crate::storage::{LinkFilter, ShortLink};
use crate::system::reload::get_reload_coordinator;

/// Server start time for uptime calculation
static START_TIME: OnceLock<Instant> = OnceLock::new();

/// LinkService instance for IPC handler
static LINK_SERVICE: OnceLock<Arc<LinkService>> = OnceLock::new();

/// ConfigService instance for IPC handler
static CONFIG_SERVICE: OnceLock<Arc<ConfigService>> = OnceLock::new();

/// Initialize the server start time
///
/// Should be called once during server startup.
pub fn init_start_time() {
    START_TIME.get_or_init(Instant::now);
}

/// Initialize LinkService for IPC handler
///
/// Should be called once during server startup, after storage and cache are created.
pub fn init_link_service(service: Arc<LinkService>) {
    let _ = LINK_SERVICE.set(service);
    debug!("IPC handler LinkService initialized");
}

/// Initialize ConfigService for IPC handler
///
/// Should be called once during server startup, after RuntimeConfig is initialized.
pub fn init_config_service(service: Arc<ConfigService>) {
    let _ = CONFIG_SERVICE.set(service);
    debug!("IPC handler ConfigService initialized");
}

/// Get server uptime in seconds
fn get_uptime_secs() -> u64 {
    START_TIME.get().map(|t| t.elapsed().as_secs()).unwrap_or(0)
}

/// Convert ShortLink to ShortLinkData for IPC transfer
fn to_link_data(link: &ShortLink) -> ShortLinkData {
    ShortLinkData {
        code: link.code.clone(),
        target: link.target.clone(),
        created_at: link.created_at.to_rfc3339(),
        expires_at: link.expires_at.map(|dt| dt.to_rfc3339()),
        password: link.password.clone(),
        click: link.click as i64,
    }
}

/// Convert ShortlinkerError to IpcResponse::Error
fn error_response(err: ShortlinkerError) -> IpcResponse {
    IpcResponse::Error {
        code: err.code().to_string(),
        message: err.to_string(),
    }
}

/// Handle an IPC command and return a response
pub async fn handle_command(cmd: IpcCommand) -> IpcResponse {
    debug!("Handling IPC command: {:?}", cmd);

    match cmd {
        IpcCommand::Ping => IpcResponse::Pong {
            version: env!("CARGO_PKG_VERSION").to_string(),
            uptime_secs: get_uptime_secs(),
        },

        IpcCommand::Reload { target } => {
            info!("IPC reload request received: {:?}", target);

            let Some(coordinator) = get_reload_coordinator() else {
                warn!("ReloadCoordinator not initialized");
                return IpcResponse::Error {
                    code: "NOT_INITIALIZED".to_string(),
                    message: "ReloadCoordinator not initialized".to_string(),
                };
            };

            match coordinator.reload(target).await {
                Ok(result) => {
                    info!(
                        "IPC reload completed: {:?} in {}ms",
                        target, result.duration_ms
                    );
                    IpcResponse::ReloadResult {
                        success: true,
                        target,
                        duration_ms: result.duration_ms,
                        message: None,
                    }
                }
                Err(e) => {
                    warn!("IPC reload failed: {:?} - {}", target, e);
                    IpcResponse::ReloadResult {
                        success: false,
                        target,
                        duration_ms: 0,
                        message: Some(e.to_string()),
                    }
                }
            }
        }

        IpcCommand::GetStatus => {
            let coordinator = get_reload_coordinator();
            let status = coordinator.map(|c| c.status());

            // Get links count from service if available
            let links_count = if let Some(service) = LINK_SERVICE.get() {
                service
                    .get_stats()
                    .await
                    .map(|s| s.total_links)
                    .unwrap_or(0)
            } else {
                0
            };

            IpcResponse::Status {
                version: env!("CARGO_PKG_VERSION").to_string(),
                uptime_secs: get_uptime_secs(),
                is_reloading: status.as_ref().map(|s| s.is_reloading).unwrap_or(false),
                last_data_reload: status
                    .as_ref()
                    .and_then(|s| s.last_data_reload.as_ref())
                    .map(|r| r.finished_at.to_rfc3339()),
                last_config_reload: status
                    .as_ref()
                    .and_then(|s| s.last_config_reload.as_ref())
                    .map(|r| r.finished_at.to_rfc3339()),
                links_count,
            }
        }

        IpcCommand::Shutdown => {
            info!("IPC shutdown request received");
            // Note: Actual shutdown is handled by the caller
            // This just acknowledges the request
            IpcResponse::ShuttingDown
        }

        // ============ Link Management Commands ============
        IpcCommand::AddLink {
            code,
            target,
            force,
            expires_at,
            password,
        } => handle_add_link(code, target, force, expires_at, password).await,

        IpcCommand::RemoveLink { code } => handle_remove_link(code).await,

        IpcCommand::UpdateLink {
            code,
            target,
            expires_at,
            password,
        } => handle_update_link(code, target, expires_at, password).await,

        IpcCommand::GetLink { code } => handle_get_link(code).await,

        IpcCommand::ListLinks {
            page,
            page_size,
            search,
        } => handle_list_links(page, page_size, search).await,

        IpcCommand::ImportLinks { links, overwrite } => handle_import_links(links, overwrite).await,

        IpcCommand::ExportLinks => handle_export_links().await,

        IpcCommand::GetLinkStats => handle_get_stats().await,

        // ============ Config Management Commands ============
        IpcCommand::ConfigList { category } => handle_config_list(category).await,

        IpcCommand::ConfigGet { key } => handle_config_get(key).await,

        IpcCommand::ConfigSet { key, value } => handle_config_set(key, value).await,

        IpcCommand::ConfigReset { key } => handle_config_reset(key).await,

        IpcCommand::ConfigImport { configs } => handle_config_import(configs).await,
    }
}

// ============ Link Management Handlers ============

async fn handle_add_link(
    code: Option<String>,
    target: String,
    force: bool,
    expires_at: Option<String>,
    password: Option<String>,
) -> IpcResponse {
    let Some(service) = LINK_SERVICE.get() else {
        return error_response(ShortlinkerError::service_unavailable(
            "Service not initialized",
        ));
    };

    let req = CreateLinkRequest {
        code,
        target,
        force,
        expires_at,
        password,
    };

    match service.create_link(req).await {
        Ok(result) => IpcResponse::LinkCreated {
            link: to_link_data(&result.link),
            generated_code: result.generated_code,
        },
        Err(e) => error_response(e),
    }
}

async fn handle_remove_link(code: String) -> IpcResponse {
    let Some(service) = LINK_SERVICE.get() else {
        return error_response(ShortlinkerError::service_unavailable(
            "Service not initialized",
        ));
    };

    match service.delete_link(&code).await {
        Ok(()) => IpcResponse::LinkDeleted { code },
        Err(e) => error_response(e),
    }
}

async fn handle_update_link(
    code: String,
    target: String,
    expires_at: Option<String>,
    password: Option<String>,
) -> IpcResponse {
    let Some(service) = LINK_SERVICE.get() else {
        return error_response(ShortlinkerError::service_unavailable(
            "Service not initialized",
        ));
    };

    let req = UpdateLinkRequest {
        target,
        expires_at,
        password,
    };

    match service.update_link(&code, req).await {
        Ok(link) => IpcResponse::LinkUpdated {
            link: to_link_data(&link),
        },
        Err(e) => error_response(e),
    }
}

async fn handle_get_link(code: String) -> IpcResponse {
    let Some(service) = LINK_SERVICE.get() else {
        return error_response(ShortlinkerError::service_unavailable(
            "Service not initialized",
        ));
    };

    match service.get_link(&code).await {
        Ok(link) => IpcResponse::LinkFound {
            link: link.map(|l| to_link_data(&l)),
        },
        Err(e) => error_response(e),
    }
}

async fn handle_list_links(page: u64, page_size: u64, search: Option<String>) -> IpcResponse {
    let Some(service) = LINK_SERVICE.get() else {
        return error_response(ShortlinkerError::service_unavailable(
            "Service not initialized",
        ));
    };

    let filter = LinkFilter {
        search,
        created_after: None,
        created_before: None,
        only_expired: false,
        only_active: false,
    };

    match service.list_links(filter, page, page_size).await {
        Ok((links, total)) => {
            let link_data: Vec<ShortLinkData> = links.iter().map(to_link_data).collect();
            IpcResponse::LinkList {
                links: link_data,
                total: total as usize,
                page,
                page_size,
            }
        }
        Err(e) => error_response(e),
    }
}

async fn handle_import_links(links: Vec<ImportLinkData>, overwrite: bool) -> IpcResponse {
    let Some(service) = LINK_SERVICE.get() else {
        return error_response(ShortlinkerError::service_unavailable(
            "Service not initialized",
        ));
    };

    // Convert IPC ImportLinkData to service ImportLinkItem
    let items: Vec<ImportLinkItem> = links
        .into_iter()
        .map(|l| ImportLinkItem {
            code: l.code,
            target: l.target,
            expires_at: l.expires_at,
            password: l.password,
        })
        .collect();

    let mode = ImportMode::from_overwrite_flag(overwrite);

    match service.import_links(items, mode).await {
        Ok(result) => {
            let errors: Vec<ImportErrorData> = result
                .errors
                .into_iter()
                .map(|e| ImportErrorData {
                    code: e.code,
                    message: e.message,
                })
                .collect();
            IpcResponse::ImportResult {
                success: result.success,
                skipped: result.skipped,
                failed: result.failed,
                errors,
            }
        }
        Err(e) => error_response(e),
    }
}

async fn handle_export_links() -> IpcResponse {
    let Some(service) = LINK_SERVICE.get() else {
        return error_response(ShortlinkerError::service_unavailable(
            "Service not initialized",
        ));
    };

    match service.export_links().await {
        Ok(links) => {
            let link_data: Vec<ShortLinkData> = links.iter().map(to_link_data).collect();
            IpcResponse::ExportResult { links: link_data }
        }
        Err(e) => error_response(e),
    }
}

async fn handle_get_stats() -> IpcResponse {
    let Some(service) = LINK_SERVICE.get() else {
        return error_response(ShortlinkerError::service_unavailable(
            "Service not initialized",
        ));
    };

    match service.get_stats().await {
        Ok(stats) => IpcResponse::StatsResult {
            total_links: stats.total_links,
            total_clicks: stats.total_clicks as i64,
            active_links: stats.active_links,
        },
        Err(e) => error_response(e),
    }
}

// ============ Config Management Handlers ============

use crate::config::definitions::get_def;
use crate::config::schema::get_schema;
use crate::config::validators;
use crate::services::ConfigItemView;

/// Build ConfigItemData from a ConfigItemView by enriching with definition metadata
fn to_config_item_data(view: ConfigItemView) -> ConfigItemData {
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
        value_type: format!("{}", view.value_type),
        default_value: def.map(|d| (d.default_fn)()).unwrap_or_default(),
        requires_restart: view.requires_restart,
        editable: def.map(|d| d.editable).unwrap_or(false),
        sensitive: view.is_sensitive,
        description: def.map(|d| d.description.to_string()).unwrap_or_default(),
        enum_options,
        updated_at: view.updated_at.to_rfc3339(),
    }
}

async fn handle_config_list(category: Option<String>) -> IpcResponse {
    let Some(service) = CONFIG_SERVICE.get() else {
        return error_response(ShortlinkerError::service_unavailable(
            "ConfigService not initialized",
        ));
    };

    let all = service.get_all();
    let configs: Vec<ConfigItemData> = all
        .into_iter()
        .filter(|item| {
            if let Some(ref cat) = category {
                get_def(&item.key)
                    .map(|d| d.category == cat.as_str())
                    .unwrap_or(false)
            } else {
                true
            }
        })
        .map(to_config_item_data)
        .collect();

    IpcResponse::ConfigListResult { configs }
}

async fn handle_config_get(key: String) -> IpcResponse {
    let Some(service) = CONFIG_SERVICE.get() else {
        return error_response(ShortlinkerError::service_unavailable(
            "ConfigService not initialized",
        ));
    };

    match service.get(&key) {
        Ok(view) => IpcResponse::ConfigGetResult {
            config: to_config_item_data(view),
        },
        Err(e) => error_response(e),
    }
}

async fn handle_config_set(key: String, value: String) -> IpcResponse {
    let Some(service) = CONFIG_SERVICE.get() else {
        return error_response(ShortlinkerError::service_unavailable(
            "ConfigService not initialized",
        ));
    };

    // Validate key exists and is editable
    let def = match get_def(&key) {
        Some(d) => d,
        None => {
            return IpcResponse::Error {
                code: "CONFIG_NOT_FOUND".to_string(),
                message: format!("Unknown configuration key: '{}'", key),
            };
        }
    };

    if !def.editable {
        return IpcResponse::Error {
            code: "CONFIG_READONLY".to_string(),
            message: format!("Configuration '{}' is read-only", key),
        };
    }

    // Validate value
    if let Err(e) = validators::validate_config_value(&key, &value) {
        return IpcResponse::Error {
            code: "CONFIG_INVALID_VALUE".to_string(),
            message: format!("Invalid value for '{}': {}", key, e),
        };
    }

    match service.update(&key, &value).await {
        Ok(view) => {
            info!("Config '{}' updated via IPC", key);
            IpcResponse::ConfigSetResult {
                key: view.key,
                value: view.value,
                requires_restart: view.requires_restart,
                is_sensitive: view.is_sensitive,
                old_value: None, // ConfigService doesn't expose old value
                message: view.message,
            }
        }
        Err(e) => error_response(e),
    }
}

async fn handle_config_reset(key: String) -> IpcResponse {
    let Some(service) = CONFIG_SERVICE.get() else {
        return error_response(ShortlinkerError::service_unavailable(
            "ConfigService not initialized",
        ));
    };

    let def = match get_def(&key) {
        Some(d) => d,
        None => {
            return IpcResponse::Error {
                code: "CONFIG_NOT_FOUND".to_string(),
                message: format!("Unknown configuration key: '{}'", key),
            };
        }
    };

    if !def.editable {
        return IpcResponse::Error {
            code: "CONFIG_READONLY".to_string(),
            message: format!("Configuration '{}' is read-only", key),
        };
    }

    let default_value = (def.default_fn)();

    match service.update(&key, &default_value).await {
        Ok(view) => {
            info!("Config '{}' reset to default via IPC", key);
            IpcResponse::ConfigResetResult {
                key: view.key,
                value: view.value,
                requires_restart: view.requires_restart,
                is_sensitive: view.is_sensitive,
                message: view.message,
            }
        }
        Err(e) => error_response(e),
    }
}

async fn handle_config_import(configs: Vec<super::types::ConfigImportItem>) -> IpcResponse {
    let Some(service) = CONFIG_SERVICE.get() else {
        return error_response(ShortlinkerError::service_unavailable(
            "ConfigService not initialized",
        ));
    };

    let mut success = 0usize;
    let mut skipped = 0usize;
    let mut failed = 0usize;
    let mut errors = Vec::new();

    for item in &configs {
        // Validate key
        let def = match get_def(&item.key) {
            Some(d) => d,
            None => {
                skipped += 1;
                errors.push(format!("{}: unknown key", item.key));
                continue;
            }
        };

        if !def.editable {
            skipped += 1;
            errors.push(format!("{}: read-only", item.key));
            continue;
        }

        // Validate value
        if let Err(e) = validators::validate_config_value(&item.key, &item.value) {
            failed += 1;
            errors.push(format!("{}: {}", item.key, e));
            continue;
        }

        match service.update(&item.key, &item.value).await {
            Ok(_) => {
                success += 1;
            }
            Err(e) => {
                failed += 1;
                errors.push(format!("{}: {}", item.key, e));
            }
        }
    }

    if success > 0 {
        info!(
            "Config import via IPC: {} success, {} skipped, {} failed",
            success, skipped, failed
        );
    }

    IpcResponse::ConfigImportResult {
        success,
        skipped,
        failed,
        errors,
    }
}
