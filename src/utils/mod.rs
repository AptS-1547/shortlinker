pub mod csv_handler;
pub mod ip;
pub mod password;
pub mod time_parser;
pub mod url_validator;

pub use time_parser::TimeParser;

/// 短码最大长度
pub const MAX_SHORT_CODE_LEN: usize = 128;

/// 验证短码格式：非空、长度 ≤ 128，字符集 [a-zA-Z0-9_.-/]
///
/// 这个函数被多处使用：
/// - `redirect.rs`: 拒绝非法短码的 HTTP 请求
/// - `click_sink.rs`: SQL 注入防护（防御性检查）
#[inline]
pub fn is_valid_short_code(code: &str) -> bool {
    !code.is_empty()
        && code.len() <= MAX_SHORT_CODE_LEN
        && code.bytes().all(
            |b| matches!(b, b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_' | b'-' | b'.' | b'/'),
        )
}

pub fn generate_random_code(length: usize) -> String {
    use rand::Rng;
    use std::iter;

    let mut rng = rand::rng();

    // 随机选择字母和数字
    let chars = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";

    // 生成指定长度的随机字符串
    iter::repeat_with(|| chars[rng.random_range(0..chars.len())] as char)
        .take(length)
        .collect()
}

/// 生成密码学安全的随机 token（hex 编码）
///
/// 使用 OsRng 生成指定字节数的随机数据，然后转换为 hex 字符串。
/// 返回的字符串长度是 length_bytes * 2。
pub fn generate_secure_token(length_bytes: usize) -> String {
    use argon2::password_hash::rand_core::{OsRng, RngCore};
    let mut bytes = vec![0u8; length_bytes];
    OsRng.fill_bytes(&mut bytes);
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============== is_valid_short_code 测试 ==============

    #[test]
    fn test_valid_short_codes() {
        assert!(is_valid_short_code("abc123"));
        assert!(is_valid_short_code("ABC"));
        assert!(is_valid_short_code("a"));
        assert!(is_valid_short_code("test_code"));
        assert!(is_valid_short_code("test-code"));
        assert!(is_valid_short_code("test.code"));
        assert!(is_valid_short_code("path/to/code"));
        assert!(is_valid_short_code("MixedCase123"));
    }

    #[test]
    fn test_invalid_short_code_empty() {
        assert!(!is_valid_short_code(""));
    }

    #[test]
    fn test_invalid_short_code_too_long() {
        let long_code = "a".repeat(MAX_SHORT_CODE_LEN + 1);
        assert!(!is_valid_short_code(&long_code));

        // 刚好 128 字符应该有效
        let max_code = "a".repeat(MAX_SHORT_CODE_LEN);
        assert!(is_valid_short_code(&max_code));
    }

    #[test]
    fn test_invalid_short_code_special_chars() {
        // SQL 注入尝试
        assert!(!is_valid_short_code("'; DROP TABLE--"));
        assert!(!is_valid_short_code("code' OR '1'='1"));

        // 其他非法字符
        assert!(!is_valid_short_code("code with space"));
        assert!(!is_valid_short_code("code@email"));
        assert!(!is_valid_short_code("code#hash"));
        assert!(!is_valid_short_code("code$dollar"));
        assert!(!is_valid_short_code("code%percent"));
        assert!(!is_valid_short_code("code&amp"));
        assert!(!is_valid_short_code("code*star"));
        assert!(!is_valid_short_code("code+plus"));
        assert!(!is_valid_short_code("code=equal"));
        assert!(!is_valid_short_code("code?query"));
        assert!(!is_valid_short_code("code!bang"));
        assert!(!is_valid_short_code("code<>"));
        assert!(!is_valid_short_code("code\"quote"));
        assert!(!is_valid_short_code("code'quote"));
    }

    #[test]
    fn test_invalid_short_code_unicode() {
        assert!(!is_valid_short_code("代码"));
        assert!(!is_valid_short_code("code_with_symbol_✓"));
        assert!(!is_valid_short_code("código"));
        assert!(!is_valid_short_code("🔗"));
    }

    // ============== generate_random_code 测试 ==============

    #[test]
    fn test_generate_random_code_length() {
        for len in [1, 6, 10, 20] {
            let code = generate_random_code(len);
            assert_eq!(code.len(), len);
        }
    }

    #[test]
    fn test_generate_random_code_charset() {
        let code = generate_random_code(100);
        // 所有字符应该是字母或数字
        assert!(code.chars().all(|c| c.is_ascii_alphanumeric()));
    }

    #[test]
    fn test_generate_random_code_uniqueness() {
        let code1 = generate_random_code(20);
        let code2 = generate_random_code(20);
        // 两次生成的代码应该不同（概率极低相同）
        assert_ne!(code1, code2);
    }

    #[test]
    fn test_generate_random_code_is_valid() {
        // 生成的代码应该通过 is_valid_short_code 验证
        for _ in 0..10 {
            let code = generate_random_code(8);
            assert!(is_valid_short_code(&code));
        }
    }

    // ============== generate_secure_token 测试 ==============

    #[test]
    fn test_generate_secure_token_length() {
        // 返回的字符串长度是 length_bytes * 2（hex 编码）
        assert_eq!(generate_secure_token(16).len(), 32);
        assert_eq!(generate_secure_token(32).len(), 64);
        assert_eq!(generate_secure_token(1).len(), 2);
    }

    #[test]
    fn test_generate_secure_token_hex_format() {
        let token = generate_secure_token(16);
        // 应该只包含 hex 字符
        assert!(token.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_generate_secure_token_uniqueness() {
        let token1 = generate_secure_token(32);
        let token2 = generate_secure_token(32);
        assert_ne!(token1, token2);
    }
}
