# Startup Parameters

These settings live in `config.toml` and require restart to take effect.

> For backend-specific database URL details, see [Storage Overview](/en/config/storage).

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
| `database.pool_size` | Integer | `20` | Pool size (MySQL/PostgreSQL only; SQLite uses built-in pool settings) |
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

### IPC

| TOML key | Type | Default | Description |
|--------|------|---------|-------------|
| `ipc.enabled` | Boolean | `true` | Enable IPC server (required for CLI/TUI communication with a running server) |
| `ipc.socket_path` | String | *(platform default)* | Custom IPC path (Unix socket / Windows named pipe) |
| `ipc.max_message_size` | Integer | `65536` | Max IPC message size in bytes |
| `ipc.timeout` | Integer | `5` | Default IPC timeout (seconds) |
| `ipc.reload_timeout` | Integer | `30` | Timeout for reload-type IPC operations (seconds) |
| `ipc.bulk_timeout` | Integer | `60` | Timeout for import/export IPC operations (seconds) |

> Notes:
> - Path priority: CLI `--socket` > `ipc.socket_path` > platform default. Defaults are Unix `./shortlinker.sock`, Windows `\\.\\pipe\\shortlinker`.
> - On Unix, the IPC socket file permission is fixed to `0600` (owner-only read/write).
> - If `ipc.enabled=false`, `./shortlinker status` and CLI/TUI IPC sync are unavailable; use Admin API `POST /admin/v1/config/reload` or restart to apply runtime config changes.

### GeoIP (startup)

| TOML key | Type | Default | Description |
|--------|------|---------|-------------|
| `analytics.maxminddb_path` | String | *(empty)* | MaxMind GeoLite2-City.mmdb path (optional; preferred when readable) |
| `analytics.geoip_api_url` | String | `http://ip-api.com/json/{ip}?fields=status,countryCode,city` | External GeoIP API URL fallback (`{ip}` placeholder) |

> Notes:
> - Provider selection: when `analytics.maxminddb_path` is set and readable, MaxMind is used; otherwise it falls back to the external API (`analytics.geoip_api_url`).
> - The external API provider has a built-in cache (not configurable): LRU max 10,000 entries, TTL 15 minutes (including negative caching on failures). Concurrent lookups for the same IP are singleflighted into one request. HTTP timeout is 2 seconds.
> - The current version initializes a GeoIP provider, but GeoIP lookup is not yet executed in the click-write path, so `click_logs.country/city` remain null by default.
