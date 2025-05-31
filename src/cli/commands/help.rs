use crate::utils::colors::*;
use std::env;

pub fn show_help() {
    let program_name = env::args().next().unwrap_or_else(|| "shortlinker".to_string());
    println!("{}{}shortlinker - 短链接管理工具{}", BOLD, MAGENTA, RESET);
    println!();
    println!("{}用法:{}", BOLD, RESET);
    println!("{}  {}                          # 启动服务器{}", CYAN, program_name, RESET);
    println!("{}  {} start                    # 启动服务器{}", CYAN, program_name, RESET);
    println!("{}  {} stop                     # 停止服务器{}", CYAN, program_name, RESET);
    println!("{}  {} restart                  # 重启服务器{}", CYAN, program_name, RESET);
    println!("{}  {} help                     # 显示帮助信息{}", CYAN, program_name, RESET);
    println!();
    println!("{}链接管理:{}", BOLD, RESET);
    println!("{}  {} add <短码> <目标URL> [选项]   # 添加短链接{}", CYAN, program_name, RESET);
    println!("{}  {} add <目标URL> [选项]         # 使用随机短码添加{}", CYAN, program_name, RESET);
    println!("{}  {} remove <短码>              # 删除短链接{}", CYAN, program_name, RESET);
    println!("{}  {} list                      # 列出所有短链接{}", CYAN, program_name, RESET);
    println!();
    println!("{}选项:{}", BOLD, RESET);
    println!("  {}--force{}     强制覆盖已存在的短码", YELLOW, RESET);
    println!("  {}--expire{}    设置过期时间 (RFC3339格式)", YELLOW, RESET);
}
