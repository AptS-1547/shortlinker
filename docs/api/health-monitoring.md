# 健康检查 API：监控集成与故障排除

本页聚焦探针策略、Kubernetes 示例、脚本化检查与安全建议。

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
