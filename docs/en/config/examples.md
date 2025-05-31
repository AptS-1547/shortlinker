# Configuration Examples

Complete configuration examples for various environments.

## Development Environment

```bash
# Server configuration
SERVER_HOST=127.0.0.1
SERVER_PORT=8080

# Feature configuration
DEFAULT_URL=https://localhost:3000
RANDOM_CODE_LENGTH=4  # Shorter for testing convenience

# Storage configuration - file storage recommended for development debugging
STORAGE_BACKEND=file
DB_FILE_NAME=./dev-links.json

# Logging configuration
RUST_LOG=debug  # Enable verbose logging for development

# Admin API (development environment)
ADMIN_TOKEN=dev_token_123
```

## Production Environment

```bash
# Server configuration
SERVER_HOST=127.0.0.1  # Expose through reverse proxy
SERVER_PORT=8080

# Storage configuration - SQLite recommended for production
STORAGE_BACKEND=sqlite
DB_FILE_NAME=/data/links.db

# Feature configuration
DEFAULT_URL=https://your-company.com
RANDOM_CODE_LENGTH=8  # Longer codes recommended for production

# Security configuration
ADMIN_TOKEN=very_secure_production_token_456
RUST_LOG=info
```

## Docker Environment

### Docker Compose

```yaml
version: '3.8'
services:
  shortlinker:
    image: e1saps/shortlinker
    environment:
      - SERVER_HOST=0.0.0.0
      - SERVER_PORT=8080
      - STORAGE_BACKEND=sqlite
      - DB_FILE_NAME=/data/links.db
      - DEFAULT_URL=https://your-domain.com
      - RANDOM_CODE_LENGTH=8
      - RUST_LOG=info
    volumes:
      - ./data:/data
    ports:
      - "127.0.0.1:8080:8080"
```

### Environment Variables File

```bash
# .env
SERVER_HOST=0.0.0.0
SERVER_PORT=8080
STORAGE_BACKEND=sqlite
DB_FILE_NAME=/data/links.db
DEFAULT_URL=https://your-site.com
RANDOM_CODE_LENGTH=8
RUST_LOG=info
```

## Cloud Service Environment

### General Cloud Configuration

```bash
# Server configuration
SERVER_HOST=0.0.0.0
SERVER_PORT=8080

# Storage configuration (using persistent storage)
STORAGE_BACKEND=sqlite
DB_FILE_NAME=/mnt/persistent/links.db

# Feature configuration
DEFAULT_URL=https://your-cloud-site.com
RANDOM_CODE_LENGTH=8
RUST_LOG=info
```

## High Concurrency Scenarios

```bash
# Server configuration
SERVER_HOST=0.0.0.0
SERVER_PORT=8080

# Storage optimization - use SQLite or Sled
STORAGE_BACKEND=sqlite
DB_FILE_NAME=/fast-ssd/links.db

# Performance optimization
RANDOM_CODE_LENGTH=6  # Balance performance and uniqueness
DEFAULT_URL=https://cdn.example.com

# Logging optimization
RUST_LOG=error  # Only log errors to reduce I/O
```

## systemd Service Configuration

```ini
[Unit]
Description=Shortlinker Service
After=network.target

[Service]
Type=simple
User=www-data
WorkingDirectory=/opt/shortlinker
ExecStart=/opt/shortlinker/shortlinker

# Environment variables
Environment=SERVER_HOST=127.0.0.1
Environment=SERVER_PORT=8080
Environment=STORAGE_BACKEND=sqlite
Environment=DB_FILE_NAME=/opt/shortlinker/data/links.db
Environment=DEFAULT_URL=https://example.com
Environment=RANDOM_CODE_LENGTH=8
Environment=RUST_LOG=info

# Security configuration
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ReadWritePaths=/opt/shortlinker/data

[Install]
WantedBy=multi-user.target
```

## Configuration Validation

### Quick Validation Script

```bash
#!/bin/bash
# validate-config.sh
echo "Validating Shortlinker configuration..."

# Check port availability
if netstat -tuln | grep -q ":${SERVER_PORT:-8080} "; then
    echo "Error: Port ${SERVER_PORT:-8080} is already in use"
    exit 1
fi

# Check storage directory permissions
STORAGE_DIR=$(dirname "${DB_FILE_NAME:-links.db}")
if [ ! -w "$STORAGE_DIR" ]; then
    echo "Error: Storage directory $STORAGE_DIR has no write permission"
    exit 1
fi

echo "Configuration validation passed ✓"
```

### Test Configuration

```bash
# Test startup
./shortlinker --version

# Test service response
curl -I http://localhost:8080/
```
