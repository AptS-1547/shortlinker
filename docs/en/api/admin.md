# Admin API Documentation

Shortlinker v0.0.5+ provides a complete HTTP API for managing short links.

## Feature Overview

Admin API supports complete CRUD operations for short links:
- Create new short links
- Query individual or all short links
- Update existing short links
- Delete short links

## Authentication Configuration

### Environment Variables

```bash
# Required: Set admin token
ADMIN_TOKEN=your_secure_admin_token

# Optional: Custom route prefix
ADMIN_ROUTE_PREFIX=/api/admin
```

### Request Headers

All Admin API requests require the Authorization header:

```http
Authorization: Bearer your_secure_admin_token
```

## API Endpoints

### Basic Information

- **Base URL**: `http://your-domain:port{ADMIN_ROUTE_PREFIX}`
- **Default Prefix**: `/admin`
- **Authentication**: Bearer Token
- **Response Format**: JSON

### Common Response Format

```json
{
  "code": 0,
  "data": { /* response data */ }
}
```

Error response:
```json
{
  "code": non-zero error code,
  "data": {
    "error": "error description"
  }
}
```

## Interface Details

### GET /admin/link - Get All Short Links

Get a list of all short links in the system.

**Request**:
```bash
curl -H "Authorization: Bearer your_token" \
     http://localhost:8080/admin/link
```

**Response**:
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

### POST /admin/link - Create Short Link

Create a new short link.

**Request Body**:
```json
{
  "code": "github",
  "target": "https://github.com",
  "expires_at": "2024-12-31T23:59:59Z"  // optional
}
```

**Example**:
```bash
curl -X POST \
     -H "Authorization: Bearer your_token" \
     -H "Content-Type: application/json" \
     -d '{"code":"github","target":"https://github.com"}' \
     http://localhost:8080/admin/link
```

**Response**:
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

### GET /admin/link/{code} - Get Specific Short Link

Get information for a specific short code.

**Request**:
```bash
curl -H "Authorization: Bearer your_token" \
     http://localhost:8080/admin/link/github
```

**Response**:
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

### PUT /admin/link/{code} - Update Short Link

Update the target address and expiration time for a specific short code.

**Request Body**:
```json
{
  "code": "github",
  "target": "https://github.com/new-repo",
  "expires_at": "2025-01-01T00:00:00Z"
}
```

**Example**:
```bash
curl -X PUT \
     -H "Authorization: Bearer your_token" \
     -H "Content-Type: application/json" \
     -d '{"code":"github","target":"https://github.com/new-repo"}' \
     http://localhost:8080/admin/link/github
```

### DELETE /admin/link/{code} - Delete Short Link

Delete the specified short link.

**Request**:
```bash
curl -X DELETE \
     -H "Authorization: Bearer your_token" \
     http://localhost:8080/admin/link/github
```

**Response**:
```json
{
  "code": 0,
  "data": {
    "message": "Link deleted successfully"
  }
}
```

## Error Codes

| Error Code | Description |
|------------|-------------|
| 0 | Success |
| 1 | General error |
| 401 | Authentication failed |

## Usage Examples

### Python Example

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

# Usage example
admin = ShortlinkerAdmin('http://localhost:8080', 'your_token')

# Create link
result = admin.create_link('test', 'https://example.com')
print(result)

# Get all links
links = admin.get_all_links()
print(links)
```

### JavaScript Example

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

// Usage example
const admin = new ShortlinkerAdmin('http://localhost:8080', 'your_token');

// Create link
admin.createLink('test', 'https://example.com')
    .then(result => console.log(result));
```

## Security Recommendations

1. **Strong Password**: Use sufficiently complex ADMIN_TOKEN
2. **HTTPS**: Production environment must use HTTPS
3. **Network Isolation**: Only expose Admin API in trusted network environments
4. **Regular Rotation**: Regularly rotate Admin Token
5. **Log Monitoring**: Monitor Admin API access logs
6. **Custom Prefix**: Use non-obvious route prefix in production
7. **Rate Limiting**: Consider implementing rate limiting for Admin API endpoints

### Token Generation Examples

```bash
# Generate secure random token
openssl rand -hex 32

# Or use uuidgen
uuidgen | tr -d '-'

# Or use pwgen
pwgen -s 32 1
```
