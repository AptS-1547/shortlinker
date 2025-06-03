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

</div>

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

Configure using environment variables or `.env` file:

| Variable | Default | Description |
|----------|---------|-------------|
| `SERVER_HOST` | `127.0.0.1` | Listen address |
| `SERVER_PORT` | `8080` | Listen port |
| `UNIX_SOCKET` | *(empty)* | Unix socket path (overrides HOST/PORT) |
| `STORAGE_BACKEND` | `sqlite` | Storage type (sqlite/file) |
| `DB_FILE_NAME` | `links.db` | Database file path |
| `DEFAULT_URL` | `https://esap.cc/repo` | Default redirect URL |
| `RANDOM_CODE_LENGTH` | `6` | Random code length |
| `ADMIN_TOKEN` | *(empty)* | Admin API token |
| `HEALTH_TOKEN` | *(empty)* | Health API token |
| `RUST_LOG` | `info` | Log level |

### .env Example

```bash
# Server - TCP
SERVER_HOST=0.0.0.0
SERVER_PORT=8080

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
- **File Storage**: Simple JSON-based storage for development

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

## Technical Highlights

- **Cross-Platform Process Management**: Smart lock files and signal handling
- **Hot Configuration Reload**: Signal-based reload (Unix) and file triggers (Windows)
- **Container-Aware**: Special handling for Docker environments
- **Unified Error Handling**: Comprehensive error types with automatic conversions
- **Memory Safe**: Zero-cost abstractions with thread safety
- **High Test Coverage**: Comprehensive unit and integration tests

## License

MIT License © AptS:1547
