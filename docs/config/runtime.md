# 运行时配置参数

本页聚焦动态配置（数据库存储）及其更新方式。

## 通过管理面板 / API 修改

通过 Web 管理面板或 API 修改动态配置：

> 首次部署请先设置管理员密码：`./shortlinker reset-password`（`api.admin_token` 默认为空，未设置时 Admin API 不可用）。

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
# 说明：CLI `config set/reset` 仅在“无需重启”的配置上自动尝试通过 IPC 热重载；
# `config import` 会在导入后统一尝试一次热重载。
# 若 IPC 不可达（服务未运行、ipc.enabled=false、socket 路径不一致等），可手动调用该接口。
curl -X POST \
     -b cookies.txt \
     -H "X-CSRF-Token: ${CSRF_TOKEN}" \
     http://localhost:8080/admin/v1/config/reload

# 查询配置历史（可选 limit 参数，默认 20，最大 100）
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

> **配置 Action 说明**：当前仅 `api.jwt_secret` 支持 `generate_token` Action。


## 动态配置参数

这些配置存储在数据库中，可通过管理面板在运行时修改。

### API 配置

| 配置键 | 类型 | 默认值 | 需要重启 | 说明 |
|--------|------|--------|----------|------|
| `api.admin_token` | String | *(空)* | 否 | 管理员登录密码（用于 `POST /admin/v1/auth/login`）。默认为空；为空时 Admin API 与前端面板会返回 `404` |
| `api.health_token` | String | *(空)* | 否 | Health API 的 Bearer Token（`Authorization: Bearer ...`，适合监控/探针；为空则仅支持 JWT Cookie）。注意：当 `api.admin_token` 与 `api.health_token` 都为空时，Health 端点会返回 `404` 视为禁用 |
| `api.jwt_secret` | String | *(自动生成)* | 是 | JWT 密钥 |
| `api.access_token_minutes` | Integer | `15` | 是 | Access Token 有效期（分钟） |
| `api.refresh_token_days` | Integer | `7` | 是 | Refresh Token 有效期（天） |
| `api.cookie_secure` | Boolean | `true` | 否 | 是否仅 HTTPS 传输（对浏览器生效；修改后建议重新登录获取新 Cookie） |
| `api.cookie_same_site` | Enum | `Lax` | 否 | Cookie SameSite 策略：`Strict` / `Lax` / `None`（修改后建议重新登录获取新 Cookie） |
| `api.cookie_domain` | String | *(空)* | 否 | Cookie 域名（修改后建议重新登录获取新 Cookie） |
| `api.trusted_proxies` | StringArray | `[]` | 否 | 可信代理 IP 或 CIDR 列表。<br>**智能检测**（默认）：留空时，连接来自私有/本地地址会自动信任 X-Forwarded-For（IPv4: RFC1918 + `127.0.0.1`；IPv6: `::1`、`fc00::/7`、`fe80::/10`），适合 Docker/nginx 反向代理。<br>**显式配置**：设置后仅信任列表中的 IP，如 `["10.0.0.1", "172.17.0.0/16"]`。<br>**安全提示**：公网 IP 默认不信任 X-Forwarded-For，防止伪造。 |

> 提示：
> - Cookie 名称当前为固定值：`shortlinker_access` / `shortlinker_refresh` / `csrf_token`（不可配置）。
> - `api.admin_token` 在数据库中存储为 Argon2 哈希；推荐使用 `./shortlinker reset-password` 重置管理员密码。
> - 当前版本不会自动生成管理员密码文件；首次部署请先执行 `./shortlinker reset-password`。
> - 当前实现中，JWT 服务会在首次使用时读取配置并在进程内缓存（`OnceLock`）；`api.jwt_secret`、`api.access_token_minutes`、`api.refresh_token_days` 已标记为“需要重启”，修改后需重启服务才会用于后续签发/校验 Token。

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

### 缓存维护配置

| 配置键 | 类型 | 默认值 | 需要重启 | 说明 |
|--------|------|--------|----------|------|
| `cache.bloom_rebuild_interval` | Integer | `14400` | 是 | Bloom Filter 定时重建间隔（秒），`0` 表示禁用定时重建 |

