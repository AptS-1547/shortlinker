//! Shortlinker domain metrics backed by AsterForge infrastructure metrics.

use std::sync::Arc;
#[cfg(feature = "metrics")]
use std::sync::OnceLock;

use aster_forge_metrics::SharedMetricsRecorder as SharedForgeMetricsRecorder;

/// Metrics owned by the short-link domain.
///
/// HTTP, database, process, and other product-neutral metrics are recorded by
/// the Forge recorder returned from [`MetricsRecorder::forge_recorder`].
#[expect(unused_variables, reason = "default no-op bodies ignore their labels")]
pub trait MetricsRecorder: Send + Sync {
    /// Returns the underlying Forge recorder shared with the rest of the
    /// process; product-neutral metrics go through it.
    fn forge_recorder(&self) -> SharedForgeMetricsRecorder {
        aster_forge_metrics::NoopMetrics::arc()
    }

    /// Counts a click event dropped because the analytics channel was full.
    ///
    /// `reason` labels why the entry was dropped (e.g. the channel kind).
    fn inc_clicks_channel_dropped(&self, reason: &str) {}

    /// Records the current number of entries waiting in the click buffer.
    fn set_clicks_buffer_entries(&self, count: f64) {}

    /// Counts a click-buffer flush attempt.
    ///
    /// `trigger` identifies what started the flush and `status` whether it
    /// succeeded or failed.
    fn inc_clicks_flush(&self, trigger: &str, status: &str) {}

    /// Counts a cache hit in the given layer (bloom, negative, object, ...).
    fn inc_cache_hit(&self, layer: &str) {}

    /// Counts a cache miss in the given layer.
    fn inc_cache_miss(&self, layer: &str) {}

    /// Observes the duration of one cache operation in seconds.
    fn observe_cache_operation(&self, operation: &str, layer: &str, duration_secs: f64) {}

    /// Records the current number of entries held by a cache layer.
    fn set_cache_entries(&self, layer: &str, count: f64) {}

    /// Counts a Bloom filter false positive that fell through to storage.
    fn inc_bloom_false_positive(&self) {}

    /// Counts a redirect response, labeled by HTTP status class.
    fn inc_redirect(&self, status: &str) {}

    /// Counts an authentication failure, labeled by the auth method that rejected it.
    fn inc_auth_failure(&self, method: &str) {}
}

/// Metrics implementation used by tests and builds without the `metrics` feature.
pub struct NoopMetrics;

impl MetricsRecorder for NoopMetrics {}

impl NoopMetrics {
    /// Creates a recorder that discards every metric.
    pub fn new() -> Self {
        Self
    }

    /// Wraps a [`NoopMetrics`] as the shared trait object.
    pub fn arc() -> Arc<dyn MetricsRecorder> {
        Arc::new(Self::new())
    }
}

impl Default for NoopMetrics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "metrics")]
mod product {
    use super::OnceLock;
    use aster_forge_metrics::MetricError;

    aster_forge_metrics::product_metrics! {
        pub struct ShortlinkerProductMetrics {
            clicks_buffer_entries: gauge(
                "shortlinker_clicks",
                "buffer_entries",
                "Current number of buffered click entries.",
                &[],
            ),
            clicks_flush_total: counter(
                "shortlinker_clicks",
                "flush_total",
                "Total click buffer flush attempts.",
                &["trigger", "status"],
            ),
            clicks_channel_dropped_total: counter(
                "shortlinker_clicks",
                "channel_dropped_total",
                "Total click events dropped before persistence.",
                &["reason"],
            ),
            cache_hits_total: counter(
                "shortlinker_cache",
                "hits_total",
                "Total cache hits by cache layer.",
                &["layer"],
            ),
            cache_misses_total: counter(
                "shortlinker_cache",
                "misses_total",
                "Total cache misses by cache layer.",
                &["layer"],
            ),
            cache_operation_duration_seconds: histogram_with_buckets(
                "shortlinker_cache",
                "operation_duration_seconds",
                "Cache operation duration in seconds.",
                &["operation", "layer"],
                &[0.0001, 0.0005, 0.001, 0.005, 0.01, 0.05, 0.1, 0.5],
            ),
            cache_entries: gauge(
                "shortlinker_cache",
                "entries",
                "Current number of cache entries by cache layer.",
                &["layer"],
            ),
            redirects_total: counter(
                "shortlinker_redirects",
                "total",
                "Total redirect responses by status.",
                &["status"],
            ),
            bloom_filter_false_positives_total: counter(
                "shortlinker_bloom_filter",
                "false_positives_total",
                "Total Bloom filter false positives.",
                &[],
            ),
            auth_failures_total: counter(
                "shortlinker_auth",
                "failures_total",
                "Total authentication failures by method.",
                &["method"],
            ),
            build_info: gauge(
                "shortlinker_build",
                "info",
                "Shortlinker build information.",
                &["version"],
            ),
        }
    }

    static PRODUCT_METRICS_WARNED: OnceLock<()> = OnceLock::new();

