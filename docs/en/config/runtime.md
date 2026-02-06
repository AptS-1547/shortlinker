# Runtime Keys and API Workflow

This page focuses on runtime config keys (stored in DB) and practical update workflow via Admin API.

## Update Runtime Config via Admin API

Runtime config can be updated via the Admin API `/{routes.admin_prefix}/v1/config` (default: `/admin/v1/config`).

Because Admin API uses **JWT cookies**, you need to login first:

```bash
# 1) Login to obtain cookies
curl -sS -X POST \
  -H "Content-Type: application/json" \
  -c cookies.txt \
  -d '{"password":"your_admin_token"}' \
  http://localhost:8080/admin/v1/auth/login

# Extract CSRF token (required for PUT/POST/DELETE write operations)
CSRF_TOKEN=$(awk '$6=="csrf_token"{print $7}' cookies.txt | tail -n 1)

# 2) List all configs
curl -sS -b cookies.txt \
  http://localhost:8080/admin/v1/config

# 3) Get a single config
curl -sS -b cookies.txt \
  http://localhost:8080/admin/v1/config/features.random_code_length

# 4) Update a config
curl -sS -X PUT \
  -b cookies.txt \
  -H "X-CSRF-Token: ${CSRF_TOKEN}" \
  -H "Content-Type: application/json" \
  -d '{"value":"8"}' \
  http://localhost:8080/admin/v1/config/features.random_code_length

# 5) Reload config
# Note: CLI `config set/reset/import` already auto-attempts this via IPC.
# If IPC is unreachable (server not running, ipc.enabled=false, socket mismatch, etc.),
# call this endpoint manually.
curl -sS -X POST -b cookies.txt \
  -H "X-CSRF-Token: ${CSRF_TOKEN}" \
  http://localhost:8080/admin/v1/config/reload

# 6) Config history (optional limit, default 20)
curl -sS -b cookies.txt \
  "http://localhost:8080/admin/v1/config/features.random_code_length/history?limit=10"
```

> Sensitive values (e.g. `api.admin_token`, `api.jwt_secret`) are masked as `[REDACTED]` in API responses.

> **Config action note**: currently only `api.jwt_secret` supports the `generate_token` action.


## Runtime Config Keys

These settings are stored in the database and can be changed at runtime via the admin panel / Admin API.

### API

| Key | Type | Default | Restart | Description |
|-----|------|---------|---------|-------------|
| `api.admin_token` | String | *(auto-generated)* | No | Admin login password for `POST /admin/v1/auth/login` |
| `api.health_token` | String | *(empty)* | No | Bearer token for Health API (`Authorization: Bearer ...`, recommended for monitoring/probes; if empty, only JWT cookie auth is available). Note: health endpoints are treated as disabled only when both `api.admin_token` and `api.health_token` are empty (returns `404`) |
| `api.jwt_secret` | String | *(auto-generated)* | Yes | JWT signing secret |
| `api.access_token_minutes` | Integer | `15` | Yes | Access token TTL (minutes) |
| `api.refresh_token_days` | Integer | `7` | Yes | Refresh token TTL (days) |
| `api.cookie_secure` | Boolean | `true` | No | HTTPS-only cookies (browser-facing; re-login recommended after changes) |
| `api.cookie_same_site` | String | `Lax` | No | SameSite policy (re-login recommended after changes) |
| `api.cookie_domain` | String | *(empty)* | No | Cookie domain (re-login recommended after changes) |
| `api.trusted_proxies` | StringArray | `[]` | No | Trusted proxy IPs or CIDRs for login rate-limit IP extraction.<br>**Auto-detect** (default): When empty, connections from private IPs (RFC1918: 10.0.0.0/8, 172.16.0.0/12, 192.168.0.0/16) or localhost automatically trust X-Forwarded-For, suitable for Docker/nginx reverse proxy.<br>**Explicit config**: When set, only trust IPs in the list, e.g., `["10.0.0.1", "172.17.0.0/16"]`.<br>**Security**: Public IPs never trust X-Forwarded-For by default to prevent spoofing. |

