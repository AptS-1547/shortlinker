pub mod password;
pub mod time_parser;
pub mod url_validator;

pub use time_parser::TimeParser;

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
