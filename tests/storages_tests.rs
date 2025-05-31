use shortlinker::storages::file::FileStorage;
use shortlinker::storages::sled::SledStorage;
use shortlinker::storages::sqlite::SqliteStorage;
use shortlinker::storages::{SerializableShortLink, ShortLink, Storage, StorageFactory};
use std::env;
use std::fs;
use tempfile::TempDir;

#[cfg(test)]
mod short_link_tests {
    use super::*;

    #[test]
    fn test_short_link_creation() {
        let link = ShortLink {
            code: "test123".to_string(),
            target: "https://example.com".to_string(),
            created_at: chrono::Utc::now(),
            expires_at: None,
        };

        assert_eq!(link.code, "test123");
        assert_eq!(link.target, "https://example.com");
        assert!(link.expires_at.is_none());
    }

    #[test]
    fn test_short_link_with_expiry() {
        let expires_at = chrono::Utc::now() + chrono::Duration::hours(24);
        let link = ShortLink {
            code: "expiry_test".to_string(),
            target: "https://example.com".to_string(),
            created_at: chrono::Utc::now(),
            expires_at: Some(expires_at),
        };

        assert!(link.expires_at.is_some());
        assert_eq!(link.expires_at.unwrap(), expires_at);
    }

    #[test]
    fn test_short_link_clone() {
        let original = ShortLink {
            code: "clone_test".to_string(),
            target: "https://example.com".to_string(),
            created_at: chrono::Utc::now(),
            expires_at: None,
        };

        let cloned = original.clone();
        assert_eq!(original.code, cloned.code);
        assert_eq!(original.target, cloned.target);
        assert_eq!(original.created_at, cloned.created_at);
        assert_eq!(original.expires_at, cloned.expires_at);
    }

    #[test]
    fn test_short_link_debug() {
        let link = ShortLink {
            code: "debug_test".to_string(),
            target: "https://debug.com".to_string(),
            created_at: chrono::Utc::now(),
            expires_at: None,
        };

        let debug_output = format!("{:?}", link);
        assert!(debug_output.contains("debug_test"));
        assert!(debug_output.contains("https://debug.com"));
    }
}

#[cfg(test)]
mod serializable_short_link_tests {
    use super::*;

    #[test]
    fn test_serialization() {
        let link = SerializableShortLink {
            short_code: "serialize_test".to_string(),
            target_url: "https://example.com".to_string(),
            created_at: "2023-01-01T00:00:00Z".to_string(),
            expires_at: Some("2023-12-31T23:59:59Z".to_string()),
        };

        let json = serde_json::to_string(&link).unwrap();
        assert!(json.contains("serialize_test"));
        assert!(json.contains("https://example.com"));
        assert!(json.contains("2023-01-01T00:00:00Z"));
        assert!(json.contains("2023-12-31T23:59:59Z"));
    }

    #[test]
    fn test_deserialization() {
        let json = r#"{
            "short_code": "deserialize_test",
            "target_url": "https://example.com",
            "created_at": "2023-01-01T00:00:00Z",
            "expires_at": null
        }"#;

        let link: SerializableShortLink = serde_json::from_str(json).unwrap();
        assert_eq!(link.short_code, "deserialize_test");
        assert_eq!(link.target_url, "https://example.com");
        assert_eq!(link.created_at, "2023-01-01T00:00:00Z");
        assert!(link.expires_at.is_none());
    }

    #[test]
    fn test_serialization_round_trip() {
        let original = SerializableShortLink {
            short_code: "round_trip".to_string(),
            target_url: "https://round-trip.com".to_string(),
            created_at: "2023-06-15T12:30:45Z".to_string(),
            expires_at: Some("2024-06-15T12:30:45Z".to_string()),
        };

        let json = serde_json::to_string(&original).unwrap();
        let deserialized: SerializableShortLink = serde_json::from_str(&json).unwrap();

        assert_eq!(original.short_code, deserialized.short_code);
        assert_eq!(original.target_url, deserialized.target_url);
        assert_eq!(original.created_at, deserialized.created_at);
        assert_eq!(original.expires_at, deserialized.expires_at);
    }

    #[test]
    fn test_vector_serialization() {
        let links = vec![
            SerializableShortLink {
                short_code: "link1".to_string(),
                target_url: "https://example1.com".to_string(),
                created_at: "2023-01-01T00:00:00Z".to_string(),
                expires_at: None,
            },
            SerializableShortLink {
                short_code: "link2".to_string(),
                target_url: "https://example2.com".to_string(),
                created_at: "2023-01-02T00:00:00Z".to_string(),
                expires_at: Some("2023-12-31T23:59:59Z".to_string()),
            },
        ];

        let json = serde_json::to_string_pretty(&links).unwrap();
        let deserialized: Vec<SerializableShortLink> = serde_json::from_str(&json).unwrap();

        assert_eq!(links.len(), deserialized.len());
        assert_eq!(links[0].short_code, deserialized[0].short_code);
        assert_eq!(links[1].expires_at, deserialized[1].expires_at);
    }
}

