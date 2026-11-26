//! IPC protocol type definitions
//!
//! Defines the types used for IPC communication:
//! - `IpcCommand`: Commands sent from client to server
//! - `IpcResponse`: Responses sent from server to client
//! - `IpcError`: IPC-related errors

use serde::{Deserialize, Serialize};
use std::fmt;
use std::io;

use crate::storage::ShortLink;
use crate::system::reload::ReloadTarget;

/// Import link data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportLinkData {
    /// Short link code; the client may omit or duplicate codes, conflicts are resolved by `overwrite`.
    pub code: String,
    /// Redirect target URL to import.
    pub target: String,
    /// Creation timestamp in RFC 3339 format.
    pub created_at: String,
    /// Expiration timestamp in RFC 3339 format, or `None` for a never-expiring link.
    pub expires_at: Option<String>,
    /// Optional password protecting the link; plain text values are hashed on import, existing Argon2 hashes are kept as-is.
    pub password: Option<String>,
    /// Pre-existing click count preserved from the source installation.
    pub click_count: usize,
}

impl From<&crate::services::ImportLinkItemRich> for ImportLinkData {
    fn from(l: &crate::services::ImportLinkItemRich) -> Self {
        Self {
            code: l.code.clone(),
            target: l.target.clone(),
            created_at: l.created_at.to_rfc3339(),
            expires_at: l.expires_at.map(|dt| dt.to_rfc3339()),
            password: l.password.clone(),
            click_count: l.click_count,
        }
    }
}

/// Import error data for IPC transfer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportErrorData {
    /// Short link code the failure relates to.
    pub code: String,
    /// Human-readable description of why the entry failed.
    pub message: String,
    /// 结构化错误码（如 "E020"），用于客户端区分具体失败原因
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_code: Option<String>,
}

/// Config item data for IPC transfer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigItemData {
    /// Configuration key, e.g. `api.admin_token`.
    pub key: String,
    /// Current value; sensitive entries are masked before transfer.
    pub value: String,
    /// Category identifier the entry belongs to (see `config/definitions.rs`).
    pub category: String,
    /// Declared value type name, e.g. `"String"`, `"Number"`, `"Boolean"`.
    pub value_type: String,
    /// Factory default value as a string.
    pub default_value: String,
    /// Whether changing this entry only takes effect after a server restart.
    pub requires_restart: bool,
    /// Whether the entry may be edited through the admin panel and IPC.
    pub editable: bool,
    /// Whether the value must be masked in listings, logs and exports.
    pub sensitive: bool,
    /// Human-readable description shown in the admin panel.
    pub description: String,
    /// Allowed values for enum-style entries, `None` for free-form types.
    pub enum_options: Option<Vec<String>>,
    /// Last modification timestamp in RFC 3339 format, empty when never changed.
    pub updated_at: String,
}

/// Import progress phase
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImportPhase {
    /// Validating entries: URL format, short code format and field sanity.
    Validating,
    /// Checking imported codes against existing links for conflicts.
    ConflictCheck,
    /// Persisting validated entries to storage.
    Writing,
    /// Refreshing caches after the written entries.
    CacheUpdate,
}

impl fmt::Display for ImportPhase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ImportPhase::Validating => write!(f, "Validating"),
            ImportPhase::ConflictCheck => write!(f, "Conflict check"),
            ImportPhase::Writing => write!(f, "Writing"),
            ImportPhase::CacheUpdate => write!(f, "Cache update"),
        }
    }
}

/// Config import item for batch import
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigImportItem {
    /// Configuration key to set.
    pub key: String,
    /// Raw string value; normalized and validated by the server before persisting.
    pub value: String,
}

