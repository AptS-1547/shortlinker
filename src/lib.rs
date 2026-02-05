//! Shortlinker - A high-performance URL shortener service
//!
//! This library provides the core functionality for the Shortlinker service,
//! including caching, storage backends, HTTP services, and management interfaces.
//!
//! # Features
//! - **server**: HTTP server mode (default)
//! - **cli**: Command-line interface
//! - **tui**: Terminal user interface
//! - **metrics**: Prometheus metrics export
//! - **full**: All features enabled
//!
//! # Architecture
//! - `cache`: Multi-level caching (L1 + L2 + Bloom filter)
//! - `storage`: Storage backends and data access
//! - `analytics`: Click tracking and statistics
//! - `api`: HTTP services and middleware
//! - `interfaces`: User interfaces (CLI, TUI)
//! - `config`: Configuration management
//! - `runtime`: Application lifecycle and execution modes
//! - `system`: Platform abstraction and system utilities

pub mod analytics;
pub mod api;
pub mod cache;
pub mod cli;
pub mod config;
pub mod errors;
mod event;
pub mod interfaces;
#[cfg(feature = "metrics")]
pub mod metrics;
pub mod runtime;
pub mod services;
pub mod storage;
pub mod system;
pub mod utils;
