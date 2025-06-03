use colored::*;
use std::env;

pub fn show_help() {
    let program_name = env::args()
        .next()
        .unwrap_or_else(|| "shortlinker".to_string());
    println!("{}", "shortlinker - 短链接管理工具".bold().magenta());
    println!();
    println!("{}", "用法:".bold());
    println!(
        "  {}                          # 启动服务器",
        program_name.cyan()
    );
    println!(
        "  {} start                    # 启动服务器",
        program_name.cyan()
    );
    println!(
        "  {} stop                     # 停止服务器",
        program_name.cyan()
    );
    println!(
        "  {} restart                  # 重启服务器",
        program_name.cyan()
    );
    println!(
        "  {} help                     # 显示帮助信息",
        program_name.cyan()
    );
    println!();
    println!("{}", "链接管理:".bold());
    println!(
        "  {} add <短码> <目标URL> [选项]   # 添加短链接",
        program_name.cyan()
    );
    println!(
        "  {} add <目标URL> [选项]         # 使用随机短码添加",
        program_name.cyan()
    );
    println!(
        "  {} update <短码> <目标URL> [选项] # 更新现有短链接",
        program_name.cyan()
    );
    println!(
        "  {} remove <短码>              # 删除短链接",
        program_name.cyan()
    );
    println!(
        "  {} list                      # 列出所有短链接",
        program_name.cyan()
    );
    println!(
        "  {} export [文件路径]           # 导出短链接为JSON",
        program_name.cyan()
    );
    println!(
        "  {} import <文件路径> [选项]     # 从JSON导入短链接",
        program_name.cyan()
    );
    println!();
    println!("{}", "选项:".bold());
    println!("  {}     强制覆盖已存在的短码", "--force".yellow());
    println!(
        "  {}    设置过期时间 (RFC3339格式或相对时间)",
        "--expire".yellow()
    );
}
