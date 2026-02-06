# Storage Backends

This page covers backend comparison, limitations, and backend-specific configuration examples.

## Storage Backend Comparison

| Feature | SQLite | PostgreSQL | MySQL | MariaDB |
|----------|---------|------------|--------|---------|
| **Basic Features** | | | | |
| Create Short Links | ✅ | ✅ | ✅ | ✅ |
| Get Short Links | ✅ | ✅ | ✅ | ✅ |
| Delete Short Links | ✅ | ✅ | ✅ | ✅ |
| Batch Import | ✅ | ✅ | ✅ | ✅ |
| **Advanced Features** | | | | |
| Click Counting | ✅ | ✅ | ✅ | ✅ |
| Click Statistics Query | ✅ | ✅ | ✅ | ✅ |
| Expiration Time | ✅ | ✅ | ✅ | ✅ |
| Auto Expiration Cleanup | ✅ | ✅ | ✅ | ✅ |
| UTF-8/Emoji Support | ✅ | ✅ | ✅ | ✅ |
| **Performance Features** | | | | |
| Concurrent Reads | ✅ Multi-read | ✅ Multi-read | ✅ Multi-read | ✅ Multi-read |
| Concurrent Writes | ⚠️ Single-write | ✅ Multi-write | ✅ Multi-write | ✅ Multi-write |
| Transaction Support | ✅ ACID | ✅ ACID | ✅ ACID | ✅ ACID |
| Connection Pool | ✅ | ✅ | ✅ | ✅ |
| **Operations Features** | | | | |
| Hot Backup | ✅ File copy | ✅ pg_dump | ✅ mysqldump | ✅ mariadb-dump |
| Incremental Backup | ❌ | ✅ WAL | ✅ binlog | ✅ binlog |
| Online Scaling | ❌ | ✅ | ✅ | ✅ |
| Cluster Support | ❌ | ✅ | ✅ | ✅ |

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

```toml
# config.toml
[database]
# Relative path
# database_url = "sqlite://./shortlinker.db"
database_url = "sqlite://./data/links.db"

# Absolute path
# database_url = "sqlite:///var/lib/shortlinker/links.db"

# In-memory database (testing)
# database_url = ":memory:"
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

```toml
# config.toml
[database]
# Standard connection URL
database_url = "postgres://user:password@localhost:5432/shortlinker"
# database_url = "postgresql://user:password@localhost:5432/shortlinker"

# Production environment example
# database_url = "postgresql://shortlinker:secure_password@db.example.com:5432/shortlinker_prod?sslmode=require"
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

```toml
# config.toml
[database]
# Standard connection URL
database_url = "mysql://user:password@localhost:3306/shortlinker"

# Production environment example
# database_url = "mysql://shortlinker:secure_password@mysql.example.com:3306/shortlinker_prod"
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

```toml
# config.toml
[database]
# MariaDB uses mariadb:// scheme (handled as MySQL protocol)
database_url = "mariadb://user:password@localhost:3306/shortlinker"

# Also supports mysql:// scheme (backward compatible)
# database_url = "mysql://shortlinker:secure_password@mariadb.example.com:3306/shortlinker_prod"
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


