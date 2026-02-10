//! Link management client (IPC-first + LinkService-fallback)

use std::sync::Arc;

use chrono::{DateTime, Utc};

use crate::services::{
    CreateLinkRequest, ImportBatchResult, ImportLinkItemRich, ImportMode, LinkCreateResult,
    UpdateLinkRequest,
};
use crate::storage::{LinkFilter, LinkStats, ShortLink};
use crate::system::ipc::{self, IpcResponse, ShortLinkData};

use super::context::ServiceContext;
use super::{ClientError, ipc_or_fallback};

/// Link operations client.
///
/// IPC-first with LinkService-fallback for all operations.
pub struct LinkClient {
    ctx: Arc<ServiceContext>,
}

impl LinkClient {
    pub fn new(ctx: Arc<ServiceContext>) -> Self {
        Self { ctx }
    }

    /// Create a new short link
    pub async fn create_link(
        &self,
        code: Option<String>,
        target: String,
        force: bool,
        expires_at: Option<String>,
        password: Option<String>,
    ) -> Result<LinkCreateResult, ClientError> {
        let ctx = self.ctx.clone();
        let req = CreateLinkRequest {
            code: code.clone(),
            target: target.clone(),
            force,
            expires_at: expires_at.clone(),
            password: password.clone(),
        };
        ipc_or_fallback(
            ipc::add_link(code, target, force, expires_at, password),
            |resp| match resp {
                IpcResponse::LinkCreated {
                    link,
                    generated_code,
                } => Ok(LinkCreateResult {
                    link: link_data_to_short_link(&link),
                    generated_code,
                }),
                other => Err(unexpected_response(other)),
            },
            || async move {
                let service = ctx.get_link_service().await?;
                Ok(service.create_link(req).await?)
            },
        )
        .await
    }

    /// Delete a short link
    pub async fn delete_link(&self, code: String) -> Result<(), ClientError> {
        let ctx = self.ctx.clone();
        let code2 = code.clone();
        ipc_or_fallback(
            ipc::remove_link(code),
            |resp| match resp {
                IpcResponse::LinkDeleted { .. } => Ok(()),
                other => Err(unexpected_response(other)),
            },
            || async move {
                let service = ctx.get_link_service().await?;
                Ok(service.delete_link(&code2).await?)
            },
        )
        .await
    }

    /// Batch delete short links
    pub async fn batch_delete(
        &self,
        codes: Vec<String>,
    ) -> Result<crate::services::BatchDeleteResult, ClientError> {
        let ctx = self.ctx.clone();
        let codes2 = codes.clone();
        ipc_or_fallback(
            ipc::batch_delete_links(codes),
            |resp| match resp {
                IpcResponse::BatchDeleteResult {
                    deleted,
                    not_found,
                    errors,
                } => Ok(crate::services::BatchDeleteResult {
                    deleted,
                    not_found,
                    errors: errors
                        .into_iter()
                        .map(|e| crate::services::BatchFailedItem {
                            code: e.code,
                            reason: e.message,
                        })
                        .collect(),
                }),
                other => Err(unexpected_response(other)),
            },
            || async move {
                let service = ctx.get_link_service().await?;
                Ok(service.batch_delete_links(codes2).await?)
            },
        )
        .await
    }

    /// Update an existing short link
    pub async fn update_link(
        &self,
        code: String,
        target: String,
        expires_at: Option<String>,
        password: Option<String>,
    ) -> Result<ShortLink, ClientError> {
        let ctx = self.ctx.clone();
        let code2 = code.clone();
        let req = UpdateLinkRequest {
            target: target.clone(),
            expires_at: expires_at.clone(),
            password: password.clone(),
        };
        ipc_or_fallback(
            ipc::update_link(code, target, expires_at, password),
            |resp| match resp {
                IpcResponse::LinkUpdated { link } => Ok(link_data_to_short_link(&link)),
                other => Err(unexpected_response(other)),
            },
            || async move {
                let service = ctx.get_link_service().await?;
                Ok(service.update_link(&code2, req).await?)
            },
        )
        .await
    }

