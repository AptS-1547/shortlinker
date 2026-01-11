//! Admin API 链接 CRUD 操作

use actix_web::http::StatusCode;
use actix_web::{HttpRequest, HttpResponse, Responder, Result as ActixResult, web};
use std::sync::Arc;
use tracing::{error, info, trace, warn};

use crate::cache::traits::CompositeCacheTrait;
use crate::storage::{LinkFilter, SeaOrmStorage, ShortLink};
use crate::utils::generate_random_code;
use crate::utils::password::{hash_password, is_argon2_hash};
use crate::utils::url_validator::validate_url;

use super::get_random_code_length;
use super::helpers::{error_response, parse_expires_at, success_response};
use super::types::{
    ApiResponse, GetLinksQuery, LinkResponse, PaginatedResponse, PaginationInfo, PostNewLink,
    StatsResponse,
};

/// 获取所有链接（支持分页和过滤）
pub async fn get_all_links(
    _req: HttpRequest,
    query: web::Query<GetLinksQuery>,
    storage: web::Data<Arc<SeaOrmStorage>>,
) -> ActixResult<impl Responder> {
    trace!(
        "Admin API: request to list all links with filters: {:?}",
        query
    );

    let page = query.page.unwrap_or(1).max(1) as u64;
    let page_size = query.page_size.unwrap_or(20).clamp(1, 100) as u64;

    // 构建过滤条件
    let filter = LinkFilter {
        search: query.search.clone(),
        created_after: query
            .created_after
            .as_ref()
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&chrono::Utc)),
        created_before: query
            .created_before
            .as_ref()
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&chrono::Utc)),
        only_expired: query.only_expired.unwrap_or(false),
        only_active: query.only_active.unwrap_or(false),
    };

    // 使用数据库分页
    let (links, total) = storage
        .load_paginated_filtered(page, page_size, filter)
        .await;
    let total = total as usize;
    let page = page as usize;
    let page_size = page_size as usize;
    let total_pages = total.div_ceil(page_size);

    let paginated_links: Vec<LinkResponse> = links.into_iter().map(LinkResponse::from).collect();

    info!(
        "Admin API: returning {} links (page {} of {}, total: {})",
        paginated_links.len(),
        page,
        total_pages,
        total
    );

    Ok(HttpResponse::Ok()
        .append_header(("Content-Type", "application/json; charset=utf-8"))
        .json(PaginatedResponse {
            code: 0,
            data: paginated_links,
            pagination: PaginationInfo {
                page,
                page_size,
                total,
                total_pages,
            },
        }))
}

