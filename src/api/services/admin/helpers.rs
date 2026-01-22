//! Admin API 帮助函数

use actix_web::HttpResponse;
use actix_web::cookie::{Cookie, SameSite};
use actix_web::http::StatusCode;
use serde::Serialize;

use crate::config::SameSitePolicy;
use crate::utils::TimeParser;

use super::types::{ApiResponse, ErrorData};

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
pub fn json_response<T: Serialize>(status: StatusCode, code: i32, data: T) -> HttpResponse {
    HttpResponse::build(status)
        .append_header(("Content-Type", "application/json; charset=utf-8"))
        .json(ApiResponse { code, data })
}

/// 构建成功响应
pub fn success_response<T: Serialize>(data: T) -> HttpResponse {
    json_response(StatusCode::OK, 0, data)
}

/// 构建错误响应
pub fn error_response(status: StatusCode, message: &str) -> HttpResponse {
    json_response(
        status,
        1,
        ErrorData {
            error: message.to_string(),
        },
    )
}

/// Cookie 构建器，消除重复的 cookie 创建代码
pub struct CookieBuilder {
    same_site: SameSite,
    secure: bool,
    domain: Option<String>,
    access_cookie_name: String,
    refresh_cookie_name: String,
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
            access_cookie_name: config.api.access_cookie_name.clone(),
            refresh_cookie_name: config.api.refresh_cookie_name.clone(),
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
            self.access_cookie_name.clone(),
            token,
            "/".to_string(),
            actix_web::cookie::time::Duration::minutes(self.access_token_minutes as i64),
        )
    }

    pub fn build_refresh_cookie(&self, token: String) -> Cookie<'static> {
        let refresh_path = format!("{}/v1/auth", self.admin_prefix);
        self.build_cookie_base(
            self.refresh_cookie_name.clone(),
            token,
            refresh_path,
            actix_web::cookie::time::Duration::days(self.refresh_token_days as i64),
        )
    }

    pub fn build_expired_access_cookie(&self) -> Cookie<'static> {
        self.build_cookie_base(
            self.access_cookie_name.clone(),
            String::new(),
            "/".to_string(),
            actix_web::cookie::time::Duration::ZERO,
        )
    }

    pub fn build_expired_refresh_cookie(&self) -> Cookie<'static> {
        let refresh_path = format!("{}/v1/auth", self.admin_prefix);
        self.build_cookie_base(
            self.refresh_cookie_name.clone(),
            String::new(),
            refresh_path,
            actix_web::cookie::time::Duration::ZERO,
        )
    }

    pub fn refresh_cookie_name(&self) -> &str {
        &self.refresh_cookie_name
    }

    pub fn access_token_minutes(&self) -> u64 {
        self.access_token_minutes
    }
}
