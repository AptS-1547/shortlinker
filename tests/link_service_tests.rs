//! LinkService tests
//!
//! Tests for the link management service layer.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use shortlinker::cache::traits::{BloomConfig, CacheResult, CompositeCacheTrait};
use shortlinker::config::init_config;
use shortlinker::errors::ShortlinkerError;
use shortlinker::services::{
    CreateLinkRequest, ImportLinkItem, ImportMode, LinkService, UpdateLinkRequest,
};
use shortlinker::storage::backend::SeaOrmStorage;
use shortlinker::storage::{LinkFilter, ShortLink};
use std::sync::Once;
use tempfile::TempDir;
use tokio::sync::RwLock;

use shortlinker::metrics_core::NoopMetrics;

// =============================================================================
// Test Setup
// =============================================================================

static INIT: Once = Once::new();

fn init_test_config() {
    INIT.call_once(|| {
        init_config();
    });
}

/// Mock cache implementation for testing
struct MockCache {
    data: RwLock<HashMap<String, ShortLink>>,
    not_found: RwLock<std::collections::HashSet<String>>,
}

impl MockCache {
    fn new() -> Self {
        Self {
            data: RwLock::new(HashMap::new()),
            not_found: RwLock::new(std::collections::HashSet::new()),
        }
    }
}

#[async_trait]
impl CompositeCacheTrait for MockCache {
    async fn get(&self, key: &str) -> CacheResult {
        if self.not_found.read().await.contains(key) {
            return CacheResult::NotFound;
        }
        match self.data.read().await.get(key) {
            Some(link) => CacheResult::Found(link.clone()),
            None => CacheResult::Miss,
        }
    }

    async fn insert(&self, key: &str, value: ShortLink, _ttl_secs: Option<u64>) {
        self.not_found.write().await.remove(key);
        self.data.write().await.insert(key.to_string(), value);
    }

    async fn remove(&self, key: &str) {
        self.data.write().await.remove(key);
    }

    async fn invalidate_all(&self) {
        self.data.write().await.clear();
        self.not_found.write().await.clear();
    }

    async fn rebuild_all(&self) -> shortlinker::errors::Result<()> {
        self.data.write().await.clear();
        self.not_found.write().await.clear();
        Ok(())
    }

    async fn mark_not_found(&self, key: &str) {
        self.not_found.write().await.insert(key.to_string());
    }

    async fn load_cache(&self, links: HashMap<String, ShortLink>) {
        let mut data = self.data.write().await;
        for (k, v) in links {
            data.insert(k, v);
        }
    }

    async fn load_bloom(&self, _codes: &[String]) {
        // No-op for mock
    }

    async fn reconfigure(&self, _config: BloomConfig) -> shortlinker::errors::Result<()> {
        Ok(())
    }

    async fn bloom_check(&self, key: &str) -> bool {
        self.data.read().await.contains_key(key)
    }

    async fn health_check(&self) -> shortlinker::cache::CacheHealthStatus {
        shortlinker::cache::CacheHealthStatus {
            status: "healthy".to_string(),
            cache_type: "mock".to_string(),
            bloom_filter_enabled: false,
            negative_cache_enabled: true,
            error: None,
        }
    }
}

/// Create a test service with temporary storage
async fn create_test_service() -> (LinkService, TempDir) {
    init_test_config();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test_service.db");
    let db_url = format!("sqlite://{}?mode=rwc", db_path.display());

    let storage = Arc::new(
        SeaOrmStorage::new(&db_url, "sqlite", NoopMetrics::arc())
            .await
            .expect("Failed to create storage"),
    );

    let cache: Arc<dyn CompositeCacheTrait> = Arc::new(MockCache::new());
    let service = LinkService::new(storage, cache);

    (service, temp_dir)
}

/// Helper to create a basic CreateLinkRequest
fn create_request(code: Option<&str>, target: &str) -> CreateLinkRequest {
    CreateLinkRequest {
        code: code.map(|s| s.to_string()),
        target: target.to_string(),
        force: false,
        expires_at: None,
        password: None,
    }
}

