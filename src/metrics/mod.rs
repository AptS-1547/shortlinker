//! Prometheus metrics module
//!
//! Provides metrics collection and export for monitoring.
//!
//! # Feature
//! This module requires the `metrics` feature to be enabled.

mod registry;
mod system;
mod traits;

pub use registry::{get_metrics, init_metrics};
pub use system::spawn_system_metrics_updater;
pub use traits::{MetricsRecorder, NoopMetrics, PrometheusMetricsWrapper};