    /// Registers the product schema on `provider` and seeds zero-valued series
    /// so dashboards see every label combination from startup.
    pub fn register(
        provider: &aster_forge_metrics::MetricsProvider,
    ) -> Option<ShortlinkerProductMetrics> {
        match ShortlinkerProductMetrics::register(provider) {
            Ok(metrics) => {
                metrics.build_info.set(&[env!("CARGO_PKG_VERSION")], 1.0);
                for layer in ["bloom_filter", "negative_cache", "object_cache"] {
                    metrics.cache_hits_total.inc(&[layer], 0);
                    metrics.cache_misses_total.inc(&[layer], 0);
                    metrics.cache_entries.set(&[layer], 0.0);
                }
                for status in ["307", "404", "500"] {
                    metrics.redirects_total.inc(&[status], 0);
                }
                Some(metrics)
            }
            Err(error) => {
                warn_once(&error);
                None
            }
        }
    }

    fn warn_once(error: &MetricError) {
        PRODUCT_METRICS_WARNED.get_or_init(|| {
            tracing::warn!(
                error = %error,
                "failed to register Shortlinker product metrics"
            );
        });
    }
}

#[cfg(feature = "metrics")]
struct ShortlinkerMetricsRecorder {
    forge: SharedForgeMetricsRecorder,
    product: Option<product::ShortlinkerProductMetrics>,
}

#[cfg(feature = "metrics")]
impl MetricsRecorder for ShortlinkerMetricsRecorder {
    fn forge_recorder(&self) -> SharedForgeMetricsRecorder {
        self.forge.clone()
    }

    fn inc_clicks_channel_dropped(&self, reason: &str) {
        if let Some(ref product) = self.product {
            product.clicks_channel_dropped_total.inc(&[reason], 1);
        }
    }

    fn set_clicks_buffer_entries(&self, count: f64) {
        if let Some(ref product) = self.product {
            product.clicks_buffer_entries.set(&[], count);
        }
    }

    fn inc_clicks_flush(&self, trigger: &str, status: &str) {
        if let Some(ref product) = self.product {
            product.clicks_flush_total.inc(&[trigger, status], 1);
        }
    }

    fn inc_cache_hit(&self, layer: &str) {
        if let Some(ref product) = self.product {
            product.cache_hits_total.inc(&[layer], 1);
        }
    }

    fn inc_cache_miss(&self, layer: &str) {
        if let Some(ref product) = self.product {
            product.cache_misses_total.inc(&[layer], 1);
        }
    }

    fn observe_cache_operation(&self, operation: &str, layer: &str, duration_secs: f64) {
        if let Some(ref product) = self.product {
            product
                .cache_operation_duration_seconds
                .observe(&[operation, layer], duration_secs);
        }
    }

    fn set_cache_entries(&self, layer: &str, count: f64) {
        if let Some(ref product) = self.product {
            product.cache_entries.set(&[layer], count);
        }
    }

    fn inc_bloom_false_positive(&self) {
        if let Some(ref product) = self.product {
            product.bloom_filter_false_positives_total.inc(&[], 1);
        }
    }

    fn inc_redirect(&self, status: &str) {
        if let Some(ref product) = self.product {
            product.redirects_total.inc(&[status], 1);
        }
    }

    fn inc_auth_failure(&self, method: &str) {
        if let Some(ref product) = self.product {
            product.auth_failures_total.inc(&[method], 1);
        }
    }
}

/// Process-wide metrics runtime: the product recorder plus the Prometheus
/// provider that backs the `/metrics` endpoint.
pub struct MetricsRuntime {
    /// All metric writes go through this recorder.
    pub recorder: Arc<dyn MetricsRecorder>,
    /// Prometheus pull-export provider; `None` without the `metrics` feature
    /// or when the Forge pipeline failed to start (recorder degrades to no-op).
    pub provider: Option<aster_forge_metrics::MetricsProvider>,
}

/// Creates the metrics runtime selected by this build.
pub fn create_metrics_runtime() -> MetricsRuntime {
    #[cfg(feature = "metrics")]
    match build_forge_runtime() {
        Ok(runtime) => return runtime,
        Err(error) => {
            tracing::warn!(error = %error, "metrics disabled: Forge pipeline failed to start");
        }
    }

    MetricsRuntime {
        recorder: NoopMetrics::arc(),
        provider: None,
    }
}

#[cfg(feature = "metrics")]
fn build_forge_runtime() -> aster_forge_metrics::MetricResult<MetricsRuntime> {
    let provider = aster_forge_metrics::MetricsProvider::builder()
        .with_profile(
            "net.shortlinker",
            aster_forge_metrics::METRICS_PROFILE_VERSION,
        )?
        .with_prometheus()?
        .build()?;
    let product = product::register(&provider);
    let forge = aster_forge_metrics::InfrastructureMetricsRecorder::new(provider.clone())?;

    Ok(MetricsRuntime {
        recorder: Arc::new(ShortlinkerMetricsRecorder {
            forge: Arc::new(forge),
            product,
        }),
        provider: Some(provider),
    })
}
