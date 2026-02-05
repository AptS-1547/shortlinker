# 配置指南

Shortlinker 的配置分为两类：

- **启动配置**：存储在 `config.toml` 文件中，修改后需要重启服务
- **动态配置**：存储在数据库中，可通过管理面板在运行时修改

## 配置架构

```
config.toml (启动时读取)
       ↓
StaticConfig (启动配置，内存)
       ↓
   数据库 (短链接数据 + 运行时配置)
       ↓
RuntimeConfig (运行时配置缓存，内存)
       ↓
    业务逻辑（路由/鉴权/缓存等）
```

首次启动时，服务会根据代码内置的配置定义把**运行时配置默认值**初始化到数据库，并加载到内存缓存；之后以数据库中的值为准。  
当前版本不会从 `config.toml` 或环境变量“迁移/覆盖”运行时配置。

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

> 说明：
> - 后端只会从**当前工作目录**读取 `config.toml`（相对路径）。
> - 可用 `./shortlinker generate-config config.toml` 生成模板（只包含启动配置）。
> - 支持通过环境变量覆盖启动配置：前缀 `SL__`，层级分隔符 `__`（优先级：ENV > `config.toml` > 默认值）。例如：`SL__SERVER__PORT=9999`。
>   - 程序启动时会尝试加载当前目录的 `.env`（不会覆盖已存在的环境变量），因此也可以在 `.env` 中写入上述 `SL__...` 变量。

### 管理面板（动态配置）

通过 Web 管理面板或 API 修改动态配置：

```bash
# 配置管理接口属于 Admin API，需要先登录获取 Cookie
curl -sS -X POST \
     -H "Content-Type: application/json" \
     -c cookies.txt \
     -d '{"password":"your_admin_token"}' \
     http://localhost:8080/admin/v1/auth/login

# 提取 CSRF Token（用于 PUT/POST/DELETE 等写操作）
CSRF_TOKEN=$(awk '$6=="csrf_token"{print $7}' cookies.txt | tail -n 1)

# 获取所有配置
curl -sS -b cookies.txt http://localhost:8080/admin/v1/config

# 获取单个配置
curl -sS -b cookies.txt http://localhost:8080/admin/v1/config/features.random_code_length

# 更新配置
curl -X PUT \
     -b cookies.txt \
     -H "X-CSRF-Token: ${CSRF_TOKEN}" \
     -H "Content-Type: application/json" \
     -d '{"value": "8"}' \
     http://localhost:8080/admin/v1/config/features.random_code_length

# 重载配置
curl -X POST \
     -b cookies.txt \
     -H "X-CSRF-Token: ${CSRF_TOKEN}" \
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

> **注意**：敏感配置（如 `api.admin_token`、`api.jwt_secret`）在 API 响应中会自动掩码为 `[REDACTED]`。

## 启动配置参数

这些配置存储在 `config.toml` 中，修改后需要重启服务。

### 服务器配置

| TOML 键 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `server.host` | String | `127.0.0.1` | 监听地址（Docker 中通常用 `0.0.0.0`） |
| `server.port` | Integer | `8080` | 监听端口 |
| `server.unix_socket` | String | *(空)* | Unix 套接字路径（设置后忽略 `server.host`/`server.port`） |
| `server.cpu_count` | Integer | *(自动)* | Worker 数量（默认 CPU 核心数，最大 32） |

### 数据库配置

| TOML 键 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `database.database_url` | String | `shortlinks.db` | 数据库连接 URL 或文件路径（后端会自动从该值推断数据库类型） |
| `database.pool_size` | Integer | `10` | 连接池大小（仅 MySQL/PostgreSQL 生效；SQLite 使用内置池配置） |
| `database.timeout` | Integer | `30` | *(当前版本暂未使用；连接超时固定为 8s)* |
| `database.retry_count` | Integer | `3` | 部分数据库操作的重试次数 |
| `database.retry_base_delay_ms` | Integer | `100` | 重试基础延迟（毫秒） |
| `database.retry_max_delay_ms` | Integer | `2000` | 重试最大延迟（毫秒） |

> 详细的存储后端配置请参考 [存储后端](/config/storage)

### 缓存配置

| TOML 键 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `cache.type` | String | `memory` | 缓存类型：`memory` / `redis` |
| `cache.default_ttl` | Integer | `3600` | 默认缓存过期时间（秒） |
| `cache.redis.url` | String | `redis://127.0.0.1:6379/` | Redis 连接地址 |
| `cache.redis.key_prefix` | String | `shortlinker:` | Redis 键前缀 |
| `cache.memory.max_capacity` | Integer | `10000` | 内存缓存最大容量 |

