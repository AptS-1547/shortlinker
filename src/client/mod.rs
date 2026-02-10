//! Client layer for CLI/TUI interfaces
//!
//! Provides IPC-first with Service-fallback execution model.
//! API handlers do NOT use this layer — they call services directly.
//!
//! # Architecture
//!
//! ```text
//! CLI/TUI → Client Layer ──→ IPC (server running)
//!                          └→ Services (server not running, lazy init)
//! ```
//!
//! # Fallback Policy
//!
//! - `IpcError::ServerNotRunning` → fallback to local service
//! - `IpcError::Timeout` → **no fallback** (risk of double-writes)
//! - Other IPC errors → no fallback, return error
//! - `IpcResponse::Error` → return as `ClientError::ServerError`

mod context;

mod config_client;
mod link_client;
mod system_client;

pub use config_client::ConfigClient;
pub use context::ServiceContext;
pub use link_client::LinkClient;
pub use system_client::SystemClient;

use std::fmt;
use std::future::Future;

use crate::errors::ShortlinkerError;
use crate::system::ipc::{IpcError, IpcResponse};

// ============ ClientError ============

/// Errors from the client layer
#[derive(Debug)]
pub enum ClientError {
    /// IPC communication error (not ServerNotRunning — that triggers fallback)
    Ipc(IpcError),
    /// Service layer error (from fallback path)
    Service(ShortlinkerError),
    /// Lazy service initialization failed
    InitFailed(String),
    /// Server returned a business error via IPC
    ServerError { code: String, message: String },
}

impl fmt::Display for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ClientError::Ipc(e) => write!(f, "IPC error: {}", e),
            ClientError::Service(e) => write!(f, "{}", e),
            ClientError::InitFailed(msg) => write!(f, "Service init failed: {}", msg),
            ClientError::ServerError { code, message } => write!(f, "{}: {}", code, message),
        }
    }
}

impl std::error::Error for ClientError {}

impl From<ShortlinkerError> for ClientError {
    fn from(err: ShortlinkerError) -> Self {
        ClientError::Service(err)
    }
}

impl From<ClientError> for ShortlinkerError {
    fn from(err: ClientError) -> Self {
        match err {
            ClientError::Service(e) => e,
            ClientError::Ipc(e) => ShortlinkerError::internal_error(format!("IPC: {}", e)),
            ClientError::InitFailed(msg) => ShortlinkerError::database_operation(msg),
            ClientError::ServerError { code, message } => {
                ShortlinkerError::internal_error(format!("{}: {}", code, message))
            }
        }
    }
}

impl From<ClientError> for crate::interfaces::cli::CliError {
    fn from(err: ClientError) -> Self {
        use crate::interfaces::cli::CliError;
        match err {
            ClientError::Ipc(e) => CliError::CommandError(format!("IPC error: {}", e)),
            ClientError::Service(e) => CliError::CommandError(e.format_simple()),
            ClientError::InitFailed(msg) => CliError::StorageError(msg),
            ClientError::ServerError { code, message } => {
                CliError::CommandError(format!("{}: {}", code, message))
            }
        }
    }
}

// ============ IPC-first + Service-fallback helper ============

