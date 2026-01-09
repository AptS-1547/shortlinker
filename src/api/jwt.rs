use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};

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
        let config = crate::config::get_config();
        Self::new(
            &config.api.jwt_secret,
            config.api.access_token_minutes,
            config.api.refresh_token_days,
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
