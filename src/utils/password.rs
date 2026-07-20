//! 密码哈希工具模块
//!
//! 使用 Argon2id 算法进行密码哈希和验证

/// 密码哈希错误
#[derive(Debug)]
pub enum PasswordError {
    HashError(String),
}

impl std::fmt::Display for PasswordError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::HashError(msg) => write!(f, "Password hash error: {}", msg),
        }
    }
}

impl std::error::Error for PasswordError {}

/// 检测字符串是否是 Argon2 哈希格式
pub fn is_argon2_hash(s: &str) -> bool {
    s.starts_with("$argon2")
}

/// 处理用户输入的新密码 - 始终哈希，不接受预哈希值
///
/// - 如果输入为空或 None，返回 None
/// - 否则对密码进行哈希
pub fn process_new_password(password: Option<&str>) -> Result<Option<String>, PasswordError> {
    match password {
        Some(pwd) if !pwd.is_empty() => aster_forge_crypto::hash_password(pwd)
            .map(Some)
            .map_err(|e| PasswordError::HashError(e.to_string())),
        _ => Ok(None),
    }
}

/// 处理用户输入的更新密码 - 始终哈希，不接受预哈希值
///
/// - 如果 `new_password` 为 None，保留 `existing_password`
/// - 如果 `new_password` 为空字符串，返回 None（移除密码）
/// - 否则对密码进行哈希
pub fn process_update_password(
    new_password: Option<&str>,
    existing_password: Option<String>,
) -> Result<Option<String>, PasswordError> {
    match new_password {
        Some(pwd) if !pwd.is_empty() => aster_forge_crypto::hash_password(pwd)
            .map(Some)
            .map_err(|e| PasswordError::HashError(e.to_string())),
        Some(_) => Ok(None),           // 空字符串 = 移除密码
        None => Ok(existing_password), // 未提供 = 保留原密码
    }
}

/// 处理导入数据的密码 - 已哈希则保留，明文则哈希
///
/// 仅用于系统内部导入（CSV），不可用于用户输入路径
pub fn process_imported_password(password: Option<&str>) -> Result<Option<String>, PasswordError> {
    match password {
        Some(pwd) if !pwd.is_empty() => {
            if is_argon2_hash(pwd) {
                Ok(Some(pwd.to_string()))
            } else {
                aster_forge_crypto::hash_password(pwd)
                    .map(Some)
                    .map_err(|e| PasswordError::HashError(e.to_string()))
            }
        }
        _ => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_and_verify() {
        let password = "test_password_123";
        let hash = aster_forge_crypto::hash_password(password).expect("hash should succeed");

        assert!(is_argon2_hash(&hash));
        assert!(
            aster_forge_crypto::verify_password(password, &hash).expect("verify should succeed")
        );
        assert!(
            !aster_forge_crypto::verify_password("wrong_password", &hash)
                .expect("verify should succeed")
        );
    }

    #[test]
    fn test_is_argon2_hash() {
        assert!(is_argon2_hash("$argon2id$v=19$m=19456,t=2,p=1$xxx"));
        assert!(is_argon2_hash("$argon2i$v=19$m=19456,t=2,p=1$xxx"));
        assert!(!is_argon2_hash("plaintext_password"));
        assert!(!is_argon2_hash("$bcrypt$xxx"));
    }
}