// =============================================================================
// Create Link Tests
// =============================================================================

#[cfg(test)]
mod create_link_tests {
    use super::*;

    #[tokio::test]
    async fn test_create_link_with_code() {
        let (service, _temp) = create_test_service().await;

        let req = create_request(Some("mycode"), "https://example.com");
        let result = service.create_link(req).await;

        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.link.code, "mycode");
        assert_eq!(result.link.target, "https://example.com");
        assert!(!result.generated_code);
    }

    #[tokio::test]
    async fn test_create_link_auto_generate_code() {
        let (service, _temp) = create_test_service().await;

        let req = create_request(None, "https://example.com");
        let result = service.create_link(req).await;

        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(!result.link.code.is_empty());
        assert!(result.generated_code);
    }

    #[tokio::test]
    async fn test_create_link_conflict_without_force() {
        let (service, _temp) = create_test_service().await;

        // Create first link
        let req1 = create_request(Some("conflict"), "https://first.com");
        service.create_link(req1).await.unwrap();

        // Try to create with same code
        let req2 = create_request(Some("conflict"), "https://second.com");
        let result = service.create_link(req2).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            ShortlinkerError::LinkAlreadyExists(msg) => {
                assert!(msg.contains("conflict"));
            }
            other => panic!("Expected LinkAlreadyExists error, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_create_link_force_overwrite() {
        let (service, _temp) = create_test_service().await;

        // Create first link
        let req1 = create_request(Some("overwrite"), "https://first.com");
        service.create_link(req1).await.unwrap();

        // Force overwrite
        let req2 = CreateLinkRequest {
            code: Some("overwrite".to_string()),
            target: "https://second.com".to_string(),
            force: true,
            expires_at: None,
            password: None,
        };
        let result = service.create_link(req2).await;

        assert!(result.is_ok());
        let link = result.unwrap().link;
        assert_eq!(link.target, "https://second.com");
    }

    #[tokio::test]
    async fn test_create_link_invalid_url() {
        let (service, _temp) = create_test_service().await;

        let req = create_request(Some("badurl"), "not-a-valid-url");
        let result = service.create_link(req).await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ShortlinkerError::LinkInvalidUrl(_)
        ));
    }

    #[tokio::test]
    async fn test_create_link_dangerous_url() {
        let (service, _temp) = create_test_service().await;

        let req = create_request(Some("js"), "javascript:alert(1)");
        let result = service.create_link(req).await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ShortlinkerError::LinkInvalidUrl(_)
        ));
    }

    #[tokio::test]
    async fn test_create_link_with_expiry() {
        let (service, _temp) = create_test_service().await;

        let req = CreateLinkRequest {
            code: Some("expiry".to_string()),
            target: "https://example.com".to_string(),
            force: false,
            expires_at: Some("1d".to_string()), // 1 day
            password: None,
        };
        let result = service.create_link(req).await;

        assert!(result.is_ok());
        let link = result.unwrap().link;
        assert!(link.expires_at.is_some());

        // Should be roughly 1 day in the future
        let expires = link.expires_at.unwrap();
        let diff = expires - Utc::now();
        assert!(diff.num_hours() >= 23 && diff.num_hours() <= 25);
    }

    #[tokio::test]
    async fn test_create_link_invalid_expiry() {
        let (service, _temp) = create_test_service().await;

        let req = CreateLinkRequest {
            code: Some("badexpiry".to_string()),
            target: "https://example.com".to_string(),
            force: false,
            expires_at: Some("invalid-time".to_string()),
            password: None,
        };
        let result = service.create_link(req).await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ShortlinkerError::LinkInvalidExpireTime(_)
        ));
    }

    #[tokio::test]
    async fn test_create_link_with_password() {
        let (service, _temp) = create_test_service().await;

        let req = CreateLinkRequest {
            code: Some("protected".to_string()),
            target: "https://example.com".to_string(),
            force: false,
            expires_at: None,
            password: Some("secret123".to_string()),
        };
        let result = service.create_link(req).await;

        assert!(result.is_ok());
        let link = result.unwrap().link;
        assert!(link.password.is_some());
        // Password should be hashed (Argon2 format starts with $argon2)
        assert!(link.password.as_ref().unwrap().starts_with("$argon2"));
    }

    #[tokio::test]
    async fn test_create_link_preserves_click_on_overwrite() {
        let (service, _temp) = create_test_service().await;

        // Create first link (we can't easily add clicks, but test the flow)
        let req1 = create_request(Some("clicks"), "https://first.com");
        service.create_link(req1).await.unwrap();

        // Overwrite
        let req2 = CreateLinkRequest {
            code: Some("clicks".to_string()),
            target: "https://second.com".to_string(),
            force: true,
            expires_at: None,
            password: None,
        };
        let result = service.create_link(req2).await.unwrap();

        // click count should be 0 (preserved from original which was 0)
        assert_eq!(result.link.click, 0);
    }
}

