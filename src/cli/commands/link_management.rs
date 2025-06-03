use super::super::CliError;
use crate::storages::{SerializableShortLink, ShortLink, Storage};
use crate::utils::generate_random_code;
use crate::utils::TimeParser;
use colored::*;
use std::env;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;
use std::sync::Arc;

pub async fn list_links(storage: Arc<dyn Storage>) -> Result<(), CliError> {
    let links = storage.load_all().await;

    if links.is_empty() {
        println!("{} 没有短链接", "ℹ".bold().blue());
    } else {
        println!("{}", "短链接列表:".bold().green());
        println!();
        for (short_code, link) in &links {
            if let Some(expires_at) = link.expires_at {
                println!(
                    "  {} -> {} {}",
                    short_code.cyan(),
                    link.target.blue().underline(),
                    format!("(过期: {})", expires_at.format("%Y-%m-%d %H:%M:%S UTC")).dimmed().yellow()
                );
            } else {
                println!(
                    "  {} -> {}",
                    short_code.cyan(),
                    link.target.blue().underline()
                );
            }
        }
        println!();
        println!(
            "{} 共 {} 个短链接",
            "ℹ".bold().blue(),
            links.len().to_string().green()
        );
    }
    Ok(())
}

pub async fn add_link(
    storage: Arc<dyn Storage>,
    short_code: Option<String>,
    target_url: String,
    force_overwrite: bool,
    expire_time: Option<String>,
) -> Result<(), CliError> {
    // 验证 URL 格式
    if !target_url.starts_with("http://") && !target_url.starts_with("https://") {
        return Err(CliError::CommandError(
            "URL 必须以 http:// 或 https:// 开头".to_string(),
        ));
    }

    let random_code_length: usize = env::var("RANDOM_CODE_LENGTH")
        .unwrap_or_else(|_| "6".to_string())
        .parse()
        .unwrap_or(6);

    let final_short_code = match short_code {
        Some(code) => code,
        None => {
            let code = generate_random_code(random_code_length);
            println!(
                "{} 生成随机短码: {}",
                "ℹ".bold().blue(),
                code.magenta()
            );
            code
        }
    };

    let links = storage.load_all().await;

    // 检查短码是否已存在
    if links.contains_key(&final_short_code) {
        if force_overwrite {
            println!(
                "{} 强制覆盖短码 '{}': {} -> {}",
                "⚠".bold().yellow(),
                final_short_code.cyan(),
                links[&final_short_code].target.dimmed().underline(),
                target_url.blue()
            );
        } else {
            return Err(CliError::CommandError(format!(
                "短码 '{}' 已存在，当前指向: {}。如需覆盖，请使用 --force 参数",
                final_short_code, links[&final_short_code].target
            )));
        }
    }

    let expires_at = if let Some(expire) = expire_time {
        match TimeParser::parse_expire_time(&expire) {
            Ok(dt) => {
                println!(
                    "{} 过期时间解析为: {}",
                    "ℹ".bold().blue(),
                    dt.format("%Y-%m-%d %H:%M:%S UTC").to_string().yellow()
                );
                Some(dt)
            }
            Err(e) => {
                return Err(CliError::CommandError(format!(
                    "过期时间格式错误: {}。支持的格式：\n  - RFC3339: 2023-10-01T12:00:00Z\n  - 相对时间: 1d, 2w, 1y, 1d2h30m",
                    e
                )));
            }
        }
    } else {
        None
    };

    let link = ShortLink {
        code: final_short_code.clone(),
        target: target_url.clone(),
        created_at: chrono::Utc::now(),
        expires_at,
    };

    storage
        .set(link)
        .await
        .map_err(|e| CliError::CommandError(format!("保存失败: {}", e)))?;

    if let Some(expire) = expires_at {
        println!(
            "{} 已添加短链接: {} -> {} (过期时间: {})",
            "✓".bold().green(),
            final_short_code.cyan(),
            target_url.blue().underline(),
            expire.format("%Y-%m-%d %H:%M:%S UTC").to_string().yellow()
        );
    } else {
        println!(
            "{} 已添加短链接: {} -> {}",
            "✓".bold().green(),
            final_short_code.cyan(),
            target_url.blue().underline()
        );
    }

    // 通知服务器重载
    if let Err(e) = crate::system::notify_server() {
        println!("{} 通知服务器失败: {}", "⚠".bold().yellow(), e);
    }

    Ok(())
}

