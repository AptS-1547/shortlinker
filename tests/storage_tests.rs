//! Storage backend tests
//!
//! Tests for SeaOrmStorage using temporary SQLite databases.

use chrono::{Duration, Utc};
use shortlinker::config::init_config;
use shortlinker::storage::backend::{
    LinkFilter, SeaOrmStorage, connect_sqlite, infer_backend_from_url, normalize_backend_name,
    run_migrations,
};
use shortlinker::storage::ShortLink;
use std::sync::Once;
use tempfile::TempDir;

// 确保 config 只初始化一次
static INIT: Once = Once::new();

fn init_test_config() {
    INIT.call_once(|| {
        init_config();
    });
}

/// 创建测试用的 ShortLink
fn create_test_link(code: &str, target: &str) -> ShortLink {
    ShortLink {
        code: code.to_string(),
        target: target.to_string(),
        created_at: Utc::now(),
        expires_at: None,
        password: None,
        click: 0,
    }
}

/// 创建带过期时间的测试链接
fn create_test_link_with_expiry(code: &str, expires_in: Duration) -> ShortLink {
    ShortLink {
        code: code.to_string(),
        target: format!("https://{}.example.com", code),
        created_at: Utc::now(),
        expires_at: Some(Utc::now() + expires_in),
        password: None,
        click: 0,
    }
}

/// 创建临时 SQLite 数据库的存储实例
async fn create_temp_storage() -> (SeaOrmStorage, TempDir) {
    init_test_config();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test.db");
    let db_url = format!("sqlite://{}?mode=rwc", db_path.display());

    let storage = SeaOrmStorage::new(&db_url, "sqlite")
        .await
        .expect("Failed to create storage");

    (storage, temp_dir)
}

// =============================================================================
// URL 推断和规范化测试
// =============================================================================

#[cfg(test)]
mod url_inference_tests {
    use super::*;

    #[test]
    fn test_infer_sqlite_from_prefix() {
        assert_eq!(
            infer_backend_from_url("sqlite:///path/to/db").unwrap(),
            "sqlite"
        );
        assert_eq!(
            infer_backend_from_url("sqlite://test.db").unwrap(),
            "sqlite"
        );
    }

    #[test]
    fn test_infer_sqlite_from_extension() {
        assert_eq!(infer_backend_from_url("test.db").unwrap(), "sqlite");
        assert_eq!(
            infer_backend_from_url("/path/to/data.sqlite").unwrap(),
            "sqlite"
        );
    }

    #[test]
    fn test_infer_sqlite_memory() {
        assert_eq!(infer_backend_from_url(":memory:").unwrap(), "sqlite");
    }

    #[test]
    fn test_infer_mysql() {
        assert_eq!(
            infer_backend_from_url("mysql://user:pass@localhost/db").unwrap(),
            "mysql"
        );
        assert_eq!(
            infer_backend_from_url("mariadb://user:pass@localhost/db").unwrap(),
            "mysql"
        );
    }

    #[test]
    fn test_infer_postgres() {
        assert_eq!(
            infer_backend_from_url("postgres://user:pass@localhost/db").unwrap(),
            "postgres"
        );
        assert_eq!(
            infer_backend_from_url("postgresql://user:pass@localhost/db").unwrap(),
            "postgres"
        );
    }

    #[test]
    fn test_infer_unknown_returns_error() {
        let result = infer_backend_from_url("unknown://something");
        assert!(result.is_err());
    }

    #[test]
    fn test_normalize_mariadb_to_mysql() {
        assert_eq!(normalize_backend_name("mariadb"), "mysql");
    }

    #[test]
    fn test_normalize_other_unchanged() {
        assert_eq!(normalize_backend_name("sqlite"), "sqlite");
        assert_eq!(normalize_backend_name("mysql"), "mysql");
        assert_eq!(normalize_backend_name("postgres"), "postgres");
    }
}

// =============================================================================
// 连接测试
// =============================================================================

#[cfg(test)]
mod connection_tests {
    use super::*;

    #[tokio::test]
    async fn test_connect_sqlite_creates_file() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("new_db.db");
        let db_url = format!("sqlite://{}?mode=rwc", db_path.display());

        let conn = connect_sqlite(&db_url).await;
        assert!(conn.is_ok(), "Should connect to SQLite: {:?}", conn);
    }

    #[tokio::test]
    async fn test_connect_sqlite_memory() {
        let conn = connect_sqlite("sqlite::memory:").await;
        assert!(conn.is_ok(), "Should connect to in-memory SQLite");
    }

    #[tokio::test]
    async fn test_run_migrations() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("migration_test.db");
        let db_url = format!("sqlite://{}?mode=rwc", db_path.display());

        let conn = connect_sqlite(&db_url).await.unwrap();
        let result = run_migrations(&conn).await;
        assert!(result.is_ok(), "Migrations should run: {:?}", result);
    }

    #[tokio::test]
    async fn test_storage_new_empty_url_fails() {
        init_test_config();
        let result = SeaOrmStorage::new("", "sqlite").await;
        assert!(result.is_err());
    }
}

