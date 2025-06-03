# Storage Backend Configuration

Shortlinker supports multiple storage backends. You can choose the most suitable storage solution based on your needs.

> 📋 **Configuration Method**: For storage-related environment variable configuration, see [Environment Variables Configuration](/en/config/)

## Storage Backend Overview

| Storage Type | Performance | Ease of Use | Use Cases |
|--------------|-------------|-------------|-----------|
| **SQLite** (default) | High | Medium | Production environment, medium to large-scale deployment |
| **File Storage** | Medium | High | Development debugging, small-scale deployment |
| **Sled** (planned) | High | Medium | High concurrency scenarios |

## SQLite Database Storage (Recommended)

### Features
- **High Performance**: Native SQL queries with index support
- **ACID Transactions**: Data consistency guarantee
- **Concurrent Reading**: Supports multiple read operations
- **Lightweight**: No additional services required

### Use Cases
- Production environment deployment
- Medium to large-scale link management (1,000+ links)
- Scenarios requiring data reliability

### Database Operations
```bash
# View table structure
sqlite3 links.db ".schema"

# View all links
sqlite3 links.db "SELECT * FROM links;"

# Backup database
cp links.db links.db.backup
```

## File Storage

### Features
- **Simple and Intuitive**: Human-readable JSON format
- **Easy to Debug**: Direct file viewing and editing
- **Version Control**: Can be managed with Git
- **Zero Dependencies**: No additional tools required

### Use Cases
- Development and testing environments
- Small-scale deployment (< 1,000 links)
- Scenarios requiring manual link editing

### File Format
```json
[
  {
    "short_code": "github",
    "target_url": "https://github.com",
    "created_at": "2024-01-01T00:00:00Z",
    "expires_at": null
  }
]
```

## Sled Database Storage (Planned)

### Features
- **High Concurrency**: Excellent concurrent read/write performance
- **Transaction Support**: ACID transaction guarantee
- **Compressed Storage**: Automatic data compression
- **Crash Recovery**: Automatic recovery mechanism

### Use Cases
- High concurrency access scenarios
- Large-scale link management (10,000+ links)
- High-performance requirement environments

## Storage Backend Selection Guide

### By Deployment Scale

```bash
# Small scale (< 1,000 links)
STORAGE_BACKEND=file
DB_FILE_NAME=links.json

# Medium scale (1,000 - 10,000 links)
STORAGE_BACKEND=sqlite
DB_FILE_NAME=links.db

# Large scale (> 10,000 links)
STORAGE_BACKEND=sqlite  # or sled (future)
DB_FILE_NAME=links.db
```

### By Use Case

```bash
# Development environment
STORAGE_BACKEND=file
DB_FILE_NAME=dev-links.json

# Production environment
STORAGE_BACKEND=sqlite
DB_FILE_NAME=/data/links.db
```

## Performance Comparison

### Read Performance
- **SQLite**: ~0.1ms (indexed queries)
- **File Storage**: ~0.05ms (in-memory hash table)
- **Sled**: ~0.1ms (B+ tree queries)

### Write Performance
- **SQLite**: ~1ms (single transaction)
- **File Storage**: ~10ms (rewrite entire file)
- **Sled**: ~0.5ms (LSM tree writes)

### Concurrency Performance
- **SQLite**: Multiple readers, single writer
- **File Storage**: Mutually exclusive access
- **Sled**: Multiple readers and writers

> 💡 **Performance Tip**: Adjust worker thread count via `CPU_COUNT` environment variable to optimize concurrent processing. Recommended to set equal to or slightly less than CPU core count.

## Version Migration

### Upgrading from v0.0.x to v0.1.0+

v0.1.0+ versions use SQLite by default. To continue using file storage:

```bash
# Explicitly configure file storage
STORAGE_BACKEND=file
DB_FILE_NAME=links.json
```

### Data Migration

The system will automatically detect and migrate data without manual intervention.

## Troubleshooting

### SQLite Issues
```bash
# Check database integrity
sqlite3 links.db "PRAGMA integrity_check;"

# Repair corrupted database
sqlite3 links.db ".dump" | sqlite3 new_links.db
```

### File Storage Issues
```bash
# Validate JSON format
jq . links.json

# Fix format errors
jq '.' links.json > fixed.json && mv fixed.json links.json
```

### Permission Issues
```bash
# Check file permissions
ls -la links.*

# Fix permissions
chown shortlinker:shortlinker links.*
chmod 644 links.*
```

## Monitoring Recommendations

Use health check API to monitor storage status:

```bash
# Check storage health status
curl -H "Authorization: Bearer $HEALTH_TOKEN" \
     http://localhost:8080/health
```

Response example:
```json
{
  "status": "healthy",
  "checks": {
    "storage": {
      "status": "healthy",
      "links_count": 1234,
      "backend": "sqlite"
    }
  }
}
```

> 🔗 **Related Documentation**: [Health Check API](/en/api/health)
