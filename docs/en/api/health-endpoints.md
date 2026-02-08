# Health Check API: Endpoints and Status Codes

This page documents `/health` endpoints in detail, including response schema and status code semantics. Monitoring integration guidance is in [Health Check API: Monitoring and Troubleshooting](/en/api/health-monitoring).

## Endpoints

**Base URL**: `http://your-domain:port/health`

> Note: `/health`, `/health/ready`, `/health/live` support `GET` and `HEAD`. `/health/metrics` is `GET` only (Prometheus scrape endpoint; requires the `metrics` feature).

### GET /health - Full health check

```bash
curl -sS -b cookies.txt \
  http://localhost:8080/health
```

**Example response**:
```json
{
  "code": 0,
  "message": "OK",
  "data": {
    "status": "healthy",
    "timestamp": "2025-06-01T12:00:00Z",
    "uptime": 3600,
    "checks": {
      "storage": {
        "status": "healthy",
        "links_count": 42,
        "backend": {
          "storage_type": "sqlite",
          "support_click": true
        }
      },
      "cache": {
        "status": "healthy",
        "cache_type": "memory",
        "bloom_filter_enabled": true,
        "negative_cache_enabled": true
      }
    },
    "response_time_ms": 15
  }
}
```

### GET /health/ready - Readiness

```bash
curl -sS -b cookies.txt \
  http://localhost:8080/health/ready
```

Returns HTTP 200 when ready (body: `OK`).

### GET /health/live - Liveness

```bash
curl -sS -b cookies.txt -I \
  http://localhost:8080/health/live
```

Returns HTTP 204 when alive.

### GET /health/metrics - Prometheus Metrics (Optional)

> This endpoint is registered only when the binary is built with the `metrics` feature. If not enabled, it will return `404`.

- Default path: `/health/metrics`
- Actual path: `{routes.health_prefix}/metrics` (controlled by runtime config `routes.health_prefix`)

Authentication is the same as other Health endpoints (Bearer Token / JWT cookies). For monitoring, Bearer Token is recommended:

```bash
HEALTH_TOKEN="your_health_token"

curl -sS \
  -H "Authorization: Bearer ${HEALTH_TOKEN}" \
  http://localhost:8080/health/metrics
```

Returns Prometheus text format (`text/plain; version=0.0.4; charset=utf-8`). On export failure, it returns `500` with a text error message.

**Exported metrics (current implementation)**:

| Metric | Type | Labels | Description |
|------|------|--------|-------------|
| `shortlinker_http_request_duration_seconds` | HistogramVec | `method`,`endpoint`,`status` | HTTP request latency (seconds) |
| `shortlinker_http_requests_total` | CounterVec | `method`,`endpoint`,`status` | Total HTTP requests |
| `shortlinker_http_active_connections` | Gauge | - | In-flight requests (approx. concurrency) |
| `shortlinker_db_query_duration_seconds` | HistogramVec | `operation` | DB query latency (seconds) |
| `shortlinker_db_queries_total` | CounterVec | `operation` | Total DB queries |
| `shortlinker_cache_operation_duration_seconds` | HistogramVec | `operation`,`layer` | Cache op latency (seconds) |
| `shortlinker_cache_entries` | GaugeVec | `layer` | Cache entries (currently updated for `object_cache` only) |
| `shortlinker_cache_hits_total` | CounterVec | `layer` | Cache hits by layer |
| `shortlinker_cache_misses_total` | CounterVec | `layer` | Cache misses by layer (currently `object_cache` only) |
| `shortlinker_redirects_total` | CounterVec | `status` | Redirects by status code (e.g. `307`/`404`) |
| `shortlinker_clicks_buffer_entries` | Gauge | - | Click buffer entries (unique short codes, not total clicks) |
| `shortlinker_clicks_flush_total` | CounterVec | `trigger`,`status` | Click buffer flushes by trigger and result |
| `shortlinker_clicks_channel_dropped` | CounterVec | `reason` | Dropped detailed-click events when channel is full/disconnected (`reason`: `full` / `disconnected`) |
| `shortlinker_auth_failures_total` | CounterVec | `method` | Auth failures (currently mainly from Admin API: `bearer`/`cookie`) |
| `shortlinker_bloom_filter_false_positives_total` | Counter | - | Bloom filter false positives |
| `shortlinker_uptime_seconds` | Gauge | - | Server uptime (seconds) |
| `shortlinker_process_memory_bytes` | GaugeVec | `type` | Process memory usage (bytes, `rss`/`virtual`) |
| `shortlinker_process_cpu_seconds` | Gauge | - | Total process CPU time (seconds, user+system) |
| `shortlinker_build_info` | GaugeVec | `version` | Build info (convention: value is always `1`, version carried in label) |

Label notes (common values):

- `endpoint`: `admin` / `health` / `frontend` / `redirect` (path-prefix classification to avoid label cardinality explosion; prefixes come from `routes.admin_prefix` / `routes.health_prefix` / `routes.frontend_prefix` and require restart to apply)
- `layer`: `bloom_filter` / `negative_cache` / `object_cache`
- `operation` (DB): `get` / `load_all` / `load_all_codes` / `count` / `paginated_query` / `batch_get` / `get_stats`
- `trigger` (click flush): `interval` / `threshold` / `manual`; `status`: `success` / `failed`

## Status codes

| Status | Meaning |
|--------|---------|
| 200 | Healthy/Ready |
| 204 | Alive (no content) |
| 401 | Unauthorized (missing/invalid auth) |
| 404 | Disabled (both `api.admin_token` and `api.health_token` are empty) |
| 503 | Unhealthy |

> Unauthorized body example (HTTP 401): `{"code":1001,"message":"Unauthorized: Invalid or missing token"}`
