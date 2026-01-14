# Health Check API

Shortlinker provides health check API for monitoring service status and storage health.

## Function Overview

- Service health status check
- Storage backend status monitoring  
- Readiness and liveness checks
- Service uptime statistics

## Configuration

Health check API requires the following environment variables. For detailed configuration, see [Environment Variables Configuration](/en/config/):

- `HEALTH_TOKEN` - Health check dedicated token (optional)
- `HEALTH_ROUTE_PREFIX` - Route prefix (optional, default `/health`)

**Authentication**:

| HEALTH_TOKEN | ADMIN_TOKEN | Result |
|--------------|-------------|--------|
| Set | Any | Authenticate with HEALTH_TOKEN |
| Not set | Set | Authenticate with ADMIN_TOKEN |
| Not set | Not set | Health API disabled |

All requests must include Authorization header:
```http
Authorization: Bearer your_secure_health_token
```

## API Endpoints

**Base URL**: `http://your-domain:port/health`

### GET /health - Complete Health Check

```bash
curl -H "Authorization: Bearer your_health_token" \
     http://localhost:8080/health
```

**Response Example**:
```json
{
  "code": 0,
  "data": {
    "status": "healthy",
    "timestamp": "2025-06-01T12:00:00Z",
    "uptime": 3600,
    "checks": {
      "storage": {
        "status": "healthy",
        "links_count": 42,
        "backend": {
          "storage_type": "sqlite",
          "support_click": true
        }
      }
    },
    "response_time_ms": 15
  }
}
```

**Response Field Description**:

| Field | Type | Description |
|-------|------|-------------|
| `code` | Integer | Response code: 0 for healthy, 1 for unhealthy |
| `data.status` | String | Overall health status: `healthy` or `unhealthy` |
| `data.timestamp` | RFC3339 | Timestamp of the check |
| `data.uptime` | Integer | Service uptime in seconds |
| `data.checks.storage.status` | String | Storage backend health status |
| `data.checks.storage.links_count` | Integer | Number of short links stored |
| `data.checks.storage.backend` | Object | Storage backend configuration |
| `data.response_time_ms` | Integer | Health check response time in milliseconds |

### GET /health/ready - Readiness Check

```bash
curl -H "Authorization: Bearer your_health_token" \
     http://localhost:8080/health/ready
```

Returns 200 status code indicating service is ready.

### GET /health/live - Liveness Check

```bash
curl -H "Authorization: Bearer your_health_token" \
     http://localhost:8080/health/live
```

Returns 204 status code indicating service is running normally.

## Status Codes

| Status Code | Description |
|-------------|-------------|
| 200 | Healthy/Ready |
| 204 | Live (no content) |
| 401 | Authentication failed |
| 503 | Service unhealthy |

## Monitoring Integration

### Kubernetes Probe Configuration

```yaml
apiVersion: v1
kind: Pod
spec:
  containers:
  - name: shortlinker
    image: e1saps/shortlinker
    livenessProbe:
      httpGet:
        path: /health/live
        port: 8080
        httpHeaders:
        - name: Authorization
          value: "Bearer your_health_token"
      initialDelaySeconds: 30
      periodSeconds: 10
    readinessProbe:
      httpGet:
        path: /health/ready
        port: 8080
        httpHeaders:
        - name: Authorization
          value: "Bearer your_health_token"
      initialDelaySeconds: 5
      periodSeconds: 5
```

### Docker Compose Health Check

```yaml
version: '3.8'
services:
  shortlinker:
    image: e1saps/shortlinker
    healthcheck:
      test: ["CMD", "curl", "-f", "-H", "Authorization: Bearer your_health_token", "http://localhost:8080/health/live"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 40s
```

## Monitoring Script Example

```bash
#!/bin/bash
# simple_monitor.sh

HEALTH_TOKEN="your_health_token"
HEALTH_URL="http://localhost:8080/health"

check_health() {
    response=$(curl -s -w "%{http_code}" -H "Authorization: Bearer $HEALTH_TOKEN" "$HEALTH_URL")
    http_code="${response: -3}"
  
    if [ "$http_code" -eq 200 ]; then
        echo "$(date): Service is healthy"
        return 0
    else
        echo "$(date): Service is unhealthy (HTTP $http_code)"
        return 1
    fi
}

# Check every 60 seconds
while true; do
    check_health || echo "$(date): Sending alert..."
    sleep 60
done
```

## Troubleshooting

```bash
# Check service status
curl -H "Authorization: Bearer your_token" http://localhost:8080/health | jq .

# Verify if API is enabled
if [ -n "$HEALTH_TOKEN" ]; then
    echo "Health API enabled"
else
    echo "Health API disabled"
fi
```

## Security Recommendations

1. **Strong Password**: Use sufficiently complex HEALTH_TOKEN
2. **Network Isolation**: Only expose health check endpoints in monitoring networks
3. **Regular Rotation**: Regularly change Health Token
