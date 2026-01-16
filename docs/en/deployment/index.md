# Deployment Guide

Shortlinker supports multiple deployment methods, from simple local running to production containerized deployment.

## Deployment Overview

### 🚀 Quick Deployment
- **Docker Deployment**: Recommended production solution, no Rust installation required
- **Pre-compiled Binaries**: Download and run, supports multiple platforms
- **Source Compilation**: Requires Rust 1.85+ (Edition 2024), suitable for custom needs

### 🔧 Production Environment
- **Reverse Proxy**: Nginx, Caddy, Apache configuration
- **System Service**: systemd, Docker Compose management
- **Monitoring & Alerting**: Health checks and log management

## System Requirements

### System Requirements
- **Operating System**: Linux, macOS, Windows
- **Architecture**: x86_64, ARM64

### Source Compilation Requirements
- **Rust**: >= 1.85.0 (required, Edition 2024)
- **Git**: For cloning the project

## Quick Start

### Docker Deployment (Recommended)
```bash
# Quick startup
docker run -d -p 8080:8080 -v $(pwd)/data:/data e1saps/shortlinker
```

### Pre-compiled Binaries
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
    ↓             ↓                ↓                   ↓
  Browser        Nginx           Docker             SQLite(default)
  curl           Caddy           systemd            MariaDB
  API            Apache          Binary             MySQL/PostgreSQL
```

## Security Recommendations

1. **Network Security**: Expose service through reverse proxy
2. **File Permissions**: Set appropriate data file permissions
3. **Process Management**: Use system service managers
4. **Data Backup**: Regular backup of link data (SQLite can directly backup .db files)

## Performance Characteristics

- **Response Time**: < 1ms (SQLite local storage)
- **Concurrency Support**: Thousands of concurrent connections
- **Memory Usage**: Extremely low memory footprint
- **Storage Format**: SQLite database (default), supports MySQL, PostgreSQL, MariaDB

## Next Steps

Choose the deployment method that suits you:

- 📦 [Docker Deployment](/en/deployment/docker) - Detailed containerized deployment guide
- 🔀 [Reverse Proxy](/en/deployment/proxy) - Nginx, Caddy configuration
- ⚙️ [System Service](/en/deployment/systemd) - systemd and process management

Need configuration help? Check [Configuration Guide](/en/config/) for environment variable settings.
