//! Admin API 链接 CRUD 操作

use actix_web::http::StatusCode;
use actix_web::{HttpRequest, HttpResponse, Responder, Result as ActixResult, web};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{error, info, trace, warn};

use crate::cache::traits::CompositeCacheTrait;
use crate::storage::{SeaOrmStorage, ShortLink};
use crate::system::reload::reload_all;
use crate::utils::generate_random_code;

use super::helpers::{error_response, parse_expires_at, success_response};
use super::types::{
    ApiResponse, GetLinksQuery, LinkResponse, PaginatedResponse, PaginationInfo, PostNewLink,
};
use super::get_random_code_length;

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

    let all_links = storage.load_all().await;
    trace!("Admin API: retrieved {} total links", all_links.len());

    let now = chrono::Utc::now();
    let mut filtered_links = filter_links(all_links, &query, now);

    // Sort by creation time (newest first)
    filtered_links.sort_by(|a, b| b.1.created_at.cmp(&a.1.created_at));

    let total = filtered_links.len();
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(20).clamp(1, 100);
    let total_pages = total.div_ceil(page_size);

    let paginated_links: Vec<LinkResponse> = paginate_links(filtered_links, page, page_size)
        .into_iter()
        .map(|(_, link)| LinkResponse::from(link))
        .collect();

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

fn filter_links(
    links: HashMap<String, ShortLink>,
    query: &GetLinksQuery,
    now: chrono::DateTime<chrono::Utc>,
) -> Vec<(String, ShortLink)> {
    // Parse time filters once outside the iteration
    let after_time = query
        .created_after
        .as_ref()
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&chrono::Utc));

    let before_time = query
        .created_before
        .as_ref()
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&chrono::Utc));

    // Prepare search query (case-insensitive)
    let search_lower = query.search.as_ref().map(|s| s.to_lowercase());

    links
        .into_iter()
        .filter(|(_, link)| {
            // Search filter (matches code or target, case-insensitive)
            if let Some(ref search) = search_lower {
                let code_matches = link.code.to_lowercase().contains(search);
                let target_matches = link.target.to_lowercase().contains(search);
                if !code_matches && !target_matches {
                    return false;
                }
            }

            // Time filters
            if let Some(after) = after_time
                && link.created_at < after
            {
                return false;
            }

            if let Some(before) = before_time
                && link.created_at > before
            {
                return false;
            }

            // Expiration status filter
            let is_expired = link.expires_at.is_some_and(|exp| exp < now);

            if query.only_expired == Some(true) && !is_expired {
                return false;
            }

            if query.only_active == Some(true) && is_expired {
                return false;
            }

            true
        })
        .collect()
}

fn paginate_links(
    mut links: Vec<(String, ShortLink)>,
    page: usize,
    page_size: usize,
) -> Vec<(String, ShortLink)> {
    let start = (page - 1) * page_size;
    let end = (start + page_size).min(links.len());

    if start < links.len() {
        links.drain(start..end).collect()
    } else {
        Vec::new()
    }
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
    if !link.target.starts_with("http://") && !link.target.starts_with("https://") {
        error!("Admin API: invalid target URL - {}", link.target);
        return Ok(error_response(
            StatusCode::BAD_REQUEST,
            "URL must start with http:// or https://",
        ));
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

    let new_link = ShortLink {
        code: code.clone(),
        target: link.target.clone(),
        created_at,
        expires_at,
        password: link.password.clone(),
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
            let _ = reload_all(cache.get_ref().clone(), storage.get_ref().clone()).await;
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
            Ok(error_response(StatusCode::INTERNAL_SERVER_ERROR, &error_msg))
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
            let _ = reload_all(cache.get_ref().clone(), storage.get_ref().clone()).await;
            Ok(success_response(serde_json::json!({
                "message": "Link deleted successfully"
            })))
        }
        Err(e) => {
            let error_msg = format!("Error deleting link: {}", e);
            error!("Admin API: failed to delete link - {}: {}", code, e);
            Ok(error_response(StatusCode::INTERNAL_SERVER_ERROR, &error_msg))
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
    if !link.target.starts_with("http://") && !link.target.starts_with("https://") {
        error!("Admin API: invalid target URL - {}", link.target);
        return Ok(error_response(
            StatusCode::BAD_REQUEST,
            "URL must start with http:// or https://",
        ));
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

    let updated_link = ShortLink {
        code: code.clone(),
        target: link.target.clone(),
        created_at: existing_link.created_at,
        expires_at,
        // 如果请求中提供了密码字段，则使用新密码；否则保持原密码
        password: if link.password.is_some() {
            link.password.clone()
        } else {
            existing_link.password
        },
        click: existing_link.click, // 保持原有的点击计数
    };

    match storage.set(updated_link.clone()).await {
        Ok(_) => {
            info!("Admin API: link updated - {}", code);
            let _ = reload_all(cache.get_ref().clone(), storage.get_ref().clone()).await;
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
            Ok(error_response(StatusCode::INTERNAL_SERVER_ERROR, &error_msg))
        }
    }
}
