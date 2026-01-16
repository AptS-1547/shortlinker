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
# 配置管理接口属于 Admin API，需要先登录获取 Cookie
curl -sS -X POST \
     -H "Content-Type: application/json" \
     -c cookies.txt \
     -d '{"password":"your_admin_token"}' \
     http://localhost:8080/admin/v1/auth/login

# 获取所有配置
curl -sS -b cookies.txt http://localhost:8080/admin/v1/config

# 获取单个配置
curl -sS -b cookies.txt http://localhost:8080/admin/v1/config/features.random_code_length

# 更新配置
curl -X PUT \
     -b cookies.txt \
     -H "Content-Type: application/json" \
     -d '{"value": "8"}' \
     http://localhost:8080/admin/v1/config/features.random_code_length

# 重载配置
curl -X POST \
     -b cookies.txt \
     http://localhost:8080/admin/v1/config/reload

# 查询配置历史（可选 limit 参数，默认 20）
curl -sS -b cookies.txt \
     "http://localhost:8080/admin/v1/config/features.random_code_length/history?limit=10"
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

> 日志格式与文件输出通过 `config.toml` 的 `[logging]` 配置（如 `logging.format`、`logging.file`）设置，当前版本未提供对应的环境变量覆盖。

## 动态配置参数

这些配置存储在数据库中，可通过管理面板在运行时修改。

### API 配置

| 配置键 | 类型 | 默认值 | 需要重启 | 说明 |
|--------|------|--------|----------|------|
| `api.admin_token` | String | *(自动生成)* | 否 | 管理员登录密码（用于 `POST /admin/v1/auth/login`） |
| `api.health_token` | String | *(空)* | 否 | Health API 的 Bearer Token（`Authorization: Bearer ...`，适合监控/探针；为空则仅支持 JWT Cookie）。注意：当 `api.admin_token` 与 `api.health_token` 都为空时，Health 端点会返回 `404` 视为禁用 |
| `api.jwt_secret` | String | *(自动生成)* | 否 | JWT 密钥 |
| `api.access_token_minutes` | Integer | `15` | 否 | Access Token 有效期（分钟） |
| `api.refresh_token_days` | Integer | `7` | 否 | Refresh Token 有效期（天） |
| `api.access_cookie_name` | String | `shortlinker_access` | 是 | Access Token Cookie 名称 |
| `api.refresh_cookie_name` | String | `shortlinker_refresh` | 是 | Refresh Token Cookie 名称 |
| `api.cookie_secure` | Boolean | `false` | 否 | 是否仅 HTTPS 传输（对浏览器生效；修改后建议重新登录获取新 Cookie） |
| `api.cookie_same_site` | String | `Lax` | 否 | Cookie SameSite 策略（修改后建议重新登录获取新 Cookie） |
| `api.cookie_domain` | String | *(空)* | 否 | Cookie 域名（修改后建议重新登录获取新 Cookie） |

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
| `click.enable_tracking` | Boolean | `true` | 是 | 启用点击统计 |
| `click.flush_interval` | Integer | `30` | 是 | 刷新间隔（秒） |
| `click.max_clicks_before_flush` | Integer | `100` | 是 | 刷新前最大点击数 |

### CORS 跨域配置

| 配置键 | 类型 | 默认值 | 需要重启 | 说明 |
|--------|------|--------|----------|------|
| `cors.enabled` | Boolean | `false` | 是 | 启用 CORS（禁用时不添加 CORS 头，浏览器维持同源策略） |
| `cors.allowed_origins` | Json | `[]` | 是 | 允许的来源（JSON 数组；`["*"]` = 允许任意来源；空数组 = 仅同源/不允许跨域） |
| `cors.allowed_methods` | Json | `["GET","POST","PUT","DELETE","OPTIONS","HEAD"]` | 是 | 允许的 HTTP 方法 |
| `cors.allowed_headers` | Json | `["Content-Type","Authorization","Accept"]` | 是 | 允许的请求头 |
| `cors.max_age` | Integer | `3600` | 是 | 预检请求缓存时间（秒） |
| `cors.allow_credentials` | Boolean | `false` | 是 | 允许携带凭证（跨域 Cookie 场景需要开启；出于安全原因不建议与 `["*"]` 同时使用） |

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
ENABLE_ADMIN_PANEL=true
```

## 热重载

Shortlinker 的“热重载/热生效”主要分两类：

1. **短链接数据热重载**：让服务重新从存储加载短链接并重建缓存（适用于 CLI/TUI 直接写数据库后通知服务刷新缓存）。
2. **运行时配置热生效**：通过 Admin API 更新“无需重启”的配置时，会直接同步到内存配置并立即生效。

### 支持热生效/热重载的内容

- ✅ 短链接数据（缓存重建）
- ✅ 标记为“无需重启”的运行时配置（通过 Admin API 更新时立即生效）

### 不支持热重载的配置

- ❌ 服务器地址和端口
- ❌ 数据库连接
- ❌ 缓存类型
- ❌ 路由前缀
- ❌ Cookie 配置

### 重载方法

```bash
# 1) 重载短链接数据/缓存（Unix 系统 - 发送 SIGUSR1 信号）
# 注意：SIGUSR1 只会触发短链接数据/缓存重载，不会重载运行时配置
kill -USR1 $(cat shortlinker.pid)

# 2) 重载运行时配置（通过 Admin API）
# 说明：如果你是通过 Admin API 直接更新配置（PUT /admin/v1/config/{key}），
#       且该配置“无需重启”，一般不需要额外 reload。
#       如果你是直接改数据库（例如使用 `./shortlinker config set`），可以调用该接口让服务重新从 DB 加载配置。
#
# 先登录获取 cookies（如已存在 cookies.txt 可跳过）
curl -sS -X POST \
     -H "Content-Type: application/json" \
     -c cookies.txt \
     -d '{"password":"your_admin_token"}' \
     http://localhost:8080/admin/v1/auth/login

curl -X POST \
     -b cookies.txt \
     http://localhost:8080/admin/v1/config/reload
```

## 下一步

- 📋 查看 [存储后端配置](/config/storage) 了解详细存储选项
- 🚀 学习 [部署配置](/deployment/) 生产环境设置
- 🛡️ 了解 [Admin API](/api/admin) 管理接口使用
- 🏥 了解 [健康检查 API](/api/health) 监控接口使用
