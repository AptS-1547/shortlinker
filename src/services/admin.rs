use actix_web::{web, HttpRequest, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use std::sync::OnceLock;
use tracing::{debug, error, info};

use crate::storages::{ShortLink, Storage};
use crate::utils::{generate_random_code, TimeParser};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SerializableShortLink {
    pub short_code: String,
    pub target_url: String,
    pub created_at: String,
    pub expires_at: Option<String>,
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

static RANDOM_CODE_LENGTH: OnceLock<usize> = OnceLock::new();

pub struct AdminService;

impl AdminService {
    pub async fn get_all_links(
        _req: HttpRequest,
        query: web::Query<GetLinksQuery>,
        storage: web::Data<Arc<dyn Storage>>,
    ) -> impl Responder {
        info!(
            "Admin API: request to list all links with filters: {:?}",
            query
        );

        let all_links = storage.load_all().await;
        info!("Admin API: retrieved {} total links", all_links.len());

        let now = chrono::Utc::now();

        // 过滤链接
        let mut filtered_links: Vec<(String, ShortLink)> = all_links
            .into_iter()
            .filter(|(_, link)| {
                // 时间过滤
                if let Some(created_after) = &query.created_after {
                    if let Ok(after_time) = chrono::DateTime::parse_from_rfc3339(created_after) {
                        if link.created_at < after_time.with_timezone(&chrono::Utc) {
                            return false;
                        }
                    }
                }

                if let Some(created_before) = &query.created_before {
                    if let Ok(before_time) = chrono::DateTime::parse_from_rfc3339(created_before) {
                        if link.created_at > before_time.with_timezone(&chrono::Utc) {
                            return false;
                        }
                    }
                }

                // 过期状态过滤
                let is_expired = link.expires_at.is_some_and(|exp| exp < now);

                if query.only_expired == Some(true) && !is_expired {
                    return false;
                }

                if query.only_active == Some(true) && is_expired {
                    return false;
                }

                true
            })
            .collect();

        // 按创建时间降序排序（最新的在前）
        filtered_links.sort_by(|a, b| b.1.created_at.cmp(&a.1.created_at));

        let total = filtered_links.len();
        let page = query.page.unwrap_or(1).max(1);
        let page_size = query.page_size.unwrap_or(20).clamp(1, 100);
        let total_pages = total.div_ceil(page_size);

        // 分页
        let start = (page - 1) * page_size;
        let end = (start + page_size).min(total);
        let paginated_links = if start < total {
            filtered_links[start..end].to_vec()
        } else {
            vec![]
        };

        let serializable_links: HashMap<String, SerializableShortLink> = paginated_links
            .into_iter()
            .map(|(code, link)| {
                let created_at = link.created_at.to_rfc3339();
                let expires_at = link.expires_at.map(|dt| dt.to_rfc3339());
                (
                    code.clone(),
                    SerializableShortLink {
                        short_code: code,
                        target_url: link.target,
                        created_at,
                        expires_at,
                    },
                )
            })
            .collect();

        info!(
            "Admin API: returning {} links (page {} of {}, total: {})",
            serializable_links.len(),
            page,
            total_pages,
            total
        );

        HttpResponse::Ok()
            .append_header(("Content-Type", "application/json; charset=utf-8"))
            .json(PaginatedResponse {
                code: 0,
                data: serializable_links,
                pagination: PaginationInfo {
                    page,
                    page_size,
                    total,
                    total_pages,
                },
            })
    }

    pub async fn post_link(
        _req: HttpRequest,
        mut link: web::Json<PostNewLink>,
        storage: web::Data<Arc<dyn Storage>>,
    ) -> impl Responder {
        // Check if a valid code is provided, otherwise generate one
        if link.code.is_none() || link.code.as_ref().unwrap().is_empty() {
            debug!("Admin API: no code provided, generating a new one");
            let random_code_length = *RANDOM_CODE_LENGTH.get_or_init(|| {
                env::var("RANDOM_CODE_LENGTH")
                    .unwrap_or_else(|_| "6".to_string())
                    .parse::<usize>()
                    .unwrap_or(6)
            });
            let random_code = generate_random_code(random_code_length);
            link.code = Some(random_code);
        } else {
            info!(
                "Admin API: using provided code: {}",
                link.code.as_ref().unwrap()
            );
        }

        info!(
            "Admin API: create link request - code: {}, target: {}",
            link.code.as_ref().unwrap_or(&"None".to_string()),
            link.target
        );

        let new_link = ShortLink {
            code: link.code.clone().unwrap(),
            target: link.target.clone(),
            created_at: chrono::Utc::now(),
            expires_at: link.expires_at.as_ref().map(|s| {
                TimeParser::parse_expire_time(s).unwrap_or_else(|_| {
                    // 如果解析失败，尝试 RFC3339 格式作为后备
                    chrono::DateTime::parse_from_rfc3339(s)
                        .unwrap()
                        .with_timezone(&chrono::Utc)
                })
            }),
        };

        match storage.set(new_link.clone()).await {
            Ok(_) => {
                info!("Admin API: link created - {}", new_link.code);
                HttpResponse::Created()
                    .append_header(("Content-Type", "application/json; charset=utf-8"))
                    .json(ApiResponse {
                        code: 0,
                        data: PostNewLink {
                            code: Some(new_link.code),
                            target: new_link.target,
                            expires_at: link.expires_at.clone(),
                        },
                    })
            }
            Err(e) => {
                error!(
                    "Admin API: failed to create link - {}: {}",
                    new_link.code, e
                );
                HttpResponse::InternalServerError()
                    .append_header(("Content-Type", "application/json; charset=utf-8"))
                    .json(ApiResponse {
                        code: 1,
                        data: serde_json::json!({
                            "error": format!("Error creating link: {}", e),
                        }),
                    })
            }
        }
    }

    pub async fn get_link(
        _req: HttpRequest,
        code: web::Path<String>,
        storage: web::Data<Arc<dyn Storage>>,
    ) -> impl Responder {
        info!("Admin API: get link request - code: {}", code);

        match storage.get(&code).await {
            Some(link) => {
                info!("Admin API: link retrieved - {}", code);
                let serializable_link = SerializableShortLink {
                    short_code: link.code.clone(),
                    target_url: link.target,
                    created_at: link.created_at.to_rfc3339(),
                    expires_at: link.expires_at.map(|dt| dt.to_rfc3339()),
                };
                HttpResponse::Ok()
                    .append_header(("Content-Type", "application/json; charset=utf-8"))
                    .json(ApiResponse {
                        code: 0,
                        data: serializable_link,
                    })
            }
            None => {
                info!("Admin API: link not found - {}", code);
                HttpResponse::NotFound()
                    .append_header(("Content-Type", "application/json; charset=utf-8"))
                    .json(ApiResponse {
                        code: 1,
                        data: serde_json::json!({ "error": "Link not found" }),
                    })
            }
        }
    }

    pub async fn delete_link(
        _req: HttpRequest,
        code: web::Path<String>,
        storage: web::Data<Arc<dyn Storage>>,
    ) -> impl Responder {
        info!("Admin API: delete link request - code: {}", code);

        match storage.remove(&code).await {
            Ok(_) => {
                info!("Admin API: link deleted - {}", code);
                HttpResponse::Ok()
                    .append_header(("Content-Type", "application/json; charset=utf-8"))
                    .json(ApiResponse {
                        code: 0,
                        data: serde_json::json!({ "message": "Link deleted successfully" }),
                    })
            }
            Err(e) => {
                error!("Admin API: failed to delete link - {}: {}", code, e);
                HttpResponse::InternalServerError()
                    .append_header(("Content-Type", "application/json; charset=utf-8"))
                    .json(ApiResponse {
                        code: 1,
                        data: serde_json::json!({ "error": format!("Error deleting link: {}", e) }),
                    })
            }
        }
    }

    pub async fn update_link(
        _req: HttpRequest,
        code: web::Path<String>,
        link: web::Json<PostNewLink>,
        storage: web::Data<Arc<dyn Storage>>,
    ) -> impl Responder {
        info!(
            "Admin API: update link request - code: {}, target: {}",
            code, link.target
        );

        // 先获取现有链接以保持创建时间
        let existing_link = match storage.get(&code).await {
            Some(link) => link,
            None => {
                info!("Admin API: attempt to update nonexistent link - {}", code);
                return HttpResponse::NotFound()
                    .append_header(("Content-Type", "application/json; charset=utf-8"))
                    .json(ApiResponse {
                        code: 1,
                        data: serde_json::json!({ "error": "Link not found" }),
                    });
            }
        };

        let updated_link = ShortLink {
            code: code.clone(),
            target: link.target.clone(),
            created_at: existing_link.created_at, // 保持原有的创建时间
            expires_at: link
                .expires_at
                .as_ref()
                .map(|s| {
                    TimeParser::parse_expire_time(s).unwrap_or_else(|_| {
                        // 如果解析失败，尝试 RFC3339 格式作为后备
                        chrono::DateTime::parse_from_rfc3339(s)
                            .unwrap()
                            .with_timezone(&chrono::Utc)
                    })
                })
                .or(existing_link.expires_at), // 如果没有提供新的过期时间，保持原有的
        };

        match storage.set(updated_link.clone()).await {
            Ok(_) => {
                info!("Admin API: link updated - {}", code);
                HttpResponse::Ok()
                    .append_header(("Content-Type", "application/json; charset=utf-8"))
                    .json(ApiResponse {
                        code: 0,
                        data: PostNewLink {
                            code: Some(updated_link.code),
                            target: updated_link.target,
                            expires_at: updated_link.expires_at.map(|dt| dt.to_rfc3339()),
                        },
                    })
            }
            Err(e) => {
                error!("Admin API: failed to update link - {}: {}", code, e);
                HttpResponse::InternalServerError()
                    .append_header(("Content-Type", "application/json; charset=utf-8"))
                    .json(ApiResponse {
                        code: 1,
                        data: serde_json::json!({
                            "error": format!("Error updating link: {}", e),
                        }),
                    })
            }
        }
    }
}
