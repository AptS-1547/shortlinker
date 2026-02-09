//! IP 地址处理工具
//!
//! 提供统一的客户端 IP 提取功能，支持：
//! - 可信代理配置（trusted_proxies）
//! - CIDR 匹配
//! - 私有 IP 自动检测

use std::net::{IpAddr, SocketAddr};

use actix_web::HttpRequest;
use actix_web::dev::ConnectionInfo;
use tracing::debug;

#[cfg(unix)]
use crate::config::get_config;
use crate::config::{get_runtime_config, keys};

/// 检查 IP 是否为私有地址或 localhost
pub fn is_private_or_local(ip: &IpAddr) -> bool {
    match ip {
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
    }
}

/// 检查 IP 是否在可信代理列表中
pub fn is_trusted_proxy(ip: &str, trusted_proxies: &[String]) -> bool {
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

/// CIDR 检查
pub fn ip_in_cidr(ip: &IpAddr, cidr: &str) -> bool {
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

/// 从 ConnectionInfo 提取真实客户端 IP（核心逻辑）
///
/// 策略（按优先级）：
/// 1. Unix Socket 模式 → 强制使用 X-Forwarded-For
/// 2. 显式配置 trusted_proxies 且匹配 → 使用 X-Forwarded-For
/// 3. 未配置 trusted_proxies 且连接来自私有 IP → 自动检测代理，使用 X-Forwarded-For
/// 4. 默认 → 使用连接 IP（公网直连场景，防止伪造）
///
/// `get_forwarded_ip` 闭包用于从请求头获取转发的 IP（X-Forwarded-For 或 X-Real-IP）
pub fn extract_client_ip_from_conn_info<F>(
    conn_info: &ConnectionInfo,
    get_forwarded_ip: F,
) -> Option<String>
where
    F: FnOnce() -> Option<String>,
{
    let rt = get_runtime_config();

    // 步骤 1: 检查是否配置了 Unix Socket 模式
    #[cfg(unix)]
    {
        let config = get_config();
        if config.server.unix_socket.is_some() {
            // Unix Socket 模式：必须使用 X-Forwarded-For
            return get_forwarded_ip();
        }
    }

    // 步骤 2: 获取 peer_addr
    let peer_ip = conn_info.peer_addr()?;

    // 步骤 3: 检查是否显式配置了可信代理
    let trusted_proxies: Vec<String> = rt.get_json_or(keys::API_TRUSTED_PROXIES, Vec::new());
    if !trusted_proxies.is_empty() {
        if is_trusted_proxy(peer_ip, &trusted_proxies) {
            let real_ip = get_forwarded_ip().unwrap_or_else(|| peer_ip.to_string());
            debug!("Trusted proxy (explicit): {} -> {}", peer_ip, real_ip);
            return Some(real_ip);
        }
        // 显式配置了但不匹配 → 使用连接 IP（不信任 X-Forwarded-For）
        debug!(
            "Connection from {}, not in trusted_proxies, using peer IP",
            peer_ip
        );
        return Some(peer_ip.to_string());
    }

    // 步骤 4: 未配置 trusted_proxies → 智能检测
    if let Ok(ip_addr) = peer_ip.parse::<IpAddr>()
        && is_private_or_local(&ip_addr)
    {
        // 连接来自私有 IP/localhost → 假设有反向代理
        if let Some(real_ip) = get_forwarded_ip() {
            debug!(
                "Auto-detect proxy (private IP {}): using X-Forwarded-For: {}",
                peer_ip, real_ip
            );
            return Some(real_ip);
        }
        // 私有 IP 但无 X-Forwarded-For（可能是内网直连）
        debug!("Private IP {} without X-Forwarded-For", peer_ip);
    }

    // 步骤 5: 默认使用连接 IP
    Some(peer_ip.to_string())
}

/// 从 HttpRequest 提取真实客户端 IP
pub fn extract_client_ip(req: &HttpRequest) -> Option<String> {
    extract_client_ip_from_conn_info(&req.connection_info(), || extract_forwarded_ip(req))
}

/// 从请求头提取转发的 IP（X-Forwarded-For 或 X-Real-IP）
fn extract_forwarded_ip(req: &HttpRequest) -> Option<String> {
    extract_forwarded_ip_from_headers(req.headers())
}

/// 从 HeaderMap 提取转发的 IP
pub fn extract_forwarded_ip_from_headers(
    headers: &actix_web::http::header::HeaderMap,
) -> Option<String> {
    // 优先 X-Forwarded-For（取第一个，即原始客户端 IP）
    headers
        .get("x-forwarded-for")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.split(',').next())
        .map(|s| s.trim().to_string())
        .or_else(|| {
            // 其次 X-Real-IP
            headers
                .get("x-real-ip")
                .and_then(|h| h.to_str().ok())
                .map(String::from)
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_private_or_local_ipv4() {
        // 私有地址
        assert!(is_private_or_local(&"10.0.0.1".parse().unwrap()));
        assert!(is_private_or_local(&"172.16.0.1".parse().unwrap()));
        assert!(is_private_or_local(&"192.168.1.1".parse().unwrap()));
        // localhost
        assert!(is_private_or_local(&"127.0.0.1".parse().unwrap()));
        // 公网地址
        assert!(!is_private_or_local(&"8.8.8.8".parse().unwrap()));
        assert!(!is_private_or_local(&"1.1.1.1".parse().unwrap()));
    }

    #[test]
    fn test_is_private_or_local_ipv6() {
        // localhost
        assert!(is_private_or_local(&"::1".parse().unwrap()));
        // ULA (fc00::/7)
        assert!(is_private_or_local(&"fd00::1".parse().unwrap()));
        assert!(is_private_or_local(&"fc00::1".parse().unwrap()));
        // Link-local (fe80::/10)
        assert!(is_private_or_local(&"fe80::1".parse().unwrap()));
        // 公网地址
        assert!(!is_private_or_local(
            &"2001:4860:4860::8888".parse().unwrap()
        ));
    }

    #[test]
    fn test_ip_in_cidr_ipv4() {
        let ip: IpAddr = "192.168.1.100".parse().unwrap();
        assert!(ip_in_cidr(&ip, "192.168.1.0/24"));
        assert!(ip_in_cidr(&ip, "192.168.0.0/16"));
        assert!(!ip_in_cidr(&ip, "192.168.2.0/24"));
        assert!(!ip_in_cidr(&ip, "10.0.0.0/8"));
    }

    #[test]
    fn test_ip_in_cidr_ipv6() {
        let ip: IpAddr = "2001:db8::1".parse().unwrap();
        assert!(ip_in_cidr(&ip, "2001:db8::/32"));
        assert!(!ip_in_cidr(&ip, "2001:db9::/32"));
    }

    #[test]
    fn test_is_trusted_proxy() {
        let proxies = vec![
            "127.0.0.1".to_string(),
            "192.168.1.0/24".to_string(),
            "10.0.0.1".to_string(),
        ];

        assert!(is_trusted_proxy("127.0.0.1", &proxies));
        assert!(is_trusted_proxy("127.0.0.1:8080", &proxies));
        assert!(is_trusted_proxy("192.168.1.50", &proxies));
        assert!(is_trusted_proxy("10.0.0.1", &proxies));
        assert!(!is_trusted_proxy("8.8.8.8", &proxies));
        assert!(!is_trusted_proxy("192.168.2.1", &proxies));
    }
}
