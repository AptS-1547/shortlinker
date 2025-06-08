use shortlinker::utils::generate_random_code;
use std::collections::HashSet;

#[test]
fn generate_random_code_length() {
    let code = generate_random_code(8);
    assert_eq!(code.len(), 8);
    assert!(code.chars().all(|c| c.is_ascii_alphanumeric()));
}

#[test]
fn generate_random_code_zero() {
    let code = generate_random_code(0);
    assert!(code.is_empty());
}

#[test]
fn generate_random_code_uniqueness() {
    let mut codes = HashSet::new();
    for _ in 0..100 {
        codes.insert(generate_random_code(6));
    }
    assert!(codes.len() > 90);
}
