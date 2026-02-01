//! Argon2 密码哈希性能基准测试

use criterion::{Criterion, criterion_group, criterion_main};
use shortlinker::utils::password::{hash_password, is_argon2_hash, verify_password};

fn bench_hash_password(c: &mut Criterion) {
    c.bench_function("password/hash", |b| {
        b.iter(|| {
            let _ = hash_password("test_password_123");
        });
    });
}

fn bench_verify_password_correct(c: &mut Criterion) {
    let password = "correct_password_456";
    let hash = hash_password(password).expect("hash should succeed");

    c.bench_function("password/verify_correct", |b| {
        b.iter(|| {
            let result = verify_password(password, &hash).expect("verify should succeed");
            assert!(result);
        });
    });
}

fn bench_verify_password_wrong(c: &mut Criterion) {
    let password = "correct_password_789";
    let hash = hash_password(password).expect("hash should succeed");

    c.bench_function("password/verify_wrong", |b| {
        b.iter(|| {
            let result = verify_password("wrong_password", &hash).expect("verify should succeed");
            assert!(!result);
        });
    });
}

fn bench_is_argon2_hash(c: &mut Criterion) {
    let valid_hash = "$argon2id$v=19$m=19456,t=2,p=1$somesaltvalue$somehashvalue";
    let invalid_hash = "plaintext_password";

    let mut group = c.benchmark_group("password/is_argon2_hash");

    group.bench_function("valid", |b| {
        b.iter(|| {
            assert!(is_argon2_hash(valid_hash));
        });
    });

    group.bench_function("invalid", |b| {
        b.iter(|| {
            assert!(!is_argon2_hash(invalid_hash));
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_hash_password,
    bench_verify_password_correct,
    bench_verify_password_wrong,
    bench_is_argon2_hash,
);
criterion_main!(benches);
