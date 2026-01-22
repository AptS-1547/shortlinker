//! Link management service
//!
//! Provides unified business logic for link operations, shared between
//! IPC handlers and HTTP handlers.

use std::fmt;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use tracing::{error, info};

use crate::cache::traits::CompositeCacheTrait;
use crate::config::get_config;
use crate::storage::{LinkFilter, SeaOrmStorage, ShortLink};
use crate::utils::TimeParser;
use crate::utils::generate_random_code;
use crate::utils::password::{hash_password, is_argon2_hash};
use crate::utils::url_validator::validate_url;

// ============ Error Types ============

/// Service layer errors
#[derive(Debug)]
pub enum ServiceError {
    /// Invalid URL format
    InvalidUrl(String),
    /// Invalid expiration time format
    InvalidExpireTime(String),
    /// Password hashing failed
    PasswordHashError,
    /// Resource not found
    NotFound(String),
    /// Resource already exists (conflict)
    Conflict(String),
    /// Database operation failed
    DatabaseError(String),
    /// Storage not initialized
    NotInitialized,
}

impl fmt::Display for ServiceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ServiceError::InvalidUrl(msg) => write!(f, "Invalid URL: {}", msg),
            ServiceError::InvalidExpireTime(msg) => write!(f, "Invalid expiration time: {}", msg),
            ServiceError::PasswordHashError => write!(f, "Failed to process password"),
            ServiceError::NotFound(msg) => write!(f, "Not found: {}", msg),
            ServiceError::Conflict(msg) => write!(f, "Conflict: {}", msg),
            ServiceError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            ServiceError::NotInitialized => write!(f, "Service not initialized"),
        }
    }
}

impl std::error::Error for ServiceError {}

/// Service error codes for protocol conversion
impl ServiceError {
    /// Get error code string for IPC/HTTP responses
    pub fn code(&self) -> &'static str {
        match self {
            ServiceError::InvalidUrl(_) => "INVALID_URL",
            ServiceError::InvalidExpireTime(_) => "INVALID_EXPIRE_TIME",
            ServiceError::PasswordHashError => "HASH_ERROR",
            ServiceError::NotFound(_) => "NOT_FOUND",
            ServiceError::Conflict(_) => "CONFLICT",
            ServiceError::DatabaseError(_) => "DATABASE_ERROR",
            ServiceError::NotInitialized => "NOT_INITIALIZED",
        }
    }
}

// ============ Request/Response DTOs ============

/// Request to create a new link
#[derive(Debug, Clone)]
pub struct CreateLinkRequest {
    /// Short code (optional, will be generated if not provided)
    pub code: Option<String>,
    /// Target URL
    pub target: String,
    /// Force overwrite if exists
    pub force: bool,
    /// Expiration time (flexible format: RFC3339, relative like "1d", "2h")
    pub expires_at: Option<String>,
    /// Password protection (plaintext or already hashed)
    pub password: Option<String>,
}

/// Request to update an existing link
#[derive(Debug, Clone)]
pub struct UpdateLinkRequest {
    /// New target URL
    pub target: String,
    /// New expiration time (None = keep existing)
    pub expires_at: Option<String>,
    /// New password (None = keep existing, Some("") = remove)
    pub password: Option<String>,
}

/// Result of link creation
#[derive(Debug, Clone)]
pub struct LinkCreateResult {
    /// The created/updated link
    pub link: ShortLink,
    /// Whether the code was auto-generated
    pub generated_code: bool,
}

/// Single import item
#[derive(Debug, Clone)]
pub struct ImportLinkItem {
    pub code: String,
    pub target: String,
    pub expires_at: Option<String>,
    pub password: Option<String>,
}

/// Import conflict resolution mode
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ImportMode {
    /// Skip existing links
    #[default]
    Skip,
    /// Overwrite existing links
    Overwrite,
    /// Return error for existing links
    Error,
}

