# 配置指南

Shortlinker 的配置分为两类：

- **启动配置**：存储在 `config.toml` 文件中，修改后需要重启服务
- **动态配置**：存储在数据库中，可通过管理面板在运行时修改

## 配置架构

```
config.toml (启动时读取)
       ↓
   数据库 (持久化存储)
       ↓
  RuntimeConfig (内存缓存)
       ↓
   AppConfig (全局配置)
       ↓
    业务逻辑
```

首次启动时，动态配置会从 `config.toml` 或环境变量迁移到数据库。之后，数据库中的配置优先。

## 配置方式

### TOML 配置文件（启动配置）

```toml
# config.toml
[server]
host = "127.0.0.1"
port = 8080

[database]
database_url = "shortlinks.db"

[cache]
type = "memory"

[logging]
level = "info"
```

### 环境变量

```bash
# .env 或系统环境变量
SERVER_HOST=127.0.0.1
SERVER_PORT=8080
DATABASE_URL=shortlinks.db
```

### 管理面板（动态配置）

通过 Web 管理面板或 API 修改动态配置：

```bash
# 获取所有配置
curl -H "Authorization: Bearer $ADMIN_TOKEN" http://localhost:8080/admin/config

# 获取单个配置
curl -H "Authorization: Bearer $ADMIN_TOKEN" http://localhost:8080/admin/config/features.random_code_length

# 更新配置
curl -X PUT \
     -H "Authorization: Bearer $ADMIN_TOKEN" \
     -H "Content-Type: application/json" \
     -d '{"value": "8"}' \
     http://localhost:8080/admin/config/features.random_code_length

# 重载配置
curl -X POST \
     -H "Authorization: Bearer $ADMIN_TOKEN" \
     http://localhost:8080/admin/config/reload

# 查询配置历史（可选 limit 参数，默认 20）
curl -H "Authorization: Bearer $ADMIN_TOKEN" \
     "http://localhost:8080/admin/config/features.random_code_length/history?limit=10"
```

**配置历史响应格式**：

```json
{
  "code": 0,
  "data": [{
    "id": 1,
    "config_key": "features.random_code_length",
    "old_value": "6",
    "new_value": "8",
    "changed_at": "2024-12-15T14:30:22Z",
    "changed_by": null
  }]
}
```

> **注意**：敏感配置（如 `api.admin_token`、`api.jwt_secret`）在 API 响应中会自动掩码为 `********`。

## 启动配置参数

这些配置存储在 `config.toml` 中，修改后需要重启服务。

### 服务器配置

| 参数 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `SERVER_HOST` | String | `127.0.0.1` | 监听地址 |
| `SERVER_PORT` | Integer | `8080` | 监听端口 |
| `UNIX_SOCKET` | String | *(空)* | Unix 套接字路径（设置后忽略 HOST/PORT） |
| `CPU_COUNT` | Integer | *(自动)* | 工作线程数量（默认为 CPU 核心数） |

### 数据库配置

| 参数 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `DATABASE_URL` | String | `shortlinks.db` | 数据库连接 URL 或文件路径 |
| `DATABASE_POOL_SIZE` | Integer | `10` | 数据库连接池大小 |
| `DATABASE_TIMEOUT` | Integer | `30` | 数据库连接超时（秒） |

> 详细的存储后端配置请参考 [存储后端](/config/storage)

### 缓存配置

| 参数 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `CACHE_TYPE` | String | `memory` | 缓存类型：memory, redis |
| `CACHE_DEFAULT_TTL` | Integer | `3600` | 默认缓存过期时间（秒） |
| `REDIS_URL` | String | `redis://127.0.0.1:6379/` | Redis 连接地址 |
| `REDIS_KEY_PREFIX` | String | `shortlinker:` | Redis 键前缀 |
| `MEMORY_MAX_CAPACITY` | Integer | `10000` | 内存缓存最大容量 |

### 日志配置

| 参数 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `RUST_LOG` | String | `info` | 日志等级：error, warn, info, debug, trace |
| `LOG_FORMAT` | String | `text` | 日志格式：text, json |
| `LOG_FILE` | String | *(空)* | 日志文件路径（空则输出到控制台） |

## 动态配置参数

这些配置存储在数据库中，可通过管理面板在运行时修改。

### API 配置