> **说明**：
> - 该配置在服务启动时读取并创建后台定时任务；修改后需重启服务生效。
> - 定时任务会触发 `ReloadTarget::Data`，用于周期性重建 Bloom Filter，降低长期运行下的误判积累。


### 详细分析配置

| 配置键 | 类型 | 默认值 | 需要重启 | 说明 |
|--------|------|--------|----------|------|
| `analytics.enable_detailed_logging` | Boolean | `false` | 是 | 启用详细点击日志（写入 click_logs 表） |
| `analytics.enable_auto_rollup` | Boolean | `true` | 是 | 启用自动数据清理/汇总表清理任务（后台任务默认每 4 小时运行一次） |
| `analytics.log_retention_days` | Integer | `30` | 否 | 原始点击日志保留天数（由后台任务自动清理；需要启用 `analytics.enable_auto_rollup`） |
| `analytics.hourly_retention_days` | Integer | `7` | 否 | 小时汇总保留天数（清理 `click_stats_hourly` / `click_stats_global_hourly`；需要启用 `analytics.enable_auto_rollup`） |
| `analytics.daily_retention_days` | Integer | `365` | 否 | 天汇总保留天数（清理 `click_stats_daily`；需要启用 `analytics.enable_auto_rollup`） |
| `analytics.enable_ip_logging` | Boolean | `true` | 否 | 是否记录 IP 地址 |
| `analytics.enable_geo_lookup` | Boolean | `false` | 否 | GeoIP 预留开关（当前版本点击写入链路尚未消费该配置，`country/city` 默认空） |
| `analytics.sample_rate` | Float | `1.0` | 否 | 详细日志采样率（0.0-1.0；1.0=记录全部点击，0.1=记录约 10% 点击） |
| `analytics.max_log_rows` | Integer | `0` | 否 | `click_logs` 最大行数（0=不限制） |
| `analytics.max_rows_action` | Enum | `cleanup` | 否 | 超过 `max_log_rows` 时的动作：`cleanup`（删除最旧数据）或 `stop`（停止详细日志） |

> **注意**：
> - `analytics.enable_detailed_logging` 标记为“需要重启”：修改后需重启服务才会生效。启用后每次点击都会记录详细信息到 `click_logs` 表（时间、来源、`user_agent_hash` 等）。User-Agent 原文会去重存储在 `user_agents` 表并通过 hash 关联（用于设备/浏览器统计）。
> - `analytics.enable_ip_logging` 控制是否记录 IP。当前实现里 GeoIP 查询尚未接入点击写入链路，因此 `click_logs.country/city` 默认为空，地理分布接口可能返回空数组（除非历史数据已包含地理字段）。
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
> - 当前实现会直接拼接请求中的原始 UTM 片段（不做额外的 URL 解码/重编码）。

### CORS 跨域配置

| 配置键 | 类型 | 默认值 | 需要重启 | 说明 |
|--------|------|--------|----------|------|
| `cors.enabled` | Boolean | `false` | 是 | 启用 CORS（禁用时不添加 CORS 头，浏览器维持同源策略） |
| `cors.allowed_origins` | StringArray | `[]` | 是 | 允许的来源（JSON 数组；`["*"]` = 允许任意来源；空数组 = 仅同源/不允许跨域） |
| `cors.allowed_methods` | EnumArray | `["GET","POST","PUT","DELETE","PATCH","HEAD","OPTIONS"]` | 是 | 允许的 HTTP 方法 |
| `cors.allowed_headers` | StringArray | `["Content-Type","Authorization","Accept"]` | 是 | 允许的请求头（跨域 + Cookie 写操作时，通常还需要加上 `X-CSRF-Token`） |
| `cors.max_age` | Integer | `3600` | 是 | 预检请求缓存时间（秒） |
| `cors.allow_credentials` | Boolean | `false` | 是 | 允许携带凭证（跨域 Cookie 场景需要开启；与 `["*"]` 同时配置时，服务会出于安全考虑强制不启用 credentials） |

> 配置优先级请参考 [配置指南](/config/) 中“配置优先级”章节，避免与配置指南页重复维护。
