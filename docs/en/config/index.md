# Environment Variables Configuration

Shortlinker is configured through environment variables, supporting both `.env` files and system environment variables.

## Configuration Methods

### .env File (Recommended)
```bash
# .env
SERVER_HOST=127.0.0.1
SERVER_PORT=8080
DEFAULT_URL=https://example.com
```

### System Environment Variables
```bash
export SERVER_HOST=0.0.0.0
export SERVER_PORT=8080
./shortlinker
```

## Configuration Parameters

### Server Configuration

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `SERVER_HOST` | String | `127.0.0.1` | Listening address |
| `SERVER_PORT` | Integer | `8080` | Listening port |
| `UNIX_SOCKET` | String | *(empty)* | Unix socket path (overrides HOST/PORT) |
| `CPU_COUNT` | Integer | *(auto)* | Worker thread count (defaults to CPU cores) |
| `DEFAULT_URL` | String | `https://esap.cc/repo` | Root path redirect URL |
| `RANDOM_CODE_LENGTH` | Integer | `6` | Random short code length |

### Storage Configuration

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `STORAGE_BACKEND` | String | `sqlite` | Storage type: `sqlite`, `file`, `sled` |
| `DB_FILE_NAME` | String | `links.db` | Database file path |

> For detailed storage backend configuration, see [Storage Backends](/en/config/storage)

### API Configuration

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `ADMIN_TOKEN` | String | *(empty)* | Admin API auth token, **disabled when empty** |
| `ADMIN_ROUTE_PREFIX` | String | `/admin` | Admin API route prefix |
| `HEALTH_TOKEN` | String | *(empty)* | Health check API auth token, **disabled when empty** |
| `HEALTH_ROUTE_PREFIX` | String | `/health` | Health check API route prefix |

> For detailed API configuration, see [Admin API](/en/api/admin) and [Health Check API](/en/api/health)

### Logging Configuration

| Parameter | Type | Default | Options |
|-----------|------|---------|---------|
| `RUST_LOG` | String | `info` | `error`, `warn`, `info`, `debug`, `trace` |

## Configuration Examples

### Development Environment
```bash
# Basic configuration
SERVER_HOST=127.0.0.1
SERVER_PORT=8080
RUST_LOG=debug

# Storage configuration - file storage for easy debugging
STORAGE_BACKEND=file
DB_FILE_NAME=dev-links.json

# API configuration - simple tokens for development
ADMIN_TOKEN=dev_admin
HEALTH_TOKEN=dev_health
```

### Production Environment
```bash
# Basic configuration
SERVER_HOST=127.0.0.1
SERVER_PORT=8080
CPU_COUNT=8
RUST_LOG=info
DEFAULT_URL=https://your-domain.com

# Storage configuration - SQLite for production performance
STORAGE_BACKEND=sqlite
DB_FILE_NAME=/data/links.db

# API configuration - use strong passwords
ADMIN_TOKEN=very_secure_production_token_456
HEALTH_TOKEN=very_secure_health_token_789
```

### Docker Environment
```bash
# Server configuration - TCP
SERVER_HOST=0.0.0.0
SERVER_PORT=8080
CPU_COUNT=4

# Server configuration - Unix socket
# UNIX_SOCKET=/tmp/shortlinker.sock

# Storage configuration
STORAGE_BACKEND=sqlite
DB_FILE_NAME=/data/links.db

# API configuration
ADMIN_TOKEN=docker_admin_token_123
HEALTH_TOKEN=docker_health_token_456
```

### Minimal Configuration (Redirect Only)
```bash
# Only provide redirect service, no management features
SERVER_HOST=127.0.0.1
SERVER_PORT=8080
# Don't set ADMIN_TOKEN and HEALTH_TOKEN
```

## API Access Control

| Scenario | ADMIN_TOKEN | HEALTH_TOKEN | Description |
|----------|-------------|--------------|-------------|
| **Service Only** | Not set | Not set | Most secure, redirect functionality only |
| **Service + Management** | Set | Not set | Enable management features |
| **Service + Monitoring** | Not set | Set | Enable monitoring features |
| **Full Features** | Set | Set | Enable all features |

## Configuration Priority

1. **Command line environment variables** (highest)
2. **System environment variables**
3. **`.env` file**
4. **Program defaults** (lowest)

## Configuration Validation

Configuration status will be displayed at startup:

```bash
[INFO] Starting server at http://127.0.0.1:8080
[INFO] SQLite storage initialized with 0 links
[INFO] Admin API available at: /admin
[INFO] Health API available at: /health
```

## Configuration Updates

### Hot Reload Support
- ✅ Storage file content changes
- ❌ Server address and port (requires restart)
- ❌ API configuration (requires restart)

### Reload Methods
```bash
# Unix systems
kill -USR1 $(cat shortlinker.pid)

# Windows systems
echo "" > shortlinker.reload
```

## Next Steps

- 📋 Check [Storage Backend Configuration](/en/config/storage) for detailed storage options
- 🚀 Learn [Deployment Configuration](/en/deployment/) for production environment setup
- 🛡️ Learn [Admin API](/en/api/admin) for management interface usage
- 🏥 Learn [Health Check API](/en/api/health) for monitoring interface usage
