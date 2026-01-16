//! Admin API 批量操作

use actix_web::{HttpRequest, Responder, Result as ActixResult, web};
use std::sync::Arc;
use tracing::info;

use crate::cache::traits::CompositeCacheTrait;
use crate::storage::{SeaOrmStorage, ShortLink};
use crate::utils::generate_random_code;
use crate::utils::password::{hash_password, is_argon2_hash};
use crate::utils::url_validator::validate_url;

use super::get_random_code_length;
use super::helpers::{parse_expires_at, success_response};
use super::types::{
    BatchCreateRequest, BatchDeleteRequest, BatchFailedItem, BatchResponse, BatchUpdateRequest,
};

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
    let mut created_links: Vec<ShortLink> = Vec::new();

    for link_req in &batch.links {
        // Generate code if not provided
        let code = match &link_req.code {
            Some(c) if !c.is_empty() => c.clone(),
            _ => generate_random_code(get_random_code_length()),
        };

        // Validate target URL
        if let Err(e) = validate_url(&link_req.target) {
            failed.push(BatchFailedItem {
                code: code.clone(),
                error: e.to_string(),
            });
            continue;
        }

        // Check if link already exists
        let existing = match storage.get(&code).await {
            Ok(opt) => opt,
            Err(e) => {
                failed.push(BatchFailedItem {
                    code: code.clone(),
                    error: format!("Database query error: {}", e),
                });
                continue;
            }
        };
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

        // Hash password if provided
        let hashed_password = match &link_req.password {
            Some(pwd) if !pwd.is_empty() => {
                if is_argon2_hash(pwd) {
                    Some(pwd.clone())
                } else {
                    match hash_password(pwd) {
                        Ok(hash) => Some(hash),
                        Err(e) => {
                            failed.push(BatchFailedItem {
                                code: code.clone(),
                                error: format!("Failed to hash password: {}", e),
                            });
                            continue;
                        }
                    }
                }
            }
            _ => None,
        };

        let new_link = ShortLink {
            code: code.clone(),
            target: link_req.target.clone(),
            created_at,
            expires_at,
            password: hashed_password,
            click,
        };

        match storage.set(new_link.clone()).await {
            Ok(_) => {
                success.push(code);
                created_links.push(new_link);
            }
            Err(e) => {
                failed.push(BatchFailedItem {
                    code,
                    error: format!("Database error: {}", e),
                });
            }
        }
    }

    // 增量更新缓存
    for link in created_links {
        let code = link.code.clone();
        let ttl = link.cache_ttl(crate::config::get_config().cache.default_ttl);
        cache.insert(&code, link, ttl).await;
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
    let mut updated_links: Vec<ShortLink> = Vec::new();

    for update in &batch.updates {
        let code = &update.code;
        let payload = &update.payload;

        // Validate target URL
        if let Err(e) = validate_url(&payload.target) {
            failed.push(BatchFailedItem {
                code: code.clone(),
                error: e.to_string(),
            });
            continue;
        }

        // Get existing link
        let existing = match storage.get(code).await {
            Ok(Some(link)) => link,
            Ok(None) => {
                failed.push(BatchFailedItem {
                    code: code.clone(),
                    error: "Link not found".to_string(),
                });
                continue;
            }
            Err(e) => {
                failed.push(BatchFailedItem {
                    code: code.clone(),
                    error: format!("Database query error: {}", e),
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

        // Hash password if provided
        let updated_password = match &payload.password {
            Some(pwd) if !pwd.is_empty() => {
                if is_argon2_hash(pwd) {
                    Some(pwd.clone())
                } else {
                    match hash_password(pwd) {
                        Ok(hash) => Some(hash),
                        Err(e) => {
                            failed.push(BatchFailedItem {
                                code: code.clone(),
                                error: format!("Failed to hash password: {}", e),
                            });
                            continue;
                        }
                    }
                }
            }
            Some(_) => None,                   // Empty string means remove password
            None => existing.password.clone(), // Not provided, keep existing
        };

        let updated_link = ShortLink {
            code: code.clone(),
            target: payload.target.clone(),
            created_at: existing.created_at,
            expires_at,
            password: updated_password,
            click: existing.click,
        };

        match storage.set(updated_link.clone()).await {
            Ok(_) => {
                success.push(code.clone());
                updated_links.push(updated_link);
            }
            Err(e) => {
                failed.push(BatchFailedItem {
                    code: code.clone(),
                    error: format!("Database error: {}", e),
                });
            }
        }
    }

    // 增量更新缓存
    for link in updated_links {
        let code = link.code.clone();
        let ttl = link.cache_ttl(crate::config::get_config().cache.default_ttl);
        cache.insert(&code, link, ttl).await;
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

    let (success, not_found) = storage
        .batch_remove(&batch.codes)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    let failed: Vec<BatchFailedItem> = not_found
        .into_iter()
        .map(|code| BatchFailedItem {
            code,
            error: "短链接不存在".to_string(),
        })
        .collect();

    // 批量清除缓存
    for code in &success {
        cache.remove(code).await;
    }

    info!(
        "Admin API: batch delete completed - {} success, {} failed",
        success.len(),
        failed.len()
    );

    Ok(success_response(BatchResponse { success, failed }))
}