/// Execute an operation with IPC-first, Service-fallback strategy.
///
/// 1. Check if server is running
/// 2. If yes, send IPC command and parse response
/// 3. If `ServerNotRunning`, execute fallback (local service)
/// 4. If `Timeout` or other IPC error, return error (no fallback)
pub(crate) async fn ipc_or_fallback<T, F, Fut>(
    ipc_call: impl Future<Output = Result<IpcResponse, IpcError>>,
    parse_response: impl FnOnce(IpcResponse) -> Result<T, ClientError>,
    fallback: F,
) -> Result<T, ClientError>
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = Result<T, ClientError>>,
{
    use crate::system::ipc;

    if !ipc::is_server_running() {
        return fallback().await;
    }

    match ipc_call.await {
        Ok(IpcResponse::Error { code, message }) => Err(ClientError::ServerError { code, message }),
        Ok(resp) => parse_response(resp),
        Err(IpcError::ServerNotRunning) => fallback().await,
        Err(e) => Err(ClientError::Ipc(e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::ShortlinkerError;
    use crate::interfaces::cli::CliError;
    use crate::system::ipc::IpcError;

    // ---- ClientError Display tests ----

    #[test]
    fn test_client_error_display_ipc_timeout() {
        let err = ClientError::Ipc(IpcError::Timeout);
        let s = format!("{}", err);
        assert!(s.contains("IPC error"), "got: {}", s);
        assert!(s.contains("timeout") || s.contains("Timeout"), "got: {}", s);
    }

    #[test]
    fn test_client_error_display_ipc_server_not_running() {
        let err = ClientError::Ipc(IpcError::ServerNotRunning);
        let s = format!("{}", err);
        assert!(s.contains("IPC error"), "got: {}", s);
    }

    #[test]
    fn test_client_error_display_ipc_protocol_error() {
        let err = ClientError::Ipc(IpcError::ProtocolError("bad data".into()));
        let s = format!("{}", err);
        assert!(s.contains("IPC error"), "got: {}", s);
        assert!(s.contains("bad data"), "got: {}", s);
    }

    #[test]
    fn test_client_error_display_service() {
        let err = ClientError::Service(ShortlinkerError::config_not_found("test key"));
        let s = format!("{}", err);
        assert!(s.contains("test key"), "got: {}", s);
    }

    #[test]
    fn test_client_error_display_init_failed() {
        let err = ClientError::InitFailed("db connection failed".into());
        assert_eq!(
            format!("{}", err),
            "Service init failed: db connection failed"
        );
    }

    #[test]
    fn test_client_error_display_server_error() {
        let err = ClientError::ServerError {
            code: "E001".into(),
            message: "bad request".into(),
        };
        assert_eq!(format!("{}", err), "E001: bad request");
    }

    // ---- From<ShortlinkerError> for ClientError ----

    #[test]
    fn test_from_shortlinker_error() {
        let se = ShortlinkerError::config_not_found("test");
        let ce: ClientError = se.into();
        assert!(matches!(ce, ClientError::Service(_)));
    }

    #[test]
    fn test_from_shortlinker_error_preserves_message() {
        let se = ShortlinkerError::database_operation("db broke");
        let ce: ClientError = se.into();
        let msg = format!("{}", ce);
        assert!(msg.contains("db broke"), "got: {}", msg);
    }

    // ---- From<ClientError> for CliError ----

    #[test]
    fn test_client_error_to_cli_error_ipc() {
        let ce = ClientError::Ipc(IpcError::Timeout);
        let cli_err: CliError = ce.into();
        match cli_err {
            CliError::CommandError(msg) => assert!(msg.contains("IPC error"), "got: {}", msg),
            other => panic!("Expected CommandError, got: {:?}", other),
        }
    }

    #[test]
    fn test_client_error_to_cli_error_service() {
        let ce = ClientError::Service(ShortlinkerError::config_not_found("test"));
        let cli_err: CliError = ce.into();
        match cli_err {
            CliError::CommandError(msg) => assert!(msg.contains("test"), "got: {}", msg),
            other => panic!("Expected CommandError, got: {:?}", other),
        }
    }

    #[test]
    fn test_client_error_to_cli_error_init_failed() {
        let ce = ClientError::InitFailed("db failed".into());
        let cli_err: CliError = ce.into();
        assert!(matches!(cli_err, CliError::StorageError(_)));
        if let CliError::StorageError(msg) = cli_err {
            assert_eq!(msg, "db failed");
        }
    }

    #[test]
    fn test_client_error_to_cli_error_server_error() {
        let ce = ClientError::ServerError {
            code: "E001".into(),
            message: "bad".into(),
        };
        let cli_err: CliError = ce.into();
        match cli_err {
            CliError::CommandError(msg) => {
                assert!(msg.contains("E001"), "got: {}", msg);
                assert!(msg.contains("bad"), "got: {}", msg);
            }
            other => panic!("Expected CommandError, got: {:?}", other),
        }
    }

    // ---- ipc_or_fallback tests ----
    // Note: ipc_or_fallback calls ipc::is_server_running() which needs config initialized.

    fn ensure_config() {
        use std::sync::Once;
        static INIT: Once = Once::new();
        INIT.call_once(|| {
            crate::config::init_config();
        });
    }

    #[tokio::test]
    async fn test_ipc_or_fallback_server_not_running_uses_fallback() {
        ensure_config();
        // In test environment, is_server_running() returns false,
        // so the fallback path is always taken.
        let result = ipc_or_fallback(
            async { Err(IpcError::ServerNotRunning) },
            |_| Ok(42),
            || async { Ok(99) },
        )
        .await;
        assert_eq!(result.unwrap(), 99);
    }

    #[tokio::test]
    async fn test_ipc_or_fallback_fallback_returns_ok() {
        ensure_config();
        let result = ipc_or_fallback(
            async { Err(IpcError::ServerNotRunning) },
            |_| Ok("ipc"),
            || async { Ok("fallback") },
        )
        .await;
        assert_eq!(result.unwrap(), "fallback");
    }

    #[tokio::test]
    async fn test_ipc_or_fallback_fallback_error_propagates() {
        ensure_config();
        let result: Result<i32, ClientError> = ipc_or_fallback(
            async { Err(IpcError::ServerNotRunning) },
            |_| Ok(42),
            || async { Err(ClientError::InitFailed("test".into())) },
        )
        .await;
        assert!(matches!(result, Err(ClientError::InitFailed(_))));
    }

    #[tokio::test]
    async fn test_ipc_or_fallback_fallback_service_error() {
        ensure_config();
        let result: Result<i32, ClientError> = ipc_or_fallback(
            async { Err(IpcError::ServerNotRunning) },
            |_| Ok(0),
            || async { Err(ClientError::Service(ShortlinkerError::not_found("missing"))) },
        )
        .await;
        assert!(matches!(result, Err(ClientError::Service(_))));
    }

    // ---- ClientError is Debug ----

    #[test]
    fn test_client_error_debug() {
        let err = ClientError::InitFailed("test".into());
        let debug = format!("{:?}", err);
        assert!(debug.contains("InitFailed"), "got: {}", debug);
    }

    // ---- ClientError implements std::error::Error ----

    #[test]
    fn test_client_error_is_std_error() {
        let err = ClientError::InitFailed("test".into());
        let _: &dyn std::error::Error = &err;
    }
}