### 日志配置

| TOML 键 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `logging.level` | String | `info` | 日志等级：error / warn / info / debug / trace |
| `logging.format` | String | `text` | 输出格式：`text` / `json` |
| `logging.file` | String | *(空)* | 日志文件路径（为空则输出到 stdout） |
| `logging.max_backups` | Integer | `5` | 日志轮转保留文件数 |
| `logging.enable_rotation` | Boolean | `true` | 是否启用轮转（当前为按天轮转） |
| `logging.max_size` | Integer | `100` | *(当前版本暂未使用；轮转按天而非按大小)* |

> 日志格式与文件输出通过 `config.toml` 的 `[logging]` 配置设置（例如 `logging.format`、`logging.file`）。

### GeoIP（分析）配置

| TOML 键 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `analytics.maxminddb_path` | String | *(空)* | MaxMindDB 文件路径（GeoLite2-City.mmdb，可选；可读时优先使用本地解析） |
| `analytics.geoip_api_url` | String | `http://ip-api.com/json/{ip}?fields=status,countryCode,city` | 外部 GeoIP API URL（MaxMindDB 不可用时 fallback；`{ip}` 为占位符） |

> 说明：
> - Provider 选择：`analytics.maxminddb_path` 可读时使用本地 MaxMind；否则使用外部 API（`analytics.geoip_api_url`）。
> - 外部 API Provider 内置缓存（不可配置）：LRU 最大 10000 条，TTL 15 分钟（包含失败的负缓存）；同一 IP 的并发查询会合并为一次请求；单次请求超时 2 秒。

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
| `api.cookie_secure` | Boolean | `true` | 否 | 是否仅 HTTPS 传输（对浏览器生效；修改后建议重新登录获取新 Cookie） |
| `api.cookie_same_site` | String | `Lax` | 否 | Cookie SameSite 策略（修改后建议重新登录获取新 Cookie） |
| `api.cookie_domain` | String | *(空)* | 否 | Cookie 域名（修改后建议重新登录获取新 Cookie） |
| `api.trusted_proxies` | StringArray | `[]` | 否 | 可信代理 IP 或 CIDR 列表。<br>**智能检测**（默认）：留空时，连接来自私有 IP（RFC1918：10.0.0.0/8、172.16.0.0/12、192.168.0.0/16）或 localhost 将自动信任 X-Forwarded-For，适合 Docker/nginx 反向代理。<br>**显式配置**：设置后仅信任列表中的 IP，如 `["10.0.0.1", "172.17.0.0/16"]`。<br>**安全提示**：公网 IP 默认不信任 X-Forwarded-For，防止伪造。 |

> 提示：
> - Cookie 名称当前为固定值：`shortlinker_access` / `shortlinker_refresh` / `csrf_token`（不可配置）。
> - `api.admin_token` 在数据库中存储为 Argon2 哈希；推荐使用 `./shortlinker reset-password` 重置管理员密码。
> - 首次启动时会自动生成一个随机管理员密码并写入 `admin_token.txt`（若文件不存在；保存后请删除该文件）。

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

### 详细分析配置

| 配置键 | 类型 | 默认值 | 需要重启 | 说明 |
|--------|------|--------|----------|------|
| `analytics.enable_detailed_logging` | Boolean | `false` | 是 | 启用详细点击日志（写入 click_logs 表） |
| `analytics.enable_auto_rollup` | Boolean | `true` | 是 | 启用自动数据清理/汇总表清理任务（后台任务默认每 4 小时运行一次） |
| `analytics.log_retention_days` | Integer | `30` | 否 | 原始点击日志保留天数（由后台任务自动清理；需要启用 `analytics.enable_auto_rollup`） |
| `analytics.hourly_retention_days` | Integer | `7` | 否 | 小时汇总保留天数（清理 `click_stats_hourly` / `click_stats_global_hourly`；需要启用 `analytics.enable_auto_rollup`） |
| `analytics.daily_retention_days` | Integer | `365` | 否 | 天汇总保留天数（清理 `click_stats_daily`；需要启用 `analytics.enable_auto_rollup`） |
| `analytics.enable_ip_logging` | Boolean | `true` | 否 | 是否记录 IP 地址 |
| `analytics.enable_geo_lookup` | Boolean | `false` | 否 | 是否启用地理位置解析 |