    /// Get a single short link
    pub async fn get_link(&self, code: String) -> Result<Option<ShortLink>, ClientError> {
        let ctx = self.ctx.clone();
        let code2 = code.clone();
        ipc_or_fallback(
            ipc::get_link(code),
            |resp| match resp {
                IpcResponse::LinkFound { link } => Ok(link.as_ref().map(link_data_to_short_link)),
                other => Err(unexpected_response(other)),
            },
            || async move {
                let service = ctx.get_link_service().await?;
                Ok(service.get_link(&code2).await?)
            },
        )
        .await
    }

    /// List links with pagination and optional search
    pub async fn list_links(
        &self,
        page: u64,
        page_size: u64,
        search: Option<String>,
    ) -> Result<(Vec<ShortLink>, u64), ClientError> {
        let ctx = self.ctx.clone();
        let search2 = search.clone();
        ipc_or_fallback(
            ipc::list_links(page, page_size, search),
            |resp| match resp {
                IpcResponse::LinkList { links, total, .. } => {
                    let short_links: Vec<ShortLink> =
                        links.iter().map(link_data_to_short_link).collect();
                    Ok((short_links, total as u64))
                }
                other => Err(unexpected_response(other)),
            },
            || async move {
                let service = ctx.get_link_service().await?;
                let filter = LinkFilter {
                    search: search2,
                    created_after: None,
                    created_before: None,
                    only_expired: false,
                    only_active: false,
                };
                Ok(service.list_links(filter, page, page_size).await?)
            },
        )
        .await
    }

    /// Import links
    pub async fn import_links(
        &self,
        items: Vec<ImportLinkItemRich>,
        overwrite: bool,
    ) -> Result<ImportBatchResult, ClientError> {
        let ctx = self.ctx.clone();
        let items2 = items.clone();
        // Convert to IPC format
        let ipc_links: Vec<crate::system::ipc::ImportLinkData> = items
            .into_iter()
            .map(|l| crate::system::ipc::ImportLinkData {
                code: l.code,
                target: l.target,
                created_at: l.created_at.to_rfc3339(),
                expires_at: l.expires_at.map(|dt| dt.to_rfc3339()),
                password: l.password,
                click_count: l.click_count,
            })
            .collect();
        ipc_or_fallback(
            ipc::import_links(ipc_links, overwrite),
            |resp| match resp {
                IpcResponse::ImportResult {
                    success,
                    skipped,
                    errors,
                    ..
                } => Ok(ImportBatchResult {
                    success_count: success,
                    skipped_count: skipped,
                    failed_items: errors
                        .into_iter()
                        .map(|e| crate::services::ImportBatchFailedItem {
                            code: e.code,
                            error: match e.error_code {
                                Some(ec) => {
                                    crate::errors::ShortlinkerError::from_error_code(&ec, e.message)
                                }
                                None => crate::errors::ShortlinkerError::import_failed(e.message),
                            },
                            row_num: None,
                        })
                        .collect(),
                }),
                other => Err(unexpected_response(other)),
            },
            || async move {
                let service = ctx.get_link_service().await?;
                let mode = ImportMode::from_overwrite_flag(overwrite);
                Ok(service.import_links_batch(items2, mode).await?)
            },
        )
        .await
    }

