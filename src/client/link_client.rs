//! Link management client (IPC-first + LinkService-fallback)

use std::sync::Arc;

use crate::services::{
    CreateLinkRequest, ImportBatchFailedItem, ImportBatchResult, ImportLinkItemRich, ImportMode,
    LinkCreateResult, UpdateLinkRequest,
};
use crate::storage::{LinkFilter, LinkStats, ShortLink};
use crate::system::ipc::{self, IpcResponse};

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
                    link,
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
                IpcResponse::LinkUpdated { link } => Ok(link),
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
                IpcResponse::LinkFound { link } => Ok(link),
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
                IpcResponse::LinkList { links, total, .. } => Ok((links, total as u64)),
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
            .iter()
            .map(crate::system::ipc::ImportLinkData::from)
            .collect();
        ipc_or_fallback(
            ipc::import_links(ipc_links, overwrite),
            |resp| match resp {
                IpcResponse::ImportResult {
                    success,
                    skipped,
                    errors,
                    ..
                } => Ok(convert_import_result(success, skipped, errors)),
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
                .map(crate::system::ipc::ImportLinkData::from)
                .collect();

            match ipc::import_links_streaming(ipc_links, overwrite, &on_progress).await {
                Ok(IpcResponse::ImportResult {
                    success,
                    skipped,
                    errors,
                    ..
                }) => {
                    return Ok(convert_import_result(success, skipped, errors));
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
                    return Ok(links);
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

/// Convert IPC ImportResult fields to ImportBatchResult
fn convert_import_result(
    success: usize,
    skipped: usize,
    errors: Vec<crate::system::ipc::types::ImportErrorData>,
) -> ImportBatchResult {
    ImportBatchResult {
        success_count: success,
        skipped_count: skipped,
        failed_items: errors
            .into_iter()
            .map(|e| ImportBatchFailedItem {
                code: e.code,
                error: match e.error_code {
                    Some(ec) => crate::errors::ShortlinkerError::from_error_code(&ec, e.message),
                    None => crate::errors::ShortlinkerError::import_failed(e.message),
                },
                row_num: None,
            })
            .collect(),
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
}
