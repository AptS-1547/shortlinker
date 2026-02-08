use colored::Colorize;
use std::env;

pub fn show_help() {
    let program_name = env::args()
        .next()
        .unwrap_or_else(|| "shortlinker".to_string());
    println!("{}", "shortlinker - URL shortening tool".bold().magenta());
    println!();
    println!("{}", "Usage:".bold());
    println!(
        "  {}                          # start server",
        program_name.cyan()
    );
    println!(
        "  {} tui                      # start TUI mode (requires 'tui' feature)",
        program_name.cyan()
    );
    println!(
        "  {} help                     # show help",
        program_name.cyan()
    );
    println!();
    println!("{}", "Link management:".bold());
    println!(
        "  {} add <code> <target URL> [options]   # add short link",
        program_name.cyan()
    );
    println!(
        "  {} add <target URL> [options]         # add with random code",
        program_name.cyan()
    );
    println!(
        "  {} update <code> <target URL> [options] # update existing link",
        program_name.cyan()
    );
    println!(
        "  {} remove <code>              # remove short link",
        program_name.cyan()
    );
    println!(
        "  {} list                      # list all short links",
        program_name.cyan()
    );
    println!(
        "  {} export [file path]           # export links as CSV",
        program_name.cyan()
    );
    println!(
        "  {} import <file path> [options]     # import links from CSV",
        program_name.cyan()
    );
    println!(
        "  {} generate-config [output path]   # generate sample config file",
        program_name.cyan()
    );
    println!();
    println!("{}", "Options:".bold());
    println!("  {}     force overwrite existing code", "--force".yellow());
    println!(
        "  {}    set expiration (RFC3339 or relative time)",
        "--expire".yellow()
    );
    println!(
        "  {}  set password protection for the link",
        "--password".yellow()
    );
}
