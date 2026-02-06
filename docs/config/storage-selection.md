# 存储选型与性能

本页聚焦按规模/场景/并发的选型建议与性能对比数据。

> 说明：各数据库的能力差异与连接示例请查看 [存储后端详解](/config/storage-backends)。

## 存储后端选择指南

### 按部署规模选择

```toml
# config.toml（设置 [database].database_url）
[database]
# 小规模部署（< 10,000 链接）
# database_url = "./links.db"
# 或使用显式 URL
# database_url = "sqlite://./links.db"

# 中等规模（10,000 - 100,000 链接）
# database_url = "sqlite://./links.db"
# 或使用 MySQL/MariaDB
# database_url = "mysql://user:pass@host:3306/db"

# 大规模（> 100,000 链接）
# database_url = "postgresql://user:pass@host:5432/db"
# 或使用 MySQL/MariaDB
# database_url = "mysql://user:pass@host:3306/db"
```

### 按使用场景选择

```toml
# config.toml（设置 [database].database_url）
[database]
# 开发环境
# database_url = "dev-links.db"
# database_url = "sqlite://./dev.db"

# 测试环境
# database_url = ":memory:"

# 生产环境（单机）
# database_url = "/data/links.db"

# 生产环境（集群）
# database_url = "postgresql://user:pass@cluster:5432/shortlinker"
```

### 按并发需求选择

```toml
# config.toml（设置 [database].database_url）
[database]
# 低并发（< 100 QPS）
# database_url = "links.db"

# 中等并发（100-1000 QPS）
# database_url = "sqlite://links.db"
# database_url = "mysql://user:pass@host:3306/db"

# 高并发（> 1000 QPS）
# database_url = "postgres://user:pass@host:5432/shortlinker"
```

## 性能对比数据

### 读取性能

- **SQLite**: ~0.1ms（索引查询）

### 写入性能

- **SQLite**: ~1ms（单个事务）

### 并发性能

- **SQLite**: 多读单写

> 💡 **性能提示**：通过 `config.toml` 的 `server.cpu_count` 调整工作线程数可优化并发处理能力。推荐设置为等于或略小于 CPU 核心数。
