//! Admin API 批量操作
//!
//! 使用 LinkService 统一业务逻辑层

use actix_web::{HttpRequest, Responder, Result as ActixResult, web};
use std::sync::Arc;
use tracing::info;

use crate::services::{CreateLinkRequest, LinkService, UpdateLinkRequest};

use super::error_code::ErrorCode;
use super::helpers::{error_response, success_response};
use super::types::{
    BatchCreateRequest, BatchDeleteRequest, BatchFailedItem, BatchResponse, BatchUpdateRequest,
};

/// 批量操作最大条目数
const MAX_BATCH_SIZE: usize = 5000;

/// 批量创建链接
pub async fn batch_create_links(
    _req: HttpRequest,
    batch: web::Json<BatchCreateRequest>,
    service: web::Data<Arc<LinkService>>,
) -> ActixResult<impl Responder> {
    // 检查批量大小限制
    if batch.links.len() > MAX_BATCH_SIZE {
        return Ok(error_response(
            actix_web::http::StatusCode::BAD_REQUEST,
            ErrorCode::BatchSizeTooLarge,
            &format!(
                "Batch size {} exceeds maximum {}",
                batch.links.len(),
                MAX_BATCH_SIZE
            ),
        ));
    }

    info!(
        "Admin API: batch create request - {} links",
        batch.links.len()
    );

    // 转换为 LinkService 请求格式
    let requests: Vec<CreateLinkRequest> = batch
        .links
        .iter()
        .map(|l| CreateLinkRequest {
            code: l.code.clone(),
            target: l.target.clone(),
            force: l.force.unwrap_or(false),
            expires_at: l.expires_at.clone(),
            password: l.password.clone(),
        })
        .collect();

    // 调用 LinkService 批量创建
    let result = service
        .batch_create_links(requests)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    // 转换为 API 响应格式
    let success: Vec<String> = result.success.iter().map(|s| s.code.clone()).collect();
    let failed: Vec<BatchFailedItem> = result
        .failed
        .into_iter()
        .map(|f| BatchFailedItem {
            code: f.code,
            error: f.reason,
            error_code: None,
        })
        .collect();

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
    service: web::Data<Arc<LinkService>>,
) -> ActixResult<impl Responder> {
    // 检查批量大小限制
    if batch.updates.len() > MAX_BATCH_SIZE {
        return Ok(error_response(
            actix_web::http::StatusCode::BAD_REQUEST,
            ErrorCode::BatchSizeTooLarge,
            &format!(
                "Batch size {} exceeds maximum {}",
                batch.updates.len(),
                MAX_BATCH_SIZE
            ),
        ));
    }

    info!(
        "Admin API: batch update request - {} links",
        batch.updates.len()
    );

    // 转换为 LinkService 请求格式
    let updates: Vec<(String, UpdateLinkRequest)> = batch
        .updates
        .iter()
        .map(|u| {
            (
                u.code.clone(),
                UpdateLinkRequest {
                    target: u.payload.target.clone(),
                    expires_at: u.payload.expires_at.clone(),
                    password: u.payload.password.clone(),
                },
            )
        })
        .collect();

    // 调用 LinkService 批量更新
    let result = service
        .batch_update_links(updates)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    // 转换为 API 响应格式
    let success: Vec<String> = result.success.iter().map(|s| s.code.clone()).collect();
    let failed: Vec<BatchFailedItem> = result
        .failed
        .into_iter()
        .map(|f| BatchFailedItem {
            code: f.code,
            error: f.reason,
            error_code: None,
        })
        .collect();

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
    service: web::Data<Arc<LinkService>>,
) -> ActixResult<impl Responder> {
    // 检查批量大小限制
    if batch.codes.len() > MAX_BATCH_SIZE {
        return Ok(error_response(
            actix_web::http::StatusCode::BAD_REQUEST,
            ErrorCode::BatchSizeTooLarge,
            &format!(
                "Batch size {} exceeds maximum {}",
                batch.codes.len(),
                MAX_BATCH_SIZE
            ),
        ));
    }

    info!(
        "Admin API: batch delete request - {} links",
        batch.codes.len()
    );

    // 调用 LinkService 批量删除
    let result = service
        .batch_delete_links(batch.codes.clone())
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    // 转换为 API 响应格式
    let success = result.deleted;
    let mut failed: Vec<BatchFailedItem> = result
        .not_found
        .into_iter()
        .map(|code| BatchFailedItem {
            code,
            error: "Link not found".to_string(),
            error_code: Some(ErrorCode::LinkNotFound as i32),
        })
        .collect();

    // 添加其他错误
    failed.extend(result.errors.into_iter().map(|f| BatchFailedItem {
        code: f.code,
        error: f.reason,
        error_code: None,
    }));

    info!(
        "Admin API: batch delete completed - {} success, {} failed",
        success.len(),
        failed.len()
    );

    Ok(success_response(BatchResponse { success, failed }))
}