/// IPC commands sent from client to server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IpcCommand {
    /// Heartbeat check - confirm server is alive and responsive
    Ping,

    /// Reload request with specific target
    Reload {
        /// Subsystem to reload (data, config or everything).
        target: ReloadTarget,
    },

    /// Query current server status
    GetStatus,

    /// Request graceful shutdown
    Shutdown,

    // ============ Link Management Commands ============
    /// Add a new short link
    AddLink {
        /// Requested short code, or `None` to let the server generate a random one.
        code: Option<String>,
        /// Redirect target URL.
        target: String,
        /// Whether to overwrite an existing link with the same code.
        force: bool,
        /// Expiration timestamp in RFC 3339 format, or `None` for a never-expiring link.
        expires_at: Option<String>,
        /// Optional password protecting the link, hashed by the server.
        password: Option<String>,
    },

    /// Remove a short link
    RemoveLink {
        /// Short code of the link to delete.
        code: String,
    },

    /// Batch delete short links
    BatchDeleteLinks {
        /// Short codes to delete; per-code results are reported in `BatchDeleteResult`.
        codes: Vec<String>,
    },

    /// Update an existing short link
    UpdateLink {
        /// Short code of the link to update.
        code: String,
        /// New redirect target URL.
        target: String,
        /// New expiration timestamp in RFC 3339 format; `None` keeps the existing expiration.
        expires_at: Option<String>,
        /// New password in plain text (hashed server-side); `None` keeps the existing one, an empty string removes it.
        password: Option<String>,
    },

    /// Get a single short link
    GetLink {
        /// Short code to look up.
        code: String,
    },

    /// List all short links with pagination
    ListLinks {
        /// 1-based page number.
        page: u64,
        /// Number of entries per page.
        page_size: u64,
        /// Optional case-insensitive substring filter matched against code and target.
        search: Option<String>,
    },

    /// Import multiple links
    ImportLinks {
        /// Entries to import; see `ImportLinkData` for the per-entry fields.
        links: Vec<ImportLinkData>,
        /// Whether existing codes are overwritten instead of reported as conflicts.
        overwrite: bool,
        /// Request streaming progress reports (default false for backward compatibility)
        #[serde(default)]
        stream_progress: bool,
    },

    /// Export all links
    ExportLinks,

    /// Get link statistics
    GetLinkStats,

    // ============ Config Management Commands ============
    /// List all configurations
    ConfigList {
        /// Optional category filter; `None` returns every category.
        category: Option<String>,
    },

    /// Get a single configuration
    ConfigGet {
        /// Configuration key to fetch.
        key: String,
    },

    /// Set a configuration value
    ConfigSet {
        /// Configuration key to update.
        key: String,
        /// New raw string value; normalized and validated by the server.
        value: String,
    },

    /// Reset a configuration to default
    ConfigReset {
        /// Configuration key to reset.
        key: String,
    },

    /// Batch import configurations
    ConfigImport {
        /// Entries to apply in order; see `ConfigImportItem`.
        configs: Vec<ConfigImportItem>,
    },
}

impl IpcCommand {
    /// 返回命令名称（不含敏感字段），用于日志
    pub fn name(&self) -> &'static str {
        match self {
            IpcCommand::Ping => "Ping",
            IpcCommand::Reload { .. } => "Reload",
            IpcCommand::GetStatus => "GetStatus",
            IpcCommand::Shutdown => "Shutdown",
            IpcCommand::AddLink { .. } => "AddLink",
            IpcCommand::RemoveLink { .. } => "RemoveLink",
            IpcCommand::BatchDeleteLinks { .. } => "BatchDeleteLinks",
            IpcCommand::UpdateLink { .. } => "UpdateLink",
            IpcCommand::GetLink { .. } => "GetLink",
            IpcCommand::ListLinks { .. } => "ListLinks",
            IpcCommand::ImportLinks { .. } => "ImportLinks",
            IpcCommand::ExportLinks => "ExportLinks",
            IpcCommand::GetLinkStats => "GetLinkStats",
            IpcCommand::ConfigList { .. } => "ConfigList",
            IpcCommand::ConfigGet { .. } => "ConfigGet",
            IpcCommand::ConfigSet { .. } => "ConfigSet",
            IpcCommand::ConfigReset { .. } => "ConfigReset",
            IpcCommand::ConfigImport { .. } => "ConfigImport",
        }
    }
}

