# Quick Start

This guide helps you configure and use Shortlinker in 5 minutes.

## Prerequisites

Please complete any installation method from the [Installation Guide](/en/guide/installation) first.

## Step 1: Basic Configuration

### Method 1: Using TOML Configuration File (Recommended)

Create `config.toml` file:

```toml
[server]
host = "127.0.0.1"
port = 8080

[features]
default_url = "https://example.com"

# Optional: Enable admin and monitoring features
# [api]
# admin_token = "your_admin_token"
# health_token = "your_health_token"
```

Or use a custom path:

```bash
# Use -c or --config to specify config file path
./shortlinker -c /etc/shortlinker/myconfig.toml

# If the file doesn't exist, it will be created automatically with defaults
./shortlinker -c ./custom.toml
# [INFO] Configuration file not found: ./custom.toml
# [INFO] Creating default configuration file...
```

### Method 2: Using Environment Variables

Create configuration file `.env`:

```bash
# Minimal configuration
SERVER_HOST=127.0.0.1
SERVER_PORT=8080
DEFAULT_URL=https://example.com

# Optional: Enable admin and monitoring features
# ADMIN_TOKEN=your_admin_token
# HEALTH_TOKEN=your_health_token
```

## Step 2: Start Service

```bash
# Start server
./shortlinker

# Success output:
# [INFO] Starting server at http://127.0.0.1:8080
# [INFO] SQLite storage initialized with 0 links
```

## Step 3: Add Short Links

```bash
# Custom short code
./shortlinker add github https://github.com

# Random short code
./shortlinker add https://www.google.com
# Output: ✓ Added short link: aB3dF1 -> https://www.google.com
```

## Step 4: Test Access

```bash
# Test redirect
curl -I http://localhost:8080/github
# HTTP/1.1 307 Temporary Redirect
# Location: https://github.com

# Browser access
# http://localhost:8080/github
```

## Common Operations

```bash
# View all short links
./shortlinker list

# Delete short link
./shortlinker remove github

# Add temporary link
./shortlinker add temp https://example.com --expire 1d

# Force overwrite
./shortlinker add github https://github.com --force
```

## Service Management

```bash
# Stop service
# Method 1: Ctrl+C
# Method 2: Send signal
kill $(cat shortlinker.pid)

# Reload config (Unix systems)
kill -USR1 $(cat shortlinker.pid)
```

## Production Environment Quick Configuration

### Recommended Configuration
```bash
# Production .env configuration
SERVER_HOST=127.0.0.1
SERVER_PORT=8080
STORAGE_BACKEND=sqlite
DB_FILE_NAME=/data/links.db
DEFAULT_URL=https://your-domain.com

# Enable API features
ADMIN_TOKEN=your_secure_admin_token
HEALTH_TOKEN=your_secure_health_token
```

### Reverse Proxy Example
```nginx
# Nginx configuration example
server {
    listen 80;
    server_name your-domain.com;
    location / {
        proxy_pass http://127.0.0.1:8080;
        proxy_set_header Host $host;
    }
}
```

### Docker Quick Deployment
```bash
# Using Docker Compose
version: '3.8'
services:
  shortlinker:
    image: e1saps/shortlinker
    ports:
      - "127.0.0.1:8080:8080"
    volumes:
      - ./data:/data
    environment:
      - STORAGE_BACKEND=sqlite
      - DB_FILE_NAME=/data/links.db
```

## Next Steps

Congratulations! You have successfully configured Shortlinker. Next you can:

- 📋 Learn [CLI Command Details](/en/cli/commands) - Master all command options
- 🚀 Check [Deployment Guide](/en/deployment/) - Production environment deployment
- ⚙️ Learn [Configuration Options](/en/config/) - Customize advanced settings
- 🛡️ Use [Admin API](/en/api/admin) - HTTP interface management
- 🏥 Configure [Health Check](/en/api/health) - Service monitoring
