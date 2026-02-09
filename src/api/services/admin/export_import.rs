//! Admin API 导出导入操作

use actix_multipart::Multipart;
use actix_web::{HttpRequest, HttpResponse, Responder, Result as ActixResult, web};
use bytes::Bytes;
use chrono::Utc;
use csv::{ReaderBuilder, WriterBuilder};
use futures_util::stream::{Stream, StreamExt};
use serde::Serialize;
use std::fmt::Display;
use std::io::Cursor;
use std::pin::Pin;
use std::sync::Arc;
use tracing::{debug, error, info, warn};

use crate::errors::ShortlinkerError;
use crate::services::{ImportLinkItem, LinkService};
use crate::storage::{LinkFilter, SeaOrmStorage, ShortLink};

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
///
/// 注意：此 handler 直接调用 storage.stream_all_filtered_cursor()，
/// 这是合理的例外，因为：
/// 1. 流式操作的性能优化在 Storage 层完成
/// 2. Service 层封装流式方法只是简单转发，无业务价值
/// 3. 对于小数据量场景，可使用 LinkService::export_links()
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
    link_service: web::Data<Arc<LinkService>>,
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
    let mut failed_items: Vec<ImportFailedItem> = Vec::new();

    // 解析 CSV，收集所有行
    let mut parsed_items: Vec<ImportLinkItem> = Vec::new();

    for (row_idx, result) in csv_reader.deserialize::<CsvLinkRow>().enumerate() {
        let row_num = row_idx + 2; // CSV 行号（1-based，跳过 header）
        total_rows += 1;

        match result {
            Ok(row) => {
                // 基础验证
                if row.code.is_empty() {
                    failed_items.push(ImportFailedItem {
                        row: row_num,
                        code: row.code,
                        error: "Empty code".to_string(),
                        error_code: Some(ErrorCode::LinkEmptyCode as i32),
                    });
                    continue;
                }

                parsed_items.push(ImportLinkItem {
                    code: row.code,
                    target: row.target,
                    expires_at: row.expires_at,
                    password: row.password,
                });
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

    // 调用 LinkService 处理导入
    let import_result = link_service
        .import_links_with_bloom(parsed_items, mode)
        .await
        .map_err(|e| {
            error!("Failed to import links: {}", e);
            actix_web::error::ErrorInternalServerError(format!("Import failed: {}", e))
        })?;

    // 合并错误信息
    for err in import_result.errors {
        failed_items.push(ImportFailedItem {
            row: 0, // Service 层不知道行号
            code: err.code,
            error: err.message,
            error_code: None,
        });
    }

    let failed_count = failed_items.len();

    info!(
        "Admin API: import completed - total: {}, success: {}, skipped: {}, failed: {}",
        total_rows, import_result.success, import_result.skipped, failed_count
    );

    Ok(success_response(ImportResponse {
        total_rows,
        success_count: import_result.success,
        skipped_count: import_result.skipped,
        failed_count,
        failed_items,
    }))
}
