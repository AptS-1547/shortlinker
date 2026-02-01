//! Admin API 批量操作

use actix_web::{HttpRequest, Responder, Result as ActixResult, web};
use std::sync::Arc;
use tracing::info;

use crate::cache::traits::CompositeCacheTrait;
use crate::storage::{SeaOrmStorage, ShortLink};
use crate::utils::generate_random_code;
use crate::utils::password::{process_new_password, process_update_password};
use crate::utils::url_validator::validate_url;

use super::error_code::ErrorCode;
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

    // 第一步：预处理所有请求，收集 codes 并验证 URL
    struct ValidatedRequest {
        code: String,
        target: String,
        expires_at_str: Option<String>,
        password: Option<String>,
        force: bool,
    }

    let mut codes_to_check: Vec<String> = Vec::new();
    let mut valid_requests: Vec<ValidatedRequest> = Vec::new();

    for link_req in &batch.links {
        // 生成或使用提供的 code
        let code = match &link_req.code {
            Some(c) if !c.is_empty() => c.clone(),
            _ => generate_random_code(get_random_code_length()),
        };

        // 验证目标 URL
        if let Err(e) = validate_url(&link_req.target) {
            failed.push(BatchFailedItem {
                code,
                error: e.to_string(),
                error_code: Some(ErrorCode::LinkInvalidUrl as i32),
            });
            continue;
        }

        codes_to_check.push(code.clone());
        valid_requests.push(ValidatedRequest {
            code,
            target: link_req.target.clone(),
            expires_at_str: link_req.expires_at.clone(),
            password: link_req.password.clone(),
            force: link_req.force.unwrap_or(false),
        });
    }

    // 第二步：批量查询现有链接（1 次 DB 查询替代 N 次）
    let codes_refs: Vec<&str> = codes_to_check.iter().map(|s| s.as_str()).collect();
    let existing_map = storage
        .batch_get(&codes_refs)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    // 第三步：内存中验证 + 构建待插入列表
    let mut links_to_insert: Vec<ShortLink> = Vec::new();

    for req in valid_requests {
        let existing = existing_map.get(&req.code);

        // 检查是否允许覆盖
        if existing.is_some() && !req.force {
            failed.push(BatchFailedItem {
                code: req.code,
                error: "Link already exists".to_string(),
                error_code: Some(ErrorCode::LinkAlreadyExists as i32),
            });
            continue;
        }

        // 解析过期时间
        let expires_at = match &req.expires_at_str {
            Some(expire_str) => match parse_expires_at(expire_str) {
                Ok(time) => Some(time),
                Err(error_msg) => {
                    failed.push(BatchFailedItem {
                        code: req.code,
                        error: error_msg,
                        error_code: Some(ErrorCode::LinkInvalidExpireTime as i32),
                    });
                    continue;
                }
            },
            None => None,
        };

        // 保留原始数据（如果强制覆盖）
        let (created_at, click) = if let Some(ex) = existing {
            (ex.created_at, ex.click)
        } else {
            (chrono::Utc::now(), 0)
        };

        // 处理密码哈希
        let hashed_password = match process_new_password(req.password.as_deref()) {
            Ok(pwd) => pwd,
            Err(e) => {
                failed.push(BatchFailedItem {
                    code: req.code,
                    error: format!("Failed to hash password: {}", e),
                    error_code: Some(ErrorCode::LinkPasswordHashError as i32),
                });
                continue;
            }
        };

        let new_link = ShortLink {
            code: req.code.clone(),
            target: req.target,
            created_at,
            expires_at,
            password: hashed_password,
            click,
        };

        success.push(req.code);
        links_to_insert.push(new_link);
    }

    // 第四步：批量插入（1 次 DB 事务替代 N 次单独写入）
    if !links_to_insert.is_empty()
        && let Err(e) = storage.batch_set(links_to_insert.clone()).await
    {
        // 批量写入失败，将所有成功标记移回失败
        for code in success.drain(..) {
            failed.push(BatchFailedItem {
                code,
                error: format!("Database error: {}", e),
                error_code: Some(ErrorCode::LinkDatabaseError as i32),
            });
        }
    }

    // 第五步：增量更新缓存
    for link in &links_to_insert {
        if success.contains(&link.code) {
            let ttl = link.cache_ttl(crate::config::get_config().cache.default_ttl);
            cache.insert(&link.code, link.clone(), ttl).await;
        }
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

    // 第一步：预处理所有请求，收集 codes 并验证 URL
    struct ValidatedUpdate<'a> {
        code: String,
        target: String,
        expires_at_str: Option<&'a String>,
        password: Option<&'a String>,
    }

    let mut codes_to_check: Vec<String> = Vec::new();
    let mut valid_updates: Vec<ValidatedUpdate> = Vec::new();

    for update in &batch.updates {
        let code = &update.code;
        let payload = &update.payload;

        // 验证目标 URL
        if let Err(e) = validate_url(&payload.target) {
            failed.push(BatchFailedItem {
                code: code.clone(),
                error: e.to_string(),
                error_code: Some(ErrorCode::LinkInvalidUrl as i32),
            });
            continue;
        }

        codes_to_check.push(code.clone());
        valid_updates.push(ValidatedUpdate {
            code: code.clone(),
            target: payload.target.clone(),
            expires_at_str: payload.expires_at.as_ref(),
            password: payload.password.as_ref(),
        });
    }

    // 第二步：批量查询现有链接（1 次 DB 查询替代 N 次）
    let codes_refs: Vec<&str> = codes_to_check.iter().map(|s| s.as_str()).collect();
    let existing_map = storage
        .batch_get(&codes_refs)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    // 第三步：内存中验证 + 构建待更新列表
    let mut links_to_update: Vec<ShortLink> = Vec::new();

    for update in valid_updates {
        // 检查链接是否存在
        let existing = match existing_map.get(&update.code) {
            Some(link) => link,
            None => {
                failed.push(BatchFailedItem {
                    code: update.code,
                    error: "Link not found".to_string(),
                    error_code: Some(ErrorCode::LinkNotFound as i32),
                });
                continue;
            }
        };

        // 解析过期时间
        let expires_at = match update.expires_at_str {
            Some(expire_str) => match parse_expires_at(expire_str) {
                Ok(time) => Some(time),
                Err(error_msg) => {
                    failed.push(BatchFailedItem {
                        code: update.code,
                        error: error_msg,
                        error_code: Some(ErrorCode::LinkInvalidExpireTime as i32),
                    });
                    continue;
                }
            },
            None => existing.expires_at,
        };

        // 处理密码
        let updated_password = match process_update_password(
            update.password.map(|s| s.as_str()),
            existing.password.clone(),
        ) {
            Ok(pwd) => pwd,
            Err(e) => {
                failed.push(BatchFailedItem {
                    code: update.code,
                    error: format!("Failed to hash password: {}", e),
                    error_code: Some(ErrorCode::LinkPasswordHashError as i32),
                });
                continue;
            }
        };

        let updated_link = ShortLink {
            code: update.code.clone(),
            target: update.target,
            created_at: existing.created_at,
            expires_at,
            password: updated_password,
            click: existing.click,
        };

        success.push(update.code);
        links_to_update.push(updated_link);
    }

    // 第四步：批量更新（1 次 DB 事务替代 N 次单独写入）
    if !links_to_update.is_empty()
        && let Err(e) = storage.batch_set(links_to_update.clone()).await
    {
        // 批量写入失败，将所有成功标记移回失败
        for code in success.drain(..) {
            failed.push(BatchFailedItem {
                code,
                error: format!("Database error: {}", e),
                error_code: Some(ErrorCode::LinkDatabaseError as i32),
            });
        }
    }

    // 第五步：增量更新缓存
    for link in &links_to_update {
        if success.contains(&link.code) {
            let ttl = link.cache_ttl(crate::config::get_config().cache.default_ttl);
            cache.insert(&link.code, link.clone(), ttl).await;
        }
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
            error: "Link not found".to_string(),
            error_code: Some(ErrorCode::LinkNotFound as i32),
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
