# 存储后端配置

Shortlinker 支持多种存储后端，您可以根据需求选择最适合的存储方案。所有后端均基于异步连接池，支持高并发和生产环境部署。

> 📋 **配置方法**：存储相关的环境变量配置请参考 [环境变量配置](/config/)

## 存储后端功能对比

| 功能特性 | SQLite | PostgreSQL | MySQL | MariaDB | 文件存储 | Sled |
|----------|---------|------------|--------|---------|----------|------|
| **基础功能** | | | | | | |
| 创建短链接 | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| 获取短链接 | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| 删除短链接 | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| 批量导入 | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| **高级功能** | | | | | | |
| 点击计数 | ✅ | ✅ | ✅ | ✅ | ❌ | ✅ |
| 点击统计查询 | ✅ | ✅ | ✅ | ✅ | ❌ | ✅ |
| 过期时间设置 | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| UTF-8/Emoji 支持 | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| 并发写入 | ⚠️ 单写 | ✅ 多写 | ✅ 多写 | ✅ 多写 | ❌ 互斥 | ✅ 多写 |
| 事务支持 | ✅ ACID | ✅ ACID | ✅ ACID | ✅ ACID | ❌ | ✅ ACID |
| 连接池 | ✅ | ✅ | ✅ | ✅ | ❌ | ✅ |
| **运维特性** | | | | | | |
| 热备份 | ✅ 文件复制 | ✅ pg_dump | ✅ mysqldump | ✅ mariadb-dump | ✅ 文件复制 | ✅ |
| 增量备份 | ❌ | ✅ WAL | ✅ binlog | ✅ binlog | ❌ | ❌ |
| 在线扩容 | ❌ | ✅ | ✅ | ✅ | ❌ | ❌ |
| 集群支持 | ❌ | ✅ | ✅ | ✅ | ❌ | ❌ |

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

### 文件存储限制

**重要限制**：

- ❌ **不支持点击计数**：无法记录访问统计
- ❌ **不支持并发写入**：文件锁机制
- ❌ **不支持自动过期**：需要手动清理
- ⚠️ **性能限制**：文件大小 > 10MB 时性能下降

**适用范围**：

- ✅ 开发测试环境
- ✅ 配置文件管理
- ✅ 少于 1000 条记录
- ❌ 不推荐生产环境

### Sled 限制

**当前状态**：

- 🚧 开发中，暂未完全集成
- ✅ 高性能嵌入式数据库
- ✅ 支持 ACID 事务

**预期限制**：

- ✅ 单机高性能
- ❌ 不支持 SQL 查询
- ❌ 不支持网络访问

## 性能基准测试

### 读取性能（单次查询延迟）

| 存储类型 | 平均延迟 | P95 延迟 | P99 延迟 |
|----------|----------|----------|----------|
| SQLite | 0.1ms | 0.3ms | 0.8ms |
| PostgreSQL | 0.2ms | 0.5ms | 1.2ms |
| MySQL | 0.15ms | 0.4ms | 1.0ms |
| MariaDB | 0.15ms | 0.4ms | 1.0ms |
| 文件存储 | 0.05ms | 0.1ms | 0.2ms |

### 写入性能（包含点击计数）

| 存储类型 | TPS | 批量写入 | 点击计数 TPS |
|----------|-----|----------|--------------|
| SQLite | 1,000 | 10,000 | 5,000 |
| PostgreSQL | 10,000 | 100,000 | 50,000 |
| MySQL | 8,000 | 80,000 | 40,000 |
| MariaDB | 8,500 | 85,000 | 42,000 |
| 文件存储 | 100 | 500 | ❌ |

### 并发性能（50 并发用户）

| 存储类型 | QPS | 错误率 | 平均响应时间 |
|----------|-----|--------|--------------|
| SQLite | 2,000 | < 0.1% | 25ms |
| PostgreSQL | 15,000 | < 0.01% | 3ms |
| MySQL | 12,000 | < 0.01% | 4ms |
| MariaDB | 12,500 | < 0.01% | 4ms |
| 文件存储 | 500 | < 1% | 100ms |

> 📊 **测试环境**：4核8GB内存，基于 Docker 容器

## 数据库后端配置

### SQLite 数据库存储（默认）

**特点**：

- ✅ 零配置，开箱即用
- ✅ ACID 事务保证
- ✅ 高性能本地查询
- ✅ 自动索引优化
- ✅ 文件级备份
- ⚠️ 单写并发限制

**配置示例**：

```bash
STORAGE_BACKEND=sqlite
DATABASE_URL=./data/links.db

# 相对路径（推荐）
DATABASE_URL=./shortlinker.db

# 绝对路径
DATABASE_URL=/var/lib/shortlinker/links.db

# 内存数据库（测试用）
DATABASE_URL=:memory:
```

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

