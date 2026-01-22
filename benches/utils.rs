//! 工具函数性能基准测试

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use shortlinker::utils::url_validator::validate_url;
use shortlinker::utils::{generate_random_code, generate_secure_token, is_valid_short_code};

// ============== is_valid_short_code 基准测试 ==============

fn bench_is_valid_short_code(c: &mut Criterion) {
    let mut group = c.benchmark_group("utils/is_valid_short_code");

    // 有效短码
    group.bench_function("valid_simple", |b| {
        b.iter(|| {
            assert!(is_valid_short_code("abc123"));
        });
    });

    group.bench_function("valid_with_special", |b| {
        b.iter(|| {
            assert!(is_valid_short_code("path/to/code-with_dots.ext"));
        });
    });

    // 无效短码
    group.bench_function("invalid_empty", |b| {
        b.iter(|| {
            assert!(!is_valid_short_code(""));
        });
    });

    group.bench_function("invalid_special_chars", |b| {
        b.iter(|| {
            assert!(!is_valid_short_code("'; DROP TABLE--"));
        });
    });

    // 长度边界测试
    let max_len_code = "a".repeat(128);
    group.bench_function("valid_max_length", |b| {
        b.iter(|| {
            assert!(is_valid_short_code(&max_len_code));
        });
    });

    let too_long_code = "a".repeat(129);
    group.bench_function("invalid_too_long", |b| {
        b.iter(|| {
            assert!(!is_valid_short_code(&too_long_code));
        });
    });

    group.finish();
}

// ============== generate_random_code 基准测试 ==============

fn bench_generate_random_code(c: &mut Criterion) {
    let mut group = c.benchmark_group("utils/generate_random_code");

    for length in [6, 8, 12, 20] {
        group.bench_with_input(BenchmarkId::new("length", length), &length, |b, &length| {
            b.iter(|| {
                let code = generate_random_code(length);
                assert_eq!(code.len(), length);
            });
        });
    }

    group.finish();
}

// ============== generate_secure_token 基准测试 ==============

fn bench_generate_secure_token(c: &mut Criterion) {
    let mut group = c.benchmark_group("utils/generate_secure_token");

    for bytes in [16, 32, 64] {
        group.bench_with_input(BenchmarkId::new("bytes", bytes), &bytes, |b, &bytes| {
            b.iter(|| {
                let token = generate_secure_token(bytes);
                assert_eq!(token.len(), bytes * 2);
            });
        });
    }

    group.finish();
}

// ============== validate_url 基准测试 ==============

fn bench_validate_url(c: &mut Criterion) {
    let mut group = c.benchmark_group("utils/validate_url");

    // 有效 URL
    group.bench_function("valid_https", |b| {
        b.iter(|| {
            assert!(validate_url("https://example.com/path?query=1").is_ok());
        });
    });

    group.bench_function("valid_http", |b| {
        b.iter(|| {
            assert!(validate_url("http://localhost:8080/api/v1").is_ok());
        });
    });

    // 无效 URL
    group.bench_function("invalid_dangerous_protocol", |b| {
        b.iter(|| {
            assert!(validate_url("javascript:alert(1)").is_err());
        });
    });

    group.bench_function("invalid_protocol", |b| {
        b.iter(|| {
            assert!(validate_url("ftp://example.com").is_err());
        });
    });

    group.bench_function("invalid_empty", |b| {
        b.iter(|| {
            assert!(validate_url("").is_err());
        });
    });

    // 长 URL
    let long_url = format!("https://example.com/{}", "a".repeat(1000));
    group.bench_function("valid_long_url", |b| {
        b.iter(|| {
            assert!(validate_url(&long_url).is_ok());
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_is_valid_short_code,
    bench_generate_random_code,
    bench_generate_secure_token,
    bench_validate_url,
);
criterion_main!(benches);
