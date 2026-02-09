---
layout: home

hero:
  name: "Shortlinker"
  text: "Minimalist URL Shortening Service"
  tagline: "Supports HTTP 307 redirects, built with Rust, easy deployment and fast response"
  image:
    src: /logo.svg
    alt: Shortlinker
  actions:
    - theme: brand
      text: Quick Start
      link: /en/guide/getting-started
    - theme: alt
      text: View on GitHub
      link: https://github.com/AptS-1547/shortlinker

features:
  - icon: 🚀
    title: High Performance
    details: Built with Rust, millisecond-level redirect response, supports high concurrency access
  - icon: 💾
    title: Multiple Storage Backends
    details: Supports SQLite (default), PostgreSQL, MySQL, MariaDB for production-grade database performance
  - icon: 🛡️
    title: Secure & Reliable
    details: Admin API authentication, health check monitoring, process protection mechanism
  - icon: 🐳
    title: Easy Deployment
    details: One-click Docker deployment, supports systemd service management
  - icon: ⚡
    title: Hot Reload
    details: Add/remove short links at runtime without restarting the server
  - icon: 🎯
    title: Smart Management
    details: Custom short codes, random generation, expiration time, CLI tool management, TUI interface
---

## Design Philosophy

### Minimalism
Focus on the core functionality of short link redirection with simple configuration and fast deployment

### High Performance
Native Rust performance guarantee, SQLite provides production-grade database performance, asynchronous concurrent processing

### Easy to Use
Command line tool management, TOML startup config + DB runtime config, one-click Docker deployment

## Core Features

- **Multiple Storage Backends**: SQLite database (default), PostgreSQL, MySQL, MariaDB for production-grade performance
- **Admin API**: HTTP API management interface with authentication and custom route prefix support
- **Health Monitoring**: Complete health check API with storage status and runtime monitoring
- **Smart Expiration**: Supports flexible time format settings, automatic expiration and cleanup
- **Cross-platform Support**: Windows, Linux, macOS, smart process locking to prevent duplicate startup
- **Container Optimization**: Docker image deployment with container restart detection support
- **TUI Interface**: Terminal user interface for interactive management and monitoring
- **TOML Configuration**: Startup config (server/database/cache/logging/analytics/ipc) + DB runtime config (auth/routes/features, etc.)

## Quick Experience

```bash
# Docker quick startup (mount config.toml; ensure container-side server.host=0.0.0.0)
docker run -d \
  -p 8080:8080 \
  -v $(pwd)/config.toml:/config.toml:ro \
  -v $(pwd)/data:/data \
  e1saps/shortlinker

# Add short link
./shortlinker add github https://github.com

# Access short link
curl -L http://localhost:8080/github

# Launch TUI interface (if compiled with TUI feature)
./shortlinker tui
```

## Use Cases

- **Marketing Promotion**: Event links, social media sharing
- **Internal Tools**: Document redirection, system integration
- **Temporary Links**: Time-limited sharing, test environment
- **API Services**: Link management between microservices

## Get Started

Ready to go? Check out the [Quick Start Guide](/en/guide/getting-started) to begin with Shortlinker
More modules: [Web Admin Panel](/en/admin-panel/) | [Cloudflare Worker Version](/en/cf-worker/).
