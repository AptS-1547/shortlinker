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

**Query Parameters**:

| Parameter | Type | Description | Example |
|-----------|------|-------------|---------|
| `page` | Integer | Page number (starts from 1) | `?page=1` |
| `page_size` | Integer | Items per page (1-100) | `?page_size=20` |
| `created_after` | RFC3339 | Filter by creation time (after) | `?created_after=2024-01-01T00:00:00Z` |
| `created_before` | RFC3339 | Filter by creation time (before) | `?created_before=2024-12-31T23:59:59Z` |
| `only_expired` | Boolean | Show only expired links | `?only_expired=true` |
| `only_active` | Boolean | Show only active links | `?only_active=true` |

**Pagination Examples**:

```bash
# Get page 2 with 10 items per page
curl -H "Authorization: Bearer your_token" \
     "http://localhost:8080/admin/link?page=2&page_size=10"

# Show only active links
curl -H "Authorization: Bearer your_token" \
     "http://localhost:8080/admin/link?only_active=true"

# Combined query: page 1, active only, filtered by time
curl -H "Authorization: Bearer your_token" \
     "http://localhost:8080/admin/link?page=1&page_size=20&only_active=true&created_after=2024-01-01T00:00:00Z"
```

**Response Format** (Paginated):

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
  "expires_at": "2024-12-31T23:59:59Z",  // optional, supports relative time (e.g., "7d")
  "password": "secret123"  // optional, password protection (experimental, storage only)
}
```

**Field Description**:
- `code`: Short code (optional), auto-generates random code if not provided
- `target`: Target URL (required)
- `expires_at`: Expiration time (optional), supports relative time (e.g., `"1d"`, `"7d"`, `"1w"`) or RFC3339 format
- `password`: Password protection (optional) ⚠️ **Note**: Current version only stores password, does not validate on redirect. This is an experimental feature.

**Create password-protected short link**:

```bash
curl -X POST \
     -H "Authorization: Bearer your_token" \
     -H "Content-Type: application/json" \
     -d '{"code":"secret","target":"https://example.com","password":"mypassword"}' \
     http://localhost:8080/admin/link
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
     -d '{"target":"https://github.com/new-repo","expires_at":"30d"}' \
     http://localhost:8080/admin/link/github
```

**Request Body Description**:
```json
{
  "target": "https://new-url.com",  // required
  "expires_at": "7d",  // optional, keeps original if not provided
  "password": "newpass"  // optional, keeps original if not provided, pass null to clear
}
```

**Notes**:
- Update preserves original creation time and click count
- `expires_at` keeps original expiration time if not provided
- `password` keeps original password if not provided, updates if new value provided

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

## Experimental Features

### Password Protection Feature ⚠️

**Current Status**: Experimental / Not Fully Implemented

Shortlinker supports setting password fields for short links, but **the current version only supports storing passwords, validation is not performed during access**.

**Implemented**:
- ✅ Create password-protected short links via API
- ✅ Store and query password fields
- ✅ Update and delete passwords

**Not Implemented**:
- ❌ Password validation when accessing short links
- ❌ Password validation page
- ❌ Encrypted password storage (currently plaintext)

**Usage Example**:

```bash
# Create password-protected short link (password stored but not validated)
curl -X POST \
     -H "Authorization: Bearer your_token" \
     -H "Content-Type: application/json" \
     -d '{"code":"secret","target":"https://example.com","password":"mypass123"}' \
     http://localhost:8080/admin/link

# Query returns password field
curl -H "Authorization: Bearer your_token" \
     http://localhost:8080/admin/link/secret
# Returns: {"code":"secret","target":"...","password":"mypass123",...}
```

**Security Warnings**:
- 🚨 Passwords are stored in plaintext in the database
- 🚨 No password required when accessing short links
- 🚨 Not recommended for production use

**Planned Improvements**:
- Implement password validation page
- Support password hashing
- Support multiple validation methods (HTTP Basic Auth, query parameters, etc.)

For complete password protection functionality, it's recommended to implement access control at the reverse proxy layer (e.g., Nginx).
