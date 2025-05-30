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
* 💾 **Multiple Storage Backends**: SQLite database, JSON file storage and Sled embedded database (v0.1.0+)
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
# Set Admin Token (required, API disabled when empty)
export ADMIN_TOKEN=your_secret_token

# Custom Route Prefix (optional)
export ADMIN_ROUTE_PREFIX=/api/admin
```

### Common Operations

```bash
# Get All Links
curl -H "Authorization: Bearer your_secret_token" \
     http://localhost:8080/admin/link

# Create Link
curl -X POST \
     -H "Authorization: Bearer your_secret_token" \
     -H "Content-Type: application/json" \
     -d '{"code":"github","target":"https://github.com"}' \
     http://localhost:8080/admin/link

# Delete Link
curl -X DELETE \
     -H "Authorization: Bearer your_secret_token" \
     http://localhost:8080/admin/link/github
```

## Storage Backends

shortlinker supports multiple storage backends starting from v0.1.0:

- **SQLite** (default, v0.1.0+): Production-grade performance, recommended for production
- **File Storage** (default before v0.1.0): Simple and easy to use, convenient for debugging
- **Sled** (v0.1.0+): High concurrency performance, suitable for high-load scenarios

```bash
# SQLite storage (default, v0.1.0+)
STORAGE_TYPE=sqlite
SQLITE_DB_PATH=links.db

# File storage (default before v0.1.0)
STORAGE_TYPE=file
LINKS_FILE=links.json

# Sled storage (v0.1.0+)
STORAGE_TYPE=sled
SLED_DB_PATH=links.sled
```

## Configuration Options

Configure using environment variables or a `.env` file:

| Environment Variable | Default Value | Description        |
| -------------------- | ------------- | ------------------ |
| `SERVER_HOST`        | `127.0.0.1`   | Listen address     |
| `SERVER_PORT`        | `8080`        | Listen port        |
| `STORAGE_TYPE`       | `sqlite`      | Storage backend type |
| `SQLITE_DB_PATH`     | `links.db`    | SQLite database path |
| `LINKS_FILE`         | `links.json`  | File storage path |
| `DEFAULT_URL`        | `https://esap.cc/repo` | Default redirect URL |
| `RANDOM_CODE_LENGTH` | `6`           | Random code length |
| `ADMIN_TOKEN`        | *(empty)*     | Admin API authentication token |
| `RUST_LOG`           | `info`        | Logging level |

### .env File Example

```bash
# Server configuration
SERVER_HOST=0.0.0.0
SERVER_PORT=8080

# Storage configuration
STORAGE_TYPE=sqlite
SQLITE_DB_PATH=data/links.db

# Feature configuration
DEFAULT_URL=https://example.com
RANDOM_CODE_LENGTH=8
RUST_LOG=info

# Admin API configuration
ADMIN_TOKEN=your_secure_admin_token
```

## Reverse Proxy Configuration

### Caddy

```caddy
esap.cc {
    reverse_proxy 127.0.0.1:8080
}
```

### Nginx

```nginx
server {
    listen 80;
    server_name esap.cc;
    location / {
        proxy_pass http://127.0.0.1:8080;
        add_header Cache-Control "no-cache, no-store, must-revalidate";
    }
}
```

### systemd

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
Environment=RUST_LOG=info

[Install]
WantedBy=multi-user.target
```

## Technical Details

* **Hot Reloading**: Automatic configuration file change detection
* **Random Code Generation**: Alphanumeric with configurable length, collision avoidance
* **Expiration Checking**: Real-time validation on request, automatic cleanup
* **Container Optimization**: Multi-stage build with `scratch` base image
* **Memory Safety**: Arc + RwLock ensures concurrent safety

## Development

```bash
# Development build
cargo run

# Production build
cargo build --release

# Run tests
cargo test
```

## License

MIT License © AptS:1547
