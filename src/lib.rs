//! shortlinker - 短链接服务
//!
//! 这是一个高性能的短链接服务，支持命令行管理和Web API。

pub mod cache;
pub mod cli;
pub mod errors;
mod event;
pub mod middleware;
pub mod services;
pub mod storages;
pub mod system;
pub mod utils;

#[cfg(test)]
mod tests {
    #![allow(unused_imports)]
    use super::*;

    #[test]
    fn test_modules_exist() {
        // 确保所有模块都能正确编译
        assert!(true);
    }
}