/// 创建新链接
pub async fn post_link(
    _req: HttpRequest,
    mut link: web::Json<PostNewLink>,
    cache: web::Data<Arc<dyn CompositeCacheTrait>>,
    storage: web::Data<Arc<SeaOrmStorage>>,
) -> ActixResult<impl Responder> {
    // Generate code if not provided, or use the provided one
    let code = match link.code.as_ref().filter(|c| !c.is_empty()) {
        Some(provided_code) => {
            info!("Admin API: using provided code: {}", provided_code);
            provided_code.clone()
        }
        None => {
            trace!("Admin API: no code provided, generating a new one");
            let random_code = generate_random_code(get_random_code_length());
            link.code = Some(random_code.clone());
            random_code
        }
    };
    info!(
        "Admin API: create link request - code: {}, target: {}",
        code, link.target
    );

    // Validate target URL
    if let Err(e) = validate_url(&link.target) {
        error!("Admin API: invalid target URL - {}: {}", link.target, e);
        return Ok(error_response(StatusCode::BAD_REQUEST, &e.to_string()));
    }

    // Check if link already exists
    let existing_link = storage.get(&code).await;
    let force = link.force.unwrap_or(false);

    if existing_link.is_some() && !force {
        warn!("Admin API: link already exists - {}", code);
        return Ok(error_response(
            StatusCode::CONFLICT,
            "Link already exists. Use force=true to overwrite.",
        ));
    }

    // Parse expiration time
    let expires_at = match &link.expires_at {
        Some(expire_str) => match parse_expires_at(expire_str) {
            Ok(time) => Some(time),
            Err(error_msg) => {
                error!("Admin API: {}", error_msg);
                return Ok(error_response(StatusCode::BAD_REQUEST, &error_msg));
            }
        },
        None => None,
    };

    // If force overwriting, preserve original created_at and click count
    let (created_at, click) = if let Some(ref existing) = existing_link {
        (existing.created_at, existing.click)
    } else {
        (chrono::Utc::now(), 0)
    };

    // Hash password if provided and not already hashed
    let hashed_password = match &link.password {
        Some(pwd) if !pwd.is_empty() => {
            if is_argon2_hash(pwd) {
                // Already hashed, use as-is
                Some(pwd.clone())
            } else {
                // Hash the plaintext password
                match hash_password(pwd) {
                    Ok(hash) => Some(hash),
                    Err(e) => {
                        error!("Admin API: failed to hash password: {}", e);
                        return Ok(error_response(
                            StatusCode::INTERNAL_SERVER_ERROR,
                            "Failed to process password",
                        ));
                    }
                }
            }
        }
        _ => None,
    };

    let new_link = ShortLink {
        code: code.clone(),
        target: link.target.clone(),
        created_at,
        expires_at,
        password: hashed_password.clone(),
        click,
    };

    match storage.set(new_link.clone()).await {
        Ok(_) => {
            let action = if existing_link.is_some() {
                "overwritten"
            } else {
                "created"
            };
            info!("Admin API: link {} - {}", action, new_link.code);
            // 增量更新缓存
            let ttl = new_link.cache_ttl(crate::config::get_config().cache.default_ttl);
            cache.insert(&new_link.code, new_link.clone(), ttl).await;
            Ok(HttpResponse::Created()
                .append_header(("Content-Type", "application/json; charset=utf-8"))
                .json(ApiResponse {
                    code: 0,
                    data: PostNewLink {
                        code: Some(new_link.code),
                        target: new_link.target,
                        expires_at: link.expires_at.clone(),
                        password: new_link.password,
                        force: None,
                    },
                }))
        }
        Err(e) => {
            let error_msg = format!("Error creating link: {}", e);
            error!(
                "Admin API: failed to create link - {}: {}",
                new_link.code, e
            );
            Ok(error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                &error_msg,
            ))
        }
    }
}

/// 获取单个链接
pub async fn get_link(
    _req: HttpRequest,
    code: web::Path<String>,
    storage: web::Data<Arc<SeaOrmStorage>>,
) -> ActixResult<impl Responder> {
    info!("Admin API: get link request - code: {}", code);

    match storage.get(&code).await {
        Some(link) => Ok(success_response(LinkResponse::from(link))),
        None => {
            info!("Admin API: link not found - {}", code);
            Ok(error_response(StatusCode::NOT_FOUND, "Link not found"))
        }
    }
}

/// 删除链接
pub async fn delete_link(
    _req: HttpRequest,
    code: web::Path<String>,
    cache: web::Data<Arc<dyn CompositeCacheTrait>>,
    storage: web::Data<Arc<SeaOrmStorage>>,
) -> ActixResult<impl Responder> {
    info!("Admin API: delete link request - code: {}", code);

    match storage.remove(&code).await {
        Ok(_) => {
            info!("Admin API: link deleted - {}", code);
            // 增量更新缓存
            cache.remove(&code).await;
            Ok(success_response(serde_json::json!({
                "message": "Link deleted successfully"
            })))
        }
        Err(e) => {
            let error_msg = format!("Error deleting link: {}", e);
            error!("Admin API: failed to delete link - {}: {}", code, e);
            Ok(error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                &error_msg,
            ))
        }
    }
}

