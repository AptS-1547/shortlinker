//! Prometheus metrics module
//!
//! Provides metrics collection and export for monitoring.
//!
//! # Feature
//! This module requires the `metrics` feature to be enabled.

mod registry;

pub use registry::METRICS;
