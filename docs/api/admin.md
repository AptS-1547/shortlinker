# Admin API 文档

Shortlinker 提供完整的 HTTP Admin API 用于管理短链接，支持 CRUD、批量操作、CSV 导入/导出，以及运行时配置管理。

## 配置方式

Admin API 相关配置属于**运行时配置（数据库）**，详细配置见 [配置指南](/config/)。

- `api.admin_token`：管理员登录密码（数据库中存储为 Argon2 哈希；首次启动会生成随机密码并写入 `admin_token.txt`，保存后请删除该文件；推荐用 `./shortlinker reset-password` 重置）
- `routes.admin_prefix`：路由前缀（默认 `/admin`，修改后需要重启）

> 实际接口路径固定包含 `/v1`，例如默认登录地址为 `http://localhost:8080/admin/v1/auth/login`。

## 鉴权方式（重要）

Admin API 支持两种鉴权方式：

1. **JWT Cookie（推荐用于浏览器/管理面板）**
   - Access Cookie：`shortlinker_access`（`Path=/`）
   - Refresh Cookie：`shortlinker_refresh`（`Path={ADMIN_ROUTE_PREFIX}/v1/auth`）
   - CSRF Cookie：`csrf_token`（`Path=/`，非 HttpOnly，用于前端读取）
2. **Bearer Token（用于 API 客户端，免 CSRF）**
   - `Authorization: Bearer <ACCESS_TOKEN>`（其中 `<ACCESS_TOKEN>` 是与 `shortlinker_access` Cookie 同一个 JWT Access Token）

> 说明：Cookie 名称当前为固定值（不可配置）；Cookie 有效期/SameSite/Secure/Domain 等可通过配置项 `api.*` 调整，见 [配置指南](/config/)。

### 1) 登录获取 Cookie

**POST** `/{ADMIN_ROUTE_PREFIX}/v1/auth/login`

请求体：
```json
{ "password": "your_admin_token" }
```

示例（把 cookie 保存到 `cookies.txt`）：
```bash
curl -sS -X POST \
  -H "Content-Type: application/json" \
  -c cookies.txt \
  -d '{"password":"your_admin_token"}' \
  http://localhost:8080/admin/v1/auth/login
```

> 该接口会通过 `Set-Cookie` 返回 access/refresh/csrf cookie；响应体不返回 token 字符串，只返回提示信息与过期时间。

### CSRF 防护（重要）

当你使用 **JWT Cookie** 鉴权访问写操作（`POST`/`PUT`/`DELETE`）时，需要同时提供：

- Cookie：`csrf_token`
- Header：`X-CSRF-Token: <csrf_token 的值>`

> 例外：`POST /auth/login`、`POST /auth/refresh`、`POST /auth/logout` 不需要 CSRF；`GET/HEAD/OPTIONS` 也不需要。  
> 如果你改用 `Authorization: Bearer <ACCESS_TOKEN>` 访问写操作，则不需要 CSRF。

示例（从 `cookies.txt` 中取出 CSRF Token）：

```bash
CSRF_TOKEN=$(awk '$6=="csrf_token"{print $7}' cookies.txt | tail -n 1)
```

### 2) 携带 Cookie 调用其它接口

示例：
```bash
curl -sS -b cookies.txt \
  http://localhost:8080/admin/v1/links
```

### 3) 刷新 Token

**POST** `/{ADMIN_ROUTE_PREFIX}/v1/auth/refresh`

示例（读取并更新 cookie）：
```bash
curl -sS -X POST \
  -b cookies.txt -c cookies.txt \
  http://localhost:8080/admin/v1/auth/refresh
```

### 4) 登出（清理 Cookie）

**POST** `/{ADMIN_ROUTE_PREFIX}/v1/auth/logout`

示例：
```bash
curl -sS -X POST -b cookies.txt -c cookies.txt \
  http://localhost:8080/admin/v1/auth/logout
```

## Base URL

默认：`http://your-domain:port/admin/v1`

> 若你修改了 `routes.admin_prefix`，只需把 `/admin` 替换为自定义前缀。

## 通用响应格式

大部分接口返回 JSON：
```json
{
  "code": 0,
  "data": { /* 响应数据 */ }
}
```

- `code = 0`：成功
- `code = 1`：业务错误（具体原因在 `data.error`）
- HTTP 状态码用于表达错误类型（如 `401/404/409/500`）

## 链接管理

### GET /links - 获取短链接列表（分页 + 过滤）

