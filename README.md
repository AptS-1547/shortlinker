# shortlinker

A minimalist URL shortener service supporting HTTP 302 redirection, built with Rust. Easy to deploy and lightning fast.

## ✨ Features

* 🚀 **High Performance**: Built with Rust + Actix-web
* 🎯 **Dynamic Management**: Add or remove links at runtime without restarting
* 🎲 **Smart Short Codes**: Supports both custom and randomly generated codes
* 💾 **Persistent Storage**: Stores data in a JSON file with hot-reloading support
* 🔄 **Cross-Platform**: Works on Windows, Linux, and macOS
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

# Manage links
./shortlinker list                    # List all links
./shortlinker remove github           # Remove specific link
```

## Configuration Options

| Environment Variable | Default Value | Description        |
| -------------------- | ------------- | ------------------ |
| `SERVER_HOST`        | `127.0.0.1`   | Listen address     |
| `SERVER_PORT`        | `8080`        | Listen port        |
| `LINKS_FILE`         | `links.json`  | Storage file       |
| `RANDOM_CODE_LENGTH` | `6`           | Random code length |
| `RUST_LOG`           | `info`        | Logging level      |

## Reverse Proxy Configuration

### Caddy

```caddy
esap.cc {
    reverse_proxy 127.0.0.1:8080
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
    }
}
```

## Technical Details

* **Hot Reloading**: Via Unix signal (`SIGUSR1`) or Windows file watcher
* **Random Code Generation**: Alphanumeric with configurable length
* **Conflict Handling**: Smart detection with force overwrite option
* **Container Optimization**: Multi-stage build with `scratch` base image

## Development

```bash
# Development build
cargo run

# Production build
cargo build --release

# Cross-compilation (requires `cross`)
cross build --release --target x86_64-unknown-linux-musl
```

## License

MIT License © AptS:1547