// =============================================================================
// Update Link Tests
// =============================================================================

#[cfg(test)]
mod update_link_tests {
    use super::*;

    #[tokio::test]
    async fn test_update_link_target() {
        let (service, _temp) = create_test_service().await;

        // Create link
        let req = create_request(Some("update_me"), "https://old.com");
        service.create_link(req).await.unwrap();

        // Update target
        let update_req = UpdateLinkRequest {
            target: "https://new.com".to_string(),
            expires_at: None,
            password: None,
        };
        let result = service.update_link("update_me", update_req).await;

        assert!(result.is_ok());
        let link = result.unwrap();
        assert_eq!(link.target, "https://new.com");
    }

    #[tokio::test]
    async fn test_update_link_not_found() {
        let (service, _temp) = create_test_service().await;

        let update_req = UpdateLinkRequest {
            target: "https://new.com".to_string(),
            expires_at: None,
            password: None,
        };
        let result = service.update_link("nonexistent", update_req).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ShortlinkerError::NotFound(_)));
    }

    #[tokio::test]
    async fn test_update_link_invalid_url() {
        let (service, _temp) = create_test_service().await;

        // Create link
        let req = create_request(Some("update_invalid"), "https://valid.com");
        service.create_link(req).await.unwrap();

        // Try to update with invalid URL
        let update_req = UpdateLinkRequest {
            target: "not-a-url".to_string(),
            expires_at: None,
            password: None,
        };
        let result = service.update_link("update_invalid", update_req).await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ShortlinkerError::LinkInvalidUrl(_)
        ));
    }

    #[tokio::test]
    async fn test_update_link_add_expiry() {
        let (service, _temp) = create_test_service().await;

        // Create link without expiry
        let req = create_request(Some("add_expiry"), "https://example.com");
        let created = service.create_link(req).await.unwrap();
        assert!(created.link.expires_at.is_none());

        // Update to add expiry
        let update_req = UpdateLinkRequest {
            target: "https://example.com".to_string(),
            expires_at: Some("2h".to_string()),
            password: None,
        };
        let result = service.update_link("add_expiry", update_req).await;

        assert!(result.is_ok());
        let link = result.unwrap();
        assert!(link.expires_at.is_some());
    }

    #[tokio::test]
    async fn test_update_link_remove_password() {
        let (service, _temp) = create_test_service().await;

        // Create link with password
        let req = CreateLinkRequest {
            code: Some("remove_pwd".to_string()),
            target: "https://example.com".to_string(),
            force: false,
            expires_at: None,
            password: Some("secret".to_string()),
        };
        let created = service.create_link(req).await.unwrap();
        assert!(created.link.password.is_some());

        // Update with empty password to remove it
        let update_req = UpdateLinkRequest {
            target: "https://example.com".to_string(),
            expires_at: None,
            password: Some("".to_string()), // Empty string = remove
        };
        let result = service.update_link("remove_pwd", update_req).await;

        assert!(result.is_ok());
        let link = result.unwrap();
        assert!(link.password.is_none());
    }

    #[tokio::test]
    async fn test_update_link_preserves_created_at() {
        let (service, _temp) = create_test_service().await;

        // Create link
        let req = create_request(Some("preserve_time"), "https://old.com");
        let created = service.create_link(req).await.unwrap();
        let original_created_at = created.link.created_at;

        // Wait a bit
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        // Update
        let update_req = UpdateLinkRequest {
            target: "https://new.com".to_string(),
            expires_at: None,
            password: None,
        };
        let updated = service
            .update_link("preserve_time", update_req)
            .await
            .unwrap();

        // created_at should be preserved
        assert_eq!(updated.created_at, original_created_at);
    }
}

