//! Admin API 认证相关端点

use actix_governor::Governor;
use actix_web::{HttpRequest, HttpResponse, Responder, Result as ActixResult, web};
use base64::Engine;
use governor::middleware::NoOpMiddleware;
use std::num::{NonZeroU32, NonZeroU64};
use tracing::{debug, error, info, warn};

use crate::api::jwt::get_jwt_service;
use crate::config::{get_runtime_config, keys};

use crate::errors::ShortlinkerError;

use super::error_code::ErrorCode;
use super::helpers::{CookieBuilder, error_from_shortlinker, success_response};
use super::types::{ApiResponse, AuthSuccessResponse, LoginCredentials, MessageResponse};

/// 生成 CSRF Token（32 bytes = 256 bits，Base64 编码）
fn generate_csrf_token() -> String {
    let bytes: [u8; 32] = rand::random();
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
}

fn trusted_proxies() -> Vec<String> {
    let mut trusted = get_runtime_config().get_json_or(keys::API_TRUSTED_PROXIES, Vec::new());
    #[cfg(unix)]
    if crate::config::get_config().server.unix_socket.is_some() {
        trusted.extend(["127.0.0.0/8".to_string(), "::1/128".to_string()]);
    }
    trusted
}

/// 创建登录限流器
///
/// 配置：每秒补充 2 个令牌，突发最多 5 次请求
/// 超限返回 HTTP 429 Too Many Requests
pub fn login_rate_limiter()
-> Governor<aster_forge_actix_middleware::rate_limit::TrustedProxyIpKeyExtractor, NoOpMiddleware> {
    let config =
        aster_forge_actix_middleware::rate_limit::build_ip_governor_config_with_rejection_response(
            NonZeroU64::new(1).expect("login interval is non-zero"),
            NonZeroU32::new(5).expect("login burst is non-zero"),
            &trusted_proxies(),
            |retry_after, mut response| {
                response.status(actix_web::http::StatusCode::TOO_MANY_REQUESTS);
                response.json(ApiResponse::<()> {
                    code: ErrorCode::RateLimitExceeded as i32,
                    message: format!("Too many requests, retry in {retry_after}s"),
                    data: None,
                })
            },
        );

    debug!("Login rate limiter created: 1 req/s, burst 5");
    Governor::new(&config)
}

/// 创建 refresh token 限流器
///
/// 配置：每 10 秒补充 1 个令牌，突发最多 10 次请求
/// 比 login 限流更宽松，因为 refresh 是正常使用场景
/// 超限返回 HTTP 429 Too Many Requests
pub fn refresh_rate_limiter()
-> Governor<aster_forge_actix_middleware::rate_limit::TrustedProxyIpKeyExtractor, NoOpMiddleware> {
    let config =
        aster_forge_actix_middleware::rate_limit::build_ip_governor_config_with_rejection_response(
            NonZeroU64::new(10).expect("refresh interval is non-zero"),
            NonZeroU32::new(10).expect("refresh burst is non-zero"),
            &trusted_proxies(),
            |retry_after, mut response| {
                response.status(actix_web::http::StatusCode::TOO_MANY_REQUESTS);
                response.json(ApiResponse::<()> {
                    code: ErrorCode::RateLimitExceeded as i32,
                    message: format!("Too many requests, retry in {retry_after}s"),
                    data: None,
                })
            },
        );

    debug!("Refresh rate limiter created: 1 req/10s, burst 10");
    Governor::new(&config)
}

