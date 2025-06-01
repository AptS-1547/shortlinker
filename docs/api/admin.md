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
  "expires_at": "2024-12-31T23:59:59Z"  // 可选
}
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
     -d '{"code":"github","target":"https://github.com/new-repo"}' \
     http://localhost:8080/admin/link/github
```

### DELETE /admin/link/{code} - 删除短链接

```bash
curl -X DELETE \
     -H "Authorization: Bearer your_token" \
     http://localhost:8080/admin/link/github
```

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
