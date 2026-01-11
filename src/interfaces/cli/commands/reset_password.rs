//! 重置密码 CLI 命令

use crate::config::runtime_config::keys;
use crate::storage::ConfigStore;
use crate::utils::password::hash_password;
use colored::Colorize;
use sea_orm::DatabaseConnection;

/// 运行 reset-password 命令
pub async fn run_reset_password(db: DatabaseConnection, new_password: &str) {
    // 验证密码长度
    if new_password.len() < 8 {
        eprintln!(
            "{} Password must be at least 8 characters long",
            "Error:".red().bold()
        );
        std::process::exit(1);
    }

    // 哈希新密码
    let hashed = match hash_password(new_password) {
        Ok(h) => h,
        Err(e) => {
            eprintln!("{} Failed to hash password: {}", "Error:".red().bold(), e);
            std::process::exit(1);
        }
    };

    // 更新数据库
    let config_store = ConfigStore::new(db);
    match config_store.set(keys::API_ADMIN_TOKEN, &hashed).await {
        Ok(_) => {
            println!("{} Admin password reset successfully", "✓".green().bold());
        }
        Err(e) => {
            eprintln!("{} Failed to update database: {}", "Error:".red().bold(), e);
            std::process::exit(1);
        }
    }
}

/// 显示 reset-password 命令帮助
pub fn show_reset_password_help() {
    println!(
        r#"{}

{}
  shortlinker reset-password <new_password>

{}
  Reset the admin API password. The new password will be hashed
  with Argon2id and stored in the database.

{}
  shortlinker reset-password "my_new_secure_password"
"#,
        "Reset Password Command".cyan().bold(),
        "USAGE:".yellow().bold(),
        "DESCRIPTION:".yellow().bold(),
        "EXAMPLE:".yellow().bold()
    );
}
