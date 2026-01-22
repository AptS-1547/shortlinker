//! IPC command handler
//!
//! Processes incoming IPC commands and returns appropriate responses.

use std::sync::{Arc, OnceLock};
use std::time::Instant;
use tracing::{debug, error, info, warn};

use super::types::{ImportLinkData, IpcCommand, IpcResponse, ShortLinkData};
use crate::cache::traits::CompositeCacheTrait;
use crate::storage::{LinkFilter, SeaOrmStorage, ShortLink};
use crate::system::reload::get_reload_coordinator;
use crate::utils::TimeParser;
use crate::utils::generate_random_code;
use crate::utils::password::{hash_password, is_argon2_hash};
use crate::utils::url_validator::validate_url;

/// Server start time for uptime calculation
static START_TIME: OnceLock<Instant> = OnceLock::new();

/// Storage instance for IPC handler
static STORAGE: OnceLock<Arc<SeaOrmStorage>> = OnceLock::new();

/// Cache instance for IPC handler
static CACHE: OnceLock<Arc<dyn CompositeCacheTrait>> = OnceLock::new();

/// Initialize the server start time
///
/// Should be called once during server startup.
pub fn init_start_time() {
    START_TIME.get_or_init(Instant::now);
}

/// Initialize storage and cache for IPC handler
///
/// Should be called once during server startup, after storage and cache are created.
pub fn init_storage_cache(storage: Arc<SeaOrmStorage>, cache: Arc<dyn CompositeCacheTrait>) {
    let _ = STORAGE.set(storage);
    let _ = CACHE.set(cache);
    debug!("IPC handler storage and cache initialized");
}

/// Get server uptime in seconds
fn get_uptime_secs() -> u64 {
    START_TIME.get().map(|t| t.elapsed().as_secs()).unwrap_or(0)
}

/// Get random code length from config
fn get_random_code_length() -> usize {
    crate::config::get_config().features.random_code_length
}

