---
layout: home

hero:
  name: "Shortlinker"
  text: "Minimalist URL Shortening Service"
  tagline: "Supports HTTP 302 redirects, built with Rust, easy to deploy and blazingly fast"
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
    details: Built with Rust + Actix-web, providing millisecond-level redirect response speed
  - icon: 🎯
    title: Dynamic Management
    details: Support runtime addition/deletion of short links without server restart
  - icon: 🎲
    title: Smart Short Codes
    details: Support custom short codes and random generation with intelligent conflict handling
  - icon: ⏰
    title: Expiration Time
    details: Support setting link expiration time with automatic invalidation and cleanup
  - icon: 💾
    title: Persistent Storage
    details: JSON file storage with hot reload and automatic configuration file detection
  - icon: 🔄
    title: Cross Platform
    details: Support Windows, Linux, macOS with intelligent process locks to prevent duplicate startup
  - icon: 🐳
    title: Containerization
    details: Optimized Docker image deployment with multi-stage builds and scratch base image
  - icon: 🛡️
    title: Admin API
    details: HTTP API management interface with authentication and custom route prefix (v0.0.5+)
---
## Why Choose Shortlinker

### 💡 Minimalist Design

Focus on core functionality, no extra features, simple configuration, fast deployment

### ⚡ Superior Performance

Rust native performance, millisecond response, supports high concurrent access

### 🛠️ Operations Friendly

Single binary, Docker support, systemd integration, comprehensive monitoring

## Quick Experience

### Three Deployment Methods

```bash
# 1. One-click Docker startup
docker run -d -p 8080:8080 e1saps/shortlinker

# 2. Download precompiled version
wget https://github.com/AptS-1547/shortlinker/releases/latest/download/shortlinker-linux-x86_64

# 3. Source code compilation
git clone https://github.com/AptS-1547/shortlinker && cd shortlinker && cargo run
```

### Basic Usage

```bash
# Add short link
./shortlinker add github https://github.com

# Access short link
curl -L http://localhost:8080/github
```

## Get Started

Ready? Check out the [Quick Start Guide](/en/guide/getting-started) to begin your Shortlinker journey!