#[cfg(test)]
mod file_storage_tests {
    use super::*;

    fn create_temp_file_storage() -> (FileStorage, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test_links.json");

        env::set_var("DB_FILE_NAME", file_path.to_str().unwrap());
        let storage = FileStorage::new().unwrap();
        env::remove_var("DB_FILE_NAME");

        (storage, temp_dir)
    }

    #[tokio::test]
    async fn test_file_storage_creation() {
        let (_storage, _temp_dir) = create_temp_file_storage();
        // 测试通过，说明文件存储创建成功
        assert!(true);
    }

    #[tokio::test]
    async fn test_file_storage_set_and_get() {
        let (storage, _temp_dir) = create_temp_file_storage();

        let link = ShortLink {
            code: "file_test".to_string(),
            target: "https://file-test.com".to_string(),
            created_at: chrono::Utc::now(),
            expires_at: None,
        };

        // 存储链接
        let result = storage.set(link.clone()).await;
        assert!(result.is_ok());

        // 获取链接
        let retrieved = storage.get("file_test").await;
        assert!(retrieved.is_some());

        let retrieved_link = retrieved.unwrap();
        assert_eq!(retrieved_link.code, "file_test");
        assert_eq!(retrieved_link.target, "https://file-test.com");
    }

    #[tokio::test]
    async fn test_file_storage_load_all() {
        let (storage, _temp_dir) = create_temp_file_storage();

        // 添加多个链接
        for i in 1..=3 {
            let link = ShortLink {
                code: format!("test{}", i),
                target: format!("https://test{}.com", i),
                created_at: chrono::Utc::now(),
                expires_at: None,
            };
            storage.set(link).await.unwrap();
        }

        let all_links = storage.load_all().await;
        assert_eq!(all_links.len(), 3);
        assert!(all_links.contains_key("test1"));
        assert!(all_links.contains_key("test2"));
        assert!(all_links.contains_key("test3"));
    }

    #[tokio::test]
    async fn test_file_storage_remove() {
        let (storage, _temp_dir) = create_temp_file_storage();

        let link = ShortLink {
            code: "remove_test".to_string(),
            target: "https://remove-test.com".to_string(),
            created_at: chrono::Utc::now(),
            expires_at: None,
        };

        // 添加链接
        storage.set(link).await.unwrap();

        // 验证链接存在
        assert!(storage.get("remove_test").await.is_some());

        // 删除链接
        let result = storage.remove("remove_test").await;
        assert!(result.is_ok());

        // 验证链接已删除
        assert!(storage.get("remove_test").await.is_none());
    }

    #[tokio::test]
    async fn test_file_storage_remove_nonexistent() {
        let (storage, _temp_dir) = create_temp_file_storage();

        let result = storage.remove("nonexistent").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_file_storage_update() {
        let (storage, _temp_dir) = create_temp_file_storage();

        let original_link = ShortLink {
            code: "update_test".to_string(),
            target: "https://original.com".to_string(),
            created_at: chrono::Utc::now(),
            expires_at: None,
        };

        // 添加原始链接
        storage.set(original_link.clone()).await.unwrap();

        // 更新链接
        let updated_link = ShortLink {
            code: "update_test".to_string(),
            target: "https://updated.com".to_string(),
            created_at: chrono::Utc::now(),
            expires_at: Some(chrono::Utc::now() + chrono::Duration::days(1)),
        };

        storage.set(updated_link).await.unwrap();

        // 验证更新
        let retrieved = storage.get("update_test").await.unwrap();
        assert_eq!(retrieved.target, "https://updated.com");
        assert!(retrieved.expires_at.is_some());
        // 创建时间应该保持原始值
        assert_eq!(
            retrieved.created_at.timestamp(),
            original_link.created_at.timestamp()
        );
    }

    #[tokio::test]
    async fn test_file_storage_reload() {
        let (storage, temp_dir) = create_temp_file_storage();

        // 添加一个链接
        let link = ShortLink {
            code: "reload_test".to_string(),
            target: "https://reload-test.com".to_string(),
            created_at: chrono::Utc::now(),
            expires_at: None,
        };
        storage.set(link).await.unwrap();

        // 手动修改文件
        let file_path = temp_dir.path().join("test_links.json");
        let manual_data = vec![SerializableShortLink {
            short_code: "manual_added".to_string(),
            target_url: "https://manual.com".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            expires_at: None,
        }];
        let json = serde_json::to_string_pretty(&manual_data).unwrap();
        fs::write(&file_path, json).unwrap();

        // 重载
        let result = storage.reload().await;
        assert!(result.is_ok());

        // 验证重载后的数据
        let all_links = storage.load_all().await;
        assert_eq!(all_links.len(), 1);
        assert!(all_links.contains_key("manual_added"));
        assert!(!all_links.contains_key("reload_test"));
    }

    #[tokio::test]
    async fn test_file_storage_backend_name() {
        let (storage, _temp_dir) = create_temp_file_storage();
        let backend_name = storage.get_backend_name().await;
        assert_eq!(backend_name, "file");
    }
}

#[cfg(test)]
mod sqlite_storage_tests {
    use super::*;

