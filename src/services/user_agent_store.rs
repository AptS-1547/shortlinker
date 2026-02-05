//! UserAgent deduplication store with parsing
//!
//! This module provides a high-performance store for UserAgent strings,
//! using xxHash64 to create compact references and woothee for parsing
//! browser/OS/device information.

use chrono::Utc;
use dashmap::{DashMap, DashSet};
use sea_orm::{
    ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    QuerySelect, sea_query::OnConflict,
};
use std::sync::OnceLock;
use tracing::{debug, info, warn};
use woothee::parser::Parser;
use xxhash_rust::xxh64::xxh64;

use migration::entities::{click_log, user_agent};

/// Global UserAgentStore instance
static GLOBAL_UA_STORE: OnceLock<UserAgentStore> = OnceLock::new();

/// Get the global UserAgentStore instance
pub fn get_user_agent_store() -> Option<&'static UserAgentStore> {
    GLOBAL_UA_STORE.get()
}

/// Set the global UserAgentStore instance (called during startup)
pub fn set_global_user_agent_store(store: UserAgentStore) {
    if GLOBAL_UA_STORE.set(store).is_err() {
        warn!("UserAgentStore already initialized");
    }
}

/// Parsed UserAgent information
#[derive(Debug, Clone)]
pub struct ParsedUserAgent {
    pub hash: String,
    pub ua_string: String,
    pub browser_name: Option<String>,
    pub browser_version: Option<String>,
    pub os_name: Option<String>,
    pub os_version: Option<String>,
    pub device_category: Option<String>,
    pub device_vendor: Option<String>,
    pub is_bot: bool,
}

/// UserAgent deduplication store with parsing
///
/// Maintains an in-memory cache of known UA hashes and batches
/// new UA insertions to reduce database pressure.
pub struct UserAgentStore {
    /// Known hashes (already in database)
    known_hashes: DashSet<String>,
    /// Pending inserts: hash -> parsed UA info
    pending_inserts: DashMap<String, ParsedUserAgent>,
}

impl UserAgentStore {
    /// Create a new empty UserAgentStore
    pub fn new() -> Self {
        Self {
            known_hashes: DashSet::new(),
            pending_inserts: DashMap::new(),
        }
    }

    /// Load known hashes from database on startup
    pub async fn load_known_hashes(&self, db: &DatabaseConnection) -> anyhow::Result<usize> {
        let hashes: Vec<user_agent::Model> = user_agent::Entity::find().all(db).await?;

        let count = hashes.len();
        for model in hashes {
            self.known_hashes.insert(model.hash);
        }

        debug!("Loaded {} known UserAgent hashes from database", count);
        Ok(count)
    }

    /// Compute xxHash64 of a string, returning 16-char hex
    #[inline]
    pub fn compute_hash(s: &str) -> String {
        format!("{:016x}", xxh64(s.as_bytes(), 0))
    }

    /// Parse a UserAgent string using woothee
    fn parse_user_agent(ua_string: &str, hash: &str) -> ParsedUserAgent {
        let parser = Parser::new();
        let result = parser.parse(ua_string).unwrap_or_default();

        ParsedUserAgent {
            hash: hash.to_string(),
            ua_string: ua_string.to_string(),
            browser_name: if result.name != "UNKNOWN" {
                Some(result.name.to_string())
            } else {
                None
            },
            browser_version: if !result.version.is_empty() {
                Some(result.version.to_string())
            } else {
                None
            },
            os_name: if result.os != "UNKNOWN" {
                Some(result.os.to_string())
            } else {
                None
            },
            os_version: if !result.os_version.is_empty() {
                Some(result.os_version.to_string())
            } else {
                None
            },
            device_category: Some(result.category.to_string()),
            device_vendor: if result.vendor != "UNKNOWN" {
                Some(result.vendor.to_string())
            } else {
                None
            },
            is_bot: result.category == "crawler",
        }
    }

    /// Get or create hash for a UserAgent string
    ///
    /// If the UA is new, it's parsed and queued for batch insertion.
    /// Returns the 16-char hex hash immediately.
    pub fn get_or_create_hash(&self, user_agent: &str) -> String {
        let hash = Self::compute_hash(user_agent);

        if !self.known_hashes.contains(&hash) {
            // Parse and queue for batch insert
            self.pending_inserts
                .entry(hash.clone())
                .or_insert_with(|| Self::parse_user_agent(user_agent, &hash));
        }

        hash
    }

    /// Get count of pending inserts
    pub fn pending_count(&self) -> usize {
        self.pending_inserts.len()
    }

