# 健康检查 API：端点与状态码

本页提供 `/health` 系列接口的完整响应说明与状态码约定。监控部署建议见 [健康检查 API：监控集成与故障排除](/api/health-monitoring)。

## API 端点

**Base URL**: `http://your-domain:port/health`

> 说明：`/health`、`/health/ready`、`/health/live` 支持 `GET` 与 `HEAD`；`/health/metrics` 仅支持 `GET`（Prometheus 抓取用，需启用 `metrics` 特性）。

### GET /health - 完整健康检查

```bash
curl -sS -b cookies.txt \
  http://localhost:8080/health
```

**响应示例**:
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
        "links_count": 42,
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

**响应字段说明**：

| 字段 | 类型 | 说明 |
|------|------|------|
| `code` | Integer | 服务端 `ErrorCode` 数字枚举：`0` 表示成功；不健康时通常为 `1030`（ServiceUnavailable） |
| `message` | String | 响应信息：成功时通常为 `OK`；失败时为错误原因 |
| `data.status` | String | 总体健康状态：`healthy` 或 `unhealthy` |
| `data.timestamp` | RFC3339 | 检查时的时间戳 |
| `data.uptime` | Integer | 服务运行时长（秒） |
| `data.checks.storage.status` | String | 存储后端健康状态 |
| `data.checks.storage.links_count` | Integer | 当前存储的短链接数量 |
| `data.checks.storage.backend` | Object | 存储后端配置信息 |
| `data.checks.cache.status` | String | 缓存健康状态 |
| `data.response_time_ms` | Integer | 健康检查响应时间（毫秒） |

### GET /health/ready - 就绪检查

```bash
curl -sS -b cookies.txt \
  http://localhost:8080/health/ready
```

返回 200 状态码表示服务就绪（响应体为 `OK`）。

### GET /health/live - 活跃性检查

```bash
curl -sS -b cookies.txt -I \
  http://localhost:8080/health/live
```

返回 204 状态码表示服务正常运行。

### GET /health/metrics - Prometheus 指标（可选）

> 该端点仅在**编译时**启用 `metrics` feature 时注册；未启用时会直接返回 `404`。

- 默认路径：`/health/metrics`
- 实际路径：`{routes.health_prefix}/metrics`（由运行时配置 `routes.health_prefix` 控制）

鉴权方式与其他 Health 端点一致（Bearer Token / JWT Cookie），建议监控系统使用 Bearer Token：

```bash
HEALTH_TOKEN="your_health_token"

curl -sS \
  -H "Authorization: Bearer ${HEALTH_TOKEN}" \
  http://localhost:8080/health/metrics
```

返回 Prometheus 文本格式（`text/plain; version=0.0.4; charset=utf-8`）。如导出失败会返回 `500`（纯文本错误信息）。

**导出的指标列表（当前实现）**：

| 指标名 | 类型 | Labels | 说明 |
|------|------|--------|------|
| `shortlinker_http_request_duration_seconds` | HistogramVec | `method`,`endpoint`,`status` | HTTP 请求延迟（秒） |
| `shortlinker_http_requests_total` | CounterVec | `method`,`endpoint`,`status` | HTTP 请求总数 |
| `shortlinker_http_active_connections` | Gauge | - | 当前进行中的请求数（近似并发） |
| `shortlinker_db_query_duration_seconds` | HistogramVec | `operation` | 数据库查询延迟（秒） |
| `shortlinker_db_queries_total` | CounterVec | `operation` | 数据库查询总数 |
| `shortlinker_cache_operation_duration_seconds` | HistogramVec | `operation`,`layer` | 缓存操作延迟（秒） |
| `shortlinker_cache_entries` | GaugeVec | `layer` | 缓存条目数（当前仅 `object_cache` 会被更新） |
| `shortlinker_cache_hits_total` | CounterVec | `layer` | 缓存命中次数（按层统计） |
| `shortlinker_cache_misses_total` | CounterVec | `layer` | 缓存未命中次数（按层统计，当前仅 `object_cache`） |
| `shortlinker_redirects_total` | CounterVec | `status` | 重定向次数（按状态码统计，例如 `307`/`404`） |
| `shortlinker_clicks_buffer_entries` | Gauge | - | 点击缓冲区条目数（唯一 short code 数量，不是总点击数） |
| `shortlinker_clicks_flush_total` | CounterVec | `trigger`,`status` | 点击刷盘次数（按触发方式与结果统计） |
| `shortlinker_clicks_channel_dropped` | CounterVec | `reason` | 详细点击事件在 channel 满/断开时的丢弃次数（`reason`: `full` / `disconnected`） |
| `shortlinker_auth_failures_total` | CounterVec | `method` | 鉴权失败次数（当前主要来自 Admin API：`bearer`/`cookie`） |
| `shortlinker_bloom_filter_false_positives_total` | Counter | - | Bloom Filter 误报次数 |
| `shortlinker_uptime_seconds` | Gauge | - | 服务运行时间（秒） |
| `shortlinker_process_memory_bytes` | GaugeVec | `type` | 进程内存占用（字节，`rss`/`virtual`） |
| `shortlinker_process_cpu_seconds` | Gauge | - | 进程累计 CPU 时间（秒，user+system） |
| `shortlinker_build_info` | GaugeVec | `version` | 构建信息（约定值恒为 `1`，用 label 标记版本号） |

Labels 取值说明（常用）：

- `endpoint`: `admin` / `health` / `frontend` / `redirect`（按路径前缀分类，避免 label 基数爆炸；前缀来自 `routes.admin_prefix` / `routes.health_prefix` / `routes.frontend_prefix`，需重启生效）
- `layer`: `bloom_filter` / `negative_cache` / `object_cache`
- `operation`（DB）: `get` / `load_all` / `load_all_codes` / `count` / `paginated_query` / `batch_get` / `get_stats`
- `trigger`（点击刷盘）: `interval` / `threshold` / `manual`；`status`: `success` / `failed`

## 状态码

| 状态码 | 说明 |
|--------|------|
| 200 | 健康/就绪 |
| 204 | 活跃（无内容） |
| 401 | 鉴权失败（缺少/无效 Cookie 或 Bearer Token） |
| 404 | 端点被禁用（`api.admin_token` 与 `api.health_token` 均为空） |
| 503 | 服务不健康 |

> 鉴权失败时（HTTP 401），响应体示例：`{"code":1001,"message":"Unauthorized: Invalid or missing token"}`