    /// Import links with streaming progress reports
    ///
    /// IPC path: uses `import_links_streaming` with progress callback.
    /// Fallback path: uses `import_links_batch_chunked` with progress callback.
    pub async fn import_links_with_progress(
        &self,
        items: Vec<ImportLinkItemRich>,
        overwrite: bool,
        on_progress: impl Fn(&crate::system::ipc::types::ImportPhase, usize, usize) + Send + Sync,
    ) -> Result<ImportBatchResult, ClientError> {
        let ctx = self.ctx.clone();

        // IPC path
        if ipc::is_server_running() {
            let ipc_links: Vec<crate::system::ipc::ImportLinkData> = items
                .iter()
                .map(|l| crate::system::ipc::ImportLinkData {
                    code: l.code.clone(),
                    target: l.target.clone(),
                    created_at: l.created_at.to_rfc3339(),
                    expires_at: l.expires_at.map(|dt| dt.to_rfc3339()),
                    password: l.password.clone(),
                    click_count: l.click_count,
                })
                .collect();

            match ipc::import_links_streaming(ipc_links, overwrite, &on_progress).await {
                Ok(IpcResponse::ImportResult {
                    success,
                    skipped,
                    errors,
                    ..
                }) => {
                    return Ok(ImportBatchResult {
                        success_count: success,
                        skipped_count: skipped,
                        failed_items: errors
                            .into_iter()
                            .map(|e| crate::services::ImportBatchFailedItem {
                                code: e.code,
                                error: match e.error_code {
                                    Some(ec) => crate::errors::ShortlinkerError::from_error_code(
                                        &ec, e.message,
                                    ),
                                    None => {
                                        crate::errors::ShortlinkerError::import_failed(e.message)
                                    }
                                },
                                row_num: None,
                            })
                            .collect(),
                    });
                }
                Ok(other) => return Err(unexpected_response(other)),
                Err(crate::system::ipc::IpcError::ServerNotRunning) => {
                    // Fall through to fallback
                }
                Err(e) => return Err(ClientError::Ipc(e)),
            }
        }

        // Fallback path: use chunked import with progress
        use crate::system::ipc::types::ImportPhase;
        let service = ctx.get_link_service().await?;
        let mode = ImportMode::from_overwrite_flag(overwrite);

        // Wrap the progress callback to emit ImportPhase::Writing
        let on_chunk = move |processed: usize, total: usize| {
            on_progress(&ImportPhase::Writing, processed, total);
        };

        Ok(service
            .import_links_batch_chunked(items, mode, 500, Some(&on_chunk))
            .await?)
    }

    /// Export all links
    pub async fn export_links(&self) -> Result<Vec<ShortLink>, ClientError> {
        let ctx = self.ctx.clone();
        // IPC path: streaming export returns Vec<ShortLinkData> directly
        if ipc::is_server_running() {
            match ipc::export_links().await {
                Ok(links) => {
                    return Ok(links.iter().map(link_data_to_short_link).collect());
                }
                Err(crate::system::ipc::IpcError::ServerNotRunning) => {
                    // Fall through to fallback
                }
                Err(e) => {
                    return Err(ClientError::Ipc(e));
                }
            }
        }

        // Fallback: use stream + collect
        use futures_util::StreamExt;
        let service = ctx.get_link_service().await?;
        let mut stream = service.export_links_stream(crate::storage::LinkFilter::default(), 10000);
        let mut all_links = Vec::new();
        while let Some(batch) = stream.next().await {
            let links = batch.map_err(ClientError::Service)?;
            all_links.extend(links);
        }
        Ok(all_links)
    }

    /// Get link statistics
    pub async fn get_stats(&self) -> Result<LinkStats, ClientError> {
        let ctx = self.ctx.clone();
        ipc_or_fallback(
            ipc::get_link_stats(),
            |resp| match resp {
                IpcResponse::StatsResult {
                    total_links,
                    total_clicks,
                    active_links,
                } => Ok(LinkStats {
                    total_links,
                    total_clicks: total_clicks.max(0) as usize,
                    active_links,
                }),
                other => Err(unexpected_response(other)),
            },
            || async move {
                let service = ctx.get_link_service().await?;
                Ok(service.get_stats().await?)
            },
        )
        .await
    }
}

// ============ Conversion helpers ============

/// Convert IPC ShortLinkData to storage ShortLink
fn link_data_to_short_link(data: &ShortLinkData) -> ShortLink {
    ShortLink {
        code: data.code.clone(),
        target: data.target.clone(),
        created_at: DateTime::parse_from_rfc3339(&data.created_at)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|e| {
                tracing::warn!(
                    "Failed to parse 'created_at' from IPC (value: '{}'): {}",
                    &data.created_at,
                    e
                );
                Utc::now()
            }),
        expires_at: data.expires_at.as_ref().and_then(|s| {
            DateTime::parse_from_rfc3339(s)
                .map(|dt| dt.with_timezone(&Utc))
                .ok()
        }),
        password: data.password.clone(),
        click: data.click.max(0) as usize,
    }
}

