//! 缓存层性能基准测试

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use shortlinker::cache::existence_filter::bloom::BloomExistenceFilterPlugin;
use shortlinker::cache::negative_cache::MokaNegativeCache;
use shortlinker::cache::{ExistenceFilter, NegativeCache};
use std::sync::Arc;

// ============== Bloom Filter 基准测试 ==============

fn bench_bloom_check(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let filter = Arc::new(BloomExistenceFilterPlugin::new().unwrap());

    // 预填充一些数据
    rt.block_on(async {
        for i in 0..1000 {
            filter.set(&format!("key_{}", i)).await;
        }
    });

    let filter_hit = Arc::clone(&filter);
    c.bench_function("bloom/check_hit", |b| {
        b.to_async(&rt).iter(|| {
            let f = Arc::clone(&filter_hit);
            async move { f.check("key_500").await }
        });
    });

    let filter_miss = Arc::clone(&filter);
    c.bench_function("bloom/check_miss", |b| {
        b.to_async(&rt).iter(|| {
            let f = Arc::clone(&filter_miss);
            async move { f.check("nonexistent_key").await }
        });
    });
}

fn bench_bloom_set(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let filter = Arc::new(BloomExistenceFilterPlugin::new().unwrap());
    let counter = std::sync::atomic::AtomicU64::new(0);

    c.bench_function("bloom/set", |b| {
        b.to_async(&rt).iter(|| {
            let f = Arc::clone(&filter);
            let i = counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            async move {
                f.set(&format!("key_{}", i)).await;
            }
        });
    });
}

fn bench_bloom_bulk_set(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let mut group = c.benchmark_group("bloom/bulk_set");

    for size in [100, 500, 1000] {
        let keys: Vec<String> = (0..size).map(|i| format!("bulk_key_{}", i)).collect();

        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(BenchmarkId::new("keys", size), &keys, |b, keys| {
            b.iter_batched(
                || BloomExistenceFilterPlugin::new().unwrap(),
                |filter| {
                    rt.block_on(async {
                        filter.bulk_set(keys).await;
                    });
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }
    group.finish();
}

// ============== Moka 负缓存基准测试 ==============

fn bench_negative_cache_contains(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let cache = Arc::new(MokaNegativeCache::new(10000, 300));

    // 预填充数据
    rt.block_on(async {
        for i in 0..1000 {
            cache.mark(&format!("key_{}", i)).await;
        }
    });

    let cache_hit = Arc::clone(&cache);
    c.bench_function("negative_cache/contains_hit", |b| {
        b.to_async(&rt).iter(|| {
            let c = Arc::clone(&cache_hit);
            async move { c.contains("key_500").await }
        });
    });

    let cache_miss = Arc::clone(&cache);
    c.bench_function("negative_cache/contains_miss", |b| {
        b.to_async(&rt).iter(|| {
            let c = Arc::clone(&cache_miss);
            async move { c.contains("nonexistent_key").await }
        });
    });
}

fn bench_negative_cache_mark(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let cache = Arc::new(MokaNegativeCache::new(100000, 300));
    let counter = std::sync::atomic::AtomicU64::new(0);

    c.bench_function("negative_cache/mark", |b| {
        b.to_async(&rt).iter(|| {
            let c = Arc::clone(&cache);
            let i = counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            async move {
                c.mark(&format!("key_{}", i)).await;
            }
        });
    });
}

// ============== 并发性能测试 ==============

fn bench_bloom_concurrent(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let mut group = c.benchmark_group("bloom/concurrent");

    for num_tasks in [2, 4, 8] {
        group.throughput(Throughput::Elements(1000));
        group.bench_with_input(
            BenchmarkId::new("tasks", num_tasks),
            &num_tasks,
            |b, &num_tasks| {
                b.to_async(&rt).iter(|| async move {
                    let filter = Arc::new(BloomExistenceFilterPlugin::new().unwrap());
                    let mut handles = vec![];

                    for t in 0..num_tasks {
                        let f = Arc::clone(&filter);
                        handles.push(tokio::spawn(async move {
                            for i in 0..(1000 / num_tasks) {
                                f.set(&format!("key_{}_{}", t, i)).await;
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

fn bench_negative_cache_concurrent(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let mut group = c.benchmark_group("negative_cache/concurrent");

    for num_tasks in [2, 4, 8] {
        group.throughput(Throughput::Elements(1000));
        group.bench_with_input(
            BenchmarkId::new("tasks", num_tasks),
            &num_tasks,
            |b, &num_tasks| {
                b.to_async(&rt).iter(|| async move {
                    let cache = Arc::new(MokaNegativeCache::new(100000, 300));
                    let mut handles = vec![];

                    for t in 0..num_tasks {
                        let c = Arc::clone(&cache);
                        handles.push(tokio::spawn(async move {
                            for i in 0..(1000 / num_tasks) {
                                c.mark(&format!("key_{}_{}", t, i)).await;
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
    bench_bloom_check,
    bench_bloom_set,
    bench_bloom_bulk_set,
    bench_negative_cache_contains,
    bench_negative_cache_mark,
    bench_bloom_concurrent,
    bench_negative_cache_concurrent,
);
criterion_main!(benches);
