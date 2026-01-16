# Docker Deployment

Shortlinker provides optimized Docker images supporting various deployment methods.

## Image Availability

```bash
# Docker Hub (recommended)
docker pull e1saps/shortlinker

# GitHub Container Registry  
docker pull ghcr.io/apts-1547/shortlinker

# Build from source
git clone https://github.com/AptS-1547/shortlinker
cd shortlinker && docker build -t shortlinker .
```

## Quick Start

### Basic Run
```bash
# Simplest startup method
docker run -d -p 8080:8080 e1saps/shortlinker
```

### Data Persistence
```bash
# Mount data directory - TCP
docker run -d \
  -p 8080:8080 \
  -v $(pwd)/data:/data \
  -e DATABASE_URL=sqlite:///data/shortlinker.db \
  e1saps/shortlinker

# Unix socket
docker run -d \
  -v $(pwd)/data:/data \
  -v $(pwd)/sock:/sock \
  -e UNIX_SOCKET=/sock/shortlinker.sock \
  -e DATABASE_URL=sqlite:///data/shortlinker.db \
  e1saps/shortlinker
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
      - ./data:/data
    environment:
      - SERVER_HOST=0.0.0.0
      - SERVER_PORT=8080
      - DATABASE_URL=sqlite:///data/shortlinker.db
      - DEFAULT_URL=https://example.com
      - RUST_LOG=info
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
      - ./data:/data
    environment:
      # TCP configuration
      - SERVER_HOST=0.0.0.0
      - SERVER_PORT=8080
      # Unix socket configuration (choose one)
      # - UNIX_SOCKET=/sock/shortlinker.sock
      - DATABASE_URL=sqlite:///data/links.db
      - DEFAULT_URL=https://your-domain.com
      - ADMIN_TOKEN=${ADMIN_TOKEN}
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
docker run -d -p 127.0.0.1:8080:8080 e1saps/shortlinker

# Unix socket
docker run -d \
  -v $(pwd)/sock:/sock \
  -e UNIX_SOCKET=/sock/shortlinker.sock \
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
# Interactive mode run
docker run -it --rm \
  -p 8080:8080 \
  -v $(pwd)/data:/data \
  -e RUST_LOG=debug \
  e1saps/shortlinker
```

## Production Environment Recommendations

1. **Use specific version tags**, avoid `latest`
2. **Set resource limits** to prevent resource abuse  
3. **Configure health checks** for automatic container restart
4. **Regular data backup** with monitoring alerts
5. **Expose through reverse proxy**, don't directly expose container ports
