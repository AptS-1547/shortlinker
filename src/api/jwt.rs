use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

/// Global cached JwtService instance
static JWT_SERVICE: OnceLock<JwtService> = OnceLock::new();

/// Get the cached JwtService instance
///
/// Uses OnceLock for thread-safe lazy initialization.
/// The service is initialized once on first use and reused for all subsequent requests.
pub fn get_jwt_service() -> &'static JwtService {
    JWT_SERVICE.get_or_init(JwtService::from_config)
}

/// Access Token Claims
#[derive(Debug, Serialize, Deserialize)]
pub struct AccessClaims {
    pub sub: String,
    pub iat: i64,
    pub exp: i64,
    pub jti: String,
    pub token_type: String,
}

/// Refresh Token Claims
#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshClaims {
    pub sub: String,
    pub iat: i64,
    pub exp: i64,
    pub jti: String,
    pub token_type: String,
}

/// JWT Service for generating and validating tokens
pub struct JwtService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    access_token_minutes: u64,
    refresh_token_days: u64,
}

impl JwtService {
    pub fn new(secret: &str, access_token_minutes: u64, refresh_token_days: u64) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(secret.as_bytes()),
            decoding_key: DecodingKey::from_secret(secret.as_bytes()),
            access_token_minutes,
            refresh_token_days,
        }
    }

    /// Create JwtService from config
    pub fn from_config() -> Self {
        let rt = crate::config::get_runtime_config();

        // 获取 JWT secret，如果为空则生成一个安全的随机值
        let jwt_secret = rt
            .get(crate::config::keys::API_JWT_SECRET)
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| {
                use tracing::warn;
                warn!("JWT secret not configured or empty, generating secure random token");
                crate::utils::generate_secure_token(32)
            });

        Self::new(
            &jwt_secret,
            rt.get_u64_or(crate::config::keys::API_ACCESS_TOKEN_MINUTES, 15),
            rt.get_u64_or(crate::config::keys::API_REFRESH_TOKEN_DAYS, 7),
        )
    }

    /// Generate Access Token (short-lived)
    pub fn generate_access_token(&self) -> Result<String, jsonwebtoken::errors::Error> {
        let now = Utc::now();
        let claims = AccessClaims {
            sub: "admin".to_string(),
            iat: now.timestamp(),
            exp: (now + Duration::minutes(self.access_token_minutes as i64)).timestamp(),
            jti: uuid::Uuid::new_v4().to_string(),
            token_type: "access".to_string(),
        };

        encode(&Header::default(), &claims, &self.encoding_key)
    }

    /// Generate Refresh Token (long-lived)
    pub fn generate_refresh_token(&self) -> Result<String, jsonwebtoken::errors::Error> {
        let now = Utc::now();
        let claims = RefreshClaims {
            sub: "admin".to_string(),
            iat: now.timestamp(),
            exp: (now + Duration::days(self.refresh_token_days as i64)).timestamp(),
            jti: uuid::Uuid::new_v4().to_string(),
            token_type: "refresh".to_string(),
        };

        encode(&Header::default(), &claims, &self.encoding_key)
    }

    /// Validate Access Token
    pub fn validate_access_token(
        &self,
        token: &str,
    ) -> Result<AccessClaims, jsonwebtoken::errors::Error> {
        let token_data = decode::<AccessClaims>(token, &self.decoding_key, &Validation::default())?;

        // Verify token type
        if token_data.claims.token_type != "access" {
            return Err(jsonwebtoken::errors::Error::from(
                jsonwebtoken::errors::ErrorKind::InvalidToken,
            ));
        }

        Ok(token_data.claims)
    }

    /// Validate Refresh Token
    pub fn validate_refresh_token(
        &self,
        token: &str,
    ) -> Result<RefreshClaims, jsonwebtoken::errors::Error> {
        let token_data =
            decode::<RefreshClaims>(token, &self.decoding_key, &Validation::default())?;

        // Verify token type
        if token_data.claims.token_type != "refresh" {
            return Err(jsonwebtoken::errors::Error::from(
                jsonwebtoken::errors::ErrorKind::InvalidToken,
            ));
        }

        Ok(token_data.claims)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_service() -> JwtService {
        JwtService::new("test_secret_key_32_bytes_long!!", 15, 7)
    }

    #[test]
    fn test_generate_and_validate_access_token() {
        let service = create_test_service();
        let token = service.generate_access_token().unwrap();
        let claims = service.validate_access_token(&token).unwrap();

        assert_eq!(claims.sub, "admin");
        assert_eq!(claims.token_type, "access");
        assert!(claims.exp > claims.iat);
    }

    #[test]
    fn test_generate_and_validate_refresh_token() {
        let service = create_test_service();
        let token = service.generate_refresh_token().unwrap();
        let claims = service.validate_refresh_token(&token).unwrap();

        assert_eq!(claims.sub, "admin");
        assert_eq!(claims.token_type, "refresh");
        assert!(claims.exp > claims.iat);
    }

    #[test]
    fn test_access_token_rejected_as_refresh() {
        let service = create_test_service();
        let access_token = service.generate_access_token().unwrap();

        let result = service.validate_refresh_token(&access_token);
        assert!(result.is_err());
    }

    #[test]
    fn test_refresh_token_rejected_as_access() {
        let service = create_test_service();
        let refresh_token = service.generate_refresh_token().unwrap();

        let result = service.validate_access_token(&refresh_token);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_token_rejected() {
        let service = create_test_service();

        let result = service.validate_access_token("invalid.token.here");
        assert!(result.is_err());

        let result = service.validate_refresh_token("invalid.token.here");
        assert!(result.is_err());
    }

    #[test]
    fn test_wrong_secret_rejected() {
        let service1 = create_test_service();
        let service2 = JwtService::new("different_secret_key_32_bytes!!", 15, 7);

        let token = service1.generate_access_token().unwrap();
        let result = service2.validate_access_token(&token);
        assert!(result.is_err());
    }

    #[test]
    fn test_expired_token_rejected() {
        // 手动创建一个已过期的 token
        let service = create_test_service();

        // 创建一个过期时间在更久之前的 claims（超过默认 leeway）
        let now = chrono::Utc::now();
        let claims = AccessClaims {
            sub: "admin".to_string(),
            iat: (now - chrono::Duration::hours(2)).timestamp(),
            exp: (now - chrono::Duration::hours(1)).timestamp(), // 1 小时前过期
            jti: uuid::Uuid::new_v4().to_string(),
            token_type: "access".to_string(),
        };

        let encoding_key =
            jsonwebtoken::EncodingKey::from_secret(b"test_secret_key_32_bytes_long!!");
        let token =
            jsonwebtoken::encode(&jsonwebtoken::Header::default(), &claims, &encoding_key).unwrap();

        let result = service.validate_access_token(&token);
        assert!(
            result.is_err(),
            "Expected expired token to be rejected, but got: {:?}",
            result
        );
    }
}
