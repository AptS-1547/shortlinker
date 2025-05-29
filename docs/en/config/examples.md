# Configuration Examples

Complete configuration examples for various environments.

## Development Environment

### Local Development (.env)

```bash
# Server configuration
SERVER_HOST=127.0.0.1
SERVER_PORT=8080

# Feature configuration
DEFAULT_URL=https://localhost:3000
RANDOM_CODE_LENGTH=4  # Shorter for testing

# Storage configuration
LINKS_FILE=./dev-links.json

# Log configuration
RUST_LOG=debug  # Enable detailed logs for development

# Admin API (development)
ADMIN_TOKEN=dev_token_123
```

### Test Environment

```bash
# Server configuration
SERVER_HOST=0.0.0.0
SERVER_PORT=3000

# Test-specific configuration
DEFAULT_URL=https://test.example.com
RANDOM_CODE_LENGTH=6

# Storage configuration
LINKS_FILE=./test-links.json

# Log configuration
RUST_LOG=info

# Admin API (testing)
ADMIN_TOKEN=test_secure_token_456
```

## Production Environment

### Basic Production Configuration (.env)

```bash
# Server configuration
SERVER_HOST=127.0.0.1  # Expose through reverse proxy
SERVER_PORT=8080

# Storage configuration
LINKS_FILE=data/links.json

# Default redirect address
DEFAULT_URL=https://your-company.com

# Random code length
RANDOM_CODE_LENGTH=8  # Longer for production

# Log level
RUST_LOG=info

# Admin API (production)
ADMIN_TOKEN=very_secure_production_token_789
```

### High Availability Configuration

```bash
# Server configuration
SERVER_HOST=0.0.0.0
SERVER_PORT=8080

# Storage configuration (shared storage)
LINKS_FILE=/shared/data/links.json

# Feature configuration
DEFAULT_URL=https://www.example.com
RANDOM_CODE_LENGTH=10  # Higher uniqueness

# Log configuration
RUST_LOG=warn  # Reduce log volume

# Admin API (high security)
ADMIN_TOKEN=ultra_secure_ha_token_abc123
```

## Docker Environment

### Docker Compose Environment Variables

```yaml
version: '3.8'
services:
  shortlinker:
    image: e1saps/shortlinker
    environment:
      - SERVER_HOST=0.0.0.0
      - SERVER_PORT=8080
      - LINKS_FILE=/data/links.json
      - DEFAULT_URL=https://your-domain.com
      - RANDOM_CODE_LENGTH=8
      - RUST_LOG=info
      - ADMIN_TOKEN=docker_secure_token_def456
    volumes:
      - ./data:/data
    ports:
      - "127.0.0.1:8080:8080"
```

### Docker Command Line

```bash
docker run -d \
  --name shortlinker \
  -p 127.0.0.1:8080:8080 \
  -v $(pwd)/data:/data \
  -e SERVER_HOST=0.0.0.0 \
  -e SERVER_PORT=8080 \
  -e LINKS_FILE=/data/links.json \
  -e DEFAULT_URL=https://your-site.com \
  -e RANDOM_CODE_LENGTH=8 \
  -e RUST_LOG=info \
  -e ADMIN_TOKEN=docker_admin_ghi789 \
  e1saps/shortlinker
```

## Cloud Service Environments

### AWS EC2

```bash
# EC2 instance configuration
SERVER_HOST=0.0.0.0
SERVER_PORT=8080

# Use EFS or EBS persistent storage
LINKS_FILE=/mnt/efs/shortlinker/links.json

# Cloud environment configuration
DEFAULT_URL=https://www.your-aws-site.com
RANDOM_CODE_LENGTH=8

# CloudWatch logs
RUST_LOG=info

# Admin API (AWS)
ADMIN_TOKEN=aws_secure_token_jkl012
```

### Azure Container Instances

```bash
# Container configuration
SERVER_HOST=0.0.0.0
SERVER_PORT=8080

# Azure Files storage
LINKS_FILE=/mnt/azure-files/links.json

# Feature configuration
DEFAULT_URL=https://your-azure-site.azurewebsites.net
RANDOM_CODE_LENGTH=8

# Azure Monitor
RUST_LOG=info

# Admin API (Azure)
ADMIN_TOKEN=azure_admin_mno345
```

