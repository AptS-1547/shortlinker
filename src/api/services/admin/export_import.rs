//! Admin API 导出导入操作

use actix_multipart::Multipart;
use actix_web::{HttpRequest, HttpResponse, Responder, Result as ActixResult, web};
use bytes::Bytes;
use chrono::Utc;
use csv::{ReaderBuilder, WriterBuilder};
use futures_util::stream::{Stream, StreamExt};
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::fmt::Display;
use std::io::Cursor;
use std::pin::Pin;
use std::sync::Arc;
use tracing::{debug, error, info, warn};

use crate::cache::traits::CompositeCacheTrait;
use crate::errors::ShortlinkerError;
use crate::storage::{LinkFilter, SeaOrmStorage, ShortLink};
use crate::utils::password::{hash_password, is_argon2_hash};
use crate::utils::url_validator::validate_url;

use super::error_code::ErrorCode;
use super::helpers::{error_from_shortlinker, error_response, success_response};
use super::types::{CsvLinkRow, ExportQuery, ImportFailedItem, ImportMode, ImportResponse};

/// 每批次序列化的链接数量
const EXPORT_BATCH_SIZE: usize = 10000;

/// 最大导入文件大小 (10MB)
const MAX_IMPORT_FILE_SIZE: usize = 10 * 1024 * 1024;

/// 通用流式 CSV 响应体生成器
///
/// 接收一个分批数据流和行映射函数，返回一个 Bytes 流。
/// 第一个 chunk 包含 CSV header。
///
/// # 参数
/// - `batch_stream`: 分批数据流
/// - `row_mapper`: 将每个 item 转换为可序列化的 CSV 行
/// - `item_name`: 用于日志的项目名称（如 "links", "logs"）
pub fn create_csv_stream<T, R, F, E>(
    batch_stream: Pin<Box<dyn Stream<Item = Result<Vec<T>, E>> + Send + 'static>>,
    row_mapper: F,
    item_name: &'static str,
) -> impl Stream<Item = Result<Bytes, actix_web::Error>>
where
    T: Send + 'static,
    R: Serialize + Send + 'static,
    F: Fn(T) -> R + Send + Clone + 'static,
    E: Display + Send + 'static,
{
    use futures_util::stream;

    stream::unfold(
        (batch_stream, true, 0usize, row_mapper, item_name),
        |(mut stream, mut first, mut count, mapper, item_name)| async move {
            match stream.next().await {
                Some(Ok(batch)) if batch.is_empty() => None,
                Some(Ok(batch)) => {
                    let batch_len = batch.len();
                    let is_first = first;
                    let mapper_clone = mapper.clone();

                    // 将 CSV 序列化移到 blocking 线程池
                    let csv_result = tokio::task::spawn_blocking(move || {
                        let mut csv_writer = WriterBuilder::new()
                            .has_headers(is_first)
                            .from_writer(vec![]);

                        let mut serialize_errors = 0usize;

                        for item in batch {
                            let row = mapper_clone(item);
                            if let Err(e) = csv_writer.serialize(&row) {
                                error!("Failed to serialize CSV row: {}", e);
                                serialize_errors += 1;
                            }
                        }

                        (csv_writer.into_inner(), serialize_errors)
                    })
                    .await;

                    match csv_result {
                        Ok((Ok(chunk), serialize_errors)) => {
                            count += batch_len;

                            if first {
                                info!(
                                    "Export stream: sent CSV header + {} {}",
                                    batch_len, item_name
                                );
                                first = false;
                            } else {
                                debug!(
                                    "Export stream: sent batch of {} {} (total: {})",
                                    batch_len, item_name, count
                                );
                            }

                            if serialize_errors > 0 {
                                warn!(
                                    "Export stream: {} serialize errors in this batch",
                                    serialize_errors
                                );
                            }

                            Some((
                                Ok(Bytes::from(chunk)),
                                (stream, first, count, mapper, item_name),
                            ))
                        }
                        Ok((Err(e), _)) => {
                            error!("Failed to finalize CSV writer: {}", e.error());
                            Some((
                                Err(actix_web::error::ErrorInternalServerError(
                                    "CSV generation error",
                                )),
                                (stream, first, count, mapper, item_name),
                            ))
                        }
                        Err(e) => {
                            error!("Blocking task panicked: {}", e);
                            Some((
                                Err(actix_web::error::ErrorInternalServerError(
                                    "CSV task failed",
                                )),
                                (stream, first, count, mapper, item_name),
                            ))
                        }
                    }
                }
                Some(Err(e)) => {
                    error!(
                        "Export stream database error at ~{} {}: {}",
                        count, item_name, e
                    );
                    let error_msg = format!("# ERROR: Database error: {}\n", e);
                    Some((
                        Ok(Bytes::from(error_msg)),
                        (stream, first, count, mapper, item_name),
                    ))
                }
                None => {
                    info!(
                        "Export stream completed: {} total {} exported",
                        count, item_name
                    );
                    None
                }
            }
        },
    )
}

