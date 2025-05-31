use std::collections::HashSet;

// 导入实际的工具函数
use shortlinker::utils::generate_random_code;

#[test]
fn test_generate_random_code_length() {
    assert_eq!(generate_random_code(6).len(), 6);
    assert_eq!(generate_random_code(10).len(), 10);
    assert_eq!(generate_random_code(1).len(), 1);
    assert_eq!(generate_random_code(0).len(), 0);
}

#[test]
fn test_generate_random_code_characters() {
    let code = generate_random_code(100);
    let valid_chars: HashSet<char> =
        "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789"
            .chars()
            .collect();

    for ch in code.chars() {
        assert!(valid_chars.contains(&ch), "Invalid character: {}", ch);
    }
}

#[test]
fn test_generate_random_code_uniqueness() {
    let mut codes = HashSet::new();

    for _ in 0..1000 {
        codes.insert(generate_random_code(8));
    }

    // 应该生成大量不同的代码
    assert!(
        codes.len() > 990,
        "Generated codes lack sufficient randomness"
    );
}

#[test]
fn test_different_lengths() {
    for length in [1, 5, 8, 12, 20] {
        let code = generate_random_code(length);
        assert_eq!(code.len(), length, "Wrong length for {}", length);
    }
}

#[test]
fn test_empty_length() {
    let code = generate_random_code(0);
    assert!(code.is_empty());
}
