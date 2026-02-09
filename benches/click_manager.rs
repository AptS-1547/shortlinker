//! ClickManager 性能基准测试

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use shortlinker::analytics::ClickSink;
use std::sync::Arc;
use tokio::time::Duration;

/// 空 sink，只用于测试 increment 性能
struct NoopSink;

#[async_trait::async_trait]
impl ClickSink for NoopSink {
    async fn flush_clicks(&self, _updates: Vec<(String, usize)>) -> anyhow::Result<()> {
        Ok(())
    }
}

fn create_manager() -> shortlinker::analytics::manager::ClickManager {
    shortlinker::analytics::manager::ClickManager::new(
        Arc::new(NoopSink) as Arc<dyn ClickSink>,
        Duration::from_secs(3600), // 长间隔，避免自动刷盘
        usize::MAX,                // 高阈值，避免阈值刷盘
        shortlinker::metrics_core::NoopMetrics::arc(),
    )
}

/// 单线程 increment 吞吐量
fn bench_increment_single_thread(c: &mut Criterion) {
    let manager = create_manager();

    c.bench_function("increment/single_thread", |b| {
        b.iter(|| {
            manager.increment("test_key");
        });
    });
}

/// 单线程 increment 多个不同 key
fn bench_increment_different_keys(c: &mut Criterion) {
    let manager = create_manager();
    let keys: Vec<String> = (0..1000).map(|i| format!("key_{}", i)).collect();
    let mut idx = 0;

    c.bench_function("increment/different_keys", |b| {
        b.iter(|| {
            manager.increment(&keys[idx % keys.len()]);
            idx += 1;
        });
    });
}

/// 多线程并发 increment 吞吐量
fn bench_concurrent_increment(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let mut group = c.benchmark_group("increment/concurrent");

    for num_threads in [2, 4, 8, 16] {
        group.throughput(Throughput::Elements(1000));
        group.bench_with_input(
            BenchmarkId::new("threads", num_threads),
            &num_threads,
            |b, &num_threads| {
                b.to_async(&rt).iter(|| async {
                    let manager = Arc::new(create_manager());
                    let mut handles = vec![];

                    for _ in 0..num_threads {
                        let mgr = Arc::clone(&manager);
                        handles.push(tokio::spawn(async move {
                            for _ in 0..1000 / num_threads {
                                mgr.increment("shared_key");
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

/// 测试 drain 性能（预填充后 drain）
fn bench_drain(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let mut group = c.benchmark_group("drain");

    for num_entries in [100, 1000, 10000] {
        group.throughput(Throughput::Elements(num_entries as u64));
        group.bench_with_input(
            BenchmarkId::new("entries", num_entries),
            &num_entries,
            |b, &num_entries| {
                b.iter_batched(
                    || {
                        // Setup: 创建并填充 manager
                        let manager = create_manager();
                        for i in 0..num_entries {
                            manager.increment(&format!("key_{}", i));
                        }
                        manager
                    },
                    |manager| rt.block_on(manager.flush()),
                    criterion::BatchSize::SmallInput,
                );
            },
        );
    }
    group.finish();
}

/// 高并发场景：大量线程同时写入不同 key
fn bench_high_concurrency_different_keys(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let mut group = c.benchmark_group("increment/high_concurrency");

    for num_threads in [32, 64, 128] {
        let ops_per_thread = 100;
        group.throughput(Throughput::Elements((num_threads * ops_per_thread) as u64));
        group.bench_with_input(
            BenchmarkId::new("threads_different_keys", num_threads),
            &num_threads,
            |b, &num_threads| {
                b.to_async(&rt).iter(|| async {
                    let manager = Arc::new(create_manager());
                    let mut handles = vec![];

                    for thread_id in 0..num_threads {
                        let mgr = Arc::clone(&manager);
                        handles.push(tokio::spawn(async move {
                            for i in 0..ops_per_thread {
                                // 每个线程写入不同的 key
                                mgr.increment(&format!("key_{}_{}", thread_id, i));
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

/// 大批量 flush 场景
fn bench_large_batch_flush(c: &mut Criterion) {
    let mut group = c.benchmark_group("flush/large_batch");

    for num_entries in [10000, 50000, 100000] {
        group.throughput(Throughput::Elements(num_entries as u64));
        group.sample_size(10); // 减少采样次数，因为大批量操作较慢

        group.bench_with_input(
            BenchmarkId::new("entries", num_entries),
            &num_entries,
            |b, &num_entries| {
                b.iter_batched(
                    || {
                        let manager = create_manager();
                        // 预填充大量不同的 key
                        for i in 0..num_entries {
                            manager.increment(&format!("key_{}", i));
                        }
                        manager
                    },
                    |manager| {
                        let rt = tokio::runtime::Runtime::new().unwrap();
                        rt.block_on(manager.flush());
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );
    }
    group.finish();
}

/// 热点 key 场景：少量 key 被大量访问
fn bench_hotspot_keys(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let mut group = c.benchmark_group("increment/hotspot");

    // 10 个热点 key，每个被访问 1000 次
    let num_hotspot_keys = 10;
    let accesses_per_key = 1000;
    let total_ops = num_hotspot_keys * accesses_per_key;

    group.throughput(Throughput::Elements(total_ops as u64));
    group.bench_function("10_keys_1000_each", |b| {
        b.to_async(&rt).iter(|| async {
            let manager = Arc::new(create_manager());
            let keys: Vec<String> = (0..num_hotspot_keys)
                .map(|i| format!("hot_{}", i))
                .collect();
            let mut handles = vec![];

            // 使用多线程并发访问热点 key
            for _ in 0..10 {
                let mgr = Arc::clone(&manager);
                let keys = keys.clone();
                handles.push(tokio::spawn(async move {
                    for _ in 0..accesses_per_key / 10 {
                        for key in &keys {
                            mgr.increment(key);
                        }
                    }
                }));
            }

            for handle in handles {
                handle.await.unwrap();
            }
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_increment_single_thread,
    bench_increment_different_keys,
    bench_concurrent_increment,
    bench_drain,
    bench_high_concurrency_different_keys,
    bench_large_batch_flush,
    bench_hotspot_keys,
);
criterion_main!(benches);
