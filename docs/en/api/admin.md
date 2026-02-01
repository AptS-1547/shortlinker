# Admin API

Shortlinker provides a full-featured HTTP Admin API for managing short links, including CRUD, batch operations, CSV import/export, and runtime config management.

## Configuration

Admin API settings can come from `config.toml`, environment variables, or runtime config (database). See [Configuration](/en/config/).

- `ADMIN_TOKEN`: admin login password (recommended to set explicitly in production; if not set, the server will auto-generate one and write it once to `admin_token.txt` (save it and delete the file))
- `ADMIN_ROUTE_PREFIX`: route prefix (optional, default: `/admin`)

> All API paths include `/v1`, e.g. the default login endpoint is `http://localhost:8080/admin/v1/auth/login`.

## Authentication (Important)

Admin API supports two authentication methods:

1. **JWT cookies (recommended for browser/admin panel)**
   - Access cookie: `shortlinker_access` (`Path=/`)
   - Refresh cookie: `shortlinker_refresh` (`Path={ADMIN_ROUTE_PREFIX}/v1/auth`)
   - CSRF cookie: `csrf_token` (`Path={ADMIN_ROUTE_PREFIX}`, not HttpOnly so the frontend can read it)
2. **Bearer token (for API clients; CSRF-free)**
   - `Authorization: Bearer <ACCESS_TOKEN>` (where `<ACCESS_TOKEN>` is the same JWT access token as the `shortlinker_access` cookie value)

> Note: cookie names are currently fixed (not configurable). Cookie TTL / SameSite / Secure / Domain can be adjusted via `api.*` (see [Configuration](/en/config/)).

### 1) Login to get cookies

**POST** `/{ADMIN_ROUTE_PREFIX}/v1/auth/login`

Body:
```json
{ "password": "your_admin_token" }
```

Example (save cookies to `cookies.txt`):
```bash
curl -sS -X POST \
  -H "Content-Type: application/json" \
  -c cookies.txt \
  -d '{"password":"your_admin_token"}' \
  http://localhost:8080/admin/v1/auth/login
```

> Tokens are returned via `Set-Cookie` (access/refresh/csrf). The response body does not include raw token strings.

### CSRF protection (Important)

When you use **JWT cookie auth** for write operations (`POST`/`PUT`/`DELETE`), you must provide:

- Cookie: `csrf_token`
- Header: `X-CSRF-Token: <value of csrf_token cookie>`

> Exceptions: `POST /auth/login`, `POST /auth/refresh`, `POST /auth/logout` do not require CSRF; `GET/HEAD/OPTIONS` also do not.  
> If you use `Authorization: Bearer <ACCESS_TOKEN>` for write operations, CSRF is not required.

Example (extract CSRF token from `cookies.txt`):

```bash
CSRF_TOKEN=$(awk '$6=="csrf_token"{print $7}' cookies.txt | tail -n 1)
```

### 2) Call other endpoints with cookies

```bash
curl -sS -b cookies.txt \
  http://localhost:8080/admin/v1/links
```

### 3) Refresh tokens

**POST** `/{ADMIN_ROUTE_PREFIX}/v1/auth/refresh`

```bash
curl -sS -X POST \
  -b cookies.txt -c cookies.txt \
  http://localhost:8080/admin/v1/auth/refresh
```

### 4) Logout (clear cookies)

**POST** `/{ADMIN_ROUTE_PREFIX}/v1/auth/logout`

```bash
curl -sS -X POST -b cookies.txt -c cookies.txt \
  http://localhost:8080/admin/v1/auth/logout
```

## Base URL

Default: `http://your-domain:port/admin/v1`

> If you changed `ADMIN_ROUTE_PREFIX`, replace `/admin` with your prefix.

## Common JSON format

Most endpoints return:
```json
{
  "code": 0,
  "data": { /* payload */ }
}
```

- `code = 0`: success
- `code = 1`: error (details in `data.error`)
- HTTP status code indicates error category (`401/404/409/500`, etc.)

