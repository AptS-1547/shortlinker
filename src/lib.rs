//! Shortlinker - A high-performance URL shortener service
//!
//! This library provides the core functionality for the Shortlinker service,
//! including caching, storage backends, HTTP services, and management interfaces.
//!
//! # Features
//! - **server**: HTTP server mode (default)
//! - **cli**: Command-line interface
//! - **metrics**: Prometheus metrics export
//! - **full**: All features enabled
//!
//! # Architecture
//! - `services`: Link cache policy and product business logic
//! - `storage`: Storage backends and data access
//! - `analytics`: Click tracking and statistics
//! - `api`: HTTP services and middleware
//! - `interfaces`: Command-line interface
//! - `config`: Configuration management
//! - `runtime`: Application lifecycle and execution modes
//! - `system`: Platform abstraction and system utilities

pub mod analytics;
pub mod api;
pub mod cli;
pub mod client;
pub mod config;
pub mod errors;
mod event;
pub mod metrics;
pub mod runtime;
pub mod services;
pub mod storage;
pub mod system;
pub mod utils;
