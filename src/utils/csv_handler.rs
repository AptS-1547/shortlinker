//! CSV 导入导出共享逻辑
//!
//! 提供统一的 CSV 读写功能，供 CLI、TUI 和 Web Admin 使用

use chrono::Utc;
use csv::{ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;

use crate::errors::ShortlinkerError;
use crate::services::{ImportLinkItemRaw, validate_import_row};
use crate::storage::ShortLink;

/// CSV 行数据结构（用于序列化/反序列化）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CsvLinkRow {
    pub code: String,
    pub target: String,
    pub created_at: String,
    #[serde(default)]
    pub expires_at: Option<String>,
    #[serde(default)]
    pub password: Option<String>,
    #[serde(default)]
    pub click_count: usize,
}

/// 点击日志 CSV 导出行（仅用于序列化）
#[derive(Debug, Clone, Serialize)]
pub struct ClickLogCsvRow {
    pub short_code: String,
    pub clicked_at: String,
    pub referrer: String,
    pub source: String,
    pub ip_address: String,
    pub country: String,
    pub city: String,
}

impl From<&ShortLink> for CsvLinkRow {
    fn from(link: &ShortLink) -> Self {
        Self {
            code: link.code.clone(),
            target: link.target.clone(),
            created_at: link.created_at.to_rfc3339(),
            expires_at: link.expires_at.map(|dt| dt.to_rfc3339()),
            password: link.password.clone(),
            click_count: link.click,
        }
    }
}

impl CsvLinkRow {
    /// 转换为 ShortLink，委托给共享验证逻辑
    pub fn into_short_link(self) -> Result<ShortLink, ShortlinkerError> {
        let raw = ImportLinkItemRaw {
            code: self.code,
            target: self.target,
            created_at: self.created_at,
            expires_at: self.expires_at,
            password: self.password,
            click_count: self.click_count,
        };
        let rich = validate_import_row(raw).map_err(|e| e.error)?;
        Ok(ShortLink {
            code: rich.code,
            target: rich.target,
            created_at: rich.created_at,
            expires_at: rich.expires_at,
            password: rich.password,
            click: rich.click_count,
        })
    }
}

/// 导出链接到 CSV 文件
pub fn export_to_csv<P: AsRef<Path>>(
    links: &[&ShortLink],
    path: P,
) -> Result<(), ShortlinkerError> {
    let file = File::create(path.as_ref())
        .map_err(|e| ShortlinkerError::file_operation(format!("Failed to create file: {}", e)))?;
    let writer = BufWriter::new(file);
    let mut csv_writer = WriterBuilder::new().from_writer(writer);

    for link in links {
        let row = CsvLinkRow::from(*link);
        csv_writer.serialize(&row).map_err(|e| {
            ShortlinkerError::serialization(format!("Failed to write CSV row: {}", e))
        })?;
    }

    csv_writer
        .flush()
        .map_err(|e| ShortlinkerError::file_operation(format!("Failed to flush CSV: {}", e)))?;

    Ok(())
}

/// 从 CSV 文件导入链接
pub fn import_from_csv<P: AsRef<Path>>(path: P) -> Result<Vec<ShortLink>, ShortlinkerError> {
    let file = File::open(path.as_ref())
        .map_err(|e| ShortlinkerError::file_operation(format!("Failed to open file: {}", e)))?;
    let reader = BufReader::new(file);
    let mut csv_reader = ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .trim(csv::Trim::All)
        .from_reader(reader);

    let mut links = Vec::new();
    let mut errors = Vec::new();

    for (row_idx, result) in csv_reader.deserialize::<CsvLinkRow>().enumerate() {
        let row_num = row_idx + 2; // CSV 行号（1-based，跳过 header）

        match result {
            Ok(row) => {
                if row.code.is_empty() {
                    errors.push(format!("Row {}: Empty code", row_num));
                    continue;
                }
                match row.into_short_link() {
                    Ok(link) => links.push(link),
                    Err(e) => errors.push(format!("Row {}: {}", row_num, e)),
                }
            }
            Err(e) => {
                errors.push(format!("Row {}: CSV parse error: {}", row_num, e));
            }
        }
    }

    if !errors.is_empty() && links.is_empty() {
        return Err(ShortlinkerError::serialization(format!(
            "Failed to import CSV:\n{}",
            errors.join("\n")
        )));
    }

    // 如果有部分错误但也有成功的，打印警告但继续
    if !errors.is_empty() {
        tracing::warn!("CSV import warnings:\n{}", errors.join("\n"));
    }

    Ok(links)
}

/// 生成默认导出文件名（带时间戳）
pub fn generate_export_filename() -> String {
    format!(
        "shortlinks_export_{}.csv",
        Utc::now().format("%Y%m%d_%H%M%S")
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_link_row_from_short_link() {
        let link = ShortLink {
            code: "test".to_string(),
            target: "https://example.com".to_string(),
            created_at: Utc::now(),
            expires_at: None,
            password: None,
            click: 42,
        };

        let row = CsvLinkRow::from(&link);
        assert_eq!(row.code, "test");
        assert_eq!(row.target, "https://example.com");
        assert_eq!(row.click_count, 42);
    }

    #[test]
    fn test_export_import_roundtrip() {
        let link = ShortLink {
            code: "roundtrip".to_string(),
            target: "https://example.com".to_string(),
            created_at: Utc::now(),
            expires_at: None,
            password: None,
            click: 10,
        };

        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();

        // Export
        export_to_csv(&[&link], path).unwrap();

        // Import
        let imported = import_from_csv(path).unwrap();
        assert_eq!(imported.len(), 1);
        assert_eq!(imported[0].code, "roundtrip");
        assert_eq!(imported[0].target, "https://example.com");
        assert_eq!(imported[0].click, 10);
    }

    #[test]
    fn test_import_invalid_url() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(
            temp_file,
            "code,target,created_at,expires_at,password,click_count"
        )
        .unwrap();
        writeln!(
            temp_file,
            "bad,javascript:alert(1),2025-01-01T00:00:00Z,,,0"
        )
        .unwrap();

        let result = import_from_csv(temp_file.path());
        assert!(result.is_err() || result.unwrap().is_empty());
    }

    #[test]
    fn test_generate_export_filename() {
        let filename = generate_export_filename();
        assert!(filename.starts_with("shortlinks_export_"));
        assert!(filename.ends_with(".csv"));
    }
}
