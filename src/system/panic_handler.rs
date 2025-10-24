//! Panic handler module
//!
//! Provides different panic handling strategies based on running mode:
//! - Server mode: Display detailed stack trace, log to crash.log
//! - CLI/TUI mode: Display simple message, log to crash.log

use std::panic;
use std::fs::OpenOptions;
use std::io::Write;
use chrono::Utc;

/// Running mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunMode {
    Server,
    Cli,
    Tui,
}

/// Install custom panic hook
pub fn install_panic_hook(mode: RunMode) {
    let _default_hook = panic::take_hook();

    panic::set_hook(Box::new(move |panic_info| {
        let payload = panic_info.payload();
        let message = if let Some(s) = payload.downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = payload.downcast_ref::<String>() {
            s.clone()
        } else {
            "Unknown panic".to_string()
        };

        let location = panic_info.location()
            .map(|loc| format!("{}:{}:{}", loc.file(), loc.line(), loc.column()))
            .unwrap_or_else(|| "Unknown location".to_string());

        let backtrace = std::backtrace::Backtrace::force_capture();
        let timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string();

        // Write to crash.log
        if let Err(e) = write_crash_log(&timestamp, &message, &location, &backtrace) {
            eprintln!("Failed to write crash log: {}", e);
        }

        match mode {
            RunMode::Server => {
                // Server mode: Display detailed stack trace
                #[cfg(feature = "server")]
                display_server_panic(&message, &location, &backtrace);

                #[cfg(not(feature = "server"))]
                display_simple_panic(&message);
            }
            RunMode::Cli | RunMode::Tui => {
                // CLI/TUI mode: Simple message
                display_simple_panic(&message);
            }
        }

        // Call default hook (optional)
        // default_hook(panic_info);
    }));
}

/// Server mode: Display detailed colored stack trace information
#[cfg(feature = "server")]
fn display_server_panic(message: &str, location: &str, backtrace: &std::backtrace::Backtrace) {
    use colored::Colorize;

    eprintln!();
    eprintln!("{}", "═══════════════════════════════════════════════════".red().bold());
    eprintln!("{}", "PANIC".red().bold());
    eprintln!("{}", "═══════════════════════════════════════════════════".red().bold());
    eprintln!();
    eprintln!("{} {}", "Reason:".yellow().bold(), message.white());
    eprintln!("{} {}", "Location:".yellow().bold(), location.white());
    eprintln!();
    eprintln!("{}", "Backtrace:".yellow().bold());
    eprintln!("{}", format!("{:?}", backtrace).dimmed());
    eprintln!();
    eprintln!("{}", "Details saved to crash.log".cyan());
    eprintln!("{}", "═══════════════════════════════════════════════════".red().bold());
    eprintln!();
}

/// CLI/TUI mode: Display simple error message
fn display_simple_panic(message: &str) {
    eprintln!();
    eprintln!("Program panicked: {}", message);
    eprintln!("Details saved to crash.log, please check the log file");
    eprintln!();
}

/// Write crash log
fn write_crash_log(
    timestamp: &str,
    message: &str,
    location: &str,
    backtrace: &std::backtrace::Backtrace,
) -> std::io::Result<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("crash.log")?;

    writeln!(file, "==========================================")?;
    writeln!(file, "Crash Report - {}", timestamp)?;
    writeln!(file, "==========================================")?;
    writeln!(file, "Message: {}", message)?;
    writeln!(file, "Location: {}", location)?;
    writeln!(file, "\nBacktrace:")?;
    writeln!(file, "{:?}", backtrace)?;
    writeln!(file, "==========================================\n")?;

    Ok(())
}
