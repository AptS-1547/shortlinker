# 存储后端详解

本页包含功能对比、限制说明、性能基准，以及 SQLite / PostgreSQL / MySQL / MariaDB 配置示例。

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

> 性能基准与选型建议已迁移至 [存储选型与性能](/config/storage-selection)，本页仅保留后端特性与配置方式。

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
- 快速开始和原型验证

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

**Docker 快速开始**：

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

**Docker 快速开始**：

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

**Docker 快速开始**：

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
