# 存储后端配置

Shortlinker 支持多种存储后端，您可以根据需求选择最适合的存储方案。所有数据库后端均基于 **Sea-ORM** 和异步连接池，支持高并发和生产环境部署。

> 📋 **配置方法**：存储相关配置请参考 [配置指南](/config/)（启动配置 `database.database_url`）。

## Sea-ORM 数据库层

从 v0.2.0 开始，Shortlinker 使用 **Sea-ORM** 作为数据库抽象层，提供：

- ✅ **原子化 upsert 操作**：防止竞态条件，确保并发安全
- ✅ **自动数据库类型检测**：从 `database.database_url` 自动推断数据库类型
- ✅ **自动创建 SQLite 数据库**：首次运行时自动创建数据库文件
- ✅ **自动 schema 迁移**：无需手动运行 SQL 脚本
- ✅ **统一接口**：所有数据库使用相同的代码路径
- ✅ **类型安全**：编译时检查数据库操作

> 💡 **提示**：当前版本 **不读取** `DATABASE_BACKEND`。Shortlinker 会从 `database.database_url` 自动推断数据库类型：  
> - SQLite：`sqlite://...` / 以 `.db` 或 `.sqlite` 结尾的文件路径 / `:memory:`  
> - MySQL/MariaDB：`mysql://...` / `mariadb://...`（会按 MySQL 协议处理）  
> - PostgreSQL：`postgres://...` / `postgresql://...`

## 存储后端功能对比

| 功能特性 | SQLite | PostgreSQL | MySQL | MariaDB |
|----------|---------|------------|--------|---------|
| **基础功能** | | | | |
| 创建短链接 | ✅ | ✅ | ✅ | ✅ |
| 获取短链接 | ✅ | ✅ | ✅ | ✅ |
| 删除短链接 | ✅ | ✅ | ✅ | ✅ |
| 批量导入 | ✅ | ✅ | ✅ | ✅ |
| **高级功能** | | | | |
| 点击计数 | ✅ | ✅ | ✅ | ✅ |
| 点击统计查询 | ✅ | ✅ | ✅ | ✅ |
| 过期时间设置 | ✅ | ✅ | ✅ | ✅ |
| UTF-8/Emoji 支持 | ✅ | ✅ | ✅ | ✅ |
| 并发写入 | ⚠️ 单写 | ✅ 多写 | ✅ 多写 | ✅ 多写 |
| 事务支持 | ✅ ACID | ✅ ACID | ✅ ACID | ✅ ACID |
| 连接池 | ✅ | ✅ | ✅ | ✅ |
| **运维特性** | | | | |
| 热备份 | ✅ 文件复制 | ✅ pg_dump | ✅ mysqldump | ✅ mariadb-dump |
| 增量备份 | ❌ | ✅ WAL | ✅ binlog | ✅ binlog |
| 在线扩容 | ❌ | ✅ | ✅ | ✅ |
| 集群支持 | ❌ | ✅ | ✅ | ✅ |

## 存储后端限制详解

### SQLite 限制

**并发限制**：

- ✅ 支持多个并发读取
- ⚠️ 只支持单个写入操作（WAL 模式下略有改善）
- ⚠️ 写入时会短暂阻塞读取

**容量限制**：

- ✅ 单表理论上限：281TB
- ✅ 实际推荐：< 100GB，< 1000万条记录
- ✅ 索引自动优化

**点击计数**：

- ✅ 支持实时点击计数
- ✅ 批量刷新机制减少锁竞争
- ⚠️ 高频点击可能影响写入性能

**其他限制**：

- ❌ 不支持网络访问
- ❌ 不支持用户权限管理
- ❌ 不支持水平扩展

### PostgreSQL 限制

**性能限制**：

- ✅ 理论上无容量限制
- ✅ 支持数十万 QPS
- ✅ 支持复杂查询和分析

**点击计数**：

- ✅ 高性能并发点击计数
- ✅ 支持实时统计查询
- ✅ 支持按时间段统计

**运维要求**：

- ⚠️ 需要专业 DBA 维护
- ⚠️ 内存消耗较大（建议 >= 1GB）
- ⚠️ 需要定期 VACUUM 清理

### MySQL/MariaDB 限制

**存储限制**：

- ✅ InnoDB 引擎：理论上 256TB
- ✅ 支持表分区和分库分表
- ✅ 成熟的集群方案

**点击计数**：

