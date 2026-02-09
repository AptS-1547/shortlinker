# Admin API：Analytics 统计分析

本页聚焦点击趋势、热门链接、来源、地理与设备分析接口。

## Analytics API（统计分析）

Analytics API 提供详细的点击统计分析功能，包括点击趋势、热门链接、来源统计、地理位置分布等。

> 需要先在运行时配置中启用 `analytics.enable_detailed_logging`（该项需要重启，修改后请重启服务）才会记录详细的点击日志。
>
> - 默认查询最近 30 天；若要指定范围，请**同时**提供 `start_date` 和 `end_date`（只提供一个会忽略并回退到默认范围；日期解析失败会回退到默认范围对应的起止值）。
> - 日期格式支持 RFC3339（如 `2024-01-01T00:00:00Z`）或 `YYYY-MM-DD`（如 `2024-01-01`；注意：`YYYY-MM-DD` 会按当天 `00:00:00Z` 解析）。
> - 当前实现中，点击写入链路尚未接入 GeoIP 查询（`click_logs.country/city` 默认空值），因此 `/analytics/geo` 与单链接 `geo_distribution` 可能为空数组（除非历史数据已包含地理字段）。
> - 启动配置 `[analytics]`（`analytics.maxminddb_path` / `analytics.geoip_api_url`）已保留用于 GeoIP provider 选择；外部 API provider 具备内置缓存（LRU 10000、TTL 15 分钟、失败负缓存、singleflight，单次请求超时 2 秒）。
> - 设备/浏览器分布（`/analytics/devices`）基于 `user_agent_hash`（User-Agent 原文会去重存储在 `user_agents` 表并通过 hash 关联）。
> - `click_logs.source` 的来源推导：优先 `utm_source`；否则 `ref:{domain}`（来自 `Referer`）；再否则 `direct`。

## 通用查询参数（多数接口）

| 参数 | 类型 | 说明 |
|------|------|------|
| `start_date` | RFC3339 / YYYY-MM-DD | 开始日期（可选；仅当与 `end_date` 同时提供时生效；缺省=最近 30 天） |
| `end_date` | RFC3339 / YYYY-MM-DD | 结束日期（可选；仅当与 `start_date` 同时提供时生效；缺省=最近 30 天） |
| `limit` | Integer | 返回数量（可选；不同端点默认值和上限不同，见各接口说明） |

> 端点默认 `limit`：`top/referrers=10`、`geo=20`、`devices=10`、`links/{code}/analytics/devices=10`。`export` 当前会忽略 `limit`。

### GET /analytics/trends - 获取点击趋势

```bash
curl -sS -b cookies.txt \
  "http://localhost:8080/admin/v1/analytics/trends?start_date=2024-01-01T00:00:00Z&end_date=2024-12-31T23:59:59Z&group_by=day"
```

**查询参数**：

- 支持通用参数：`start_date`、`end_date`
- 专属参数：`group_by`（可选；默认 `day`）：`hour`/`day`/`week`/`month`

**响应格式**：
```json
{
  "code": 0,
  "message": "OK",
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

- 支持通用参数：`start_date`、`end_date`、`limit`
- `limit` 默认 `10`，最大 `100`

**响应格式**：
```json
{
  "code": 0,
  "message": "OK",
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

- 支持通用参数：`start_date`、`end_date`、`limit`
- `limit` 默认 `10`，最大 `100`

**响应格式**：
```json
{
  "code": 0,
  "message": "OK",
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

- 支持通用参数：`start_date`、`end_date`、`limit`
- `limit` 默认 `20`，最大 `100`

**响应格式**：
```json
{
  "code": 0,
  "message": "OK",
  "data": []
}
```

### GET /analytics/devices - 获取设备分析

```bash
curl -sS -b cookies.txt \
  "http://localhost:8080/admin/v1/analytics/devices?limit=10"
```

**查询参数**：

- 支持通用参数：`start_date`、`end_date`、`limit`
- `limit` 默认 `10`，最大 `20`

**响应格式**：
```json
{
  "code": 0,
  "message": "OK",
  "data": {
    "browsers": [
      {"name": "Chrome", "count": 120, "percentage": 60.0}
    ],
    "operating_systems": [
      {"name": "Mac OS X", "count": 80, "percentage": 40.0}
    ],
    "devices": [
      {"name": "pc", "count": 150, "percentage": 75.0}
    ],
    "bot_percentage": 12.3,
    "total_with_ua": 200
  }
}
```

### GET /links/{code}/analytics - 获取单链接详细统计

```bash
curl -sS -b cookies.txt \
  "http://localhost:8080/admin/v1/links/github/analytics"
```

**查询参数**：

- 支持通用参数：`start_date`、`end_date`

**响应格式**：
```json
{
  "code": 0,
  "message": "OK",
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
    "geo_distribution": []
  }
}
```

### GET /links/{code}/analytics/devices - 获取单链接设备分析

```bash
curl -sS -b cookies.txt \
  "http://localhost:8080/admin/v1/links/github/analytics/devices?limit=10"
