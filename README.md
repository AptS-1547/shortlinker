# shortlinker

<div align="center">

[![GitHub release (latest by date)](https://img.shields.io/github/v/release/AptS-1547/shortlinker)](https://github.com/AptS-1547/shortlinker/releases)
[![Rust Release](https://img.shields.io/github/actions/workflow/status/AptS-1547/shortlinker/rust-release.yml?label=rust%20release)](https://github.com/AptS-1547/shortlinker/actions/workflows/rust-release.yml)
[![Docker Build](https://img.shields.io/github/actions/workflow/status/AptS-1547/shortlinker/docker-image.yml?label=docker%20build)](https://github.com/AptS-1547/shortlinker/actions/workflows/docker-image.yml)
[![CodeFactor](https://www.codefactor.io/repository/github/apts-1547/shortlinker/badge)](https://www.codefactor.io/repository/github/apts-1547/shortlinker)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Docker Pulls](https://img.shields.io/docker/pulls/e1saps/shortlinker)](https://hub.docker.com/r/e1saps/shortlinker)

**A minimalist URL shortener service supporting HTTP 307 redirection, built with Rust. Easy to deploy and lightning fast.**

[English](README.md) • [中文](README.zh.md)

![1749756794700](assets/admin-panel-dashboard.png)

</div>

## 🚀 Benchmark (v0.2.0)

**Environment**

- OS: Linux
- CPU: Single-core @ 12th Gen Intel(R) Core(TM) i5-12500
- Tool: [`wrk`](https://github.com/wg/wrk)

| Type       | Scenario                        | QPS Peak          | Cache Hit | Bloom Filter | DB Access |
| ---------- | ------------------------------- | ----------------- | --------- | ------------ | --------- |
| Cache Hit  | Hot shortlink (repeated access) | **677,963.46** | ✅ Yes    | ✅ Yes       | ❌ No     |
| Cache Miss | Cold shortlink (random access)  | **600,622.46** | ❌ No     | ✅ Yes       | ✅ Yes    |

> 💡 Even under cache miss, the system sustains nearly 600k QPS — demonstrating excellent performance with SQLite, `actix-web`, and async caching.

## ✨ Features

* 🚀 **High Performance**: Built with Rust + Actix-web
* 🎯 **Dynamic Management**: Add or remove links at runtime without restarting
* 🎲 **Smart Short Codes**: Supports both custom and randomly generated codes
* ⏰ **Expiration Support**: Set expiration times with flexible time formats (v0.1.1+)
* 💾 **Multiple Storage Backends**: SQLite database, JSON file storage
* 🔄 **Cross-Platform**: Works on Windows, Linux, and macOS
* 🛡️ **Admin API**: HTTP API for link management (v0.0.5+)
* 🏥 **Health Monitoring**: Built-in health check endpoints
* 🐳 **Containerized**: Optimized Docker image for easy deployment
* 🎨 **Beautiful CLI**: Colorized command-line interface
* 🔌 **Unix Socket**: Support for Unix socket binding

## Quick Start

### Run Locally

```bash
git clone https://github.com/AptS-1547/shortlinker
cd shortlinker
cargo run
```

### Deploy with Docker

```bash
# TCP port
docker run -d -p 8080:8080 -v $(pwd)/data:/data e1saps/shortlinker

# Unix socket
docker run -d -v $(pwd)/data:/data -v $(pwd)/sock:/sock \
  -e UNIX_SOCKET=/sock/shortlinker.sock e1saps/shortlinker
```

## Usage Example

Once your domain (e.g. `esap.cc`) is bound:

* `https://esap.cc/github` → custom short link
* `https://esap.cc/aB3dF1` → random short link
* `https://esap.cc/` → default homepage

## Command-Line Management

```bash
# Start the server
./shortlinker

# Add short links
./shortlinker add github https://github.com           # Custom code
./shortlinker add https://github.com                  # Random code
./shortlinker add github https://new-url.com --force  # Overwrite existing

# Using relative time format (v0.1.1+)
./shortlinker add daily https://example.com --expire 1d      # Expires in 1 day
./shortlinker add weekly https://example.com --expire 1w     # Expires in 1 week
./shortlinker add complex https://example.com --expire 1d2h30m  # Complex format

# Manage links
./shortlinker update github https://new-github.com --expire 30d
./shortlinker list                    # List all links
./shortlinker remove github           # Remove specific link

# Server control
./shortlinker start                   # Start server
./shortlinker stop                    # Stop server
./shortlinker restart                 # Restart server
```

## Admin API (v0.0.5+)

HTTP API for link management with Bearer token authentication.

### Setup

```bash
export ADMIN_TOKEN=your_secret_token
export ADMIN_ROUTE_PREFIX=/admin  # optional
```

### Examples

```bash
# Get all links
curl -H "Authorization: Bearer your_secret_token" \
     http://localhost:8080/admin/link

# Create link with relative time
curl -X POST \
     -H "Authorization: Bearer your_secret_token" \
     -H "Content-Type: application/json" \
     -d '{"code":"github","target":"https://github.com","expires_at":"7d"}' \
     http://localhost:8080/admin/link

# Auto-generate random code
curl -X POST \
     -H "Authorization: Bearer your_secret_token" \
     -H "Content-Type: application/json" \
     -d '{"target":"https://github.com","expires_at":"30d"}' \
     http://localhost:8080/admin/link

# Update link
curl -X PUT \
     -H "Authorization: Bearer your_secret_token" \
     -H "Content-Type: application/json" \
     -d '{"target":"https://new-url.com"}' \
     http://localhost:8080/admin/link/github

# Delete link
curl -X DELETE \
     -H "Authorization: Bearer your_secret_token" \
     http://localhost:8080/admin/link/github
```

## Health Check API

Monitor service health and storage status.

```bash
# Setup
export HEALTH_TOKEN=your_health_token

# Health check
curl -H "Authorization: Bearer your_health_token" \
     http://localhost:8080/health

# Readiness check
curl http://localhost:8080/health/ready

# Liveness check  
curl http://localhost:8080/health/live
```

## Time Format Support (v0.1.1+)

### Relative Time (Recommended)

```bash
1s, 5m, 2h, 1d, 1w, 1M, 1y    # Single units
1d2h30m                        # Combined format
```

### RFC3339 Format

```bash
2024-12-31T23:59:59Z           # UTC time
2024-12-31T23:59:59+08:00      # With timezone
```

## Configuration

**shortlinker now supports TOML configuration files!**

Supports both TOML configuration files and environment variables. TOML configuration is clearer and more readable, so it's recommended.

### Custom Configuration File Path

You can specify a custom configuration file path using the `-c` or `--config` parameter:

```bash
# Use custom config file
./shortlinker -c /path/to/your/config.toml
./shortlinker --config /path/to/your/config.toml

# If the specified file doesn't exist, it will be created automatically with default settings
./shortlinker -c /etc/shortlinker/custom.toml
# [INFO] Configuration file not found: /etc/shortlinker/custom.toml
# [INFO] Creating default configuration file...
# [INFO] Default configuration file created at: /etc/shortlinker/custom.toml
```

### TOML Configuration File

Create a `config.toml` file:

```toml
[server]
# Server listening address
host = "127.0.0.1"
# Server listening port
port = 8080
# Unix Socket path (if set, overrides host and port)
# unix_socket = "/tmp/shortlinker.sock"
# CPU core count (defaults to system cores)
cpu_count = 4

[storage]
# Storage backend type: sqlite, postgres, mysql, mariadb
type = "sqlite"
# Database connection URL or file path
database_url = "shortlinks.db"
# Database connection pool size
pool_size = 10
# Database connection timeout (seconds)
timeout = 30

[cache]
# Cache type: memory, redis (currently only memory is supported)
type = "memory"
# Default cache expiration time (seconds)
default_ttl = 3600

[cache.redis]
# Redis connection URL
url = "redis://127.0.0.1:6379/"
# Redis key prefix
key_prefix = "shortlinker:"
# Redis connection pool size
pool_size = 10

[cache.memory]
# Memory cache maximum capacity (entries)
max_capacity = 10000

[api]
# Admin API Token (leave empty to disable admin API)
admin_token = ""
# Health check API Token (leave empty to use admin_token)
health_token = ""

[routes]
# Admin API route prefix
admin_prefix = "/admin"
# Health check route prefix
health_prefix = "/health"
# Frontend panel route prefix
frontend_prefix = "/panel"

[features]
# Whether to enable Web admin panel
enable_admin_panel = false
# Random short code length
random_code_length = 6
# Default redirect URL
default_url = "https://esap.cc/repo"

[logging]
# Log level: trace, debug, info, warn, error
level = "info"
```

**Configuration file loading:**

When using `-c/--config` parameter:
- Uses the specified path (auto-creates if not exists)
- Example: `./shortlinker -c /path/to/config.toml`

When no parameter is specified:
- Only searches for `config.toml` in the current directory
- If not found, uses in-memory default configuration

### Environment Variables (Backward Compatible)

Still supports the original environment variable configuration method. **Environment variables will override TOML configuration:**

| Variable               | Default                  | Description                                 |
| ---------------------- | ------------------------ | ------------------------------------------- |
| `SERVER_HOST`        | `127.0.0.1`            | Listen address                              |
| `SERVER_PORT`        | `8080`                 | Listen port                                 |
| `UNIX_SOCKET`        | *(empty)*              | Unix socket path (overrides HOST/PORT)      |
| `CPU_COUNT`          | *(auto)*               | Worker thread count (defaults to CPU cores) |
| `DATABASE_BACKEND`   | `sqlite`               | Storage type: sqlite, postgres, mysql, mariadb |
| `DATABASE_URL`       | `shortlinks.db`        | Database URL or file path                   |
| `DATABASE_POOL_SIZE` | `10`                   | Database connection pool size               |
| `DATABASE_TIMEOUT`   | `30`                   | Database connection timeout (seconds)       |
| `CACHE_TYPE`         | `memory`               | Cache type: memory, redis                   |
| `CACHE_DEFAULT_TTL`  | `3600`                 | Default cache TTL in seconds                |
| `REDIS_URL`          | `redis://127.0.0.1:6379/` | Redis connection URL                    |
| `REDIS_KEY_PREFIX`   | `shortlinker:`         | Redis key prefix                            |
| `REDIS_POOL_SIZE`    | `10`                   | Redis connection pool size                  |
| `MEMORY_MAX_CAPACITY`| `10000`                | Memory cache max capacity (entries)         |
| `ADMIN_TOKEN`        | *(empty)*              | Admin API token                             |
| `HEALTH_TOKEN`       | *(empty)*              | Health API token                            |
| `ADMIN_ROUTE_PREFIX` | `/admin`               | Admin API route prefix                      |
| `HEALTH_ROUTE_PREFIX`| `/health`              | Health API route prefix                     |
| `FRONTEND_ROUTE_PREFIX` | `/panel`            | Web admin panel route prefix                |
| `ENABLE_ADMIN_PANEL` | `false`                | Enable web admin panel                      |
| `RANDOM_CODE_LENGTH` | `6`                    | Random code length                          |
| `DEFAULT_URL`        | `https://esap.cc/repo` | Default redirect URL                        |
| `RUST_LOG`           | `info`                 | Log level                                   |

> **Note**: The web admin panel is a new feature and may be unstable.

### .env Example

```bash
# Server - TCP
SERVER_HOST=0.0.0.0
SERVER_PORT=8080
CPU_COUNT=4

# Server - Unix socket
# UNIX_SOCKET=/tmp/shortlinker.sock

# Storage
STORAGE_BACKEND=sqlite
DB_FILE_NAME=data/links.db

# APIs
ADMIN_TOKEN=your_admin_token
HEALTH_TOKEN=your_health_token

# Features
DEFAULT_URL=https://example.com
RANDOM_CODE_LENGTH=8
RUST_LOG=info
```

## Storage Backends

- **SQLite** (default, v0.1.0+): Production-ready, recommended
- **MySQL / MariaDB** (v0.1.2+): Production-ready, recommended
- **Postgres** (v0.1.3+): Production-ready, recommended

```bash
# SQLite (recommended)
STORAGE_BACKEND=sqlite
DB_FILE_NAME=links.db

# File storage
STORAGE_BACKEND=file
DB_FILE_NAME=links.json
```

## Deployment

### Reverse Proxy (Nginx)

```nginx
# TCP port
server {
    listen 80;
    server_name esap.cc;
    location / {
        proxy_pass http://127.0.0.1:8080;
        add_header Cache-Control "no-cache, no-store, must-revalidate";
    }
}

# Unix socket
server {
    listen 80;
    server_name esap.cc;
    location / {
        proxy_pass http://unix:/tmp/shortlinker.sock;
        add_header Cache-Control "no-cache, no-store, must-revalidate";
    }
}
```

### systemd Service

```ini
[Unit]
Description=ShortLinker Service
After=network.target

[Service]
Type=simple
User=www-data
WorkingDirectory=/opt/shortlinker
ExecStart=/opt/shortlinker/shortlinker
Restart=always
Environment=SERVER_HOST=127.0.0.1
Environment=SERVER_PORT=8080

[Install]
WantedBy=multi-user.target
```

## Development

```bash
# Development
cargo run

# Production build
cargo build --release

# Run tests
cargo test

# Code quality
cargo fmt && cargo clippy
```
## Related Modules

- **Web Admin Panel**: GUI to manage links in `admin-panel/` ([docs](/admin-panel/))
- **Cloudflare Worker**: Serverless version in `cf-worker/` ([docs](/cf-worker/))

## License

MIT License © AptS:1547

<pre>
        ／＞　 フ
       | 　_　_|    AptS:1547
     ／` ミ＿xノ    — shortlinker assistant bot —
    /　　　　 |
   /　 ヽ　　 ﾉ      Rust / SQLite / Bloom / CLI
   │　　|　|　|
／￣|　　 |　|　|
(￣ヽ＿_ヽ_)__)
＼二)

   「ready to 307 !」
</pre>

> [🔗 Visit Project Docs](https://esap.cc/docs)
> [💬 Powered by AptS:1547](https://github.com/AptS-1547)
