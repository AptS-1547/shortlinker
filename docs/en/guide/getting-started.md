# Quick Start

This guide helps you configure and use Shortlinker in 5 minutes.

## Prerequisites

Please complete any installation method from the [Installation Guide](/en/guide/installation) first.

## Step 1: Basic Configuration

### Method 1: Using TOML Configuration File (Recommended)

Use the `generate-config` command to generate a configuration file:

```bash
./shortlinker generate-config config.toml
# Generates a startup config template (server/database/cache/logging/analytics)
```

Then modify `config.toml` as needed:

```toml
[server]
host = "127.0.0.1"
port = 8080

[database]
database_url = "shortlinks.db"

[logging]
level = "info"
```

::: tip
If you don't create a configuration file, the program will run with built-in default settings.
:::

> Note: Runtime config (e.g. `features.default_url`, `api.health_token`, `features.enable_admin_panel`) is stored in the database and should be changed via Admin API or CLI; the current version does not read these from `config.toml` or environment variables.

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

# Reload short link data / caches (Unix systems)
# Note: SIGUSR1 only reloads link data/caches; it does NOT reload runtime config.
# Reload runtime config via Admin API `/admin/v1/config/reload` or restart the service.
kill -USR1 $(cat shortlinker.pid)
```

## Production Environment Quick Configuration

### Recommended Configuration
```toml
# config.toml (production)
[server]
host = "127.0.0.1"
port = 8080

[database]
database_url = "sqlite:///data/links.db"

[logging]
level = "info"
```

Runtime config (stored in the DB) can be updated via CLI/Admin API, for example:

```bash
# Root default redirect (no restart)
./shortlinker config set features.default_url https://your-domain.com

# Health Bearer token (no restart)
./shortlinker config set api.health_token your_health_token

# Rotate admin password (recommended)
./shortlinker reset-password
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
      - ./config.toml:/config.toml:ro
      - ./data:/data
```

## Next Steps

Congratulations! You have successfully configured Shortlinker. Next you can:

- 📋 Learn [CLI Command Details](/en/cli/commands) - Master all command options
- 🚀 Check [Deployment Guide](/en/deployment/) - Production environment deployment
- ⚙️ Learn [Configuration Options](/en/config/) - Customize advanced settings
- 🛡️ Use [Admin API](/en/api/admin) - HTTP interface management
- 🏥 Configure [Health Check](/en/api/health) - Service monitoring
