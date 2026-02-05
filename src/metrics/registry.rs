//! Global metrics registry
//!
//! Defines all Prometheus metrics used in the application.

use once_cell::sync::Lazy;
use prometheus::{CounterVec, Encoder, Gauge, Opts, Registry, TextEncoder};

/// Global metrics instance
pub static METRICS: Lazy<Metrics> = Lazy::new(Metrics::new);

/// Application metrics container
pub struct Metrics {
    /// Internal Prometheus registry
    registry: Registry,

    // ===== Redirect metrics =====
    /// Total number of redirects by status code
    pub redirects_total: CounterVec,

    // ===== Click buffer metrics =====
    /// Current size of the click buffer
    pub clicks_buffer_size: Gauge,

    // ===== Cache metrics =====
    /// Cache hits by layer (bloom_filter, negative_cache, object_cache)
    pub cache_hits_total: CounterVec,
    /// Cache misses by layer
    pub cache_misses_total: CounterVec,

    // ===== System metrics =====
    /// Server uptime in seconds
    pub uptime_seconds: Gauge,
}

impl Metrics {
    fn new() -> Self {
        let registry = Registry::new();

        // Redirect metrics
        let redirects_total = CounterVec::new(
            Opts::new(
                "shortlinker_redirects_total",
                "Total number of redirects by status",
            ),
            &["status"],
        )
        .expect("Failed to create redirects_total metric");

        // Click buffer metrics
        let clicks_buffer_size = Gauge::new(
            "shortlinker_clicks_buffer_size",
            "Current size of the click buffer",
        )
        .expect("Failed to create clicks_buffer_size metric");

        // Cache metrics
        let cache_hits_total = CounterVec::new(
            Opts::new(
                "shortlinker_cache_hits_total",
                "Total cache hits by layer",
            ),
            &["layer"],
        )
        .expect("Failed to create cache_hits_total metric");

        let cache_misses_total = CounterVec::new(
            Opts::new(
                "shortlinker_cache_misses_total",
                "Total cache misses by layer",
            ),
            &["layer"],
        )
        .expect("Failed to create cache_misses_total metric");

        // System metrics
        let uptime_seconds = Gauge::new(
            "shortlinker_uptime_seconds",
            "Server uptime in seconds",
        )
        .expect("Failed to create uptime_seconds metric");

        // Register all metrics
        registry
            .register(Box::new(redirects_total.clone()))
            .expect("Failed to register redirects_total");
        registry
            .register(Box::new(clicks_buffer_size.clone()))
            .expect("Failed to register clicks_buffer_size");
        registry
            .register(Box::new(cache_hits_total.clone()))
            .expect("Failed to register cache_hits_total");
        registry
            .register(Box::new(cache_misses_total.clone()))
            .expect("Failed to register cache_misses_total");
        registry
            .register(Box::new(uptime_seconds.clone()))
            .expect("Failed to register uptime_seconds");

        Self {
            registry,
            redirects_total,
            clicks_buffer_size,
            cache_hits_total,
            cache_misses_total,
            uptime_seconds,
        }
    }

    /// Export metrics in Prometheus text format
    pub fn export(&self) -> String {
        let encoder = TextEncoder::new();
        let metric_families = self.registry.gather();
        let mut buffer = Vec::new();
        encoder
            .encode(&metric_families, &mut buffer)
            .expect("Failed to encode metrics");
        String::from_utf8(buffer).expect("Metrics output is not valid UTF-8")
    }
}
