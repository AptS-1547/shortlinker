# 健康检查 API

Shortlinker 提供健康检查 API，用于监控服务状态和存储健康状况。

## 功能概述

- 服务健康状态检查
- 存储后端状态监控
- 就绪和活跃性检查
- 服务运行时间统计

## 配置方式

健康检查 API 的路由前缀可通过环境变量配置，详细配置见 [配置指南](/config/)：

- `HEALTH_ROUTE_PREFIX` - 路由前缀（可选，默认 `/health`）

> 备注：配置项 `api.health_token` / 环境变量 `HEALTH_TOKEN` 在当前实现中不会用于 Health API 的鉴权（仅作为配置项保留）；Health API 目前复用 Admin 的鉴权机制。

## 鉴权方式（重要）

Health API 当前复用 Admin API 的 **JWT Cookie** 鉴权：

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

> 若 `api.admin_token` 为空，Health 端点会返回 `404 Not Found`（视为禁用）。默认情况下若你未显式设置 `ADMIN_TOKEN`，程序会在首次启动时自动生成并在日志中提示一次。

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
      }
    },
    "response_time_ms": 15
  }
}
```

**响应字段说明**：

| 字段 | 类型 | 说明 |
|------|------|------|
| `code` | Integer | 响应码：0 表示健康，1 表示不健康 |
| `data.status` | String | 总体健康状态：`healthy` 或 `unhealthy` |
| `data.timestamp` | RFC3339 | 检查时的时间戳 |
| `data.uptime` | Integer | 服务运行时长（秒） |
| `data.checks.storage.status` | String | 存储后端健康状态 |
| `data.checks.storage.links_count` | Integer | 当前存储的短链接数量 |
| `data.checks.storage.backend` | Object | 存储后端配置信息 |
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
| 401 | 鉴权失败（缺少/无效 Cookie） |
| 503 | 服务不健康 |

> 鉴权失败时，响应体示例：`{"code":401,"data":{"error":"Unauthorized: Invalid or missing token"}}`

## 监控集成（注意事项）

由于当前 Health API 采用 Cookie 鉴权，Kubernetes 的 `httpGet` 探针不方便直接携带有效 JWT（Access Token 有有效期）。

建议策略：

1. **简单存活探针**：直接探测根路径 `/`（会返回 `307`，Kubernetes 视为成功），用于确认进程存活
2. **深度健康检查**：使用外部监控系统/脚本先登录获取 Cookie，再调用 `/health`

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

1. **强密码**：使用足够复杂的 `ADMIN_TOKEN`
2. **网络隔离**：仅在受信任网络中访问 Health 端点
3. **HTTPS**：生产环境建议启用 HTTPS，并正确配置 Cookie 安全参数