    /// Flush pending UA inserts to database
    ///
    /// Returns the number of successfully inserted UAs.
    pub async fn flush_pending(&self, db: &DatabaseConnection) -> anyhow::Result<usize> {
        if self.pending_inserts.is_empty() {
            return Ok(0);
        }

        // Collect pending inserts
        let pending: Vec<ParsedUserAgent> = self
            .pending_inserts
            .iter()
            .map(|r| r.value().clone())
            .collect();

        let now = Utc::now();
        let count = pending.len();

        // Build batch of ActiveModels
        let models: Vec<user_agent::ActiveModel> = pending
            .iter()
            .map(|parsed| user_agent::ActiveModel {
                hash: Set(parsed.hash.clone()),
                user_agent_string: Set(parsed.ua_string.clone()),
                first_seen: Set(now),
                last_seen: Set(now),
                browser_name: Set(parsed.browser_name.clone()),
                browser_version: Set(parsed.browser_version.clone()),
                os_name: Set(parsed.os_name.clone()),
                os_version: Set(parsed.os_version.clone()),
                device_category: Set(parsed.device_category.clone()),
                device_vendor: Set(parsed.device_vendor.clone()),
                is_bot: Set(parsed.is_bot),
            })
            .collect();

        // Batch insert with on_conflict do_nothing (skip duplicates)
        if let Err(e) = user_agent::Entity::insert_many(models)
            .on_conflict(
                OnConflict::column(user_agent::Column::Hash)
                    .do_nothing()
                    .to_owned(),
            )
            .exec(db)
            .await
        {
            // on_conflict do_nothing may produce "no rows inserted" errors which are expected
            debug!("Batch insert UserAgents: {} (duplicates skipped)", e);
        }

        // Mark all as known and clear pending
        for parsed in &pending {
            self.known_hashes.insert(parsed.hash.clone());
            self.pending_inserts.remove(&parsed.hash);
        }

        if count > 0 {
            debug!("Flushed {} UserAgents to database", count);
        }

        Ok(count)
    }

    /// Get the count of known hashes
    pub fn known_count(&self) -> usize {
        self.known_hashes.len()
    }

    /// Migrate existing click_logs user_agent data to the new hash-based system
    ///
    /// This should be called once during startup to migrate historical data.
    /// It processes records in batches to avoid memory issues.
    pub async fn migrate_historical_data(
        &self,
        db: &DatabaseConnection,
    ) -> anyhow::Result<MigrationStats> {
        let batch_size = 1000u64;
        let mut stats = MigrationStats::default();

        // Check if migration is needed by counting records with user_agent but no user_agent_hash
        let pending_count = click_log::Entity::find()
            .filter(click_log::Column::UserAgent.is_not_null())
            .filter(click_log::Column::UserAgentHash.is_null())
            .count(db)
            .await?;

        if pending_count == 0 {
            debug!("No historical UserAgent data to migrate");
            return Ok(stats);
        }

        info!(
            "Starting UserAgent migration for {} click_logs records...",
            pending_count
        );

        let now = Utc::now();

        loop {
            // Fetch batch of records needing migration
            let records: Vec<click_log::Model> = click_log::Entity::find()
                .filter(click_log::Column::UserAgent.is_not_null())
                .filter(click_log::Column::UserAgentHash.is_null())
                .limit(batch_size)
                .all(db)
                .await?;

            if records.is_empty() {
                break;
            }

            for record in &records {
                if let Some(ref ua_string) = record.user_agent {
                    if ua_string.is_empty() {
                        continue;
                    }

                    let hash = Self::compute_hash(ua_string);

                    // Insert into user_agents if not exists
                    if !self.known_hashes.contains(&hash) {
                        let existing = user_agent::Entity::find()
                            .filter(user_agent::Column::Hash.eq(&hash))
                            .one(db)
                            .await?;

                        if existing.is_none() {
                            // Parse and insert with all fields
                            let parsed = Self::parse_user_agent(ua_string, &hash);
                            let ua_model = user_agent::ActiveModel {
                                hash: Set(hash.clone()),
                                user_agent_string: Set(ua_string.clone()),
                                first_seen: Set(now),
                                last_seen: Set(now),
                                browser_name: Set(parsed.browser_name),
                                browser_version: Set(parsed.browser_version),
                                os_name: Set(parsed.os_name),
                                os_version: Set(parsed.os_version),
                                device_category: Set(parsed.device_category),
                                device_vendor: Set(parsed.device_vendor),
                                is_bot: Set(parsed.is_bot),
                            };

                            if user_agent::Entity::insert(ua_model).exec(db).await.is_ok() {
                                stats.unique_uas_inserted += 1;
                            }
                        }

                        self.known_hashes.insert(hash.clone());
                    }

                    // Update click_log with hash
                    let mut active: click_log::ActiveModel = record.clone().into();
                    active.user_agent_hash = Set(Some(hash));

                    if click_log::Entity::update(active).exec(db).await.is_ok() {
                        stats.records_updated += 1;
                    }
                }
            }

            stats.batches_processed += 1;

            if records.len() < batch_size as usize {
                break;
            }
        }

        info!(
            "UserAgent migration completed: {} records updated, {} unique UAs inserted",
            stats.records_updated, stats.unique_uas_inserted
        );

        Ok(stats)
    }

