//! Admin API 帮助函数

use actix_web::HttpResponse;
use actix_web::cookie::{Cookie, SameSite};
use actix_web::http::StatusCode;
use serde::Serialize;

use crate::api::constants;
use crate::config::SameSitePolicy;
use crate::errors::ShortlinkerError;
use crate::utils::TimeParser;

use super::error_code::ErrorCode;
use super::types::ApiResponse;

/// 解析过期时间字符串，支持相对格式（如 '1h', '30m'）和 RFC3339 格式
pub fn parse_expires_at(expire_str: &str) -> Result<chrono::DateTime<chrono::Utc>, String> {
    TimeParser::parse_expire_time(expire_str).or_else(|_| {
        chrono::DateTime::parse_from_rfc3339(expire_str)
            .map(|time| time.with_timezone(&chrono::Utc))
            .map_err(|_| {
                format!(
                    "Invalid expires_at format: {}. Use relative format (e.g., '1h', '30m') or RFC3339 format",
                    expire_str
                )
            })
    })
}

/// 构建 JSON 响应
pub fn json_response<T: Serialize>(
    status: StatusCode,
    code: ErrorCode,
    message: impl Into<String>,
    data: Option<T>,
) -> HttpResponse {
    HttpResponse::build(status)
        .append_header(("Content-Type", "application/json; charset=utf-8"))
        .json(ApiResponse {
            code: code as i32,
            message: message.into(),
            data,
        })
}

/// 构建成功响应
pub fn success_response<T: Serialize>(data: T) -> HttpResponse {
    json_response(StatusCode::OK, ErrorCode::Success, "OK", Some(data))
}

/// 构建错误响应
pub fn error_response(status: StatusCode, error_code: ErrorCode, message: &str) -> HttpResponse {
    json_response::<()>(status, error_code, message, None)
}

/// 从 ShortlinkerError 构建错误响应（自动映射 HTTP 状态码和 ErrorCode）
pub fn error_from_shortlinker(err: &ShortlinkerError) -> HttpResponse {
    let status = err.http_status();
    let error_code = ErrorCode::from(err.clone());
    error_response(status, error_code, err.message())
}

/// 统一 Result → HttpResponse 转换
///
/// 成功时返回 200 OK + JSON 数据，失败时自动映射 ShortlinkerError。
pub fn api_result<T, E>(result: Result<T, E>) -> HttpResponse
where
    T: Serialize,
    E: Into<ShortlinkerError>,
{
    match result {
        Ok(data) => success_response(data),
        Err(e) => {
            let err: ShortlinkerError = e.into();
            error_from_shortlinker(&err)
        }
    }
}

/// Cookie 构建器，消除重复的 cookie 创建代码
pub struct CookieBuilder {
    same_site: SameSite,
    secure: bool,
    domain: Option<String>,
    access_token_minutes: u64,
    refresh_token_days: u64,
    admin_prefix: String,
}

impl CookieBuilder {
    pub fn from_config() -> Self {
        let config = crate::config::get_config();

        let same_site = match config.api.cookie_same_site {
            SameSitePolicy::Strict => SameSite::Strict,
            SameSitePolicy::None => SameSite::None,
            SameSitePolicy::Lax => SameSite::Lax,
        };

        Self {
            same_site,
            secure: config.api.cookie_secure,
            domain: config.api.cookie_domain.clone(),
            access_token_minutes: config.api.access_token_minutes,
            refresh_token_days: config.api.refresh_token_days,
            admin_prefix: config.routes.admin_prefix.clone(),
        }
    }

