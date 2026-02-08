# Storage Selection and Benchmarks

This page focuses on backend selection guidance and benchmark data.

> For backend capabilities and connection format examples, see [Storage Backends](/en/config/storage-backends).

## Storage Backend Selection Guide

### Selection by Deployment Scale

```toml
# config.toml ([database].database_url)
[database]
# Small scale (< 10,000 links)
# database_url = "sqlite://./shortlinks.db"

# Medium scale (10,000 - 100,000 links)
# database_url = "sqlite://./shortlinks.db"
# Or use MySQL/MariaDB
# database_url = "mysql://user:pass@host:3306/db"

# Large scale (> 100,000 links)
# database_url = "postgresql://user:pass@host:5432/db"
# Or use MySQL/MariaDB
# database_url = "mysql://user:pass@host:3306/db"
```

### Selection by Use Case

```toml
# config.toml ([database].database_url)
[database]
# Development environment
# database_url = "sqlite://./dev.db"

# Testing environment
# database_url = ":memory:"

# Production environment (single machine)
# database_url = "sqlite:///data/shortlinks.db"

# Production environment (cluster)
# database_url = "postgresql://user:pass@cluster:5432/shortlinker"
```

### Selection by Concurrency Requirements

```toml
# config.toml ([database].database_url)
[database]
# Low concurrency (< 100 QPS)
# database_url = "sqlite://shortlinks.db"

# Medium concurrency (100-1000 QPS)
# database_url = "sqlite://shortlinks.db"
# Or MySQL/MariaDB
# database_url = "mysql://user:pass@host:3306/db"

# High concurrency (> 1000 QPS)
# database_url = "postgres://user:pass@host:5432/shortlinker"
```

## Performance Benchmark Data

### Read Performance (Single Query Latency)

| Storage Type | Average Latency | P95 Latency | P99 Latency |
|----------|----------|----------|----------|
| SQLite | 0.1ms | 0.3ms | 0.8ms |
| PostgreSQL | 0.2ms | 0.5ms | 1.2ms |
| MySQL | 0.15ms | 0.4ms | 1.0ms |
| MariaDB | 0.15ms | 0.4ms | 1.0ms |

### Write Performance (Including Click Counting)

| Storage Type | TPS | Batch Write | Click Count TPS |
|----------|-----|----------|--------------|
| SQLite | 1,000 | 10,000 | 5,000 |
| PostgreSQL | 10,000 | 100,000 | 50,000 |
| MySQL | 8,000 | 80,000 | 40,000 |
| MariaDB | 8,500 | 85,000 | 42,000 |

### Concurrent Performance (50 Concurrent Users)

| Storage Type | QPS | Error Rate | Average Response Time |
|----------|-----|--------|--------------|
| SQLite | 2,000 | < 0.1% | 25ms |
| PostgreSQL | 15,000 | < 0.01% | 3ms |
| MySQL | 12,000 | < 0.01% | 4ms |
| MariaDB | 12,500 | < 0.01% | 4ms |

> 📊 **Test Environment**: 4-core 8GB memory, Docker container based