> **注意**：
> - 启用 `analytics.enable_detailed_logging` 后（需要重启生效），每次点击都会记录详细信息到 `click_logs` 表（时间、来源、`user_agent_hash` 等）。User-Agent 原文会去重存储在 `user_agents` 表并通过 hash 关联（用于设备/浏览器统计）。
> - 若同时开启 `analytics.enable_ip_logging` 才会记录 IP；开启 `analytics.enable_geo_lookup` 才会进行 GeoIP 解析（并使用启动配置 `[analytics]` 选择 provider）。这些数据用于 Analytics API 的趋势分析、来源统计和地理分布等功能。
> - 数据清理任务由 `analytics.enable_auto_rollup` 控制：启用后会按 `analytics.log_retention_days` / `analytics.hourly_retention_days` / `analytics.daily_retention_days` 定期清理过期数据。
> - 当前实现中，保留天数参数在后台任务启动时读取；修改保留天数后，可能需要重启服务才能让清理任务使用新值。

### CORS 跨域配置

| 配置键 | 类型 | 默认值 | 需要重启 | 说明 |
|--------|------|--------|----------|------|
| `cors.enabled` | Boolean | `false` | 是 | 启用 CORS（禁用时不添加 CORS 头，浏览器维持同源策略） |
| `cors.allowed_origins` | StringArray | `[]` | 是 | 允许的来源（JSON 数组；`["*"]` = 允许任意来源；空数组 = 仅同源/不允许跨域） |
| `cors.allowed_methods` | EnumArray | `["GET","POST","PUT","DELETE","PATCH","HEAD","OPTIONS"]` | 是 | 允许的 HTTP 方法 |
| `cors.allowed_headers` | StringArray | `["Content-Type","Authorization","Accept"]` | 是 | 允许的请求头（跨域 + Cookie 写操作时，通常还需要加上 `X-CSRF-Token`） |
| `cors.max_age` | Integer | `3600` | 是 | 预检请求缓存时间（秒） |
| `cors.allow_credentials` | Boolean | `false` | 是 | 允许携带凭证（跨域 Cookie 场景需要开启；出于安全原因不建议与 `["*"]` 同时使用） |

## 配置优先级

1. **数据库（运行时配置）**：`api.*` / `routes.*` / `features.*` / `click.*` / `cors.*` / `analytics.*`（点击分析相关）
2. **环境变量（启动配置覆盖）**：`SL__...`（会覆盖 `config.toml` 中的 `[server]` / `[database]` / `[cache]` / `[logging]` / `[analytics]`）
3. **`config.toml`（启动配置）**：`[server]` / `[database]` / `[cache]` / `[logging]` / `[analytics]`（GeoIP provider 相关）
4. **程序默认值**：当数据库、环境变量或 `config.toml` 中未设置时使用

> 说明：环境变量仅影响**启动配置**；当前版本不会从环境变量或 `config.toml` 自动“迁移/覆盖”运行时配置到数据库。

## 安全最佳实践

### 登录限流配置

Shortlinker 使用智能代理检测进行登录限流 IP 提取，兼顾安全性和易用性。

**直连部署**（无反向代理）：
- 无需额外配置，公网 IP 不会信任 `X-Forwarded-For`，安全且自动

**反向代理部署**（Nginx/Caddy/Docker）：
- **自动检测**（推荐）：无需配置 `api.trusted_proxies`，连接来自私有 IP（10.x、172.16-31.x、192.168.x）或 localhost 时自动信任 `X-Forwarded-For`
- **显式配置**：如需精确控制，可在管理面板配置 `api.trusted_proxies`，列出可信代理的 IP 或 CIDR

**Unix Socket 连接**（nginx 同机器）：
- 自动使用 `X-Forwarded-For` 提取客户端真实 IP
- 确保 nginx 配置了 `proxy_set_header X-Forwarded-For $remote_addr;`

示例配置（可选）：

