# Admin API 文档

Shortlinker v0.0.5+ 版本提供了完整的 HTTP API 用于管理短链接。

## 功能概述

Admin API 支持对短链接的完整 CRUD 操作：
- 创建新的短链接
- 查询单个或所有短链接
- 更新现有短链接
- 删除短链接

## 鉴权配置

### 环境变量

```bash
# 必需：设置管理员令牌
ADMIN_TOKEN=your_secure_admin_token

# 可选：自定义路由前缀
ADMIN_ROUTE_PREFIX=/api/admin
```

### 请求头

所有 Admin API 请求都需要携带 Authorization 头：

```http
Authorization: Bearer your_secure_admin_token
```

## API 端点

### 基础信息

- **Base URL**: `http://your-domain:port{ADMIN_ROUTE_PREFIX}`
- **默认前缀**: `/admin`
- **认证方式**: Bearer Token
- **响应格式**: JSON

### 通用响应格式

```json
{
  "code": 0,
  "data": { /* 响应数据 */ }
}
```

错误响应：
```json
{
  "code": 非零错误码,
  "data": {
    "error": "错误描述"
  }
}
```

## 接口详情

### GET /admin/link - 获取所有短链接

获取系统中所有的短链接列表。

**请求**:
```bash
curl -H "Authorization: Bearer your_token" \
     http://localhost:8080/admin/link
```

**响应**:
```json
{
  "code": 0,
  "data": {
    "github": {
      "short_code": "github",
      "target_url": "https://github.com",
      "created_at": "2024-01-01T00:00:00Z",
      "expires_at": null
    },
    "temp": {
      "short_code": "temp", 
      "target_url": "https://example.com",
      "created_at": "2024-01-01T00:00:00Z",
      "expires_at": "2024-12-31T23:59:59Z"
    }
  }
}
```

### POST /admin/link - 创建短链接

创建新的短链接。

**请求体**:
```json
{
  "code": "github",
  "target": "https://github.com",
  "expires_at": "2024-12-31T23:59:59Z"  // 可选
}
```

**示例**:
```bash
curl -X POST \
     -H "Authorization: Bearer your_token" \
     -H "Content-Type: application/json" \
     -d '{"code":"github","target":"https://github.com"}' \
     http://localhost:8080/admin/link
```

**响应**:
```json
{
  "code": 0,
  "data": {
    "code": "github",
    "target": "https://github.com",
    "expires_at": null
  }
}
```

### GET /admin/link/{code} - 获取指定短链接

获取指定短码的链接信息。

**请求**:
```bash
curl -H "Authorization: Bearer your_token" \
     http://localhost:8080/admin/link/github
```

**响应**:
```json
{
  "code": 0,
  "data": {
    "short_code": "github",
    "target_url": "https://github.com",
    "created_at": "2024-01-01T00:00:00Z",
    "expires_at": null
  }
}
```

### PUT /admin/link/{code} - 更新短链接

更新指定短码的目标地址和过期时间。

**请求体**:
```json
{
  "code": "github",
  "target": "https://github.com/new-repo",
  "expires_at": "2025-01-01T00:00:00Z"
}
```

**示例**:
```bash
curl -X PUT \
     -H "Authorization: Bearer your_token" \
     -H "Content-Type: application/json" \
     -d '{"code":"github","target":"https://github.com/new-repo"}' \
     http://localhost:8080/admin/link/github
```

### DELETE /admin/link/{code} - 删除短链接

删除指定的短链接。

**请求**:
```bash
curl -X DELETE \
     -H "Authorization: Bearer your_token" \
     http://localhost:8080/admin/link/github
```

**响应**:
```json
{
  "code": 0,
  "data": {
    "message": "Link deleted successfully"
  }
}
```

## 错误码

| 错误码 | 说明 |
|--------|------|
| 0 | 成功 |
| 1 | 一般错误 |
| 401 | 鉴权失败 |

## 使用示例

### Python 示例

```python
import requests
import json

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
    
    def delete_link(self, code):
        response = requests.delete(
            f'{self.base_url}/admin/link/{code}',
            headers=self.headers
        )
        return response.json()

# 使用示例
admin = ShortlinkerAdmin('http://localhost:8080', 'your_token')

# 创建链接
result = admin.create_link('test', 'https://example.com')
print(result)

# 获取所有链接
links = admin.get_all_links()
print(links)
```

### JavaScript 示例

```javascript
class ShortlinkerAdmin {
    constructor(baseUrl, token) {
        this.baseUrl = baseUrl.replace(/\/$/, '');
        this.headers = {
            'Authorization': `Bearer ${token}`,
            'Content-Type': 'application/json'
        };
    }
    
    async createLink(code, target, expiresAt = null) {
        const data = { code, target };
        if (expiresAt) data.expires_at = expiresAt;
        
        const response = await fetch(`${this.baseUrl}/admin/link`, {
            method: 'POST',
            headers: this.headers,
            body: JSON.stringify(data)
        });
        
        return response.json();
    }
    
    async getAllLinks() {
        const response = await fetch(`${this.baseUrl}/admin/link`, {
            headers: this.headers
        });
        
        return response.json();
    }
    
    async deleteLink(code) {
        const response = await fetch(`${this.baseUrl}/admin/link/${code}`, {
            method: 'DELETE',
            headers: this.headers
        });
        
        return response.json();
    }
}

// 使用示例
const admin = new ShortlinkerAdmin('http://localhost:8080', 'your_token');

// 创建链接
admin.createLink('test', 'https://example.com')
    .then(result => console.log(result));
```

## 安全建议

1. **强密码**: 使用足够复杂的 ADMIN_TOKEN
2. **HTTPS**: 生产环境必须使用 HTTPS
3. **网络隔离**: 仅在受信任的网络环境中暴露 Admin API
4. **定期轮换**: 定期更换 Admin Token
5. **日志监控**: 监控 Admin API 的访问日志
