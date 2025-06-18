# Storage Backend Configuration

Shortlinker supports multiple storage backends. You can choose the most suitable storage solution based on your needs. All backends are built on asynchronous connection pools, supporting high concurrency and production environment deployment.

> 📋 **Configuration**: For storage-related environment variable configuration, please refer to [Environment Variable Configuration](/en/config/)

## Storage Backend Comparison

| Feature | SQLite | PostgreSQL | MySQL | MariaDB | File Storage | Sled |
|----------|---------|------------|--------|---------|----------|------|
| **Basic Features** | | | | | | |
| Create Short Links | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Get Short Links | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Delete Short Links | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Batch Import | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| **Advanced Features** | | | | | | |
| Click Counting | ✅ | ✅ | ✅ | ✅ | ❌ | ✅ |
| Click Statistics Query | ✅ | ✅ | ✅ | ✅ | ❌ | ✅ |
| Expiration Time | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Auto Expiration Cleanup | ✅ | ✅ | ✅ | ✅ | ❌ | ✅ |
| UTF-8/Emoji Support | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| **Performance Features** | | | | | | |
| Concurrent Reads | ✅ Multi-read | ✅ Multi-read | ✅ Multi-read | ✅ Multi-read | ✅ Multi-read | ✅ Multi-read |
| Concurrent Writes | ⚠️ Single-write | ✅ Multi-write | ✅ Multi-write | ✅ Multi-write | ❌ Mutex | ✅ Multi-write |
| Transaction Support | ✅ ACID | ✅ ACID | ✅ ACID | ✅ ACID | ❌ | ✅ ACID |
| Connection Pool | ✅ | ✅ | ✅ | ✅ | ❌ | ✅ |
| **Operations Features** | | | | | | |
| Hot Backup | ✅ File copy | ✅ pg_dump | ✅ mysqldump | ✅ mariadb-dump | ✅ File copy | ✅ |
| Incremental Backup | ❌ | ✅ WAL | ✅ binlog | ✅ binlog | ❌ | ❌ |
| Online Scaling | ❌ | ✅ | ✅ | ✅ | ❌ | ❌ |
| Cluster Support | ❌ | ✅ | ✅ | ✅ | ❌ | ❌ |

## Storage Backend Limitations

### SQLite Limitations

**Concurrency Limitations**:

- ✅ Supports multiple concurrent reads
- ⚠️ Only supports single write operation (slightly improved in WAL mode)
- ⚠️ Writes temporarily block reads

**Capacity Limitations**:

- ✅ Single table theoretical limit: 281TB
- ✅ Practical recommendation: < 100GB, < 10 million records
- ✅ Automatic index optimization

**Click Counting**:

- ✅ Supports real-time click counting
- ✅ Batch refresh mechanism reduces lock contention
- ⚠️ High-frequency clicks may affect write performance

**Other Limitations**:

- ❌ No network access support
- ❌ No user permission management
- ❌ No horizontal scaling support

### PostgreSQL Limitations

**Performance Limitations**:

- ✅ Theoretically unlimited capacity
- ✅ Supports hundreds of thousands of QPS
- ✅ Supports complex queries and analysis

**Click Counting**:

- ✅ High-performance concurrent click counting
- ✅ Supports real-time statistics queries
- ✅ Supports time-period statistics

**Operations Requirements**:

- ⚠️ Requires professional DBA maintenance
- ⚠️ High memory consumption (recommended >= 1GB)
- ⚠️ Requires regular VACUUM cleanup

### MySQL/MariaDB Limitations

**Storage Limitations**:

- ✅ InnoDB engine: theoretical 256TB
- ✅ Supports table partitioning and sharding
- ✅ Mature cluster solutions

**Click Counting**:

- ✅ High-performance click counting
- ✅ Supports triggers and stored procedures
- ✅ Rich statistical query capabilities

**Character Set Notes**:

- ✅ Uses utf8mb4 by default for full emoji support
- ⚠️ Older versions may require manual character set configuration

### File Storage Limitations

**Important Limitations**:

- ❌ **No click counting support**: Cannot record access statistics
- ❌ **No concurrent write support**: File locking mechanism
- ❌ **No auto-expiration support**: Requires manual cleanup
- ⚠️ **Performance limitations**: Performance degrades when file size > 10MB

