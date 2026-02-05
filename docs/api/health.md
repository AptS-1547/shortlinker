# 健康检查 API

Shortlinker 提供健康检查 API，用于监控服务状态和存储健康状况。

## 功能概述

- 服务健康状态检查
- 存储后端状态监控
- 就绪和活跃性检查
- 服务运行时间统计

## 配置方式

健康检查 API 的路由前缀由运行时配置 `routes.health_prefix` 控制（默认 `/health`，修改后需要重启），详细配置见 [配置指南](/config/)。

> 备注：Health API 支持两种鉴权方式：
> - **Bearer Token**：`Authorization: Bearer <HEALTH_TOKEN>`（适合监控/探针，无需 Cookie）
> - **JWT Cookie**：复用 Admin API 登录后下发的 Cookie（适合管理面板/浏览器）

## 鉴权方式（重要）

Health API 需要鉴权，可通过 **Bearer Token**（`HEALTH_TOKEN`）或 **JWT Cookie**（Admin 登录后下发）访问。  
当且仅当 `api.admin_token` 与 `api.health_token` 都为空时，Health 端点会返回 `404 Not Found`（视为禁用）。

### 方式 A：Bearer Token（推荐用于监控/探针）

当你配置了运行时配置 `api.health_token` 后，可以直接通过请求头访问健康检查接口：

```bash
HEALTH_TOKEN="your_health_token"

curl -sS \
  -H "Authorization: Bearer ${HEALTH_TOKEN}" \
  http://localhost:8080/health
```

### 方式 B：JWT Cookie（推荐用于管理面板/浏览器）

1. 先调用 `POST /admin/v1/auth/login` 登录获取 Cookie
2. 再携带 Cookie 调用 `/health`、`/health/ready`、`/health/live`

示例（curl 保存并复用 cookie）：
```bash
# 1) 登录获取 cookies
curl -sS -X POST \
  -H "Content-Type: application/json" \
  -c cookies.txt \
  -d '{"password":"your_admin_token"}' \
  http://localhost:8080/admin/v1/auth/login

# 2) 调用健康检查接口
curl -sS -b cookies.txt \
  http://localhost:8080/health
```

> 首次启动时会自动生成一个随机管理员密码并写入 `admin_token.txt`（若文件不存在；保存后请删除该文件）。

## API 端点

**Base URL**: `http://your-domain:port/health`

> 所有端点同时支持 `GET` 与 `HEAD`。

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

## 状态码

| 状态码 | 说明 |
|--------|------|
| 200 | 健康/就绪 |
| 204 | 活跃（无内容） |
| 401 | 鉴权失败（缺少/无效 Cookie 或 Bearer Token） |
| 404 | 端点被禁用（`api.admin_token` 与 `api.health_token` 均为空） |
| 503 | 服务不健康 |

> 鉴权失败时（HTTP 401），响应体示例：`{"code":1001,"message":"Unauthorized: Invalid or missing token"}`

## 监控集成（注意事项）

如果你使用 **Bearer Token**（即运行时配置 `api.health_token` 的值），就可以避免 JWT Cookie 有有效期的问题，更适合自动化监控。

建议策略：

1. **推荐：设置 `api.health_token` 并用 Bearer Token 探测 `/health/live` 或 `/health/ready`**
2. **兼容方案：探测根路径 `/`**（会返回 `307`，Kubernetes 视为成功），用于确认进程存活
3. **兼容方案：登录获取 Cookie 再探测 `/health`**（适合已有登录流程的监控脚本）

### Kubernetes 探针示例（Bearer Token）

```yaml
apiVersion: v1
kind: Pod
spec:
  containers:
  - name: shortlinker
    image: e1saps/shortlinker
    livenessProbe:
      httpGet:
        path: /health/live
        port: 8080
        httpHeaders:
          - name: Authorization
            value: "Bearer your_health_token"
      initialDelaySeconds: 10
      periodSeconds: 10
```

### Kubernetes 探针示例（简单存活）

```yaml
apiVersion: v1
kind: Pod
spec:
  containers:
  - name: shortlinker
    image: e1saps/shortlinker
    livenessProbe:
      httpGet:
        path: /
        port: 8080
      initialDelaySeconds: 10
      periodSeconds: 10
```

## 监控脚本示例（登录 + 健康检查）

```bash
#!/bin/bash
set -euo pipefail

ADMIN_TOKEN="your_admin_token"
BASE_URL="http://localhost:8080"
COOKIE_JAR="$(mktemp)"

# 登录获取 cookies
curl -sS -X POST \
  -H "Content-Type: application/json" \
  -c "$COOKIE_JAR" \
  -d "{\"password\":\"${ADMIN_TOKEN}\"}" \
  "${BASE_URL}/admin/v1/auth/login" >/dev/null

# 检查健康状态（HTTP 200 表示健康，503 表示不健康）
curl -sS -b "$COOKIE_JAR" "${BASE_URL}/health"
```

## 故障排除

```bash
# 先登录再检查
curl -sS -X POST -H "Content-Type: application/json" -c cookies.txt \
  -d '{"password":"your_admin_token"}' \
  http://localhost:8080/admin/v1/auth/login

curl -sS -b cookies.txt http://localhost:8080/health | jq .
```

## 安全建议

1. **强密码**：使用足够复杂的管理员密码（`api.admin_token`）
2. **网络隔离**：仅在受信任网络中访问 Health 端点
3. **HTTPS**：生产环境建议启用 HTTPS，并正确配置 Cookie 安全参数
