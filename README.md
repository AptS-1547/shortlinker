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
* 💾 **Multiple Storage Backends**: SQLite database, JSON file storage, with Sled embedded database coming soon
* 🔄 **Cross-Platform**: Works on Windows, Linux, and macOS
* 🔐 **Process Management**: Smart process locking to prevent duplicate instances
* 🐳 **Containerized**: Optimized Docker image for easy deployment
* 🛡️ **Admin API**: HTTP API for link management (v0.0.5+)
* 🧪 **High Test Coverage**: Comprehensive unit and integration test coverage
* 🔧 **Strong Type Safety**: Complete error handling and type system
* 🎨 **Colorized Output**: Beautiful command-line interface with color support

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
./shortlinker update github https://new-github.com    # Update existing link
./shortlinker list                    # List all links
./shortlinker remove github           # Remove specific link

# Server control
./shortlinker start                   # Start server
./shortlinker stop                    # Stop server
./shortlinker restart                 # Restart server
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

### API Endpoints

#### GET /admin/link
Get all short links.

```bash
curl -H "Authorization: Bearer your_secret_token" \
     http://localhost:8080/admin/link
```

#### POST /admin/link
Create a new short link.

```bash
curl -X POST \
     -H "Authorization: Bearer your_secret_token" \
     -H "Content-Type: application/json" \
     -d '{"code":"github","target":"https://github.com","expires_at":"2024-12-31T23:59:59Z"}' \
     http://localhost:8080/admin/link
```

#### GET /admin/link/{code}
Get a specific short link.

```bash
curl -H "Authorization: Bearer your_secret_token" \
     http://localhost:8080/admin/link/github
```

#### PUT /admin/link/{code}
Update an existing short link.

```bash
curl -X PUT \
     -H "Authorization: Bearer your_secret_token" \
     -H "Content-Type: application/json" \
     -d '{"target":"https://new-github.com","expires_at":"2025-01-31T23:59:59Z"}' \
     http://localhost:8080/admin/link/github
```

#### DELETE /admin/link/{code}
Delete a short link.

```bash
curl -X DELETE \
     -H "Authorization: Bearer your_secret_token" \
     http://localhost:8080/admin/link/github
```

### Common Operations

```bash
# Get All Links
curl -H "Authorization: Bearer your_secret_token" \
     http://localhost:8080/admin/link

# Create Link with auto-generated code
curl -X POST \
     -H "Authorization: Bearer your_secret_token" \
     -H "Content-Type: application/json" \
     -d '{"target":"https://github.com"}' \
     http://localhost:8080/admin/link

# Update Link
curl -X PUT \
     -H "Authorization: Bearer your_secret_token" \
     -H "Content-Type: application/json" \
     -d '{"target":"https://new-url.com"}' \
     http://localhost:8080/admin/link/github

# Delete Link
curl -X DELETE \
     -H "Authorization: Bearer your_secret_token" \
     http://localhost:8080/admin/link/github
```

## Configuration Options

Configure using environment variables or a `.env` file:

| Environment Variable | Default Value | Description        |
| -------------------- | ------------- | ------------------ |
| `SERVER_HOST`        | `127.0.0.1`   | Listen address     |
| `SERVER_PORT`        | `8080`        | Listen port        |
| `STORAGE_BACKEND`    | `sqlite`      | Storage backend type |
| `DB_FILE_NAME`       | `links.db`    | Database file path (common for all backends) |
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
STORAGE_BACKEND=sqlite
DB_FILE_NAME=data/links.db

# Feature configuration
DEFAULT_URL=https://example.com
RANDOM_CODE_LENGTH=8
RUST_LOG=info

# Admin API configuration
ADMIN_TOKEN=your_secure_admin_token
```

## Storage Backends

shortlinker supports multiple storage backends starting from v0.1.0:

- **SQLite** (default, v0.1.0+): Production-grade performance, recommended for production
- **File Storage** (default before v0.1.0): Simple and easy to use, convenient for debugging
- **Sled** (coming soon): High concurrency performance, suitable for high-load scenarios

```bash
# SQLite storage (default, v0.1.0+)
STORAGE_BACKEND=sqlite
DB_FILE_NAME=links.db

# File storage (default before v0.1.0)
STORAGE_BACKEND=file
DB_FILE_NAME=links.json

# Sled storage (coming soon)
# STORAGE_BACKEND=sled
# DB_FILE_NAME=links.sled
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

## Code Quality & Testing

The shortlinker project emphasizes code quality and reliability:

### Test Coverage

- **CLI Module Tests**: Command parsing, argument validation, error handling
- **Storage Layer Tests**: File storage, SQLite, Sled multi-backend testing
- **Service Layer Tests**: Admin API, authentication middleware, HTTP handling
- **Utilities Tests**: Random code generation, color output, utility functions
- **Error Handling Tests**: Complete error types and conversion testing
- **System Integration Tests**: Process management, signal handling, concurrency safety
- **Performance Tests**: Large dataset handling, concurrent operations, memory usage

### Running Tests

```bash
# Run all tests
cargo test

# Run specific module tests
cargo test cli_tests
cargo test storages_tests
cargo test services_tests
cargo test utils_tests
cargo test errors_tests

# Show test coverage
cargo test --verbose

# Parallel testing (faster)
cargo test -- --test-threads=4
```

### Code Quality Features

- **Type Safety**: Strict Rust type system with compile-time error checking
- **Memory Safety**: Zero-cost abstractions without GC, preventing memory leaks
- **Concurrency Safety**: Arc + Mutex/RwLock ensures thread safety
- **Error Handling**: Unified error types and propagation mechanisms
- **Modular Design**: Clear module boundaries and separation of concerns
- **Complete Documentation**: Detailed code comments and API documentation

## Technical Details

* **Hot Reloading**: Automatic configuration file change detection
* **Random Code Generation**: Alphanumeric with configurable length, collision avoidance
* **Expiration Checking**: Real-time validation on request, automatic cleanup
* **Container Optimization**: Multi-stage build with `scratch` base image
* **Memory Safety**: Arc + RwLock ensures concurrent safety
* **Colorized Terminal**: Beautiful output with ANSI color code support
* **Smart Retry**: Automatic retry mechanisms for network and storage operations
* **Graceful Shutdown**: Signal handling and resource cleanup

## Development

```bash
# Development build
cargo run

# Production build
cargo build --release

# Run tests
cargo test

# Code formatting
cargo fmt

# Code linting
cargo clippy

# Generate documentation
cargo doc --open
```

### Development Guidelines

1. **Adding New Features**: Ensure corresponding unit tests are written
2. **Modifying Storage Layer**: Update implementations for all storage backends
3. **API Changes**: Update Admin API tests and documentation
4. **Error Handling**: Use the unified `ShortlinkerError` type
5. **Logging Output**: Use the `log` crate for structured logging

## License

MIT License © AptS:1547
