# Admin API 文档

Shortlinker 提供完整的 HTTP API 用于管理短链接，支持 CRUD 操作。

## 配置方式

Admin API 需要以下环境变量，详细配置请参考 [环境变量配置](/config/)：

- `ADMIN_TOKEN` - 管理员令牌（必需）
- `ADMIN_ROUTE_PREFIX` - 路由前缀（可选，默认 `/admin`）

所有请求需要携带 Authorization 头：
```http
Authorization: Bearer your_secure_admin_token
```

## API 端点

**Base URL**: `http://your-domain:port/admin`

### 通用响应格式

```json
{
  "code": 0,
  "data": { /* 响应数据 */ }
}
```

### GET /admin/link - 获取所有短链接

```bash
curl -H "Authorization: Bearer your_token" \
     http://localhost:8080/admin/link
```

**查询参数**：

| 参数 | 类型 | 说明 | 示例 |
|------|------|------|------|
| `page` | Integer | 页码（从1开始） | `?page=1` |
| `page_size` | Integer | 每页数量（1-100） | `?page_size=20` |
| `search` | String | 模糊搜索短码和目标 URL | `?search=github` |
| `created_after` | RFC3339 | 创建时间过滤（晚于） | `?created_after=2024-01-01T00:00:00Z` |
| `created_before` | RFC3339 | 创建时间过滤（早于） | `?created_before=2024-12-31T23:59:59Z` |
| `only_expired` | Boolean | 仅显示已过期 | `?only_expired=true` |
| `only_active` | Boolean | 仅显示未过期 | `?only_active=true` |

**分页查询示例**：

```bash
# 获取第2页，每页10条
curl -H "Authorization: Bearer your_token" \
     "http://localhost:8080/admin/link?page=2&page_size=10"

# 仅显示活跃链接
curl -H "Authorization: Bearer your_token" \
     "http://localhost:8080/admin/link?only_active=true"

# 组合查询：第1页，仅活跃，按时间过滤
curl -H "Authorization: Bearer your_token" \
     "http://localhost:8080/admin/link?page=1&page_size=20&only_active=true&created_after=2024-01-01T00:00:00Z"

# 搜索包含 github 的链接
curl -H "Authorization: Bearer your_token" \
     "http://localhost:8080/admin/link?search=github"
```

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

### POST /admin/link - 创建短链接

```bash
curl -X POST \
     -H "Authorization: Bearer your_token" \
     -H "Content-Type: application/json" \
     -d '{"code":"github","target":"https://github.com"}' \
     http://localhost:8080/admin/link
```

**请求体**:
```json
{
  "code": "github",
  "target": "https://github.com",
  "expires_at": "2024-12-31T23:59:59Z",  // 可选，支持相对时间格式（如 "7d"）
  "password": "secret123"  // 可选，密码保护（实验性功能，仅存储）
}
```

**说明**：
- `code`：短码（可选），不提供则自动生成随机短码
- `target`：目标 URL（必需）
- `expires_at`：过期时间（可选），支持相对时间（如 `"1d"`, `"7d"`, `"1w"`）或 RFC3339 格式
- `password`：密码保护（可选）⚠️ **注意**：当前版本仅存储密码，重定向时暂不验证，此为实验性功能

**创建带密码的短链接**：

```bash
curl -X POST \
     -H "Authorization: Bearer your_token" \
     -H "Content-Type: application/json" \
     -d '{"code":"secret","target":"https://example.com","password":"mypassword"}' \
     http://localhost:8080/admin/link
```

### GET /admin/link/{code} - 获取指定短链接

```bash
curl -H "Authorization: Bearer your_token" \
     http://localhost:8080/admin/link/github
```

### PUT /admin/link/{code} - 更新短链接

```bash
curl -X PUT \
     -H "Authorization: Bearer your_token" \
     -H "Content-Type: application/json" \
     -d '{"target":"https://github.com/new-repo","expires_at":"30d"}' \
     http://localhost:8080/admin/link/github
```

**请求体说明**：
```json
{
  "target": "https://new-url.com",  // 必需
  "expires_at": "7d",  // 可选，不提供则保持原值
  "password": "newpass"  // 可选，不提供则保持原值，传 null 可清除密码
}
```

**说明**：
- 更新时会保留原有的创建时间和点击计数
- `expires_at` 不提供则保持原过期时间
- `password` 不提供则保持原密码，提供新值则更新密码

### DELETE /admin/link/{code} - 删除短链接

```bash
curl -X DELETE \
     -H "Authorization: Bearer your_token" \
     http://localhost:8080/admin/link/github
```

### GET /admin/stats - 获取统计信息

```bash
curl -H "Authorization: Bearer your_token" \
     http://localhost:8080/admin/stats
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

**字段说明**：
- `total_links`：短链接总数
- `total_clicks`：总点击次数
- `active_links`：未过期的活跃链接数

## 批量操作

### POST /admin/link/batch - 批量创建短链接

```bash
curl -X POST \
     -H "Authorization: Bearer your_token" \
     -H "Content-Type: application/json" \
     -d '[{"code":"link1","target":"https://example1.com"},{"code":"link2","target":"https://example2.com"}]' \
     http://localhost:8080/admin/link/batch