## Link management

### GET /links - List short links (pagination + filters)

```bash
curl -sS -b cookies.txt \
  "http://localhost:8080/admin/v1/links?page=1&page_size=20"
```

**Query params**:

| Param | Type | Description | Example |
|------|------|-------------|---------|
| `page` | Integer | page index (starts from 1) | `?page=1` |
| `page_size` | Integer | page size (1-100) | `?page_size=20` |
| `search` | String | fuzzy search on code + target | `?search=github` |
| `created_after` | RFC3339 | created_at >= | `?created_after=2024-01-01T00:00:00Z` |
| `created_before` | RFC3339 | created_at <= | `?created_before=2024-12-31T23:59:59Z` |
| `only_expired` | Boolean | only expired links | `?only_expired=true` |
| `only_active` | Boolean | only active (not expired) | `?only_active=true` |

**Response**:
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

### POST /links - Create a short link

```bash
curl -sS -X POST \
  -b cookies.txt \
  -H "X-CSRF-Token: ${CSRF_TOKEN}" \
  -H "Content-Type: application/json" \
  -d '{"code":"github","target":"https://github.com"}' \
  http://localhost:8080/admin/v1/links
```

**Body**:
```json
{
  "code": "github",
  "target": "https://github.com",
  "expires_at": "2024-12-31T23:59:59Z",
  "password": "secret123",
  "force": false
}
```

Notes:
- `code` optional (auto-generated if omitted)
- `target` required
- `expires_at` optional (relative like `"7d"` or RFC3339)
- `force` optional (default `false`); when `code` exists and `force=false`, returns `409 Conflict`
- `password` experimental
  - Admin API hashes plaintext passwords using Argon2 (if input already starts with `$argon2...`, it will be stored as-is)
  - Redirect does not validate password in current version (stored only)

### GET /links/{code} - Get a link

```bash
curl -sS -b cookies.txt \
  http://localhost:8080/admin/v1/links/github
```

### PUT /links/{code} - Update a link

```bash
curl -sS -X PUT \
  -b cookies.txt \
  -H "X-CSRF-Token: ${CSRF_TOKEN}" \
  -H "Content-Type: application/json" \
  -d '{"target":"https://github.com/new-repo","expires_at":"30d"}' \
  http://localhost:8080/admin/v1/links/github
```

**Body**:
```json
{
  "target": "https://new-url.com",
  "expires_at": "7d",
  "password": ""
}
```

Notes:
- `target` is required
- `expires_at` omitted => keep existing value
- `password`
  - omitted => keep existing
  - empty string `""` => remove password
  - plaintext => hash with Argon2
  - `$argon2...` => store as-is

### DELETE /links/{code} - Delete a link

```bash
curl -sS -X DELETE -b cookies.txt \
  -H "X-CSRF-Token: ${CSRF_TOKEN}" \
  http://localhost:8080/admin/v1/links/github
```

### GET /stats - Stats

```bash
curl -sS -b cookies.txt \
  http://localhost:8080/admin/v1/stats
```

## Batch operations

### POST /links/batch - Batch create

> The request body is an object with `links`, not a raw array.

```bash
curl -sS -X POST \
  -b cookies.txt \
  -H "X-CSRF-Token: ${CSRF_TOKEN}" \
  -H "Content-Type: application/json" \
  -d '{"links":[{"code":"link1","target":"https://example1.com"},{"code":"link2","target":"https://example2.com"}]}' \
  http://localhost:8080/admin/v1/links/batch
```

### PUT /links/batch - Batch update

> The request body uses `updates`, each item includes `code` and `payload`.

```bash
curl -sS -X PUT \
  -b cookies.txt \
  -H "X-CSRF-Token: ${CSRF_TOKEN}" \
  -H "Content-Type: application/json" \
  -d '{"updates":[{"code":"link1","payload":{"target":"https://new-example1.com"}},{"code":"link2","payload":{"target":"https://new-example2.com"}}]}' \
  http://localhost:8080/admin/v1/links/batch
```

