#![cfg(test)]

use shortlinker::storages::{file::FileStorage, ShortLink, Storage};
use shortlinker::utils::generate_random_code;
use std::env;
use std::time::Instant;
use tempfile::TempDir;

fn setup_test_env() -> TempDir {
    let temp_dir = TempDir::new().unwrap();
    let links_file = temp_dir.path().join("benchmark_links.json");
    env::set_var("LINKS_FILE", links_file.to_str().unwrap());
    env::set_var("STORAGE_BACKEND", "file"); // 强制使用文件存储
    temp_dir
}

#[tokio::test]
async fn benchmark_random_code_generation() {
    let _temp_dir = setup_test_env();
    let start = Instant::now();
    let mut codes = Vec::new();

    for _ in 0..10000 {
        codes.push(generate_random_code(8));
    }

    let duration = start.elapsed();
    println!("Generated 10,000 random codes in: {:?}", duration);

    // 验证没有重复（在统计上应该很少重复）
    let mut unique_codes = std::collections::HashSet::new();
    for code in codes {
        unique_codes.insert(code);
    }

    assert!(
        unique_codes.len() > 9900,
        "Too many duplicate codes generated"
    );
}

#[tokio::test]
async fn benchmark_storage_operations() {
    let _temp_dir = setup_test_env();
    let storage = FileStorage::new();

    // 基准测试：插入操作
    let start = Instant::now();
    for i in 0..1000 {
        let link = ShortLink {
            code: format!("code{}", i),
            target: format!("https://example.com/{}", i),
            created_at: chrono::Utc::now(),
            expires_at: None,
        };
        storage.set(link).await.unwrap();
    }
    let insert_duration = start.elapsed();
    println!("Inserted 1,000 links in: {:?}", insert_duration);

    // 基准测试：查询操作
    let start = Instant::now();
    for i in 0..1000 {
        let _ = storage.get(&format!("code{}", i)).await;
    }
    let query_duration = start.elapsed();
    println!("Queried 1,000 links in: {:?}", query_duration);

    // 基准测试：加载所有链接
    let start = Instant::now();
    let all_links = storage.load_all().await;
    let load_all_duration = start.elapsed();
    println!(
        "Loaded all {} links in: {:?}",
        all_links.len(),
        load_all_duration
    );

    assert_eq!(all_links.len(), 1000);
}