```

**查询参数**：

- 支持通用参数：`start_date`、`end_date`、`limit`
- `limit` 默认 `10`，最大 `20`

**响应格式**：
```json
{
  "code": 0,
  "message": "OK",
  "data": {
    "browsers": [
      {"name": "Chrome", "count": 80, "percentage": 53.3}
    ],
    "operating_systems": [
      {"name": "Windows", "count": 60, "percentage": 40.0}
    ],
    "devices": [
      {"name": "pc", "count": 120, "percentage": 80.0}
    ],
    "bot_percentage": 8.5,
    "total_with_ua": 150
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

- 支持通用参数：`start_date`、`end_date`、`limit`
- `limit` 当前会被忽略，导出时间范围内全部记录；如需控制大小请缩小日期范围

导出的 CSV 包含以下字段：
`short_code,clicked_at,referrer,source,ip_address,country,city`

响应头 `Content-Disposition` 的文件名格式为：`analytics_YYYYMMDD_YYYYMMDD.csv`（基于解析后的起止日期）。

说明：
- `source` 字段来自重定向时的来源推导规则（`utm_source` → `ref:{domain}` → `direct`）。
- 导出不包含 `user_agent` 原文字段；设备分析通过 `user_agent_hash` 与 `user_agents` 表完成。

### Analytics 相关配置

在运行时配置中，可以调整以下与 Analytics 相关的配置项：

| 配置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `analytics.enable_detailed_logging` | bool | false | 启用详细日志记录（写入 click_logs 表；该项需要重启） |
| `analytics.enable_auto_rollup` | bool | true | 启用自动数据清理任务（需要重启生效；默认每 4 小时运行一次） |
| `analytics.log_retention_days` | int | 30 | 原始点击日志保留天数（由后台任务自动清理；需要启用 `analytics.enable_auto_rollup`） |
| `analytics.hourly_retention_days` | int | 7 | 小时汇总保留天数（清理 `click_stats_hourly` / `click_stats_global_hourly`；需要启用 `analytics.enable_auto_rollup`） |
| `analytics.daily_retention_days` | int | 365 | 天汇总保留天数（清理 `click_stats_daily` / `click_stats_global_daily`；需要启用 `analytics.enable_auto_rollup`） |
| `analytics.enable_ip_logging` | bool | true | 是否记录 IP 地址 |
| `analytics.enable_geo_lookup` | bool | false | GeoIP 预留开关（当前版本点击写入链路尚未消费该配置，`country/city` 默认空） |
| `analytics.sample_rate` | float | 1.0 | 详细日志采样率（0.0-1.0；1.0=全量记录） |
| `analytics.max_log_rows` | int | 0 | `click_logs` 最大行数（0=不限制） |
| `analytics.max_rows_action` | enum | cleanup | 超过最大行数时动作：`cleanup`（删最旧）/`stop`（停止详细日志） |
| `utm.enable_passthrough` | bool | false | 重定向时透传 UTM 参数到目标 URL（`utm_source`/`utm_medium`/`utm_campaign`/`utm_term`/`utm_content`） |

说明：当前实现中，保留天数参数在后台任务启动时读取；修改保留天数后，可能需要重启服务才能让清理任务使用新值。
