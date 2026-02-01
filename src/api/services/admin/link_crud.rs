//! Admin API 链接 CRUD 操作
//!
//! Uses LinkService for unified business logic.

use actix_web::http::StatusCode;
use actix_web::{HttpRequest, HttpResponse, Responder, Result as ActixResult, web};
use std::sync::Arc;
use tracing::{info, trace};

use crate::services::{CreateLinkRequest, LinkService, ServiceError, UpdateLinkRequest};
use crate::storage::LinkFilter;

use super::error_code::ErrorCode;
use super::helpers::{error_response, success_response};
use super::types::{
    ApiResponse, GetLinksQuery, LinkResponse, MessageResponse, PaginatedResponse, PaginationInfo,
    PostNewLink, StatsResponse,
};

/// Convert ServiceError to HTTP response
fn service_error_response(err: ServiceError) -> HttpResponse {
    let (status, error_code) = match &err {
        ServiceError::InvalidUrl(_) => (StatusCode::BAD_REQUEST, ErrorCode::LinkInvalidUrl),
        ServiceError::InvalidExpireTime(_) => (StatusCode::BAD_REQUEST, ErrorCode::LinkInvalidExpireTime),
        ServiceError::PasswordHashError => (StatusCode::INTERNAL_SERVER_ERROR, ErrorCode::LinkPasswordHashError),
        ServiceError::DatabaseError(_) => (StatusCode::INTERNAL_SERVER_ERROR, ErrorCode::LinkDatabaseError),
        ServiceError::NotFound(_) => (StatusCode::NOT_FOUND, ErrorCode::LinkNotFound),
        ServiceError::Conflict(_) => (StatusCode::CONFLICT, ErrorCode::LinkAlreadyExists),
        ServiceError::NotInitialized => (StatusCode::SERVICE_UNAVAILABLE, ErrorCode::ServiceUnavailable),
    };
    error_response(status, error_code, &err.to_string())
}

/// 获取所有链接（支持分页和过滤）
pub async fn get_all_links(
    _req: HttpRequest,
    query: web::Query<GetLinksQuery>,
    service: web::Data<Arc<LinkService>>,
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

    match service.list_links(filter, page, page_size).await {
        Ok((links, total)) => {
            let total = total as usize;
            let page = page as usize;
            let page_size = page_size as usize;
            let total_pages = total.div_ceil(page_size);

            let paginated_links: Vec<LinkResponse> =
                links.into_iter().map(LinkResponse::from).collect();

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
                    code: ErrorCode::Success as i32,
                    message: "OK".to_string(),
                    data: Some(paginated_links),
                    pagination: PaginationInfo {
                        page,
                        page_size,
                        total,
                        total_pages,
                    },
                }))
        }
        Err(e) => Ok(service_error_response(e)),
    }
}

/// 创建新链接
pub async fn post_link(
    _req: HttpRequest,
    link: web::Json<PostNewLink>,
    service: web::Data<Arc<LinkService>>,
) -> ActixResult<impl Responder> {
    info!(
        "Admin API: create link request - code: {:?}, target: {}",
        link.code, link.target
    );

    let req = CreateLinkRequest {
        code: link.code.clone(),
        target: link.target.clone(),
        force: link.force.unwrap_or(false),
        expires_at: link.expires_at.clone(),
        password: link.password.clone(),
    };

    match service.create_link(req).await {
        Ok(result) => {
            let action = if result.generated_code {
                "created with generated code"
            } else {
                "created"
            };
            info!("Admin API: link {} - {}", action, result.link.code);

            Ok(HttpResponse::Created()
                .append_header(("Content-Type", "application/json; charset=utf-8"))
                .json(ApiResponse {
                    code: ErrorCode::Success as i32,
                    message: "Link created".to_string(),
                    data: Some(PostNewLink {
                        code: Some(result.link.code),
                        target: result.link.target,
                        expires_at: link.expires_at.clone(),
                        password: result.link.password,
                        force: None,
                    }),
                }))
        }
        Err(e) => Ok(service_error_response(e)),
    }
}

/// 获取单个链接
pub async fn get_link(
    _req: HttpRequest,
    code: web::Path<String>,
    service: web::Data<Arc<LinkService>>,
) -> ActixResult<impl Responder> {
    info!("Admin API: get link request - code: {}", code);

    match service.get_link(&code).await {
        Ok(Some(link)) => Ok(success_response(LinkResponse::from(link))),
        Ok(None) => {
            info!("Admin API: link not found - {}", code);
            Ok(error_response(StatusCode::NOT_FOUND, ErrorCode::LinkNotFound, "Link not found"))
        }
        Err(e) => Ok(service_error_response(e)),
    }
}

/// 删除链接
pub async fn delete_link(
    _req: HttpRequest,
    code: web::Path<String>,
    service: web::Data<Arc<LinkService>>,
) -> ActixResult<impl Responder> {
    info!("Admin API: delete link request - code: {}", code);

    match service.delete_link(&code).await {
        Ok(()) => {
            info!("Admin API: link deleted - {}", code);
            Ok(success_response(MessageResponse {
                message: "Link deleted successfully".to_string(),
            }))
        }
        Err(e) => Ok(service_error_response(e)),
    }
}

/// 更新链接
pub async fn update_link(
    _req: HttpRequest,
    code: web::Path<String>,
    link: web::Json<PostNewLink>,
    service: web::Data<Arc<LinkService>>,
) -> ActixResult<impl Responder> {
    info!(
        "Admin API: update link request - code: {}, target: {}",
        code, link.target
    );

    let req = UpdateLinkRequest {
        target: link.target.clone(),
        expires_at: link.expires_at.clone(),
        password: link.password.clone(),
    };

    match service.update_link(&code, req).await {
        Ok(updated_link) => {
            info!("Admin API: link updated - {}", code);
            Ok(success_response(PostNewLink {
                code: Some(updated_link.code),
                target: updated_link.target,
                expires_at: updated_link.expires_at.map(|dt| dt.to_rfc3339()),
                password: updated_link.password,
                force: None,
            }))
        }
        Err(e) => Ok(service_error_response(e)),
    }
}

/// 获取链接统计信息
pub async fn get_stats(
    _req: HttpRequest,
    service: web::Data<Arc<LinkService>>,
) -> ActixResult<impl Responder> {
    trace!("Admin API: request to get link stats");

    match service.get_stats().await {
        Ok(stats) => {
            info!(
                "Admin API: returning stats - total: {}, clicks: {}, active: {}",
                stats.total_links, stats.total_clicks, stats.active_links
            );

            Ok(HttpResponse::Ok()
                .append_header(("Content-Type", "application/json; charset=utf-8"))
                .json(ApiResponse {
                    code: ErrorCode::Success as i32,
                    message: "OK".to_string(),
                    data: Some(StatsResponse {
                        total_links: stats.total_links,
                        total_clicks: stats.total_clicks,
                        active_links: stats.active_links,
                    }),
                }))
        }
        Err(e) => Ok(service_error_response(e)),
    }
}