// =============================================================================
// 基本 CRUD 测试
// =============================================================================

#[cfg(test)]
mod crud_tests {
    use super::*;

    #[tokio::test]
    async fn test_set_and_get() {
        let (storage, _temp) = create_temp_storage().await;

        let link = create_test_link("abc123", "https://example.com");
        storage.set(link.clone()).await.expect("set should succeed");

        let retrieved = storage.get("abc123").await.expect("get should succeed");
        assert!(retrieved.is_some());

        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.code, "abc123");
        assert_eq!(retrieved.target, "https://example.com");
    }

    #[tokio::test]
    async fn test_get_nonexistent_returns_none() {
        let (storage, _temp) = create_temp_storage().await;

        let result = storage.get("nonexistent").await.expect("get should succeed");
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_set_updates_existing() {
        let (storage, _temp) = create_temp_storage().await;

        // 先创建
        let link1 = create_test_link("update_test", "https://old.com");
        storage.set(link1).await.unwrap();

        // 更新
        let link2 = create_test_link("update_test", "https://new.com");
        storage.set(link2).await.unwrap();

        // 验证更新
        let retrieved = storage.get("update_test").await.unwrap().unwrap();
        assert_eq!(retrieved.target, "https://new.com");
    }

    #[tokio::test]
    async fn test_remove_existing() {
        let (storage, _temp) = create_temp_storage().await;

        let link = create_test_link("to_delete", "https://example.com");
        storage.set(link).await.unwrap();

        // 删除
        let result = storage.remove("to_delete").await;
        assert!(result.is_ok());

        // 验证已删除
        let retrieved = storage.get("to_delete").await.unwrap();
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_remove_nonexistent_fails() {
        let (storage, _temp) = create_temp_storage().await;

        let result = storage.remove("never_existed").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_count() {
        let (storage, _temp) = create_temp_storage().await;

        // 初始为 0
        let count = storage.count().await.unwrap();
        assert_eq!(count, 0);

        // 添加几条
        for i in 0..5 {
            let link = create_test_link(&format!("count_{}", i), "https://example.com");
            storage.set(link).await.unwrap();
        }

        let count = storage.count().await.unwrap();
        assert_eq!(count, 5);
    }
}

// =============================================================================
// 批量操作测试
// =============================================================================

#[cfg(test)]
mod batch_tests {
    use super::*;

    #[tokio::test]
    async fn test_batch_set() {
        let (storage, _temp) = create_temp_storage().await;

        let links: Vec<ShortLink> = (0..10)
            .map(|i| create_test_link(&format!("batch_{}", i), &format!("https://{}.com", i)))
            .collect();

        storage.batch_set(links).await.expect("batch_set should succeed");

        // 验证全部存在
        for i in 0..10 {
            let link = storage.get(&format!("batch_{}", i)).await.unwrap();
            assert!(link.is_some(), "Link batch_{} should exist", i);
        }
    }

    #[tokio::test]
    async fn test_batch_set_empty() {
        let (storage, _temp) = create_temp_storage().await;

        let result = storage.batch_set(vec![]).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_batch_remove() {
        let (storage, _temp) = create_temp_storage().await;

        // 先创建一些链接
        for i in 0..5 {
            let link = create_test_link(&format!("br_{}", i), "https://example.com");
            storage.set(link).await.unwrap();
        }

        // 批量删除（包含存在和不存在的）
        let codes: Vec<String> = vec![
            "br_0".to_string(),
            "br_2".to_string(),
            "br_99".to_string(), // 不存在
        ];

        let (deleted, not_found) = storage.batch_remove(&codes).await.unwrap();

        assert_eq!(deleted.len(), 2);
        assert!(deleted.contains(&"br_0".to_string()));
        assert!(deleted.contains(&"br_2".to_string()));

        assert_eq!(not_found.len(), 1);
        assert!(not_found.contains(&"br_99".to_string()));

        // 验证删除的确实不存在了
        assert!(storage.get("br_0").await.unwrap().is_none());
        assert!(storage.get("br_2").await.unwrap().is_none());

        // 未删除的还在
        assert!(storage.get("br_1").await.unwrap().is_some());
    }

    #[tokio::test]
    async fn test_batch_remove_empty() {
        let (storage, _temp) = create_temp_storage().await;

        let (deleted, not_found) = storage.batch_remove(&[]).await.unwrap();
        assert!(deleted.is_empty());
        assert!(not_found.is_empty());
    }

    #[tokio::test]
    async fn test_batch_get() {
        let (storage, _temp) = create_temp_storage().await;

        // 创建测试数据
        for i in 0..3 {
            let link = create_test_link(&format!("bg_{}", i), &format!("https://{}.com", i));
            storage.set(link).await.unwrap();
        }

        let codes = vec!["bg_0", "bg_1", "bg_99"]; // bg_99 不存在
        let result = storage.batch_get(&codes).await.unwrap();

        assert_eq!(result.len(), 2);
        assert!(result.contains_key("bg_0"));
        assert!(result.contains_key("bg_1"));
        assert!(!result.contains_key("bg_99"));
    }
}

// =============================================================================
// 查询和分页测试
// =============================================================================

#[cfg(test)]
mod query_tests {
    use super::*;

    #[tokio::test]
    async fn test_load_all() {
        let (storage, _temp) = create_temp_storage().await;

        for i in 0..3 {
            let link = create_test_link(&format!("all_{}", i), "https://example.com");
            storage.set(link).await.unwrap();
        }

        let all = storage.load_all().await.unwrap();
        assert_eq!(all.len(), 3);
    }

    #[tokio::test]
    async fn test_load_all_codes() {
        let (storage, _temp) = create_temp_storage().await;

        for i in 0..3 {
            let link = create_test_link(&format!("code_{}", i), "https://example.com");
            storage.set(link).await.unwrap();
        }

        let codes = storage.load_all_codes().await.unwrap();
        assert_eq!(codes.len(), 3);
        assert!(codes.contains(&"code_0".to_string()));
        assert!(codes.contains(&"code_1".to_string()));
        assert!(codes.contains(&"code_2".to_string()));
    }

    #[tokio::test]
    async fn test_load_paginated_filtered_basic() {
        let (storage, _temp) = create_temp_storage().await;

        // 创建 15 条链接
        for i in 0..15 {
            let link = create_test_link(&format!("page_{:02}", i), "https://example.com");
            storage.set(link).await.unwrap();
        }

        // 第一页，每页 5 条
        let filter = LinkFilter::default();
        let (links, total) = storage
            .load_paginated_filtered(1, 5, filter.clone())
            .await
            .unwrap();

        assert_eq!(total, 15);
        assert_eq!(links.len(), 5);

        // 第三页
        let (links, _) = storage.load_paginated_filtered(3, 5, filter).await.unwrap();
        assert_eq!(links.len(), 5);
    }

    #[tokio::test]
    async fn test_load_paginated_filtered_with_search() {
        let (storage, _temp) = create_temp_storage().await;

        // 创建混合数据
        storage
            .set(create_test_link("github_1", "https://github.com/a"))
            .await
            .unwrap();
        storage
            .set(create_test_link("github_2", "https://github.com/b"))
            .await
            .unwrap();
        storage
            .set(create_test_link("google", "https://google.com"))
            .await
            .unwrap();
        storage
            .set(create_test_link("other", "https://github.com/other"))
            .await
            .unwrap();

        // 搜索 "github"（应该匹配 code 或 target）
        let filter = LinkFilter {
            search: Some("github".to_string()),
            ..Default::default()
        };

        let (links, total) = storage.load_paginated_filtered(1, 10, filter).await.unwrap();

        // github_1, github_2 的 code 包含 github
        // other 的 target 包含 github
        // google 不包含 github
        assert_eq!(total, 3);
        assert_eq!(links.len(), 3);
    }

    #[tokio::test]
    async fn test_load_paginated_filtered_only_active() {
        let (storage, _temp) = create_temp_storage().await;

        // 创建过期和未过期的链接
        storage
            .set(create_test_link("active_1", "https://example.com"))
            .await
            .unwrap(); // 无过期时间
        storage
            .set(create_test_link_with_expiry("active_2", Duration::hours(1)))
            .await
            .unwrap(); // 未来过期
        storage
            .set(create_test_link_with_expiry("expired", Duration::hours(-1)))
            .await
            .unwrap(); // 已过期

        let filter = LinkFilter {
            only_active: true,
            ..Default::default()
        };

        let (links, total) = storage.load_paginated_filtered(1, 10, filter).await.unwrap();

        assert_eq!(total, 2);
        let codes: Vec<&str> = links.iter().map(|l| l.code.as_str()).collect();
        assert!(codes.contains(&"active_1"));
        assert!(codes.contains(&"active_2"));
        assert!(!codes.contains(&"expired"));
    }

    #[tokio::test]
    async fn test_load_paginated_filtered_only_expired() {
        let (storage, _temp) = create_temp_storage().await;

        storage
            .set(create_test_link("active", "https://example.com"))
            .await
            .unwrap();
        storage
            .set(create_test_link_with_expiry("expired_1", Duration::hours(-1)))
            .await
            .unwrap();
        storage
            .set(create_test_link_with_expiry("expired_2", Duration::days(-1)))
            .await
            .unwrap();

        let filter = LinkFilter {
            only_expired: true,
            ..Default::default()
        };

        let (links, total) = storage.load_paginated_filtered(1, 10, filter).await.unwrap();

        assert_eq!(total, 2);
        let codes: Vec<&str> = links.iter().map(|l| l.code.as_str()).collect();
        assert!(codes.contains(&"expired_1"));
        assert!(codes.contains(&"expired_2"));
        assert!(!codes.contains(&"active"));
    }
}

// =============================================================================
// 统计测试
// =============================================================================

#[cfg(test)]
mod stats_tests {
    use super::*;

    #[tokio::test]
    async fn test_get_stats_empty() {
        let (storage, _temp) = create_temp_storage().await;

        let stats = storage.get_stats().await.unwrap();
        assert_eq!(stats.total_links, 0);
        assert_eq!(stats.total_clicks, 0);
        assert_eq!(stats.active_links, 0);
    }

    #[tokio::test]
    async fn test_get_stats_with_data() {
        let (storage, _temp) = create_temp_storage().await;

        // 创建 3 个活跃链接
        for i in 0..3 {
            storage
                .set(create_test_link(&format!("stat_{}", i), "https://example.com"))
                .await
                .unwrap();
        }

        // 创建 1 个过期链接
        storage
            .set(create_test_link_with_expiry("stat_expired", Duration::hours(-1)))
            .await
            .unwrap();

        let stats = storage.get_stats().await.unwrap();
        assert_eq!(stats.total_links, 4);
        assert_eq!(stats.active_links, 3);
    }

    #[tokio::test]
    async fn test_get_stats_with_clicks() {
        let (storage, _temp) = create_temp_storage().await;

        // 创建带点击数的链接
        let mut link = create_test_link("clicks_test", "https://example.com");
        link.click = 100;
        storage.set(link).await.unwrap();

        let mut link2 = create_test_link("clicks_test_2", "https://example.com");
        link2.click = 50;
        storage.set(link2).await.unwrap();

        let stats = storage.get_stats().await.unwrap();
        assert_eq!(stats.total_clicks, 150);
    }
}

// =============================================================================
// 边界条件测试
// =============================================================================

#[cfg(test)]
mod edge_case_tests {
    use super::*;

    #[tokio::test]
    async fn test_unicode_in_target() {
        let (storage, _temp) = create_temp_storage().await;

        let link = create_test_link("unicode", "https://例え.jp/路径/文件");
        storage.set(link).await.unwrap();

        let retrieved = storage.get("unicode").await.unwrap().unwrap();
        assert_eq!(retrieved.target, "https://例え.jp/路径/文件");
    }

    #[tokio::test]
    async fn test_long_url() {
        let (storage, _temp) = create_temp_storage().await;

        let long_url = format!("https://example.com/{}", "a".repeat(2000));
        let link = create_test_link("long_url", &long_url);
        storage.set(link).await.unwrap();

        let retrieved = storage.get("long_url").await.unwrap().unwrap();
        assert_eq!(retrieved.target, long_url);
    }

    #[tokio::test]
    async fn test_special_characters_in_code() {
        let (storage, _temp) = create_temp_storage().await;

        // 测试短码中的特殊字符（假设允许）
        let link = create_test_link("test-code_123", "https://example.com");
        storage.set(link).await.unwrap();

        let retrieved = storage.get("test-code_123").await.unwrap();
        assert!(retrieved.is_some());
    }

    #[tokio::test]
    async fn test_password_field() {
        let (storage, _temp) = create_temp_storage().await;

        let mut link = create_test_link("with_pass", "https://example.com");
        link.password = Some("secret_hash".to_string());
        storage.set(link).await.unwrap();

        let retrieved = storage.get("with_pass").await.unwrap().unwrap();
        assert_eq!(retrieved.password, Some("secret_hash".to_string()));
    }

    #[tokio::test]
    async fn test_load_all_filtered_export() {
        let (storage, _temp) = create_temp_storage().await;

        for i in 0..5 {
            storage
                .set(create_test_link(&format!("export_{}", i), "https://example.com"))
                .await
                .unwrap();
        }

        let filter = LinkFilter::default();
        let all = storage.load_all_filtered(filter).await.unwrap();
        assert_eq!(all.len(), 5);
    }
}
