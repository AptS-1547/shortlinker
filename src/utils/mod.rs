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