/// IPC responses sent from server to client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IpcResponse {
    /// Heartbeat response
    Pong {
        /// Server version
        version: String,
        /// Server uptime in seconds
        uptime_secs: u64,
    },

    /// Reload operation result
    ReloadResult {
        /// Whether the reload was successful
        success: bool,
        /// What was reloaded
        target: ReloadTarget,
        /// Duration in milliseconds
        duration_ms: u64,
        /// Optional message (error message on failure)
        message: Option<String>,
    },

    /// Current server status
    Status {
        /// Server version
        version: String,
        /// Server uptime in seconds
        uptime_secs: u64,
        /// Whether a reload is currently in progress
        is_reloading: bool,
        /// Last data reload time (ISO8601)
        last_data_reload: Option<String>,
        /// Last config reload time (ISO8601)
        last_config_reload: Option<String>,
        /// Number of links in cache
        links_count: usize,
    },

    /// Shutdown acknowledgment
    ShuttingDown,

    /// Error response
    Error {
        /// Error code
        code: String,
        /// Error message
        message: String,
    },

    // ============ Link Management Responses ============
    /// Link created successfully
    LinkCreated {
        /// The newly created link as persisted.
        link: ShortLink,
        /// Generated code if none was provided
        generated_code: bool,
    },

    /// Link deleted successfully
    LinkDeleted {
        /// Short code that was removed.
        code: String,
    },

    /// Batch delete result
    BatchDeleteResult {
        /// Codes that were deleted.
        deleted: Vec<String>,
        /// Codes that did not exist and were left untouched.
        not_found: Vec<String>,
        /// Per-code failures that prevented deletion.
        errors: Vec<ImportErrorData>,
    },

    /// Link updated successfully
    LinkUpdated {
        /// The link snapshot after applying the update.
        link: ShortLink,
    },

    /// Get link result
    LinkFound {
        /// The matching link, or `None` when the code does not exist.
        link: Option<ShortLink>,
    },

    /// List links result
    LinkList {
        /// Entries of the requested page.
        links: Vec<ShortLink>,
        /// Total number of links matching the filter, across all pages.
        total: usize,
        /// 1-based page number this result corresponds to.
        page: u64,
        /// Page size the result was produced with.
        page_size: u64,
    },

    /// Import result
    ImportResult {
        /// Entries that were written.
        success: usize,
        /// Entries skipped without overwriting (conflicts when `overwrite` is false).
        skipped: usize,
        /// Entries that failed validation or persistence.
        failed: usize,
        /// Failure details, one per failed entry.
        errors: Vec<ImportErrorData>,
    },

    /// Import progress (streaming import)
    ImportProgress {
        /// Phase the import is currently in.
        phase: ImportPhase,
        /// Entries handled so far in this phase, for percentage display.
        processed: usize,
        /// Total entries to process.
        total: usize,
        /// Entries written so far.
        success: usize,
        /// Entries skipped so far.
        skipped: usize,
        /// Entries failed so far.
        failed: usize,
    },

    /// Export result
    ExportResult {
        /// All links, sent in a single message when streaming is not requested.
        links: Vec<ShortLink>,
    },

    /// Export chunk (streaming export)
    ExportChunk {
        /// Batch of links; repeated messages until `ExportDone` arrives.
        links: Vec<ShortLink>,
    },

    /// Export done marker (streaming export)
    ExportDone {
        /// Total number of links across all chunks.
        total: usize,
    },

    /// Stats result
    StatsResult {
        /// Number of links in storage.
        total_links: usize,
        /// Sum of recorded clicks across all links.
        total_clicks: i64,
        /// Number of links that are not expired.
        active_links: usize,
    },

    // ============ Config Management Responses ============
    /// Config list result
    ConfigListResult {
        /// Config entries, optionally filtered by category.
        configs: Vec<ConfigItemData>,
    },

    /// Config get result
    ConfigGetResult {
        /// The requested config entry.
        config: ConfigItemData,
    },

    /// Config set result
    ConfigSetResult {
        /// Configuration key that was updated.
        key: String,
        /// Persisted value after normalization; masked when sensitive.
        value: String,
        /// Whether the change requires a server restart to take effect.
        requires_restart: bool,
        /// Whether the entry is sensitive and its value must be masked.
        is_sensitive: bool,
        /// Previous value; currently always `None` because ConfigService does not expose it.
        old_value: Option<String>,
        /// Optional server note about the change, e.g. an applied action.
        message: Option<String>,
    },

    /// Config reset result
    ConfigResetResult {
        /// Configuration key that was reset.
        key: String,
        /// Restored default value; masked when sensitive.
        value: String,
        /// Whether the reset requires a server restart to take effect.
        requires_restart: bool,
        /// Whether the entry is sensitive and its value must be masked.
        is_sensitive: bool,
        /// Optional server note about the reset.
        message: Option<String>,
    },

    /// Config import result
    ConfigImportResult {
        /// Entries that were applied.
        success: usize,
        /// Entries skipped, e.g. identical to the current value.
        skipped: usize,
        /// Entries that failed validation or persistence.
        failed: usize,
        /// Failure details, one per failed entry.
        errors: Vec<ImportErrorData>,
    },
}

