# Installation Guide

Choose the installation method that suits you to quickly deploy Shortlinker.

## Requirements

### Runtime Environment
- Operating System: Linux, macOS, Windows
- Network Connection: Required for downloading dependencies

### Source Compilation Environment
- **Rust**: >= 1.82.0 (required)
- **Git**: For cloning the project

## Installation Methods

### 🐳 Docker Deployment (Recommended)

No dependencies required, start with one command:

```bash
# Basic run
docker run -d -p 8080:8080 e1saps/shortlinker

# Data persistence (recommended)
docker run -d -p 8080:8080 -v $(pwd)/data:/data e1saps/shortlinker
```

### 📦 Pre-compiled Binaries

Download the pre-compiled version for your platform:

```bash
# Linux x64
wget https://github.com/AptS-1547/shortlinker/releases/latest/download/shortlinker-linux-x64.tar.gz
tar -xzf shortlinker-linux-x64.tar.gz
./shortlinker

# macOS
wget https://github.com/AptS-1547/shortlinker/releases/latest/download/shortlinker-macos.tar.gz

# Windows
# Download shortlinker-windows.zip and extract
```

### 🔧 Source Compilation

Suitable for users who need customization:

```bash
# 1. Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# 2. Check version
rustc --version  # Should be >= 1.82.0

# 3. Clone and compile
git clone https://github.com/AptS-1547/shortlinker.git
cd shortlinker
cargo build --release

# 4. Run
./target/release/shortlinker
```

## Quick Verification

After installation, verify the service is working properly:

```bash
# Start service
./shortlinker

# Test in another terminal
curl -I http://localhost:8080/
# Should return 307 redirect
```

## Common Issues

### Rust Version Too Old
```bash
# Update to latest version
rustup update
```

### Compilation Failed
```bash
# Clean and rebuild
cargo clean && cargo build --release
```

### Port Already in Use
```bash
# Use a different port
SERVER_PORT=3000 ./shortlinker
```

## Next Steps

After installation, continue reading:
- 🚀 [Quick Start](/en/guide/getting-started) - Learn basic usage
- ⚙️ [Configuration](/en/config/) - Understand configuration options
- 📋 [CLI Tools](/en/cli/) - Master command line operations
