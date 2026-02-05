# Docker Deployment

Shortlinker provides optimized Docker images supporting various deployment methods.

## Image Availability

### Standard (Default)

Without Prometheus metrics export, smaller image size.

```bash
# Docker Hub (recommended)
docker pull e1saps/shortlinker

# GitHub Container Registry
docker pull ghcr.io/apts-1547/shortlinker
```

### Metrics Edition

Includes Prometheus metrics export (`/health/metrics` endpoint), suitable for monitored production environments.

```bash
# Docker Hub
docker pull e1saps/shortlinker:latest-metrics

# GitHub Container Registry
docker pull ghcr.io/apts-1547/shortlinker:latest-metrics
```

### Available Tags

| Tag | Description |
|-----|-------------|
| `latest` | Latest build (standard) |
| `latest-metrics` | Latest build (with Prometheus metrics) |
| `stable` / `stable-metrics` | Latest stable release |
| `edge` / `edge-metrics` | Latest pre-release (alpha/beta/rc) |
| `v0.5.0-alpha.6` | Specific version (standard) |
| `v0.5.0-alpha.6-metrics` | Specific version (with Prometheus metrics) |

### Build from Source

```bash
git clone https://github.com/AptS-1547/shortlinker
cd shortlinker

# Standard
docker build -t shortlinker .

# Metrics edition
docker build --build-arg CARGO_FEATURES="cli,metrics" -t shortlinker:metrics .
```

## Quick Start

### Basic Run
```bash
# 1) Prepare a minimal startup config (backend reads config.toml from current working directory; in container it's usually /config.toml)
cat > config.toml << 'EOF'
[server]
host = "0.0.0.0"
port = 8080

[database]
database_url = "sqlite:///data/shortlinker.db"
EOF

mkdir -p data

# 2) Run
docker run -d --name shortlinker \
  -p 8080:8080 \
  -v $(pwd)/config.toml:/config.toml:ro \
  -v $(pwd)/data:/data \
  e1saps/shortlinker
```

### Data Persistence
```bash
# TCP (recommended)
docker run -d --name shortlinker \
  -p 8080:8080 \
  -v $(pwd)/config.toml:/config.toml:ro \
  -v $(pwd)/data:/data \
  e1saps/shortlinker

# Unix socket
cat > config.toml << 'EOF'
[server]
unix_socket = "/sock/shortlinker.sock"

[database]
database_url = "sqlite:///data/shortlinker.db"
EOF

docker run -d --name shortlinker \
  -v $(pwd)/config.toml:/config.toml:ro \
  -v $(pwd)/data:/data \
  -v $(pwd)/sock:/sock \
  e1saps/shortlinker
```

### Runtime Config (Admin/Health/Panel)

> Runtime config is stored in the database (e.g. `features.default_url`, `features.enable_admin_panel`, `api.health_token`). Update it via the in-container CLI or Admin API. See [Configuration Guide](/en/config/).

```bash
# Read the auto-generated admin password (usually /admin_token.txt inside the container)
docker exec shortlinker cat /admin_token.txt

# Set root default redirect (no restart)
docker exec shortlinker /shortlinker config set features.default_url https://example.com

# Set Health Bearer token (no restart)
docker exec shortlinker /shortlinker config set api.health_token your_health_token

# Enable admin panel (restart required)
docker exec shortlinker /shortlinker config set features.enable_admin_panel true
docker restart shortlinker
```

## Docker Compose

### Basic Configuration
```yaml
# docker-compose.yml
version: '3.8'

services:
  shortlinker:
    image: e1saps/shortlinker
    container_name: shortlinker
    ports:
      - "8080:8080"
    volumes:
      - ./config.toml:/config.toml:ro
      - ./data:/data
    restart: unless-stopped
```

### Production Configuration
```yaml
# docker-compose.prod.yml
version: '3.8'

services:
  shortlinker:
    image: e1saps/shortlinker:latest
    # TCP port
    ports:
      - "127.0.0.1:8080:8080"
    # Unix socket mount (choose one)
    # volumes:
    #   - ./sock:/sock
    volumes:
      - ./config.toml:/config.toml:ro
      - ./data:/data
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "wget", "--quiet", "--tries=1", "--spider", "http://localhost:8080/"]
      interval: 30s
      timeout: 10s
      retries: 3
```