/// IPC connection errors
#[derive(Debug)]
pub enum IpcError {
    /// Server is not running (socket doesn't exist or cannot connect)
    ServerNotRunning,
    /// Connection timeout
    Timeout,
    /// Protocol error (invalid message format)
    ProtocolError(String),
    /// IO error during communication
    IoError(io::Error),
}

impl fmt::Display for IpcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IpcError::ServerNotRunning => write!(f, "Server is not running"),
            IpcError::Timeout => write!(f, "Connection timeout"),
            IpcError::ProtocolError(msg) => write!(f, "Protocol error: {}", msg),
            IpcError::IoError(e) => write!(f, "IO error: {}", e),
        }
    }
}

impl std::error::Error for IpcError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            IpcError::IoError(e) => Some(e),
            _ => None,
        }
    }
}

impl From<io::Error> for IpcError {
    fn from(err: io::Error) -> Self {
        match err.kind() {
            io::ErrorKind::ConnectionRefused | io::ErrorKind::NotFound => {
                IpcError::ServerNotRunning
            }
            _ => IpcError::IoError(err),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn test_ipc_error_display() {
        assert_eq!(
            format!("{}", IpcError::ServerNotRunning),
            "Server is not running"
        );
        assert_eq!(format!("{}", IpcError::Timeout), "Connection timeout");
        assert_eq!(
            format!("{}", IpcError::ProtocolError("bad data".into())),
            "Protocol error: bad data"
        );

        let io_err = io::Error::other("test error");
        let ipc_err = IpcError::IoError(io_err);
        assert!(format!("{}", ipc_err).contains("IO error"));
    }

    #[test]
    fn test_ipc_error_from_io_connection_refused() {
        let io_err = io::Error::new(io::ErrorKind::ConnectionRefused, "refused");
        let ipc_err: IpcError = io_err.into();
        assert!(matches!(ipc_err, IpcError::ServerNotRunning));
    }

    #[test]
    fn test_ipc_error_from_io_not_found() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "not found");
        let ipc_err: IpcError = io_err.into();
        assert!(matches!(ipc_err, IpcError::ServerNotRunning));
    }

    #[test]
    fn test_ipc_error_from_io_other() {
        let io_err = io::Error::other("other error");
        let ipc_err: IpcError = io_err.into();
        assert!(matches!(ipc_err, IpcError::IoError(_)));
    }

    #[test]
    fn test_ipc_error_source() {
        let err = IpcError::ServerNotRunning;
        assert!(err.source().is_none());

        let io_err = io::Error::other("test");
        let err = IpcError::IoError(io_err);
        assert!(err.source().is_some());
    }

    #[test]
    fn test_import_error_data_with_error_code() {
        let data = ImportErrorData {
            code: "dup".into(),
            message: "Link already exists".into(),
            error_code: Some("E021".into()),
        };
        let json = serde_json::to_string(&data).unwrap();
        assert!(json.contains("\"error_code\":\"E021\""));

        let decoded: ImportErrorData = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.error_code, Some("E021".into()));
    }

    #[test]
    fn test_import_error_data_without_error_code() {
        let data = ImportErrorData {
            code: "key".into(),
            message: "unknown".into(),
            error_code: None,
        };
        let json = serde_json::to_string(&data).unwrap();
        assert!(!json.contains("error_code"));

        let json_no_code = r#"{"code":"key","message":"unknown"}"#;
        let decoded: ImportErrorData = serde_json::from_str(json_no_code).unwrap();
        assert_eq!(decoded.error_code, None);
    }
}
