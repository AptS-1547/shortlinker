# Health Check API

Shortlinker provides a health check API for monitoring service status and storage health.

## Overview

- Service health status
- Storage backend health checks
- Readiness and liveness endpoints
- Uptime and response time metrics

## Configuration

Route prefix can be configured via environment variables (see [Configuration](/en/config/)):

- `HEALTH_ROUTE_PREFIX` - route prefix (optional, default: `/health`)

> Note: `api.health_token` / `HEALTH_TOKEN` is currently not used for Health API authentication (kept as a config field). Health endpoints currently reuse Admin authentication.

## Authentication (Important)

Health endpoints reuse Admin **JWT Cookie** authentication:

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

> If `api.admin_token` is empty, health endpoints return `404 Not Found` (treated as disabled). If you didn't set `ADMIN_TOKEN`, the server will auto-generate one on first startup and print it once in logs.

## Endpoints

**Base URL**: `http://your-domain:port/health`

> All endpoints support both `GET` and `HEAD`.

### GET /health - Full health check

```bash
curl -sS -b cookies.txt \
  http://localhost:8080/health
```

**Example response**:
```json
{
  "code": 0,
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

## Status codes

| Status | Meaning |
|--------|---------|
| 200 | Healthy/Ready |
| 204 | Alive (no content) |
| 401 | Unauthorized (missing/invalid cookie) |
| 503 | Unhealthy |

> Unauthorized body example: `{"code":401,"data":{"error":"Unauthorized: Invalid or missing token"}}`

## Monitoring integration notes

Because Health API uses cookie-based auth (access token expires), Kubernetes `httpGet` probes are not convenient to use directly for `/health/*`.

Recommended options:

1. **Simple liveness**: probe `/` (returns `307`, treated as success in Kubernetes) to ensure the process is up
2. **Deep checks**: external monitor/script that logs in and calls `/health`

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

1. Use a strong `ADMIN_TOKEN`
2. Restrict access to health endpoints to trusted networks
3. Use HTTPS in production and configure cookie security correctly

