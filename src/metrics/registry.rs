//! Global metrics registry
//!
//! Defines all Prometheus metrics used in the application.

use prometheus::{
    Counter, CounterVec, Encoder, Gauge, GaugeVec, HistogramOpts, HistogramVec, Opts, Registry,
    TextEncoder,
};
use std::sync::OnceLock;

use super::traits::MetricsRecorder;

/// Global metrics instance (initialized explicitly via `init_metrics()`)
static METRICS: OnceLock<Metrics> = OnceLock::new();

/// Initialize the global metrics registry.
///
/// Returns `Ok(())` if metrics were successfully created and registered,
/// or if they were already initialized. Returns `Err` on Prometheus errors.
pub fn init_metrics() -> Result<(), prometheus::Error> {
    if METRICS.get().is_some() {
        return Ok(());
    }
    let metrics = Metrics::try_new()?;
    // Another thread may have initialized between our check and here; that's fine.
    let _ = METRICS.set(metrics);
    Ok(())
}

/// Get a reference to the global metrics, if initialized.
pub fn get_metrics() -> Option<&'static Metrics> {
    METRICS.get()
}

/// Application metrics container
pub struct Metrics {
    /// Internal Prometheus registry
    registry: Registry,

    // ===== HTTP 指标 (P0/P1) =====
    /// HTTP 请求延迟直方图
    pub http_request_duration_seconds: HistogramVec,
    /// HTTP 请求总数
    pub http_requests_total: CounterVec,
    /// 当前活跃连接数
    pub http_active_connections: Gauge,

    // ===== 数据库指标 (P0/P1) =====
    /// 数据库查询延迟直方图
    pub db_query_duration_seconds: HistogramVec,
    /// 数据库查询总数
    pub db_queries_total: CounterVec,

    // ===== 缓存指标 (P0/P1) =====
    /// 缓存操作延迟直方图
    pub cache_operation_duration_seconds: HistogramVec,
    /// 缓存条目数
    pub cache_entries: GaugeVec,
    /// Cache hits by layer (bloom_filter, negative_cache, object_cache)
    pub cache_hits_total: CounterVec,
    /// Cache misses by layer
    pub cache_misses_total: CounterVec,

    // ===== 重定向指标 =====
    /// Total number of redirects by status code
    pub redirects_total: CounterVec,

    // ===== 点击统计指标 (P1) =====
    /// Number of unique keys in the click buffer
    pub clicks_buffer_entries: Gauge,
    /// 点击刷盘次数
    pub clicks_flush_total: CounterVec,
    /// 详细点击事件丢弃次数（channel 满或断开）
    pub clicks_channel_dropped: CounterVec,

    // ===== 认证指标 (P2) =====
    /// 认证失败次数
    pub auth_failures_total: CounterVec,

    // ===== Bloom Filter 指标 (P2) =====
    /// Bloom Filter 误报次数
    pub bloom_filter_false_positives_total: Counter,

    // ===== 系统指标 (P2) =====
    /// Server uptime in seconds
    pub uptime_seconds: Gauge,
    /// 进程内存使用
    pub process_memory_bytes: GaugeVec,
    /// 进程 CPU 时间 (用户态 + 内核态)
    pub process_cpu_seconds: Gauge,
    /// Build information (version label, value always 1.0)
    pub build_info: GaugeVec,
}