**Applicable Range**:

- ✅ Development and testing environments
- ✅ Configuration file management
- ✅ Less than 1000 records
- ❌ Not recommended for production environments

### Sled Limitations

**Current Status**:

- 🚧 Under development, not fully integrated yet
- ✅ High-performance embedded database
- ✅ Supports ACID transactions

**Expected Limitations**:

- ✅ Single-machine high performance
- ❌ No SQL query support
- ❌ No network access support

## Database Backend Configuration

### SQLite Database Storage (Default)

**Features**:

- ✅ Zero configuration, works out of the box
- ✅ ACID transaction guarantee
- ✅ High-performance local queries
- ✅ Automatic index optimization
- ✅ File-level backup
- ⚠️ Single-write concurrency limitation

**Configuration Examples**:

```bash
STORAGE_BACKEND=sqlite
DATABASE_URL=./data/links.db

# Relative path (recommended)
DATABASE_URL=./shortlinker.db

# Absolute path
DATABASE_URL=/var/lib/shortlinker/links.db

# In-memory database (for testing)
DATABASE_URL=:memory:
```

**Use Cases**:

- Single-machine deployment
- Medium scale (< 100,000 links)
- Quick startup and prototyping

### PostgreSQL Database Storage

**Features**:

- ✅ Enterprise-grade reliability
- ✅ High concurrency multi-read multi-write
- ✅ Powerful JSON support
- ✅ Rich index types
- ✅ Horizontal scaling support
- ✅ Mature monitoring ecosystem

**Configuration Examples**:

```bash
STORAGE_BACKEND=postgres
DATABASE_URL=postgresql://user:password@localhost:5432/shortlinker

# Production environment example
DATABASE_URL=postgresql://shortlinker:secure_password@db.example.com:5432/shortlinker_prod?sslmode=require
```

**Docker Quick Start**:

```bash
docker run --name postgres-shortlinker \
  -e POSTGRES_DB=shortlinker \
  -e POSTGRES_USER=shortlinker \
  -e POSTGRES_PASSWORD=your_password \
  -p 5432:5432 -d postgres:15
```

**Use Cases**:

- Enterprise production environments
- High concurrency access (1000+ QPS)
- Large-scale data (millions of links)
- Complex queries and analysis requirements

### MySQL Database Storage

**Features**:

- ✅ Wide ecosystem support
- ✅ Mature operations tools
- ✅ High concurrent read-write performance
- ✅ Rich engine choices (InnoDB)
- ✅ Complete backup and recovery solutions
- ✅ Full UTF-8 support

**Configuration Examples**:

```bash
STORAGE_BACKEND=mysql
DATABASE_URL=mysql://user:password@localhost:3306/shortlinker

# Production environment example
DATABASE_URL=mysql://shortlinker:secure_password@mysql.example.com:3306/shortlinker_prod
```

**Docker Quick Start**:

```bash
docker run --name mysql-shortlinker \
  -e MYSQL_DATABASE=shortlinker \
  -e MYSQL_USER=shortlinker \
  -e MYSQL_PASSWORD=your_password \
  -e MYSQL_ROOT_PASSWORD=root_password \
  -p 3306:3306 -d mysql:8.0
```

**Use Cases**:

- Traditional enterprise environments
- Existing MySQL infrastructure
- Integration with existing MySQL applications

### MariaDB Database Storage

**Features**:

- ✅ 100% MySQL compatible
- ✅ Open source friendly license
- ✅ Faster query optimizer
- ✅ Enhanced JSON support
- ✅ Better performance monitoring
- ✅ Active community support

**Configuration Examples**:

```bash
STORAGE_BACKEND=mariadb
DATABASE_URL=mysql://user:password@localhost:3306/shortlinker

# Note: MariaDB uses the same mysql:// protocol
DATABASE_URL=mysql://shortlinker:secure_password@mariadb.example.com:3306/shortlinker_prod
```

**Docker Quick Start**:

```bash
docker run --name mariadb-shortlinker \
  -e MARIADB_DATABASE=shortlinker \
  -e MARIADB_USER=shortlinker \
  -e MARIADB_PASSWORD=your_password \
  -e MARIADB_ROOT_PASSWORD=root_password \
  -p 3306:3306 -d mariadb:11.1
```