- ✅ 高性能点击计数
- ✅ 支持触发器和存储过程
- ✅ 丰富的统计查询功能

**字符集注意**：

- ✅ 默认使用 utf8mb4 完全支持 emoji
- ⚠️ 旧版本可能需要手动配置字符集

## 性能基准测试

### 读取性能（单次查询延迟）

| 存储类型 | 平均延迟 | P95 延迟 | P99 延迟 |
|----------|----------|----------|----------|
| SQLite | 0.1ms | 0.3ms | 0.8ms |
| PostgreSQL | 0.2ms | 0.5ms | 1.2ms |
| MySQL | 0.15ms | 0.4ms | 1.0ms |
| MariaDB | 0.15ms | 0.4ms | 1.0ms |

### 写入性能（包含点击计数）

| 存储类型 | TPS | 批量写入 | 点击计数 TPS |
|----------|-----|----------|--------------|
| SQLite | 1,000 | 10,000 | 5,000 |
| PostgreSQL | 10,000 | 100,000 | 50,000 |
| MySQL | 8,000 | 80,000 | 40,000 |
| MariaDB | 8,500 | 85,000 | 42,000 |

### 并发性能（50 并发用户）

| 存储类型 | QPS | 错误率 | 平均响应时间 |
|----------|-----|--------|--------------|
| SQLite | 2,000 | < 0.1% | 25ms |
| PostgreSQL | 15,000 | < 0.01% | 3ms |
| MySQL | 12,000 | < 0.01% | 4ms |
| MariaDB | 12,500 | < 0.01% | 4ms |

> 📊 **测试环境**：4核8GB内存，基于 Docker 容器

## 数据库后端配置

### SQLite 数据库存储（默认）

**特点**：

- ✅ 零配置，开箱即用
- ✅ ACID 事务保证
- ✅ 高性能本地查询
- ✅ 自动索引优化
- ✅ 文件级备份
- ✅ **自动创建数据库文件**（Sea-ORM）
- ✅ **原子 upsert 操作**（使用 ON CONFLICT）
- ⚠️ 单写并发限制

**配置示例**：

```toml
# config.toml
[database]
# 相对路径（自动创建）
# database_url = "./shortlinker.db"
# database_url = "data/links.db"

# 绝对路径
# database_url = "/var/lib/shortlinker/links.db"

# 显式 SQLite URL（推荐）
database_url = "sqlite://./data/links.db"
# database_url = "sqlite:///absolute/path/to/links.db"

# 内存数据库（测试用）
# database_url = ":memory:"
```

**性能优化**（自动应用）：

- WAL（Write-Ahead Logging）模式
- 优化的 cache_size（-64000）
- 内存临时存储
- MMAP 启用（512MB）
- 自动 checkpoint（每1000次写入）

**适用场景**：

- 单机部署
- 中小规模（< 10万链接）
- 快速启动和原型验证

### PostgreSQL 数据库存储

**特点**：

- ✅ 企业级可靠性
- ✅ 高并发多读多写
- ✅ 强大的 JSON 支持
- ✅ 丰富的索引类型
- ✅ 水平扩展支持
- ✅ 成熟的监控生态
- ✅ **原子 upsert 操作**（使用 ON CONFLICT）

**配置示例**：

```toml
# config.toml
[database]
# 标准连接 URL
database_url = "postgres://user:password@localhost:5432/shortlinker"
# database_url = "postgresql://user:password@localhost:5432/shortlinker"

# 生产环境示例
# database_url = "postgresql://shortlinker:secure_password@db.example.com:5432/shortlinker_prod?sslmode=require"
```

**Docker 快速启动**：

```bash
docker run --name postgres-shortlinker \
  -e POSTGRES_DB=shortlinker \
  -e POSTGRES_USER=shortlinker \
  -e POSTGRES_PASSWORD=your_password \
  -p 5432:5432 -d postgres:15
```

**适用场景**：

- 企业级生产环境
- 高并发访问（1000+ QPS）
- 大规模数据（百万级链接）
- 需要复杂查询和分析

### MySQL 数据库存储

**特点**：

- ✅ 广泛的生态支持
- ✅ 成熟的运维工具
- ✅ 高并发读写性能
- ✅ 丰富的引擎选择（InnoDB）
- ✅ 完整的备份恢复方案
- ✅ UTF-8 完全支持
- ✅ **原子 upsert 操作**（使用 try-insert-then-update）