// =============================================================================
// Delete Link Tests
// =============================================================================

#[cfg(test)]
mod delete_link_tests {
    use super::*;

    #[tokio::test]
    async fn test_delete_link() {
        let (service, _temp) = create_test_service().await;

        // Create and delete
        let req = create_request(Some("to_delete"), "https://example.com");
        service.create_link(req).await.unwrap();

        let result = service.delete_link("to_delete").await;
        assert!(result.is_ok());

        // Verify deleted
        let get_result = service.get_link("to_delete").await.unwrap();
        assert!(get_result.is_none());
    }

    #[tokio::test]
    async fn test_delete_link_not_found() {
        let (service, _temp) = create_test_service().await;

        let result = service.delete_link("never_existed").await;
        assert!(result.is_err());
    }
}

// =============================================================================
// Get/List Tests
// =============================================================================

#[cfg(test)]
mod query_tests {
    use super::*;

    #[tokio::test]
    async fn test_get_link() {
        let (service, _temp) = create_test_service().await;

        // Create
        let req = create_request(Some("get_me"), "https://example.com");
        service.create_link(req).await.unwrap();

        // Get
        let result = service.get_link("get_me").await.unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().code, "get_me");
    }

    #[tokio::test]
    async fn test_get_link_not_found() {
        let (service, _temp) = create_test_service().await;

        let result = service.get_link("nonexistent").await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_list_links_pagination() {
        let (service, _temp) = create_test_service().await;

        // Create 15 links
        for i in 0..15 {
            let req = create_request(Some(&format!("list_{:02}", i)), "https://example.com");
            service.create_link(req).await.unwrap();
        }

        // Get first page
        let (links, total) = service
            .list_links(LinkFilter::default(), 1, 5)
            .await
            .unwrap();
        assert_eq!(total, 15);
        assert_eq!(links.len(), 5);

        // Get third page
        let (links, _) = service
            .list_links(LinkFilter::default(), 3, 5)
            .await
            .unwrap();
        assert_eq!(links.len(), 5);
    }

    #[tokio::test]
    async fn test_list_links_with_filter() {
        let (service, _temp) = create_test_service().await;

        // Create mixed links
        let req = create_request(Some("github_link"), "https://github.com/test");
        service.create_link(req).await.unwrap();

        let req = create_request(Some("google_link"), "https://google.com");
        service.create_link(req).await.unwrap();

        // Filter by search
        let filter = LinkFilter {
            search: Some("github".to_string()),
            ..Default::default()
        };

        let (links, total) = service.list_links(filter, 1, 10).await.unwrap();
        assert_eq!(total, 1);
        assert_eq!(links[0].code, "github_link");
    }

    #[tokio::test]
    async fn test_get_stats() {
        let (service, _temp) = create_test_service().await;

        // Empty stats
        let stats = service.get_stats().await.unwrap();
        assert_eq!(stats.total_links, 0);

        // Add some links
        for i in 0..5 {
            let req = create_request(Some(&format!("stat_{}", i)), "https://example.com");
            service.create_link(req).await.unwrap();
        }

        let stats = service.get_stats().await.unwrap();
        assert_eq!(stats.total_links, 5);
        assert_eq!(stats.active_links, 5);
    }
}

// =============================================================================
// Import/Export Tests
// =============================================================================

#[cfg(test)]
mod import_export_tests {
    use super::*;

