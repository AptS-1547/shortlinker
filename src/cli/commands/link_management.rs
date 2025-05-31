use super::super::CliError;
use crate::storages::{ShortLink, Storage};
use crate::utils::generate_random_code;
use crate::utils::colors::*;
use std::env;
use std::sync::Arc;

pub async fn list_links(storage: Arc<dyn Storage>) -> Result<(), CliError> {
    let links = storage.load_all().await;
    
    if links.is_empty() {
        println!("{}{}ℹ{} 没有短链接", BOLD, BLUE, RESET);
    } else {
        println!("{}{}短链接列表:{}", BOLD, GREEN, RESET);
        println!();
        for (short_code, link) in &links {
            if let Some(expires_at) = link.expires_at {
                println!(
                    "  {}{}{} -> {}{}{} {}(过期: {}{}{}){}",
                    CYAN, short_code, RESET,
                    BLUE, link.target, RESET,
                    DIM, YELLOW, expires_at.format("%Y-%m-%d %H:%M:%S UTC"), DIM, RESET
                );
            } else {
                println!(
                    "  {}{}{} -> {}{}{}",
                    CYAN, short_code, RESET,
                    BLUE, link.target, RESET
                );
            }
        }
        println!();
        println!("{}{}ℹ{} 共 {}{}{} 个短链接", BOLD, BLUE, RESET, GREEN, links.len(), RESET);
    }
    Ok(())
}

pub async fn add_link(
    storage: Arc<dyn Storage>, 
    short_code: Option<String>, 
    target_url: String,
    force_overwrite: bool,
    expire_time: Option<String>
) -> Result<(), CliError> {
    // 验证 URL 格式
    if !target_url.starts_with("http://") && !target_url.starts_with("https://") {
        return Err(CliError::CommandError("URL 必须以 http:// 或 https:// 开头".to_string()));
    }

    let random_code_length: usize = env::var("RANDOM_CODE_LENGTH")
        .unwrap_or_else(|_| "6".to_string())
        .parse()
        .unwrap_or(6);

    let final_short_code = match short_code {
        Some(code) => code,
        None => {
            let code = generate_random_code(random_code_length);
            println!("{}{}ℹ{} 生成随机短码: {}{}{}", BOLD, BLUE, RESET, MAGENTA, code, RESET);
            code
        }
    };

    let links = storage.load_all().await;
    
    // 检查短码是否已存在
    if links.contains_key(&final_short_code) {
        if force_overwrite {
            println!(
                "{}{}⚠{} 强制覆盖短码 '{}{}{}': {}{}{} -> {}{}{}",
                BOLD, YELLOW, RESET,
                CYAN, final_short_code, RESET,
                DIM, links[&final_short_code].target, RESET,
                BLUE, target_url, RESET
            );
        } else {
            return Err(CliError::CommandError(format!(
                "短码 '{}' 已存在，当前指向: {}。如需覆盖，请使用 --force 参数",
                final_short_code, links[&final_short_code].target
            )));
        }
    }

    let expires_at = if let Some(expire) = expire_time {
        match chrono::DateTime::parse_from_rfc3339(&expire) {
            Ok(dt) => Some(dt.with_timezone(&chrono::Utc)),
            Err(_) => {
                return Err(CliError::CommandError(
                    "过期时间格式不正确，应为 RFC3339 格式，如 2023-10-01T12:00:00Z".to_string()
                ));
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

    storage.set(link).await
        .map_err(|e| CliError::CommandError(format!("保存失败: {}", e)))?;

    if let Some(expire) = expires_at {
        println!(
            "{}{}✓{} 已添加短链接: {}{}{} -> {}{}{} (过期时间: {}{}{})",
            BOLD, GREEN, RESET,
            CYAN, final_short_code, RESET,
            BLUE, target_url, RESET,
            YELLOW, expire.format("%Y-%m-%d %H:%M:%S UTC"), RESET
        );
    } else {
        println!(
            "{}{}✓{} 已添加短链接: {}{}{} -> {}{}{}",
            BOLD, GREEN, RESET,
            CYAN, final_short_code, RESET,
            BLUE, target_url, RESET
        );
    }

    // 通知服务器重载
    if let Err(e) = crate::system::notify_server() {
        println!("{}{}⚠{} 通知服务器失败: {}", BOLD, YELLOW, RESET, e);
    }

    Ok(())
}

pub async fn remove_link(storage: Arc<dyn Storage>, short_code: String) -> Result<(), CliError> {
    let links = storage.load_all().await;

    if !links.contains_key(&short_code) {
        return Err(CliError::CommandError(format!("短链接不存在: {}", short_code)));
    }

    storage.remove(&short_code).await
        .map_err(|e| CliError::CommandError(format!("删除失败: {}", e)))?;

    println!("{}{}✓{} 已删除短链接: {}{}{}", BOLD, GREEN, RESET, CYAN, short_code, RESET);

    // 通知服务器重载
    if let Err(e) = crate::system::notify_server() {
        println!("{}{}⚠{} 通知服务器失败: {}", BOLD, YELLOW, RESET, e);
    }

    Ok(())
}