> Notes:
> - Cookie names are fixed: `shortlinker_access` / `shortlinker_refresh` / `csrf_token` (not configurable).
> - `api.admin_token` is stored as an Argon2 hash in the database. Use `./shortlinker reset-password` to rotate the admin password.
> - On first startup, the server auto-generates an admin password and writes it to `admin_token.txt` (if the file doesn't already exist; save it and delete the file).
> - Current implementation detail: JWT service reads config on first use and then caches it in-process (`OnceLock`). These keys are marked as requiring restart; after changing `api.jwt_secret`, `api.access_token_minutes`, or `api.refresh_token_days`, restart the service to affect newly issued/validated tokens.

### Routes

> Note: these prefixes are treated as “reserved short-code prefixes”. Short link `code` cannot equal these prefixes (without the leading `/`) and cannot start with `{prefix}/`, otherwise it will conflict with system routes.

| Key | Type | Default | Restart | Description |
|-----|------|---------|---------|-------------|
| `routes.admin_prefix` | String | `/admin` | Yes | Admin API prefix |
| `routes.health_prefix` | String | `/health` | Yes | Health API prefix |
| `routes.frontend_prefix` | String | `/panel` | Yes | Admin panel (frontend) prefix |

### Features

| Key | Type | Default | Restart | Description |
|-----|------|---------|---------|-------------|
| `features.enable_admin_panel` | Boolean | `false` | Yes | Enable web admin panel |
| `features.random_code_length` | Integer | `6` | No | Random short code length |
| `features.default_url` | String | `https://esap.cc/repo` | No | Default redirect URL for `/` |

### Click tracking

| Key | Type | Default | Restart | Description |
|-----|------|---------|---------|-------------|
| `click.enable_tracking` | Boolean | `true` | Yes | Enable click tracking |
| `click.flush_interval` | Integer | `30` | Yes | Flush interval (seconds) |
| `click.max_clicks_before_flush` | Integer | `100` | Yes | Max clicks before flush |

### Detailed Analytics

| Key | Type | Default | Restart | Description |
|-----|------|---------|---------|-------------|
| `analytics.enable_detailed_logging` | Boolean | `false` | Yes | Enable detailed click logging (writes to click_logs table) |
| `analytics.enable_auto_rollup` | Boolean | `true` | Yes | Enable automatic data retention / rollup-table cleanup task (runs every 4 hours by default) |
| `analytics.log_retention_days` | Integer | `30` | No | Raw click log retention in days (cleaned by the background task; requires `analytics.enable_auto_rollup`) |
| `analytics.hourly_retention_days` | Integer | `7` | No | Hourly rollup retention in days (cleans `click_stats_hourly` / `click_stats_global_hourly`; requires `analytics.enable_auto_rollup`) |
| `analytics.daily_retention_days` | Integer | `365` | No | Daily rollup retention in days (cleans `click_stats_daily`; requires `analytics.enable_auto_rollup`) |
| `analytics.enable_ip_logging` | Boolean | `true` | No | Whether to record IP addresses |
| `analytics.enable_geo_lookup` | Boolean | `false` | No | Whether to enable geo-IP lookup |
| `analytics.sample_rate` | Float | `1.0` | No | Detailed logging sample rate (0.0-1.0; 1.0 = log all clicks, 0.1 = log ~10% of clicks) |
| `analytics.max_log_rows` | Integer | `0` | No | Maximum rows in `click_logs` (0 = unlimited) |
| `analytics.max_rows_action` | Enum | `cleanup` | No | Behavior when `max_log_rows` is exceeded: `cleanup` (delete oldest rows) or `stop` (stop detailed logging) |

> **Note**:
> - `analytics.enable_detailed_logging` is marked as restart-required. After changing it, restart the server for the setting to take effect. When enabled, each click is recorded to the `click_logs` table with detailed fields (timestamp, referrer, `user_agent_hash`, etc). User-Agent strings are deduplicated into the `user_agents` table and linked by hash (used by device/browser analytics).
> - IPs are only recorded when `analytics.enable_ip_logging` is enabled, and geo lookup only happens when `analytics.enable_geo_lookup` is enabled (using the startup `[analytics]` provider settings). This data powers Analytics API features like trend analysis, referrer stats, geographic distribution, and device analytics.
> - `click_logs.source` is derived by this order: use `utm_source` from request query first; if absent, extract domain from `Referer` and store `ref:{domain}`; if both are missing, store `direct`.
> - Data retention/cleanup is controlled by `analytics.enable_auto_rollup`: when enabled, it periodically cleans expired data according to `analytics.log_retention_days` / `analytics.hourly_retention_days` / `analytics.daily_retention_days`.
> - In the current implementation, retention parameters are read when the background task starts; after changing retention days, you may need to restart the server for the cleanup task to pick up new values.

### UTM passthrough

| Key | Type | Default | Restart | Description |
|-----|------|---------|---------|-------------|
| `utm.enable_passthrough` | Boolean | `false` | No | Enable UTM passthrough during redirect (only `utm_source` / `utm_medium` / `utm_campaign` / `utm_term` / `utm_content`) |

> **Notes**:
> - Disabled by default. When enabled, UTM params are appended only if those keys exist in the incoming request URL.
> - If target URL already has a query string, params are appended with `&`; otherwise with `?`.
> - Current implementation appends raw incoming UTM query fragments directly (no extra URL decode/re-encode step).

### CORS

| Key | Type | Default | Restart | Description |
|-----|------|---------|---------|-------------|
| `cors.enabled` | Boolean | `false` | Yes | Enable CORS (when disabled, no CORS headers are added; browser keeps same-origin policy) |
| `cors.allowed_origins` | StringArray | `[]` | Yes | Allowed origins (JSON array; `["*"]` = allow any origin; empty array = same-origin only / no cross-origin) |
| `cors.allowed_methods` | EnumArray | `["GET","POST","PUT","DELETE","PATCH","HEAD","OPTIONS"]` | Yes | Allowed methods |
| `cors.allowed_headers` | StringArray | `["Content-Type","Authorization","Accept"]` | Yes | Allowed headers (for cross-origin + cookie write ops, you typically also need `X-CSRF-Token`) |
| `cors.max_age` | Integer | `3600` | Yes | Preflight cache TTL (seconds) |
| `cors.allow_credentials` | Boolean | `false` | Yes | Allow credentials (needed for cross-origin cookies; not recommended together with `["*"]` for security) |

> For config priority, see [Configuration Guide](/en/config/) to keep a single source of truth.
