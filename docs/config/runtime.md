# 运行时配置参数

本页聚焦动态配置（数据库存储）及其更新方式。

## 通过管理面板 / API 修改

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
# 说明：CLI `config set/reset/import` 已会自动尝试通过 IPC 触发重载；
# 若 IPC 不可达（服务未运行、ipc.enabled=false、socket 路径不一致等），可手动调用该接口。
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
  "message": "OK",
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

> 说明：这些前缀会被视为“保留短码前缀”。短链接 `code` 不能等于这些前缀（去掉开头 `/` 后的值），也不能以 `{prefix}/` 开头，否则会与系统路由冲突。

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
> - `click_logs.source` 的推导规则为：优先读取请求 Query 中的 `utm_source`；若不存在则尝试从 `Referer` 提取域名并记录为 `ref:{domain}`；两者都没有则记录为 `direct`。
> - 数据清理任务由 `analytics.enable_auto_rollup` 控制：启用后会按 `analytics.log_retention_days` / `analytics.hourly_retention_days` / `analytics.daily_retention_days` 定期清理过期数据。
> - 当前实现中，保留天数参数在后台任务启动时读取；修改保留天数后，可能需要重启服务才能让清理任务使用新值。

### UTM 参数透传配置

| 配置键 | 类型 | 默认值 | 需要重启 | 说明 |
|--------|------|--------|----------|------|
| `utm.enable_passthrough` | Boolean | `false` | 否 | 启用重定向时 UTM 参数透传（仅透传 `utm_source` / `utm_medium` / `utm_campaign` / `utm_term` / `utm_content`） |

> **说明**：
> - 默认关闭。开启后，仅当请求 URL 中存在上述 UTM 参数时，才会附加到目标 URL。
> - 目标 URL 已有 Query 时使用 `&` 追加；没有 Query 时使用 `?` 追加。
> - 透传参数会进行 URL 解码后再编码，确保 `Location` 头中的 URL 合法。

### CORS 跨域配置

| 配置键 | 类型 | 默认值 | 需要重启 | 说明 |
|--------|------|--------|----------|------|
| `cors.enabled` | Boolean | `false` | 是 | 启用 CORS（禁用时不添加 CORS 头，浏览器维持同源策略） |
| `cors.allowed_origins` | StringArray | `[]` | 是 | 允许的来源（JSON 数组；`["*"]` = 允许任意来源；空数组 = 仅同源/不允许跨域） |
| `cors.allowed_methods` | EnumArray | `["GET","POST","PUT","DELETE","PATCH","HEAD","OPTIONS"]` | 是 | 允许的 HTTP 方法 |
| `cors.allowed_headers` | StringArray | `["Content-Type","Authorization","Accept"]` | 是 | 允许的请求头（跨域 + Cookie 写操作时，通常还需要加上 `X-CSRF-Token`） |
| `cors.max_age` | Integer | `3600` | 是 | 预检请求缓存时间（秒） |
| `cors.allow_credentials` | Boolean | `false` | 是 | 允许携带凭证（跨域 Cookie 场景需要开启；出于安全原因不建议与 `["*"]` 同时使用） |

> 配置优先级请参考 [配置指南](/config/) 中“配置优先级”章节，避免与配置指南页重复维护。