```bash
curl -sS -b cookies.txt \
  "http://localhost:8080/admin/v1/links?page=1&page_size=20"
```

**查询参数**：

| 参数 | 类型 | 说明 | 示例 |
|------|------|------|------|
| `page` | Integer | 页码（从 1 开始） | `?page=1` |
| `page_size` | Integer | 每页数量（1-100） | `?page_size=20` |
| `search` | String | 模糊搜索短码和目标 URL | `?search=github` |
| `created_after` | RFC3339 | 创建时间过滤（晚于） | `?created_after=2024-01-01T00:00:00Z` |
| `created_before` | RFC3339 | 创建时间过滤（早于） | `?created_before=2024-12-31T23:59:59Z` |
| `only_expired` | Boolean | 仅显示已过期 | `?only_expired=true` |
| `only_active` | Boolean | 仅显示未过期 | `?only_active=true` |

**响应格式**（分页）：
```json
{
  "code": 0,
  "data": [
    {
      "code": "github",
      "target": "https://github.com",
      "created_at": "2024-12-15T14:30:22Z",
      "expires_at": null,
      "password": null,
      "click_count": 42
    }
  ],
  "pagination": {
    "page": 1,
    "page_size": 20,
    "total": 42,
    "total_pages": 3
  }
}
```

### POST /links - 创建短链接

```bash
curl -sS -X POST \
  -b cookies.txt \
  -H "X-CSRF-Token: ${CSRF_TOKEN}" \
  -H "Content-Type: application/json" \
  -d '{"code":"github","target":"https://github.com"}' \
  http://localhost:8080/admin/v1/links
```

**请求体**：
```json
{
  "code": "github",
  "target": "https://github.com",
  "expires_at": "2024-12-31T23:59:59Z",
  "password": "secret123",
  "force": false
}
```

**说明**：
- `code`：短码（可选），不提供则自动生成随机短码
- `target`：目标 URL（必需）
- `expires_at`：过期时间（可选），支持相对时间（如 `"1d"`, `"7d"`, `"1w"`）或 RFC3339
- `force`：当 `code` 已存在时，是否覆盖（可选，默认 `false`；未开启时会返回 `409 Conflict`）
- `password`：密码保护字段（实验性）
  - 通过 Admin API 写入时会自动使用 Argon2 哈希（若传入的字符串已是 `$argon2...` 格式则会原样保存）
  - 当前版本重定向时不验证密码，仅存储

### GET /links/{code} - 获取指定短链接

```bash
curl -sS -b cookies.txt \
  http://localhost:8080/admin/v1/links/github
```

### PUT /links/{code} - 更新短链接

```bash
curl -sS -X PUT \
  -b cookies.txt \
  -H "X-CSRF-Token: ${CSRF_TOKEN}" \
  -H "Content-Type: application/json" \
  -d '{"target":"https://github.com/new-repo","expires_at":"30d"}' \
  http://localhost:8080/admin/v1/links/github
```

**请求体说明**：
```json
{
  "target": "https://new-url.com",
  "expires_at": "7d",
  "password": ""
}
```

**说明**：
- `target` 必填
- `expires_at` 不提供则保持原值
- `password`
  - 不提供：保持原值
  - 传空字符串 `""`：清除密码
  - 传明文：自动 Argon2 哈希后保存
  - 传 `$argon2...`：视为已哈希，原样保存

### DELETE /links/{code} - 删除短链接

```bash
curl -sS -X DELETE -b cookies.txt \
  -H "X-CSRF-Token: ${CSRF_TOKEN}" \
  http://localhost:8080/admin/v1/links/github
```

### GET /stats - 获取统计信息

```bash
curl -sS -b cookies.txt \
  http://localhost:8080/admin/v1/stats
```

**响应格式**：
```json
{
  "code": 0,
  "data": {
    "total_links": 100,
    "total_clicks": 5000,
    "active_links": 80
  }
}
```

## 批量操作

### POST /links/batch - 批量创建短链接

> 注意：请求体是对象，字段名为 `links`，不是纯数组。

```bash
curl -sS -X POST \
  -b cookies.txt \
  -H "X-CSRF-Token: ${CSRF_TOKEN}" \
  -H "Content-Type: application/json" \
  -d '{"links":[{"code":"link1","target":"https://example1.com"},{"code":"link2","target":"https://example2.com"}]}' \
  http://localhost:8080/admin/v1/links/batch
```

### PUT /links/batch - 批量更新短链接

> 注意：请求体字段名为 `updates`，每一项包含 `code` 与 `payload`。

