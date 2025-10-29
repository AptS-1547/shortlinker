use actix_web::http::StatusCode;
use actix_web::{HttpRequest, HttpResponse, Responder, Result as ActixResult, web};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::OnceLock;
use tracing::{error, info, trace};

use crate::cache::traits::CompositeCacheTrait;
use crate::storage::{SeaOrmStorage, ShortLink};
use crate::system::reload::reload_all;
use crate::utils::{TimeParser, generate_random_code};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LoginCredentials {
    pub password: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ApiResponse<T> {
    pub code: i32,
    pub data: T,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PostNewLink {
    pub code: Option<String>,
    pub target: String,
    pub expires_at: Option<String>,
    pub password: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GetLinksQuery {
    pub page: Option<usize>,
    pub page_size: Option<usize>,
    pub created_after: Option<String>,
    pub created_before: Option<String>,
    pub only_expired: Option<bool>,
    pub only_active: Option<bool>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PaginatedResponse<T> {
    pub code: i32,
    pub data: T,
    pub pagination: PaginationInfo,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PaginationInfo {
    pub page: usize,
    pub page_size: usize,
    pub total: usize,
    pub total_pages: usize,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LinkResponse {
    pub code: String,
    pub target: String,
    pub created_at: String,
    pub expires_at: Option<String>,
    pub password: Option<String>,
    pub click_count: usize,
}

impl From<ShortLink> for LinkResponse {
    fn from(link: ShortLink) -> Self {
        Self {
            code: link.code,
            target: link.target,
            created_at: link.created_at.to_rfc3339(),
            expires_at: link.expires_at.map(|dt| dt.to_rfc3339()),
            password: link.password,
            click_count: link.click,
        }
    }
}

static RANDOM_CODE_LENGTH: OnceLock<usize> = OnceLock::new();

pub struct AdminService;

impl AdminService {
    fn parse_expires_at(expire_str: &str) -> Result<chrono::DateTime<chrono::Utc>, String> {
        TimeParser::parse_expire_time(expire_str)
            .or_else(|_| {
                chrono::DateTime::parse_from_rfc3339(expire_str)
                    .map(|time| time.with_timezone(&chrono::Utc))
                    .map_err(|_| format!(
                        "Invalid expires_at format: {}. Use relative format (e.g., '1h', '30m') or RFC3339 format",
                        expire_str
                    ))
            })
    }

    fn json_response<T: Serialize>(status: StatusCode, code: i32, data: T) -> HttpResponse {
        HttpResponse::build(status)
            .append_header(("Content-Type", "application/json; charset=utf-8"))
            .json(ApiResponse { code, data })
    }

    fn success_response<T: Serialize>(data: T) -> HttpResponse {
        Self::json_response(StatusCode::OK, 0, data)
    }

    fn error_response(status: StatusCode, message: &str) -> HttpResponse {
        Self::json_response(status, 1, serde_json::json!({ "error": message }))
    }

    fn get_random_code_length() -> usize {
        *RANDOM_CODE_LENGTH.get_or_init(|| crate::config::get_config().features.random_code_length)
    }

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
        let mut filtered_links = Self::filter_links(all_links, &query, now);

        // Sort by creation time (newest first)
        filtered_links.sort_by(|a, b| b.1.created_at.cmp(&a.1.created_at));

        let total = filtered_links.len();
        let page = query.page.unwrap_or(1).max(1);
        let page_size = query.page_size.unwrap_or(20).clamp(1, 100);
        let total_pages = total.div_ceil(page_size);

        let paginated_links: Vec<LinkResponse> =
            Self::paginate_links(filtered_links, page, page_size)
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

        links
            .into_iter()
            .filter(|(_, link)| {
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

    pub async fn post_link(
        _req: HttpRequest,
        mut link: web::Json<PostNewLink>,
        cache: web::Data<Arc<dyn CompositeCacheTrait>>,
        storage: web::Data<Arc<SeaOrmStorage>>,
    ) -> ActixResult<impl Responder> {
        // Generate code if not provided
        if link.code.as_ref().is_none_or(|c| c.is_empty()) {
            trace!("Admin API: no code provided, generating a new one");
            let random_code = generate_random_code(Self::get_random_code_length());
            link.code = Some(random_code);
        } else {
            info!(
                "Admin API: using provided code: {}",
                link.code.as_ref().unwrap()
            );
        }

        let code = link.code.as_ref().unwrap();
        info!(
            "Admin API: create link request - code: {}, target: {}",
            code, link.target
        );

        // Parse expiration time
        let expires_at = match &link.expires_at {
            Some(expire_str) => match Self::parse_expires_at(expire_str) {
                Ok(time) => Some(time),
                Err(error_msg) => {
                    error!("Admin API: {}", error_msg);
                    return Ok(Self::error_response(StatusCode::BAD_REQUEST, &error_msg));
                }
            },
            None => None,
        };

        let new_link = ShortLink {
            code: code.clone(),
            target: link.target.clone(),
            created_at: chrono::Utc::now(),
            expires_at,
            password: link.password.clone(),
            click: 0,
        };

        match storage.set(new_link.clone()).await {
            Ok(_) => {
                info!("Admin API: link created - {}", new_link.code);
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
                        },
                    }))
            }
            Err(e) => {
                let error_msg = format!("Error creating link: {}", e);
                error!(
                    "Admin API: failed to create link - {}: {}",
                    new_link.code, e
                );
                Ok(Self::error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    &error_msg,
                ))
            }
        }
    }

    pub async fn get_link(
        _req: HttpRequest,
        code: web::Path<String>,
        storage: web::Data<Arc<SeaOrmStorage>>,
    ) -> ActixResult<impl Responder> {
        info!("Admin API: get link request - code: {}", code);

        match storage.get(&code).await {
            Some(link) => Ok(Self::success_response(LinkResponse::from(link))),
            None => {
                info!("Admin API: link not found - {}", code);
                Ok(Self::error_response(
                    StatusCode::NOT_FOUND,
                    "Link not found",
                ))
            }
        }
    }

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
                Ok(Self::success_response(serde_json::json!({
                    "message": "Link deleted successfully"
                })))
            }
            Err(e) => {
                let error_msg = format!("Error deleting link: {}", e);
                error!("Admin API: failed to delete link - {}: {}", code, e);
                Ok(Self::error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    &error_msg,
                ))
            }
        }
    }

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

        // Get existing link to preserve creation time
        let existing_link = match storage.get(&code).await {
            Some(link) => link,
            None => {
                info!("Admin API: attempt to update nonexistent link - {}", code);
                return Ok(Self::error_response(
                    StatusCode::NOT_FOUND,
                    "Link not found",
                ));
            }
        };

        // Parse expiration time
        let expires_at = match &link.expires_at {
            Some(expire_str) => match Self::parse_expires_at(expire_str) {
                Ok(time) => Some(time),
                Err(error_msg) => {
                    error!("Admin API: {}", error_msg);
                    return Ok(Self::error_response(StatusCode::BAD_REQUEST, &error_msg));
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
                Ok(Self::success_response(PostNewLink {
                    code: Some(updated_link.code),
                    target: updated_link.target,
                    expires_at: updated_link.expires_at.map(|dt| dt.to_rfc3339()),
                    password: updated_link.password,
                }))
            }
            Err(e) => {
                let error_msg = format!("Error updating link: {}", e);
                error!("Admin API: failed to update link - {}: {}", code, e);
                Ok(Self::error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    &error_msg,
                ))
            }
        }
    }

    pub async fn check_admin_token(
        _req: HttpRequest,
        login_body: web::Json<LoginCredentials>,
    ) -> ActixResult<impl Responder> {
        let config = crate::config::get_config();
        let admin_token = &config.api.admin_token;

        if login_body.password == *admin_token {
            info!("Admin API: login successful");
            Ok(Self::success_response(serde_json::json!({
                "message": "Login successful"
            })))
        } else {
            error!("Admin API: login failed - invalid token");
            Ok(Self::error_response(
                StatusCode::UNAUTHORIZED,
                "Invalid admin token",
            ))
        }
    }
}
