//! IPC command handler
//!
//! Processes incoming IPC commands and returns appropriate responses.
//! Uses LinkService for link management operations.

use std::sync::{Arc, OnceLock};
use std::time::Instant;
use tracing::{debug, info, warn};

use super::types::{ImportLinkData, IpcCommand, IpcResponse, ShortLinkData};
use crate::services::{
    CreateLinkRequest, ImportLinkItem, ImportMode, LinkService, ServiceError, UpdateLinkRequest,
};
use crate::storage::{LinkFilter, ShortLink};
use crate::system::reload::get_reload_coordinator;

/// Server start time for uptime calculation
static START_TIME: OnceLock<Instant> = OnceLock::new();

/// LinkService instance for IPC handler
static LINK_SERVICE: OnceLock<Arc<LinkService>> = OnceLock::new();

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

/// Convert ServiceError to IpcResponse::Error
fn error_response(err: ServiceError) -> IpcResponse {
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
        return error_response(ServiceError::NotInitialized);
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
        return error_response(ServiceError::NotInitialized);
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
        return error_response(ServiceError::NotInitialized);
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
        return error_response(ServiceError::NotInitialized);
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
        return error_response(ServiceError::NotInitialized);
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
        return error_response(ServiceError::NotInitialized);
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
            let errors: Vec<String> = result
                .errors
                .into_iter()
                .map(|e| format!("{}: {}", e.code, e.message))
                .collect();
            IpcResponse::ImportResult {
                success: result.success,
                failed: result.failed + result.skipped,
                errors,
            }
        }
        Err(e) => error_response(e),
    }
}

async fn handle_export_links() -> IpcResponse {
    let Some(service) = LINK_SERVICE.get() else {
        return error_response(ServiceError::NotInitialized);
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
        return error_response(ServiceError::NotInitialized);
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