### Google Cloud Run

```bash
# Cloud Run configuration
SERVER_HOST=0.0.0.0
SERVER_PORT=8080

# Cloud Storage or Cloud Filestore
LINKS_FILE=/mnt/gcs/links.json

# Feature configuration
DEFAULT_URL=https://your-gcp-site.appspot.com
RANDOM_CODE_LENGTH=8

# Cloud Logging
RUST_LOG=info

# Admin API (GCP)
ADMIN_TOKEN=gcp_secure_pqr678
```

## Special Scenario Configurations

### High Concurrency Scenario

```bash
# Server configuration
SERVER_HOST=0.0.0.0
SERVER_PORT=8080

# Storage optimization
LINKS_FILE=/fast-ssd/links.json

# Performance optimization
RANDOM_CODE_LENGTH=6  # Balance performance and uniqueness
DEFAULT_URL=https://cdn.example.com

# Log optimization
RUST_LOG=error  # Only log errors, reduce I/O

# Admin API (performance focused)
ADMIN_TOKEN=perf_optimized_stu901
```

### Internal Network Deployment

```bash
# Internal network configuration
SERVER_HOST=192.168.1.100
SERVER_PORT=8080

# Internal network storage
LINKS_FILE=/shared/nfs/links.json

# Internal network address
DEFAULT_URL=https://intranet.company.com

# Configuration
RANDOM_CODE_LENGTH=6
RUST_LOG=info

# Admin API (internal)
ADMIN_TOKEN=internal_admin_vwx234
```

### Microservice Environment

```bash
# Kubernetes configuration
SERVER_HOST=0.0.0.0
SERVER_PORT=8080

# PVC storage
LINKS_FILE=/data/links.json

# Service mesh configuration
DEFAULT_URL=https://api.company.com/home

# Configuration
RANDOM_CODE_LENGTH=8
RUST_LOG=info

# Admin API (microservice)
ADMIN_TOKEN=k8s_service_yzab567
```

## systemd Service Configuration

### Environment Variables in Service File

```ini
[Unit]
Description=Shortlinker Service
After=network.target

[Service]
Type=simple
User=www-data
WorkingDirectory=/opt/shortlinker
ExecStart=/opt/shortlinker/shortlinker

# Production environment variables
Environment=SERVER_HOST=127.0.0.1
Environment=SERVER_PORT=8080
Environment=LINKS_FILE=/opt/shortlinker/data/links.json
Environment=DEFAULT_URL=https://example.com
Environment=RANDOM_CODE_LENGTH=8
Environment=RUST_LOG=info
Environment=ADMIN_TOKEN=systemd_secure_cdef890

# Security configuration
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ReadWritePaths=/opt/shortlinker/data

[Install]
WantedBy=multi-user.target
```

## Configuration Validation

### Validation Script

```bash
#!/bin/bash
# validate-config.sh - Configuration validation script

echo "Validating Shortlinker configuration..."

# Check necessary environment variables
if [ -z "$LINKS_FILE" ]; then
    echo "Warning: LINKS_FILE not set, will use default value"
fi

# Check if port is available
if netstat -tuln | grep -q ":${SERVER_PORT:-8080} "; then
    echo "Error: Port ${SERVER_PORT:-8080} is already in use"
    exit 1
fi

# Check storage directory permissions
STORAGE_DIR=$(dirname "${LINKS_FILE:-links.json}")
if [ ! -w "$STORAGE_DIR" ]; then
    echo "Error: Storage directory $STORAGE_DIR has no write permission"
    exit 1
fi

# Check Admin API token strength (if set)
if [ -n "$ADMIN_TOKEN" ] && [ ${#ADMIN_TOKEN} -lt 16 ]; then
    echo "Warning: ADMIN_TOKEN is too short, recommend at least 16 characters"
fi

echo "Configuration validation passed ✓"
```

### Configuration Testing

```bash
# Test if configuration is correct
./shortlinker --version  # Check if program can start normally
curl -I http://localhost:8080/  # Check if service responds
```