impl ImportMode {
    /// Convert from IPC's boolean overwrite flag
    pub fn from_overwrite_flag(overwrite: bool) -> Self {
        if overwrite {
            ImportMode::Overwrite
        } else {
            ImportMode::Skip
        }
    }
}

/// Result of import operation
#[derive(Debug, Clone, Default)]
pub struct ImportResult {
    pub success: usize,
    pub skipped: usize,
    pub failed: usize,
    pub errors: Vec<ImportError>,
}

/// Single import error
#[derive(Debug, Clone)]
pub struct ImportError {
    pub code: String,
    pub message: String,
}

// ============ LinkService Implementation ============

/// Service for link management operations
///
/// This service encapsulates all business logic for link CRUD operations,
/// ensuring consistent behavior across IPC and HTTP interfaces.
pub struct LinkService {
    storage: Arc<SeaOrmStorage>,
    cache: Arc<dyn CompositeCacheTrait>,
}

impl LinkService {
    /// Create a new LinkService instance
    pub fn new(storage: Arc<SeaOrmStorage>, cache: Arc<dyn CompositeCacheTrait>) -> Self {
        Self { storage, cache }
    }

    /// Get the configured random code length
    fn random_code_length(&self) -> usize {
        get_config().features.random_code_length
    }

    /// Get the default cache TTL
    fn default_cache_ttl(&self) -> u64 {
        get_config().cache.default_ttl
    }

    /// Process password field - hash if needed
    fn process_password(&self, password: Option<&str>) -> Result<Option<String>, ServiceError> {
        match password {
            Some(pwd) if !pwd.is_empty() => {
                if is_argon2_hash(pwd) {
                    Ok(Some(pwd.to_string()))
                } else {
                    hash_password(pwd).map(Some).map_err(|e| {
                        error!("Failed to hash password: {}", e);
                        ServiceError::PasswordHashError
                    })
                }
            }
            _ => Ok(None),
        }
    }

    /// Parse expiration time from flexible format
    fn parse_expires_at(
        &self,
        expires_at: Option<&str>,
    ) -> Result<Option<DateTime<Utc>>, ServiceError> {
        match expires_at {
            Some(s) if !s.is_empty() => TimeParser::parse_expire_time(s)
                .map(Some)
                .map_err(|e| ServiceError::InvalidExpireTime(e.to_string())),
            _ => Ok(None),
        }
    }

    /// Update cache with a link
    async fn update_cache(&self, link: &ShortLink) {
        let ttl = link.cache_ttl(self.default_cache_ttl());
        self.cache.insert(&link.code, link.clone(), ttl).await;
    }

    // ============ CRUD Operations ============

    /// Create a new short link
    pub async fn create_link(
        &self,
        req: CreateLinkRequest,
    ) -> Result<LinkCreateResult, ServiceError> {
        // Validate URL
        validate_url(&req.target).map_err(|e| ServiceError::InvalidUrl(e.to_string()))?;

        // Generate code if not provided
        let (code, generated) = match req.code.filter(|c| !c.is_empty()) {
            Some(c) => (c, false),
            None => (generate_random_code(self.random_code_length()), true),
        };

        // Check if code already exists
        let existing = self.storage.get(&code).await.map_err(|e| {
            ServiceError::DatabaseError(format!("Failed to check existing link: {}", e))
        })?;

        if existing.is_some() && !req.force {
            return Err(ServiceError::Conflict(format!(
                "Code '{}' already exists. Use force=true to overwrite.",
                code
            )));
        }

        // Parse expiration time
        let expires_at = self.parse_expires_at(req.expires_at.as_deref())?;

        // Process password
        let password = self.process_password(req.password.as_deref())?;

        // Preserve original created_at and click count if overwriting
        let (created_at, click) = if let Some(ref existing_link) = existing {
            (existing_link.created_at, existing_link.click)
        } else {
            (Utc::now(), 0)
        };

        let new_link = ShortLink {
            code: code.clone(),
            target: req.target,
            created_at,
            expires_at,
            password,
            click,
        };

        // Save to storage
        self.storage
            .set(new_link.clone())
            .await
            .map_err(|e| ServiceError::DatabaseError(format!("Failed to save link: {}", e)))?;

        // Update cache
        self.update_cache(&new_link).await;

        let action = if existing.is_some() {
            "overwrote"
        } else {
            "created"
        };
        info!(
            "LinkService: {} link '{}' -> '{}'",
            action, new_link.code, new_link.target
        );

        Ok(LinkCreateResult {
            link: new_link,
            generated_code: generated,
        })
    }

