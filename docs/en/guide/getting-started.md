# Quick Start

This guide helps you complete Shortlinker configuration and basic usage in 5 minutes.

## Prerequisites

Please first complete any installation method from the [Installation Guide](/en/guide/installation).

## Step 1: Basic Configuration

Create configuration file `.env`:

```bash
# Minimal configuration
SERVER_HOST=127.0.0.1
SERVER_PORT=8080
DEFAULT_URL=https://example.com

# Storage configuration (v0.1.0+, optional)
STORAGE_BACKEND=sqlite
DB_FILE_NAME=links.db
```

## Step 2: Start Service

```bash
# Start server
./shortlinker

# You should see output like:
# [INFO] Starting server at http://127.0.0.1:8080
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

### View All Short Links
```bash
./shortlinker list
```

### Delete Short Link
```bash
./shortlinker remove github
```

### Add Temporary Link
```bash
./shortlinker add temp https://example.com --expire 2024-12-31T23:59:59Z
```

### Force Overwrite
```bash
./shortlinker add github https://github.com --force
```

## Service Management

### Stop Service
```bash
# Method 1: Ctrl+C
# Method 2: Send signal
kill $(cat shortlinker.pid)
```

### Reload Configuration
```bash
# Unix systems
kill -HUP $(cat shortlinker.pid)
```

## Production Environment Recommendations

### Reverse Proxy
It's recommended to use Nginx or Caddy as reverse proxy:

```nginx
# Nginx configuration example
server {
    listen 80;
    server_name your-domain.com;
    location / {
        proxy_pass http://127.0.0.1:8080;
    }
}
```

### System Service
Use systemd to manage service:

```bash
# Install as system service
sudo cp shortlinker.service /etc/systemd/system/
sudo systemctl enable shortlinker
sudo systemctl start shortlinker
```

## Next Steps

Congratulations! You have successfully configured Shortlinker. Next you can:

- 📋 Learn [CLI Command Details](/en/cli/commands)
- 🚀 Check [Deployment Guide](/en/deployment/) for production deployment
- ⚙️ Understand [Advanced Configuration](/en/config/examples)
