//! Admin API 导出导入操作

use actix_multipart::Multipart;
use actix_web::http::StatusCode;
use actix_web::{HttpRequest, HttpResponse, Responder, Result as ActixResult, web};
use chrono::Utc;
use csv::{ReaderBuilder, WriterBuilder};
use futures_util::StreamExt;
use std::collections::HashSet;
use std::io::Cursor;
use std::sync::Arc;
use tracing::{error, info};

use crate::cache::traits::CompositeCacheTrait;
use crate::storage::{LinkFilter, SeaOrmStorage, ShortLink};
use crate::utils::password::{hash_password, is_argon2_hash};
use crate::utils::url_validator::validate_url;

use super::helpers::{error_response, success_response};
use super::types::{CsvLinkRow, ExportQuery, ImportFailedItem, ImportMode, ImportResponse};

/// 导出链接为 CSV
pub async fn export_links(
    _req: HttpRequest,
    query: web::Query<ExportQuery>,
    storage: web::Data<Arc<SeaOrmStorage>>,
) -> ActixResult<impl Responder> {
    info!("Admin API: export links request with filters: {:?}", query);

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

    // 加载符合条件的所有链接
    let links = storage.load_all_filtered(filter).await.map_err(|e| {
        error!("Failed to load links for export: {}", e);
        actix_web::error::ErrorInternalServerError(format!("Database error: {}", e))
    })?;

    // 构建 CSV
    let mut csv_writer = WriterBuilder::new().from_writer(vec![]);

    for link in &links {
        let row = CsvLinkRow {
            code: link.code.clone(),
            target: link.target.clone(),
            created_at: link.created_at.to_rfc3339(),
            expires_at: link.expires_at.map(|dt| dt.to_rfc3339()),
            password: link.password.clone(),
            click_count: link.click,
        };
        if let Err(e) = csv_writer.serialize(&row) {
            error!("Failed to serialize CSV row: {}", e);
        }
    }

    let csv_data = csv_writer.into_inner().unwrap_or_default();

    // 生成文件名
    let filename = format!(
        "shortlinks_export_{}.csv",
        Utc::now().format("%Y%m%d_%H%M%S")
    );

    info!(
        "Admin API: exported {} links ({} bytes) to {}",
        links.len(),
        csv_data.len(),
        filename
    );

    Ok(HttpResponse::Ok()
        .content_type("text/csv; charset=utf-8")
        .insert_header((
            "Content-Disposition",
            format!("attachment; filename=\"{}\"", filename),
        ))
        .body(csv_data))
}