**Use Cases**:

- Open source priority environments
- Modern MySQL alternative
- Better performance and open source licensing needs

## Non-Database Backend Configuration

### File Storage

**File Storage Features**:

- **Simple and Intuitive**: Human-readable JSON format
- **Easy to Debug**: Direct file viewing and editing
- **Version Control**: Can be managed with Git
- **Zero Dependencies**: No additional tools required

**File Storage Use Cases**:

- Development and testing environments
- Small-scale deployment (< 1,000 links)
- Scenarios requiring manual link editing

**File Format**:

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

### Sled Database Storage (Planned)

**Sled Features**:

- **High Concurrency**: Excellent concurrent read-write performance
- **Transaction Support**: ACID transaction guarantee
- **Compressed Storage**: Automatic data compression
- **Crash Recovery**: Automatic recovery mechanism

**Sled Use Cases**:

- High concurrency access scenarios
- Large-scale link management (10,000+ links)
- High-performance requirement environments

## Storage Backend Selection Guide

### Selection by Deployment Scale

```bash
# Small scale deployment (< 1,000 links)
STORAGE_BACKEND=file
DATABASE_URL=links.json

# Medium scale (1,000 - 100,000 links)
STORAGE_BACKEND=sqlite
DATABASE_URL=./links.db

# Large scale (> 100,000 links)
STORAGE_BACKEND=postgres  # or mysql/mariadb
DATABASE_URL=postgresql://user:pass@host:5432/db
```

### Selection by Use Case

```bash
# Development environment
STORAGE_BACKEND=file
DATABASE_URL=dev-links.json

# Testing environment
STORAGE_BACKEND=sqlite
DATABASE_URL=:memory:

# Production environment (single machine)
STORAGE_BACKEND=sqlite
DATABASE_URL=/data/links.db

# Production environment (cluster)
STORAGE_BACKEND=postgres
DATABASE_URL=postgresql://user:pass@cluster:5432/shortlinker
```

### Selection by Concurrency Requirements

```bash
# Low concurrency (< 100 QPS)
STORAGE_BACKEND=sqlite

# Medium concurrency (100-1000 QPS)
STORAGE_BACKEND=sqlite  # or mysql/mariadb

# High concurrency (> 1000 QPS)
STORAGE_BACKEND=postgres  # recommended
```

## Performance Benchmark Data

### Read Performance (Single Query Latency)

| Storage Type | Average Latency | P95 Latency | P99 Latency |
|----------|----------|----------|----------|
| SQLite | 0.1ms | 0.3ms | 0.8ms |
| PostgreSQL | 0.2ms | 0.5ms | 1.2ms |
| MySQL | 0.15ms | 0.4ms | 1.0ms |
| MariaDB | 0.15ms | 0.4ms | 1.0ms |
| File Storage | 0.05ms | 0.1ms | 0.2ms |

### Write Performance (Including Click Counting)

| Storage Type | TPS | Batch Write | Click Count TPS |
|----------|-----|----------|--------------|
| SQLite | 1,000 | 10,000 | 5,000 |
| PostgreSQL | 10,000 | 100,000 | 50,000 |
| MySQL | 8,000 | 80,000 | 40,000 |
| MariaDB | 8,500 | 85,000 | 42,000 |
| File Storage | 100 | 500 | ❌ |

### Concurrent Performance (50 Concurrent Users)

| Storage Type | QPS | Error Rate | Average Response Time |
|----------|-----|--------|--------------|
| SQLite | 2,000 | < 0.1% | 25ms |
| PostgreSQL | 15,000 | < 0.01% | 3ms |
| MySQL | 12,000 | < 0.01% | 4ms |
| MariaDB | 12,500 | < 0.01% | 4ms |
| File Storage | 500 | < 1% | 100ms |

> 📊 **Test Environment**: 4-core 8GB memory, Docker container based

## Version Migration

### Upgrading from v0.0.x to v0.1.0+

v0.1.0+ uses SQLite by default. To continue using file storage:

```bash
# Explicitly configure file storage
STORAGE_BACKEND=file
DATABASE_URL=links.json
```

### Data Migration

The system automatically detects and migrates data without manual intervention.

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
