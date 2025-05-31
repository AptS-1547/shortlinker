# Docker Deployment

Shortlinker provides optimized Docker images supporting multiple deployment methods.

## Image Sources

### Official Images

```bash
# Docker Hub
docker pull e1saps/shortlinker

# GitHub Container Registry  
docker pull ghcr.io/apts-1547/shortlinker
```

### Self-Build Images

```bash
# Clone project
git clone https://github.com/AptS-1547/shortlinker
cd shortlinker

# Build image
docker build -t shortlinker .
```

## Quick Start

### Basic Run

```bash
# Simplest startup method
docker run -d -p 8080:8080 e1saps/shortlinker
```

### Data Persistence

```bash
# Mount data directory
docker run -d \
  -p 8080:8080 \
  -v $(pwd)/data:/data \
  e1saps/shortlinker
```

### Complete Configuration

```bash
docker run -d \
  --name shortlinker \
  -p 8080:8080 \
  -v $(pwd)/data:/data \
  -e SERVER_HOST=0.0.0.0 \
  -e SERVER_PORT=8080 \
  -e STORAGE_BACKEND=sqlite \
  -e DB_FILE_NAME=/data/shortlinker.data \
  -e DEFAULT_URL=https://example.com \
  -e RANDOM_CODE_LENGTH=8 \
  -e RUST_LOG=info \
  -e ADMIN_TOKEN=your_secure_admin_token \
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
      - STORAGE_BACKEND=sqlite
      - DB_FILE_NAME=/data/shortlinker.data
      - DEFAULT_URL=https://example.com
      - RUST_LOG=info
      - ADMIN_TOKEN=dev_token_123
    restart: unless-stopped
```

### Production Environment Configuration

```yaml
# docker-compose.prod.yml
version: '3.8'

services:
  shortlinker:
    image: e1saps/shortlinker:latest
    container_name: shortlinker-prod
    ports:
      - "127.0.0.1:8080:8080"  # Local listen only
    volumes:
      - ./data:/data
      - ./logs:/logs
    environment:
      - SERVER_HOST=0.0.0.0
      - SERVER_PORT=8080
      - STORAGE_BACKEND=sqlite
      - DB_FILE_NAME=/data/links.db
      - DEFAULT_URL=https://your-domain.com
      - RANDOM_CODE_LENGTH=8
      - RUST_LOG=info
      - ADMIN_TOKEN=${ADMIN_TOKEN}  # Use environment variable
      - ADMIN_ROUTE_PREFIX=/secure-admin
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "wget", "--no-verbose", "--tries=1", "--spider", "http://localhost:8080/"]
      interval: 30s
      timeout: 10s
      retries: 3
    
  nginx:
    image: nginx:alpine
    container_name: nginx-proxy
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf
      - ./ssl:/etc/nginx/ssl
    depends_on:
      - shortlinker
    restart: unless-stopped
```

## Startup and Management

```bash
# Start services
docker-compose up -d

# View logs
docker-compose logs -f shortlinker

# Stop services
docker-compose down

# Restart services
docker-compose restart shortlinker

# Update images
docker-compose pull
docker-compose up -d
```

## Image Features

### Multi-stage Build

Dockerfile uses multi-stage builds to optimize image size:

```dockerfile
# Build stage
FROM rust:1.82 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

# Runtime stage
FROM scratch
COPY --from=builder /app/target/release/shortlinker /shortlinker
EXPOSE 8080
CMD ["/shortlinker"]
```

### Image Advantages

- **Minimal Size**: Based on scratch image, final size only a few MB
- **Security**: No operating system, reduced attack surface
- **Performance**: Single binary file, fast startup
- **Cross-Platform**: Supports amd64, arm64 architectures

## Environment Variables

Container supports all standard environment variables:

```bash
# Server configuration
SERVER_HOST=0.0.0.0
SERVER_PORT=8080

# Storage configuration  
STORAGE_BACKEND=sqlite
DB_FILE_NAME=/data/links.db

# Feature configuration
DEFAULT_URL=https://example.com
RANDOM_CODE_LENGTH=8

# Admin API configuration (v0.0.5+)
ADMIN_TOKEN=your_secure_admin_token
ADMIN_ROUTE_PREFIX=/admin

# Log configuration
RUST_LOG=info
```

## Data Management

### Data Directory Structure

```
data/
├── links.json          # Link data file
└── backup/            # Backup directory (optional)
    ├── links.json.20240101
    └── links.json.20240102
```

### Backup Strategy

```bash
# Create backup
docker exec shortlinker cp /data/links.json /data/backup/links.json.$(date +%Y%m%d)

# Scheduled backup script
#!/bin/bash
docker exec shortlinker cp /data/links.json /data/backup/links.json.$(date +%Y%m%d_%H%M%S)
find ./data/backup -name "links.json.*" -mtime +7 -delete
```

## Troubleshooting

### Common Issues

1. **Container Won't Start**
   ```bash
   # Check logs
   docker logs shortlinker
   
   # Check port usage
   docker ps -a
   netstat -tlnp | grep 8080
   ```

2. **Data Loss**
   ```bash
   # Check mount points
   docker inspect shortlinker | grep Mounts -A 10
   
   # Check permissions
   ls -la data/
   ```

3. **Performance Issues**
   ```bash
   # Monitor resource usage
   docker stats shortlinker
   
   # Check container health
   docker inspect shortlinker --format='{{.State.Health.Status}}'
   ```

### Debug Mode

```bash
# Run in interactive mode (for debugging)
docker run -it --rm \
  -p 8080:8080 \
  -v $(pwd)/data:/data \
  -e RUST_LOG=debug \
  e1saps/shortlinker

# Access running container (if base image supports it)
docker exec -it shortlinker /bin/sh
```

## Security Configuration

### Network Security

```bash
# Local listen only
docker run -d \
  -p 127.0.0.1:8080:8080 \
  e1saps/shortlinker

# Custom network
docker network create shortlinker-net
docker run -d \
  --network shortlinker-net \
  --name shortlinker \
  e1saps/shortlinker
```

### Resource Limits

```yaml
# docker-compose.yml
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

### Admin API Security in Docker

```bash
# Generate secure token
ADMIN_TOKEN=$(openssl rand -hex 32)

# Set environment variable
echo "ADMIN_TOKEN=$ADMIN_TOKEN" > .env

# Use in Docker Compose
docker-compose --env-file .env up -d
```

## Production Environment Recommendations

1. **Use specific version tags**, avoid `latest`
2. **Set resource limits** to prevent resource abuse  
3. **Configure health checks** for automatic container restart
4. **Run containers as non-root user**
5. **Regular data backups** with monitoring and alerting
6. **Expose services through reverse proxy**, don't expose container ports directly
7. **Use strong Admin API tokens** and custom route prefixes
8. **Enable HTTPS** for all Admin API communications
