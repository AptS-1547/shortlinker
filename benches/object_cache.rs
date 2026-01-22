//! Moka 对象缓存性能基准测试

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use shortlinker::cache::object_cache::moka::MokaCacheWrapper;
use shortlinker::cache::{CacheResult, ObjectCache};
use shortlinker::storage::ShortLink;
use std::sync::Arc;

fn create_test_link(code: &str) -> ShortLink {
    ShortLink {
        code: code.to_string(),
        target: "https://example.com/very/long/path/to/destination".to_string(),
        created_at: chrono::Utc::now(),
        expires_at: None,
        password: None,
        click: 0,
    }
}

// ============== Moka Object Cache 基准测试 ==============

fn bench_moka_get_hit(c: &mut Criterion) {
    // 需要初始化配置
    shortlinker::config::init_config();

    let rt = tokio::runtime::Runtime::new().unwrap();
    let cache = rt.block_on(async { Arc::new(MokaCacheWrapper::new().await.unwrap()) });

    // 预填充数据
    rt.block_on(async {
        for i in 0..1000 {
            let link = create_test_link(&format!("key_{}", i));
            cache.insert(&format!("key_{}", i), link, None).await;
        }
    });

    let cache_clone = Arc::clone(&cache);
    c.bench_function("moka/get_hit", |b| {
        b.to_async(&rt).iter(|| {
            let c = Arc::clone(&cache_clone);
            async move {
                let result = c.get("key_500").await;
                assert!(matches!(result, CacheResult::Found(_)));
            }
        });
    });
}

fn bench_moka_get_miss(c: &mut Criterion) {
    shortlinker::config::init_config();

    let rt = tokio::runtime::Runtime::new().unwrap();
    let cache = rt.block_on(async { Arc::new(MokaCacheWrapper::new().await.unwrap()) });

    let cache_clone = Arc::clone(&cache);
    c.bench_function("moka/get_miss", |b| {
        b.to_async(&rt).iter(|| {
            let c = Arc::clone(&cache_clone);
            async move {
                let result = c.get("nonexistent_key").await;
                assert!(matches!(result, CacheResult::Miss));
            }
        });
    });
}

fn bench_moka_insert(c: &mut Criterion) {
    shortlinker::config::init_config();

    let rt = tokio::runtime::Runtime::new().unwrap();
    let cache = rt.block_on(async { Arc::new(MokaCacheWrapper::new().await.unwrap()) });
    let counter = std::sync::atomic::AtomicU64::new(0);

    c.bench_function("moka/insert", |b| {
        b.to_async(&rt).iter(|| {
            let c = Arc::clone(&cache);
            let i = counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            let link = create_test_link(&format!("insert_key_{}", i));
            async move {
                c.insert(&format!("insert_key_{}", i), link, None).await;
            }
        });
    });
}

fn bench_moka_remove(c: &mut Criterion) {
    shortlinker::config::init_config();

    let rt = tokio::runtime::Runtime::new().unwrap();
    let cache = rt.block_on(async { Arc::new(MokaCacheWrapper::new().await.unwrap()) });

    // 预填充数据
    rt.block_on(async {
        for i in 0..10000 {
            let link = create_test_link(&format!("remove_key_{}", i));
            cache.insert(&format!("remove_key_{}", i), link, None).await;
        }
    });

    let counter = std::sync::atomic::AtomicU64::new(0);
    c.bench_function("moka/remove", |b| {
        b.to_async(&rt).iter(|| {
            let c = Arc::clone(&cache);
            let i = counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            async move {
                c.remove(&format!("remove_key_{}", i)).await;
            }
        });
    });
}

fn bench_moka_concurrent_get(c: &mut Criterion) {
    shortlinker::config::init_config();

    let rt = tokio::runtime::Runtime::new().unwrap();
    let mut group = c.benchmark_group("moka/concurrent_get");

    for num_tasks in [2, 4, 8] {
        group.throughput(Throughput::Elements(1000));
        group.bench_with_input(
            BenchmarkId::new("tasks", num_tasks),
            &num_tasks,
            |b, &num_tasks| {
                b.to_async(&rt).iter(|| async move {
                    let cache = Arc::new(MokaCacheWrapper::new().await.unwrap());

                    // 预填充
                    for i in 0..1000 {
                        let link = create_test_link(&format!("concurrent_key_{}", i));
                        cache
                            .insert(&format!("concurrent_key_{}", i), link, None)
                            .await;
                    }

                    let mut handles = vec![];
                    for t in 0..num_tasks {
                        let c = Arc::clone(&cache);
                        handles.push(tokio::spawn(async move {
                            for i in 0..(1000 / num_tasks) {
                                let key = format!("concurrent_key_{}", t * (1000 / num_tasks) + i);
                                let _ = c.get(&key).await;
                            }
                        }));
                    }

                    for handle in handles {
                        handle.await.unwrap();
                    }
                });
            },
        );
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_moka_get_hit,
    bench_moka_get_miss,
    bench_moka_insert,
    bench_moka_remove,
    bench_moka_concurrent_get,
);
criterion_main!(benches);
