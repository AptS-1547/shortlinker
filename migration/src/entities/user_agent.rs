//! `UserAgent` lookup table entity

use sea_orm::entity::prelude::*;

/// One distinct User-Agent string, keyed by hash with parsed client metadata.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "user_agents")]
pub struct Model {
    /// Primary key: 16-character lowercase hex digest of the xxHash64 of the
    /// UA string; click logs reference this value instead of storing the raw
    /// string.
    #[sea_orm(primary_key, auto_increment = false)]
    pub hash: String, // CHAR(16) xxHash64 hex
    /// Full, unparsed `User-Agent` header value.
    #[sea_orm(column_type = "Text")]
    pub user_agent_string: String,
    /// UTC timestamp of the first request carrying this UA.
    pub first_seen: DateTimeUtc,
    /// UTC timestamp of the most recent request carrying this UA.
    pub last_seen: DateTimeUtc,
    // 解析后的字段
    /// Parsed browser name (e.g. Chrome, Firefox); `None` while unparsed or unrecognized.
    pub browser_name: Option<String>,
    /// Parsed browser major version string; `None` while unparsed or unrecognized.
    pub browser_version: Option<String>,
    /// Parsed operating system name (e.g. macOS, Windows); `None` while unparsed or unrecognized.
    pub os_name: Option<String>,
    /// Parsed operating system version; `None` while unparsed or unrecognized.
    pub os_version: Option<String>,
    /// Parsed device category (e.g. desktop, mobile, tablet); `None` while unparsed or unrecognized.
    pub device_category: Option<String>,
    /// Parsed device vendor when identifiable; `None` otherwise.
    pub device_vendor: Option<String>,
    /// Whether the UA was classified as a bot/crawler; defaults to `false`.
    #[sea_orm(default_value = false)]
    pub is_bot: bool,
}

/// Standalone lookup table; click logs reference it by hash without a
/// declared `SeaORM` relation.
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
