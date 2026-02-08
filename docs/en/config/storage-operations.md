# Storage Operations and Monitoring

This page focuses on migration, troubleshooting, and monitoring recommendations.

## Version Migration

### Data Migration

The system automatically detects and migrates data without manual intervention.

## Troubleshooting

### SQLite Issues

```bash
# Confirm SQLite path from database.database_url (default: shortlinks.db)
DB_FILE="shortlinks.db"

# Check database integrity
sqlite3 "$DB_FILE" "PRAGMA integrity_check;"

# Database corruption repair
sqlite3 "$DB_FILE" ".dump" | sqlite3 "${DB_FILE%.db}_recovered.db"
```

### Permission Issues

```bash
# Confirm SQLite path from database.database_url (default: shortlinks.db)
DB_FILE="shortlinks.db"

# Check file permissions
ls -la "$DB_FILE" "${DB_FILE}-wal" "${DB_FILE}-shm" 2>/dev/null

# Fix permissions
chown shortlinker:shortlinker "$DB_FILE" "${DB_FILE}-wal" "${DB_FILE}-shm" 2>/dev/null || true
chmod 600 "$DB_FILE" "${DB_FILE}-wal" "${DB_FILE}-shm" 2>/dev/null || true
```

## Monitoring Recommendations

Use health check API to monitor storage status:

```bash
# Option A (recommended): set runtime config api.health_token and use Bearer auth (best for monitoring/probes)
# HEALTH_TOKEN="your_health_token"
# curl -sS -H "Authorization: Bearer ${HEALTH_TOKEN}" http://localhost:8080/health/live -I

# Option B: reuse Admin JWT-cookie auth, login first to obtain cookies
curl -sS -X POST \
  -H "Content-Type: application/json" \
  -c cookies.txt \
  -d '{"password":"your_admin_token"}' \
  http://localhost:8080/admin/v1/auth/login

# Check storage health status
curl -sS -b cookies.txt http://localhost:8080/health
```

Response example:

```json
{
  "code": 0,
  "message": "OK",
  "data": {
    "status": "healthy",
    "timestamp": "2025-06-01T12:00:00Z",
    "uptime": 3600,
    "checks": {
      "storage": {
        "status": "healthy",
        "links_count": 1234,
        "backend": {
          "storage_type": "sqlite",
          "support_click": true
        }
      },
      "cache": {
        "status": "healthy",
        "cache_type": "memory",
        "bloom_filter_enabled": true,
        "negative_cache_enabled": true
      }
    },
    "response_time_ms": 15
  }
}
```

> 🔗 **Related Documentation**: [Health Check API](/en/api/health)