```

### PUT /admin/link/batch - 批量更新短链接

```bash
curl -X PUT \
     -H "Authorization: Bearer your_token" \
     -H "Content-Type: application/json" \
     -d '[{"code":"link1","target":"https://new-example1.com"},{"code":"link2","target":"https://new-example2.com"}]' \
     http://localhost:8080/admin/link/batch
```

### DELETE /admin/link/batch - 批量删除短链接

```bash
curl -X DELETE \
     -H "Authorization: Bearer your_token" \
     -H "Content-Type: application/json" \
     -d '["link1","link2","link3"]' \
     http://localhost:8080/admin/link/batch
```

## 认证接口

### POST /admin/v1/auth/login - 登录

```bash
curl -X POST \
     -H "Content-Type: application/json" \
     -d '{"password":"your_admin_token"}' \
     http://localhost:8080/admin/v1/auth/login
```

**响应**：返回 JWT Access Token 和 Refresh Token（通过 Cookie 或响应体）。

### POST /admin/v1/auth/refresh - 刷新 Token

```bash
curl -X POST \
     -H "Authorization: Bearer your_refresh_token" \
     http://localhost:8080/admin/v1/auth/refresh
```

**响应**：返回新的 Access Token。

### POST /admin/v1/auth/logout - 登出

```bash
curl -X POST \
     -H "Authorization: Bearer your_token" \
     http://localhost:8080/admin/v1/auth/logout
```

**响应**：清除认证 Cookie 并使 Token 失效。

### GET /admin/v1/auth/verify - 验证 Token

```bash
curl -H "Authorization: Bearer your_token" \
     http://localhost:8080/admin/v1/auth/verify
```

**响应**：验证当前 Token 是否有效。

## 错误码

| 错误码 | 说明 |
|--------|------|
| 0 | 成功 |
| 1 | 一般错误 |
| 401 | 鉴权失败 |

## Python 客户端示例

```python
import requests

class ShortlinkerAdmin:
    def __init__(self, base_url, token):
        self.base_url = base_url.rstrip('/')
        self.headers = {
            'Authorization': f'Bearer {token}',
            'Content-Type': 'application/json'
        }
    
    def create_link(self, code, target, expires_at=None):
        data = {'code': code, 'target': target}
        if expires_at:
            data['expires_at'] = expires_at
        
        response = requests.post(
            f'{self.base_url}/admin/link',
            headers=self.headers,
            json=data
        )
        return response.json()
    
    def get_all_links(self):
        response = requests.get(
            f'{self.base_url}/admin/link',
            headers=self.headers
        )
        return response.json()

# 使用示例
admin = ShortlinkerAdmin('http://localhost:8080', 'your_token')
result = admin.create_link('test', 'https://example.com')
```

## 安全建议

1. **强密码**: 使用足够复杂的 ADMIN_TOKEN
2. **HTTPS**: 生产环境必须使用 HTTPS
3. **网络隔离**: 仅在受信任的网络环境中暴露 Admin API
4. **定期轮换**: 定期更换 Admin Token

## 实验性功能

### 密码保护功能 ⚠️

**当前状态**：实验性 / 部分实现

Shortlinker 支持为短链接设置密码字段，**当前版本支持存储密码（使用 Argon2 哈希加密），但访问时暂不验证**。

**已实现**：
- ✅ 通过 API 创建带密码的短链接
- ✅ 密码使用 Argon2 算法哈希存储（安全存储）
- ✅ 存储和查询密码字段（API 返回时显示为已设置状态）
- ✅ 更新和删除密码

**未实现**：
- ❌ 访问短链接时的密码验证
- ❌ 密码验证页面

**使用示例**：

```bash
# 创建带密码的短链接（密码会被哈希存储但访问时不验证）
curl -X POST \
     -H "Authorization: Bearer your_token" \
     -H "Content-Type: application/json" \
     -d '{"code":"secret","target":"https://example.com","password":"mypass123"}' \
     http://localhost:8080/admin/link

# 查询时返回密码哈希值
curl -H "Authorization: Bearer your_token" \
     http://localhost:8080/admin/link/secret
# 返回: {"code":"secret","target":"...","password":"$argon2id$...",...}
```

**安全说明**：
- ✅ 密码使用 Argon2 算法哈希存储，不可逆
- ⚠️ 访问短链接时暂不要求输入密码
- ⚠️ 功能尚未完全实现，不建议在生产环境依赖此功能

**计划改进**：
- 实现密码验证页面
- 支持多种验证方式（HTTP Basic Auth、查询参数等）

如需完整的密码保护功能，建议在反向代理层（如 Nginx）实现访问控制。
