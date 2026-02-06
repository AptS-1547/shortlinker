# Admin API 文档

Shortlinker 的 Admin API 已按主题拆分，避免单页过长，便于按场景查阅。

## 文档导航

- [Admin API：链接与批量操作](/api/admin-links)
- [Admin API：运行时配置与自动化示例](/api/admin-config)
- [Admin API：Analytics 统计分析](/api/admin-analytics)

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

### 2) CSRF 防护（Cookie 鉴权写操作必需）

当你使用 **JWT Cookie** 鉴权访问写操作（`POST`/`PUT`/`DELETE`）时，需要同时提供：

- Cookie：`csrf_token`
- Header：`X-CSRF-Token: <csrf_token 的值>`

> 例外：`POST /auth/login`、`POST /auth/refresh`、`POST /auth/logout` 不需要 CSRF；`GET/HEAD/OPTIONS` 也不需要。
>
> 如果你改用 `Authorization: Bearer <ACCESS_TOKEN>` 访问写操作，则不需要 CSRF。

提取 CSRF Token 示例：

```bash
CSRF_TOKEN=$(awk '$6=="csrf_token"{print $7}' cookies.txt | tail -n 1)
```

### 3) 刷新 Token

**POST** `/{ADMIN_ROUTE_PREFIX}/v1/auth/refresh`

```bash
curl -sS -X POST \
  -b cookies.txt -c cookies.txt \
  http://localhost:8080/admin/v1/auth/refresh
```

### 4) 登出（清理 Cookie）

**POST** `/{ADMIN_ROUTE_PREFIX}/v1/auth/logout`

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
  "message": "OK",
  "data": { /* 响应数据 */ }
}
```

- `code = 0`：成功
- `code != 0`：失败（值为服务端 `ErrorCode` 数字枚举；错误原因在 `message`，`data` 通常会省略）
- `message`：始终存在的人类可读提示；成功时通常为 `OK`
- HTTP 状态码用于表达错误类型（如 `401/404/409/500`）

## 安全建议

1. **强密码**：使用足够复杂的管理员密码（`api.admin_token`）（不要使用默认的自动生成值直接上生产）
2. **HTTPS**：生产环境建议启用 HTTPS，并将 `api.cookie_secure=true`
3. **网络隔离**：仅在受信任网络环境中暴露 Admin API
4. **定期轮换**：定期更换管理员密码（`api.admin_token`）（并重新登录获取新 Cookie）
