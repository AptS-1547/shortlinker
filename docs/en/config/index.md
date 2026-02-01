# Configuration Guide

Shortlinker configuration is split into two layers:

- **Startup config**: stored in `config.toml`, changes require a restart
- **Runtime config**: stored in the database, can be updated at runtime via the admin panel / Admin API

## Configuration Architecture

```
config.toml (read at startup)
       ↓
Database (persistent storage)
       ↓
RuntimeConfig (in-memory cache)
       ↓
AppConfig (global config)
       ↓
Business logic
```

On first startup, runtime config is migrated from `config.toml` and/or environment variables into the database. After that, **database values take precedence**.

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

### Environment variables

```bash
SERVER_HOST=127.0.0.1
SERVER_PORT=8080
DATABASE_URL=shortlinks.db
RUST_LOG=debug
```

### Admin panel / API (runtime config)

Runtime config can be updated via the Admin API `/{ADMIN_ROUTE_PREFIX}/v1/config`.

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

These settings live in `config.toml` and can be overridden by environment variables.

### Server

| Env | Type | Default | Description |
|-----|------|---------|-------------|
| `SERVER_HOST` | String | `127.0.0.1` | Bind address |
| `SERVER_PORT` | Integer | `8080` | Bind port |
| `UNIX_SOCKET` | String | *(empty)* | Unix socket path (overrides host/port) |
| `CPU_COUNT` | Integer | *(auto)* | Worker threads (defaults to CPU cores) |

### Database

| Env | Type | Default | Description |
|-----|------|---------|-------------|
| `DATABASE_URL` | String | `shortlinks.db` | Database URL or file path |
| `DATABASE_POOL_SIZE` | Integer | `10` | Connection pool size |
| `DATABASE_TIMEOUT` | Integer | `30` | Connection timeout (seconds) |

See [Storage Backends](/en/config/storage) for details.

### Cache

| Env | Type | Default | Description |
|-----|------|---------|-------------|
| `CACHE_TYPE` | String | `memory` | Cache type: `memory`, `redis` |
| `CACHE_DEFAULT_TTL` | Integer | `3600` | Default TTL (seconds) |
| `REDIS_URL` | String | `redis://127.0.0.1:6379/` | Redis URL |
| `REDIS_KEY_PREFIX` | String | `shortlinker:` | Redis key prefix |
| `MEMORY_MAX_CAPACITY` | Integer | `10000` | In-memory cache max entries |

### Logging

| Env | Type | Default | Description |
|-----|------|---------|-------------|
| `RUST_LOG` | String | `info` | Log level: `error`, `warn`, `info`, `debug`, `trace` |

> Log format and file output are configured via `config.toml` `[logging]` (e.g. `logging.format`, `logging.file`). The current version does not provide env overrides for them.

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
| `api.trusted_proxies` | Json | `[]` | No | Trusted proxy IPs or CIDRs (for login rate-limit IP extraction). Empty = trust no proxies (use connection IP only). Example: `["10.0.0.1", "192.168.1.0/24"]` |

> Notes:
> - Cookie names are fixed: `shortlinker_access` / `shortlinker_refresh` / `csrf_token` (not configurable).
> - `api.admin_token` is stored as an Argon2 hash in the database. Use `./shortlinker reset-password` to rotate the admin password.
> - If you didn't set `ADMIN_TOKEN`, the server will auto-generate one on first startup and write it to `admin_token.txt` (save it and delete the file).

### Routes

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

### CORS

| Key | Type | Default | Restart | Description |
|-----|------|---------|---------|-------------|
| `cors.enabled` | Boolean | `false` | Yes | Enable CORS (when disabled, no CORS headers are added; browser keeps same-origin policy) |
| `cors.allowed_origins` | Json | `[]` | Yes | Allowed origins (JSON array; `["*"]` = allow any origin; empty array = same-origin only / no cross-origin) |
| `cors.allowed_methods` | Json | `["GET","POST","PUT","DELETE","OPTIONS","HEAD"]` | Yes | Allowed methods |
| `cors.allowed_headers` | Json | `["Content-Type","Authorization","Accept","X-CSRF-Token"]` | Yes | Allowed headers |
| `cors.max_age` | Integer | `3600` | Yes | Preflight cache TTL (seconds) |
| `cors.allow_credentials` | Boolean | `false` | Yes | Allow credentials (needed for cross-origin cookies; not recommended together with `["*"]` for security) |

## Priority

1. **Database config** (runtime config, highest priority)
2. **Environment variables**
3. **TOML config file**
4. **Program defaults** (lowest)

## Examples

### Development

```bash
SERVER_HOST=127.0.0.1
SERVER_PORT=8080
RUST_LOG=debug
DATABASE_URL=dev-links.db
ADMIN_TOKEN=dev_admin
```

### Production

```toml
[server]
host = "127.0.0.1"
port = 8080
cpu_count = 8

[database]
database_url = "/data/shortlinks.db"
pool_size = 20
timeout = 60

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

### Docker (first startup seeding)

```bash
SERVER_HOST=0.0.0.0
SERVER_PORT=8080
CPU_COUNT=4
DATABASE_URL=/data/links.db
ADMIN_TOKEN=secure_admin_token_here
ENABLE_ADMIN_PANEL=true
```

## Hot Reload

Shortlinker has two kinds of “hot reload / hot apply”:

1. **Short link data hot reload**: reload links from the storage backend and rebuild in-memory caches (useful after CLI/TUI writes directly to the database).
2. **Runtime config hot apply**: when you update a “no restart” config via the Admin API, it is synced into memory and takes effect immediately.

### Supported

- ✅ Short link data (cache rebuild)
- ✅ Runtime configs marked as “no restart” (applies immediately when updated via Admin API)

### Not supported

- ❌ Server host/port
- ❌ Database connection
- ❌ Cache type
- ❌ Route prefixes
- ❌ Cookie settings

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