    #[tokio::test]
    async fn test_import_links_skip_mode() {
        let (service, _temp) = create_test_service().await;

        // Create existing link
        let req = create_request(Some("existing"), "https://old.com");
        service.create_link(req).await.unwrap();

        // Import with skip mode
        let items = vec![
            ImportLinkItem {
                code: "existing".to_string(),
                target: "https://new.com".to_string(),
                expires_at: None,
                password: None,
            },
            ImportLinkItem {
                code: "new_link".to_string(),
                target: "https://new.com".to_string(),
                expires_at: None,
                password: None,
            },
        ];

        let result = service.import_links(items, ImportMode::Skip).await.unwrap();

        assert_eq!(result.success, 1);
        assert_eq!(result.skipped, 1);
        assert_eq!(result.failed, 0);

        // Verify existing wasn't overwritten
        let link = service.get_link("existing").await.unwrap().unwrap();
        assert_eq!(link.target, "https://old.com");
    }

    #[tokio::test]
    async fn test_import_links_overwrite_mode() {
        let (service, _temp) = create_test_service().await;

        // Create existing link
        let req = create_request(Some("overwrite_import"), "https://old.com");
        service.create_link(req).await.unwrap();

        // Import with overwrite mode
        let items = vec![ImportLinkItem {
            code: "overwrite_import".to_string(),
            target: "https://new.com".to_string(),
            expires_at: None,
            password: None,
        }];

        let result = service
            .import_links(items, ImportMode::Overwrite)
            .await
            .unwrap();

        assert_eq!(result.success, 1);
        assert_eq!(result.skipped, 0);

        // Verify it was overwritten
        let link = service.get_link("overwrite_import").await.unwrap().unwrap();
        assert_eq!(link.target, "https://new.com");
    }

    #[tokio::test]
    async fn test_import_links_error_mode() {
        let (service, _temp) = create_test_service().await;

        // Create existing link
        let req = create_request(Some("error_import"), "https://old.com");
        service.create_link(req).await.unwrap();

        // Import with error mode
        let items = vec![
            ImportLinkItem {
                code: "error_import".to_string(),
                target: "https://new.com".to_string(),
                expires_at: None,
                password: None,
            },
            ImportLinkItem {
                code: "new_one".to_string(),
                target: "https://new.com".to_string(),
                expires_at: None,
                password: None,
            },
        ];

        let result = service
            .import_links(items, ImportMode::Error)
            .await
            .unwrap();

        assert_eq!(result.success, 1);
        assert_eq!(result.failed, 1);
        assert_eq!(result.errors.len(), 1);
        assert_eq!(result.errors[0].code, "error_import");
    }

    #[tokio::test]
    async fn test_import_links_invalid_url() {
        let (service, _temp) = create_test_service().await;

        let items = vec![
            ImportLinkItem {
                code: "valid".to_string(),
                target: "https://valid.com".to_string(),
                expires_at: None,
                password: None,
            },
            ImportLinkItem {
                code: "invalid".to_string(),
                target: "not-a-url".to_string(),
                expires_at: None,
                password: None,
            },
        ];

        let result = service.import_links(items, ImportMode::Skip).await.unwrap();

        assert_eq!(result.success, 1);
        assert_eq!(result.failed, 1);
    }

    #[tokio::test]
    async fn test_export_links() {
        let (service, _temp) = create_test_service().await;

        // Create some links
        for i in 0..3 {
            let req = create_request(Some(&format!("export_{}", i)), "https://example.com");
            service.create_link(req).await.unwrap();
        }

        let exported = service.export_links().await.unwrap();
        assert_eq!(exported.len(), 3);
    }
}

// =============================================================================
// Edge Cases
// =============================================================================

#[cfg(test)]
mod edge_cases {
    use super::*;

    #[tokio::test]
    async fn test_empty_code_generates_random() {
        let (service, _temp) = create_test_service().await;

        let req = CreateLinkRequest {
            code: Some("".to_string()), // Empty string
            target: "https://example.com".to_string(),
            force: false,
            expires_at: None,
            password: None,
        };
        let result = service.create_link(req).await.unwrap();

        assert!(result.generated_code);
        assert!(!result.link.code.is_empty());
    }

