//! Admin API 认证相关端点

use actix_governor::{Governor, GovernorConfigBuilder, KeyExtractor, SimpleKeyExtractionError};
use actix_web::dev::ServiceRequest;
use actix_web::{HttpRequest, HttpResponse, Responder, Result as ActixResult, web};
use base64::Engine;
use governor::middleware::NoOpMiddleware;
use rand::Rng;
use std::net::{IpAddr, SocketAddr};
use tracing::{debug, error, info, warn};

use crate::api::jwt::JwtService;
use crate::config::{get_config, get_runtime_config, keys};
use crate::utils::password::verify_password;

use crate::errors::ShortlinkerError;

use super::error_code::ErrorCode;
use super::helpers::{CookieBuilder, error_from_shortlinker, success_response};
use super::types::{ApiResponse, AuthSuccessResponse, LoginCredentials, MessageResponse};

/// 生成 CSRF Token（32 bytes = 256 bits，Base64 编码）
fn generate_csrf_token() -> String {
    let mut bytes = [0u8; 32];
    rand::rng().fill(&mut bytes);
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
}

/// 基于 IP 地址的限流 key 提取器（智能版 v2）
///
/// 策略（按优先级）：
/// 1. Unix Socket 模式（检查配置）→ 强制要求 X-Forwarded-For
/// 2. 显式配置 trusted_proxies 且匹配 → 使用 X-Forwarded-For
/// 3. 未配置 trusted_proxies 且连接来自私有 IP/localhost → 自动检测代理，使用 X-Forwarded-For
/// 4. 默认 → 使用连接 IP（公网直连场景）
#[derive(Clone, Copy)]
pub struct LoginKeyExtractor;

impl KeyExtractor for LoginKeyExtractor {
    type Key = String;
    type KeyExtractionError = SimpleKeyExtractionError<&'static str>;

    fn extract(&self, req: &ServiceRequest) -> Result<Self::Key, Self::KeyExtractionError> {
        let conn_info = req.connection_info();
        let config = get_config();
        let rt = get_runtime_config();

        // 步骤 1: 检查是否配置了 Unix Socket 模式
        #[cfg(unix)]
        if config.server.unix_socket.is_some() {
            // Unix Socket 模式：强制要求 X-Forwarded-For
            if let Some(real_ip) = conn_info.realip_remote_addr() {
                debug!("Unix Socket mode: using X-Forwarded-For: {}", real_ip);
                return Ok(real_ip.to_string());
            } else {
                // Unix Socket 模式下没有 X-Forwarded-For 是配置错误
                error!(
                    "Unix Socket mode enabled but X-Forwarded-For header missing. \
                     Ensure nginx/proxy sets: proxy_set_header X-Forwarded-For $remote_addr;"
                );
                return Err(SimpleKeyExtractionError::new(
                    "Unix Socket mode requires X-Forwarded-For header",
                ));
            }
        }

        // 步骤 2: TCP 连接模式 - 获取 peer_addr
        let peer_ip = conn_info.peer_addr().ok_or_else(|| {
            // 这不应该发生（非 Unix Socket 模式下 peer_addr 为 None）
            warn!("Unable to extract peer IP in TCP mode - this should not happen");
            SimpleKeyExtractionError::new("Unable to extract peer IP")
        })?;

        // 步骤 3: 检查是否显式配置了可信代理（优先级最高）
        let trusted_proxies_json = rt.get_or(keys::API_TRUSTED_PROXIES, "[]");
        let trusted_proxies: Vec<String> = match serde_json::from_str(&trusted_proxies_json) {
            Ok(v) => v,
            Err(e) => {
                warn!(
                    "Invalid JSON for trusted_proxies '{}': {}, using empty list",
                    trusted_proxies_json, e
                );
                Vec::new()
            }
        };
        if !trusted_proxies.is_empty() {
            if is_trusted_proxy(peer_ip, &trusted_proxies) {
                let real_ip = conn_info.realip_remote_addr().unwrap_or(peer_ip);
                debug!("Trusted proxy (explicit): {} -> {}", peer_ip, real_ip);
                return Ok(real_ip.to_string());
            }
            // 显式配置了但不匹配 → 使用连接 IP（不信任 X-Forwarded-For）
            debug!("Connection from {}, not in trusted_proxies", peer_ip);
            return Ok(peer_ip.to_string());
        }

        // 步骤 4: 未配置 trusted_proxies → 智能检测
        if let Ok(socket_addr) = peer_ip.parse::<SocketAddr>() {
            let ip_addr = socket_addr.ip();

            // 检查是否为私有 IP 或 localhost
            let is_private_or_local = match ip_addr {
                IpAddr::V4(v4) => v4.is_private() || v4.is_loopback(),
                IpAddr::V6(v6) => {
                    // IPv6 私有地址：
                    // - fc00::/7 (ULA, RFC 4193): fc00::/8 + fd00::/8
                    // - fe80::/10 (Link-local)
                    // - ::1 (Loopback)
                    v6.is_loopback()
                        || (v6.segments()[0] & 0xfe00) == 0xfc00 // fc00::/7 (包含 fc00 和 fd00)
                        || (v6.segments()[0] & 0xffc0) == 0xfe80 // fe80::/10 (link-local)
                }
            };

            if is_private_or_local {
                // 连接来自私有 IP/localhost → 假设有反向代理
                if let Some(real_ip) = conn_info.realip_remote_addr() {
                    debug!(
                        "Auto-detect proxy (private IP {}): using X-Forwarded-For: {}",
                        peer_ip, real_ip
                    );
                    return Ok(real_ip.to_string());
                }
                // 私有 IP 但无 X-Forwarded-For（可能是内网直连）
                debug!("Private IP {} without X-Forwarded-For", peer_ip);
            }
        }

        // 步骤 5: 默认使用连接 IP（公网直连或内网直连无 X-Forwarded-For）
        Ok(peer_ip.to_string())
    }
}