impl Metrics {
    fn try_new() -> Result<Self, prometheus::Error> {
        let registry = Registry::new();

        // ===== HTTP 指标 =====
        let http_request_duration_seconds = HistogramVec::new(
            HistogramOpts::new(
                "shortlinker_http_request_duration_seconds",
                "HTTP request latency in seconds",
            )
            .buckets(vec![
                0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0,
            ]),
            &["method", "endpoint", "status"],
        )?;

        let http_requests_total = CounterVec::new(
            Opts::new(
                "shortlinker_http_requests_total",
                "Total number of HTTP requests",
            ),
            &["method", "endpoint", "status"],
        )?;

        let http_active_connections = Gauge::new(
            "shortlinker_http_active_connections",
            "Number of active HTTP connections",
        )?;

        // ===== 数据库指标 =====
        let db_query_duration_seconds = HistogramVec::new(
            HistogramOpts::new(
                "shortlinker_db_query_duration_seconds",
                "Database query latency in seconds",
            )
            .buckets(vec![0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0, 5.0]),
            &["operation"],
        )?;

        let db_queries_total = CounterVec::new(
            Opts::new(
                "shortlinker_db_queries_total",
                "Total number of database queries",
            ),
            &["operation"],
        )?;

        // ===== 缓存指标 =====
        let cache_operation_duration_seconds = HistogramVec::new(
            HistogramOpts::new(
                "shortlinker_cache_operation_duration_seconds",
                "Cache operation latency in seconds",
            )
            .buckets(vec![0.0001, 0.0005, 0.001, 0.005, 0.01, 0.05, 0.1]),
            &["operation", "layer"],
        )?;

        let cache_entries = GaugeVec::new(
            Opts::new("shortlinker_cache_entries", "Number of entries in cache"),
            &["layer"],
        )?;

        let cache_hits_total = CounterVec::new(
            Opts::new("shortlinker_cache_hits_total", "Total cache hits by layer"),
            &["layer"],
        )?;

        let cache_misses_total = CounterVec::new(
            Opts::new(
                "shortlinker_cache_misses_total",
                "Total cache misses by layer",
            ),
            &["layer"],
        )?;

        // ===== 重定向指标 =====
        let redirects_total = CounterVec::new(
            Opts::new(
                "shortlinker_redirects_total",
                "Total number of redirects by status",
            ),
            &["status"],
        )?;

        // ===== 点击统计指标 =====
        let clicks_buffer_entries = Gauge::new(
            "shortlinker_clicks_buffer_entries",
            "Number of unique keys in the click buffer",
        )?;

        let clicks_flush_total = CounterVec::new(
            Opts::new(
                "shortlinker_clicks_flush_total",
                "Total number of click buffer flushes",
            ),
            &["trigger", "status"],
        )?;

        let clicks_channel_dropped = CounterVec::new(
            Opts::new(
                "shortlinker_clicks_channel_dropped",
                "Total number of detailed click events dropped due to channel full or disconnected",
            ),
            &["reason"],
        )?;

        // ===== 认证指标 =====
        let auth_failures_total = CounterVec::new(
            Opts::new(
                "shortlinker_auth_failures_total",
                "Total number of authentication failures",
            ),
            &["method"],
        )?;

        // ===== Bloom Filter 指标 =====
        let bloom_filter_false_positives_total = Counter::new(
            "shortlinker_bloom_filter_false_positives_total",
            "Total number of Bloom filter false positives",
        )?;

        // ===== 系统指标 =====
        let uptime_seconds = Gauge::new("shortlinker_uptime_seconds", "Server uptime in seconds")?;

        let process_memory_bytes = GaugeVec::new(
            Opts::new(
                "shortlinker_process_memory_bytes",
                "Process memory usage in bytes",
            ),
            &["type"],
        )?;

        let process_cpu_seconds = Gauge::new(
            "shortlinker_process_cpu_seconds",
            "Total CPU time consumed by the process in seconds (user + system)",
        )?;

        let build_info = GaugeVec::new(
            Opts::new(
                "shortlinker_build_info",
                "Build information about the running binary",
            ),
            &["version"],
        )?;

        // Register all metrics
        macro_rules! register {
            ($registry:expr, $metric:ident) => {
                $registry.register(Box::new($metric.clone()))?;
            };
        }
        register!(registry, http_request_duration_seconds);
        register!(registry, http_requests_total);
        register!(registry, http_active_connections);
        register!(registry, db_query_duration_seconds);
        register!(registry, db_queries_total);
        register!(registry, cache_operation_duration_seconds);
        register!(registry, cache_entries);
        register!(registry, cache_hits_total);
        register!(registry, cache_misses_total);
        register!(registry, redirects_total);
        register!(registry, clicks_buffer_entries);
        register!(registry, clicks_flush_total);
        register!(registry, clicks_channel_dropped);
        register!(registry, auth_failures_total);
        register!(registry, bloom_filter_false_positives_total);
        register!(registry, uptime_seconds);
        register!(registry, process_memory_bytes);
        register!(registry, process_cpu_seconds);
        register!(registry, build_info);

        // Initialize build info (value 1.0 is Prometheus convention for info metrics)
        build_info
            .with_label_values(&[env!("CARGO_PKG_VERSION")])
            .set(1.0);

        Ok(Self {
            registry,
            http_request_duration_seconds,
            http_requests_total,
            http_active_connections,
            db_query_duration_seconds,
            db_queries_total,
            cache_operation_duration_seconds,
            cache_entries,
            cache_hits_total,
            cache_misses_total,
            redirects_total,
            clicks_buffer_entries,
            clicks_flush_total,
            clicks_channel_dropped,
            auth_failures_total,
            bloom_filter_false_positives_total,
            uptime_seconds,
            process_memory_bytes,
            process_cpu_seconds,
            build_info,
        })
    }