### DELETE /links/batch - Batch delete

> The request body uses `codes`.

```bash
curl -sS -X DELETE \
  -b cookies.txt \
  -H "X-CSRF-Token: ${CSRF_TOKEN}" \
  -H "Content-Type: application/json" \
  -d '{"codes":["link1","link2","link3"]}' \
  http://localhost:8080/admin/v1/links/batch
```

## CSV export/import

### GET /links/export - Export CSV

The exported CSV contains a header and these columns:
`code,target,created_at,expires_at,password,click_count`

```bash
curl -sS -b cookies.txt \
  -o shortlinks_export.csv \
  "http://localhost:8080/admin/v1/links/export?only_active=true"
```

### POST /links/import - Import CSV

Multipart form fields:
- `file`: CSV file
- `mode` (optional): `skip` (default) / `overwrite` / `error`

```bash
curl -sS -X POST \
  -b cookies.txt -c cookies.txt \
  -H "X-CSRF-Token: ${CSRF_TOKEN}" \
  -F "mode=overwrite" \
  -F "file=@./shortlinks_export.csv" \
  http://localhost:8080/admin/v1/links/import
```

## Runtime config management

Config endpoints are under `/{ADMIN_ROUTE_PREFIX}/v1/config`. Sensitive values are masked as `[REDACTED]`.

### GET /config
```bash
curl -sS -b cookies.txt \
  http://localhost:8080/admin/v1/config
```

### GET /config/schema

Returns schema metadata for all config keys (type, default value, whether restart is required, enum options, etc.). Mainly used by the admin panel to render/validate config forms.

```bash
curl -sS -b cookies.txt \
  http://localhost:8080/admin/v1/config/schema
```

### GET /config/{key}
```bash
curl -sS -b cookies.txt \
  http://localhost:8080/admin/v1/config/features.random_code_length
```

### PUT /config/{key}
```bash
curl -sS -X PUT \
  -b cookies.txt \
  -H "X-CSRF-Token: ${CSRF_TOKEN}" \
  -H "Content-Type: application/json" \
  -d '{"value":"8"}' \
  http://localhost:8080/admin/v1/config/features.random_code_length
```

### GET /config/{key}/history
```bash
curl -sS -b cookies.txt \
  "http://localhost:8080/admin/v1/config/features.random_code_length/history?limit=10"
```

### POST /config/reload
```bash
curl -sS -X POST -b cookies.txt \
  -H "X-CSRF-Token: ${CSRF_TOKEN}" \
  http://localhost:8080/admin/v1/config/reload
```

## Auth endpoints notes

- `POST /auth/login`: no cookies required; validates `ADMIN_TOKEN` and sets cookies
- `POST /auth/refresh`: no access cookie required, but refresh cookie is required
- `POST /auth/logout`: no cookies required; clears cookies
- `GET /auth/verify`: requires access cookie

## Python example (requests)

```python
import requests

class ShortlinkerAdmin:
    def __init__(self, base_url: str, admin_token: str):
        self.base_url = base_url.rstrip("/")
        self.session = requests.Session()

        resp = self.session.post(
            f"{self.base_url}/admin/v1/auth/login",
            json={"password": admin_token},
            timeout=10,
        )
        resp.raise_for_status()

    def list_links(self, page=1, page_size=20):
        resp = self.session.get(
            f"{self.base_url}/admin/v1/links",
            params={"page": page, "page_size": page_size},
            timeout=10,
        )
        resp.raise_for_status()
        return resp.json()

admin = ShortlinkerAdmin("http://localhost:8080", "your_admin_token")
print(admin.list_links())
```

## Security notes

1. Use a strong `ADMIN_TOKEN` (do not rely on the auto-generated one in production)
2. Use HTTPS in production and set `api.cookie_secure=true`
3. Expose Admin API only to trusted networks
4. Rotate `ADMIN_TOKEN` regularly and re-login to get new cookies
