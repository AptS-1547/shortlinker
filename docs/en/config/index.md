# Configuration Guide

Shortlinker configuration is split into two layers:

- **Startup config**: stored in `config.toml`, changes require a restart
- **Runtime config**: stored in the database, can be updated at runtime via the admin panel / Admin API

## Configuration Architecture

```
config.toml (read at startup)
       ↓
StaticConfig (startup config, in-memory)
       ↓
Database (short links + runtime config)
       ↓
RuntimeConfig (runtime config cache, in-memory)
       ↓
Business logic (routes/auth/cache/etc)
```

On first startup, the server initializes **runtime config defaults** into the database (based on built-in definitions) and loads them into memory. After that, **database values take precedence**.  
The current version does **not** migrate/override runtime config from `config.toml` or environment variables.

## Configuration Methods

### TOML config file (startup config)

```toml
# config.toml
[server]
host = "127.0.0.1"
port = 8080

[database]
database_url = "shortlinks.db"

[cache]
type = "memory"

[logging]
level = "info"
```

> Notes:
> - The backend reads `config.toml` from the **current working directory** (relative path).
> - You can generate a template via `./shortlinker generate-config config.toml` (startup config only).
> - Startup config can be overridden via environment variables: prefix `SL__`, nesting separator `__` (priority: ENV > `config.toml` > defaults). Example: `SL__SERVER__PORT=9999`.
>   - On startup, the program also tries to load a `.env` file from the current directory (it does not override existing env vars), so you can put `SL__...` variables there as well.

### Admin panel / API (runtime config)

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
curl -sS -X POST -b cookies.txt \
  -H "X-CSRF-Token: ${CSRF_TOKEN}" \
  http://localhost:8080/admin/v1/config/reload

# 6) Config history (optional limit, default 20)
curl -sS -b cookies.txt \
  "http://localhost:8080/admin/v1/config/features.random_code_length/history?limit=10"
```

> Sensitive values (e.g. `api.admin_token`, `api.jwt_secret`) are masked as `[REDACTED]` in API responses.

## Startup Config Parameters

These settings live in `config.toml` and require a restart to take effect.

### Server

| TOML key | Type | Default | Description |
|--------|------|---------|-------------|
| `server.host` | String | `127.0.0.1` | Bind address (use `0.0.0.0` in Docker) |
| `server.port` | Integer | `8080` | Bind port |
| `server.unix_socket` | String | *(empty)* | Unix socket path (overrides host/port) |
| `server.cpu_count` | Integer | *(auto)* | Worker threads (defaults to CPU cores, capped at 32) |

### Database

| TOML key | Type | Default | Description |
|--------|------|---------|-------------|
| `database.database_url` | String | `shortlinks.db` | Database URL or file path (backend type inferred from this value) |
| `database.pool_size` | Integer | `10` | Pool size (MySQL/PostgreSQL only; SQLite uses built-in pool settings) |
| `database.timeout` | Integer | `30` | *(currently unused; connect/acquire timeout is fixed at 8s)* |
| `database.retry_count` | Integer | `3` | Retry count for some DB operations |
| `database.retry_base_delay_ms` | Integer | `100` | Retry base delay (ms) |
| `database.retry_max_delay_ms` | Integer | `2000` | Retry max delay (ms) |

See [Storage Backends](/en/config/storage) for URL formats.

### Cache

| TOML key | Type | Default | Description |
|--------|------|---------|-------------|
| `cache.type` | String | `memory` | Cache type: `memory` / `redis` |
| `cache.default_ttl` | Integer | `3600` | Default TTL (seconds) |
| `cache.redis.url` | String | `redis://127.0.0.1:6379/` | Redis URL |
| `cache.redis.key_prefix` | String | `shortlinker:` | Redis key prefix |
| `cache.memory.max_capacity` | Integer | `10000` | In-memory cache max entries |

### Logging

| TOML key | Type | Default | Description |
|--------|------|---------|-------------|
| `logging.level` | String | `info` | Log level: error / warn / info / debug / trace |
| `logging.format` | String | `text` | Output format: `text` / `json` |
| `logging.file` | String | *(empty)* | Log file path (empty = stdout) |
| `logging.max_backups` | Integer | `5` | How many rotated files to keep |
| `logging.enable_rotation` | Boolean | `true` | Enable rotation (currently daily rotation) |
| `logging.max_size` | Integer | `100` | *(currently unused; rotation is time-based)* |

### GeoIP (startup)

