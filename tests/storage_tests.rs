// 本测试存在部分问题，需要等待解决

use std::env;
use tempfile::TempDir;

// 导入实际的存储代码
use shortlinker::storages::{STORAGE, ShortLink};

fn setup_test_env() -> TempDir {
    let temp_dir = TempDir::new().unwrap();
    let links_file = temp_dir.path().join("test_storage_links.json");
    env::set_var("LINKS_FILE", links_file.to_str().unwrap());
    env::set_var("STORAGE_BACKEND", "file"); // 强制使用文件存储
    temp_dir
}

fn create_test_link(code: &str, target: &str) -> ShortLink {
    ShortLink {
        code: code.to_string(),
        target: target.to_string(),
        created_at: chrono::Utc::now(),
        expires_at: None,
    }
}

fn create_expired_link(code: &str, target: &str) -> ShortLink {
    ShortLink {
        code: code.to_string(),
        target: target.to_string(),
        created_at: chrono::Utc::now() - chrono::Duration::hours(2),
        expires_at: Some(chrono::Utc::now() - chrono::Duration::hours(1)),
    }
}

#[tokio::test]
async fn test_storage_basic_operations() {
    let _temp_dir = setup_test_env();

    // 测试保存和获取
    let test_link = create_test_link("test", "https://example.com");
    assert!(STORAGE.set(test_link.clone()).await.is_ok());

    let retrieved = STORAGE.get("test").await;
    assert!(retrieved.is_some());
    let link = retrieved.unwrap();
    assert_eq!(link.code, "test");
    assert_eq!(link.target, "https://example.com");

    // 测试删除
    assert!(STORAGE.remove("test").await.is_ok());
    assert!(STORAGE.get("test").await.is_none());
}

#[tokio::test]
async fn test_storage_persistence() {
    let temp_dir = setup_test_env();
    let storage_path = temp_dir.path().join("persist_links.json");

    // 第一个存储实例
    {
        env::set_var("LINKS_FILE", storage_path.to_str().unwrap());
        let link = create_test_link("persist", "https://persist.com");
        STORAGE.set(link).await.unwrap();
    }

    // 第二个存储实例应该能读取到数据
    {
        env::set_var("LINKS_FILE", storage_path.to_str().unwrap());
        let retrieved = STORAGE.get("persist").await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().target, "https://persist.com");
    }
}

#[tokio::test]
async fn test_storage_multiple_links() {
    let _temp_dir = setup_test_env();

    // 添加多个链接
    for i in 0..10 {
        let link = create_test_link(&format!("code{}", i), &format!("https://example{}.com", i));
        STORAGE.set(link).await.unwrap();
    }

    // 验证所有链接都存在
    for i in 0..10 {
        let retrieved = STORAGE.get(&format!("code{}", i)).await;
        assert!(retrieved.is_some());
    }

    // 删除一半链接
    for i in 0..5 {
        STORAGE.remove(&format!("code{}", i)).await.unwrap();
    }

    // 验证删除结果
    for i in 0..5 {
        assert!(STORAGE.get(&format!("code{}", i)).await.is_none());
    }
    for i in 5..10 {
        assert!(STORAGE.get(&format!("code{}", i)).await.is_some());
    }
}

#[tokio::test]
async fn test_storage_expired_links() {
    let _temp_dir = setup_test_env();

    let expired_link = create_expired_link("expired", "https://expired.com");
    STORAGE.set(expired_link).await.unwrap();

    let retrieved = STORAGE.get("expired").await;
    assert!(retrieved.is_some());

    let link = retrieved.unwrap();
    assert!(link.expires_at.is_some());
    assert!(link.expires_at.unwrap() < chrono::Utc::now());
}

#[tokio::test]
async fn test_storage_load_all() {
    let _temp_dir = setup_test_env();

    // 添加多个链接
    for i in 0..5 {
        let link = create_test_link(&format!("code{}", i), &format!("https://example{}.com", i));
        STORAGE.set(link).await.unwrap();
    }

    let all_links = STORAGE.load_all().await;
    assert_eq!(all_links.len(), 5);

    for i in 0..5 {
        assert!(all_links.contains_key(&format!("code{}", i)));
    }
}

#[tokio::test]
async fn test_storage_overwrite() {
    let _temp_dir = setup_test_env();

    // 创建原始链接
    let original_link = create_test_link("test", "https://original.com");
    STORAGE.set(original_link).await.unwrap();

    // 覆盖链接
    let updated_link = create_test_link("test", "https://updated.com");
    STORAGE.set(updated_link).await.unwrap();

    // 验证已更新
    let retrieved = STORAGE.get("test").await.unwrap();
    assert_eq!(retrieved.target, "https://updated.com");
}

#[tokio::test]
async fn test_storage_reload() {
    let _temp_dir = setup_test_env();

    // 添加测试数据
    let link = create_test_link("reload_test", "https://reload.com");
    STORAGE.set(link).await.unwrap();

    // 测试重载
    assert!(STORAGE.reload().await.is_ok());

    // 验证数据仍存在
    let retrieved = STORAGE.get("reload_test").await;
    assert!(retrieved.is_some());
}

#[tokio::test]
async fn test_storage_nonexistent_operations() {
    let _temp_dir = setup_test_env();

    // 获取不存在的链接
    assert!(STORAGE.get("nonexistent").await.is_none());

    // 删除不存在的链接应该成功
    assert!(STORAGE.remove("nonexistent").await.is_ok());
}

#[tokio::test]
async fn test_storage_empty_state() {
    let _temp_dir = setup_test_env();

    // 测试空状态下的操作
    let all_links = STORAGE.load_all().await;
    assert!(all_links.is_empty());

    assert!(STORAGE.get("anything").await.is_none());
}

#[tokio::test]
async fn test_storage_large_dataset() {
    let _temp_dir = setup_test_env();

    // 测试大量数据
    let count = 100;
    for i in 0..count {
        let link = create_test_link(&format!("bulk{}", i), &format!("https://bulk{}.com", i));
        STORAGE.set(link).await.unwrap();
    }

    let all_links = STORAGE.load_all().await;
    assert_eq!(all_links.len(), count);

    // 随机检查一些链接
    for i in [0, 25, 50, 75, 99] {
        let retrieved = STORAGE.get(&format!("bulk{}", i)).await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().target, format!("https://bulk{}.com", i));
    }
}