## Startup and Management

```bash
# Start service
docker-compose up -d

# View logs
docker-compose logs -f shortlinker

# Stop service
docker-compose down

# Update image
docker-compose pull && docker-compose up -d
```

## Image Features

### Advantages
- **Minimal Size**: Based on scratch image, final size only a few MB
- **Security**: No operating system, reduced attack surface
- **Performance**: Single binary file, fast startup
- **Cross-platform**: Supports amd64, arm64 architectures

### Prometheus Metrics (Optional)

`/health/metrics` (Prometheus text format) is provided by the compile-time `metrics` feature.

If `GET /health/metrics` returns `404`, the current image/binary was built without that feature.

**Recommended: Use the pre-built metrics image**

```bash
# Use the official image with -metrics suffix
docker pull e1saps/shortlinker:latest-metrics
```

**Manual build:**

```bash
# Add metrics on top of default features
cargo build --release --features metrics

# Or explicitly enable (common in Dockerfile-style builds)
cargo build --release --features cli,metrics

# Full build (includes metrics)
cargo build --release --features full

# Docker self-build
docker build --build-arg CARGO_FEATURES="cli,metrics" -t shortlinker:metrics .
```

### Multi-stage Build
```dockerfile
# Build stage
FROM rust:1.92-slim as builder
WORKDIR /app
COPY . .
RUN cargo build --release

# Runtime stage
FROM scratch
COPY --from=builder /app/target/release/shortlinker /shortlinker
EXPOSE 8080
CMD ["/shortlinker"]
```

## Data Management

### Data Directory Structure
```
data/
├── links.db            # SQLite database file
└── backup/            # Backup directory (optional)
    └── links.db.20240101
```

### Backup Strategy
```bash
# Create backup script
cat > backup.sh << 'EOF'
#!/bin/bash
docker exec shortlinker cp /data/links.db /data/backup/links.db.$(date +%Y%m%d_%H%M%S)
find ./data/backup -name "links.db.*" -mtime +7 -delete
EOF

chmod +x backup.sh

# Scheduled backup (add to crontab)
0 2 * * * /path/to/backup.sh
```

## Security Configuration

### Network Security
```bash
# Local listening only - TCP
docker run -d --name shortlinker \
  -p 127.0.0.1:8080:8080 \
  -v $(pwd)/config.toml:/config.toml:ro \
  -v $(pwd)/data:/data \
  e1saps/shortlinker

# Unix socket
docker run -d --name shortlinker \
  -v $(pwd)/config.toml:/config.toml:ro \
  -v $(pwd)/data:/data \
  -v $(pwd)/sock:/sock \
  e1saps/shortlinker

# Custom network
docker network create shortlinker-net
docker run -d --network shortlinker-net --name shortlinker e1saps/shortlinker
```

### Resource Limits
```yaml
services:
  shortlinker:
    image: e1saps/shortlinker
    deploy:
      resources:
        limits:
          memory: 128M
          cpus: '0.5'
        reservations:
          memory: 64M
          cpus: '0.25'
```

## Troubleshooting

### Common Issues
```bash
# Check container status
docker ps -a
docker logs shortlinker

# Check port usage
netstat -tlnp | grep 8080

# Check mount points
docker inspect shortlinker | grep -A 10 Mounts

# Monitor resource usage
docker stats shortlinker
```

### Debug Mode
```bash
# Interactive mode run (set `logging.level = "debug"` in config.toml for more logs)
docker run -it --rm \
  -p 8080:8080 \
  -v $(pwd)/config.toml:/config.toml:ro \
  -v $(pwd)/data:/data \
  e1saps/shortlinker
```

## Production Environment Recommendations

1. **Use specific version tags**, avoid `latest`
2. **Set resource limits** to prevent resource abuse  
3. **Configure health checks** for automatic container restart
4. **Regular data backup** with monitoring alerts
5. **Expose through reverse proxy**, don't directly expose container ports
