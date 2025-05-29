# API Documentation

Shortlinker provides a simple HTTP API interface for short link redirection.

## Interface Overview

Shortlinker mainly provides a redirection interface that supports GET and HEAD methods.

## Basic Information

- **Base URL**: `http://your-domain:port/`
- **Protocol**: HTTP/1.1
- **Encoding**: UTF-8
- **Redirect Type**: 307 Temporary Redirect

## Interface Details

### GET/HEAD /{path}

Redirect to the target URL corresponding to the specified short code.

**Request Method**: `GET` | `HEAD`

**Request Path**: `/{short_code}`

**Path Parameters**:
- `short_code` (string): Short link code

**Response**:

#### Successful Redirect (307)
```http
HTTP/1.1 307 Temporary Redirect
Location: https://example.com
Cache-Control: no-cache, no-store, must-revalidate
```

#### Short Code Not Found (404)
```http
HTTP/1.1 404 Not Found
Content-Type: text/html; charset=utf-8
Connection: close
Cache-Control: no-cache, no-store, must-revalidate

Not Found
```

#### Short Link Expired (404)
```http
HTTP/1.1 404 Not Found
Content-Type: text/html; charset=utf-8
Connection: close
Cache-Control: no-cache, no-store, must-revalidate

Not Found
```

## Special Paths

### Root Path Redirect

When accessing the root path `/`, it redirects to the default URL (configured via `DEFAULT_URL` environment variable).

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

## Examples

### Using curl

```bash
# Redirect request
curl -I http://localhost:8080/example
# HTTP/1.1 307 Temporary Redirect
# Location: https://www.example.com

# Follow redirect
curl -L http://localhost:8080/example
# (Returns target website content)

# Non-existent short code
curl -I http://localhost:8080/nonexistent
# HTTP/1.1 404 Not Found
```

### Using JavaScript

```javascript
// Check if short link exists
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

// Usage example
checkShortLink('example').then(targetUrl => {
    if (targetUrl) {
        console.log('Target URL:', targetUrl);
    } else {
        console.log('Short link does not exist or has expired');
    }
});
```

### Using Python

```python
import requests

def check_short_link(base_url, short_code):
    """Check short link and return target URL"""
    try:
        response = requests.head(
            f"{base_url}/{short_code}",
            allow_redirects=False
        )
        
        if response.status_code == 307:
            return response.headers.get('Location')
        else:
            return None
    except requests.RequestException as e:
        print(f"Request failed: {e}")
        return None

# Usage example
target_url = check_short_link("http://localhost:8080", "example")
if target_url:
    print(f"Target URL: {target_url}")
else:
    print("Short link does not exist or has expired")
```

## Caching Strategy

All responses include the `Cache-Control: no-cache, no-store, must-revalidate` header to ensure:
- Browsers don't cache redirect responses
- Short link changes take effect immediately
- Expiration checks are performed in real-time

## Performance Characteristics

- **Response Time**: < 1ms (local storage)
- **Concurrency Support**: Thousands of concurrent connections
- **Memory Usage**: Extremely low memory footprint
- **CPU Usage**: Efficient hash lookups

## Monitoring and Logging

The server records the following information:
- Redirect operation logs
- 404 error logs
- Expired link access logs
- Performance statistics

Log examples:
```
[2024-01-01T12:00:00Z INFO] Redirect example -> https://www.example.com
[2024-01-01T12:00:01Z INFO] Link expired: temp
```

## Admin API (v0.0.5+)

For management operations, see [Admin API Documentation](/en/api/admin) which provides:
- Create, read, update, delete short links
- Bearer token authentication
- RESTful JSON API
- Disabled by default for security