pub async fn remove_link(storage: Arc<dyn Storage>, short_code: String) -> Result<(), CliError> {
    let links = storage.load_all().await;

    if !links.contains_key(&short_code) {
        return Err(CliError::CommandError(format!(
            "短链接不存在: {}",
            short_code
        )));
    }

    storage
        .remove(&short_code)
        .await
        .map_err(|e| CliError::CommandError(format!("删除失败: {}", e)))?;

    println!(
        "{} 已删除短链接: {}",
        "✓".bold().green(),
        short_code.cyan()
    );

    // 通知服务器重载
    if let Err(e) = crate::system::notify_server() {
        println!("{} 通知服务器失败: {}", "⚠".bold().yellow(), e);
    }

    Ok(())
}

pub async fn update_link(
    storage: Arc<dyn Storage>,
    short_code: String,
    target_url: String,
    expire_time: Option<String>,
) -> Result<(), CliError> {
    // 验证 URL 格式
    if !target_url.starts_with("http://") && !target_url.starts_with("https://") {
        return Err(CliError::CommandError(
            "URL 必须以 http:// 或 https:// 开头".to_string(),
        ));
    }

    // 检查短码是否存在
    let old_link = match storage.get(&short_code).await {
        Some(link) => link,
        None => {
            return Err(CliError::CommandError(format!(
                "短链接不存在: {}",
                short_code
            )));
        }
    };

    let expires_at = if let Some(expire) = expire_time {
        match TimeParser::parse_expire_time(&expire) {
            Ok(dt) => {
                println!(
                    "{} 过期时间解析为: {}",
                    "ℹ".bold().blue(),
                    dt.format("%Y-%m-%d %H:%M:%S UTC").to_string().yellow()
                );
                Some(dt)
            }
            Err(e) => {
                return Err(CliError::CommandError(format!(
                    "过期时间格式错误: {}。支持的格式：\n  - RFC3339: 2023-10-01T12:00:00Z\n  - 相对时间: 1d, 2w, 1y, 1d2h30m",
                    e
                )));
            }
        }
    } else {
        old_link.expires_at // 保持原有的过期时间
    };

    let updated_link = ShortLink {
        code: short_code.clone(),
        target: target_url.clone(),
        created_at: old_link.created_at, // 保持原有的创建时间
        expires_at,
    };

    storage
        .set(updated_link)
        .await
        .map_err(|e| CliError::CommandError(format!("更新失败: {}", e)))?;

    println!(
        "{} 短链接已从 {} 更新为 {}",
        "✓".bold().green(),
        old_link.target.dimmed().underline(),
        target_url.blue().underline()
    );

    if let Some(expire) = expires_at {
        println!(
            "{} 过期时间: {}",
            "ℹ".bold().blue(),
            expire.format("%Y-%m-%d %H:%M:%S UTC").to_string().yellow()
        );
    }

    // 通知服务器重载
    if let Err(e) = crate::system::notify_server() {
        println!("{} 通知服务器失败: {}", "⚠".bold().yellow(), e);
    }

    Ok(())
}

pub async fn export_links(
    storage: Arc<dyn Storage>,
    file_path: Option<String>,
) -> Result<(), CliError> {
    let links = storage.load_all().await;

    if links.is_empty() {
        println!("{} 没有短链接可导出", "ℹ".bold().blue());
        return Ok(());
    }

    // 转换为可序列化格式
    let serializable_links: Vec<SerializableShortLink> = links
        .values()
        .map(|link| SerializableShortLink {
            short_code: link.code.clone(),
            target_url: link.target.clone(),
            created_at: link.created_at.to_rfc3339(),
            expires_at: link.expires_at.map(|dt| dt.to_rfc3339()),
            click: 0, // 默认点击数为0
        })
        .collect();

    let output_path = file_path.unwrap_or_else(|| {
        format!(
            "shortlinks_export_{}.json",
            chrono::Utc::now().format("%Y%m%d_%H%M%S")
        )
    });

    let file = File::create(&output_path).map_err(|e| {
        CliError::CommandError(format!("无法创建导出文件 '{}': {}", output_path, e))
    })?;

    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, &serializable_links)
        .map_err(|e| CliError::CommandError(format!("导出JSON数据失败: {}", e)))?;

    println!(
        "{} 已导出 {} 个短链接到: {}",
        "✓".bold().green(),
        links.len().to_string().green(),
        output_path.cyan()
    );

    Ok(())
}