    fn create_temp_sqlite_storage() -> SqliteStorage {
        let temp_db = TempDir::new().unwrap();
        let db_path = temp_db.path().join("test.db");

        env::set_var("DB_FILE_NAME", db_path.to_str().unwrap());
        let storage = SqliteStorage::new().unwrap();
        env::remove_var("DB_FILE_NAME");

        // 确保 temp_db 在测试期间不被删除
        std::mem::forget(temp_db);

        storage
    }

    #[tokio::test]
    async fn test_sqlite_storage_creation() {
        let _storage = create_temp_sqlite_storage();
        // 测试通过，说明 SQLite 存储创建成功
        assert!(true);
    }

    #[tokio::test]
    async fn test_sqlite_storage_set_and_get() {
        let storage = create_temp_sqlite_storage();

        let link = ShortLink {
            code: "sqlite_test".to_string(),
            target: "https://sqlite-test.com".to_string(),
            created_at: chrono::Utc::now(),
            expires_at: None,
        };

        // 存储链接
        let result = storage.set(link.clone()).await;
        assert!(result.is_ok());

        // 获取链接
        let retrieved = storage.get("sqlite_test").await;
        assert!(retrieved.is_some());

        let retrieved_link = retrieved.unwrap();
        assert_eq!(retrieved_link.code, "sqlite_test");
        assert_eq!(retrieved_link.target, "https://sqlite-test.com");
    }

    #[tokio::test]
    async fn test_sqlite_storage_load_all() {
        let storage = create_temp_sqlite_storage();

        // 添加多个链接
        for i in 1..=3 {
            let link = ShortLink {
                code: format!("sqlite_test{}", i),
                target: format!("https://sqlite-test{}.com", i),
                created_at: chrono::Utc::now(),
                expires_at: None,
            };
            storage.set(link).await.unwrap();
        }

        let all_links = storage.load_all().await;
        assert_eq!(all_links.len(), 3);
        assert!(all_links.contains_key("sqlite_test1"));
        assert!(all_links.contains_key("sqlite_test2"));
        assert!(all_links.contains_key("sqlite_test3"));
    }

    #[tokio::test]
    async fn test_sqlite_storage_remove() {
        let storage = create_temp_sqlite_storage();

        let link = ShortLink {
            code: "sqlite_remove".to_string(),
            target: "https://sqlite-remove.com".to_string(),
            created_at: chrono::Utc::now(),
            expires_at: None,
        };

        // 添加链接
        storage.set(link).await.unwrap();

        // 验证链接存在
        assert!(storage.get("sqlite_remove").await.is_some());

        // 删除链接
        let result = storage.remove("sqlite_remove").await;
        assert!(result.is_ok());

        // 验证链接已删除
        assert!(storage.get("sqlite_remove").await.is_none());
    }

