# Shortlinker Introduction

Shortlinker is a minimalist URL shortening service built with Rust, focusing on providing high-performance HTTP 302 redirect functionality.

## Design Philosophy

### Minimalism
- Single function: Focus on short link redirection
- Zero dependencies: No database required, no complex configuration
- Lightweight: Minimal resource usage

### High Performance
- Rust native performance guarantee
- Memory-mapped storage access
- Asynchronous concurrent processing

### Easy to Use
- Command-line tool management
- Environment variable configuration
- One-click Docker deployment

## Core Features

### 🔄 Hot Reload Mechanism
Configuration and data files support runtime reload without server restart.

### ⏰ Smart Expiration
- Automatic detection of expired links
- Support for RFC3339 time format
- Real-time validation on access

### 🛡️ Process Protection
- Prevent duplicate startup
- Cross-platform locking mechanism
- Graceful shutdown handling

### 📊 Simple Monitoring
- Structured log output
- Performance metrics statistics
- Health status checks

### 🛡️ Admin API (v0.0.5+)
- Complete CRUD operations for short links
- Bearer token authentication
- Customizable route prefix
- Disabled by default for security

## Technical Architecture

```
┌─────────────┐    ┌──────────────┐    ┌─────────────┐
│ User Request │───▶│  HTTP Server │───▶│Storage Engine│
└─────────────┘    └──────────────┘    └─────────────┘
                          │                     │
                          ▼                     ▼
┌─────────────┐    ┌──────────────┐    ┌─────────────┐
│  CLI Tools  │───▶│Management API│───▶│ JSON Files  │
└─────────────┘    └──────────────┘    └─────────────┘
```

## Use Cases

- **Marketing Promotion**: Event links, social media sharing
- **Internal Tools**: Document jumping, system integration
- **Temporary Links**: Time-limited sharing, test environments
- **API Services**: Link management between microservices

## Next Steps

- 📚 [Installation Guide](/en/guide/installation) - Learn installation requirements and methods
- 🚀 [Quick Start](/en/guide/getting-started) - Get up and running in 5 minutes
- ⚙️ [Configuration](/en/config/) - Understand all configuration options
