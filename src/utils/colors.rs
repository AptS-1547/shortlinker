//! ANSI 颜色码和格式化工具

// ANSI 颜色码
pub const RESET: &str = "\x1b[0m";
pub const RED: &str = "\x1b[31m";
pub const GREEN: &str = "\x1b[32m";
pub const YELLOW: &str = "\x1b[33m";
pub const BLUE: &str = "\x1b[34m";
pub const MAGENTA: &str = "\x1b[35m";
pub const CYAN: &str = "\x1b[36m";
pub const WHITE: &str = "\x1b[37m";

// 格式化
pub const BOLD: &str = "\x1b[1m";
pub const DIM: &str = "\x1b[2m";
pub const ITALIC: &str = "\x1b[3m";
pub const UNDERLINE: &str = "\x1b[4m";

// 便捷的宏定义
#[macro_export]
macro_rules! print_error {
    ($($arg:tt)*) => {
        println!("{}{}错误:{} {}", 
            $crate::utils::colors::BOLD, 
            $crate::utils::colors::RED, 
            $crate::utils::colors::RESET, 
            format!($($arg)*))
    };
}

#[macro_export]
macro_rules! print_success {
    ($($arg:tt)*) => {
        println!("{}{}✓{} {}", 
            $crate::utils::colors::BOLD, 
            $crate::utils::colors::GREEN, 
            $crate::utils::colors::RESET, 
            format!($($arg)*))
    };
}

#[macro_export]
macro_rules! print_info {
    ($($arg:tt)*) => {
        println!("{}{}ℹ{} {}", 
            $crate::utils::colors::BOLD, 
            $crate::utils::colors::BLUE, 
            $crate::utils::colors::RESET, 
            format!($($arg)*))
    };
}

#[macro_export]
macro_rules! print_warning {
    ($($arg:tt)*) => {
        println!("{}{}⚠{} {}", 
            $crate::utils::colors::BOLD, 
            $crate::utils::colors::YELLOW, 
            $crate::utils::colors::RESET, 
            format!($($arg)*))
    };
}

#[macro_export]
macro_rules! print_usage {
    ($($arg:tt)*) => {
        println!("{}{}{}", 
            $crate::utils::colors::CYAN, 
            format!($($arg)*), 
            $crate::utils::colors::RESET)
    };
}

// 辅助函数
pub fn colorize(text: &str, color: &str) -> String {
    format!("{}{}{}", color, text, RESET)
}

pub fn bold(text: &str) -> String {
    format!("{}{}{}", BOLD, text, RESET)
}

pub fn dim(text: &str) -> String {
    format!("{}{}{}", DIM, text, RESET)
}