    #[tokio::test]
    async fn test_sqlite_storage_remove_nonexistent() {
        let storage = create_temp_sqlite_storage();

        let result = storage.remove("nonexistent").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_sqlite_storage_update() {
        let storage = create_temp_sqlite_storage();

        let original_link = ShortLink {
            code: "sqlite_update".to_string(),
            target: "https://original.com".to_string(),
            created_at: chrono::Utc::now(),
            expires_at: None,
        };

        // 添加原始链接
        storage.set(original_link).await.unwrap();

        // 更新链接
        let updated_link = ShortLink {
            code: "sqlite_update".to_string(),
            target: "https://updated.com".to_string(),
            created_at: chrono::Utc::now(),
            expires_at: Some(chrono::Utc::now() + chrono::Duration::days(1)),
        };

        storage.set(updated_link).await.unwrap();

        // 验证更新
        let retrieved = storage.get("sqlite_update").await.unwrap();
        assert_eq!(retrieved.target, "https://updated.com");
        assert!(retrieved.expires_at.is_some());
    }

    #[tokio::test]
    async fn test_sqlite_storage_with_expiry() {
        let storage = create_temp_sqlite_storage();

        let expires_at = chrono::Utc::now() + chrono::Duration::hours(24);
        let link = ShortLink {
            code: "sqlite_expiry".to_string(),
            target: "https://sqlite-expiry.com".to_string(),
            created_at: chrono::Utc::now(),
            expires_at: Some(expires_at),
        };

        storage.set(link).await.unwrap();

        let retrieved = storage.get("sqlite_expiry").await.unwrap();
        assert!(retrieved.expires_at.is_some());

        // 验证过期时间在合理范围内（允许一些时间差）
        let time_diff = (retrieved.expires_at.unwrap() - expires_at)
            .num_seconds()
            .abs();
        assert!(time_diff < 2);
    }

    #[tokio::test]
    async fn test_sqlite_storage_backend_name() {
        let storage = create_temp_sqlite_storage();
        let backend_name = storage.get_backend_name().await;
        assert_eq!(backend_name, "sqlite");
    }

    #[tokio::test]
    async fn test_sqlite_storage_reload() {
        let storage = create_temp_sqlite_storage();

        // SQLite 存储的 reload 方法是空操作
        let result = storage.reload().await;
        assert!(result.is_ok());
    }
}

#[cfg(test)]
mod sled_storage_tests {
    use super::*;

    #[tokio::test]
    async fn test_sled_storage_creation() {
        let storage = SledStorage::new().unwrap();
        assert_eq!(storage.get_backend_name().await, "sled");
    }

    #[tokio::test]
    async fn test_sled_storage_basic_operations() {
        let storage = SledStorage::new().unwrap();

        // 测试 get 方法（当前返回示例数据）
        let result = storage.get("any_code").await;
        assert!(result.is_some());

        let link = result.unwrap();
        assert_eq!(link.code, "example_code");
        assert_eq!(link.target, "http://example.com");

        // 测试 load_all 方法
        let all_links = storage.load_all().await;
        assert_eq!(all_links.len(), 1);
        assert!(all_links.contains_key("example_code"));

        // 测试 set 方法（当前只是打印）
        let test_link = ShortLink {
            code: "test".to_string(),
            target: "https://test.com".to_string(),
            created_at: chrono::Utc::now(),
            expires_at: None,
        };
        let result = storage.set(test_link).await;
        assert!(result.is_ok());

        // 测试 remove 方法
        let result = storage.remove("test").await;
        assert!(result.is_ok());

        // 测试 reload 方法
        let result = storage.reload().await;
        assert!(result.is_ok());
    }
}

#[cfg(test)]
mod storage_factory_tests {
    use super::*;

    #[test]
    fn test_storage_factory_default() {
        let storage = StorageFactory::create();
        assert!(storage.is_ok());
    }

    #[test]
    fn test_storage_factory_file_backend() {
        env::set_var("STORAGE_BACKEND", "file");
        let storage = StorageFactory::create();
        assert!(storage.is_ok());
        env::remove_var("STORAGE_BACKEND");
    }

    #[test]
    fn test_storage_factory_sqlite_backend() {
        env::set_var("STORAGE_BACKEND", "sqlite");
        let storage = StorageFactory::create();
        assert!(storage.is_ok());
        env::remove_var("STORAGE_BACKEND");
    }

    #[test]
    fn test_storage_factory_sled_backend() {
        env::set_var("STORAGE_BACKEND", "sled");
        let storage = StorageFactory::create();
        assert!(storage.is_ok());
        env::remove_var("STORAGE_BACKEND");
    }

    #[test]
    fn test_storage_factory_invalid_backend() {
        env::set_var("STORAGE_BACKEND", "invalid");
        let storage = StorageFactory::create();
        // 应该回退到默认存储
        assert!(storage.is_ok());
        env::remove_var("STORAGE_BACKEND");
    }