    /// Export metrics in Prometheus text format
    pub fn export(&self) -> Result<String, String> {
        let encoder = TextEncoder::new();
        let metric_families = self.registry.gather();
        let mut buffer = Vec::new();
        encoder
            .encode(&metric_families, &mut buffer)
            .map_err(|e| format!("Failed to encode metrics: {}", e))?;
        String::from_utf8(buffer).map_err(|e| format!("Metrics output is not valid UTF-8: {}", e))
    }
}

impl MetricsRecorder for Metrics {
    // ===== Analytics (ClickManager) =====

    fn inc_clicks_channel_dropped(&self, reason: &str) {
        self.clicks_channel_dropped
            .with_label_values(&[reason])
            .inc();
    }

    fn set_clicks_buffer_entries(&self, count: f64) {
        self.clicks_buffer_entries.set(count);
    }

    fn inc_clicks_flush(&self, trigger: &str, status: &str) {
        self.clicks_flush_total
            .with_label_values(&[trigger, status])
            .inc();
    }

    // ===== Cache =====

    fn inc_cache_hit(&self, layer: &str) {
        self.cache_hits_total.with_label_values(&[layer]).inc();
    }

    fn inc_cache_miss(&self, layer: &str) {
        self.cache_misses_total.with_label_values(&[layer]).inc();
    }

    fn observe_cache_operation(&self, operation: &str, layer: &str, duration_secs: f64) {
        self.cache_operation_duration_seconds
            .with_label_values(&[operation, layer])
            .observe(duration_secs);
    }

    fn set_cache_entries(&self, layer: &str, count: f64) {
        self.cache_entries.with_label_values(&[layer]).set(count);
    }

    // ===== Redirect =====

    fn inc_bloom_false_positive(&self) {
        self.bloom_filter_false_positives_total.inc();
    }

    fn inc_redirect(&self, status: &str) {
        self.redirects_total.with_label_values(&[status]).inc();
    }

    // ===== Auth =====

    fn inc_auth_failure(&self, method: &str) {
        self.auth_failures_total.with_label_values(&[method]).inc();
    }

    // ===== HTTP (timing middleware) =====

    fn inc_active_connections(&self) {
        self.http_active_connections.inc();
    }

    fn dec_active_connections(&self) {
        self.http_active_connections.dec();
    }

    fn observe_http_request(&self, method: &str, endpoint: &str, status: &str, duration_secs: f64) {
        self.http_request_duration_seconds
            .with_label_values(&[method, endpoint, status])
            .observe(duration_secs);
    }

    fn inc_http_request(&self, method: &str, endpoint: &str, status: &str) {
        self.http_requests_total
            .with_label_values(&[method, endpoint, status])
            .inc();
    }

    // ===== Database =====

    fn observe_db_query(&self, operation: &str, duration_secs: f64) {
        self.db_query_duration_seconds
            .with_label_values(&[operation])
            .observe(duration_secs);
    }

    fn inc_db_query(&self, operation: &str) {
        self.db_queries_total.with_label_values(&[operation]).inc();
    }
}
