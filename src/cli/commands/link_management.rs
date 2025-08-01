use super::super::CliError;
use crate::storages::{ShortLink, Storage};
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
            let mut info_parts = vec![format!(
                "{} -> {}",
                short_code.cyan(),
                link.target.blue().underline()
            )];

            if let Some(expires_at) = link.expires_at {
                info_parts.push(
                    format!("(expires: {})", expires_at.format("%Y-%m-%d %H:%M:%S UTC"))
                        .dimmed()
                        .yellow()
                        .to_string(),
                );
            }

            if link.password.is_some() {
                info_parts.push("🔒".to_string());
            }

            if link.click > 0 {
                info_parts.push(
                    format!("(clicks: {})", link.click)
                        .dimmed()
                        .cyan()
                        .to_string(),
                );
            }

            println!("  {}", info_parts.join(" "));
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
    password: Option<String>,
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
                "{} Generated random code: {}",
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
        password,
        click: 0,
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

    println!(
        "{} Deleted short link: {}",
        "✓".bold().green(),
        short_code.cyan()
    );

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
    password: Option<String>,
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
        password: password.or(old_link.password), // 如果提供新密码则更新，否则保持原密码
        click: old_link.click,                    // 保持原有的点击计数
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
    } // 收集所有链接
    let links_vec: Vec<&ShortLink> = links.values().collect();

    let output_path = file_path.unwrap_or_else(|| {
        format!(
            "shortlinks_export_{}.json",
            chrono::Utc::now().format("%Y%m%d_%H%M%S")
        )
    });

    let file = File::create(&output_path).map_err(|e| {
        CliError::CommandError(format!(
            "Failed to create export file '{}': {}",
            output_path, e
        ))
    })?;
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, &links_vec)
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

    let file = File::open(&file_path).map_err(|e| {
        CliError::CommandError(format!("Failed to open import file '{}': {}", file_path, e))
    })?;
    let reader = BufReader::new(file);
    let imported_links: Vec<ShortLink> = serde_json::from_reader(reader)
        .map_err(|e| CliError::CommandError(format!("Failed to parse JSON file: {}", e)))?;

    if imported_links.is_empty() {
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

    for imported_link in imported_links {
        // 检查短码是否已存在
        if !force_overwrite && existing_links.contains_key(&imported_link.code) {
            println!(
                "{} Skipping existing code: {} (use --force to overwrite)",
                "⚠".bold().yellow(),
                imported_link.code.cyan()
            );
            skipped_count += 1;
            continue;
        }

        // 验证URL格式
        if !imported_link.target.starts_with("http://")
            && !imported_link.target.starts_with("https://")
        {
            println!(
                "{} Skipping code '{}': invalid URL - {}",
                "✗".bold().red(),
                imported_link.code.cyan(),
                imported_link.target
            );
            error_count += 1;
            continue;
        }

        // 直接使用导入的链接，因为它已经是完整的 ShortLink 结构
        match storage.set(imported_link.clone()).await {
            Ok(_) => {
                imported_count += 1;
                println!(
                    "{} Imported: {} -> {}",
                    "✓".bold().green(),
                    imported_link.code.cyan(),
                    imported_link.target.blue().underline()
                );
            }
            Err(e) => {
                println!(
                    "{} Failed to import '{}': {}",
                    "✗".bold().red(),
                    imported_link.code.cyan(),
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