    /// Update an existing link
    pub async fn update_link(
        &self,
        code: &str,
        req: UpdateLinkRequest,
    ) -> Result<ShortLink, ServiceError> {
        // Validate URL
        validate_url(&req.target).map_err(|e| ServiceError::InvalidUrl(e.to_string()))?;

        // Get existing link
        let existing = self
            .storage
            .get(code)
            .await
            .map_err(|e| ServiceError::DatabaseError(format!("Failed to get link: {}", e)))?
            .ok_or_else(|| ServiceError::NotFound(format!("Link '{}' not found", code)))?;

        // Parse expiration time (None = keep existing)
        let expires_at = if req.expires_at.is_some() {
            self.parse_expires_at(req.expires_at.as_deref())?
        } else {
            existing.expires_at
        };

        // Process password
        let password = match req.password.as_deref() {
            Some(pwd) if !pwd.is_empty() => self.process_password(Some(pwd))?,
            Some(_) => None,                   // Empty string = remove password
            None => existing.password.clone(), // Not provided = keep existing
        };

        let updated_link = ShortLink {
            code: code.to_string(),
            target: req.target,
            created_at: existing.created_at,
            expires_at,
            password,
            click: existing.click,
        };

        // Save to storage
        self.storage
            .set(updated_link.clone())
            .await
            .map_err(|e| ServiceError::DatabaseError(format!("Failed to update link: {}", e)))?;

        // Update cache
        self.update_cache(&updated_link).await;

        info!("LinkService: updated '{}'", code);
        Ok(updated_link)
    }

    /// Delete a link
    pub async fn delete_link(&self, code: &str) -> Result<(), ServiceError> {
        self.storage
            .remove(code)
            .await
            .map_err(|e| ServiceError::DatabaseError(format!("Failed to remove link: {}", e)))?;

        self.cache.remove(code).await;

        info!("LinkService: deleted '{}'", code);
        Ok(())
    }

    /// Get a single link
    pub async fn get_link(&self, code: &str) -> Result<Option<ShortLink>, ServiceError> {
        self.storage
            .get(code)
            .await
            .map_err(|e| ServiceError::DatabaseError(format!("Failed to get link: {}", e)))
    }

    /// List links with pagination and filtering
    pub async fn list_links(
        &self,
        filter: LinkFilter,
        page: u64,
        page_size: u64,
    ) -> Result<(Vec<ShortLink>, u64), ServiceError> {
        let page = page.max(1);
        let page_size = page_size.clamp(1, 100);

        self.storage
            .load_paginated_filtered(page, page_size, filter)
            .await
            .map_err(|e| ServiceError::DatabaseError(format!("Failed to list links: {}", e)))
    }

    /// Get link statistics
    pub async fn get_stats(&self) -> Result<crate::storage::LinkStats, ServiceError> {
        self.storage
            .get_stats()
            .await
            .map_err(|e| ServiceError::DatabaseError(format!("Failed to get stats: {}", e)))
    }

    // ============ Batch Operations ============