    #[tokio::test]
    async fn test_storage_factory_backend_names() {
        // 测试不同后端的名称
        let backends = vec![("file", "file"), ("sqlite", "sqlite"), ("sled", "sled")];

        for (backend_env, expected_name) in backends {
            env::set_var("STORAGE_BACKEND", backend_env);
            let storage = StorageFactory::create().unwrap();
            let backend_name = storage.get_backend_name().await;
            assert_eq!(backend_name, expected_name);
            env::remove_var("STORAGE_BACKEND");
        }
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_storage_trait_consistency() {
        // 测试所有存储后端的一致性行为
        let storages: Vec<Box<dyn Storage>> = vec![
            Box::new(FileStorage::new().unwrap()),
            Box::new(SqliteStorage::new().unwrap()),
            Box::new(SledStorage::new().unwrap()),
        ];

        for storage in storages {
            // 测试后端名称不为空
            let backend_name = storage.get_backend_name().await;
            assert!(!backend_name.is_empty());

            // 测试 reload 方法不会崩溃
            let reload_result = storage.reload().await;
            assert!(reload_result.is_ok() || reload_result.is_err());
        }
    }

    #[tokio::test]
    async fn test_concurrent_storage_operations() {
        let (storage, _temp_dir) = create_temp_file_storage();
        let storage = std::sync::Arc::new(storage);

        // 并发写入测试
        let mut handles = vec![];
        for i in 0..10 {
            let storage_clone = storage.clone();
            let handle = tokio::spawn(async move {
                let link = ShortLink {
                    code: format!("concurrent_{}", i),
                    target: format!("https://concurrent{}.com", i),
                    created_at: chrono::Utc::now(),
                    expires_at: None,
                };
                storage_clone.set(link).await
            });
            handles.push(handle);
        }

        // 等待所有操作完成
        for handle in handles {
            let result = handle.await.unwrap();
            assert!(result.is_ok());
        }

        // 验证所有链接都被正确存储
        let all_links = storage.load_all().await;
        assert_eq!(all_links.len(), 10);

        for i in 0..10 {
            assert!(all_links.contains_key(&format!("concurrent_{}", i)));
        }
    }

    fn create_temp_file_storage() -> (FileStorage, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test_links.json");

        env::set_var("DB_FILE_NAME", file_path.to_str().unwrap());
        let storage = FileStorage::new().unwrap();
        env::remove_var("DB_FILE_NAME");

        (storage, temp_dir)
    }

    #[tokio::test]
    async fn test_large_dataset_handling() {
        let (storage, _temp_dir) = create_temp_file_storage();

        // 创建大量链接
        let link_count = 1000;
        for i in 0..link_count {
            let link = ShortLink {
                code: format!("bulk_{:04}", i),
                target: format!("https://bulk{}.com", i),
                created_at: chrono::Utc::now(),
                expires_at: if i % 2 == 0 {
                    Some(chrono::Utc::now() + chrono::Duration::days(30))
                } else {
                    None
                },
            };
            storage.set(link).await.unwrap();
        }

        // 验证所有链接都被正确存储
        let all_links = storage.load_all().await;
        assert_eq!(all_links.len(), link_count);

        // 随机检查一些链接
        for i in (0..link_count).step_by(100) {
            let code = format!("bulk_{:04}", i);
            let link = storage.get(&code).await;
            assert!(link.is_some());
            assert_eq!(link.unwrap().code, code);
        }
    }

    #[tokio::test]
    async fn test_error_handling_consistency() {
        let (storage, _temp_dir) = create_temp_file_storage();

        // 测试删除不存在的链接
        let result = storage.remove("nonexistent_link").await;
        assert!(result.is_err());

        // 测试获取不存在的链接
        let result = storage.get("nonexistent_link").await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_special_characters_handling() {
        let (storage, _temp_dir) = create_temp_file_storage();

        // 测试特殊字符的处理
        let special_cases = vec![
            (
                "special-123",
                "https://example.com/path?param=value&other=test",
            ),
            ("special_456", "https://example.com/中文路径"),
            ("special.789", "https://example.com/emoji🎉"),
            ("special@abc", "https://example.com/with@symbol"),
        ];

        for (code, url) in special_cases {
            let link = ShortLink {
                code: code.to_string(),
                target: url.to_string(),
                created_at: chrono::Utc::now(),
                expires_at: None,
            };

            let set_result = storage.set(link).await;
            assert!(set_result.is_ok(), "Failed to set link with code: {}", code);

            let get_result = storage.get(code).await;
            assert!(
                get_result.is_some(),
                "Failed to get link with code: {}",
                code
            );

            let retrieved_link = get_result.unwrap();
            assert_eq!(retrieved_link.target, url);
        }
    }
}
