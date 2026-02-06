# 存储迁移与运维

本页聚焦版本迁移、故障排除与健康监控建议。

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