/// 登录验证 - 检查管理员 token
#[aster_forge_api_docs_macros::path(
        post,
        path = "/admin/v1/auth/login",
        tag = "auth",
        operation_id = "admin_login",
        request_body = LoginCredentials,
        responses(
            (status = 200, description = "Login successful", body = ApiResponse<AuthSuccessResponse>),
            (status = 401, description = "Invalid administrator token"),
            (status = 429, description = "Login rate limit exceeded"),
        )
)]
pub async fn check_admin_token(
    req: HttpRequest,
    login_body: web::Json<LoginCredentials>,
) -> ActixResult<impl Responder> {
    let peer = req.peer_addr().map(|address| address.ip());
    #[cfg(unix)]
    let peer = peer.or_else(|| {
        crate::config::get_config()
            .server
            .unix_socket
            .as_ref()
            .map(|_| std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST))
    });
    let client_ip = peer
        .map(|peer| {
            aster_forge_actix_middleware::client_ip::real_ip_from_headers(
                req.headers(),
                peer,
                &trusted_proxies(),
            )
            .to_string()
        })
        .unwrap_or_else(|| "unknown".to_string());
    let rt = get_runtime_config();
    let admin_token = rt.get_or(keys::API_ADMIN_TOKEN, "");

    // 验证密码（启动时已自动迁移明文为哈希）
    let password_valid =
        match aster_forge_crypto::verify_password(&login_body.password, &admin_token) {
            Ok(valid) => valid,
            Err(e) => {
                error!("Admin API: password verification error: {}", e);
                return Ok(error_from_shortlinker(&ShortlinkerError::internal_error(
                    "Authentication error",
                )));
            }
        };

    if !password_valid {
        warn!(
            "Admin API: login failed - invalid token (from {})",
            client_ip
        );
        return Ok(error_from_shortlinker(
            &ShortlinkerError::auth_password_invalid("Invalid admin token"),
        ));
    }

    info!("Admin API: login successful (from {})", client_ip);

    // Generate JWT tokens using cached service
    let jwt_service = get_jwt_service();
    let access_token = match jwt_service.generate_access_token() {
        Ok(token) => token,
        Err(e) => {
            error!("Admin API: failed to generate access token: {}", e);
            return Ok(error_from_shortlinker(&ShortlinkerError::internal_error(
                "Failed to generate token",
            )));
        }
    };

    let refresh_token = match jwt_service.generate_refresh_token() {
        Ok(token) => token,
        Err(e) => {
            error!("Admin API: failed to generate refresh token: {}", e);
            return Ok(error_from_shortlinker(&ShortlinkerError::internal_error(
                "Failed to generate token",
            )));
        }
    };

    // Build cookies using helper
    let cookie_builder = CookieBuilder::from_config();
    let access_cookie = cookie_builder.build_access_cookie(access_token);
    let refresh_cookie = cookie_builder.build_refresh_cookie(refresh_token);

    // Generate and build CSRF cookie
    let csrf_token = generate_csrf_token();
    let csrf_cookie = cookie_builder.build_csrf_cookie(csrf_token);

    Ok(HttpResponse::Ok()
        .cookie(access_cookie)
        .cookie(refresh_cookie)
        .cookie(csrf_cookie)
        .append_header(("Content-Type", "application/json; charset=utf-8"))
        .json(ApiResponse {
            code: ErrorCode::Success as i32,
            message: "Login successful".to_string(),
            data: Some(AuthSuccessResponse {
                message: "Login successful".to_string(),
                expires_in: cookie_builder.access_token_minutes() * 60,
            }),
        }))
}

