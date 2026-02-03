# API Documentation

Shortlinker provides a simple HTTP API interface for short link redirection.

## API Overview

Shortlinker mainly provides a redirection interface that supports GET and HEAD methods.

## Basic Information

- **Base URL**: `http://your-domain:port/`
- **Protocol**: HTTP/1.1
- **Encoding**: UTF-8
- **Redirect Type**: 307 Temporary Redirect

## API Details

### GET/HEAD /{path...}

Redirects to the target URL corresponding to the specified short code.

**Request Methods**: `GET` | `HEAD`

**Request Path**: `/{path}` (multi-level paths supported, e.g. `/foo/bar`)

**Path Parameters**:
- `path` (string): Short link code (case-sensitive)

**Short code constraints** (violations return `404` immediately):
- Max length: 128
- Allowed characters: `[a-zA-Z0-9_.-/]`

**Responses**:

#### Successful Redirect (307)
```http
HTTP/1.1 307 Temporary Redirect
Location: https://example.com
Cache-Control: no-cache, no-store, must-revalidate
```

#### Short Code Not Found/Expired (404)
```http
HTTP/1.1 404 Not Found
Content-Type: text/html; charset=utf-8
Cache-Control: public, max-age=60

Not Found
```

## Special Paths

### Root Path Redirect

When accessing the root path `/`, it redirects to the default URL (runtime config key `features.default_url`).

**Request**:
```http
GET / HTTP/1.1
Host: localhost:8080
```

**Response**:
```http
HTTP/1.1 307 Temporary Redirect
Location: https://esap.cc/repo
```

## Usage Examples

### curl Examples

```bash
# Redirect request
curl -I http://localhost:8080/example
# HTTP/1.1 307 Temporary Redirect
# Location: https://www.example.com

# Follow redirects
curl -L http://localhost:8080/example

# Non-existent short code
curl -I http://localhost:8080/nonexistent
# HTTP/1.1 404 Not Found
```

### JavaScript Example

```javascript
async function checkShortLink(shortCode) {
    try {
        const response = await fetch(`http://localhost:8080/${shortCode}`, {
            method: 'HEAD',
            redirect: 'manual'
        });
        
        if (response.status === 307) {
            return response.headers.get('Location');
        } else {
            return null;
        }
    } catch (error) {
        console.error('Check failed:', error);
        return null;
    }
}
```

### Python Example

```python
import requests

def check_short_link(base_url, short_code):
    """Check short link and return target URL"""
    try:
        response = requests.head(
            f"{base_url}/{short_code}",
            allow_redirects=False
        )
        return response.headers.get('Location') if response.status_code == 307 else None
    except requests.RequestException:
        return None
```

## Caching Strategy

- **307 Redirect responses**: Include `Cache-Control: no-cache, no-store, must-revalidate` to ensure:
  - Browsers won't cache redirect responses
  - Short link modifications take effect immediately
  - Expiration checks are performed in real-time

- **404 Not Found responses**: Include `Cache-Control: public, max-age=60` to allow short-term caching and reduce invalid requests.

## Performance Characteristics

- **Response Time**: < 1ms (SQLite local storage)
- **Concurrency Support**: Thousands of concurrent connections
- **Memory Usage**: Extremely low memory footprint
- **Storage Backends**: Supports SQLite, MySQL, PostgreSQL, MariaDB multiple storage types

## Monitoring and Logging

The server logs the following information:
- Redirect operation logs
- 404 error logs
- Expired link access logs

Log examples:
```
[INFO] Redirect example -> https://www.example.com
[INFO] Link expired: temp
```
