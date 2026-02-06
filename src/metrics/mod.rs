//! Prometheus metrics module
//!
//! Provides metrics collection and export for monitoring.
//!
//! # Feature
//! This module requires the `metrics` feature to be enabled.

mod registry;
mod system;
mod traits;

pub use registry::METRICS;
pub use system::spawn_system_metrics_updater;
pub use traits::{MetricsRecorder, NoopMetrics, PrometheusMetricsWrapper};
