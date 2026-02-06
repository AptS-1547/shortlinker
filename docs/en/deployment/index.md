# Deployment Guide

Shortlinker supports multiple deployment methods, from simple local running to production containerized deployment.

## Suggested Reading Order

1. [Docker Deployment Overview](/en/deployment/docker)
2. [Docker Quick Start and Compose](/en/deployment/docker-quickstart)
3. [Reverse Proxy Overview](/en/deployment/proxy)
4. [Systemd Service Overview](/en/deployment/systemd)

For production operations details:

- [Docker Operations and Security](/en/deployment/docker-operations)
- [Reverse Proxy Performance and Monitoring](/en/deployment/proxy-operations)
- [Systemd Docker Compose and Operations](/en/deployment/systemd-operations)

## Deployment Options at a Glance

| Option | Best For | Recommendation |
|--------|----------|----------------|
| Docker | Most production environments | ⭐⭐⭐⭐⭐ |
| Prebuilt binary | Fast local validation / lightweight deploy | ⭐⭐⭐⭐ |
| Source build | Custom build features | ⭐⭐⭐ |

## Prerequisites

- **OS**: Linux, macOS, Windows
- **Architecture**: x86_64, ARM64
- **For source builds**: Rust `>= 1.88.0` (Edition 2024), Git

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

- 📦 [Docker Deployment Overview](/en/deployment/docker)
- ⚡ [Docker Quick Start and Compose](/en/deployment/docker-quickstart)
- 🛠️ [Docker Operations and Security](/en/deployment/docker-operations)
- 🔀 [Reverse Proxy Overview](/en/deployment/proxy)
- 📈 [Reverse Proxy Performance and Monitoring](/en/deployment/proxy-operations)
- ⚙️ [Systemd Service Overview](/en/deployment/systemd)
- 🔧 [Systemd Docker Compose and Operations](/en/deployment/systemd-operations)

Need configuration help? Check [Configuration Guide](/en/config/) for `config.toml` (startup config) and DB-backed runtime config.