/// 更新链接
pub async fn update_link(
    _req: HttpRequest,
    code: web::Path<String>,
    link: web::Json<PostNewLink>,
    cache: web::Data<Arc<dyn CompositeCacheTrait>>,
    storage: web::Data<Arc<SeaOrmStorage>>,
) -> ActixResult<impl Responder> {
    info!(
        "Admin API: update link request - code: {}, target: {}",
        code, link.target
    );

    // Validate target URL
    if let Err(e) = validate_url(&link.target) {
        error!("Admin API: invalid target URL - {}: {}", link.target, e);
        return Ok(error_response(StatusCode::BAD_REQUEST, &e.to_string()));
    }

    // Get existing link to preserve creation time
    let existing_link = match storage.get(&code).await {
        Some(link) => link,
        None => {
            info!("Admin API: attempt to update nonexistent link - {}", code);
            return Ok(error_response(StatusCode::NOT_FOUND, "Link not found"));
        }
    };

    // Parse expiration time
    let expires_at = match &link.expires_at {
        Some(expire_str) => match parse_expires_at(expire_str) {
            Ok(time) => Some(time),
            Err(error_msg) => {
                error!("Admin API: {}", error_msg);
                return Ok(error_response(StatusCode::BAD_REQUEST, &error_msg));
            }
        },
        None => existing_link.expires_at,
    };

    // Hash password if provided and not already hashed
    let updated_password = match &link.password {
        Some(pwd) if !pwd.is_empty() => {
            if is_argon2_hash(pwd) {
                // Already hashed, use as-is
                Some(pwd.clone())
            } else {
                // Hash the plaintext password
                match hash_password(pwd) {
                    Ok(hash) => Some(hash),
                    Err(e) => {
                        error!("Admin API: failed to hash password: {}", e);
                        return Ok(error_response(
                            StatusCode::INTERNAL_SERVER_ERROR,
                            "Failed to process password",
                        ));
                    }
                }
            }
        }
        Some(_) => None,                        // Empty string means remove password
        None => existing_link.password.clone(), // Not provided, keep existing
    };

    let updated_link = ShortLink {
        code: code.clone(),
        target: link.target.clone(),
        created_at: existing_link.created_at,
        expires_at,
        password: updated_password.clone(),
        click: existing_link.click, // 保持原有的点击计数
    };

    match storage.set(updated_link.clone()).await {
        Ok(_) => {
            info!("Admin API: link updated - {}", code);
            // 增量更新缓存
            let ttl = updated_link.cache_ttl(crate::config::get_config().cache.default_ttl);
            cache
                .insert(&updated_link.code, updated_link.clone(), ttl)
                .await;
            Ok(success_response(PostNewLink {
                code: Some(updated_link.code),
                target: updated_link.target,
                expires_at: updated_link.expires_at.map(|dt| dt.to_rfc3339()),
                password: updated_link.password,
                force: None,
            }))
        }
        Err(e) => {
            let error_msg = format!("Error updating link: {}", e);
            error!("Admin API: failed to update link - {}: {}", code, e);
            Ok(error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                &error_msg,
            ))
        }
    }
}

/// 获取链接统计信息
pub async fn get_stats(
    _req: HttpRequest,
    storage: web::Data<Arc<SeaOrmStorage>>,
) -> ActixResult<impl Responder> {
    trace!("Admin API: request to get link stats");

    let stats = storage.get_stats().await;

    info!(
        "Admin API: returning stats - total: {}, clicks: {}, active: {}",
        stats.total_links, stats.total_clicks, stats.active_links
    );

    Ok(HttpResponse::Ok()
        .append_header(("Content-Type", "application/json; charset=utf-8"))
        .json(ApiResponse {
            code: 0,
            data: StatsResponse {
                total_links: stats.total_links,
                total_clicks: stats.total_clicks,
                active_links: stats.active_links,
            },
        }))
}