    /// 基础 cookie 构建方法，消除重复代码
    fn build_cookie_base(
        &self,
        name: String,
        value: String,
        path: String,
        max_age: actix_web::cookie::time::Duration,
    ) -> Cookie<'static> {
        let mut cookie = Cookie::new(name, value);
        cookie.set_path(path);
        cookie.set_http_only(true);
        cookie.set_secure(self.secure);
        cookie.set_same_site(self.same_site);
        cookie.set_max_age(max_age);
        if let Some(ref domain) = self.domain {
            cookie.set_domain(domain.clone());
        }
        cookie
    }

    pub fn build_access_cookie(&self, token: String) -> Cookie<'static> {
        self.build_cookie_base(
            constants::ACCESS_COOKIE_NAME.to_string(),
            token,
            "/".to_string(),
            actix_web::cookie::time::Duration::minutes(self.access_token_minutes as i64),
        )
    }

    pub fn build_refresh_cookie(&self, token: String) -> Cookie<'static> {
        let refresh_path = format!("{}/v1/auth", self.admin_prefix);
        self.build_cookie_base(
            constants::REFRESH_COOKIE_NAME.to_string(),
            token,
            refresh_path,
            actix_web::cookie::time::Duration::days(self.refresh_token_days as i64),
        )
    }

    pub fn build_expired_access_cookie(&self) -> Cookie<'static> {
        self.build_cookie_base(
            constants::ACCESS_COOKIE_NAME.to_string(),
            String::new(),
            "/".to_string(),
            actix_web::cookie::time::Duration::ZERO,
        )
    }

    pub fn build_expired_refresh_cookie(&self) -> Cookie<'static> {
        let refresh_path = format!("{}/v1/auth", self.admin_prefix);
        self.build_cookie_base(
            constants::REFRESH_COOKIE_NAME.to_string(),
            String::new(),
            refresh_path,
            actix_web::cookie::time::Duration::ZERO,
        )
    }

    /// 构建 CSRF Cookie（非 HttpOnly，前端需要读取）
    pub fn build_csrf_cookie(&self, token: String) -> Cookie<'static> {
        let mut cookie = Cookie::new(constants::CSRF_COOKIE_NAME.to_string(), token);
        // CSRF cookie path 与 admin_prefix 保持一致
        cookie.set_path(self.admin_prefix.clone());
        // CSRF cookie 不能是 HttpOnly，因为前端 JS 需要读取它
        cookie.set_http_only(false);
        cookie.set_secure(self.secure);
        // CSRF cookie 使用 Lax，允许顶级导航携带但防止跨站请求
        cookie.set_same_site(SameSite::Lax);
        // 与 access token 同步过期
        cookie.set_max_age(actix_web::cookie::time::Duration::minutes(
            self.access_token_minutes as i64,
        ));
        if let Some(ref domain) = self.domain {
            cookie.set_domain(domain.clone());
        }
        cookie
    }

    /// 构建过期的 CSRF Cookie（登出时清除）
    pub fn build_expired_csrf_cookie(&self) -> Cookie<'static> {
        let mut cookie = Cookie::new(constants::CSRF_COOKIE_NAME.to_string(), String::new());
        // CSRF cookie path 与 admin_prefix 保持一致
        cookie.set_path(self.admin_prefix.clone());
        cookie.set_http_only(false);
        cookie.set_secure(self.secure);
        cookie.set_same_site(SameSite::Lax);
        cookie.set_max_age(actix_web::cookie::time::Duration::ZERO);
        if let Some(ref domain) = self.domain {
            cookie.set_domain(domain.clone());
        }
        cookie
    }

    pub fn refresh_cookie_name(&self) -> &str {
        constants::REFRESH_COOKIE_NAME
    }

    pub fn access_token_minutes(&self) -> u64 {
        self.access_token_minutes
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, Utc};

    #[test]
    fn test_parse_expires_at_relative_hours() {
        let result = parse_expires_at("1h");
        assert!(result.is_ok());
        let time = result.unwrap();
        let now = Utc::now();
        // 应该在 59-61 分钟之间（允许一些误差）
        let diff = (time - now).num_minutes();
        assert!((59..=61).contains(&diff));
    }

    #[test]
    fn test_parse_expires_at_relative_minutes() {
        let result = parse_expires_at("30m");
        assert!(result.is_ok());
        let time = result.unwrap();
        let now = Utc::now();
        let diff = (time - now).num_minutes();
        assert!((29..=31).contains(&diff));
    }

    #[test]
    fn test_parse_expires_at_relative_days() {
        let result = parse_expires_at("7d");
        assert!(result.is_ok());
        let time = result.unwrap();
        let now = Utc::now();
        let diff = (time - now).num_days();
        assert!((6..=7).contains(&diff));
    }

    #[test]
    fn test_parse_expires_at_rfc3339() {
        let future = Utc::now() + Duration::hours(2);
        let rfc3339_str = future.to_rfc3339();
        let result = parse_expires_at(&rfc3339_str);
        assert!(result.is_ok());
        let parsed = result.unwrap();
        // 解析后的时间应该接近原始时间（秒级精度）
        let diff = (parsed - future).num_seconds().abs();
        assert!(diff <= 1);
    }

    #[test]
    fn test_parse_expires_at_invalid_format() {
        let result = parse_expires_at("invalid");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("Invalid expires_at format"));
    }

    #[test]
    fn test_parse_expires_at_empty_string() {
        let result = parse_expires_at("");
        assert!(result.is_err());
    }

    #[test]
    fn test_json_response_structure() {
        let response = json_response(StatusCode::OK, ErrorCode::Success, "OK", Some("test_data"));
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[test]
    fn test_success_response() {
        let response = success_response("success_data");
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[test]
    fn test_error_response() {
        let response = error_response(
            StatusCode::BAD_REQUEST,
            ErrorCode::BadRequest,
            "Something went wrong",
        );
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_error_response_not_found() {
        let response = error_response(
            StatusCode::NOT_FOUND,
            ErrorCode::NotFound,
            "Resource not found",
        );
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn test_error_response_internal_error() {
        let response = error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            ErrorCode::InternalServerError,
            "Internal error",
        );
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }
}