```bash
CSRF_TOKEN=$(awk '$6=="csrf_token"{print $7}' cookies.txt | tail -n 1)

# Nginx 在本地
curl -X PUT -b cookies.txt \
     -H "X-CSRF-Token: ${CSRF_TOKEN}" \
     -H "Content-Type: application/json" \
     -d '{"value": "[\"127.0.0.1\"]"}' \
     http://localhost:8080/admin/v1/config/api.trusted_proxies

# Cloudflare CDN（使用 Cloudflare IP 段）
curl -X PUT -b cookies.txt \
     -H "X-CSRF-Token: ${CSRF_TOKEN}" \
     -H "Content-Type: application/json" \
     -d '{"value": "[\"103.21.244.0/22\", \"103.22.200.0/22\"]"}' \
     http://localhost:8080/admin/v1/config/api.trusted_proxies
```

> **注意**：
> - **智能检测模式**（默认）：适合绝大多数场景，但如果 shortlinker 直接绑定在 VPC 内网 IP 且无代理，建议显式配置 `trusted_proxies` 防止伪造攻击
> - **显式配置模式**：错误配置可能导致所有用户共享同一限流桶（代理 IP 未匹配）或重新引入绕过风险（信任了不安全的代理）
> - 查看启动日志确认当前检测模式：`Login rate limiting: Auto-detect mode enabled` 或 `Explicit trusted proxies configured`

### IPC Socket 权限

Unix socket 文件（`./shortlinker.sock`）权限已自动设置为 `0600`（仅属主可访问），防止本地其他用户绕过 Admin API。

如果需要允许特定用户访问 CLI：
```bash
# 方法 1: 使用 setfacl 添加访问权限
setfacl -m u:username:rw ./shortlinker.sock

# 方法 2: 使用用户组
chgrp shortlinker-users ./shortlinker.sock
chmod 660 ./shortlinker.sock
```

## 配置示例

### 开发环境

```toml
# config.toml（开发）
[server]
host = "127.0.0.1"
port = 8080

[database]
database_url = "dev-links.db"

[logging]
level = "debug"
```

> 运行时配置（如 `features.enable_admin_panel`、`api.health_token`）请通过 Admin API 或 CLI 写入数据库；`api.admin_token` 请使用 `./shortlinker reset-password` 重置。

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

Docker 场景建议通过**挂载配置文件**来设置启动配置（尤其是把 `server.host` 设为 `0.0.0.0`）：

```toml
# /config.toml（容器内）
[server]
host = "0.0.0.0"
port = 8080

[database]
database_url = "sqlite:///data/links.db"
```

运行时配置（写入数据库）可在容器内用 CLI 设置；其中标记为“需要重启”的配置需要重启容器生效：

```bash
# 启用管理面板（需要重启）
/shortlinker config set features.enable_admin_panel true

# 配置 Health Bearer Token（无需重启）
/shortlinker config set api.health_token "your_health_token"
```

## 热重载

Shortlinker 的“热重载/热生效”主要分两类：

1. **短链接数据热重载**：让服务重新从存储加载短链接并重建缓存（适用于 CLI/TUI 直接写数据库后通知服务刷新缓存）。
2. **运行时配置热生效**：通过 Admin API 更新“无需重启”的配置时，会直接同步到内存配置并立即生效。

### 支持热生效/热重载的内容

- ✅ 短链接数据（缓存重建）
- ✅ 标记为“无需重启”的运行时配置（通过 Admin API 更新时立即生效）
- ✅ Cookie 配置（`api.cookie_*`）：对新下发的 Cookie 生效，修改后建议重新登录获取新 Cookie

### 不支持热重载的配置

- ❌ 服务器地址和端口
- ❌ 数据库连接
- ❌ 缓存类型
- ❌ 路由前缀

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

CSRF_TOKEN=$(awk '$6=="csrf_token"{print $7}' cookies.txt | tail -n 1)

curl -X POST \
     -b cookies.txt \
     -H "X-CSRF-Token: ${CSRF_TOKEN}" \
     http://localhost:8080/admin/v1/config/reload
```

## 下一步

- 📋 查看 [存储后端配置](/config/storage) 了解详细存储选项
- 🚀 学习 [部署配置](/deployment/) 生产环境设置
- 🛡️ 了解 [Admin API](/api/admin) 管理接口使用
- 🏥 了解 [健康检查 API](/api/health) 监控接口使用
