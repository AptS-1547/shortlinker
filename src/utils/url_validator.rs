//! URL 验证模块
//!
//! 验证 URL 安全性，阻止危险协议

use url::Url;

/// URL 验证错误
#[derive(Debug)]
pub enum UrlValidationError {
    EmptyUrl,
    InvalidProtocol(String),
    DangerousProtocol(String),
    InvalidFormat(String),
}

impl std::fmt::Display for UrlValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyUrl => write!(f, "URL cannot be empty"),
            Self::InvalidProtocol(proto) => write!(
                f,
                "Invalid protocol: {}. Only http:// and https:// are allowed",
                proto
            ),
            Self::DangerousProtocol(proto) => {
                write!(f, "Dangerous protocol blocked: {}", proto)
            }
            Self::InvalidFormat(msg) => write!(f, "Invalid URL format: {}", msg),
        }
    }
}

impl std::error::Error for UrlValidationError {}

/// 危险协议列表
const DANGEROUS_PROTOCOLS: &[&str] = &[
    "javascript:",
    "data:",
    "file:",
    "vbscript:",
    "about:",
    "blob:",
];

/// 验证 URL 安全性
///
/// 检查项目：
/// 1. URL 不为空
/// 2. 不是危险协议（javascript:, data:, file: 等）
/// 3. 必须是 http:// 或 https://
/// 4. URL 格式有效
pub fn validate_url(url: &str) -> Result<(), UrlValidationError> {
    let url = url.trim();

    if url.is_empty() {
        return Err(UrlValidationError::EmptyUrl);
    }

    let url_lower = url.to_lowercase();

    // 检查危险协议
    for proto in DANGEROUS_PROTOCOLS {
        if url_lower.starts_with(proto) {
            return Err(UrlValidationError::DangerousProtocol(proto.to_string()));
        }
    }

    // 检查协议
    if !url_lower.starts_with("http://") && !url_lower.starts_with("https://") {
        let proto = url_lower
            .split(':')
            .next()
            .map(|s| format!("{}:", s))
            .unwrap_or_default();
        return Err(UrlValidationError::InvalidProtocol(proto));
    }

    // 解析 URL 验证格式
    Url::parse(url).map_err(|e| UrlValidationError::InvalidFormat(e.to_string()))?;

    Ok(())
}

/// 获取 URL 验证错误的用户友好消息
pub fn validation_error_message(error: &UrlValidationError) -> &'static str {
    match error {
        UrlValidationError::EmptyUrl => "URL cannot be empty",
        UrlValidationError::InvalidProtocol(_) => "URL must start with http:// or https://",
        UrlValidationError::DangerousProtocol(_) => "This URL protocol is not allowed",
        UrlValidationError::InvalidFormat(_) => "Invalid URL format",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_urls() {
        assert!(validate_url("http://example.com").is_ok());
        assert!(validate_url("https://example.com").is_ok());
        assert!(validate_url("https://example.com/path?query=1").is_ok());
        assert!(validate_url("http://localhost:8080").is_ok());
    }

    #[test]
    fn test_dangerous_protocols() {
        assert!(matches!(
            validate_url("javascript:alert(1)"),
            Err(UrlValidationError::DangerousProtocol(_))
        ));
        assert!(matches!(
            validate_url("data:text/html,<script>alert(1)</script>"),
            Err(UrlValidationError::DangerousProtocol(_))
        ));
        assert!(matches!(
            validate_url("file:///etc/passwd"),
            Err(UrlValidationError::DangerousProtocol(_))
        ));
        assert!(matches!(
            validate_url("vbscript:msgbox(1)"),
            Err(UrlValidationError::DangerousProtocol(_))
        ));
    }

    #[test]
    fn test_invalid_protocols() {
        assert!(matches!(
            validate_url("ftp://example.com"),
            Err(UrlValidationError::InvalidProtocol(_))
        ));
        assert!(matches!(
            validate_url("mailto:test@example.com"),
            Err(UrlValidationError::InvalidProtocol(_))
        ));
    }

    #[test]
    fn test_empty_url() {
        assert!(matches!(
            validate_url(""),
            Err(UrlValidationError::EmptyUrl)
        ));
        assert!(matches!(
            validate_url("   "),
            Err(UrlValidationError::EmptyUrl)
        ));
    }

    #[test]
    fn test_case_insensitive() {
        assert!(matches!(
            validate_url("JAVASCRIPT:alert(1)"),
            Err(UrlValidationError::DangerousProtocol(_))
        ));
        assert!(validate_url("HTTP://example.com").is_ok());
        assert!(validate_url("HTTPS://example.com").is_ok());
    }
}
