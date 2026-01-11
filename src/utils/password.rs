//! 密码哈希工具模块
//!
//! 使用 Argon2id 算法进行密码哈希和验证

use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};

/// 密码哈希错误
#[derive(Debug)]
pub enum PasswordError {
    HashError(String),
    VerifyError(String),
}

impl std::fmt::Display for PasswordError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::HashError(msg) => write!(f, "Password hash error: {}", msg),
            Self::VerifyError(msg) => write!(f, "Password verify error: {}", msg),
        }
    }
}

impl std::error::Error for PasswordError {}

/// 对密码进行 Argon2id 哈希
pub fn hash_password(password: &str) -> Result<String, PasswordError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    argon2
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|e| PasswordError::HashError(e.to_string()))
}

/// 验证密码是否匹配哈希
pub fn verify_password(password: &str, hash: &str) -> Result<bool, PasswordError> {
    let parsed_hash =
        PasswordHash::new(hash).map_err(|e| PasswordError::VerifyError(e.to_string()))?;

    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

/// 检测字符串是否是 Argon2 哈希格式
pub fn is_argon2_hash(s: &str) -> bool {
    s.starts_with("$argon2")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_and_verify() {
        let password = "test_password_123";
        let hash = hash_password(password).expect("hash should succeed");

        assert!(is_argon2_hash(&hash));
        assert!(verify_password(password, &hash).expect("verify should succeed"));
        assert!(!verify_password("wrong_password", &hash).expect("verify should succeed"));
    }

    #[test]
    fn test_is_argon2_hash() {
        assert!(is_argon2_hash("$argon2id$v=19$m=19456,t=2,p=1$xxx"));
        assert!(is_argon2_hash("$argon2i$v=19$m=19456,t=2,p=1$xxx"));
        assert!(!is_argon2_hash("plaintext_password"));
        assert!(!is_argon2_hash("$bcrypt$xxx"));
    }
}