    #[tokio::test]
    async fn test_page_size_clamped() {
        let (service, _temp) = create_test_service().await;

        // Create some links
        for i in 0..5 {
            let req = create_request(Some(&format!("clamp_{}", i)), "https://example.com");
            service.create_link(req).await.unwrap();
        }

        // Request huge page size
        let (links, _) = service
            .list_links(LinkFilter::default(), 1, 1000)
            .await
            .unwrap();

        // Should be clamped to 100 max
        assert!(links.len() <= 100);
    }

    #[tokio::test]
    async fn test_page_zero_treated_as_one() {
        let (service, _temp) = create_test_service().await;

        let req = create_request(Some("page_test"), "https://example.com");
        service.create_link(req).await.unwrap();

        // Page 0 should work like page 1
        let (links, _) = service
            .list_links(LinkFilter::default(), 0, 10)
            .await
            .unwrap();

        assert_eq!(links.len(), 1);
    }

    #[tokio::test]
    async fn test_import_empty_list() {
        let (service, _temp) = create_test_service().await;

        let result = service
            .import_links(vec![], ImportMode::Skip)
            .await
            .unwrap();

        assert_eq!(result.success, 0);
        assert_eq!(result.skipped, 0);
        assert_eq!(result.failed, 0);
    }

    #[tokio::test]
    async fn test_pre_hashed_password_not_preserved() {
        let (service, _temp) = create_test_service().await;

        // Use an already hashed password - should be re-hashed, not preserved
        let hashed = "$argon2id$v=19$m=19456,t=2,p=1$somesalt$somehash";
        let req = CreateLinkRequest {
            code: Some("prehashed".to_string()),
            target: "https://example.com".to_string(),
            force: false,
            expires_at: None,
            password: Some(hashed.to_string()),
        };

        let result = service.create_link(req).await.unwrap();

        // Should be re-hashed, not stored as-is
        assert!(result.link.password.is_some());
        assert_ne!(result.link.password, Some(hashed.to_string()));
    }

    #[tokio::test]
    async fn test_relative_time_formats() {
        let (service, _temp) = create_test_service().await;

        let test_cases = [
            ("1h", 1),   // 1 hour
            ("24h", 24), // 24 hours
            ("1d", 24),  // 1 day
            ("7d", 168), // 7 days
        ];

        for (i, (time_str, expected_hours)) in test_cases.iter().enumerate() {
            let req = CreateLinkRequest {
                code: Some(format!("time_test_{}", i)),
                target: "https://example.com".to_string(),
                force: false,
                expires_at: Some(time_str.to_string()),
                password: None,
            };

            let result = service.create_link(req).await.unwrap();
            let expires = result.link.expires_at.unwrap();
            let diff = expires - Utc::now();

            // Allow some tolerance
            assert!(
                diff.num_hours() >= expected_hours - 1 && diff.num_hours() <= expected_hours + 1,
                "Time '{}' should be ~{} hours, got {} hours",
                time_str,
                expected_hours,
                diff.num_hours()
            );
        }
    }
}

// =============================================================================
// Batch Operations Tests
// =============================================================================

#[cfg(test)]
mod batch_operations_tests {
    use super::*;

    #[tokio::test]
    async fn test_batch_create_links_success() {
        let (service, _temp) = create_test_service().await;

        let requests = vec![
            create_request(Some("batch1"), "https://example1.com"),
            create_request(Some("batch2"), "https://example2.com"),
            create_request(Some("batch3"), "https://example3.com"),
        ];

        let result = service.batch_create_links(requests).await.unwrap();
        assert_eq!(result.success.len(), 3);
        assert!(result.failed.is_empty());
    }

    #[tokio::test]
    async fn test_batch_create_links_with_invalid_url() {
        let (service, _temp) = create_test_service().await;

        let requests = vec![
            create_request(Some("valid_batch"), "https://valid.com"),
            create_request(Some("invalid_batch"), "not-a-url"),
        ];

        let result = service.batch_create_links(requests).await.unwrap();
        assert_eq!(result.success.len(), 1);
        assert_eq!(result.failed.len(), 1);
        assert_eq!(result.failed[0].code, "invalid_batch");
    }

