//! Admin API 认证相关端点

use actix_web::http::StatusCode;
use actix_web::{HttpRequest, HttpResponse, Responder, Result as ActixResult, web};
use tracing::{error, info, warn};

use crate::api::jwt::JwtService;
use crate::config::get_config;

use super::helpers::{CookieBuilder, error_response, success_response};
use super::types::{ApiResponse, LoginCredentials};

/// 登录验证 - 检查管理员 token
pub async fn check_admin_token(
    _req: HttpRequest,
    login_body: web::Json<LoginCredentials>,
) -> ActixResult<impl Responder> {
    let config = get_config();
    let admin_token = &config.api.admin_token;

    if login_body.password != *admin_token {
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
            data: serde_json::json!({
                "message": "Login successful",
                "expires_in": cookie_builder.access_token_minutes() * 60
            }),
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
            data: serde_json::json!({
                "message": "Token refreshed",
                "expires_in": cookie_builder.access_token_minutes() * 60
            }),
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
            data: serde_json::json!({
                "message": "Logout successful"
            }),
        }))
}

/// 验证 token - 如果中间件通过，则 token 有效
pub async fn verify_token(_req: HttpRequest) -> ActixResult<impl Responder> {
    Ok(success_response(serde_json::json!({
        "message": "Token is valid"
    })))
}
