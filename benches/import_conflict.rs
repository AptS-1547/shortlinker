//! CSV 导入冲突检测策略基准测试
//!
//! 对比三种冲突检测策略的性能：
//! 1. 全量加载 (load_all_codes) - 原方案
//! 2. 批量查询 (batch_check) - 中间方案
//! 3. Bloom 预筛选 (bloom_prefilter) - 当前方案

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use csv::{ReaderBuilder, WriterBuilder};
use shortlinker::cache::existence_filter::bloom::BloomExistenceFilterPlugin;
use shortlinker::cache::ExistenceFilter;
use std::collections::HashSet;
use std::io::Cursor;
use std::sync::Arc;

/// 生成测试用 CSV 数据
fn generate_csv_data(num_rows: usize) -> Vec<u8> {
    let mut wtr = WriterBuilder::new().from_writer(Vec::new());
    wtr.write_record(["code", "target", "created_at", "expires_at", "password", "click_count"])
        .unwrap();
    for i in 0..num_rows {
        wtr.write_record([
            &format!("code_{}", i),
            &format!("https://example.com/{}", i),
            "2024-01-01T00:00:00Z",
            "",
            "",
            "0",
        ])
        .unwrap();
    }
    wtr.into_inner().unwrap()
}

/// 从 CSV 提取所有 code（预扫描阶段）
fn extract_codes_from_csv(csv_data: &[u8]) -> Vec<String> {
    let cursor = Cursor::new(csv_data);
    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .trim(csv::Trim::All)
        .from_reader(cursor);

    reader
        .records()
        .filter_map(|r| r.ok())
        .filter_map(|record| record.get(0).map(|s| s.to_string()))
        .filter(|code| !code.is_empty())
        .collect()
}

// ============== CSV 预扫描性能 ==============

fn bench_csv_prescan(c: &mut Criterion) {
    let mut group = c.benchmark_group("import/csv_prescan");

    for csv_size in [100, 1000, 10000] {
        let csv_data = generate_csv_data(csv_size);
        group.throughput(Throughput::Elements(csv_size as u64));
        group.bench_with_input(
            BenchmarkId::new("rows", csv_size),
            &csv_data,
            |b, data| {
                b.iter(|| extract_codes_from_csv(data));
            },
        );
    }
    group.finish();
}

// ============== Bloom 预筛选 vs 全量 HashSet 查找 ==============

fn bench_bloom_prefilter(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let mut group = c.benchmark_group("import/bloom_prefilter");

    // 场景：DB 有 db_size 条记录，CSV 有 csv_size 条（全新 code）
    for (db_size, csv_size) in [(0, 1000), (10000, 100), (100000, 1000)] {
        let filter = Arc::new(BloomExistenceFilterPlugin::new().unwrap());

        // 模拟 DB 中已有的 codes 加入 Bloom Filter
        rt.block_on(async {
            let keys: Vec<String> = (0..db_size).map(|i| format!("existing_{}", i)).collect();
            filter.bulk_set(&keys).await;
        });

        // CSV 中的 codes（全新，不在 DB 中）
        let csv_codes: Vec<String> = (0..csv_size).map(|i| format!("new_code_{}", i)).collect();

        let label = format!("db{}_csv{}", db_size, csv_size);
        group.throughput(Throughput::Elements(csv_size as u64));

        // 方案 A：Bloom 预筛选
        let f = Arc::clone(&filter);
        let codes = csv_codes.clone();
        group.bench_with_input(
            BenchmarkId::new("bloom_filter", &label),
            &(),
            |b, _| {
                b.to_async(&rt).iter(|| {
                    let f = Arc::clone(&f);
                    let codes = codes.clone();
                    async move {
                        let mut maybe_exist = Vec::new();
                        for code in &codes {
                            if f.check(code).await {
                                maybe_exist.push(code.clone());
                            }
                        }
                        maybe_exist
                    }
                });
            },
        );

        // 方案 B：直接 HashSet lookup（模拟 batch_check 的结果）
        let existing_set: HashSet<String> =
            (0..db_size).map(|i| format!("existing_{}", i)).collect();
        let codes = csv_codes.clone();
        group.bench_with_input(
            BenchmarkId::new("hashset_lookup", &label),
            &(),
            |b, _| {
                b.iter(|| {
                    let mut conflicts = Vec::new();
                    for code in &codes {
                        if existing_set.contains(code) {
                            conflicts.push(code.clone());
                        }
                    }
                    conflicts
                });
            },
        );
    }
    group.finish();
}