    #[tokio::test]
    async fn test_batch_create_links_auto_generate_code() {
        let (service, _temp) = create_test_service().await;

        let requests = vec![
            create_request(None, "https://auto1.com"),
            create_request(None, "https://auto2.com"),
        ];

        let result = service.batch_create_links(requests).await.unwrap();
        assert_eq!(result.success.len(), 2);
        // All should have generated codes
        for item in &result.success {
            assert!(!item.code.is_empty());
        }
    }

    #[tokio::test]
    async fn test_batch_create_links_conflict_without_force() {
        let (service, _temp) = create_test_service().await;

        // Create existing link
        let req = create_request(Some("batch_conflict"), "https://old.com");
        service.create_link(req).await.unwrap();

        // Try batch create with same code
        let requests = vec![create_request(Some("batch_conflict"), "https://new.com")];

        let result = service.batch_create_links(requests).await.unwrap();
        assert!(result.success.is_empty());
        assert_eq!(result.failed.len(), 1);
    }

    #[tokio::test]
    async fn test_batch_create_links_with_force() {
        let (service, _temp) = create_test_service().await;

        // Create existing link
        let req = create_request(Some("batch_force"), "https://old.com");
        service.create_link(req).await.unwrap();

        // Batch create with force
        let requests = vec![CreateLinkRequest {
            code: Some("batch_force".to_string()),
            target: "https://new.com".to_string(),
            force: true,
            expires_at: None,
            password: None,
        }];

        let result = service.batch_create_links(requests).await.unwrap();
        assert_eq!(result.success.len(), 1);

        // Verify overwritten
        let link = service.get_link("batch_force").await.unwrap().unwrap();
        assert_eq!(link.target, "https://new.com");
    }

    #[tokio::test]
    async fn test_batch_create_links_with_expiry() {
        let (service, _temp) = create_test_service().await;

        let requests = vec![
            CreateLinkRequest {
                code: Some("batch_exp1".to_string()),
                target: "https://example.com".to_string(),
                force: false,
                expires_at: Some("1h".to_string()),
                password: None,
            },
            CreateLinkRequest {
                code: Some("batch_exp2".to_string()),
                target: "https://example.com".to_string(),
                force: false,
                expires_at: Some("invalid-time".to_string()),
                password: None,
            },
        ];

        let result = service.batch_create_links(requests).await.unwrap();
        assert_eq!(result.success.len(), 1);
        assert_eq!(result.failed.len(), 1);
    }

    #[tokio::test]
    async fn test_batch_update_links_success() {
        let (service, _temp) = create_test_service().await;

        // Create links first
        for i in 1..=3 {
            let req = create_request(Some(&format!("upd{}", i)), "https://old.com");
            service.create_link(req).await.unwrap();
        }

        // Batch update
        let updates = vec![
            (
                "upd1".to_string(),
                UpdateLinkRequest {
                    target: "https://new1.com".to_string(),
                    expires_at: None,
                    password: None,
                },
            ),
            (
                "upd2".to_string(),
                UpdateLinkRequest {
                    target: "https://new2.com".to_string(),
                    expires_at: None,
                    password: None,
                },
            ),
        ];

        let result = service.batch_update_links(updates).await.unwrap();
        assert_eq!(result.success.len(), 2);
        assert!(result.failed.is_empty());

        // Verify updates
        let link1 = service.get_link("upd1").await.unwrap().unwrap();
        assert_eq!(link1.target, "https://new1.com");
    }

    #[tokio::test]
    async fn test_batch_update_links_not_found() {
        let (service, _temp) = create_test_service().await;

        let updates = vec![(
            "nonexistent_batch".to_string(),
            UpdateLinkRequest {
                target: "https://new.com".to_string(),
                expires_at: None,
                password: None,
            },
        )];

        let result = service.batch_update_links(updates).await.unwrap();
        assert!(result.success.is_empty());
        assert_eq!(result.failed.len(), 1);
    }