**配置示例**：

```toml
# config.toml
[database]
# 标准连接 URL
database_url = "mysql://user:password@localhost:3306/shortlinker"

# 生产环境示例
# database_url = "mysql://shortlinker:secure_password@mysql.example.com:3306/shortlinker_prod?charset=utf8mb4"
```

**Docker 快速启动**：

```bash
docker run --name mysql-shortlinker \
  -e MYSQL_DATABASE=shortlinker \
  -e MYSQL_USER=shortlinker \
  -e MYSQL_PASSWORD=your_password \
  -e MYSQL_ROOT_PASSWORD=root_password \
  -p 3306:3306 -d mysql:8.0
```

**适用场景**：

- 传统企业环境
- 已有 MySQL 基础设施
- 需要与现有 MySQL 应用集成

### MariaDB 数据库存储

**特点**：

- ✅ 100% MySQL 兼容
- ✅ 开源友好许可
- ✅ 更快的查询优化器
- ✅ 增强的 JSON 支持
- ✅ 更好的性能监控
- ✅ 活跃的社区支持
- ✅ **原子 upsert 操作**（使用 MySQL 协议）

**配置示例**：

```toml
# config.toml
[database]
# MariaDB 使用 mariadb:// scheme（自动按 MySQL 协议处理）
database_url = "mariadb://user:password@localhost:3306/shortlinker"

# 也可以使用 mysql:// scheme（向后兼容）
# database_url = "mysql://shortlinker:secure_password@mariadb.example.com:3306/shortlinker_prod?charset=utf8mb4"
```

**Docker 快速启动**：

```bash
docker run --name mariadb-shortlinker \
  -e MARIADB_DATABASE=shortlinker \
  -e MARIADB_USER=shortlinker \
  -e MARIADB_PASSWORD=your_password \
  -e MARIADB_ROOT_PASSWORD=root_password \
  -p 3306:3306 -d mariadb:11.1
```

**适用场景**：

- 开源优先的环境
- MySQL 的现代化替代
- 需要更好的性能和开源许可

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

## 版本迁移

### 从 v0.1.x 升级到 v0.2.0+

v0.2.0+ 版本迁移到 Sea-ORM，带来以下变化：

**新特性**：
- ✅ 原子化 upsert 操作（防止竞态条件）
- ✅ 从 `database.database_url` 自动检测数据库类型
- ✅ SQLite 数据库文件自动创建
- ✅ 自动 schema 迁移

**配置变更**：
- 存储后端类型完全由 `database.database_url` 决定（`sqlite://` / `mysql://` / `mariadb://` / `postgres://` 等）

**数据迁移**：

系统会自动检测并迁移数据，无需手动操作。从 v0.1.x 的 SQLite/MySQL/PostgreSQL 数据库升级时，Sea-ORM 会自动运行 schema 迁移。

**推荐配置**（v0.2.0+）：

```toml
# config.toml
[database]
# SQLite（推荐）
# database_url = "sqlite://./data/links.db"

# PostgreSQL
# database_url = "postgres://user:pass@localhost:5432/shortlinker"

# MySQL
# database_url = "mysql://user:pass@localhost:3306/shortlinker"
```

## 故障排除

### SQLite 问题

```bash
# 检查数据库完整性
sqlite3 links.db "PRAGMA integrity_check;"

# 数据库损坏修复
sqlite3 links.db ".dump" | sqlite3 new_links.db
```

### 权限问题

```bash
# 检查文件权限
ls -la links.*

# 修复权限
chown shortlinker:shortlinker links.*
chmod 644 links.*
```

## 监控建议

使用健康检查 API 监控存储状态：

```bash
# 方案 A（推荐）：配置运行时配置 api.health_token 后使用 Bearer Token（更适合监控/探针）
# HEALTH_TOKEN="your_health_token"
# curl -sS -H "Authorization: Bearer ${HEALTH_TOKEN}" http://localhost:8080/health/live -I

# 方案 B：复用 Admin 的 JWT Cookie（需要先登录获取 cookies）
curl -sS -X POST \
  -H "Content-Type: application/json" \
  -c cookies.txt \
  -d '{"password":"your_admin_token"}' \
  http://localhost:8080/admin/v1/auth/login

# 检查存储健康状态
curl -sS -b cookies.txt http://localhost:8080/health
```

响应示例：

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

> 🔗 **相关文档**：[健康检查 API](/api/health)
