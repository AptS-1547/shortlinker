# Health Check API

Health documentation is now split into overview, endpoint details, and monitoring/troubleshooting pages so it is easier to navigate.

## Navigation

- [Health Check API: Endpoints and Status Codes](/en/api/health-endpoints)
- [Health Check API: Monitoring and Troubleshooting](/en/api/health-monitoring)

## Overview

- Service health checks
- Storage backend status checks
- Readiness and liveness probes
- Uptime and optional metrics export

## Configuration

Health route prefix is controlled by runtime config `routes.health_prefix` (default `/health`; restart required after change). See [Configuration Guide](/en/config/).

> Health API supports two auth modes:
>
> - **Bearer token**: `Authorization: Bearer <HEALTH_TOKEN>` (recommended for monitoring/probes)
> - **JWT cookies**: reuse cookies issued from Admin API login (recommended for browser/admin panel)

## Authentication (Important)

Health API requires authentication via either **Bearer token** (`HEALTH_TOKEN`) or **JWT cookies** (issued by Admin API login).

If both `api.admin_token` and `api.health_token` are empty, health endpoints return `404 Not Found` (treated as disabled).

### Option A: Bearer token (recommended for monitoring/probes)

When `api.health_token` is configured, call the endpoint directly with auth header:

```bash
HEALTH_TOKEN="your_health_token"

curl -sS \
  -H "Authorization: Bearer ${HEALTH_TOKEN}" \
  http://localhost:8080/health
```

### Option B: JWT cookies (recommended for admin panel/browser)

1. Login via `POST /admin/v1/auth/login` and store cookies
2. Reuse cookies for `/health`, `/health/ready`, `/health/live`

Example:

```bash
# 1) login and save cookies
curl -sS -X POST \
  -H "Content-Type: application/json" \
  -c cookies.txt \
  -d '{"password":"your_admin_token"}' \
  http://localhost:8080/admin/v1/auth/login

# 2) call health check
curl -sS -b cookies.txt \
  http://localhost:8080/health
```

> On first startup, the server may generate `admin_token.txt` (if missing). Store it securely, then delete the file.

## Next

- Endpoint payload details and status semantics: [Health Check API: Endpoints and Status Codes](/en/api/health-endpoints)
- Probe strategy, scripts, and troubleshooting: [Health Check API: Monitoring and Troubleshooting](/en/api/health-monitoring)