    #[tokio::test]
    async fn test_batch_update_links_invalid_url() {
        let (service, _temp) = create_test_service().await;

        // Create link
        let req = create_request(Some("batch_upd_invalid"), "https://old.com");
        service.create_link(req).await.unwrap();

        let updates = vec![(
            "batch_upd_invalid".to_string(),
            UpdateLinkRequest {
                target: "not-a-url".to_string(),
                expires_at: None,
                password: None,
            },
        )];

        let result = service.batch_update_links(updates).await.unwrap();
        assert!(result.success.is_empty());
        assert_eq!(result.failed.len(), 1);
    }

    #[tokio::test]
    async fn test_batch_update_links_mixed() {
        let (service, _temp) = create_test_service().await;

        // Create one link
        let req = create_request(Some("batch_mix_exists"), "https://old.com");
        service.create_link(req).await.unwrap();

        let updates = vec![
            (
                "batch_mix_exists".to_string(),
                UpdateLinkRequest {
                    target: "https://new.com".to_string(),
                    expires_at: None,
                    password: None,
                },
            ),
            (
                "batch_mix_missing".to_string(),
                UpdateLinkRequest {
                    target: "https://new.com".to_string(),
                    expires_at: None,
                    password: None,
                },
            ),
        ];

        let result = service.batch_update_links(updates).await.unwrap();
        assert_eq!(result.success.len(), 1);
        assert_eq!(result.failed.len(), 1);
    }

    #[tokio::test]
    async fn test_batch_update_links_with_password() {
        let (service, _temp) = create_test_service().await;

        // Create link
        let req = create_request(Some("batch_pwd"), "https://old.com");
        service.create_link(req).await.unwrap();

        let updates = vec![(
            "batch_pwd".to_string(),
            UpdateLinkRequest {
                target: "https://old.com".to_string(),
                expires_at: None,
                password: Some("newpassword".to_string()),
            },
        )];

        let result = service.batch_update_links(updates).await.unwrap();
        assert_eq!(result.success.len(), 1);

        let link = service.get_link("batch_pwd").await.unwrap().unwrap();
        assert!(link.password.is_some());
    }

    #[tokio::test]
    async fn test_batch_delete_links_success() {
        let (service, _temp) = create_test_service().await;

        // Create links
        for i in 1..=3 {
            let req = create_request(Some(&format!("del{}", i)), "https://example.com");
            service.create_link(req).await.unwrap();
        }

        let codes = vec!["del1".to_string(), "del2".to_string()];
        let result = service.batch_delete_links(codes).await.unwrap();

        assert_eq!(result.deleted.len(), 2);
        assert!(result.not_found.is_empty());

        // Verify deleted
        assert!(service.get_link("del1").await.unwrap().is_none());
        assert!(service.get_link("del2").await.unwrap().is_none());
        // del3 should still exist
        assert!(service.get_link("del3").await.unwrap().is_some());
    }

    #[tokio::test]
    async fn test_batch_delete_links_not_found() {
        let (service, _temp) = create_test_service().await;

        let codes = vec!["never_existed1".to_string(), "never_existed2".to_string()];
        let result = service.batch_delete_links(codes).await.unwrap();

        assert!(result.deleted.is_empty());
        assert_eq!(result.not_found.len(), 2);
    }

    #[tokio::test]
    async fn test_batch_delete_links_mixed() {
        let (service, _temp) = create_test_service().await;

        // Create one link
        let req = create_request(Some("del_exists"), "https://example.com");
        service.create_link(req).await.unwrap();

        let codes = vec!["del_exists".to_string(), "del_missing".to_string()];
        let result = service.batch_delete_links(codes).await.unwrap();

        assert_eq!(result.deleted.len(), 1);
        assert_eq!(result.not_found.len(), 1);
    }

    #[tokio::test]
    async fn test_batch_delete_links_empty() {
        let (service, _temp) = create_test_service().await;

        let result = service.batch_delete_links(vec![]).await.unwrap();
        assert!(result.deleted.is_empty());
        assert!(result.not_found.is_empty());
    }
}