pub async fn import_links(
    storage: Arc<dyn Storage>,
    file_path: String,
    force_overwrite: bool,
) -> Result<(), CliError> {
    // 检查文件是否存在
    if !Path::new(&file_path).exists() {
        return Err(CliError::CommandError(format!(
            "导入文件不存在: {}",
            file_path
        )));
    }

    let file = File::open(&file_path)
        .map_err(|e| CliError::CommandError(format!("无法打开导入文件 '{}': {}", file_path, e)))?;

    let reader = BufReader::new(file);
    let serializable_links: Vec<SerializableShortLink> = serde_json::from_reader(reader)
        .map_err(|e| CliError::CommandError(format!("解析JSON文件失败: {}", e)))?;

    if serializable_links.is_empty() {
        println!("{} 导入文件为空", "ℹ".bold().blue());
        return Ok(());
    }

    let existing_links = if !force_overwrite {
        storage.load_all().await
    } else {
        std::collections::HashMap::new()
    };

    let mut imported_count = 0;
    let mut skipped_count = 0;
    let mut error_count = 0;

    for serializable_link in serializable_links {
        // 检查短码是否已存在
        if !force_overwrite && existing_links.contains_key(&serializable_link.short_code) {
            println!(
                "{} 跳过已存在的短码: {} (使用 --force 强制覆盖)",
                "⚠".bold().yellow(),
                serializable_link.short_code.cyan()
            );
            skipped_count += 1;
            continue;
        }

        // 解析时间
        let created_at = match chrono::DateTime::parse_from_rfc3339(&serializable_link.created_at) {
            Ok(dt) => dt.with_timezone(&chrono::Utc),
            Err(e) => {
                println!(
                    "{} 跳过短码 '{}': 创建时间解析失败 - {}",
                    "✗".bold().red(),
                    serializable_link.short_code.cyan(),
                    e
                );
                error_count += 1;
                continue;
            }
        };

        let expires_at = if let Some(expire_str) = serializable_link.expires_at {
            match chrono::DateTime::parse_from_rfc3339(&expire_str) {
                Ok(dt) => Some(dt.with_timezone(&chrono::Utc)),
                Err(e) => {
                    println!(
                        "{} 跳过短码 '{}': 过期时间解析失败 - {}",
                        "✗".bold().red(),
                        serializable_link.short_code.cyan(),
                        e
                    );
                    error_count += 1;
                    continue;
                }
            }
        } else {
            None
        };

        // 验证URL格式
        if !serializable_link.target_url.starts_with("http://")
            && !serializable_link.target_url.starts_with("https://")
        {
            println!(
                "{} 跳过短码 '{}': URL格式无效 - {}",
                "✗".bold().red(),
                serializable_link.short_code.cyan(),
                serializable_link.target_url
            );
            error_count += 1;
            continue;
        }

        let link = ShortLink {
            code: serializable_link.short_code.clone(),
            target: serializable_link.target_url.clone(),
            created_at,
            expires_at,
        };

        match storage.set(link).await {
            Ok(_) => {
                imported_count += 1;
                println!(
                    "{} 已导入: {} -> {}",
                    "✓".bold().green(),
                    serializable_link.short_code.cyan(),
                    serializable_link.target_url.blue().underline()
                );
            }
            Err(e) => {
                println!(
                    "{} 导入失败 '{}': {}",
                    "✗".bold().red(),
                    serializable_link.short_code.cyan(),
                    e
                );
                error_count += 1;
            }
        }
    }

    println!();
    println!(
        "{} 成功 {} 个，跳过 {} 个，失败 {} 个",
        "导入完成:".bold().green(),
        imported_count.to_string().green(),
        skipped_count.to_string().yellow(),
        error_count.to_string().red()
    );

    // 通知服务器重载
    if imported_count > 0 {
        if let Err(e) = crate::system::notify_server() {
            println!("{} 通知服务器失败: {}", "⚠".bold().yellow(), e);
        }
    }

    Ok(())
}