// ============== 冲突检测：有冲突场景 ==============

fn bench_bloom_with_conflicts(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let mut group = c.benchmark_group("import/bloom_conflicts");

    // 场景：CSV 中有 conflict_pct% 的 code 已存在于 DB
    for (csv_size, conflict_pct) in [(1000, 0), (1000, 10), (1000, 50), (1000, 100)] {
        let conflict_count = csv_size * conflict_pct / 100;
        let new_count = csv_size - conflict_count;

        let filter = Arc::new(BloomExistenceFilterPlugin::new().unwrap());

        // DB 中的 codes
        let db_codes: Vec<String> = (0..conflict_count)
            .map(|i| format!("shared_code_{}", i))
            .collect();

        rt.block_on(async {
            filter.bulk_set(&db_codes).await;
        });

        // CSV codes = 冲突部分 + 新增部分
        let mut csv_codes: Vec<String> = (0..conflict_count)
            .map(|i| format!("shared_code_{}", i))
            .collect();
        csv_codes.extend((0..new_count).map(|i| format!("new_code_{}", i)));

        let label = format!("csv{}_conflict{}pct", csv_size, conflict_pct);
        group.throughput(Throughput::Elements(csv_size as u64));

        let f = Arc::clone(&filter);
        let codes = csv_codes.clone();
        group.bench_with_input(
            BenchmarkId::new("bloom_prefilter", &label),
            &(),
            |b, _| {
                b.to_async(&rt).iter(|| {
                    let f = Arc::clone(&f);
                    let codes = codes.clone();
                    async move {
                        let mut maybe_exist = Vec::new();
                        for code in &codes {
                            if f.check(code).await {
                                maybe_exist.push(code.clone());
                            }
                        }
                        maybe_exist.len()
                    }
                });
            },
        );
    }
    group.finish();
}

// ============== 全流程对比（模拟） ==============

fn bench_conflict_strategy(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let mut group = c.benchmark_group("import/strategy");

    let db_size = 50000;
    let csv_size = 500;

    // 准备 Bloom Filter（模拟启动时加载）
    let filter = Arc::new(BloomExistenceFilterPlugin::new().unwrap());
    let db_codes: Vec<String> = (0..db_size).map(|i| format!("db_code_{}", i)).collect();
    rt.block_on(async {
        filter.bulk_set(&db_codes).await;
    });

    // CSV 数据（全新 codes）
    let csv_data = generate_csv_data(csv_size);
    let csv_codes: Vec<String> = (0..csv_size).map(|i| format!("code_{}", i)).collect();

    // 策略 1：Overwrite 模式（零检测）
    group.throughput(Throughput::Elements(csv_size as u64));
    let data = csv_data.clone();
    group.bench_function("overwrite_no_check", |b| {
        b.iter(|| {
            // 只解析 CSV，不做冲突检测
            extract_codes_from_csv(&data);
        });
    });

    // 策略 2：Bloom 预筛选 + 精确查询
    let f = Arc::clone(&filter);
    let codes = csv_codes.clone();
    group.bench_function("skip_bloom_prefilter", |b| {
        b.to_async(&rt).iter(|| {
            let f = Arc::clone(&f);
            let codes = codes.clone();
            async move {
                // Step 1: 预扫描（已完成，直接用 codes）
                // Step 2: Bloom 预筛选
                let mut maybe_exist = Vec::new();
                for code in &codes {
                    if f.check(code).await {
                        maybe_exist.push(code.clone());
                    }
                }
                // Step 3: 模拟 batch_check（这里用 HashSet 模拟）
                let db_set: HashSet<&str> = HashSet::new(); // 空集（全新 codes）
                let existing: HashSet<String> = maybe_exist
                    .into_iter()
                    .filter(|c| db_set.contains(c.as_str()))
                    .collect();
                existing
            }
        });
    });

    // 策略 3：直接全量 HashSet（模拟 load_all_codes）
    let all_codes_set: HashSet<String> = db_codes.into_iter().collect();
    let codes = csv_codes.clone();
    group.bench_function("load_all_hashset", |b| {
        b.iter(|| {
            let mut conflicts = Vec::new();
            for code in &codes {
                if all_codes_set.contains(code) {
                    conflicts.push(code.clone());
                }
            }
            conflicts
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_csv_prescan,
    bench_bloom_prefilter,
    bench_bloom_with_conflicts,
    bench_conflict_strategy,
);
criterion_main!(benches);