/// 刷新 token
#[aster_forge_api_docs_macros::path(
        post,
        path = "/admin/v1/auth/refresh",
        tag = "auth",
        operation_id = "refresh_admin_token",
        responses(
            (status = 200, description = "Token refreshed", body = ApiResponse<AuthSuccessResponse>),
            (status = 401, description = "Refresh token is missing or invalid"),
            (status = 429, description = "Refresh rate limit exceeded"),
        )
)]
pub async fn refresh_token(req: HttpRequest) -> ActixResult<impl Responder> {
    let cookie_builder = CookieBuilder::from_config();

    // Get refresh token from cookie
    let refresh_token = match req.cookie(cookie_builder.refresh_cookie_name()) {
        Some(cookie) => cookie.value().to_string(),
        None => {
            info!("Admin API: refresh token not found in cookie");
            return Ok(error_from_shortlinker(
                &ShortlinkerError::auth_token_invalid("Refresh token not found"),
            ));
        }
    };

    // Validate refresh token using cached service
    let jwt_service = get_jwt_service();
    if let Err(e) = jwt_service.validate_refresh_token(&refresh_token) {
        info!("Admin API: invalid refresh token: {}", e);
        return Ok(error_from_shortlinker(
            &ShortlinkerError::auth_token_invalid("Invalid refresh token"),
        ));
    }

    info!("Admin API: token refresh successful");

    // Generate new tokens (sliding expiration)
    let new_access_token = match jwt_service.generate_access_token() {
        Ok(token) => token,
        Err(e) => {
            error!("Admin API: failed to generate access token: {}", e);
            return Ok(error_from_shortlinker(&ShortlinkerError::internal_error(
                "Failed to generate token",
            )));
        }
    };

    let new_refresh_token = match jwt_service.generate_refresh_token() {
        Ok(token) => token,
        Err(e) => {
            error!("Admin API: failed to generate refresh token: {}", e);
            return Ok(error_from_shortlinker(&ShortlinkerError::internal_error(
                "Failed to generate token",
            )));
        }
    };

    // Build cookies
    let access_cookie = cookie_builder.build_access_cookie(new_access_token);
    let refresh_cookie = cookie_builder.build_refresh_cookie(new_refresh_token);

    // Generate and build CSRF cookie (refresh on token refresh)
    let csrf_token = generate_csrf_token();
    let csrf_cookie = cookie_builder.build_csrf_cookie(csrf_token);

    Ok(HttpResponse::Ok()
        .cookie(access_cookie)
        .cookie(refresh_cookie)
        .cookie(csrf_cookie)
        .append_header(("Content-Type", "application/json; charset=utf-8"))
        .json(ApiResponse {
            code: ErrorCode::Success as i32,
            message: "Token refreshed".to_string(),
            data: Some(AuthSuccessResponse {
                message: "Token refreshed".to_string(),
                expires_in: cookie_builder.access_token_minutes() * 60,
            }),
        }))
}

/// 登出 - 清除 cookies
#[aster_forge_api_docs_macros::path(
        post,
        path = "/admin/v1/auth/logout",
        tag = "auth",
        operation_id = "admin_logout",
        responses((status = 200, description = "Logout successful", body = ApiResponse<MessageResponse>))
)]
pub async fn logout(_req: HttpRequest) -> ActixResult<impl Responder> {
    info!("Admin API: logout");

    let cookie_builder = CookieBuilder::from_config();
    let access_cookie = cookie_builder.build_expired_access_cookie();
    let refresh_cookie = cookie_builder.build_expired_refresh_cookie();
    let csrf_cookie = cookie_builder.build_expired_csrf_cookie();

    Ok(HttpResponse::Ok()
        .cookie(access_cookie)
        .cookie(refresh_cookie)
        .cookie(csrf_cookie)
        .append_header(("Content-Type", "application/json; charset=utf-8"))
        .json(ApiResponse {
            code: ErrorCode::Success as i32,
            message: "Logout successful".to_string(),
            data: Some(MessageResponse {
                message: "Logout successful".to_string(),
            }),
        }))
}

/// 验证 token - 如果中间件通过，则 token 有效
#[aster_forge_api_docs_macros::path(
        get,
        path = "/admin/v1/auth/verify",
        tag = "auth",
        operation_id = "verify_admin_token",
        responses(
            (status = 200, description = "Token is valid", body = ApiResponse<MessageResponse>),
            (status = 401, description = "Token is missing or invalid"),
        )
)]
pub async fn verify_token(_req: HttpRequest) -> ActixResult<impl Responder> {
    Ok(success_response(MessageResponse {
        message: "Token is valid".to_string(),
    }))
}
