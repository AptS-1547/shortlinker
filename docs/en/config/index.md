# Environment Variable Configuration

Shortlinker is configured through environment variables, supporting both `.env` files and system environment variables.

## Configuration Methods

### .env File (Recommended)
Create a `.env` file in the project root directory:

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

### Command Line Specification
```bash
SERVER_PORT=3000 ./shortlinker
```

## Configuration Parameters

### Server Configuration

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `SERVER_HOST` | String | `127.0.0.1` | Listen address |
| `SERVER_PORT` | Integer | `8080` | Listen port |

### Feature Configuration

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `DEFAULT_URL` | String | `https://esap.cc/repo` | Root path redirect address |
| `RANDOM_CODE_LENGTH` | Integer | `6` | Random short code length |

### Admin API Configuration (v0.0.5+)

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `ADMIN_TOKEN` | String | *(empty string)* | Admin API authentication token, **Admin API disabled when empty** |
| `ADMIN_ROUTE_PREFIX` | String | `/admin` | Admin API route prefix |

**Important Notes**:
- By default, Admin API is **disabled** to ensure security
- Admin API is only enabled when `ADMIN_TOKEN` environment variable is set
- Accessing Admin routes without token returns 404 Not Found

### Storage Configuration (v0.1.0+)

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `STORAGE_BACKEND` | String | `sqlite` | Storage backend type (`sqlite`, `file`, or `sled`), multiple backends supported since v0.1.0+ |
| `DB_FILE_NAME` | String | `links.db` (SQLite), `links.json` (File), `links.sled` (Sled) | Database file path (varies by backend) |

**Version Notes**:
- **v0.1.0+**: Supports multiple storage backends, SQLite as default
- **< v0.1.0**: Only supports file storage, no need to configure `STORAGE_BACKEND`

### Log Configuration

| Parameter | Type | Default | Options |
|-----------|------|---------|---------|
| `RUST_LOG` | String | `info` | `error`, `warn`, `info`, `debug`, `trace` |

## Configuration Priority

1. **Command-line environment variables** (highest)
2. **System environment variables**
3. **`.env` file**
4. **Program defaults** (lowest)

## Configuration Validation

Current configuration is displayed at startup:

```bash
[INFO] Starting server at http://127.0.0.1:8080
[INFO] Admin API is disabled (ADMIN_TOKEN not set)
# or
[INFO] Admin API available at: /admin/link
```

## Common Configuration Scenarios

### Development Environment
```bash
SERVER_HOST=127.0.0.1
SERVER_PORT=8080
RUST_LOG=debug
RANDOM_CODE_LENGTH=4

# Storage configuration - file storage for easy debugging
STORAGE_BACKEND=file
DB_FILE_NAME=dev-links.json

# Enable Admin API (development environment)
ADMIN_TOKEN=dev_token_123
```

### Production Environment
```bash
SERVER_HOST=127.0.0.1  # Access through reverse proxy
SERVER_PORT=8080
RUST_LOG=info
RANDOM_CODE_LENGTH=8

# Storage configuration - SQLite recommended for production (v0.1.0+)
STORAGE_BACKEND=sqlite
DB_FILE_NAME=/data/links.db

# Production environment strongly recommends strong password
ADMIN_TOKEN=very_secure_production_token_456
```

### Docker Environment
```bash
SERVER_HOST=0.0.0.0
SERVER_PORT=8080

# SQLite storage (recommended, v0.1.0+)
STORAGE_BACKEND=sqlite
DB_FILE_NAME=/data/links.db

# Or file storage (compatible with older versions)
# STORAGE_BACKEND=file
# DB_FILE_NAME=/data/links.json

# Optional: Enable Admin API
ADMIN_TOKEN=docker_admin_token_789
```

### Version Compatibility Configuration

#### v0.1.0+ Configuration
```bash
# Explicitly specify storage type (recommended)
STORAGE_BACKEND=sqlite
DB_FILE_NAME=data/links.db
```

#### v0.0.x Compatibility Configuration
```bash
# When upgrading from older versions, continue using file storage
STORAGE_BACKEND=file
DB_FILE_NAME=links.json
```

## Configuration Updates

### Hot Reload Support
- ✅ Storage file content changes
- ❌ Server address and port
- ❌ Log level
- ❌ Admin API configuration (requires server restart)

### Reload Methods
```bash
# Unix systems send SIGHUP signal
kill -HUP $(cat shortlinker.pid)

# Windows systems automatically monitor file changes
```

## Next Steps

- 📋 Check [Configuration Examples](/en/config/examples) for different scenario configurations
- 🚀 Learn [Deployment Configuration](/en/deployment/) for production environment setup
- 🛡️ Understand [Admin API](/en/api/admin) management interface usage
