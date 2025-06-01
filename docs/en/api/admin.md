# Admin API Documentation

Shortlinker provides complete HTTP API for managing short links, supporting CRUD operations.

## Configuration

Admin API requires the following environment variables. For detailed configuration, see [Environment Variables Configuration](/en/config/):

- `ADMIN_TOKEN` - Admin token (required)
- `ADMIN_ROUTE_PREFIX` - Route prefix (optional, default `/admin`)

All requests must include Authorization header:
```http
Authorization: Bearer your_secure_admin_token
```

## API Endpoints

**Base URL**: `http://your-domain:port/admin`

### Common Response Format

```json
{
  "code": 0,
  "data": { /* response data */ }
}
```

### GET /admin/link - Get All Short Links

```bash
curl -H "Authorization: Bearer your_token" \
     http://localhost:8080/admin/link
```

### POST /admin/link - Create Short Link

```bash
curl -X POST \
     -H "Authorization: Bearer your_token" \
     -H "Content-Type: application/json" \
     -d '{"code":"github","target":"https://github.com"}' \
     http://localhost:8080/admin/link
```

**Request Body**:
```json
{
  "code": "github",
  "target": "https://github.com",
  "expires_at": "2024-12-31T23:59:59Z"  // optional
}
```

### GET /admin/link/{code} - Get Specific Short Link

```bash
curl -H "Authorization: Bearer your_token" \
     http://localhost:8080/admin/link/github
```

### PUT /admin/link/{code} - Update Short Link

```bash
curl -X PUT \
     -H "Authorization: Bearer your_token" \
     -H "Content-Type: application/json" \
     -d '{"code":"github","target":"https://github.com/new-repo"}' \
     http://localhost:8080/admin/link/github
```

### DELETE /admin/link/{code} - Delete Short Link

```bash
curl -X DELETE \
     -H "Authorization: Bearer your_token" \
     http://localhost:8080/admin/link/github
```

## Error Codes

| Error Code | Description |
|------------|-------------|
| 0 | Success |
| 1 | General error |
| 401 | Authentication failed |

## Python Client Example

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

# Usage example
admin = ShortlinkerAdmin('http://localhost:8080', 'your_token')
result = admin.create_link('test', 'https://example.com')
```

## Security Recommendations

1. **Strong Password**: Use sufficiently complex ADMIN_TOKEN
2. **HTTPS**: Production environment must use HTTPS
3. **Network Isolation**: Only expose Admin API in trusted network environments
4. **Regular Rotation**: Regularly change Admin Token
