# shortlinker

<div align="center">

[![GitHub release (latest by date)](https://img.shields.io/github/v/release/AptS-1547/shortlinker)](https://github.com/AptS-1547/shortlinker/releases)
[![Rust Release](https://img.shields.io/github/actions/workflow/status/AptS-1547/shortlinker/rust-release.yml?label=rust%20release)](https://github.com/AptS-1547/shortlinker/actions/workflows/rust-release.yml)
[![Docker Build](https://img.shields.io/github/actions/workflow/status/AptS-1547/shortlinker/docker-image.yml?label=docker%20build)](https://github.com/AptS-1547/shortlinker/actions/workflows/docker-image.yml)
[![CodeFactor](https://www.codefactor.io/repository/github/apts-1547/shortlinker/badge)](https://www.codefactor.io/repository/github/apts-1547/shortlinker)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Docker Pulls](https://img.shields.io/docker/pulls/e1saps/shortlinker)](https://hub.docker.com/r/e1saps/shortlinker)

**A minimalist URL shortener service supporting HTTP 302 redirection, built with Rust. Easy to deploy and lightning fast.**

[English](README.md) • [中文](README.zh.md)

</div>

## ✨ Features

* 🚀 **High Performance**: Built with Rust + Actix-web
* 🎯 **Dynamic Management**: Add or remove links at runtime without restarting
* 🎲 **Smart Short Codes**: Supports both custom and randomly generated codes
* ⏰ **Expiration Support**: Set expiration times for links with automatic cleanup
* 💾 **Multiple Storage Backends**: SQLite database, JSON file storage and Sled embedded database
* 🔄 **Cross-Platform**: Works on Windows, Linux, and macOS
* 🔐 **Process Management**: Smart process locking to prevent duplicate instances
* 🐳 **Containerized**: Optimized Docker image for easy deployment
* 🛡️ **Admin API**: HTTP API for link management (v0.0.5+)

## Quick Start

### Run Locally

```bash
git clone https://github.com/AptS-1547/shortlinker
cd shortlinker
cargo run
```

### Deploy with Docker

```bash
# Pull from Docker Hub
docker run -d -p 8080:8080 -v $(pwd)/data:/data e1saps/shortlinker

# Or pull from GitHub Container Registry
docker run -d -p 8080:8080 -v $(pwd)/data:/data ghcr.io/apts-1547/shortlinker

# Build yourself
docker build -t shortlinker .
docker run -d -p 8080:8080 -v $(pwd)/data:/data shortlinker
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
./shortlinker add temp https://example.com --expires "2024-12-31T23:59:59Z"  # With expiration

# Manage links
./shortlinker list                    # List all links
./shortlinker remove github           # Remove specific link
```

## Admin API (v0.0.5+)

Starting from v0.0.5, HTTP API support for link management is available.

### Authentication Setup

```bash
# Set Admin Token
export ADMIN_TOKEN=your_secret_token

# Custom Admin Route Prefix (optional, defaults to /admin)
export ADMIN_ROUTE_PREFIX=/api/admin
```

### API Endpoints

#### Get All Links
```bash
curl -H "Authorization: Bearer your_secret_token" \
     http://localhost:8080/admin/link
```

#### Create Link
```bash
curl -X POST \
     -H "Authorization: Bearer your_secret_token" \
     -H "Content-Type: application/json" \
     -d '{"code":"github","target":"https://github.com"}' \
     http://localhost:8080/admin/link
```

#### Get Specific Link
```bash
curl -H "Authorization: Bearer your_secret_token" \
     http://localhost:8080/admin/link/github
```

#### Update Link
```bash
curl -X PUT \
     -H "Authorization: Bearer your_secret_token" \
     -H "Content-Type: application/json" \
     -d '{"code":"github","target":"https://github.com/new"}' \
     http://localhost:8080/admin/link/github
```

#### Delete Link
```bash
curl -X DELETE \
     -H "Authorization: Bearer your_secret_token" \
     http://localhost:8080/admin/link/github
```

## Storage Backends

shortlinker supports multiple storage backends, allowing you to choose the right storage solution for your needs.

### SQLite Database Storage (Default)

Uses SQLite lightweight relational database for optimal performance and reliability.

**Pros**:
- High-performance SQL queries
- ACID transaction support
- Mature and stable, production-proven
- Concurrent read support
- Data integrity guarantees
- Lightweight, no additional services required

**Cons**:
- Data not directly editable (requires SQL tools)
- Limited high-concurrency writes

**Configuration**:
```bash
# SQLite storage is used by default
STORAGE_TYPE=sqlite        # Optional, defaults to sqlite
SQLITE_DB_PATH=links.db    # Database file path
```

### File Storage

Stores short link data in JSON files. Simple to use and easy to backup.

**Pros**:
- Simple configuration, no additional dependencies
- Human-readable data format, easy to debug
- Hot-reloading support
- Easy to backup and version control

**Cons**:
- Lower write performance under high concurrency
- Slower loading with large datasets
- No transaction support

**Configuration**:
```bash
STORAGE_TYPE=file          # Enable file storage
LINKS_FILE=links.json      # Storage file path
```

### Sled Database Storage

Uses Sled embedded database for high concurrency performance.

**Pros**:
- High concurrent read/write performance
- Built-in transaction support
- Data compression, smaller storage footprint
- Crash recovery capabilities

**Cons**:
- Data not directly editable
- Higher memory usage
- Newer technology, ecosystem less mature than SQLite

**Configuration**:
```bash
STORAGE_TYPE=sled          # Enable Sled storage
SLED_DB_PATH=links.sled    # Database file path
```

### Recommendations

- **Production environments**: SQLite storage recommended (default)
- **High concurrency scenarios**: SQLite or Sled storage recommended
- **Small deployments** (< 1,000 links): Any storage backend works
- **Medium to large deployments** (> 10,000 links): SQLite storage recommended
- **Frequent backups needed**: File storage recommended
- **Development/debugging**: File storage recommended

## Configuration Options

You can configure the service using environment variables or a `.env` file. The program automatically reads the `.env` file from the project root directory.

| Environment Variable | Default Value | Description        |
| -------------------- | ------------- | ------------------ |
| `SERVER_HOST`        | `127.0.0.1`   | Listen address     |
| `SERVER_PORT`        | `8080`        | Listen port        |
| `STORAGE_TYPE`       | `sqlite`      | Storage backend type (`sqlite`, `file` or `sled`) |
| `SQLITE_DB_PATH`     | `links.db`    | SQLite database path (SQLite storage only) |
| `LINKS_FILE`         | `links.json`  | File storage path (file storage only) |
| `SLED_DB_PATH`       | `links.sled`  | Sled database path (Sled storage only) |
| `DEFAULT_URL`        | `https://esap.cc/repo` | Default redirect URL for root path |
| `RANDOM_CODE_LENGTH` | `6`           | Random code length |
| `RUST_LOG`           | `info`        | Logging level (`error`, `warn`, `info`, `debug`, `trace`) |
| `ADMIN_TOKEN`        | *(empty string)* | Admin API authentication token, Admin API is disabled when empty (v0.0.5+) |
| `ADMIN_ROUTE_PREFIX` | `/admin`      | Admin API route prefix (v0.0.5+) |

### .env File Example

Create a `.env` file in the project root directory:

```bash
# Server configuration
SERVER_HOST=0.0.0.0
SERVER_PORT=3000

# Storage configuration - choose one
# SQLite storage (default)
STORAGE_TYPE=sqlite
SQLITE_DB_PATH=data/links.db

# Or use file storage
# STORAGE_TYPE=file
# LINKS_FILE=data/links.json

# Or use Sled storage
# STORAGE_TYPE=sled
# SLED_DB_PATH=data/links.sled

# Default redirect URL
DEFAULT_URL=https://example.com

# Random code length
RANDOM_CODE_LENGTH=8

# Log level
RUST_LOG=debug

# Admin API configuration (v0.0.5+)
ADMIN_TOKEN=your_secure_admin_token
ADMIN_ROUTE_PREFIX=/api/admin
```

**Note**: Environment variables take precedence over `.env` file settings.

## Server Management

### Starting and Stopping

```bash
# Start server
./shortlinker

# Stop server
kill $(cat shortlinker.pid)
```

### Process Protection

- **Unix Systems**: Uses PID file (`shortlinker.pid`) to prevent duplicate instances
- **Windows Systems**: Uses lock file (`.shortlinker.lock`) to prevent duplicate instances
- Automatically detects running instances and provides helpful messages

## Data Format

Link data is stored in JSON format:

```json
{
  "github": {
    "target": "https://github.com",
    "created_at": "2024-01-01T00:00:00Z",
    "expires_at": null
  },
  "temp": {
    "target": "https://example.com",
    "created_at": "2024-01-01T00:00:00Z",
    "expires_at": "2024-12-31T23:59:59Z"
  }
}
```

## Reverse Proxy Configuration

### Caddy

```caddy
esap.cc {
    reverse_proxy 127.0.0.1:8080
    
    # Optional: Add cache control
    header {
        Cache-Control "no-cache, no-store, must-revalidate"
    }
}
```

### Nginx

```nginx
server {
    listen 80;
    server_name esap.cc;
    
    location / {
        proxy_pass http://127.0.0.1:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        
        # Disable caching
        add_header Cache-Control "no-cache, no-store, must-revalidate";
    }
}
```

### System Service (systemd)

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
RestartSec=5

Environment=SERVER_HOST=127.0.0.1
Environment=SERVER_PORT=8080
Environment=RUST_LOG=info

[Install]
WantedBy=multi-user.target
```

## Technical Details

* **Hot Reloading**: Automatic configuration file change detection
* **Random Code Generation**: Alphanumeric with configurable length, collision avoidance
* **Conflict Handling**: Smart detection with force overwrite option
* **Expiration Checking**: Real-time validation on request, automatic cleanup
* **Container Optimization**: Multi-stage build with `scratch` base image
* **Memory Safety**: Arc + RwLock ensures concurrent safety

## Development

```bash
# Development build
cargo run

# Production build
cargo build --release

# Cross-compilation (requires `cross`)
cross build --release --target x86_64-unknown-linux-musl

# Run tests
cargo test

# Check code formatting
cargo fmt
cargo clippy
```

## Performance Optimization

- Uses `Arc<RwLock<HashMap>>` for high-concurrency reads
- 302 temporary redirects to avoid browser caching
- Minimal memory footprint and CPU usage
- Async I/O handling for high concurrency

## Data Migration

### Migrating from File Storage to SQLite

```bash
# 1. Stop the service
./shortlinker stop

# 2. Backup existing data
cp links.json links.json.backup

# 3. Update configuration
export STORAGE_TYPE=sqlite
export SQLITE_DB_PATH=links.db

# 4. Start service (will automatically load data from file to SQLite)
./shortlinker
```

### Migrating from Sled to SQLite

```bash
# 1. Export data (via Admin API)
curl -H "Authorization: Bearer your_token" \
     http://localhost:8080/admin/link > links_export.json

# 2. Stop the service
./shortlinker stop

# 3. Update configuration
export STORAGE_TYPE=sqlite
export SQLITE_DB_PATH=links.db

# 4. Convert data format and start service
./shortlinker import links_export.json
```

### Migrating from SQLite to File Storage

```bash
# 1. Export data (via Admin API)
curl -H "Authorization: Bearer your_token" \
     http://localhost:8080/admin/link > links_export.json

# 2. Stop the service
./shortlinker stop

# 3. Update configuration
export STORAGE_TYPE=file
export LINKS_FILE=links.json

# 4. Convert data format and start service
./shortlinker import links_export.json
```

## Troubleshooting

### Common Issues

1. **Port Already in Use**
   ```bash
   # Check port usage
   lsof -i :8080
   netstat -tlnp | grep 8080
   ```

2. **Permission Issues**
   ```bash
   # Ensure proper permissions
   chmod 755 /path/to/shortlinker
   chown user:group links.json
   ```

3. **Corrupted Configuration File (File Storage)**
   ```bash
   # Validate JSON format
   jq . links.json
   ```

4. **SQLite Database Issues**
   ```bash
   # Check database file permissions
   ls -la links.db
   
   # Use sqlite3 tool to check database
   sqlite3 links.db ".tables"
   sqlite3 links.db "SELECT COUNT(*) FROM links;"
   ```

5. **Sled Database Lock**
   ```bash
   # Check if other processes are using the database
   ps aux | grep shortlinker
   
   # If no other processes exist, try removing lock files
   rm -rf links.sled/db
   ```

6. **Storage Backend Switching Issues**
   ```bash
   # Ensure configuration is correct
   echo $STORAGE_TYPE
   
   # Check file permissions
   ls -la links.json links.db links.sled/
   ```

## License

MIT License © AptS:1547
