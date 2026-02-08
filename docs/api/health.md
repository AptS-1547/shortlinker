# 健康检查 API

健康检查文档已拆分为“概览 / 端点详情 / 监控与故障排除”三部分，便于按需查阅。

## 文档导航

- [健康检查 API：端点与状态码](/api/health-endpoints)
- [健康检查 API：监控集成与故障排除](/api/health-monitoring)

## 功能概览

- 服务健康状态检查
- 存储后端状态监控
- 就绪（Readiness）与活跃性（Liveness）检查
- 服务运行时间与指标导出（可选）

## 配置方式

健康检查 API 的路由前缀由运行时配置 `routes.health_prefix` 控制（默认 `/health`，修改后需要重启），详细配置见 [配置指南](/config/)。

> Health API 支持两种鉴权方式：
>
> - **Bearer Token**：`Authorization: Bearer <HEALTH_TOKEN>`（适合监控/探针，无需 Cookie）
> - **JWT Cookie**：复用 Admin API 登录后下发的 Cookie（适合管理面板/浏览器）

## 鉴权方式（重要）

Health API 需要鉴权，可通过 **Bearer Token**（`HEALTH_TOKEN`）或 **JWT Cookie**（Admin 登录后下发）访问。

当且仅当 `api.admin_token` 与 `api.health_token` 都为空时，Health 端点会返回 `404 Not Found`（视为禁用）。

### 方式 A：Bearer Token（推荐用于监控/探针）

配置了 `api.health_token` 后，可直接通过请求头访问：

```bash
HEALTH_TOKEN="your_health_token"

curl -sS \
  -H "Authorization: Bearer ${HEALTH_TOKEN}" \
  http://localhost:8080/health
```

### 方式 B：JWT Cookie（推荐用于管理面板/浏览器）

1. 先调用 `POST /admin/v1/auth/login` 登录获取 Cookie
2. 再携带 Cookie 调用 `/health`、`/health/ready`、`/health/live`

示例：

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

> 若尚未设置 `api.admin_token`，请先执行 `./shortlinker reset-password`，否则 Admin API 不可用。

## 下一步

- 端点响应结构与状态码：见 [健康检查 API：端点与状态码](/api/health-endpoints)
- K8s 探针、监控脚本与故障排除：见 [健康检查 API：监控集成与故障排除](/api/health-monitoring)
