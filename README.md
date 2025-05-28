# shortlinker

A minimalist URL shortener service supporting HTTP 302 redirection, built with Rust. Easy to deploy and lightning fast.

## ✨ Features

* 🚀 **High Performance**: Built with Rust + Actix-web
* 🎯 **Dynamic Management**: Add or remove links at runtime without restarting
* 🎲 **Smart Short Codes**: Supports both custom and randomly generated codes
* ⏰ **Expiration Support**: Set expiration times for links with automatic cleanup
* 💾 **Persistent Storage**: Stores data in a JSON file with hot-reloading support
* 🔄 **Cross-Platform**: Works on Windows, Linux, and macOS
* 🔐 **Process Management**: Smart process locking to prevent duplicate instances
* 🐳 **Containerized**: Optimized Docker image for easy deployment

## Quick Start

### Run Locally

```bash
git clone https://github.com/AptS-1547/shortlinker
cd shortlinker
cargo run
```

### Deploy with Docker

```bash
# Pull from Docker Hub
docker run -d -p 8080:8080 -v $(pwd)/data:/data e1saps/shortlinker

# Or pull from GitHub Container Registry
docker run -d -p 8080:8080 -v $(pwd)/data:/data ghcr.io/apts-1547/shortlinker

# Build yourself
docker build -t shortlinker .
docker run -d -p 8080:8080 -v $(pwd)/data:/data shortlinker
```

## Usage Example

Once your domain (e.g. `esap.cc`) is bound:

* `https://esap.cc/github` → custom short link
* `https://esap.cc/aB3dF1` → random short link
* `https://esap.cc/` → default homepage

## Command-Line Management

```bash
# Start the server
./shortlinker

# Add short links
./shortlinker add github https://github.com           # Custom code
./shortlinker add https://github.com                  # Random code
./shortlinker add github https://new-url.com --force  # Overwrite existing
./shortlinker add temp https://example.com --expires "2024-12-31T23:59:59Z"  # With expiration

# Manage links
./shortlinker list                    # List all links
./shortlinker remove github           # Remove specific link
```

## Configuration Options

You can configure the service using environment variables or a `.env` file. The program automatically reads the `.env` file from the project root directory.

| Environment Variable | Default Value | Description        |
| -------------------- | ------------- | ------------------ |
| `SERVER_HOST`        | `127.0.0.1`   | Listen address     |
| `SERVER_PORT`        | `8080`        | Listen port        |
| `LINKS_FILE`         | `links.json`  | Storage file path  |
| `DEFAULT_URL`        | `https://esap.cc/repo` | Default redirect URL for root path |
| `RANDOM_CODE_LENGTH` | `6`           | Random code length |
| `RUST_LOG`           | `info`        | Logging level (`error`, `warn`, `info`, `debug`, `trace`) |

### .env File Example

Create a `.env` file in the project root directory:

```bash
# Server configuration
SERVER_HOST=0.0.0.0
SERVER_PORT=3000

# Storage configuration
LINKS_FILE=data/links.json

# Default redirect URL
DEFAULT_URL=https://example.com

# Random code length
RANDOM_CODE_LENGTH=8

# Log level
RUST_LOG=debug
```

**Note**: Environment variables take precedence over `.env` file settings.

## Server Management

### Starting and Stopping

```bash
# Start server
./shortlinker

# Stop server
kill $(cat shortlinker.pid)
```

### Process Protection

- **Unix Systems**: Uses PID file (`shortlinker.pid`) to prevent duplicate instances
- **Windows Systems**: Uses lock file (`.shortlinker.lock`) to prevent duplicate instances
- Automatically detects running instances and provides helpful messages

## Data Format

Link data is stored in JSON format:

```json
{
  "github": {
    "target": "https://github.com",
    "created_at": "2024-01-01T00:00:00Z",
    "expires_at": null
  },
  "temp": {
    "target": "https://example.com",
    "created_at": "2024-01-01T00:00:00Z",
    "expires_at": "2024-12-31T23:59:59Z"
  }
}
```

## Reverse Proxy Configuration

### Caddy

```caddy
esap.cc {
    reverse_proxy 127.0.0.1:8080
    
    # Optional: Add cache control
    header {
        Cache-Control "no-cache, no-store, must-revalidate"
    }
}
```

### Nginx

```nginx
server {
    listen 80;
    server_name esap.cc;
    
    location / {
        proxy_pass http://127.0.0.1:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        
        # Disable caching
        add_header Cache-Control "no-cache, no-store, must-revalidate";
    }
}
```

### System Service (systemd)

```ini
[Unit]
Description=ShortLinker Service
After=network.target

[Service]
Type=simple
User=www-data
WorkingDirectory=/opt/shortlinker
ExecStart=/opt/shortlinker/shortlinker
Restart=always
RestartSec=5

Environment=SERVER_HOST=127.0.0.1
Environment=SERVER_PORT=8080
Environment=RUST_LOG=info

[Install]
WantedBy=multi-user.target
```

## Technical Details

* **Hot Reloading**: Automatic configuration file change detection
* **Random Code Generation**: Alphanumeric with configurable length, collision avoidance
* **Conflict Handling**: Smart detection with force overwrite option
* **Expiration Checking**: Real-time validation on request, automatic cleanup
* **Container Optimization**: Multi-stage build with `scratch` base image
* **Memory Safety**: Arc + RwLock ensures concurrent safety

## Development

```bash
# Development build
cargo run

# Production build
cargo build --release

# Cross-compilation (requires `cross`)
cross build --release --target x86_64-unknown-linux-musl

# Run tests
cargo test

# Check code formatting
cargo fmt
cargo clippy
```

## Performance Optimization

- Uses `Arc<RwLock<HashMap>>` for high-concurrency reads
- 302 temporary redirects to avoid browser caching
- Minimal memory footprint and CPU usage
- Async I/O handling for high concurrency

## Troubleshooting

### Common Issues

1. **Port Already in Use**
   ```bash
   # Check port usage
   lsof -i :8080
   netstat -tlnp | grep 8080
   ```

2. **Permission Issues**
   ```bash
   # Ensure proper permissions
   chmod 755 /path/to/shortlinker
   chown user:group links.json
   ```

3. **Corrupted Configuration File**
   ```bash
   # Validate JSON format
   jq . links.json
   ```

## License

MIT License © AptS:1547