    /// Backfill parsed fields for existing user_agents records that have NULL browser_name
    ///
    /// This handles records that were inserted before the parsing feature was added.
    pub async fn backfill_parsed_fields(&self, db: &DatabaseConnection) -> anyhow::Result<usize> {
        let batch_size = 500u64;
        let mut backfilled = 0;

        // Find records with user_agent_string but NULL browser_name (indicating unparsed)
        let pending_count = user_agent::Entity::find()
            .filter(user_agent::Column::BrowserName.is_null())
            .count(db)
            .await?;

        if pending_count == 0 {
            debug!("No UserAgent records need backfill");
            return Ok(0);
        }

        info!(
            "Backfilling parsed fields for {} existing UserAgent records...",
            pending_count
        );

        loop {
            let records: Vec<user_agent::Model> = user_agent::Entity::find()
                .filter(user_agent::Column::BrowserName.is_null())
                .limit(batch_size)
                .all(db)
                .await?;

            if records.is_empty() {
                break;
            }

            for record in &records {
                let parsed = Self::parse_user_agent(&record.user_agent_string, &record.hash);

                let mut active: user_agent::ActiveModel = record.clone().into();
                active.browser_name = Set(parsed.browser_name);
                active.browser_version = Set(parsed.browser_version);
                active.os_name = Set(parsed.os_name);
                active.os_version = Set(parsed.os_version);
                active.device_category = Set(parsed.device_category);
                active.device_vendor = Set(parsed.device_vendor);
                active.is_bot = Set(parsed.is_bot);

                if user_agent::Entity::update(active).exec(db).await.is_ok() {
                    backfilled += 1;
                }
            }

            if records.len() < batch_size as usize {
                break;
            }
        }

        if backfilled > 0 {
            info!(
                "Backfilled parsed fields for {} UserAgent records",
                backfilled
            );
        }

        Ok(backfilled)
    }
}

/// Statistics from historical data migration
#[derive(Debug, Default)]
pub struct MigrationStats {
    pub records_updated: usize,
    pub unique_uas_inserted: usize,
    pub batches_processed: usize,
}

impl Default for UserAgentStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_hash() {
        let ua = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36";
        let hash = UserAgentStore::compute_hash(ua);

        // Hash should be 16 hex characters
        assert_eq!(hash.len(), 16);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));

        // Same input should produce same hash
        assert_eq!(hash, UserAgentStore::compute_hash(ua));

        // Different input should produce different hash
        let ua2 = "Mozilla/5.0 (Windows NT 10.0; Win64; x64)";
        assert_ne!(hash, UserAgentStore::compute_hash(ua2));
    }

    #[test]
    fn test_get_or_create_hash() {
        let store = UserAgentStore::new();
        let ua = "Test/1.0";

        // First call should add to pending
        let hash1 = store.get_or_create_hash(ua);
        assert_eq!(store.pending_count(), 1);

        // Same UA should return same hash but not duplicate pending
        let hash2 = store.get_or_create_hash(ua);
        assert_eq!(hash1, hash2);
        assert_eq!(store.pending_count(), 1);

        // Different UA should add another pending entry
        store.get_or_create_hash("Other/2.0");
        assert_eq!(store.pending_count(), 2);
    }

    #[test]
    fn test_known_hashes() {
        let store = UserAgentStore::new();
        let ua = "Test/1.0";
        let hash = UserAgentStore::compute_hash(ua);

        // Pre-mark as known
        store.known_hashes.insert(hash.clone());

        // Should not add to pending since it's known
        let result = store.get_or_create_hash(ua);
        assert_eq!(result, hash);
        assert_eq!(store.pending_count(), 0);
    }

    #[test]
    fn test_parse_user_agent_chrome() {
        let ua = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";
        let parsed = UserAgentStore::parse_user_agent(ua, "test_hash");

        assert_eq!(parsed.browser_name, Some("Chrome".to_string()));
        assert!(parsed.browser_version.is_some());
        assert_eq!(parsed.os_name, Some("Windows 10".to_string()));
        assert_eq!(parsed.device_category, Some("pc".to_string()));
        assert!(!parsed.is_bot);
    }

    #[test]
    fn test_parse_user_agent_iphone() {
        let ua = "Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Mobile/15E148 Safari/604.1";
        let parsed = UserAgentStore::parse_user_agent(ua, "test_hash");

        assert_eq!(parsed.browser_name, Some("Safari".to_string()));
        assert_eq!(parsed.os_name, Some("iPhone".to_string()));
        assert_eq!(parsed.device_category, Some("smartphone".to_string()));
        assert!(!parsed.is_bot);
    }

    #[test]
    fn test_parse_user_agent_googlebot() {
        let ua = "Mozilla/5.0 (compatible; Googlebot/2.1; +http://www.google.com/bot.html)";
        let parsed = UserAgentStore::parse_user_agent(ua, "test_hash");

        assert_eq!(parsed.browser_name, Some("Googlebot".to_string()));
        assert_eq!(parsed.device_category, Some("crawler".to_string()));
        assert!(parsed.is_bot);
    }
}
