# Docker Deployment: Operations and Security

This page focuses on image features, data management, security hardening, troubleshooting, and production recommendations.

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
# Start from the base run command in /en/deployment/docker-quickstart
# Here we only show security-related differences

# Local listening only - TCP
docker run -d --name shortlinker -p 127.0.0.1:8080:8080 ...

# Unix socket
docker run -d --name shortlinker -v $(pwd)/sock:/sock ...

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
