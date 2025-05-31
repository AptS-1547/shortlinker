use std::collections::HashSet;
use std::env;
use std::time::Instant;
use tempfile::TempDir;

// 导入实际的项目代码
use shortlinker::storages::{ShortLink, STORAGE};
use shortlinker::utils::generate_random_code;

fn setup_test_env() -> TempDir {
    let temp_dir = TempDir::new().unwrap();
    let links_file = temp_dir.path().join("performance_links.json");
    env::set_var("LINKS_FILE", links_file.to_str().unwrap());
    env::set_var("SQLITE_DB_PATH", links_file.to_str().unwrap());
    env::set_var("STORAGE_BACKEND", "file"); // 强制使用文件存储
    temp_dir
}

#[test]
fn benchmark_random_code_generation() {
    let start = Instant::now();
    let mut codes = Vec::new();

    for _ in 0..10000 {
        codes.push(generate_random_code(8));
    }

    let duration = start.elapsed();
    println!("Generated 10,000 random codes in: {:?}", duration);

    // 验证唯一性
    let unique_codes: HashSet<_> = codes.into_iter().collect();
    assert!(
        unique_codes.len() > 9500,
        "Not enough unique codes generated"
    );

    // 性能断言：应该在合理时间内完成
    assert!(
        duration.as_millis() < 1000,
        "Code generation too slow: {:?}",
        duration
    );
}

#[test]
fn benchmark_code_uniqueness() {
    let mut codes = HashSet::new();
    let start = Instant::now();

    // 生成大量代码测试唯一性
    for _ in 0..100000 {
        codes.insert(generate_random_code(8));
    }

    let duration = start.elapsed();
    println!("Generated {} unique codes in: {:?}", codes.len(), duration);

    // 验证唯一性比例
    let uniqueness_ratio = codes.len() as f64 / 100000.0;
    assert!(
        uniqueness_ratio > 0.99,
        "Uniqueness ratio too low: {}",
        uniqueness_ratio
    );
}

#[test]
fn benchmark_different_code_lengths() {
    let lengths = vec![4, 6, 8, 10, 12, 16];

    for length in lengths {
        let start = Instant::now();

        for _ in 0..1000 {
            generate_random_code(length);
        }

        let duration = start.elapsed();
        println!(
            "Generated 1,000 codes of length {} in: {:?}",
            length, duration
        );

        // 长度不应该显著影响性能
        assert!(
            duration.as_millis() < 500,
            "Code generation for length {} too slow",
            length
        );
    }
}

#[tokio::test]
async fn benchmark_storage_operations() {
    let _temp_dir = setup_test_env();

    // 基准测试：插入操作
    let start = Instant::now();
    for i in 0..1000 {
        let link = ShortLink {
            code: format!("code{}", i),
            target: format!("https://example.com/{}", i),
            created_at: chrono::Utc::now(),
            expires_at: None,
        };
        STORAGE.set(link).await.unwrap();
    }
    let insert_duration = start.elapsed();
    println!("Inserted 1,000 links in: {:?}", insert_duration);

    // 基准测试：查询操作
    let start = Instant::now();
    for i in 0..1000 {
        let _ = STORAGE.get(&format!("code{}", i)).await;
    }
    let query_duration = start.elapsed();
    println!("Queried 1,000 links in: {:?}", query_duration);

    // 基准测试：加载所有链接
    let start = Instant::now();
    let all_links = STORAGE.load_all().await;
    let load_all_duration = start.elapsed();
    println!(
        "Loaded all {} links in: {:?}",
        all_links.len(),
        load_all_duration
    );

    assert_eq!(all_links.len(), 1000);

    // 性能断言
    assert!(
        insert_duration.as_millis() < 5000,
        "Insert operations too slow"
    );
    assert!(
        query_duration.as_millis() < 2000,
        "Query operations too slow"
    );
    assert!(
        load_all_duration.as_millis() < 1000,
        "Load all operation too slow"
    );
}

#[test]
fn stress_test_concurrent_generation() {
    use std::sync::{Arc, Mutex};
    use std::thread;

    let codes = Arc::new(Mutex::new(HashSet::new()));
    let mut handles = vec![];

    let start = Instant::now();

    // 启动多个线程并发生成代码
    for _ in 0..4 {
        let codes_clone = Arc::clone(&codes);
        let handle = thread::spawn(move || {
            let mut local_codes = HashSet::new();
            for _ in 0..1000 {
                local_codes.insert(generate_random_code(8));
            }

            let mut global_codes = codes_clone.lock().unwrap();
            global_codes.extend(local_codes);
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let duration = start.elapsed();
    let final_codes = codes.lock().unwrap();

    println!(
        "Generated {} unique codes concurrently in: {:?}",
        final_codes.len(),
        duration
    );
    assert!(
        final_codes.len() > 3500,
        "Not enough unique codes in concurrent test"
    );
    assert!(
        duration.as_millis() < 2000,
        "Concurrent generation too slow"
    );
}

#[test]
fn memory_usage_test() {
    // 测试大量代码生成的内存使用
    let mut codes = Vec::new();

    let start = Instant::now();
    for _ in 0..50000 {
        codes.push(generate_random_code(8));
    }
    let duration = start.elapsed();

    println!("Generated {} codes in: {:?}", codes.len(), duration);
    assert_eq!(codes.len(), 50000);

    // 验证所有代码都有正确长度
    for code in &codes[..100] {
        // 检查前100个
        assert_eq!(code.len(), 8);
    }

    // 性能断言
    assert!(duration.as_millis() < 3000, "Memory test too slow");
}