/// 导出链接为 CSV（流式响应）
pub async fn export_links(
    _req: HttpRequest,
    query: web::Query<ExportQuery>,
    storage: web::Data<Arc<SeaOrmStorage>>,
) -> ActixResult<impl Responder> {
    info!(
        "Admin API: export links (streaming) with filters: {:?}",
        query
    );

    // 解析并验证日期参数
    let created_after = match &query.created_after {
        Some(s) => match chrono::DateTime::parse_from_rfc3339(s) {
            Ok(dt) => Some(dt.with_timezone(&chrono::Utc)),
            Err(_) => {
                return Ok(error_response(
                    actix_web::http::StatusCode::BAD_REQUEST,
                    ErrorCode::InvalidDateFormat,
                    &format!(
                        "Invalid created_after: '{}'. Use RFC3339 (e.g., 2024-01-01T00:00:00Z)",
                        s
                    ),
                ));
            }
        },
        None => None,
    };

    let created_before = match &query.created_before {
        Some(s) => match chrono::DateTime::parse_from_rfc3339(s) {
            Ok(dt) => Some(dt.with_timezone(&chrono::Utc)),
            Err(_) => {
                return Ok(error_response(
                    actix_web::http::StatusCode::BAD_REQUEST,
                    ErrorCode::InvalidDateFormat,
                    &format!(
                        "Invalid created_before: '{}'. Use RFC3339 (e.g., 2024-01-01T00:00:00Z)",
                        s
                    ),
                ));
            }
        },
        None => None,
    };

    // 构建过滤条件
    let filter = LinkFilter {
        search: query.search.clone(),
        created_after,
        created_before,
        only_expired: query.only_expired.unwrap_or(false),
        only_active: query.only_active.unwrap_or(false),
    };

    // 获取游标分页流式数据
    let batch_stream = storage.stream_all_filtered_cursor(filter, EXPORT_BATCH_SIZE as u64);

    // 行映射：ShortLink → CsvLinkRow
    let row_mapper = |link: ShortLink| CsvLinkRow {
        code: link.code,
        target: link.target,
        created_at: link.created_at.to_rfc3339(),
        expires_at: link.expires_at.map(|dt| dt.to_rfc3339()),
        password: link.password,
        click_count: link.click,
    };

    // 创建 CSV 流
    let csv_stream = create_csv_stream(batch_stream, row_mapper, "links");

    // 生成文件名
    let filename = format!(
        "shortlinks_export_{}.csv",
        Utc::now().format("%Y%m%d_%H%M%S")
    );

    info!("Admin API: starting streaming export to {}", filename);

    // 返回流式响应
    Ok(HttpResponse::Ok()
        .content_type("text/csv; charset=utf-8")
        .insert_header((
            "Content-Disposition",
            format!("attachment; filename=\"{}\"", filename),
        ))
        .insert_header(("Transfer-Encoding", "chunked"))
        .streaming(csv_stream))
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
                return Ok(error_from_shortlinker(
                    &ShortlinkerError::invalid_multipart_data(format!(
                        "Invalid multipart data: {}",
                        e
                    )),
                ));
            }
        };

        let field_name = field.name().unwrap_or("").to_string();

        match field_name.as_str() {
            "file" => {
                // 读取文件内容（带大小限制）
                let mut data = Vec::new();
                while let Some(chunk) = field.next().await {
                    match chunk {
                        Ok(bytes) => {
                            // 检查累积大小
                            if data.len() + bytes.len() > MAX_IMPORT_FILE_SIZE {
                                return Ok(error_response(
                                    actix_web::http::StatusCode::BAD_REQUEST,
                                    ErrorCode::FileTooLarge,
                                    &format!(
                                        "File size exceeds maximum {} MB",
                                        MAX_IMPORT_FILE_SIZE / 1024 / 1024
                                    ),
                                ));
                            }
                            data.extend_from_slice(&bytes);
                        }
                        Err(e) => {
                            error!("Failed to read file chunk: {}", e);
                            return Ok(error_from_shortlinker(&ShortlinkerError::file_read_error(
                                format!("Failed to read file: {}", e),
                            )));
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
            return Ok(error_from_shortlinker(&ShortlinkerError::csv_file_missing(
                "No CSV file provided",
            )));
        }
    };

    info!(
        "Admin API: import mode={:?}, file size={} bytes",
        mode,
        csv_data.len()
    );

    // 解析 CSV（单次解析，收集所有行）
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

    // 单次解析：收集所有行和 codes
    let mut parsed_rows: Vec<(usize, CsvLinkRow)> = Vec::new();
    let mut all_codes: Vec<String> = Vec::new();

    for (row_idx, result) in csv_reader.deserialize::<CsvLinkRow>().enumerate() {
        let row_num = row_idx + 2; // CSV 行号（1-based，跳过 header）
        total_rows += 1;

        match result {
            Ok(row) => {
                if !row.code.is_empty() {
                    all_codes.push(row.code.clone());
                }
                parsed_rows.push((row_num, row));
            }
            Err(e) => {
                failed_items.push(ImportFailedItem {
                    row: row_num,
                    code: String::new(),
                    error: format!("CSV parse error: {}", e),
                    error_code: Some(ErrorCode::CsvParseError as i32),
                });
            }
        }
    }

    // 冲突检测：Overwrite 模式不需要，Skip/Error 用 Bloom 预筛选
    let existing_codes: HashSet<String> = match mode {
        ImportMode::Overwrite => HashSet::new(),
        ImportMode::Skip | ImportMode::Error => {
            // Bloom Filter 预筛选：false = 一定不存在，跳过 DB 查询
            let mut maybe_exist = Vec::new();
            for code in &all_codes {
                if cache.bloom_check(code).await {
                    maybe_exist.push(code.clone());
                }
            }

            // 只对 Bloom 返回"可能存在"的 codes 精确查询
            if maybe_exist.is_empty() {
                HashSet::new()
            } else {
                storage
                    .batch_check_codes_exist(&maybe_exist)
                    .await
                    .map_err(|e| {
                        error!("Failed to check existing codes for import: {}", e);
                        actix_web::error::ErrorInternalServerError(format!("Database error: {}", e))
                    })?
            }
        }
    };

    // 用 HashMap 避免 Overwrite 模式下 CSV 重复 code 导致 batch_set 失败
    let mut links_to_insert: HashMap<String, ShortLink> = HashMap::new();
    // 已处理的代码（用于检测 CSV 内重复）
    let mut processed_codes: HashSet<String> = HashSet::new();

    // 处理收集的行数据
    for (row_num, row) in parsed_rows {
        // 验证 code
        if row.code.is_empty() {
            failed_items.push(ImportFailedItem {
                row: row_num,
                code: row.code,
                error: "Empty code".to_string(),
                error_code: Some(ErrorCode::LinkEmptyCode as i32),
            });
            continue;
        }

        // 验证 URL
        if let Err(e) = validate_url(&row.target) {
            failed_items.push(ImportFailedItem {
                row: row_num,
                code: row.code,
                error: format!("Invalid URL: {}", e),
                error_code: Some(ErrorCode::LinkInvalidUrl as i32),
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
                        error_code: Some(ErrorCode::LinkAlreadyExists as i32),
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
            .unwrap_or_else(|_| {
                warn!(
                    "Row {}: Invalid created_at '{}', using current time",
                    row_num, row.created_at
                );
                Utc::now()
            });

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
                                error_code: Some(ErrorCode::LinkPasswordHashError as i32),
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

        processed_codes.insert(row.code.clone());
        links_to_insert.insert(row.code, link);
        success_count += 1;
    }

    // 批量插入数据库
    let links_vec: Vec<ShortLink> = links_to_insert.into_values().collect();
    if !links_vec.is_empty() {
        if let Err(e) = storage.batch_set(links_vec.clone()).await {
            error!("Failed to batch insert links: {}", e);
            return Ok(error_from_shortlinker(&ShortlinkerError::import_failed(
                format!("Database error: {}", e),
            )));
        }

        // 更新缓存
        let default_ttl = crate::config::get_config().cache.default_ttl;
        for link in links_vec {
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
