# Docker Deployment: Quick Start and Compose

This page focuses on quick container startup, persistence, runtime config, and Docker Compose examples.

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