| TOML key | Type | Default | Description |
|--------|------|---------|-------------|
| `analytics.maxminddb_path` | String | *(empty)* | MaxMind GeoLite2-City.mmdb path (optional; preferred when readable) |
| `analytics.geoip_api_url` | String | `http://ip-api.com/json/{ip}?fields=status,countryCode,city` | External GeoIP API URL fallback (`{ip}` placeholder) |

> Notes:
> - Provider selection: when `analytics.maxminddb_path` is set and readable, MaxMind is used; otherwise it falls back to the external API (`analytics.geoip_api_url`).
> - The external API provider has a built-in cache (not configurable): LRU max 10,000 entries, TTL 15 minutes (including negative caching on failures). Concurrent lookups for the same IP are singleflighted into one request. HTTP timeout is 2 seconds.

## Runtime Config Keys

These settings are stored in the database and can be changed at runtime via the admin panel / Admin API.

### API

| Key | Type | Default | Restart | Description |
|-----|------|---------|---------|-------------|
| `api.admin_token` | String | *(auto-generated)* | No | Admin login password for `POST /admin/v1/auth/login` |
| `api.health_token` | String | *(empty)* | No | Bearer token for Health API (`Authorization: Bearer ...`, recommended for monitoring/probes; if empty, only JWT cookie auth is available). Note: health endpoints are treated as disabled only when both `api.admin_token` and `api.health_token` are empty (returns `404`) |
| `api.jwt_secret` | String | *(auto-generated)* | No | JWT signing secret |
| `api.access_token_minutes` | Integer | `15` | No | Access token TTL (minutes) |
| `api.refresh_token_days` | Integer | `7` | No | Refresh token TTL (days) |
| `api.cookie_secure` | Boolean | `true` | No | HTTPS-only cookies (browser-facing; re-login recommended after changes) |
| `api.cookie_same_site` | String | `Lax` | No | SameSite policy (re-login recommended after changes) |
| `api.cookie_domain` | String | *(empty)* | No | Cookie domain (re-login recommended after changes) |
| `api.trusted_proxies` | StringArray | `[]` | No | Trusted proxy IPs or CIDRs for login rate-limit IP extraction.<br>**Auto-detect** (default): When empty, connections from private IPs (RFC1918: 10.0.0.0/8, 172.16.0.0/12, 192.168.0.0/16) or localhost automatically trust X-Forwarded-For, suitable for Docker/nginx reverse proxy.<br>**Explicit config**: When set, only trust IPs in the list, e.g., `["10.0.0.1", "172.17.0.0/16"]`.<br>**Security**: Public IPs never trust X-Forwarded-For by default to prevent spoofing. |

