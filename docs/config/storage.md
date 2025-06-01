# 存储后端配置

Shortlinker 支持多种存储后端，您可以根据需求选择最适合的存储方案。

> 📋 **配置方法**：存储相关的环境变量配置请参考 [环境变量配置](/config/)

## 存储后端概述

| 存储类型 | 性能 | 易用性 | 适用场景 |
|----------|------|--------|----------|
| **SQLite**（默认） | 高 | 中 | 生产环境，中大规模部署 |
| **文件存储** | 中 | 高 | 开发调试，小规模部署 |
| **Sled**（计划中） | 高 | 中 | 高并发场景 |

## SQLite 数据库存储（推荐）

### 特点
- **高性能**：原生 SQL 查询，索引支持
- **ACID 事务**：数据一致性保证
- **并发读取**：支持多个读取操作
- **轻量级**：无需额外服务

### 适用场景
- 生产环境部署
- 中大规模链接管理（1,000+ 链接）
- 需要数据可靠性的场景

### 数据库操作
```bash
# 查看表结构
sqlite3 links.db ".schema"

# 查看所有链接
sqlite3 links.db "SELECT * FROM links;"

# 备份数据库
cp links.db links.db.backup
```

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

### 特点
- **高并发**：优秀的并发读写性能
- **事务支持**：ACID 事务保证
- **压缩存储**：自动数据压缩
- **崩溃恢复**：自动恢复机制

### 适用场景
- 高并发访问场景
- 大规模链接管理（10,000+ 链接）
- 性能要求较高的环境

## 存储后端选择指南

### 按部署规模选择

```bash
# 小规模（< 1,000 链接）
STORAGE_BACKEND=file
DB_FILE_NAME=links.json

# 中等规模（1,000 - 10,000 链接）
STORAGE_BACKEND=sqlite
DB_FILE_NAME=links.db

# 大规模（> 10,000 链接）
STORAGE_BACKEND=sqlite  # 或 sled（未来）
DB_FILE_NAME=links.db
```

### 按使用场景选择

```bash
# 开发环境
STORAGE_BACKEND=file
DB_FILE_NAME=dev-links.json

# 生产环境
STORAGE_BACKEND=sqlite
DB_FILE_NAME=/data/links.db
```

## 性能对比

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

## 版本迁移

### 从 v0.0.x 升级到 v0.1.0+

v0.1.0+ 版本默认使用 SQLite，如需继续使用文件存储：

```bash
# 显式配置文件存储
STORAGE_BACKEND=file
DB_FILE_NAME=links.json
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
