//! CLI 模块测试
//!
//! 测试 CLI 命令的核心功能：配置文件生成、配置管理、链接管理。

use shortlinker::config::init_config;
use shortlinker::config::runtime_config::init_runtime_config;
use shortlinker::interfaces::cli::commands::config_management;
use shortlinker::interfaces::cli::commands::{add_link, list_links, remove_link, update_link};
use shortlinker::metrics_core::NoopMetrics;
use shortlinker::storage::backend::{SeaOrmStorage, connect_sqlite, run_migrations};
use std::sync::{Arc, Once};
use tempfile::TempDir;

// =============================================================================
// 全局初始化
// =============================================================================

static INIT: Once = Once::new();
static TEST_DIR: std::sync::OnceLock<TempDir> = std::sync::OnceLock::new();
static RT_INIT: tokio::sync::OnceCell<()> = tokio::sync::OnceCell::const_new();

fn init_static_config() {
    INIT.call_once(|| {
        init_config();
    });
}

async fn init_test_runtime_config() {
    init_static_config();
    RT_INIT
        .get_or_init(|| async {
            let td = TempDir::new().unwrap();
            let p = td.path().join("cli_rt.db");
            let u = format!("sqlite://{}?mode=rwc", p.display());
            let db = connect_sqlite(&u).await.unwrap();
            run_migrations(&db).await.unwrap();
            init_runtime_config(db).await.unwrap();
            let _ = TEST_DIR.set(td);
        })
        .await;
}

async fn create_temp_storage() -> (Arc<SeaOrmStorage>, TempDir) {
    init_test_runtime_config().await;
    let td = TempDir::new().unwrap();
    let p = td.path().join("cli_test.db");
    let u = format!("sqlite://{}?mode=rwc", p.display());
    let s = SeaOrmStorage::new(&u, "sqlite", NoopMetrics::arc())
        .await
        .unwrap();
    (Arc::new(s), td)
}

// =============================================================================
// 配置文件生成测试
// =============================================================================

#[cfg(test)]
mod config_gen_tests {
    use super::*;

    #[tokio::test]
    async fn test_config_generate_creates_file() {
        init_static_config();
        let td = TempDir::new().unwrap();
        let path = td
            .path()
            .join("test_config.toml")
            .to_string_lossy()
            .to_string();

        let result = config_management::config_generate(Some(path.clone()), true).await;
        assert!(result.is_ok(), "config_generate 失败: {:?}", result);
        assert!(std::path::Path::new(&path).exists());

        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("[server]"));
        assert!(content.contains("[database]"));
    }

    #[tokio::test]
    async fn test_config_generate_force_overwrite() {
        init_static_config();
        let td = TempDir::new().unwrap();
        let path = td
            .path()
            .join("existing.toml")
            .to_string_lossy()
            .to_string();

        std::fs::write(&path, "old content").unwrap();
        let result = config_management::config_generate(Some(path.clone()), true).await;
        assert!(result.is_ok());

        let content = std::fs::read_to_string(&path).unwrap();
        assert_ne!(content, "old content");
        assert!(content.contains("[server]"));
    }
}

// =============================================================================
// 配置管理命令测试
// =============================================================================

#[cfg(test)]
mod config_management_tests {
    use super::*;

    async fn create_temp_db() -> (sea_orm::DatabaseConnection, TempDir) {
        init_static_config();
        let td = TempDir::new().unwrap();
        let p = td.path().join("cfg_test.db");
        let u = format!("sqlite://{}?mode=rwc", p.display());
        let db = connect_sqlite(&u).await.unwrap();
        run_migrations(&db).await.unwrap();
        (db, td)
    }

    #[tokio::test]
    async fn test_config_list_all() {
        let (db, _td) = create_temp_db().await;
        let result = config_management::config_list(db, None, false).await;
        assert!(result.is_ok(), "config_list 失败: {:?}", result);
    }

    #[tokio::test]
    async fn test_config_list_by_category() {
        let (db, _td) = create_temp_db().await;
        let result = config_management::config_list(db, Some("auth".to_string()), false).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_config_list_json() {
        let (db, _td) = create_temp_db().await;
        let result = config_management::config_list(db, None, true).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_config_get_known_key() {
        let (db, _td) = create_temp_db().await;
        let result =
            config_management::config_get(db, "features.random_code_length".to_string(), false)
                .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_config_get_unknown_key() {
        let (db, _td) = create_temp_db().await;
        let result = config_management::config_get(db, "nonexistent.key".to_string(), false).await;
        // 未知 key 应返回错误
        assert!(result.is_err());
    }
}

// =============================================================================
// 链接管理命令测试
// =============================================================================

#[cfg(test)]
mod link_management_tests {
    use super::*;

    #[tokio::test]
    async fn test_add_and_list_links() {
        let (storage, _td) = create_temp_storage().await;

        let result = add_link(
            storage.clone(),
            Some("cli-test1".to_string()),
            "https://example.com/cli".to_string(),
            false,
            None,
            None,
        )
        .await;
        assert!(result.is_ok(), "add_link 失败: {:?}", result);

        let result = list_links(storage).await;
        assert!(result.is_ok(), "list_links 失败: {:?}", result);
    }

    #[tokio::test]
    async fn test_add_link_auto_code() {
        let (storage, _td) = create_temp_storage().await;

        let result = add_link(
            storage,
            None, // 自动生成 code
            "https://example.com/auto".to_string(),
            false,
            None,
            None,
        )
        .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_update_link() {
        let (storage, _td) = create_temp_storage().await;

        // 先创建
        add_link(
            storage.clone(),
            Some("cli-upd".to_string()),
            "https://example.com/old".to_string(),
            false,
            None,
            None,
        )
        .await
        .unwrap();

        // 更新
        let result = update_link(
            storage,
            "cli-upd".to_string(),
            "https://example.com/new".to_string(),
            None,
            None,
        )
        .await;
        assert!(result.is_ok(), "update_link 失败: {:?}", result);
    }

    #[tokio::test]
    async fn test_remove_link() {
        let (storage, _td) = create_temp_storage().await;

        add_link(
            storage.clone(),
            Some("cli-del".to_string()),
            "https://example.com/del".to_string(),
            false,
            None,
            None,
        )
        .await
        .unwrap();

        let result = remove_link(storage, "cli-del".to_string()).await;
        assert!(result.is_ok(), "remove_link 失败: {:?}", result);
    }

    #[tokio::test]
    async fn test_remove_nonexistent_link() {
        let (storage, _td) = create_temp_storage().await;
        let result = remove_link(storage, "nonexistent".to_string()).await;
        // 删除不存在的链接应返回错误
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_add_duplicate_without_force() {
        let (storage, _td) = create_temp_storage().await;

        add_link(
            storage.clone(),
            Some("cli-dup".to_string()),
            "https://example.com/first".to_string(),
            false,
            None,
            None,
        )
        .await
        .unwrap();

        // 不使用 force 重复添加应失败
        let result = add_link(
            storage,
            Some("cli-dup".to_string()),
            "https://example.com/second".to_string(),
            false,
            None,
            None,
        )
        .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_add_duplicate_with_force() {
        let (storage, _td) = create_temp_storage().await;

        add_link(
            storage.clone(),
            Some("cli-force".to_string()),
            "https://example.com/first".to_string(),
            false,
            None,
            None,
        )
        .await
        .unwrap();

        // 使用 force 覆盖
        let result = add_link(
            storage,
            Some("cli-force".to_string()),
            "https://example.com/second".to_string(),
            true,
            None,
            None,
        )
        .await;
        assert!(result.is_ok());
    }
}
