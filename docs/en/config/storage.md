# Storage Backend Configuration

Starting from v0.1.0, Shortlinker supports multiple storage backends. You can choose the most suitable storage solution based on your needs.

## Version Information

- **v0.1.0+**: Supports SQLite, file storage, and Sled backends, with SQLite as default
- **< v0.1.0**: Only supports JSON file storage

## Storage Backend Overview

| Storage Type | Version Support | Default | Performance | Ease of Use | Use Cases |
|--------------|-----------------|---------|-------------|-------------|-----------|
| SQLite | v0.1.0+ | ✅ | High | Medium | Production, medium to large deployments |
| File Storage | All versions | ❌ | Medium | High | Development, debugging, small deployments |
| Sled | v0.1.0+ | ❌ | High | Medium | High concurrency scenarios |

## SQLite Database Storage (Default, v0.1.0+)

### Introduction
SQLite is a lightweight relational database that provides excellent performance and reliability. It has been the recommended choice for production environments since v0.1.0.

### Configuration Parameters
```bash
STORAGE_TYPE=sqlite        # Enable SQLite storage
SQLITE_DB_PATH=links.db    # Database file path
```

### Advantages
- **High Performance**: Native SQL queries with index support
- **ACID Transactions**: Data consistency guarantee
- **Concurrent Reads**: Supports multiple read operations
- **Mature and Stable**: Production environment validated
- **Lightweight**: No additional services required

### Disadvantages
- **Write Limitations**: Limited high-concurrency write performance
- **Tool Dependencies**: Requires SQL tools to view data

### Configuration Examples
```bash
# Basic configuration
STORAGE_TYPE=sqlite
SQLITE_DB_PATH=data/links.db

# Production environment
STORAGE_TYPE=sqlite
SQLITE_DB_PATH=/var/lib/shortlinker/links.db

# Docker environment
STORAGE_TYPE=sqlite
SQLITE_DB_PATH=/data/links.db
```

## File Storage (All Versions)

### Introduction
Uses JSON files to store data, simple and intuitive, suitable for development and small-scale deployments. This was the default storage method before v0.1.0.

### Configuration Parameters
```bash
STORAGE_TYPE=file          # Enable file storage
LINKS_FILE=links.json      # JSON file path
```

### Advantages
- **Simple and Intuitive**: Human-readable JSON format
- **Easy to Debug**: Direct file viewing and editing
- **Version Control**: Can be included in Git management
- **Zero Dependencies**: No additional tools required

### Disadvantages
- **Performance Limitations**: Slow loading with large amounts of data
- **Concurrency Limitations**: Mutually exclusive write operations
- **No Transactions**: Data consistency depends on file system

## Sled Database Storage (v0.1.0+)

### Introduction
Sled is a modern embedded database designed for high-concurrency scenarios, supported since v0.1.0.

### Configuration Parameters
```bash
STORAGE_TYPE=sled          # Enable Sled storage
SLED_DB_PATH=links.sled    # Database directory path
```

### Advantages
- **High Concurrency**: Excellent concurrent read/write performance
- **Transaction Support**: ACID transaction guarantee
- **Compressed Storage**: Automatic data compression
- **Crash Recovery**: Automatic recovery mechanism

### Disadvantages
- **Memory Usage**: Relatively higher memory consumption
- **Ecosystem Maturity**: Newer technology
- **Tool Support**: Fewer specialized tools

## Storage Backend Selection Guide

### By Deployment Scale

#### Small Scale (< 1,000 links)
```bash
# Recommended: File storage (development-friendly)
STORAGE_TYPE=file
LINKS_FILE=links.json
```

#### Medium Scale (1,000 - 10,000 links)
```bash
# Recommended: SQLite (balanced performance and ease of use)
STORAGE_TYPE=sqlite
SQLITE_DB_PATH=links.db
```

#### Large Scale (> 10,000 links)
```bash
# Recommended: SQLite or Sled
STORAGE_TYPE=sqlite
SQLITE_DB_PATH=links.db
```

### By Use Case

#### Development Environment
```bash
# File storage - easy to debug
STORAGE_TYPE=file
LINKS_FILE=dev-links.json
RUST_LOG=debug
```

#### Production Environment
```bash
# SQLite - stable and reliable
STORAGE_TYPE=sqlite
SQLITE_DB_PATH=/data/links.db
```

#### High Concurrency Scenarios
```bash
# Sled - high-performance concurrency
STORAGE_TYPE=sled
SLED_DB_PATH=/data/links.sled
```

## Version Migration Guide

### Upgrading from v0.0.x to v0.1.0+

If you're upgrading from an earlier version, the default storage method has changed from file storage to SQLite:

```bash
# v0.0.x default configuration (automatically uses file storage)
# No configuration needed, automatically uses links.json

# v0.1.0+ default configuration (automatically uses SQLite)
STORAGE_TYPE=sqlite
SQLITE_DB_PATH=links.db

# To continue using file storage, explicitly configure
STORAGE_TYPE=file
LINKS_FILE=links.json
```

### Data Migration

```bash
# Migrate from file storage to SQLite
# 1. Backup existing data
cp links.json links.json.backup

# 2. Set new storage configuration
export STORAGE_TYPE=sqlite
export SQLITE_DB_PATH=links.db

# 3. Restart service, system will automatically detect and migrate data
./shortlinker
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

## Troubleshooting

### SQLite Issues
```bash
# Check database integrity
sqlite3 links.db "PRAGMA integrity_check;"

# Database corruption repair
sqlite3 links.db ".dump" | sqlite3 new_links.db
```

### File Storage Issues
```bash
# Validate JSON format
jq . links.json

# Fix format errors
jq '.' links.json > fixed.json && mv fixed.json links.json
```

### Sled Issues
```bash
# Check lock status
lsof +D links.sled/

# Force unlock (use with caution)
rm -rf links.sled/db
```

## Monitoring Recommendations

### SQLite Monitoring
```bash
# Database size
du -h links.db

# Link count
sqlite3 links.db "SELECT COUNT(*) FROM links;"
```

### File Storage Monitoring
```bash
# File size
ls -lh links.json

# Link count
jq 'length' links.json
```

### Sled Monitoring
```bash
# Directory size
du -sh links.sled/

# Memory usage (via process monitoring)
ps aux | grep shortlinker
```