/// Get default cache TTL from config
fn get_default_cache_ttl() -> u64 {
    crate::config::get_config().cache.default_ttl
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

            // Get links count from storage if available
            let links_count = if let Some(storage) = STORAGE.get() {
                storage
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
    let Some(storage) = STORAGE.get() else {
        return IpcResponse::Error {
            code: "NOT_INITIALIZED".to_string(),
            message: "Storage not initialized".to_string(),
        };
    };
    let Some(cache) = CACHE.get() else {
        return IpcResponse::Error {
            code: "NOT_INITIALIZED".to_string(),
            message: "Cache not initialized".to_string(),
        };
    };

    // Validate URL
    if let Err(e) = validate_url(&target) {
        return IpcResponse::Error {
            code: "INVALID_URL".to_string(),
            message: e.to_string(),
        };
    }

    // Generate code if not provided
    let (final_code, generated) = match code {
        Some(c) if !c.is_empty() => (c, false),
        _ => (generate_random_code(get_random_code_length()), true),
    };

    // Check if code already exists
    let existing = match storage.get(&final_code).await {
        Ok(link) => link,
        Err(e) => {
            error!("IPC AddLink: failed to check existing link: {}", e);
            return IpcResponse::Error {
                code: "DATABASE_ERROR".to_string(),
                message: format!("Failed to check existing link: {}", e),
            };
        }
    };

    if existing.is_some() && !force {
        return IpcResponse::Error {
            code: "CONFLICT".to_string(),
            message: format!(
                "Code '{}' already exists. Use force=true to overwrite.",
                final_code
            ),
        };
    }

    // Parse expiration time
    let parsed_expires_at = if let Some(expire_str) = expires_at {
        match TimeParser::parse_expire_time(&expire_str) {
            Ok(dt) => Some(dt),
            Err(e) => {
                return IpcResponse::Error {
                    code: "INVALID_EXPIRE_TIME".to_string(),
                    message: format!("Invalid expiration time: {}", e),
                };
            }
        }
    } else {
        None
    };

    // Hash password if provided
    let hashed_password = match password {
        Some(pwd) if !pwd.is_empty() => {
            if is_argon2_hash(&pwd) {
                Some(pwd)
            } else {
                match hash_password(&pwd) {
                    Ok(hash) => Some(hash),
                    Err(e) => {
                        error!("IPC AddLink: failed to hash password: {}", e);
                        return IpcResponse::Error {
                            code: "HASH_ERROR".to_string(),
                            message: "Failed to process password".to_string(),
                        };
                    }
                }
            }
        }
        _ => None,
    };

    // Preserve original created_at and click if overwriting
    let (created_at, click) = if let Some(ref existing_link) = existing {
        (existing_link.created_at, existing_link.click)
    } else {
        (chrono::Utc::now(), 0)
    };

    let new_link = ShortLink {
        code: final_code.clone(),
        target,
        created_at,
        expires_at: parsed_expires_at,
        password: hashed_password,
        click,
    };

    // Save to storage
    if let Err(e) = storage.set(new_link.clone()).await {
        error!("IPC AddLink: failed to save link: {}", e);
        return IpcResponse::Error {
            code: "DATABASE_ERROR".to_string(),
            message: format!("Failed to save link: {}", e),
        };
    }

    // Update cache incrementally
    let ttl = new_link.cache_ttl(get_default_cache_ttl());
    cache.insert(&new_link.code, new_link.clone(), ttl).await;

    info!(
        "IPC AddLink: {} link '{}' -> '{}'",
        if existing.is_some() {
            "overwrote"
        } else {
            "created"
        },
        new_link.code,
        new_link.target
    );

    IpcResponse::LinkCreated {
        link: to_link_data(&new_link),
        generated_code: generated,
    }
}

async fn handle_remove_link(code: String) -> IpcResponse {
    let Some(storage) = STORAGE.get() else {
        return IpcResponse::Error {
            code: "NOT_INITIALIZED".to_string(),
            message: "Storage not initialized".to_string(),
        };
    };
    let Some(cache) = CACHE.get() else {
        return IpcResponse::Error {
            code: "NOT_INITIALIZED".to_string(),
            message: "Cache not initialized".to_string(),
        };
    };

    // Remove from storage
    if let Err(e) = storage.remove(&code).await {
        error!("IPC RemoveLink: failed to remove link: {}", e);
        return IpcResponse::Error {
            code: "DATABASE_ERROR".to_string(),
            message: format!("Failed to remove link: {}", e),
        };
    }

    // Update cache incrementally
    cache.remove(&code).await;

    info!("IPC RemoveLink: deleted '{}'", code);
    IpcResponse::LinkDeleted { code }
}

async fn handle_update_link(
    code: String,
    target: String,
    expires_at: Option<String>,
    password: Option<String>,
) -> IpcResponse {
    let Some(storage) = STORAGE.get() else {
        return IpcResponse::Error {
            code: "NOT_INITIALIZED".to_string(),
            message: "Storage not initialized".to_string(),
        };
    };
    let Some(cache) = CACHE.get() else {
        return IpcResponse::Error {
            code: "NOT_INITIALIZED".to_string(),
            message: "Cache not initialized".to_string(),
        };
    };

    // Validate URL
    if let Err(e) = validate_url(&target) {
        return IpcResponse::Error {
            code: "INVALID_URL".to_string(),
            message: e.to_string(),
        };
    }

    // Get existing link
    let existing = match storage.get(&code).await {
        Ok(Some(link)) => link,
        Ok(None) => {
            return IpcResponse::Error {
                code: "NOT_FOUND".to_string(),
                message: format!("Link '{}' not found", code),
            };
        }
        Err(e) => {
            error!("IPC UpdateLink: failed to get link: {}", e);
            return IpcResponse::Error {
                code: "DATABASE_ERROR".to_string(),
                message: format!("Failed to get link: {}", e),
            };
        }
    };

    // Parse expiration time
    let parsed_expires_at = if let Some(expire_str) = expires_at {
        match TimeParser::parse_expire_time(&expire_str) {
            Ok(dt) => Some(dt),
            Err(e) => {
                return IpcResponse::Error {
                    code: "INVALID_EXPIRE_TIME".to_string(),
                    message: format!("Invalid expiration time: {}", e),
                };
            }
        }
    } else {
        existing.expires_at
    };

    // Handle password update
    let updated_password = match password {
        Some(pwd) if !pwd.is_empty() => {
            if is_argon2_hash(&pwd) {
                Some(pwd)
            } else {
                match hash_password(&pwd) {
                    Ok(hash) => Some(hash),
                    Err(e) => {
                        error!("IPC UpdateLink: failed to hash password: {}", e);
                        return IpcResponse::Error {
                            code: "HASH_ERROR".to_string(),
                            message: "Failed to process password".to_string(),
                        };
                    }
                }
            }
        }
        Some(_) => None,                   // Empty string means remove password
        None => existing.password.clone(), // Not provided, keep existing
    };

    let updated_link = ShortLink {
        code: code.clone(),
        target,
        created_at: existing.created_at,
        expires_at: parsed_expires_at,
        password: updated_password,
        click: existing.click,
    };

    // Save to storage
    if let Err(e) = storage.set(updated_link.clone()).await {
        error!("IPC UpdateLink: failed to update link: {}", e);
        return IpcResponse::Error {
            code: "DATABASE_ERROR".to_string(),
            message: format!("Failed to update link: {}", e),
        };
    }

    // Update cache incrementally
    let ttl = updated_link.cache_ttl(get_default_cache_ttl());
    cache
        .insert(&updated_link.code, updated_link.clone(), ttl)
        .await;

    info!("IPC UpdateLink: updated '{}'", code);
    IpcResponse::LinkUpdated {
        link: to_link_data(&updated_link),
    }
}

async fn handle_get_link(code: String) -> IpcResponse {
    let Some(storage) = STORAGE.get() else {
        return IpcResponse::Error {
            code: "NOT_INITIALIZED".to_string(),
            message: "Storage not initialized".to_string(),
        };
    };

    match storage.get(&code).await {
        Ok(Some(link)) => IpcResponse::LinkFound {
            link: Some(to_link_data(&link)),
        },
        Ok(None) => IpcResponse::LinkFound { link: None },
        Err(e) => {
            error!("IPC GetLink: failed to get link: {}", e);
            IpcResponse::Error {
                code: "DATABASE_ERROR".to_string(),
                message: format!("Failed to get link: {}", e),
            }
        }
    }
}

async fn handle_list_links(page: u64, page_size: u64, search: Option<String>) -> IpcResponse {
    let Some(storage) = STORAGE.get() else {
        return IpcResponse::Error {
            code: "NOT_INITIALIZED".to_string(),
            message: "Storage not initialized".to_string(),
        };
    };

    let page = page.max(1);
    let page_size = page_size.clamp(1, 100);

    let filter = LinkFilter {
        search,
        created_after: None,
        created_before: None,
        only_expired: false,
        only_active: false,
    };

    match storage
        .load_paginated_filtered(page, page_size, filter)
        .await
    {
        Ok((links, total)) => {
            let link_data: Vec<ShortLinkData> = links.iter().map(to_link_data).collect();
            IpcResponse::LinkList {
                links: link_data,
                total: total as usize,
                page,
                page_size,
            }
        }
        Err(e) => {
            error!("IPC ListLinks: failed to list links: {}", e);
            IpcResponse::Error {
                code: "DATABASE_ERROR".to_string(),
                message: format!("Failed to list links: {}", e),
            }
        }
    }
}

async fn handle_import_links(links: Vec<ImportLinkData>, overwrite: bool) -> IpcResponse {
    let Some(storage) = STORAGE.get() else {
        return IpcResponse::Error {
            code: "NOT_INITIALIZED".to_string(),
            message: "Storage not initialized".to_string(),
        };
    };
    let Some(cache) = CACHE.get() else {
        return IpcResponse::Error {
            code: "NOT_INITIALIZED".to_string(),
            message: "Cache not initialized".to_string(),
        };
    };

    let mut success = 0;
    let mut failed = 0;
    let mut errors = Vec::new();

    for import_data in links {
        // Validate URL
        if let Err(e) = validate_url(&import_data.target) {
            failed += 1;
            errors.push(format!("{}: {}", import_data.code, e));
            continue;
        }

        // Check if exists
        let existing = match storage.get(&import_data.code).await {
            Ok(link) => link,
            Err(e) => {
                failed += 1;
                errors.push(format!("{}: Database error - {}", import_data.code, e));
                continue;
            }
        };

        if existing.is_some() && !overwrite {
            failed += 1;
            errors.push(format!("{}: Already exists", import_data.code));
            continue;
        }

        // Parse expiration time
        let parsed_expires_at = if let Some(expire_str) = &import_data.expires_at {
            match TimeParser::parse_expire_time(expire_str) {
                Ok(dt) => Some(dt),
                Err(e) => {
                    failed += 1;
                    errors.push(format!(
                        "{}: Invalid expiration time - {}",
                        import_data.code, e
                    ));
                    continue;
                }
            }
        } else {
            None
        };

        // Hash password if provided
        let hashed_password = match &import_data.password {
            Some(pwd) if !pwd.is_empty() => {
                if is_argon2_hash(pwd) {
                    Some(pwd.clone())
                } else {
                    match hash_password(pwd) {
                        Ok(hash) => Some(hash),
                        Err(_) => {
                            failed += 1;
                            errors.push(format!("{}: Failed to hash password", import_data.code));
                            continue;
                        }
                    }
                }
            }
            _ => None,
        };

        let (created_at, click) = if let Some(ref existing_link) = existing {
            (existing_link.created_at, existing_link.click)
        } else {
            (chrono::Utc::now(), 0)
        };

        let new_link = ShortLink {
            code: import_data.code.clone(),
            target: import_data.target,
            created_at,
            expires_at: parsed_expires_at,
            password: hashed_password,
            click,
        };

        // Save to storage
        if let Err(e) = storage.set(new_link.clone()).await {
            failed += 1;
            errors.push(format!("{}: Failed to save - {}", import_data.code, e));
            continue;
        }

        // Update cache incrementally
        let ttl = new_link.cache_ttl(get_default_cache_ttl());
        cache.insert(&new_link.code.clone(), new_link, ttl).await;

        success += 1;
    }

    info!(
        "IPC ImportLinks: imported {} links, {} failed",
        success, failed
    );
    IpcResponse::ImportResult {
        success,
        failed,
        errors,
    }
}

async fn handle_export_links() -> IpcResponse {
    let Some(storage) = STORAGE.get() else {
        return IpcResponse::Error {
            code: "NOT_INITIALIZED".to_string(),
            message: "Storage not initialized".to_string(),
        };
    };

    match storage.load_all().await {
        Ok(links_map) => {
            let links: Vec<ShortLinkData> = links_map.values().map(to_link_data).collect();
            info!("IPC ExportLinks: exported {} links", links.len());
            IpcResponse::ExportResult { links }
        }
        Err(e) => {
            error!("IPC ExportLinks: failed to load links: {}", e);
            IpcResponse::Error {
                code: "DATABASE_ERROR".to_string(),
                message: format!("Failed to load links: {}", e),
            }
        }
    }
}

async fn handle_get_stats() -> IpcResponse {
    let Some(storage) = STORAGE.get() else {
        return IpcResponse::Error {
            code: "NOT_INITIALIZED".to_string(),
            message: "Storage not initialized".to_string(),
        };
    };

    match storage.get_stats().await {
        Ok(stats) => IpcResponse::StatsResult {
            total_links: stats.total_links,
            total_clicks: stats.total_clicks as i64,
            active_links: stats.active_links,
        },
        Err(e) => {
            error!("IPC GetStats: failed to get stats: {}", e);
            IpcResponse::Error {
                code: "DATABASE_ERROR".to_string(),
                message: format!("Failed to get stats: {}", e),
            }
        }
    }
}
