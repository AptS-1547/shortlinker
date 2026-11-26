//! Click log entity for detailed click tracking

use sea_orm::entity::prelude::*;

/// A single click event stored in the `click_logs` table.
///
/// One row per redirect serving a click detail log; retention and rollup
/// eviction are governed by the analytics retention settings.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "click_logs")]
pub struct Model {
    /// Auto-incrementing primary key; also used as the cursor for
    /// cursor-paginated log export.
    #[sea_orm(primary_key)]
    pub id: i64,
    /// Short code of the link that was clicked.
    pub short_code: String,
    /// UTC timestamp of the redirect response.
    pub clicked_at: DateTimeUtc,
    /// Raw `Referer` header of the request; `None` when the browser sent none.
    #[sea_orm(column_type = "Text", nullable)]
    pub referrer: Option<String>,
    /// Client IP address as seen by the server; `None` when unavailable.
    pub ip_address: Option<String>,
    /// ISO 3166-1 alpha-2 country code resolved via `GeoIP`; `None` when lookup failed or is disabled.
    pub country: Option<String>,
    /// City name resolved via `GeoIP`; `None` when lookup failed or is disabled.
    pub city: Option<String>,
    /// Traffic source (`utm_source` param, `ref:{domain}`, or direct)
    pub source: Option<String>,
    /// `UserAgent` hash (references `user_agents.hash`)
    pub user_agent_hash: Option<String>,
}

/// No foreign-key relations are declared; `user_agent_hash` is joined
/// manually by the analytics queries.
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
