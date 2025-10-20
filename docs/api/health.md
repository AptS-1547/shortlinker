# 健康检查 API

Shortlinker 提供健康检查 API，用于监控服务状态和存储健康状况。

## 功能概述

- 服务健康状态检查
- 存储后端状态监控  
- 就绪和活跃性检查
- 服务运行时间统计

## 配置方式

健康检查 API 需要以下环境变量，详细配置请参考 [环境变量配置](/config/)：

- `HEALTH_TOKEN` - 健康检查令牌（必需）
- `HEALTH_ROUTE_PREFIX` - 路由前缀（可选，默认 `/health`）

所有请求需要携带 Authorization 头：
```http
Authorization: Bearer your_secure_health_token
```

## API 端点

**Base URL**: `http://your-domain:port/health`

### GET /health - 完整健康检查

```bash
curl -H "Authorization: Bearer your_health_token" \
     http://localhost:8080/health
```

**响应示例**:
```json
{
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
```

**响应字段说明**：

| 字段 | 类型 | 说明 |
|------|------|------|
| `status` | String | 总体健康状态：`healthy` 或 `unhealthy` |
| `timestamp` | RFC3339 | 检查时的时间戳 |
| `uptime` | Integer | 服务运行时长（秒） |
| `checks.storage.status` | String | 存储后端健康状态 |
| `checks.storage.links_count` | Integer | 当前存储的短链接数量 |
| `checks.storage.backend` | Object | 存储后端配置信息 |
| `response_time_ms` | Integer | 健康检查响应时间（毫秒） |

### GET /health/ready - 就绪检查

```bash
curl -H "Authorization: Bearer your_health_token" \
     http://localhost:8080/health/ready
```

返回 200 状态码表示服务就绪。

### GET /health/live - 活跃性检查

```bash
curl -H "Authorization: Bearer your_health_token" \
     http://localhost:8080/health/live
```

返回 204 状态码表示服务正常运行。

## 状态码

| 状态码 | 说明 |
|--------|------|
| 200 | 健康/就绪 |
| 204 | 活跃（无内容） |
| 401 | 鉴权失败 |
| 503 | 服务不健康 |

## 监控集成

### Kubernetes 探针配置

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
      initialDelaySeconds: 30
      periodSeconds: 10
    readinessProbe:
      httpGet:
        path: /health/ready
        port: 8080
        httpHeaders:
        - name: Authorization
          value: "Bearer your_health_token"
      initialDelaySeconds: 5
      periodSeconds: 5
```

### Docker Compose 健康检查

```yaml
version: '3.8'
services:
  shortlinker:
    image: e1saps/shortlinker
    healthcheck:
      test: ["CMD", "curl", "-f", "-H", "Authorization: Bearer your_health_token", "http://localhost:8080/health/live"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 40s
```

## 监控脚本示例

```bash
#!/bin/bash
# simple_monitor.sh

HEALTH_TOKEN="your_health_token"
HEALTH_URL="http://localhost:8080/health"

check_health() {
    response=$(curl -s -w "%{http_code}" -H "Authorization: Bearer $HEALTH_TOKEN" "$HEALTH_URL")
    http_code="${response: -3}"
  
    if [ "$http_code" -eq 200 ]; then
        echo "$(date): Service is healthy"
        return 0
    else
        echo "$(date): Service is unhealthy (HTTP $http_code)"
        return 1
    fi
}

# 每60秒检查一次
while true; do
    check_health || echo "$(date): Sending alert..."
    sleep 60
done
```

## 故障排除

```bash
# 检查服务状态
curl -H "Authorization: Bearer your_token" http://localhost:8080/health | jq .

# 验证 API 是否启用
if [ -n "$HEALTH_TOKEN" ]; then
    echo "Health API enabled"
else
    echo "Health API disabled"
fi
```

## 安全建议

1. **强密码**: 使用足够复杂的 HEALTH_TOKEN
2. **网络隔离**: 仅在监控网络中暴露健康检查端点
3. **定期轮换**: 定期更换 Health Token
