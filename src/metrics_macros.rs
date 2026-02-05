//! Metrics helper macros
//!
//! Reduces boilerplate for common metrics patterns across the codebase.

/// Observe duration of an operation on a HistogramVec.
///
/// Usage:
/// ```ignore
/// observe_duration!(METRICS.cache_operation_duration_seconds, &["get", "bloom_filter"], {
///     self.filter_plugin.check(key).await
/// })
/// ```
#[allow(unused_macros)]
macro_rules! observe_duration {
    ($histogram:expr, $labels:expr, $body:expr) => {{
        #[cfg(feature = "metrics")]
        let _metrics_start = std::time::Instant::now();

        let _result = $body;

        #[cfg(feature = "metrics")]
        $histogram
            .with_label_values($labels)
            .observe(_metrics_start.elapsed().as_secs_f64());

        _result
    }};
}

/// Increment a CounterVec with given labels.
///
/// Usage:
/// ```ignore
/// inc_counter!(METRICS.cache_hits_total, &["bloom_filter"]);
/// ```
macro_rules! inc_counter {
    ($counter:expr, $labels:expr) => {
        #[cfg(feature = "metrics")]
        $counter.with_label_values($labels).inc();
    };
}

/// Increment a plain Counter (no labels).
///
/// Usage:
/// ```ignore
/// inc_plain_counter!(METRICS.bloom_filter_false_positives_total);
/// ```
macro_rules! inc_plain_counter {
    ($counter:expr) => {
        #[cfg(feature = "metrics")]
        $counter.inc();
    };
}

/// Set a GaugeVec with given labels to a value.
///
/// Usage:
/// ```ignore
/// set_gauge!(METRICS.cache_entries, &["object_cache"], count as f64);
/// ```
macro_rules! set_gauge {
    ($gauge:expr, $labels:expr, $value:expr) => {
        #[cfg(feature = "metrics")]
        $gauge.with_label_values($labels).set($value);
    };
}

/// Set a plain Gauge (no labels) to a value.
///
/// Usage:
/// ```ignore
/// set_plain_gauge!(METRICS.clicks_buffer_entries, count as f64);
/// ```
macro_rules! set_plain_gauge {
    ($gauge:expr, $value:expr) => {
        #[cfg(feature = "metrics")]
        $gauge.set($value);
    };
}