| 配置键 | 类型 | 默认值 | 需要重启 | 说明 |
|--------|------|--------|----------|------|
| `api.admin_token` | String | *(空)* | 否 | 管理 API 令牌 |
| `api.health_token` | String | *(空)* | 否 | 健康检查令牌 |
| `api.jwt_secret` | String | *(自动生成)* | 否 | JWT 密钥 |
| `api.access_token_minutes` | Integer | `15` | 否 | Access Token 有效期（分钟） |
| `api.refresh_token_days` | Integer | `7` | 否 | Refresh Token 有效期（天） |
| `api.access_cookie_name` | String | `shortlinker_access` | 是 | Access Token Cookie 名称 |
| `api.refresh_cookie_name` | String | `shortlinker_refresh` | 是 | Refresh Token Cookie 名称 |
| `api.cookie_secure` | Boolean | `false` | 是 | 是否仅 HTTPS 传输 |
| `api.cookie_same_site` | String | `Lax` | 是 | Cookie SameSite 策略 |
| `api.cookie_domain` | String | *(空)* | 是 | Cookie 域名 |

### 路由配置

| 配置键 | 类型 | 默认值 | 需要重启 | 说明 |
|--------|------|--------|----------|------|
| `routes.admin_prefix` | String | `/admin` | 是 | 管理 API 路由前缀 |
| `routes.health_prefix` | String | `/health` | 是 | 健康检查路由前缀 |
| `routes.frontend_prefix` | String | `/panel` | 是 | 前端面板路由前缀 |

### 功能配置

| 配置键 | 类型 | 默认值 | 需要重启 | 说明 |
|--------|------|--------|----------|------|
| `features.enable_admin_panel` | Boolean | `false` | 是 | 启用 Web 管理面板 |
| `features.random_code_length` | Integer | `6` | 否 | 随机短码长度 |
| `features.default_url` | String | `https://esap.cc/repo` | 否 | 默认跳转 URL |

### 点击统计配置

| 配置键 | 类型 | 默认值 | 需要重启 | 说明 |
|--------|------|--------|----------|------|
| `click.enable_tracking` | Boolean | `true` | 否 | 启用点击统计 |
| `click.flush_interval` | Integer | `30` | 否 | 刷新间隔（秒） |
| `click.max_clicks_before_flush` | Integer | `100` | 否 | 刷新前最大点击数 |

## 配置优先级

1. **数据库配置**（动态配置，最高优先级）
2. **环境变量**
3. **TOML 配置文件**
4. **程序默认值**（最低优先级）

> **注意**：动态配置只在首次启动时从环境变量/TOML 迁移到数据库。之后，数据库中的值优先。

## 配置示例

### 开发环境

```bash
# 基础配置
SERVER_HOST=127.0.0.1
SERVER_PORT=8080
RUST_LOG=debug

# 存储配置 - SQLite 便于调试
DATABASE_URL=dev-links.db

# API 配置 - 开发环境使用简单 token
ADMIN_TOKEN=dev_admin
HEALTH_TOKEN=dev_health
```

### 生产环境

```toml
# config.toml
[server]
host = "127.0.0.1"
port = 8080
cpu_count = 8

[database]
database_url = "/data/shortlinks.db"
pool_size = 20
timeout = 60

[cache]
type = "memory"
default_ttl = 7200

[cache.memory]
max_capacity = 50000

[logging]
level = "info"
format = "json"
file = "/var/log/shortlinker/app.log"
enable_rotation = true
```

### Docker 环境

```bash
# 服务器配置
SERVER_HOST=0.0.0.0
SERVER_PORT=8080
CPU_COUNT=4

# 存储配置
DATABASE_URL=/data/links.db

# 首次启动时的动态配置
ADMIN_TOKEN=secure_admin_token_here
HEALTH_TOKEN=secure_health_token_here
ENABLE_ADMIN_PANEL=true
```

## 热重载

### 支持热重载的配置

- ✅ 大部分动态配置（标记为"不需要重启"的）
- ✅ 短链接数据

### 不支持热重载的配置

- ❌ 服务器地址和端口
- ❌ 数据库连接
- ❌ 缓存类型
- ❌ 路由前缀
- ❌ Cookie 配置

### 重载方法

```bash
# Unix 系统 - 发送 SIGUSR1 信号
kill -USR1 $(cat shortlinker.pid)

# 通过 API
curl -X POST \
     -H "Authorization: Bearer $ADMIN_TOKEN" \
     http://localhost:8080/admin/config/reload
```

## 下一步

- 📋 查看 [存储后端配置](/config/storage) 了解详细存储选项
- 🚀 学习 [部署配置](/deployment/) 生产环境设置
- 🛡️ 了解 [Admin API](/api/admin) 管理接口使用
- 🏥 了解 [健康检查 API](/api/health) 监控接口使用