fn unexpected_response(resp: IpcResponse) -> ClientError {
    ClientError::Ipc(crate::system::ipc::IpcError::ProtocolError(format!(
        "Unexpected response: {:?}",
        resp
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_link_data(
        code: &str,
        target: &str,
        created_at: &str,
        expires_at: Option<&str>,
        password: Option<&str>,
        click: i64,
    ) -> ShortLinkData {
        ShortLinkData {
            code: code.into(),
            target: target.into(),
            created_at: created_at.into(),
            expires_at: expires_at.map(|s| s.into()),
            password: password.map(|s| s.into()),
            click,
        }
    }

    // ---- link_data_to_short_link tests ----

    #[test]
    fn test_link_data_to_short_link_basic() {
        let data = make_link_data(
            "abc",
            "https://example.com",
            "2025-01-01T00:00:00Z",
            Some("2025-12-31T23:59:59Z"),
            Some("hashed"),
            42,
        );
        let link = link_data_to_short_link(&data);
        assert_eq!(link.code, "abc");
        assert_eq!(link.target, "https://example.com");
        assert_eq!(link.click, 42);
        assert!(link.expires_at.is_some());
        assert_eq!(link.password, Some("hashed".into()));
    }

    #[test]
    fn test_link_data_to_short_link_valid_created_at() {
        let data = make_link_data(
            "test",
            "https://example.com",
            "2025-06-15T12:30:00+00:00",
            None,
            None,
            0,
        );
        let link = link_data_to_short_link(&data);
        assert_eq!(link.created_at.year(), 2025);
        assert_eq!(link.created_at.month(), 6);
        assert_eq!(link.created_at.day(), 15);
    }

    #[test]
    fn test_link_data_to_short_link_no_expires() {
        let data = make_link_data(
            "abc",
            "https://example.com",
            "2025-01-01T00:00:00Z",
            None,
            None,
            0,
        );
        let link = link_data_to_short_link(&data);
        assert!(link.expires_at.is_none());
        assert!(link.password.is_none());
    }

    #[test]
    fn test_link_data_to_short_link_invalid_created_at() {
        let data = make_link_data("abc", "https://example.com", "not-a-date", None, None, 0);
        let link = link_data_to_short_link(&data);
        // Invalid created_at falls back to Utc::now()
        let now = Utc::now();
        let diff = (now - link.created_at).num_seconds().abs();
        assert!(diff < 5, "Expected created_at near now, diff={}s", diff);
    }

    #[test]
    fn test_link_data_to_short_link_invalid_expires_at() {
        let data = make_link_data(
            "abc",
            "https://example.com",
            "2025-01-01T00:00:00Z",
            Some("also-not-a-date"),
            None,
            0,
        );
        let link = link_data_to_short_link(&data);
        // Invalid expires_at becomes None
        assert!(link.expires_at.is_none());
    }

    #[test]
    fn test_link_data_to_short_link_click_conversion() {
        let data = make_link_data(
            "abc",
            "https://example.com",
            "2025-01-01T00:00:00Z",
            None,
            None,
            999999,
        );
        let link = link_data_to_short_link(&data);
        assert_eq!(link.click, 999999);
    }

    #[test]
    fn test_link_data_to_short_link_zero_click() {
        let data = make_link_data(
            "abc",
            "https://example.com",
            "2025-01-01T00:00:00Z",
            None,
            None,
            0,
        );
        let link = link_data_to_short_link(&data);
        assert_eq!(link.click, 0);
    }

    // ---- unexpected_response tests ----

    #[test]
    fn test_unexpected_response_pong() {
        let err = unexpected_response(IpcResponse::Pong {
            version: "1.0".into(),
            uptime_secs: 0,
        });
        assert!(matches!(err, ClientError::Ipc(_)));
        let msg = format!("{}", err);
        assert!(msg.contains("Unexpected response"), "got: {}", msg);
    }

    #[test]
    fn test_unexpected_response_shutting_down() {
        let err = unexpected_response(IpcResponse::ShuttingDown);
        assert!(matches!(err, ClientError::Ipc(_)));
    }

    use chrono::Datelike;
}
