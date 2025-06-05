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
        println!("{} No short links found", "ℹ".bold().blue());
    } else {
        println!("{}", "Short link list:".bold().green());
        println!();
        for (short_code, link) in &links {
            if let Some(expires_at) = link.expires_at {
                println!(
                    "  {} -> {} {}",
                    short_code.cyan(),
                    link.target.blue().underline(),
                    format!("(expires: {})", expires_at.format("%Y-%m-%d %H:%M:%S UTC"))
                        .dimmed()
                        .yellow()
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
            "{} Total {} short links",
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
            println!("{} Generated random code: {}", "ℹ".bold().blue(), code.magenta());
            code
        }
    };

    let links = storage.load_all().await;

    // 检查短码是否已存在
    if links.contains_key(&final_short_code) {
        if force_overwrite {
            println!(
                "{} Force overwriting code '{}': {} -> {}",
                "⚠".bold().yellow(),
                final_short_code.cyan(),
                links[&final_short_code].target.dimmed().underline(),
                target_url.blue()
            );
        } else {
            return Err(CliError::CommandError(format!(
                "Code '{}' already exists and points to {}. Use --force to overwrite",
                final_short_code, links[&final_short_code].target
            )));
        }
    }

    let expires_at = if let Some(expire) = expire_time {
        match TimeParser::parse_expire_time(&expire) {
            Ok(dt) => {
                println!(
                    "{} Expiration parsed as: {}",
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
            "{} Added short link: {} -> {} (expires: {})",
            "✓".bold().green(),
            final_short_code.cyan(),
            target_url.blue().underline(),
            expire.format("%Y-%m-%d %H:%M:%S UTC").to_string().yellow()
        );
    } else {
        println!(
            "{} Added short link: {} -> {}",
            "✓".bold().green(),
            final_short_code.cyan(),
            target_url.blue().underline()
        );
    }

    // Notify server to reload
    if let Err(e) = crate::system::notify_server() {
        println!("{} Failed to notify server: {}", "⚠".bold().yellow(), e);
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

    println!("{} Deleted short link: {}", "✓".bold().green(), short_code.cyan());

    // Notify server to reload
    if let Err(e) = crate::system::notify_server() {
        println!("{} Failed to notify server: {}", "⚠".bold().yellow(), e);
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
        "{} Short link updated from {} to {}",
        "✓".bold().green(),
        old_link.target.dimmed().underline(),
        target_url.blue().underline()
    );

    if let Some(expire) = expires_at {
        println!(
            "{} Expiration: {}",
            "ℹ".bold().blue(),
            expire.format("%Y-%m-%d %H:%M:%S UTC").to_string().yellow()
        );
    }

    // Notify server to reload
    if let Err(e) = crate::system::notify_server() {
        println!("{} Failed to notify server: {}", "⚠".bold().yellow(), e);
    }

    Ok(())
}

pub async fn export_links(
    storage: Arc<dyn Storage>,
    file_path: Option<String>,
) -> Result<(), CliError> {
    let links = storage.load_all().await;

    if links.is_empty() {
        println!("{} No short links to export", "ℹ".bold().blue());
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
        CliError::CommandError(format!("Failed to create export file '{}': {}", output_path, e))
    })?;

    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, &serializable_links)
        .map_err(|e| CliError::CommandError(format!("Failed to export JSON data: {}", e)))?;

    println!(
        "{} Exported {} short links to: {}",
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
    // Check if file exists
    if !Path::new(&file_path).exists() {
        return Err(CliError::CommandError(format!(
            "Import file not found: {}",
            file_path
        )));
    }

    let file = File::open(&file_path)
        .map_err(|e| CliError::CommandError(format!("Failed to open import file '{}': {}", file_path, e)))?;

    let reader = BufReader::new(file);
    let serializable_links: Vec<SerializableShortLink> = serde_json::from_reader(reader)
        .map_err(|e| CliError::CommandError(format!("Failed to parse JSON file: {}", e)))?;

    if serializable_links.is_empty() {
        println!("{} Import file is empty", "ℹ".bold().blue());
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
                "{} Skipping existing code: {} (use --force to overwrite)",
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
                    "{} Skipping code '{}': failed to parse created_at - {}",
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
                        "{} Skipping code '{}': failed to parse expiration - {}",
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
                "{} Skipping code '{}': invalid URL - {}",
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
                    "{} Imported: {} -> {}",
                    "✓".bold().green(),
                    serializable_link.short_code.cyan(),
                    serializable_link.target_url.blue().underline()
                );
            }
            Err(e) => {
                println!(
                    "{} Failed to import '{}': {}",
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
        "{} Success: {} , skipped {} , failed {}",
        "Import finished:".bold().green(),
        imported_count.to_string().green(),
        skipped_count.to_string().yellow(),
        error_count.to_string().red()
    );

    // Notify server to reload
    if imported_count > 0 {
        if let Err(e) = crate::system::notify_server() {
            println!("{} Failed to notify server: {}", "⚠".bold().yellow(), e);
        }
    }

    Ok(())
}