/// 导入链接从 CSV
pub async fn import_links(
    _req: HttpRequest,
    mut payload: Multipart,
    cache: web::Data<Arc<dyn CompositeCacheTrait>>,
    storage: web::Data<Arc<SeaOrmStorage>>,
) -> ActixResult<impl Responder> {
    info!("Admin API: import links request");

    let mut csv_data: Option<Vec<u8>> = None;
    let mut mode = ImportMode::Skip; // 默认模式

    // 解析 multipart form data
    while let Some(item) = payload.next().await {
        let mut field = match item {
            Ok(f) => f,
            Err(e) => {
                error!("Failed to parse multipart field: {}", e);
                return Ok(error_response(
                    StatusCode::BAD_REQUEST,
                    &format!("Invalid multipart data: {}", e),
                ));
            }
        };

        let field_name = field.name().unwrap_or("").to_string();

        match field_name.as_str() {
            "file" => {
                // 读取文件内容
                let mut data = Vec::new();
                while let Some(chunk) = field.next().await {
                    match chunk {
                        Ok(bytes) => data.extend_from_slice(&bytes),
                        Err(e) => {
                            error!("Failed to read file chunk: {}", e);
                            return Ok(error_response(
                                StatusCode::BAD_REQUEST,
                                &format!("Failed to read file: {}", e),
                            ));
                        }
                    }
                }
                csv_data = Some(data);
            }
            "mode" => {
                // 读取模式参数
                let mut data = Vec::new();
                while let Some(chunk) = field.next().await {
                    if let Ok(bytes) = chunk {
                        data.extend_from_slice(&bytes);
                    }
                }
                let mode_str = String::from_utf8_lossy(&data).to_string();
                mode = match mode_str.to_lowercase().as_str() {
                    "skip" => ImportMode::Skip,
                    "overwrite" => ImportMode::Overwrite,
                    "error" => ImportMode::Error,
                    _ => ImportMode::Skip,
                };
            }
            _ => {
                // 忽略未知字段
            }
        }
    }

    // 验证文件存在
    let csv_data = match csv_data {
        Some(data) if !data.is_empty() => data,
        _ => {
            return Ok(error_response(
                StatusCode::BAD_REQUEST,
                "No CSV file provided",
            ));
        }
    };

    info!(
        "Admin API: import mode={:?}, file size={} bytes",
        mode,
        csv_data.len()
    );

    // 解析 CSV
    let cursor = Cursor::new(&csv_data);
    let mut csv_reader = ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .trim(csv::Trim::All)
        .from_reader(cursor);

    let mut total_rows = 0;
    let mut success_count = 0;
    let mut skipped_count = 0;
    let mut failed_items: Vec<ImportFailedItem> = Vec::new();
    let mut links_to_insert: Vec<ShortLink> = Vec::new();

    // 预加载所有现有链接代码（用于检查冲突）
    let existing_codes: HashSet<String> = storage
        .load_all()
        .await
        .map_err(|e| {
            error!("Failed to load existing links for import: {}", e);
            actix_web::error::ErrorInternalServerError(format!("Database error: {}", e))
        })?
        .keys()
        .cloned()
        .collect();

    // 已处理的代码（用于 overwrite 模式下检测重复）
    let mut processed_codes: HashSet<String> = HashSet::new();

    for (row_idx, result) in csv_reader.deserialize::<CsvLinkRow>().enumerate() {
        let row_num = row_idx + 2; // CSV 行号（1-based，跳过 header）
        total_rows += 1;

        let row: CsvLinkRow = match result {
            Ok(r) => r,
            Err(e) => {
                failed_items.push(ImportFailedItem {
                    row: row_num,
                    code: String::new(),
                    error: format!("CSV parse error: {}", e),
                });
                continue;
            }
        };

        // 验证 code
        if row.code.is_empty() {
            failed_items.push(ImportFailedItem {
                row: row_num,
                code: row.code,
                error: "Empty code".to_string(),
            });
            continue;
        }

        // 验证 URL
        if let Err(e) = validate_url(&row.target) {
            failed_items.push(ImportFailedItem {
                row: row_num,
                code: row.code,
                error: format!("Invalid URL: {}", e),
            });
            continue;
        }

        // 检查冲突
        let exists = existing_codes.contains(&row.code) || processed_codes.contains(&row.code);
        if exists {
            match mode {
                ImportMode::Skip => {
                    skipped_count += 1;
                    continue;
                }
                ImportMode::Error => {
                    failed_items.push(ImportFailedItem {
                        row: row_num,
                        code: row.code,
                        error: "Link already exists".to_string(),
                    });
                    continue;
                }
                ImportMode::Overwrite => {
                    // 继续处理，允许覆盖
                }
            }
        }

        // 解析 created_at
        let created_at = chrono::DateTime::parse_from_rfc3339(&row.created_at)
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .unwrap_or_else(|_| Utc::now());

        // 解析 expires_at
        let expires_at = row.expires_at.as_ref().and_then(|s| {
            if s.is_empty() {
                None
            } else {
                chrono::DateTime::parse_from_rfc3339(s)
                    .ok()
                    .map(|dt| dt.with_timezone(&chrono::Utc))
            }
        });

        // 处理密码字段
        // 策略：如果是 Argon2 哈希则直接使用，否则进行哈希
        let password = match &row.password {
            Some(pwd) if !pwd.is_empty() => {
                if is_argon2_hash(pwd) {
                    Some(pwd.clone())
                } else {
                    match hash_password(pwd) {
                        Ok(hash) => Some(hash),
                        Err(e) => {
                            failed_items.push(ImportFailedItem {
                                row: row_num,
                                code: row.code,
                                error: format!("Password hash error: {}", e),
                            });
                            continue;
                        }
                    }
                }
            }
            _ => None,
        };

        let link = ShortLink {
            code: row.code.clone(),
            target: row.target,
            created_at,
            expires_at,
            password,
            click: row.click_count,
        };

        processed_codes.insert(row.code);
        links_to_insert.push(link);
        success_count += 1;
    }

    // 批量插入数据库
    if !links_to_insert.is_empty() {
        if let Err(e) = storage.batch_set(links_to_insert.clone()).await {
            error!("Failed to batch insert links: {}", e);
            return Ok(error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                &format!("Database error: {}", e),
            ));
        }

        // 更新缓存
        let default_ttl = crate::config::get_config().cache.default_ttl;
        for link in links_to_insert {
            let code = link.code.clone();
            let ttl = link.cache_ttl(default_ttl);
            cache.insert(&code, link, ttl).await;
        }
    }

    let failed_count = failed_items.len();

    info!(
        "Admin API: import completed - total: {}, success: {}, skipped: {}, failed: {}",
        total_rows, success_count, skipped_count, failed_count
    );

    Ok(success_response(ImportResponse {
        total_rows,
        success_count,
        skipped_count,
        failed_count,
        failed_items,
    }))
}
