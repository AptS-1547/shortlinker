//! Shortlinker - A high-performance URL shortener service
//!
//! This library provides the core functionality for the Shortlinker service,
//! including caching, storage backends, HTTP services, and management interfaces.
//!
//! # Features
//! - **server**: HTTP server mode (default)
//! - **cli**: Command-line interface
//! - **tui**: Terminal user interface
//! - **full**: All features enabled
//!
//! # Architecture
//! - `cache`: Multi-level caching (L1 + L2 + Bloom filter)
//! - `storages`: Storage backends (SQLite, PostgreSQL, MySQL, Sled)
//! - `services`: HTTP services (Admin, Health, Frontend, Redirect)
//! - `middleware`: Authentication and guards
//! - `system`: Platform abstraction, lifecycle management, and mode routing

pub mod cache;
#[cfg(feature = "cli")]
pub mod cli;
pub mod errors;
mod event;
pub mod middleware;
pub mod services;
pub mod repository;
pub mod system;
#[cfg(feature = "tui")]
pub mod tui;
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
