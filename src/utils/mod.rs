/// CSV 导入导出处理
pub mod csv_handler;
/// 密码哈希与验证工具
pub mod password;
/// 时间字符串解析（RFC3339 与相对时间）
pub mod time_parser;

pub use time_parser::TimeParser;

/// 短码最大长度
pub const MAX_SHORT_CODE_LEN: usize = 128;

/// 将 `u64` 计数转换为 `f64`。
///
/// `f64` 尾数只有 53 位，超过 2^53 的整数按 IEEE-754 就近舍入；拆分高低 32 位
/// 经 `u32` 无损转换后再组合，舍入结果与 `as` 强制转换一致，但显式表达转换意图。
///
/// # Panics
///
/// 仅当 32 位拆分逻辑实现错误导致越界时 panic；`u64` 高/低 32 位必然落在
/// `u32` 范围内，正常不会触发。
#[must_use]
pub fn u64_to_f64(value: u64) -> f64 {
    let high = f64::from(u32::try_from(value >> 32).expect("u64 高 32 位必然落在 u32 范围内"));
    let low =
        f64::from(u32::try_from(value & 0xFFFF_FFFF).expect("u64 低 32 位必然落在 u32 范围内"));
    // 2^32 是精确的 f64 值，乘法只调整指数不损失有效位
    high * 4_294_967_296.0 + low
}

/// 将 `i64` 计数转换为 `f64`。
///
/// 与 `as f64` 一致：按 IEEE-754 就近舍入，符号保留。
#[must_use]
pub fn i64_to_f64(value: i64) -> f64 {
    let magnitude = u64_to_f64(value.unsigned_abs());
    if value < 0 { -magnitude } else { magnitude }
}

/// 计算 `count / total * 100` 的百分比；`total` 为 0 时返回 0.0。
#[must_use]
pub fn percentage(count: u64, total: u64) -> f64 {
    if total == 0 {
        0.0
    } else {
        u64_to_f64(count) / u64_to_f64(total) * 100.0
    }
}

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

/// 获取保留的路由前缀
///
/// 必须从 RuntimeConfig 读取，因为配置可能在数据库中被修改。
/// RuntimeConfig 未初始化时使用默认值（仅启动早期）。
pub fn get_reserved_prefixes() -> Vec<String> {
    use crate::config::{keys, try_get_runtime_config};

    let Some(rt) = try_get_runtime_config() else {
        return vec!["admin".into(), "health".into(), "panel".into()];
    };

    vec![
        rt.get_or(keys::ROUTES_ADMIN_PREFIX, "/admin"),
        rt.get_or(keys::ROUTES_HEALTH_PREFIX, "/health"),
        rt.get_or(keys::ROUTES_FRONTEND_PREFIX, "/panel"),
    ]
    .into_iter()
    .map(|p| p.trim_start_matches('/').to_string())
    .collect()
}

/// 检查短码是否与保留路由冲突
///
/// 检查规则：
/// - 短码完全匹配保留前缀（如 "admin"）
/// - 短码以 "保留前缀/" 开头（如 "admin/xxx"）
pub fn is_reserved_short_code(code: &str) -> bool {
    let prefixes = get_reserved_prefixes();
    prefixes
        .iter()
        .any(|prefix| code == prefix || code.starts_with(&format!("{}/", prefix)))
}

/// 从固定字符集中随机生成指定长度的短码
pub fn generate_random_code(length: usize) -> String {
    use std::iter;

    // 随机选择字母和数字
    let chars = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";

    // 生成指定长度的随机字符串
    iter::repeat_with(|| chars[rand::random_range(0..chars.len())] as char)
        .take(length)
        .collect()
}

/// 生成密码学安全的随机 token（hex 编码）
///
/// 使用 OsRng 生成指定字节数的随机数据，然后转换为 hex 字符串。
/// 返回的字符串长度是 length_bytes * 2。
pub fn generate_secure_token(length_bytes: usize) -> String {
    let mut bytes = vec![0u8; length_bytes];
    rand::fill(&mut bytes);
    aster_forge_crypto::bytes_to_hex(&bytes)
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
