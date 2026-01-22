//! Admin API 认证相关端点

use actix_governor::{Governor, GovernorConfigBuilder, KeyExtractor, SimpleKeyExtractionError};
use actix_web::dev::ServiceRequest;
use actix_web::http::StatusCode;
use actix_web::{HttpRequest, HttpResponse, Responder, Result as ActixResult, web};
use governor::middleware::NoOpMiddleware;
use tracing::{debug, error, info, warn};

use crate::api::jwt::JwtService;
use crate::config::get_config;
use crate::utils::password::verify_password;

use super::helpers::{CookieBuilder, error_response, success_response};
use super::types::{ApiResponse, AuthSuccessResponse, LoginCredentials, MessageResponse};

/// 基于 IP 地址的限流 key 提取器
///
/// 优先使用 X-Forwarded-For 头部（反向代理场景），
/// 回退到直连 IP。
#[derive(Clone, Copy)]
pub struct LoginKeyExtractor;

impl KeyExtractor for LoginKeyExtractor {
    type Key = String;
    type KeyExtractionError = SimpleKeyExtractionError<&'static str>;

    fn extract(&self, req: &ServiceRequest) -> Result<Self::Key, Self::KeyExtractionError> {
        // 优先使用 X-Forwarded-For（反向代理场景）
        req.connection_info()
            .realip_remote_addr()
            .map(|ip| ip.to_string())
            .ok_or_else(|| SimpleKeyExtractionError::new("Unable to extract IP"))
    }
}

/// 创建登录限流器
///
/// 配置：每秒补充 2 个令牌，突发最多 5 次请求
/// 超限返回 HTTP 429 Too Many Requests
pub fn login_rate_limiter() -> Governor<LoginKeyExtractor, NoOpMiddleware> {
    let config = GovernorConfigBuilder::default()
        .seconds_per_request(1) // 令牌补充速率：每秒 1 个（等效于 per_second(1)）
        .burst_size(5) // 突发最多 5 次请求
        .key_extractor(LoginKeyExtractor)
        .finish()
        .expect("Invalid rate limit config");

    debug!("Login rate limiter created: 1 req/s, burst 5");
    Governor::new(&config)
}

/// 登录验证 - 检查管理员 token
pub async fn check_admin_token(
    _req: HttpRequest,
    login_body: web::Json<LoginCredentials>,
) -> ActixResult<impl Responder> {
    let config = get_config();
    let admin_token = &config.api.admin_token;

    // 验证密码（启动时已自动迁移明文为哈希）
    let password_valid = match verify_password(&login_body.password, admin_token) {
        Ok(valid) => valid,
        Err(e) => {
            error!("Admin API: password verification error: {}", e);
            return Ok(error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Authentication error",
            ));
        }
    };

    if !password_valid {
        error!("Admin API: login failed - invalid token");
        return Ok(error_response(
            StatusCode::UNAUTHORIZED,
            "Invalid admin token",
        ));
    }

    info!("Admin API: login successful");

    // Generate JWT tokens
    let jwt_service = JwtService::from_config();
    let access_token = match jwt_service.generate_access_token() {
        Ok(token) => token,
        Err(e) => {
            error!("Admin API: failed to generate access token: {}", e);
            return Ok(error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to generate token",
            ));
        }
    };

    let refresh_token = match jwt_service.generate_refresh_token() {
        Ok(token) => token,
        Err(e) => {
            error!("Admin API: failed to generate refresh token: {}", e);
            return Ok(error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to generate token",
            ));
        }
    };

    // Build cookies using helper
    let cookie_builder = CookieBuilder::from_config();
    let access_cookie = cookie_builder.build_access_cookie(access_token);
    let refresh_cookie = cookie_builder.build_refresh_cookie(refresh_token);

    Ok(HttpResponse::Ok()
        .cookie(access_cookie)
        .cookie(refresh_cookie)
        .append_header(("Content-Type", "application/json; charset=utf-8"))
        .json(ApiResponse {
            code: 0,
            data: AuthSuccessResponse {
                message: "Login successful".to_string(),
                expires_in: cookie_builder.access_token_minutes() * 60,
            },
        }))
}

/// 刷新 token
pub async fn refresh_token(req: HttpRequest) -> ActixResult<impl Responder> {
    let cookie_builder = CookieBuilder::from_config();

    // Get refresh token from cookie
    let refresh_token = match req.cookie(cookie_builder.refresh_cookie_name()) {
        Some(cookie) => cookie.value().to_string(),
        None => {
            warn!("Admin API: refresh token not found in cookie");
            return Ok(error_response(
                StatusCode::UNAUTHORIZED,
                "Refresh token not found",
            ));
        }
    };

    // Validate refresh token
    let jwt_service = JwtService::from_config();
    if let Err(e) = jwt_service.validate_refresh_token(&refresh_token) {
        warn!("Admin API: invalid refresh token: {}", e);
        return Ok(error_response(
            StatusCode::UNAUTHORIZED,
            "Invalid refresh token",
        ));
    }

    info!("Admin API: token refresh successful");

    // Generate new tokens (sliding expiration)
    let new_access_token = match jwt_service.generate_access_token() {
        Ok(token) => token,
        Err(e) => {
            error!("Admin API: failed to generate access token: {}", e);
            return Ok(error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to generate token",
            ));
        }
    };

    let new_refresh_token = match jwt_service.generate_refresh_token() {
        Ok(token) => token,
        Err(e) => {
            error!("Admin API: failed to generate refresh token: {}", e);
            return Ok(error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to generate token",
            ));
        }
    };

    // Build cookies
    let access_cookie = cookie_builder.build_access_cookie(new_access_token);
    let refresh_cookie = cookie_builder.build_refresh_cookie(new_refresh_token);

    Ok(HttpResponse::Ok()
        .cookie(access_cookie)
        .cookie(refresh_cookie)
        .append_header(("Content-Type", "application/json; charset=utf-8"))
        .json(ApiResponse {
            code: 0,
            data: AuthSuccessResponse {
                message: "Token refreshed".to_string(),
                expires_in: cookie_builder.access_token_minutes() * 60,
            },
        }))
}

/// 登出 - 清除 cookies
pub async fn logout(_req: HttpRequest) -> ActixResult<impl Responder> {
    info!("Admin API: logout");

    let cookie_builder = CookieBuilder::from_config();
    let access_cookie = cookie_builder.build_expired_access_cookie();
    let refresh_cookie = cookie_builder.build_expired_refresh_cookie();

    Ok(HttpResponse::Ok()
        .cookie(access_cookie)
        .cookie(refresh_cookie)
        .append_header(("Content-Type", "application/json; charset=utf-8"))
        .json(ApiResponse {
            code: 0,
            data: MessageResponse {
                message: "Logout successful".to_string(),
            },
        }))
}

/// 验证 token - 如果中间件通过，则 token 有效
pub async fn verify_token(_req: HttpRequest) -> ActixResult<impl Responder> {
    Ok(success_response(MessageResponse {
        message: "Token is valid".to_string(),
    }))
}