/// 检查 IP 是否在可信代理列表中
fn is_trusted_proxy(ip: &str, trusted_proxies: &[String]) -> bool {
    use std::net::{IpAddr, SocketAddr};

    // 先尝试解析为 SocketAddr（支持 ip:port），如果失败再尝试纯 IpAddr
    let ip_addr = if let Ok(socket_addr) = ip.parse::<SocketAddr>() {
        socket_addr.ip()
    } else if let Ok(ip_addr) = ip.parse::<IpAddr>() {
        ip_addr
    } else {
        return false;
    };

    for proxy in trusted_proxies {
        if proxy.contains('/') {
            // CIDR 格式（如 "192.168.1.0/24"）
            if ip_in_cidr(&ip_addr, proxy) {
                return true;
            }
        } else {
            // 单 IP
            if let Ok(proxy_addr) = proxy.parse::<IpAddr>()
                && ip_addr == proxy_addr
            {
                return true;
            }
        }
    }
    false
}

/// CIDR 检查（简易实现）
fn ip_in_cidr(ip: &IpAddr, cidr: &str) -> bool {
    let Some((network, prefix_len)) = cidr.split_once('/') else {
        return false;
    };

    let Ok(prefix_len): Result<u8, _> = prefix_len.parse() else {
        return false;
    };

    let Ok(network_addr) = network.parse::<IpAddr>() else {
        return false;
    };

    match (ip, network_addr) {
        (IpAddr::V4(ip), IpAddr::V4(net)) => {
            if prefix_len > 32 {
                return false;
            }
            let mask = u32::MAX.checked_shl(32 - prefix_len as u32).unwrap_or(0);
            let ip_bits = u32::from_be_bytes(ip.octets());
            let net_bits = u32::from_be_bytes(net.octets());
            (ip_bits & mask) == (net_bits & mask)
        }
        (IpAddr::V6(ip), IpAddr::V6(net)) => {
            if prefix_len > 128 {
                return false;
            }
            let mask = u128::MAX.checked_shl(128 - prefix_len as u32).unwrap_or(0);
            let ip_bits = u128::from_be_bytes(ip.octets());
            let net_bits = u128::from_be_bytes(net.octets());
            (ip_bits & mask) == (net_bits & mask)
        }
        _ => false, // IPv4 vs IPv6 不匹配
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
    let rt = get_runtime_config();
    let admin_token = rt.get_or(keys::API_ADMIN_TOKEN, "");

    // 验证密码（启动时已自动迁移明文为哈希）
    let password_valid = match verify_password(&login_body.password, &admin_token) {
        Ok(valid) => valid,
        Err(e) => {
            error!("Admin API: password verification error: {}", e);
            return Ok(error_from_shortlinker(&ShortlinkerError::internal_error(
                "Authentication error",
            )));
        }
    };

    if !password_valid {
        error!("Admin API: login failed - invalid token");
        return Ok(error_from_shortlinker(
            &ShortlinkerError::auth_password_invalid("Invalid admin token"),
        ));
    }

    info!("Admin API: login successful");

    // Generate JWT tokens
    let jwt_service = JwtService::from_config();
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
pub async fn refresh_token(req: HttpRequest) -> ActixResult<impl Responder> {
    let cookie_builder = CookieBuilder::from_config();

    // Get refresh token from cookie
    let refresh_token = match req.cookie(cookie_builder.refresh_cookie_name()) {
        Some(cookie) => cookie.value().to_string(),
        None => {
            warn!("Admin API: refresh token not found in cookie");
            return Ok(error_from_shortlinker(
                &ShortlinkerError::auth_token_invalid("Refresh token not found"),
            ));
        }
    };

    // Validate refresh token
    let jwt_service = JwtService::from_config();
    if let Err(e) = jwt_service.validate_refresh_token(&refresh_token) {
        warn!("Admin API: invalid refresh token: {}", e);
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
pub async fn verify_token(_req: HttpRequest) -> ActixResult<impl Responder> {
    Ok(success_response(MessageResponse {
        message: "Token is valid".to_string(),
    }))
}
