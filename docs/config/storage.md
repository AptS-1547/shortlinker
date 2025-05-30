# 存储后端配置

Shortlinker 从 v0.1.0 版本开始支持多种存储后端，您可以根据需求选择最适合的存储方案。

## 版本说明

- **v0.1.0+**: 支持 SQLite、文件存储、Sled 三种后端，SQLite 为默认选择
- **< v0.1.0**: 仅支持 JSON 文件存储

## 存储后端概述

| 存储类型 | 版本支持 | 默认 | 性能 | 易用性 | 适用场景 |
|----------|----------|------|------|--------|----------|
| SQLite | v0.1.0+ | ✅ | 高 | 中 | 生产环境，中大规模部署 |
| 文件存储 | 全版本 | ❌ | 中 | 高 | 开发调试，小规模部署 |
| Sled | v0.1.0+ | ❌ | 高 | 中 | 高并发场景 |

## SQLite 数据库存储（默认，v0.1.0+）

### 简介
SQLite 是一个轻量级的关系数据库，提供了出色的性能和可靠性，从 v0.1.0 版本开始成为生产环境的推荐选择。

### 配置参数
```bash
STORAGE_TYPE=sqlite        # 启用 SQLite 存储
SQLITE_DB_PATH=links.db    # 数据库文件路径
```

### 优势
- **高性能**：原生 SQL 查询，索引支持
- **ACID 事务**：数据一致性保证
- **并发读取**：支持多个读取操作
- **成熟稳定**：生产环境验证
- **轻量级**：无需额外服务

### 劣势
- **写入限制**：高并发写入性能有限
- **工具依赖**：需要 SQL 工具查看数据

### 配置示例
```bash
# 基础配置
STORAGE_TYPE=sqlite
SQLITE_DB_PATH=data/links.db

# 生产环境
STORAGE_TYPE=sqlite
SQLITE_DB_PATH=/var/lib/shortlinker/links.db

# Docker 环境
STORAGE_TYPE=sqlite
SQLITE_DB_PATH=/data/links.db
```

### 数据库操作
```bash
# 查看表结构
sqlite3 links.db ".schema"

# 查看所有链接
sqlite3 links.db "SELECT * FROM links;"

# 统计链接数量
sqlite3 links.db "SELECT COUNT(*) FROM links;"

# 备份数据库
cp links.db links.db.backup
```

## 文件存储（全版本支持）

### 简介
使用 JSON 文件存储数据，简单直观，适合开发和小规模部署。这是 v0.1.0 之前版本的默认存储方式。

### 配置参数
```bash
STORAGE_TYPE=file          # 启用文件存储
LINKS_FILE=links.json      # JSON 文件路径
```

### 优势
- **简单直观**：人类可读的 JSON 格式
- **易于调试**：直接查看和编辑文件
- **版本控制**：可纳入 Git 管理
- **零依赖**：无需额外工具

### 劣势
- **性能限制**：大量数据时加载较慢
- **并发限制**：写入操作互斥
- **无事务**：数据一致性依赖文件系统

### 配置示例
```bash
# 开发环境
STORAGE_TYPE=file
LINKS_FILE=dev-links.json

# 生产环境
STORAGE_TYPE=file
LINKS_FILE=/var/lib/shortlinker/links.json

# 相对路径
STORAGE_TYPE=file
LINKS_FILE=data/links.json
```

### 文件格式
```json
[
  {
    "short_code": "github",
    "target_url": "https://github.com",
    "created_at": "2024-01-01T00:00:00Z",
    "expires_at": null
  },
  {
    "short_code": "temp",
    "target_url": "https://example.com",
    "created_at": "2024-01-01T00:00:00Z",
    "expires_at": "2024-12-31T23:59:59Z"
  }
]
```

## Sled 数据库存储（v0.1.0+）

### 简介
Sled 是一个现代的嵌入式数据库，专为高并发场景设计，从 v0.1.0 版本开始支持。

### 配置参数
```bash
STORAGE_TYPE=sled          # 启用 Sled 存储
SLED_DB_PATH=links.sled    # 数据库目录路径
```

### 优势
- **高并发**：优秀的并发读写性能
- **事务支持**：ACID 事务保证
- **压缩存储**：自动数据压缩
- **崩溃恢复**：自动恢复机制

### 劣势
- **内存占用**：相对更高的内存使用
- **生态成熟度**：较新的技术
- **工具支持**：专用工具较少

### 配置示例
```bash
# 基础配置
STORAGE_TYPE=sled
SLED_DB_PATH=data/links.sled

# 高并发环境
STORAGE_TYPE=sled
SLED_DB_PATH=/fast-ssd/links.sled

# 临时目录
STORAGE_TYPE=sled
SLED_DB_PATH=/tmp/links.sled
```

## 存储后端选择指南

### 按部署规模选择

#### 小规模（< 1,000 链接）
```bash
# 推荐：文件存储（开发友好）
STORAGE_TYPE=file
LINKS_FILE=links.json
```

#### 中等规模（1,000 - 10,000 链接）
```bash
# 推荐：SQLite（平衡性能和易用性）
STORAGE_TYPE=sqlite
SQLITE_DB_PATH=links.db
```

#### 大规模（> 10,000 链接）
```bash
# 推荐：SQLite 或 Sled
STORAGE_TYPE=sqlite
SQLITE_DB_PATH=links.db
```

### 按使用场景选择

#### 开发环境
```bash
# 文件存储 - 便于调试
STORAGE_TYPE=file
LINKS_FILE=dev-links.json
RUST_LOG=debug
```

#### 测试环境
```bash
# SQLite - 接近生产环境
STORAGE_TYPE=sqlite
SQLITE_DB_PATH=test-links.db
```

#### 生产环境
```bash
# SQLite - 稳定可靠
STORAGE_TYPE=sqlite
SQLITE_DB_PATH=/data/links.db
```

#### 高并发场景
```bash
# Sled - 高性能并发
STORAGE_TYPE=sled
SLED_DB_PATH=/data/links.sled
```

## 版本迁移指南

### 从 v0.0.x 升级到 v0.1.0+

如果您从早期版本升级，默认存储方式已从文件存储变更为 SQLite：

```bash
# v0.0.x 默认配置（自动使用文件存储）
# 无需配置，自动使用 links.json

# v0.1.0+ 默认配置（自动使用 SQLite）
STORAGE_TYPE=sqlite
SQLITE_DB_PATH=links.db

# 如需继续使用文件存储，请显式配置
STORAGE_TYPE=file
LINKS_FILE=links.json
```

### 数据迁移

```bash
# 从文件存储迁移到 SQLite
# 1. 备份现有数据
cp links.json links.json.backup

# 2. 设置新的存储配置
export STORAGE_TYPE=sqlite
export SQLITE_DB_PATH=links.db

# 3. 重启服务，系统会自动检测并迁移数据
./shortlinker
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

### Sled 问题
```bash
# 检查锁定状态
lsof +D links.sled/

# 强制解锁（谨慎使用）
rm -rf links.sled/db
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

### SQLite 监控
```bash
# 数据库大小
du -h links.db

# 链接数量
sqlite3 links.db "SELECT COUNT(*) FROM links;"
```

### 文件存储监控
```bash
# 文件大小
ls -lh links.json

# 链接数量
jq 'length' links.json
```

### Sled 监控
```bash
# 目录大小
du -sh links.sled/

# 内存使用（通过进程监控）
ps aux | grep shortlinker
```