> Notes:
> - Cookie names are fixed: `shortlinker_access` / `shortlinker_refresh` / `csrf_token` (not configurable).
> - `api.admin_token` is stored as an Argon2 hash in the database. Use `./shortlinker reset-password` to rotate the admin password.
> - On first startup, the server auto-generates an admin password and writes it to `admin_token.txt` (if the file doesn't already exist; save it and delete the file).

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

> **Note**:
> - `analytics.enable_detailed_logging` requires a server restart to take effect. When enabled, each click is recorded to the `click_logs` table with detailed fields (timestamp, referrer, `user_agent_hash`, etc). User-Agent strings are deduplicated into the `user_agents` table and linked by hash (used by device/browser analytics).
> - IPs are only recorded when `analytics.enable_ip_logging` is enabled, and geo lookup only happens when `analytics.enable_geo_lookup` is enabled (using the startup `[analytics]` provider settings). This data powers Analytics API features like trend analysis, referrer stats, geographic distribution, and device analytics.
> - Data retention/cleanup is controlled by `analytics.enable_auto_rollup`: when enabled, it periodically cleans expired data according to `analytics.log_retention_days` / `analytics.hourly_retention_days` / `analytics.daily_retention_days`.
> - In the current implementation, retention parameters are read when the background task starts; after changing retention days, you may need to restart the server for the cleanup task to pick up new values.

### CORS

| Key | Type | Default | Restart | Description |
|-----|------|---------|---------|-------------|
| `cors.enabled` | Boolean | `false` | Yes | Enable CORS (when disabled, no CORS headers are added; browser keeps same-origin policy) |
| `cors.allowed_origins` | StringArray | `[]` | Yes | Allowed origins (JSON array; `["*"]` = allow any origin; empty array = same-origin only / no cross-origin) |
| `cors.allowed_methods` | EnumArray | `["GET","POST","PUT","DELETE","PATCH","HEAD","OPTIONS"]` | Yes | Allowed methods |
| `cors.allowed_headers` | StringArray | `["Content-Type","Authorization","Accept"]` | Yes | Allowed headers (for cross-origin + cookie write ops, you typically also need `X-CSRF-Token`) |
| `cors.max_age` | Integer | `3600` | Yes | Preflight cache TTL (seconds) |
| `cors.allow_credentials` | Boolean | `false` | Yes | Allow credentials (needed for cross-origin cookies; not recommended together with `["*"]` for security) |

## Priority

1. **Database (runtime config)**: `api.*` / `routes.*` / `features.*` / `click.*` / `cors.*` / `analytics.*` (click analytics)
2. **Environment variables (startup overrides)**: `SL__...` (overrides `[server]` / `[database]` / `[cache]` / `[logging]` / `[analytics]` in `config.toml`)
3. **`config.toml` (startup config)**: `[server]` / `[database]` / `[cache]` / `[logging]` / `[analytics]` (GeoIP provider)
4. **Program defaults**: used when DB / env vars / `config.toml` doesn't provide a value

> Note: environment variables only affect **startup config**. The current version does not migrate/override runtime config from environment variables or `config.toml`.

## Examples

### Development

```toml
# config.toml (dev)
[server]
host = "127.0.0.1"
port = 8080

[database]
database_url = "dev-links.db"

[logging]
level = "debug"
```

> Runtime config (e.g. `features.enable_admin_panel`, `api.health_token`) lives in the database and should be changed via Admin API or CLI; rotate `api.admin_token` via `./shortlinker reset-password`.

### Production

```toml
[server]
host = "127.0.0.1"
port = 8080
cpu_count = 8

[database]
database_url = "/data/shortlinks.db"
pool_size = 20

[cache]
type = "memory"
default_ttl = 7200

[cache.memory]
max_capacity = 50000

[logging]
level = "info"
format = "json"
file = "/var/log/shortlinker/app.log"
enable_rotation = true
```

### Docker

Mount a startup config file (make sure `server.host` is `0.0.0.0`):

```toml
# /config.toml (inside container)
[server]
host = "0.0.0.0"
port = 8080

[database]
database_url = "sqlite:///data/links.db"
```

Runtime config (stored in the DB) can be set via the built-in CLI; configs marked as “restart required” need a restart to take effect:

```bash
# Enable admin panel (restart required)
/shortlinker config set features.enable_admin_panel true

# Health Bearer token (no restart)
/shortlinker config set api.health_token "your_health_token"
```

## Hot Reload

Shortlinker has two kinds of “hot reload / hot apply”:

1. **Short link data hot reload**: reload links from the storage backend and rebuild in-memory caches (useful after CLI/TUI writes directly to the database).
2. **Runtime config hot apply**: when you update a “no restart” config via the Admin API, it is synced into memory and takes effect immediately.

### Supported

- ✅ Short link data (cache rebuild)
- ✅ Runtime configs marked as “no restart” (applies immediately when updated via Admin API)
- ✅ Cookie settings (`api.cookie_*`): affect newly issued cookies; re-login to get updated cookies

### Not supported

- ❌ Server host/port
- ❌ Database connection
- ❌ Cache type
- ❌ Route prefixes

### Reload methods

```bash
# 1) Reload short link data / caches (Unix: send SIGUSR1)
# Note: SIGUSR1 only reloads link data/caches; it does NOT reload runtime config.
kill -USR1 $(cat shortlinker.pid)

# 2) Reload runtime config from DB (Admin API; requires cookies)
# Notes:
# - If you update a “no restart” config via Admin API (PUT /admin/v1/config/{key}),
#   it usually applies immediately and you don't need this.
# - If you changed configs directly in the database (e.g. via `./shortlinker config set`),
#   call this endpoint to let the server re-load configs from DB.
curl -sS -X POST \
  -H "Content-Type: application/json" \
  -c cookies.txt \
  -d '{"password":"your_admin_token"}' \
  http://localhost:8080/admin/v1/auth/login

CSRF_TOKEN=$(awk '$6=="csrf_token"{print $7}' cookies.txt | tail -n 1)

curl -sS -X POST -b cookies.txt \
  -H "X-CSRF-Token: ${CSRF_TOKEN}" \
  http://localhost:8080/admin/v1/config/reload
```

## Next steps

- 📋 [Storage Backends](/en/config/storage)
- 🛡️ [Admin API](/en/api/admin)
- 🏥 [Health Check API](/en/api/health)
