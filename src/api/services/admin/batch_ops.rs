//! Admin API 批量操作

use actix_web::{HttpRequest, Responder, Result as ActixResult, web};
use std::sync::Arc;
use tracing::info;

use crate::cache::traits::CompositeCacheTrait;
use crate::storage::{SeaOrmStorage, ShortLink};
use crate::system::reload::reload_all;
use crate::utils::generate_random_code;

use super::helpers::{parse_expires_at, success_response};
use super::types::{
    BatchCreateRequest, BatchDeleteRequest, BatchFailedItem, BatchResponse, BatchUpdateRequest,
};
use super::get_random_code_length;

/// 批量创建链接
pub async fn batch_create_links(
    _req: HttpRequest,
    batch: web::Json<BatchCreateRequest>,
    cache: web::Data<Arc<dyn CompositeCacheTrait>>,
    storage: web::Data<Arc<SeaOrmStorage>>,
) -> ActixResult<impl Responder> {
    info!(
        "Admin API: batch create request - {} links",
        batch.links.len()
    );

    let mut success: Vec<String> = Vec::new();
    let mut failed: Vec<BatchFailedItem> = Vec::new();

    for link_req in &batch.links {
        // Generate code if not provided
        let code = match &link_req.code {
            Some(c) if !c.is_empty() => c.clone(),
            _ => generate_random_code(get_random_code_length()),
        };

        // Validate target URL
        if !link_req.target.starts_with("http://") && !link_req.target.starts_with("https://") {
            failed.push(BatchFailedItem {
                code: code.clone(),
                error: "URL must start with http:// or https://".to_string(),
            });
            continue;
        }

        // Check if link already exists
        let existing = storage.get(&code).await;
        let force = link_req.force.unwrap_or(false);

        if existing.is_some() && !force {
            failed.push(BatchFailedItem {
                code: code.clone(),
                error: "Link already exists".to_string(),
            });
            continue;
        }

        // Parse expiration time
        let expires_at = match &link_req.expires_at {
            Some(expire_str) => match parse_expires_at(expire_str) {
                Ok(time) => Some(time),
                Err(error_msg) => {
                    failed.push(BatchFailedItem {
                        code: code.clone(),
                        error: error_msg,
                    });
                    continue;
                }
            },
            None => None,
        };

        // Preserve original data if force overwriting
        let (created_at, click) = if let Some(ref ex) = existing {
            (ex.created_at, ex.click)
        } else {
            (chrono::Utc::now(), 0)
        };

        let new_link = ShortLink {
            code: code.clone(),
            target: link_req.target.clone(),
            created_at,
            expires_at,
            password: link_req.password.clone(),
            click,
        };

        match storage.set(new_link).await {
            Ok(_) => success.push(code),
            Err(e) => {
                failed.push(BatchFailedItem {
                    code,
                    error: format!("Database error: {}", e),
                });
            }
        }
    }

    // Reload cache once after all operations
    if !success.is_empty() {
        let _ = reload_all(cache.get_ref().clone(), storage.get_ref().clone()).await;
    }

    info!(
        "Admin API: batch create completed - {} success, {} failed",
        success.len(),
        failed.len()
    );

    Ok(success_response(BatchResponse { success, failed }))
}

/// 批量更新链接
pub async fn batch_update_links(
    _req: HttpRequest,
    batch: web::Json<BatchUpdateRequest>,
    cache: web::Data<Arc<dyn CompositeCacheTrait>>,
    storage: web::Data<Arc<SeaOrmStorage>>,
) -> ActixResult<impl Responder> {
    info!(
        "Admin API: batch update request - {} links",
        batch.updates.len()
    );

    let mut success: Vec<String> = Vec::new();
    let mut failed: Vec<BatchFailedItem> = Vec::new();

    for update in &batch.updates {
        let code = &update.code;
        let payload = &update.payload;

        // Validate target URL
        if !payload.target.starts_with("http://") && !payload.target.starts_with("https://") {
            failed.push(BatchFailedItem {
                code: code.clone(),
                error: "URL must start with http:// or https://".to_string(),
            });
            continue;
        }

        // Get existing link
        let existing = match storage.get(code).await {
            Some(link) => link,
            None => {
                failed.push(BatchFailedItem {
                    code: code.clone(),
                    error: "Link not found".to_string(),
                });
                continue;
            }
        };

        // Parse expiration time
        let expires_at = match &payload.expires_at {
            Some(expire_str) => match parse_expires_at(expire_str) {
                Ok(time) => Some(time),
                Err(error_msg) => {
                    failed.push(BatchFailedItem {
                        code: code.clone(),
                        error: error_msg,
                    });
                    continue;
                }
            },
            None => existing.expires_at,
        };

        let updated_link = ShortLink {
            code: code.clone(),
            target: payload.target.clone(),
            created_at: existing.created_at,
            expires_at,
            password: if payload.password.is_some() {
                payload.password.clone()
            } else {
                existing.password
            },
            click: existing.click,
        };

        match storage.set(updated_link).await {
            Ok(_) => success.push(code.clone()),
            Err(e) => {
                failed.push(BatchFailedItem {
                    code: code.clone(),
                    error: format!("Database error: {}", e),
                });
            }
        }
    }

    // Reload cache once after all operations
    if !success.is_empty() {
        let _ = reload_all(cache.get_ref().clone(), storage.get_ref().clone()).await;
    }

    info!(
        "Admin API: batch update completed - {} success, {} failed",
        success.len(),
        failed.len()
    );

    Ok(success_response(BatchResponse { success, failed }))
}

/// 批量删除链接
pub async fn batch_delete_links(
    _req: HttpRequest,
    batch: web::Json<BatchDeleteRequest>,
    cache: web::Data<Arc<dyn CompositeCacheTrait>>,
    storage: web::Data<Arc<SeaOrmStorage>>,
) -> ActixResult<impl Responder> {
    info!(
        "Admin API: batch delete request - {} links",
        batch.codes.len()
    );

    let mut success: Vec<String> = Vec::new();
    let mut failed: Vec<BatchFailedItem> = Vec::new();

    for code in &batch.codes {
        match storage.remove(code).await {
            Ok(_) => success.push(code.clone()),
            Err(e) => {
                failed.push(BatchFailedItem {
                    code: code.clone(),
                    error: format!("{}", e),
                });
            }
        }
    }

    // Reload cache once after all operations
    if !success.is_empty() {
        let _ = reload_all(cache.get_ref().clone(), storage.get_ref().clone()).await;
    }

    info!(
        "Admin API: batch delete completed - {} success, {} failed",
        success.len(),
        failed.len()
    );

    Ok(success_response(BatchResponse { success, failed }))
}
