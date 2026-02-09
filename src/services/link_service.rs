//! Link management service
//!
//! Provides unified business logic for link operations, shared between
//! IPC handlers and HTTP handlers.

use std::collections::HashSet;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::{error, info};
use ts_rs::TS;

use crate::cache::traits::CompositeCacheTrait;
use crate::config::{get_config, keys, try_get_runtime_config};
use crate::errors::ShortlinkerError;
use crate::storage::{LinkFilter, SeaOrmStorage, ShortLink};
use crate::utils::TimeParser;
use crate::utils::generate_random_code;
use crate::utils::password::{process_imported_password, process_new_password};
use crate::utils::url_validator::validate_url;

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
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../admin-panel/src/services/types.generated.ts")]
#[serde(rename_all = "lowercase")]
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

#[cfg(test)]
mod import_mode_tests {
    use super::*;

    #[test]
    fn test_import_mode_from_overwrite_flag() {
        assert_eq!(ImportMode::from_overwrite_flag(true), ImportMode::Overwrite);
        assert_eq!(ImportMode::from_overwrite_flag(false), ImportMode::Skip);
    }

    #[test]
    fn test_import_mode_default() {
        assert_eq!(ImportMode::default(), ImportMode::Skip);
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

// ============ Batch Operation DTOs ============

/// Single successful batch operation item
#[derive(Debug, Clone)]
pub struct BatchSuccessItem {
    pub code: String,
    pub link: ShortLink,
}

/// Single failed batch operation item
#[derive(Debug, Clone)]
pub struct BatchFailedItem {
    pub code: String,
    pub reason: String,
}

/// Result of batch create/update operation
#[derive(Debug, Clone, Default)]
pub struct BatchOperationResult {
    pub success: Vec<BatchSuccessItem>,
    pub failed: Vec<BatchFailedItem>,
}

/// Result of batch delete operation
#[derive(Debug, Clone, Default)]
pub struct BatchDeleteResult {
    pub deleted: Vec<String>,
    pub not_found: Vec<String>,
    pub errors: Vec<BatchFailedItem>,
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
        try_get_runtime_config()
            .and_then(|rt| rt.get_usize(keys::FEATURES_RANDOM_CODE_LENGTH))
            .unwrap_or(6)
    }

    /// Get the default cache TTL
    fn default_cache_ttl(&self) -> u64 {
        get_config().cache.default_ttl
    }

    /// Process password field - always hash, never accept pre-hashed values
    fn process_password(&self, password: Option<&str>) -> Result<Option<String>, ShortlinkerError> {
        process_new_password(password).map_err(|e| {
            error!("Failed to hash password: {}", e);
            ShortlinkerError::link_password_hash_error(e.to_string())
        })
    }

    /// Parse expiration time from flexible format
    fn parse_expires_at(
        &self,
        expires_at: Option<&str>,
    ) -> Result<Option<DateTime<Utc>>, ShortlinkerError> {
        match expires_at {
            Some(s) if !s.is_empty() => TimeParser::parse_expire_time(s)
                .map(Some)
                .map_err(|e| ShortlinkerError::link_invalid_expire_time(e.to_string())),
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
    ) -> Result<LinkCreateResult, ShortlinkerError> {
        // Validate URL
        validate_url(&req.target).map_err(|e| ShortlinkerError::link_invalid_url(e.to_string()))?;

        // Generate code if not provided, or validate user-provided code
        let (code, generated) = match req.code.filter(|c| !c.is_empty()) {
            Some(c) => {
                // Validate short code format
                if !crate::utils::is_valid_short_code(&c) {
                    return Err(ShortlinkerError::link_invalid_code(format!(
                        "Invalid short code '{}'. Only alphanumeric, underscore, hyphen, dot, and slash allowed.",
                        c
                    )));
                }
                // Check reserved route conflicts (reads from RuntimeConfig)
                if crate::utils::is_reserved_short_code(&c) {
                    return Err(ShortlinkerError::link_reserved_code(format!(
                        "Short code '{}' conflicts with reserved routes",
                        c
                    )));
                }
                (c, false)
            }
            None => (generate_random_code(self.random_code_length()), true),
        };

        // Check if code already exists
        let existing = self.storage.get(&code).await.map_err(|e| {
            ShortlinkerError::database_operation(format!("Failed to check existing link: {}", e))
        })?;

        if existing.is_some() && !req.force {
            return Err(ShortlinkerError::link_already_exists(format!(
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
        self.storage.set(new_link.clone()).await.map_err(|e| {
            ShortlinkerError::database_operation(format!("Failed to save link: {}", e))
        })?;

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
    ) -> Result<ShortLink, ShortlinkerError> {
        // Validate URL
        validate_url(&req.target).map_err(|e| ShortlinkerError::link_invalid_url(e.to_string()))?;

        // Get existing link
        let existing = self
            .storage
            .get(code)
            .await
            .map_err(|e| {
                ShortlinkerError::database_operation(format!("Failed to get link: {}", e))
            })?
            .ok_or_else(|| ShortlinkerError::not_found(format!("Link '{}' not found", code)))?;

        // Parse expiration time (None = keep existing)
        let expires_at = if req.expires_at.is_some() {
            self.parse_expires_at(req.expires_at.as_deref())?
        } else {
            existing.expires_at
        };

        // Process password using the shared utility function
        let password = crate::utils::password::process_update_password(
            req.password.as_deref(),
            existing.password.clone(),
        )
        .map_err(|e| {
            error!("Failed to hash password: {}", e);
            ShortlinkerError::link_password_hash_error(e.to_string())
        })?;

        let updated_link = ShortLink {
            code: code.to_string(),
            target: req.target,
            created_at: existing.created_at,
            expires_at,
            password,
            click: existing.click,
        };

        // Save to storage
        self.storage.set(updated_link.clone()).await.map_err(|e| {
            ShortlinkerError::database_operation(format!("Failed to update link: {}", e))
        })?;

        // Update cache
        self.update_cache(&updated_link).await;

        info!("LinkService: updated '{}'", code);
        Ok(updated_link)
    }

    /// Delete a link
    pub async fn delete_link(&self, code: &str) -> Result<(), ShortlinkerError> {
        self.storage.remove(code).await.map_err(|e| {
            ShortlinkerError::database_operation(format!("Failed to remove link: {}", e))
        })?;

        self.cache.remove(code).await;

        info!("LinkService: deleted '{}'", code);
        Ok(())
    }

    /// Get a single link
    pub async fn get_link(&self, code: &str) -> Result<Option<ShortLink>, ShortlinkerError> {
        self.storage
            .get(code)
            .await
            .map_err(|e| ShortlinkerError::database_operation(format!("Failed to get link: {}", e)))
    }

    /// List links with pagination and filtering
    pub async fn list_links(
        &self,
        filter: LinkFilter,
        page: u64,
        page_size: u64,
    ) -> Result<(Vec<ShortLink>, u64), ShortlinkerError> {
        let page = page.max(1);
        let page_size = page_size.clamp(1, 100);

        self.storage
            .load_paginated_filtered(page, page_size, filter)
            .await
            .map_err(|e| {
                ShortlinkerError::database_operation(format!("Failed to list links: {}", e))
            })
    }

    /// Get link statistics
    pub async fn get_stats(&self) -> Result<crate::storage::LinkStats, ShortlinkerError> {
        self.storage.get_stats().await.map_err(|e| {
            ShortlinkerError::database_operation(format!("Failed to get stats: {}", e))
        })
    }

    // ============ Batch Operations ============

    /// Import multiple links
    pub async fn import_links(
        &self,
        links: Vec<ImportLinkItem>,
        mode: ImportMode,
    ) -> Result<ImportResult, ShortlinkerError> {
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
            ShortlinkerError::database_operation(format!(
                "Failed to batch check existing links: {}",
                e
            ))
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

            // Process password (import path: preserve existing hashes)
            let password = match process_imported_password(item.password.as_deref()) {
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
    pub async fn export_links(&self) -> Result<Vec<ShortLink>, ShortlinkerError> {
        let links_map = self.storage.load_all().await.map_err(|e| {
            ShortlinkerError::database_operation(format!("Failed to load links: {}", e))
        })?;

        let links: Vec<ShortLink> = links_map.into_values().collect();
        info!("LinkService: exported {} links", links.len());
        Ok(links)
    }

    /// 批量导入链接（带 Bloom filter 优化）
    ///
    /// 相比现有 `import_links()`，增加 Bloom filter 预筛选：
    /// - Skip/Error 模式：使用 Bloom 预筛选 + 精确查询（减少 DB 查询）
    /// - Overwrite 模式：跳过 Bloom，直接批量插入
    pub async fn import_links_with_bloom(
        &self,
        links: Vec<ImportLinkItem>,
        mode: ImportMode,
    ) -> Result<ImportResult, ShortlinkerError> {
        let mut result = ImportResult::default();

        // Step 1: 验证 URL 并收集 codes
        let mut valid_items = Vec::new();
        let mut codes_to_check = Vec::new();

        for item in links {
            if let Err(e) = validate_url(&item.target) {
                result.failed += 1;
                result.errors.push(ImportError {
                    code: item.code,
                    message: e.to_string(),
                });
                continue;
            }
            codes_to_check.push(item.code.clone());
            valid_items.push(item);
        }

        // Step 2: 冲突检测（Bloom 优化）
        let existing_codes: HashSet<String> = match mode {
            ImportMode::Overwrite => HashSet::new(),
            ImportMode::Skip | ImportMode::Error => {
                // Bloom Filter 预筛选：false = 一定不存在，跳过 DB 查询
                let mut maybe_exist = Vec::new();
                for code in &codes_to_check {
                    if self.cache.bloom_check(code).await {
                        maybe_exist.push(code.clone());
                    }
                }

                // 只对 Bloom 返回"可能存在"的 codes 精确查询
                if maybe_exist.is_empty() {
                    HashSet::new()
                } else {
                    self.storage
                        .batch_check_codes_exist(&maybe_exist)
                        .await
                        .map_err(|e| {
                            ShortlinkerError::database_operation(format!(
                                "Failed to check existing codes: {}",
                                e
                            ))
                        })?
                }
            }
        };

        // Step 3: 处理每个 item
        let mut links_to_insert = Vec::new();
        let mut processed_codes = HashSet::new();

        for item in valid_items {
            let exists =
                existing_codes.contains(&item.code) || processed_codes.contains(&item.code);

            if exists {
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
                        // 继续处理
                    }
                }
            }

            // 解析 expires_at
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

            // 处理密码
            let password = match process_imported_password(item.password.as_deref()) {
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

            let new_link = ShortLink {
                code: item.code.clone(),
                target: item.target,
                created_at: Utc::now(),
                expires_at,
                password,
                click: 0,
            };

            processed_codes.insert(item.code.clone());
            links_to_insert.push(new_link);
        }

        // Step 4: 批量插入 + 缓存更新
        if !links_to_insert.is_empty() {
            self.storage
                .batch_set(links_to_insert.clone())
                .await
                .map_err(|e| {
                    ShortlinkerError::database_operation(format!("Failed to batch insert: {}", e))
                })?;

            for link in &links_to_insert {
                self.update_cache(link).await;
            }
            result.success = links_to_insert.len();
        }

        info!(
            "LinkService: imported {} links, {} skipped, {} failed",
            result.success, result.skipped, result.failed
        );

        Ok(result)
    }

    /// Batch create links
    ///
    /// Creates multiple links in a single operation. Each link is validated
    /// and processed independently - failures do not affect other links.
    pub async fn batch_create_links(
        &self,
        requests: Vec<CreateLinkRequest>,
    ) -> Result<BatchOperationResult, ShortlinkerError> {
        let mut result = BatchOperationResult::default();

        // Step 1: Collect codes and validate URLs
        struct ValidatedRequest {
            code: String,
            target: String,
            expires_at: Option<String>,
            password: Option<String>,
            force: bool,
        }

        let mut codes_to_check: Vec<String> = Vec::new();
        let mut valid_requests: Vec<ValidatedRequest> = Vec::new();

        for req in requests {
            // Validate URL first
            if let Err(e) = validate_url(&req.target) {
                let code = req
                    .code
                    .clone()
                    .unwrap_or_else(|| "<generated>".to_string());
                result.failed.push(BatchFailedItem {
                    code,
                    reason: format!("Invalid URL: {}", e),
                });
                continue;
            }

            // Generate code if not provided
            let code = match req.code.filter(|c| !c.is_empty()) {
                Some(c) => c,
                None => generate_random_code(self.random_code_length()),
            };

            codes_to_check.push(code.clone());
            valid_requests.push(ValidatedRequest {
                code,
                target: req.target,
                expires_at: req.expires_at,
                password: req.password,
                force: req.force,
            });
        }

        // Step 2: Batch check existing codes
        let codes_refs: Vec<&str> = codes_to_check.iter().map(|s| s.as_str()).collect();
        let existing_map = self.storage.batch_get(&codes_refs).await.map_err(|e| {
            ShortlinkerError::database_operation(format!("Failed to batch check codes: {}", e))
        })?;

        // Step 3: Process each request
        let mut links_to_save: Vec<ShortLink> = Vec::new();

        for req in valid_requests {
            let existing = existing_map.get(&req.code);

            // Check existence conflict
            if existing.is_some() && !req.force {
                result.failed.push(BatchFailedItem {
                    code: req.code,
                    reason: "Code already exists. Use force=true to overwrite.".to_string(),
                });
                continue;
            }

            // Parse expiration time
            let expires_at = match self.parse_expires_at(req.expires_at.as_deref()) {
                Ok(dt) => dt,
                Err(e) => {
                    result.failed.push(BatchFailedItem {
                        code: req.code,
                        reason: format!("Invalid expires_at: {}", e),
                    });
                    continue;
                }
            };

            // Process password
            let password = match self.process_password(req.password.as_deref()) {
                Ok(pwd) => pwd,
                Err(e) => {
                    result.failed.push(BatchFailedItem {
                        code: req.code,
                        reason: format!("Password error: {}", e),
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
                code: req.code.clone(),
                target: req.target,
                created_at,
                expires_at,
                password,
                click,
            };

            links_to_save.push(new_link);
        }

        // Step 4: Batch save to storage
        if !links_to_save.is_empty() {
            self.storage
                .batch_set(links_to_save.clone())
                .await
                .map_err(|e| {
                    ShortlinkerError::database_operation(format!("Failed to batch save: {}", e))
                })?;

            // Update cache for each saved link
            for link in &links_to_save {
                self.update_cache(link).await;
                result.success.push(BatchSuccessItem {
                    code: link.code.clone(),
                    link: link.clone(),
                });
            }
        }

        info!(
            "LinkService: batch created {} links, {} failed",
            result.success.len(),
            result.failed.len()
        );

        Ok(result)
    }

    /// Batch update links
    ///
    /// Updates multiple links in a single operation. Each update is validated
    /// and processed independently.
    pub async fn batch_update_links(
        &self,
        updates: Vec<(String, UpdateLinkRequest)>,
    ) -> Result<BatchOperationResult, ShortlinkerError> {
        let mut result = BatchOperationResult::default();

        // Step 1: Collect codes and validate URLs
        struct ValidatedUpdate {
            code: String,
            target: String,
            expires_at: Option<String>,
            password: Option<String>,
        }

        let mut codes_to_check: Vec<String> = Vec::new();
        let mut valid_updates: Vec<ValidatedUpdate> = Vec::new();

        for (code, req) in updates {
            // Validate URL first
            if let Err(e) = validate_url(&req.target) {
                result.failed.push(BatchFailedItem {
                    code,
                    reason: format!("Invalid URL: {}", e),
                });
                continue;
            }

            codes_to_check.push(code.clone());
            valid_updates.push(ValidatedUpdate {
                code,
                target: req.target,
                expires_at: req.expires_at,
                password: req.password,
            });
        }

        // Step 2: Batch fetch existing links
        let codes_refs: Vec<&str> = codes_to_check.iter().map(|s| s.as_str()).collect();
        let existing_map = self.storage.batch_get(&codes_refs).await.map_err(|e| {
            ShortlinkerError::database_operation(format!("Failed to batch fetch: {}", e))
        })?;

        // Step 3: Process each update
        let mut links_to_save: Vec<ShortLink> = Vec::new();

        for update in valid_updates {
            let existing = match existing_map.get(&update.code) {
                Some(link) => link,
                None => {
                    result.failed.push(BatchFailedItem {
                        code: update.code,
                        reason: "Link not found".to_string(),
                    });
                    continue;
                }
            };

            // Parse expiration time (None = keep existing)
            let expires_at = if update.expires_at.is_some() {
                match self.parse_expires_at(update.expires_at.as_deref()) {
                    Ok(dt) => dt,
                    Err(e) => {
                        result.failed.push(BatchFailedItem {
                            code: update.code,
                            reason: format!("Invalid expires_at: {}", e),
                        });
                        continue;
                    }
                }
            } else {
                existing.expires_at
            };

            // Process password
            let password = match crate::utils::password::process_update_password(
                update.password.as_deref(),
                existing.password.clone(),
            ) {
                Ok(pwd) => pwd,
                Err(e) => {
                    result.failed.push(BatchFailedItem {
                        code: update.code,
                        reason: format!("Password error: {}", e),
                    });
                    continue;
                }
            };

            let updated_link = ShortLink {
                code: update.code.clone(),
                target: update.target,
                created_at: existing.created_at,
                expires_at,
                password,
                click: existing.click,
            };

            links_to_save.push(updated_link);
        }

        // Step 4: Batch save to storage
        if !links_to_save.is_empty() {
            self.storage
                .batch_set(links_to_save.clone())
                .await
                .map_err(|e| {
                    ShortlinkerError::database_operation(format!("Failed to batch update: {}", e))
                })?;

            // Update cache for each saved link
            for link in &links_to_save {
                self.update_cache(link).await;
                result.success.push(BatchSuccessItem {
                    code: link.code.clone(),
                    link: link.clone(),
                });
            }
        }

        info!(
            "LinkService: batch updated {} links, {} failed",
            result.success.len(),
            result.failed.len()
        );

        Ok(result)
    }

    /// Batch delete links
    ///
    /// Deletes multiple links in a single operation.
    pub async fn batch_delete_links(
        &self,
        codes: Vec<String>,
    ) -> Result<BatchDeleteResult, ShortlinkerError> {
        let mut result = BatchDeleteResult::default();

        // Step 1: Batch check which codes exist
        let codes_refs: Vec<&str> = codes.iter().map(|s| s.as_str()).collect();
        let existing_map = self.storage.batch_get(&codes_refs).await.map_err(|e| {
            ShortlinkerError::database_operation(format!("Failed to batch check: {}", e))
        })?;

        // Step 2: Separate existing and non-existing codes
        let mut codes_to_delete: Vec<String> = Vec::new();

        for code in codes {
            if existing_map.contains_key(&code) {
                codes_to_delete.push(code);
            } else {
                result.not_found.push(code);
            }
        }

        // Step 3: Batch delete from storage
        if !codes_to_delete.is_empty() {
            self.storage
                .batch_remove(&codes_to_delete)
                .await
                .map_err(|e| {
                    ShortlinkerError::database_operation(format!("Failed to batch delete: {}", e))
                })?;

            // Remove from cache
            for code in &codes_to_delete {
                self.cache.remove(code).await;
            }

            result.deleted = codes_to_delete;
        }

        info!(
            "LinkService: batch deleted {} links, {} not found",
            result.deleted.len(),
            result.not_found.len()
        );

        Ok(result)
    }
}
