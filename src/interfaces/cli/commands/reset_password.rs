//! 重置密码 CLI 命令

use crate::config::runtime_config::keys;
use crate::storage::ConfigStore;
use crate::utils::password::hash_password;
use colored::Colorize;
use sea_orm::DatabaseConnection;
use std::io::{self, BufRead, Write};

/// 从不同来源获取密码
fn get_password(password: Option<String>, stdin: bool) -> Result<String, String> {
    if stdin {
        // 从 stdin 读取
        let stdin = io::stdin();
        let mut line = String::new();
        stdin
            .lock()
            .read_line(&mut line)
            .map_err(|e| format!("Failed to read from stdin: {}", e))?;
        Ok(line.trim().to_string())
    } else if let Some(pwd) = password {
        // 命令行参数
        Ok(pwd)
    } else {
        // 交互式输入
        prompt_password_with_confirm()
    }
}

/// 交互式输入密码（带确认）
fn prompt_password_with_confirm() -> Result<String, String> {
    // 检查是否为 TTY
    if !atty_check() {
        return Err(
            "No password provided. Use --password or --stdin flag, or run interactively."
                .to_string(),
        );
    }

    print!("Enter new password: ");
    io::stdout().flush().map_err(|e| e.to_string())?;

    let password =
        rpassword::read_password().map_err(|e| format!("Failed to read password: {}", e))?;

    print!("Confirm password: ");
    io::stdout().flush().map_err(|e| e.to_string())?;

    let confirm =
        rpassword::read_password().map_err(|e| format!("Failed to read password: {}", e))?;

    if password != confirm {
        return Err("Passwords do not match".to_string());
    }

    Ok(password)
}

/// 检查 stdin 是否为 TTY
fn atty_check() -> bool {
    #[cfg(unix)]
    {
        use nix::libc;
        unsafe { libc::isatty(libc::STDIN_FILENO) != 0 }
    }
    #[cfg(windows)]
    {
        use std::os::windows::io::AsRawHandle;
        use winapi::um::consoleapi::GetConsoleMode;
        let handle = io::stdin().as_raw_handle();
        let mut mode = 0;
        unsafe { GetConsoleMode(handle as *mut _, &mut mode) != 0 }
    }
    #[cfg(not(any(unix, windows)))]
    {
        true // 默认假设是 TTY
    }
}

/// 运行 reset-password 命令
pub async fn run_reset_password(db: DatabaseConnection, password: Option<String>, stdin: bool) {
    // 获取密码
    let new_password = match get_password(password, stdin) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("{} {}", "Error:".red().bold(), e);
            std::process::exit(1);
        }
    };

    // 验证密码长度
    if new_password.len() < 8 {
        eprintln!(
            "{} Password must be at least 8 characters long",
            "Error:".red().bold()
        );
        std::process::exit(1);
    }

    // 哈希新密码
    let hashed = match hash_password(&new_password) {
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
  shortlinker reset-password [OPTIONS]

{}
  --password <PASSWORD>  New password (not recommended, visible in shell history)
  --stdin                Read password from stdin (for scripting)

{}
  Reset the admin API password. The new password will be hashed
  with Argon2id and stored in the database.

  If no options provided, will prompt for password interactively (recommended).

{}
  # Interactive (recommended)
  shortlinker reset-password

  # From stdin (scripting)
  echo "my_password" | shortlinker reset-password --stdin

  # Command line (not recommended)
  shortlinker reset-password --password "my_password"
"#,
        "Reset Password Command".cyan().bold(),
        "USAGE:".yellow().bold(),
        "OPTIONS:".yellow().bold(),
        "DESCRIPTION:".yellow().bold(),
        "EXAMPLES:".yellow().bold()
    );
}
