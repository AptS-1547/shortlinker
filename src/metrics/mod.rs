//! Prometheus metrics module
//!
//! Provides metrics collection and export for monitoring.
//!
//! # Feature
//! This module requires the `metrics` feature to be enabled.

mod registry;
mod system;

pub use registry::METRICS;
pub use system::update_system_metrics;
