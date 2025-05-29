# Deployment Guide

Shortlinker supports multiple deployment methods, from simple local running to production containerized deployment.

## Deployment Methods Overview

### 🚀 Quick Deployment
- **Docker Deployment**: Recommended production solution, no Rust installation required
- **Precompiled Binary**: Download and run, multi-platform support
- **Source Compilation**: Requires Rust 1.82+, suitable for customization needs

### 🔧 Production Environment
- **Reverse Proxy**: Nginx, Caddy, Apache configuration
- **System Service**: systemd, Docker Compose management
- **Monitoring & Alerting**: Health checks and log management

## Environment Requirements

### System Requirements
- **Operating System**: Linux, macOS, Windows
- **Architecture**: x86_64, ARM64

### Source Compilation Requirements
- **Rust**: >= 1.82.0 (required)
- **Git**: For cloning the project

## Quick Start

### Docker Deployment (Recommended)
```bash
# Quick startup
docker run -d -p 8080:8080 -v $(pwd)/data:/data e1saps/shortlinker
```

### Precompiled Binary
```bash
# Download and run
wget https://github.com/AptS-1547/shortlinker/releases/latest/download/shortlinker-linux-x64.tar.gz
tar -xzf shortlinker-linux-x64.tar.gz
./shortlinker
```

### Source Compilation
```bash
# Clone and compile
git clone https://github.com/AptS-1547/shortlinker
cd shortlinker
cargo build --release
./target/release/shortlinker
```

## Deployment Architecture

```
User Request → Reverse Proxy → Shortlinker Service → Data Storage
    ↓              ↓                   ↓                ↓
  Browser        Nginx              Docker          JSON File
  curl           Caddy              systemd      
  API            Apache             Binary
```

## Security Recommendations

1. **Network Security**: Expose service through reverse proxy
2. **File Permissions**: Set appropriate data file permissions
3. **Process Management**: Use system service managers
4. **Data Backup**: Regular backup of link data
5. **Admin API Security**: Use strong tokens and HTTPS

## Performance Characteristics

- **Response Time**: < 1ms (local storage)
- **Concurrency Support**: Thousands of concurrent connections
- **Memory Usage**: Extremely low memory footprint
- **Storage Format**: JSON files with hot reload support

## Admin API Security (v0.0.5+)

### Security Best Practices

```bash
# Use strong Admin API token
ADMIN_TOKEN=very_long_secure_random_token_at_least_32_characters

# Custom route prefix to avoid scanning
ADMIN_ROUTE_PREFIX=/my-secret-admin-path

# Always use HTTPS in production
# Never expose Admin API on public networks without proper authentication
```

### Admin API Configuration Examples

```bash
# Development (less secure, more convenient)
ADMIN_TOKEN=dev_token_123
ADMIN_ROUTE_PREFIX=/admin

# Production (highly secure)
ADMIN_TOKEN=prod_$(openssl rand -hex 32)
ADMIN_ROUTE_PREFIX=/management-$(openssl rand -hex 8)
```

## Next Steps

Choose the deployment method that suits you:

- 📦 [Docker Deployment](/en/deployment/docker) - Detailed containerized deployment guide
- 🔀 [Reverse Proxy](/en/deployment/proxy) - Nginx, Caddy configuration
- ⚙️ [System Service](/en/deployment/systemd) - systemd and process management

Need configuration help? Check [Configuration Guide](/en/config/) to understand environment variable settings.