```bash
curl -sS -X PUT \
  -b cookies.txt \
  -H "X-CSRF-Token: ${CSRF_TOKEN}" \
  -H "Content-Type: application/json" \
  -d '{"updates":[{"code":"link1","payload":{"target":"https://new-example1.com"}},{"code":"link2","payload":{"target":"https://new-example2.com"}}]}' \
  http://localhost:8080/admin/v1/links/batch
```

### DELETE /links/batch - 批量删除短链接

> 注意：请求体字段名为 `codes`。

```bash
curl -sS -X DELETE \
  -b cookies.txt \
  -H "X-CSRF-Token: ${CSRF_TOKEN}" \
  -H "Content-Type: application/json" \
  -d '{"codes":["link1","link2","link3"]}' \
  http://localhost:8080/admin/v1/links/batch
```

## CSV 导出/导入

### GET /links/export - 导出为 CSV

导出会生成可直接用于导入的 CSV（包含 header），字段：
`code,target,created_at,expires_at,password,click_count`

```bash
curl -sS -b cookies.txt \
  -o shortlinks_export.csv \
  "http://localhost:8080/admin/v1/links/export?only_active=true"
```

### POST /links/import - 从 CSV 导入

上传 `multipart/form-data`：
- `file`：CSV 文件
- `mode`（可选）：冲突处理模式，`skip`（默认）/`overwrite`/`error`

```bash
curl -sS -X POST \
  -b cookies.txt -c cookies.txt \
  -H "X-CSRF-Token: ${CSRF_TOKEN}" \
  -F "mode=overwrite" \
  -F "file=@./shortlinks_export.csv" \
  http://localhost:8080/admin/v1/links/import
```

**响应**：
```json
{
  "code": 0,
  "data": {
    "total_rows": 10,
    "success_count": 9,
    "skipped_count": 1,
    "failed_count": 0,
    "failed_items": []
  }
}
```

## 运行时配置管理

配置管理接口位于 `/{ADMIN_ROUTE_PREFIX}/v1/config` 下，返回值统一为 `{code,data}` 结构；敏感配置会自动掩码为 `[REDACTED]`。

### GET /config - 获取所有配置

```bash
curl -sS -b cookies.txt \
  http://localhost:8080/admin/v1/config
```

### GET /config/schema - 获取配置 Schema（元信息）

返回所有配置项的元信息（类型、默认值、是否需要重启、枚举选项等），主要用于前端动态渲染配置表单/校验。

```bash
curl -sS -b cookies.txt \
  http://localhost:8080/admin/v1/config/schema
```

### GET /config/{key} - 获取单个配置

```bash
curl -sS -b cookies.txt \
  http://localhost:8080/admin/v1/config/features.random_code_length
```

### PUT /config/{key} - 更新配置

```bash
curl -sS -X PUT \
  -b cookies.txt \
  -H "X-CSRF-Token: ${CSRF_TOKEN}" \
  -H "Content-Type: application/json" \
  -d '{"value":"8"}' \
  http://localhost:8080/admin/v1/config/features.random_code_length
```

### GET /config/{key}/history - 获取变更历史

```bash
curl -sS -b cookies.txt \
  "http://localhost:8080/admin/v1/config/features.random_code_length/history?limit=10"
```

### POST /config/reload - 重新加载配置

```bash
curl -sS -X POST -b cookies.txt \
  -H "X-CSRF-Token: ${CSRF_TOKEN}" \
  http://localhost:8080/admin/v1/config/reload
```

## 认证接口补充说明

- `POST /auth/login`：无需 Cookie；验证管理员密码（`api.admin_token` 的明文）成功后下发 Cookie
- `POST /auth/refresh`：无需 Access Cookie，但需要 Refresh Cookie
- `POST /auth/logout`：无需 Cookie；用于清理 Cookie
- `GET /auth/verify`：需要 Access Cookie（中间件校验通过即有效）

## Python 客户端示例（requests）

