# Health Check API

Shortlinker provides a health check API for monitoring service status and storage health.

## Overview

- Service health status
- Storage backend health checks
- Readiness and liveness endpoints
- Uptime and response time metrics

## Configuration

Route prefix is controlled by runtime config `routes.health_prefix` (default: `/health`, restart required). See [Configuration](/en/config/).

> Note: Health endpoints support two authentication methods:
> - **Bearer Token**: `Authorization: Bearer <HEALTH_TOKEN>` (recommended for monitoring/probes, no cookies)
> - **JWT Cookies**: reuse cookies issued by Admin login (recommended for the admin panel/browser)

## Authentication (Important)

Health endpoints require authentication and can be accessed via **Bearer token** (`HEALTH_TOKEN`) or **JWT cookies** (issued after Admin login).  
Health endpoints are treated as disabled only when **both** `api.admin_token` and `api.health_token` are empty (returns `404 Not Found`).

### Option A: Bearer token (recommended for monitoring/probes)

If you configure runtime config `api.health_token`, you can call health endpoints directly with an `Authorization` header:

```bash
HEALTH_TOKEN="your_health_token"

curl -sS \
  -H "Authorization: Bearer ${HEALTH_TOKEN}" \
  http://localhost:8080/health
```

### Option B: JWT cookies (recommended for admin panel/browser)

1. Login via `POST /admin/v1/auth/login` to obtain cookies
2. Call `/health`, `/health/ready`, `/health/live` with those cookies

Example:
```bash
# 1) Login and save cookies
curl -sS -X POST \
  -H "Content-Type: application/json" \
  -c cookies.txt \
  -d '{"password":"your_admin_token"}' \
  http://localhost:8080/admin/v1/auth/login

# 2) Call health check
curl -sS -b cookies.txt \
  http://localhost:8080/health
```

> On first startup, the server auto-generates an admin password and writes it to `admin_token.txt` (if the file doesn't already exist; save it and delete the file).

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

## Monitoring integration notes

If you use **Bearer token** (the value of runtime config `api.health_token`), you can avoid JWT cookie expiration and make automated monitoring easier.

Recommended options:

1. **Recommended: set `api.health_token` and probe `/health/live` or `/health/ready` with `Authorization: Bearer <token>`**
2. **Fallback: probe `/`** (returns `307`, treated as success in Kubernetes) to ensure the process is up
3. **Fallback: login + cookies + `/health`** (for monitors that already have a login step)

### Kubernetes probe example (Bearer token)

```yaml
apiVersion: v1
kind: Pod
spec:
  containers:
  - name: shortlinker
    image: e1saps/shortlinker
    livenessProbe:
      httpGet:
        path: /health/live
        port: 8080
        httpHeaders:
          - name: Authorization
            value: "Bearer your_health_token"
      initialDelaySeconds: 10
      periodSeconds: 10
```

### Kubernetes probe example (simple liveness)

```yaml
apiVersion: v1
kind: Pod
spec:
  containers:
  - name: shortlinker
    image: e1saps/shortlinker
    livenessProbe:
      httpGet:
        path: /
        port: 8080
      initialDelaySeconds: 10
      periodSeconds: 10
```

## Script example (login + health check)

```bash
#!/bin/bash
set -euo pipefail

ADMIN_TOKEN="your_admin_token"
BASE_URL="http://localhost:8080"
COOKIE_JAR="$(mktemp)"

curl -sS -X POST \
  -H "Content-Type: application/json" \
  -c "$COOKIE_JAR" \
  -d "{\"password\":\"${ADMIN_TOKEN}\"}" \
  "${BASE_URL}/admin/v1/auth/login" >/dev/null

curl -sS -b "$COOKIE_JAR" "${BASE_URL}/health"
```

## Security notes

1. Use a strong admin password (`api.admin_token`)
2. Restrict access to health endpoints to trusted networks
3. Use HTTPS in production and configure cookie security correctly