**配置示例**：

```bash
STORAGE_BACKEND=postgres
DATABASE_URL=postgresql://user:password@localhost:5432/shortlinker

# 生产环境示例
DATABASE_URL=postgresql://shortlinker:secure_password@db.example.com:5432/shortlinker_prod?sslmode=require
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

**配置示例**：

```bash
STORAGE_BACKEND=mysql
DATABASE_URL=mysql://user:password@localhost:3306/shortlinker

# 生产环境示例
DATABASE_URL=mysql://shortlinker:secure_password@mysql.example.com:3306/shortlinker_prod
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

**配置示例**：

```bash
STORAGE_BACKEND=mariadb
DATABASE_URL=mysql://user:password@localhost:3306/shortlinker

# 注意：MariaDB 使用相同的 mysql:// 协议
DATABASE_URL=mysql://shortlinker:secure_password@mariadb.example.com:3306/shortlinker_prod
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

## 非数据库后端配置

## 文件存储

### 特点

- **简单直观**：人类可读的 JSON 格式
- **易于调试**：直接查看和编辑文件
- **版本控制**：可纳入 Git 管理
- **零依赖**：无需额外工具

### 适用场景

- 开发和测试环境
- 小规模部署（< 1,000 链接）
- 需要手动编辑链接的场景

### 文件格式

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

## Sled 数据库存储（计划中）

### Sled 特点

- **高并发**：优秀的并发读写性能
- **事务支持**：ACID 事务保证
- **压缩存储**：自动数据压缩
- **崩溃恢复**：自动恢复机制

### Sled 适用场景

- 高并发访问场景
- 大规模链接管理（10,000+ 链接）
- 性能要求较高的环境

## 存储后端选择指南

### 按部署规模选择

```bash
# 小规模部署（< 1,000 链接）
STORAGE_BACKEND=file
DATABASE_URL=links.json

# 中等规模（1,000 - 100,000 链接）
STORAGE_BACKEND=sqlite
DATABASE_URL=./links.db

# 大规模（> 100,000 链接）
STORAGE_BACKEND=postgres  # 或 mysql/mariadb
DATABASE_URL=postgresql://user:pass@host:5432/db
```

### 按使用场景选择

```bash
# 开发环境
STORAGE_BACKEND=file
DATABASE_URL=dev-links.json

# 测试环境
STORAGE_BACKEND=sqlite
DATABASE_URL=:memory:

# 生产环境（单机）
STORAGE_BACKEND=sqlite
DATABASE_URL=/data/links.db

# 生产环境（集群）
STORAGE_BACKEND=postgres
DATABASE_URL=postgresql://user:pass@cluster:5432/shortlinker
```

### 按并发需求选择

```bash
# 低并发（< 100 QPS）
STORAGE_BACKEND=sqlite

# 中等并发（100-1000 QPS）
STORAGE_BACKEND=sqlite  # 或 mysql/mariadb

# 高并发（> 1000 QPS）
STORAGE_BACKEND=postgres  # 推荐
```

## 性能对比数据

### 读取性能

- **SQLite**: ~0.1ms（索引查询）
- **文件存储**: ~0.05ms（内存哈希表）
- **Sled**: ~0.1ms（B+ 树查询）

### 写入性能

- **SQLite**: ~1ms（单个事务）
- **文件存储**: ~10ms（重写整个文件）
- **Sled**: ~0.5ms（LSM 树写入）

### 并发性能

- **SQLite**: 多读单写
- **文件存储**: 互斥访问
- **Sled**: 多读多写

> 💡 **性能提示**：通过 `CPU_COUNT` 环境变量调整工作线程数可优化并发处理能力。推荐设置为等于或略小于 CPU 核心数。

## 版本迁移

### 从 v0.0.x 升级到 v0.1.0+

v0.1.0+ 版本默认使用 SQLite，如需继续使用文件存储：

```bash
# 显式配置文件存储
STORAGE_BACKEND=file
DATABASE_URL=links.json
```

### 数据迁移

系统会自动检测并迁移数据，无需手动操作。

## 故障排除

### SQLite 问题

```bash
# 检查数据库完整性
sqlite3 links.db "PRAGMA integrity_check;"

# 数据库损坏修复
sqlite3 links.db ".dump" | sqlite3 new_links.db
```

### 文件存储问题

```bash
# 验证 JSON 格式
jq . links.json

# 修复格式错误
jq '.' links.json > fixed.json && mv fixed.json links.json
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
# 检查存储健康状态
curl -H "Authorization: Bearer $HEALTH_TOKEN" \
     http://localhost:8080/health
```

响应示例：

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

> 🔗 **相关文档**：[健康检查 API](/api/health)