```python
import requests

class ShortlinkerAdmin:
    def __init__(self, base_url: str, admin_token: str):
        self.base_url = base_url.rstrip("/")
        self.session = requests.Session()
        self.csrf_token = None

        # 登录：Set-Cookie 会被 requests.Session 自动保存
        resp = self.session.post(
            f"{self.base_url}/admin/v1/auth/login",
            json={"password": admin_token},
            timeout=10,
        )
        resp.raise_for_status()
        self.csrf_token = self.session.cookies.get("csrf_token")

    def get_all_links(self, page=1, page_size=20):
        resp = self.session.get(
            f"{self.base_url}/admin/v1/links",
            params={"page": page, "page_size": page_size},
            timeout=10,
        )
        resp.raise_for_status()
        return resp.json()

    def create_link(self, code, target, expires_at=None, force=False):
        payload = {"code": code, "target": target, "force": force}
        if expires_at:
            payload["expires_at"] = expires_at
        resp = self.session.post(
            f"{self.base_url}/admin/v1/links",
            headers={"X-CSRF-Token": self.csrf_token or ""},
            json=payload,
            timeout=10,
        )
        resp.raise_for_status()
        return resp.json()

# 使用示例
admin = ShortlinkerAdmin("http://localhost:8080", "your_admin_token")
print(admin.get_all_links())
```

## 安全建议

1. **强密码**：使用足够复杂的管理员密码（`api.admin_token`）（不要使用默认的自动生成值直接上生产）
2. **HTTPS**：生产环境建议启用 HTTPS，并将 `api.cookie_secure=true`
3. **网络隔离**：仅在受信任网络环境中暴露 Admin API
4. **定期轮换**：定期更换管理员密码（`api.admin_token`）（并重新登录获取新 Cookie）

## Analytics API（统计分析）

Analytics API 提供详细的点击统计分析功能，包括点击趋势、热门链接、来源统计、地理位置分布等。

> 需要先在运行时配置中启用 `analytics.enable_detailed_logging`（需要重启服务生效）才会记录详细的点击日志。
>
> - 默认查询最近 30 天；如果要指定范围，请**同时**提供 `start_date` 和 `end_date`。
> - 日期格式支持 RFC3339（如 `2024-01-01T00:00:00Z`）或 `YYYY-MM-DD`（如 `2024-01-01`）。
> - 地理分布数据需要额外开启 `analytics.enable_geo_lookup=true`（并保留 `analytics.enable_ip_logging=true` 才能拿到 IP）；GeoIP provider 使用启动配置 `[analytics]`（`analytics.maxminddb_path` / `analytics.geoip_api_url`）。
>   - 使用外部 API provider 时，内部带缓存（LRU 10000、TTL 15 分钟、失败负缓存、singleflight 合并并发请求），单次请求超时 2 秒。

### GET /analytics/trends - 获取点击趋势

```bash
curl -sS -b cookies.txt \
  "http://localhost:8080/admin/v1/analytics/trends?start_date=2024-01-01T00:00:00Z&end_date=2024-12-31T23:59:59Z&group_by=day"
```

**查询参数**：

| 参数 | 类型 | 说明 | 示例 |
|------|------|------|------|
| `start_date` | RFC3339 / YYYY-MM-DD | 开始日期（可选；需与 `end_date` 成对出现；缺省=最近 30 天） | `?start_date=2024-01-01T00:00:00Z` |
| `end_date` | RFC3339 / YYYY-MM-DD | 结束日期（可选；需与 `start_date` 成对出现；缺省=最近 30 天） | `?end_date=2024-12-31T23:59:59Z` |
| `group_by` | String | 分组方式（可选；默认 `day`）：`hour`/`day`/`week`/`month` | `?group_by=day` |

**响应格式**：
```json
{
  "code": 0,
  "data": {
    "labels": ["2024-01-01", "2024-01-02", "2024-01-03"],
    "values": [100, 150, 120]
  }
}
```

### GET /analytics/top - 获取热门链接

```bash
curl -sS -b cookies.txt \
  "http://localhost:8080/admin/v1/analytics/top?limit=10"
```

**查询参数**：

| 参数 | 类型 | 说明 | 示例 |
|------|------|------|------|
| `start_date` | RFC3339 / YYYY-MM-DD | 开始日期（可选；需与 `end_date` 成对出现；缺省=最近 30 天） | `?start_date=2024-01-01T00:00:00Z` |
| `end_date` | RFC3339 / YYYY-MM-DD | 结束日期（可选；需与 `start_date` 成对出现；缺省=最近 30 天） | `?end_date=2024-12-31T23:59:59Z` |
| `limit` | Integer | 返回数量（可选；默认 10；最大 100） | `?limit=10` |

**响应格式**：
```json
{
  "code": 0,
  "data": [
    {"code": "github", "clicks": 500},
    {"code": "google", "clicks": 300}
  ]
}
```

### GET /analytics/referrers - 获取来源统计

```bash
curl -sS -b cookies.txt \
  "http://localhost:8080/admin/v1/analytics/referrers?limit=10"
```

**查询参数**：

