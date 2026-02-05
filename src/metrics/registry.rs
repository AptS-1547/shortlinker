//! Global metrics registry
//!
//! Defines all Prometheus metrics used in the application.

use once_cell::sync::Lazy;
use prometheus::{
    Counter, CounterVec, Encoder, Gauge, GaugeVec, HistogramOpts, HistogramVec, Opts, Registry,
    TextEncoder,
};

/// Global metrics instance
pub static METRICS: Lazy<Metrics> = Lazy::new(Metrics::new);

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
    fn new() -> Self {
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
        )
        .expect("Failed to create http_request_duration_seconds metric");

        let http_requests_total = CounterVec::new(
            Opts::new(
                "shortlinker_http_requests_total",
                "Total number of HTTP requests",
            ),
            &["method", "endpoint", "status"],
        )
        .expect("Failed to create http_requests_total metric");

        let http_active_connections = Gauge::new(
            "shortlinker_http_active_connections",
            "Number of active HTTP connections",
        )
        .expect("Failed to create http_active_connections metric");

        // ===== 数据库指标 =====
        let db_query_duration_seconds = HistogramVec::new(
            HistogramOpts::new(
                "shortlinker_db_query_duration_seconds",
                "Database query latency in seconds",
            )
            .buckets(vec![0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0, 5.0]),
            &["operation"],
        )
        .expect("Failed to create db_query_duration_seconds metric");

        let db_queries_total = CounterVec::new(
            Opts::new(
                "shortlinker_db_queries_total",
                "Total number of database queries",
            ),
            &["operation"],
        )
        .expect("Failed to create db_queries_total metric");

        // ===== 缓存指标 =====
        let cache_operation_duration_seconds = HistogramVec::new(
            HistogramOpts::new(
                "shortlinker_cache_operation_duration_seconds",
                "Cache operation latency in seconds",
            )
            .buckets(vec![0.0001, 0.0005, 0.001, 0.005, 0.01, 0.05, 0.1]),
            &["operation", "layer"],
        )
        .expect("Failed to create cache_operation_duration_seconds metric");

        let cache_entries = GaugeVec::new(
            Opts::new("shortlinker_cache_entries", "Number of entries in cache"),
            &["layer"],
        )
        .expect("Failed to create cache_entries metric");

        let cache_hits_total = CounterVec::new(
            Opts::new("shortlinker_cache_hits_total", "Total cache hits by layer"),
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

        // ===== 重定向指标 =====
        let redirects_total = CounterVec::new(
            Opts::new(
                "shortlinker_redirects_total",
                "Total number of redirects by status",
            ),
            &["status"],
        )
        .expect("Failed to create redirects_total metric");

        // ===== 点击统计指标 =====
        let clicks_buffer_entries = Gauge::new(
            "shortlinker_clicks_buffer_entries",
            "Number of unique keys in the click buffer",
        )
        .expect("Failed to create clicks_buffer_entries metric");

        let clicks_flush_total = CounterVec::new(
            Opts::new(
                "shortlinker_clicks_flush_total",
                "Total number of click buffer flushes",
            ),
            &["trigger", "status"],
        )
        .expect("Failed to create clicks_flush_total metric");

        // ===== 认证指标 =====
        let auth_failures_total = CounterVec::new(
            Opts::new(
                "shortlinker_auth_failures_total",
                "Total number of authentication failures",
            ),
            &["method"],
        )
        .expect("Failed to create auth_failures_total metric");

        // ===== Bloom Filter 指标 =====
        let bloom_filter_false_positives_total = Counter::new(
            "shortlinker_bloom_filter_false_positives_total",
            "Total number of Bloom filter false positives",
        )
        .expect("Failed to create bloom_filter_false_positives_total metric");

        // ===== 系统指标 =====
        let uptime_seconds = Gauge::new("shortlinker_uptime_seconds", "Server uptime in seconds")
            .expect("Failed to create uptime_seconds metric");

        let process_memory_bytes = GaugeVec::new(
            Opts::new(
                "shortlinker_process_memory_bytes",
                "Process memory usage in bytes",
            ),
            &["type"],
        )
        .expect("Failed to create process_memory_bytes metric");

        let process_cpu_seconds = Gauge::new(
            "shortlinker_process_cpu_seconds",
            "Total CPU time consumed by the process in seconds (user + system)",
        )
        .expect("Failed to create process_cpu_seconds metric");

        let build_info = GaugeVec::new(
            Opts::new(
                "shortlinker_build_info",
                "Build information about the running binary",
            ),
            &["version"],
        )
        .expect("Failed to create build_info metric");

        // Register all metrics
        macro_rules! register {
            ($metric:ident) => {
                registry
                    .register(Box::new($metric.clone()))
                    .expect(concat!("Failed to register ", stringify!($metric)));
            };
        }
        register!(http_request_duration_seconds);
        register!(http_requests_total);
        register!(http_active_connections);
        register!(db_query_duration_seconds);
        register!(db_queries_total);
        register!(cache_operation_duration_seconds);
        register!(cache_entries);
        register!(cache_hits_total);
        register!(cache_misses_total);
        register!(redirects_total);
        register!(clicks_buffer_entries);
        register!(clicks_flush_total);
        register!(auth_failures_total);
        register!(bloom_filter_false_positives_total);
        register!(uptime_seconds);
        register!(process_memory_bytes);
        register!(process_cpu_seconds);
        register!(build_info);

        // Initialize build info (value 1.0 is Prometheus convention for info metrics)
        build_info
            .with_label_values(&[env!("CARGO_PKG_VERSION")])
            .set(1.0);

        Self {
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
            auth_failures_total,
            bloom_filter_false_positives_total,
            uptime_seconds,
            process_memory_bytes,
            process_cpu_seconds,
            build_info,
        }
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
