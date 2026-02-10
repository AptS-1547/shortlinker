//! 导入项验证逻辑
//!
//! 提供统一的 "raw string fields → ImportLinkItemRich" 转换和验证，
//! 消除 Admin API、IPC Handler、CSV Handler 三处重复。

use chrono::{DateTime, Utc};
use tracing::warn;

use crate::errors::ShortlinkerError;
use crate::services::ImportLinkItemRich;
use crate::utils::password::process_imported_password;
use crate::utils::url_validator::validate_url;

/// 原始导入项（string 日期，未处理的密码）
///
/// `CsvLinkRow`、`ImportLinkData` 等均可直接转为此类型。
#[derive(Debug, Clone)]
pub struct ImportLinkItemRaw {
    pub code: String,
    pub target: String,
    pub created_at: String,
    pub expires_at: Option<String>,
    pub password: Option<String>,
    pub click_count: usize,
    /// CSV 行号（1-based），仅 Admin API 设置，IPC/CSV 路径为 None
    pub row_num: Option<usize>,
}

/// 单行验证错误
#[derive(Debug, Clone)]
pub struct ImportRowError {
    pub code: String,
    pub error: ShortlinkerError,
    /// 来源行号，直接从 `ImportLinkItemRaw.row_num` 透传
    pub row_num: Option<usize>,
}

/// 验证并转换单个导入行
///
/// 验证顺序：
/// 1. code 非空
/// 2. URL 有效
/// 3. created_at 解析（失败 fallback 到 now）
/// 4. expires_at 解析（失败忽略）
/// 5. 密码处理（已哈希保留，明文哈希）
pub fn validate_import_row(raw: ImportLinkItemRaw) -> Result<ImportLinkItemRich, ImportRowError> {
    let row_num = raw.row_num;

    // 1. 空 code 检查
    if raw.code.is_empty() {
        return Err(ImportRowError {
            code: raw.code,
            error: ShortlinkerError::link_invalid_code("Empty code"),
            row_num,
        });
    }

    // 2. URL 验证
    if let Err(e) = validate_url(&raw.target) {
        return Err(ImportRowError {
            code: raw.code,
            error: ShortlinkerError::link_invalid_url(format!("Invalid URL: {}", e)),
            row_num,
        });
    }

    // 3. 解析 created_at
    let created_at = DateTime::parse_from_rfc3339(&raw.created_at)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| {
            warn!(
                "Import: invalid created_at '{}' for code '{}', using now",
                raw.created_at, raw.code
            );
            Utc::now()
        });

    // 4. 解析 expires_at
    let expires_at = raw.expires_at.as_ref().and_then(|s| {
        if s.is_empty() {
            None
        } else {
            DateTime::parse_from_rfc3339(s)
                .ok()
                .map(|dt| dt.with_timezone(&Utc))
        }
    });

    // 5. 密码处理
    let password = match process_imported_password(raw.password.as_deref()) {
        Ok(pwd) => pwd,
        Err(e) => {
            return Err(ImportRowError {
                code: raw.code,
                error: ShortlinkerError::link_password_hash_error(format!(
                    "Password hash error: {}",
                    e
                )),
                row_num,
            });
        }
    };

    Ok(ImportLinkItemRich {
        code: raw.code,
        target: raw.target,
        created_at,
        expires_at,
        password,
        click_count: raw.click_count,
        row_num,
    })
}

/// 批量验证导入行，返回 (成功项, 失败项)
pub fn validate_import_rows(
    rows: Vec<ImportLinkItemRaw>,
) -> (Vec<ImportLinkItemRich>, Vec<ImportRowError>) {
    let mut valid = Vec::with_capacity(rows.len());
    let mut errors = Vec::new();

    for raw in rows {
        match validate_import_row(raw) {
            Ok(item) => valid.push(item),
            Err(e) => errors.push(e),
        }
    }

    (valid, errors)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_raw(code: &str, target: &str) -> ImportLinkItemRaw {
        ImportLinkItemRaw {
            code: code.to_string(),
            target: target.to_string(),
            created_at: "2025-01-01T00:00:00Z".to_string(),
            expires_at: None,
            password: None,
            click_count: 0,
            row_num: None,
        }
    }

    #[test]
    fn test_valid_row() {
        let raw = make_raw("test", "https://example.com");
        let result = validate_import_row(raw);
        assert!(result.is_ok());
        let rich = result.unwrap();
        assert_eq!(rich.code, "test");
        assert_eq!(rich.target, "https://example.com");
    }

    #[test]
    fn test_empty_code() {
        let raw = make_raw("", "https://example.com");
        let result = validate_import_row(raw);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.error.code(), "E024"); // LinkInvalidCode
    }

    #[test]
    fn test_invalid_url() {
        let raw = make_raw("test", "javascript:alert(1)");
        let result = validate_import_row(raw);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.error.code(), "E020"); // LinkInvalidUrl
    }

    #[test]
    fn test_invalid_created_at_fallback() {
        let mut raw = make_raw("test", "https://example.com");
        raw.created_at = "not-a-date".to_string();
        let result = validate_import_row(raw);
        assert!(result.is_ok()); // fallback to now
    }

    #[test]
    fn test_batch_validation() {
        let rows = vec![
            make_raw("good", "https://example.com"),
            make_raw("", "https://example.com"),
            make_raw("bad-url", "not-a-url"),
            make_raw("also-good", "https://test.com"),
        ];
        let (valid, errors) = validate_import_rows(rows);
        assert_eq!(valid.len(), 2);
        assert_eq!(errors.len(), 2);
    }

    // ---- row_num propagation tests ----

    #[test]
    fn test_row_num_propagated_to_valid_item() {
        let mut raw = make_raw("test", "https://example.com");
        raw.row_num = Some(5);
        let rich = validate_import_row(raw).unwrap();
        assert_eq!(rich.row_num, Some(5));
    }

    #[test]
    fn test_row_num_propagated_to_error() {
        let mut raw = make_raw("test", "javascript:alert(1)");
        raw.row_num = Some(10);
        let err = validate_import_row(raw).unwrap_err();
        assert_eq!(err.row_num, Some(10));
        assert_eq!(err.error.code(), "E020");
    }

    #[test]
    fn test_row_num_none_when_unset() {
        let raw = make_raw("test", "https://example.com");
        let rich = validate_import_row(raw).unwrap();
        assert_eq!(rich.row_num, None);
    }

    #[test]
    fn test_batch_row_num_with_duplicates() {
        // 模拟 CSV 重复 code 场景：同 code 不同行号
        let mut row5 = make_raw("dup", "https://valid.com");
        row5.row_num = Some(5);
        let mut row10 = make_raw("dup", "not-a-url");
        row10.row_num = Some(10);

        let (valid, errors) = validate_import_rows(vec![row5, row10]);
        assert_eq!(valid.len(), 1);
        assert_eq!(errors.len(), 1);

        // 成功项保留行号 5
        assert_eq!(valid[0].row_num, Some(5));
        // 失败项保留行号 10（不会错指到行号 5）
        assert_eq!(errors[0].row_num, Some(10));
        assert_eq!(errors[0].code, "dup");
    }

    #[test]
    fn test_empty_code_error_carries_row_num() {
        let mut raw = make_raw("", "https://example.com");
        raw.row_num = Some(3);
        let err = validate_import_row(raw).unwrap_err();
        assert_eq!(err.row_num, Some(3));
        assert_eq!(err.error.code(), "E024");
    }
}