| 参数 | 类型 | 说明 | 示例 |
|------|------|------|------|
| `start_date` | RFC3339 / YYYY-MM-DD | 开始日期（可选；需与 `end_date` 成对出现；缺省=最近 30 天） | `?start_date=2024-01-01T00:00:00Z` |
| `end_date` | RFC3339 / YYYY-MM-DD | 结束日期（可选；需与 `start_date` 成对出现；缺省=最近 30 天） | `?end_date=2024-12-31T23:59:59Z` |
| `limit` | Integer | 返回数量（可选；默认 10；最大 100） | `?limit=10` |

**响应格式**：
```json
{
  "code": 0,
  "data": [
    {"referrer": "https://google.com", "count": 200, "percentage": 40.0},
    {"referrer": "(direct)", "count": 150, "percentage": 30.0}
  ]
}
```

### GET /analytics/geo - 获取地理位置分布

```bash
curl -sS -b cookies.txt \
  "http://localhost:8080/admin/v1/analytics/geo?limit=20"
```

**查询参数**：

| 参数 | 类型 | 说明 | 示例 |
|------|------|------|------|
| `start_date` | RFC3339 / YYYY-MM-DD | 开始日期（可选；需与 `end_date` 成对出现；缺省=最近 30 天） | `?start_date=2024-01-01T00:00:00Z` |
| `end_date` | RFC3339 / YYYY-MM-DD | 结束日期（可选；需与 `start_date` 成对出现；缺省=最近 30 天） | `?end_date=2024-12-31T23:59:59Z` |
| `limit` | Integer | 返回数量（可选；默认 20；最大 100） | `?limit=20` |

**响应格式**：
```json
{
  "code": 0,
  "data": [
    {"country": "CN", "city": "Beijing", "count": 100},
    {"country": "US", "city": "New York", "count": 80}
  ]
}
```

### GET /links/{code}/analytics - 获取单链接详细统计

```bash
curl -sS -b cookies.txt \
  "http://localhost:8080/admin/v1/links/github/analytics"
```

**查询参数**：

| 参数 | 类型 | 说明 | 示例 |
|------|------|------|------|
| `start_date` | RFC3339 / YYYY-MM-DD | 开始日期（可选；需与 `end_date` 成对出现；缺省=最近 30 天） | `?start_date=2024-01-01` |
| `end_date` | RFC3339 / YYYY-MM-DD | 结束日期（可选；需与 `start_date` 成对出现；缺省=最近 30 天） | `?end_date=2024-12-31` |

**响应格式**：
```json
{
  "code": 0,
  "data": {
    "code": "github",
    "total_clicks": 500,
    "trend": {
      "labels": ["2024-01-01", "2024-01-02"],
      "values": [100, 150]
    },
    "top_referrers": [
      {"referrer": "https://google.com", "count": 100, "percentage": 20.0}
    ],
    "geo_distribution": [
      {"country": "CN", "city": "Beijing", "count": 50}
    ]
  }
}
```

### GET /analytics/export - 导出分析报告（CSV）

```bash
curl -sS -b cookies.txt \
  -o analytics_report.csv \
  "http://localhost:8080/admin/v1/analytics/export?start_date=2024-01-01T00:00:00Z&end_date=2024-12-31T23:59:59Z"
```

**查询参数**：

| 参数 | 类型 | 说明 | 示例 |
|------|------|------|------|
| `start_date` | RFC3339 / YYYY-MM-DD | 开始日期（可选；需与 `end_date` 成对出现；缺省=最近 30 天） | `?start_date=2024-01-01T00:00:00Z` |
| `end_date` | RFC3339 / YYYY-MM-DD | 结束日期（可选；需与 `start_date` 成对出现；缺省=最近 30 天） | `?end_date=2024-12-31T23:59:59Z` |
| `limit` | Integer | 导出条数（可选；默认 10000；最大 100000） | `?limit=10000` |

导出的 CSV 包含以下字段：
`short_code,clicked_at,referrer,user_agent,ip_address,country,city`

### Analytics 相关配置

在运行时配置中，可以调整以下与 Analytics 相关的配置项：

| 配置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `analytics.enable_detailed_logging` | bool | false | 启用详细日志记录（写入 click_logs 表；需要重启生效） |
| `analytics.log_retention_days` | int | 30 | 日志保留天数（当前版本暂未实现自动清理） |
| `analytics.enable_ip_logging` | bool | true | 是否记录 IP 地址 |
| `analytics.enable_geo_lookup` | bool | false | 是否启用地理位置解析 |
