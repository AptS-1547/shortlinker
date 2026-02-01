//! 模型转换器性能基准测试

use chrono::{Duration, Utc};
use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use migration::entities::short_link;
use shortlinker::storage::ShortLink;
use shortlinker::storage::backend::{model_to_shortlink, shortlink_to_active_model};

fn create_test_model() -> short_link::Model {
    short_link::Model {
        short_code: "abc123".to_string(),
        target_url: "https://example.com/very/long/path/to/resource".to_string(),
        created_at: Utc::now(),
        expires_at: Some(Utc::now() + Duration::days(7)),
        password: Some("$argon2id$v=19$m=19456,t=2,p=1$hash".to_string()),
        click_count: 12345,
    }
}

fn create_test_shortlink() -> ShortLink {
    ShortLink {
        code: "xyz789".to_string(),
        target: "https://target.com/another/long/path".to_string(),
        created_at: Utc::now(),
        expires_at: Some(Utc::now() + Duration::hours(24)),
        password: Some("$argon2id$v=19$m=19456,t=2,p=1$hash".to_string()),
        click: 9999,
    }
}

/// Model -> ShortLink 转换性能
fn bench_model_to_shortlink(c: &mut Criterion) {
    let mut group = c.benchmark_group("converters/model_to_shortlink");
    group.throughput(Throughput::Elements(1));

    // 完整字段
    group.bench_function("full_fields", |b| {
        let model = create_test_model();
        b.iter(|| {
            let _ = model_to_shortlink(model.clone());
        });
    });

    // 最小字段（无可选字段）
    group.bench_function("minimal_fields", |b| {
        let model = short_link::Model {
            short_code: "abc".to_string(),
            target_url: "https://x.co".to_string(),
            created_at: Utc::now(),
            expires_at: None,
            password: None,
            click_count: 0,
        };
        b.iter(|| {
            let _ = model_to_shortlink(model.clone());
        });
    });

    group.finish();
}

/// ShortLink -> ActiveModel 转换性能
fn bench_shortlink_to_active_model(c: &mut Criterion) {
    let mut group = c.benchmark_group("converters/shortlink_to_active_model");
    group.throughput(Throughput::Elements(1));

    // 新建模式
    group.bench_function("new_record", |b| {
        let link = create_test_shortlink();
        b.iter(|| {
            let _ = shortlink_to_active_model(&link, true);
        });
    });

    // 更新模式
    group.bench_function("update_record", |b| {
        let link = create_test_shortlink();
        b.iter(|| {
            let _ = shortlink_to_active_model(&link, false);
        });
    });

    group.finish();
}

/// 批量转换性能
fn bench_batch_conversion(c: &mut Criterion) {
    let mut group = c.benchmark_group("converters/batch");

    for batch_size in [10, 100, 1000] {
        group.throughput(Throughput::Elements(batch_size as u64));

        group.bench_function(format!("model_to_shortlink_{}", batch_size), |b| {
            let models: Vec<_> = (0..batch_size)
                .map(|i| short_link::Model {
                    short_code: format!("code_{}", i),
                    target_url: format!("https://example.com/{}", i),
                    created_at: Utc::now(),
                    expires_at: Some(Utc::now() + Duration::days(7)),
                    password: None,
                    click_count: i as i64,
                })
                .collect();

            b.iter(|| {
                let _: Vec<_> = models.iter().cloned().map(model_to_shortlink).collect();
            });
        });

        group.bench_function(format!("shortlink_to_active_model_{}", batch_size), |b| {
            let links: Vec<_> = (0..batch_size)
                .map(|i| ShortLink {
                    code: format!("code_{}", i),
                    target: format!("https://example.com/{}", i),
                    created_at: Utc::now(),
                    expires_at: Some(Utc::now() + Duration::days(7)),
                    password: None,
                    click: i,
                })
                .collect();

            b.iter(|| {
                let _: Vec<_> = links
                    .iter()
                    .map(|l| shortlink_to_active_model(l, true))
                    .collect();
            });
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_model_to_shortlink,
    bench_shortlink_to_active_model,
    bench_batch_conversion,
);
criterion_main!(benches);