    /// Import multiple links
    pub async fn import_links(
        &self,
        links: Vec<ImportLinkItem>,
        mode: ImportMode,
    ) -> Result<ImportResult, ServiceError> {
        let mut result = ImportResult::default();

        // Step 1: Validate URLs and collect codes
        struct ValidatedItem {
            code: String,
            target: String,
            expires_at: Option<String>,
            password: Option<String>,
        }

        let mut codes_to_check: Vec<String> = Vec::new();
        let mut valid_items: Vec<ValidatedItem> = Vec::new();

        for item in links {
            // Validate URL first
            if let Err(e) = validate_url(&item.target) {
                result.failed += 1;
                result.errors.push(ImportError {
                    code: item.code,
                    message: e.to_string(),
                });
                continue;
            }

            codes_to_check.push(item.code.clone());
            valid_items.push(ValidatedItem {
                code: item.code,
                target: item.target,
                expires_at: item.expires_at,
                password: item.password,
            });
        }

        // Step 2: Batch fetch existing links (1 query instead of N)
        let codes_refs: Vec<&str> = codes_to_check.iter().map(|s| s.as_str()).collect();
        let existing_map = self.storage.batch_get(&codes_refs).await.map_err(|e| {
            ServiceError::DatabaseError(format!("Failed to batch check existing links: {}", e))
        })?;

        // Step 3: Process each item with in-memory existence check
        for item in valid_items {
            let existing = existing_map.get(&item.code);

            // Handle existence based on mode
            if existing.is_some() {
                match mode {
                    ImportMode::Skip => {
                        result.skipped += 1;
                        continue;
                    }
                    ImportMode::Error => {
                        result.failed += 1;
                        result.errors.push(ImportError {
                            code: item.code,
                            message: "Already exists".to_string(),
                        });
                        continue;
                    }
                    ImportMode::Overwrite => {
                        // Continue processing
                    }
                }
            }

            // Parse expiration time
            let expires_at = match self.parse_expires_at(item.expires_at.as_deref()) {
                Ok(dt) => dt,
                Err(e) => {
                    result.failed += 1;
                    result.errors.push(ImportError {
                        code: item.code,
                        message: e.to_string(),
                    });
                    continue;
                }
            };

            // Process password
            let password = match self.process_password(item.password.as_deref()) {
                Ok(pwd) => pwd,
                Err(_) => {
                    result.failed += 1;
                    result.errors.push(ImportError {
                        code: item.code,
                        message: "Failed to hash password".to_string(),
                    });
                    continue;
                }
            };

            // Preserve created_at and click if overwriting
            let (created_at, click) = if let Some(existing_link) = existing {
                (existing_link.created_at, existing_link.click)
            } else {
                (Utc::now(), 0)
            };

            let new_link = ShortLink {
                code: item.code.clone(),
                target: item.target,
                created_at,
                expires_at,
                password,
                click,
            };

            // Save to storage
            if let Err(e) = self.storage.set(new_link.clone()).await {
                result.failed += 1;
                result.errors.push(ImportError {
                    code: item.code,
                    message: format!("Failed to save: {}", e),
                });
                continue;
            }

            // Update cache
            self.update_cache(&new_link).await;
            result.success += 1;
        }

        info!(
            "LinkService: imported {} links, {} skipped, {} failed",
            result.success, result.skipped, result.failed
        );

        Ok(result)
    }

    /// Export all links
    pub async fn export_links(&self) -> Result<Vec<ShortLink>, ServiceError> {
        let links_map = self
            .storage
            .load_all()
            .await
            .map_err(|e| ServiceError::DatabaseError(format!("Failed to load links: {}", e)))?;

        let links: Vec<ShortLink> = links_map.into_values().collect();
        info!("LinkService: exported {} links", links.len());
        Ok(links)
    }

    /// Export links with filter
    pub async fn export_links_filtered(
        &self,
        filter: LinkFilter,
    ) -> Result<Vec<ShortLink>, ServiceError> {
        self.storage.load_all_filtered(filter).await.map_err(|e| {
            ServiceError::DatabaseError(format!("Failed to load filtered links: {}", e))
        })
    }
}
