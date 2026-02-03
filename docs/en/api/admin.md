# Admin API

Shortlinker provides a full-featured HTTP Admin API for managing short links, including CRUD, batch operations, CSV import/export, and runtime config management.

## Configuration

Admin API settings are **runtime config (database)**. See [Configuration](/en/config/).

- `api.admin_token`: admin login password (stored as an Argon2 hash in the DB; on first startup a random password is generated and written to `admin_token.txt`; save it and delete the file; rotate via `./shortlinker reset-password`)
- `routes.admin_prefix`: route prefix (default: `/admin`, restart required)

> All API paths include `/v1`, e.g. the default login endpoint is `http://localhost:8080/admin/v1/auth/login`.

## Authentication (Important)

Admin API supports two authentication methods:

1. **JWT cookies (recommended for browser/admin panel)**
   - Access cookie: `shortlinker_access` (`Path=/`)
   - Refresh cookie: `shortlinker_refresh` (`Path={ADMIN_ROUTE_PREFIX}/v1/auth`)
   - CSRF cookie: `csrf_token` (`Path=/`, not HttpOnly so the frontend can read it)
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

> If you changed `routes.admin_prefix`, replace `/admin` with your prefix.

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

- `POST /auth/login`: no cookies required; validates the admin password (plaintext for `api.admin_token`) and sets cookies
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

1. Use a strong admin password (`api.admin_token`) (do not rely on the auto-generated one in production)
2. Use HTTPS in production and set `api.cookie_secure=true`
3. Expose Admin API only to trusted networks
4. Rotate the admin password (`api.admin_token`) regularly and re-login to get new cookies

## Analytics API

Analytics API provides detailed click statistics, including click trends, top links, referrer stats, and geographic distribution.

> You need to enable `analytics.enable_detailed_logging` in runtime config (restart required) to record detailed click logs.
>
> - Default range: last 30 days. To set a custom range, provide **both** `start_date` and `end_date`.
> - Date formats: RFC3339 (e.g. `2024-01-01T00:00:00Z`) or `YYYY-MM-DD` (e.g. `2024-01-01`).
> - Geo distribution requires `analytics.enable_geo_lookup=true` (and `analytics.enable_ip_logging=true` to keep IPs). The GeoIP provider is configured via startup `[analytics]` (`analytics.maxminddb_path` / `analytics.geoip_api_url`).
>   - When using the external API provider, it has a built-in cache (LRU 10,000, TTL 15 minutes, negative caching on failures, and singleflight for concurrent requests). HTTP timeout is 2 seconds.

### GET /analytics/trends - Get click trends

```bash
curl -sS -b cookies.txt \
  "http://localhost:8080/admin/v1/analytics/trends?start_date=2024-01-01T00:00:00Z&end_date=2024-12-31T23:59:59Z&group_by=day"
```

**Query params**:

| Param | Type | Description | Example |
|-------|------|-------------|---------|
| `start_date` | RFC3339 / YYYY-MM-DD | Start date (optional; must be provided together with `end_date`; default = last 30 days) | `?start_date=2024-01-01T00:00:00Z` |
| `end_date` | RFC3339 / YYYY-MM-DD | End date (optional; must be provided together with `start_date`; default = last 30 days) | `?end_date=2024-12-31T23:59:59Z` |
| `group_by` | String | Grouping (optional; default `day`): `hour`/`day`/`week`/`month` | `?group_by=day` |

**Response**:
```json
{
  "code": 0,
  "data": {
    "labels": ["2024-01-01", "2024-01-02", "2024-01-03"],
    "values": [100, 150, 120]
  }
}
```

### GET /analytics/top - Get top links

```bash
curl -sS -b cookies.txt \
  "http://localhost:8080/admin/v1/analytics/top?limit=10"
```

**Query params**:

| Param | Type | Description | Example |
|-------|------|-------------|---------|
| `start_date` | RFC3339 / YYYY-MM-DD | Start date (optional; must be provided together with `end_date`; default = last 30 days) | `?start_date=2024-01-01T00:00:00Z` |
| `end_date` | RFC3339 / YYYY-MM-DD | End date (optional; must be provided together with `start_date`; default = last 30 days) | `?end_date=2024-12-31T23:59:59Z` |
| `limit` | Integer | Number of results (optional; default 10; max 100) | `?limit=10` |

**Response**:
```json
{
  "code": 0,
  "data": [
    {"code": "github", "clicks": 500},
    {"code": "google", "clicks": 300}
  ]
}
```

### GET /analytics/referrers - Get referrer stats

```bash
curl -sS -b cookies.txt \
  "http://localhost:8080/admin/v1/analytics/referrers?limit=10"
```

**Query params**:

| Param | Type | Description | Example |
|-------|------|-------------|---------|
| `start_date` | RFC3339 / YYYY-MM-DD | Start date (optional; must be provided together with `end_date`; default = last 30 days) | `?start_date=2024-01-01T00:00:00Z` |
| `end_date` | RFC3339 / YYYY-MM-DD | End date (optional; must be provided together with `start_date`; default = last 30 days) | `?end_date=2024-12-31T23:59:59Z` |
| `limit` | Integer | Number of results (optional; default 10; max 100) | `?limit=10` |

**Response**:
```json
{
  "code": 0,
  "data": [
    {"referrer": "https://google.com", "count": 200, "percentage": 40.0},
    {"referrer": "(direct)", "count": 150, "percentage": 30.0}
  ]
}
```

### GET /analytics/geo - Get geographic distribution

```bash
curl -sS -b cookies.txt \
  "http://localhost:8080/admin/v1/analytics/geo?limit=20"
```

**Query params**:

| Param | Type | Description | Example |
|-------|------|-------------|---------|
| `start_date` | RFC3339 / YYYY-MM-DD | Start date (optional; must be provided together with `end_date`; default = last 30 days) | `?start_date=2024-01-01T00:00:00Z` |
| `end_date` | RFC3339 / YYYY-MM-DD | End date (optional; must be provided together with `start_date`; default = last 30 days) | `?end_date=2024-12-31T23:59:59Z` |
| `limit` | Integer | Number of results (optional; default 20; max 100) | `?limit=20` |

**Response**:
```json
{
  "code": 0,
  "data": [
    {"country": "CN", "city": "Beijing", "count": 100},
    {"country": "US", "city": "New York", "count": 80}
  ]
}
```

### GET /links/{code}/analytics - Get single link analytics

```bash
curl -sS -b cookies.txt \
  "http://localhost:8080/admin/v1/links/github/analytics"
```

**Query params**:

| Param | Type | Description | Example |
|-------|------|-------------|---------|
| `start_date` | RFC3339 / YYYY-MM-DD | Start date (optional; must be provided together with `end_date`; default = last 30 days) | `?start_date=2024-01-01` |
| `end_date` | RFC3339 / YYYY-MM-DD | End date (optional; must be provided together with `start_date`; default = last 30 days) | `?end_date=2024-12-31` |

**Response**:
```json
{
  "code": 0,
  "data": {
    "code": "github",
    "total_clicks": 500,
    "trend": {
      "labels": ["2024-01-01", "2024-01-02"],
      "values": [100, 150]
    },
    "top_referrers": [
      {"referrer": "https://google.com", "count": 100, "percentage": 20.0}
    ],
    "geo_distribution": [
      {"country": "CN", "city": "Beijing", "count": 50}
    ]
  }
}
```

### GET /analytics/export - Export analytics report (CSV)

```bash
curl -sS -b cookies.txt \
  -o analytics_report.csv \
  "http://localhost:8080/admin/v1/analytics/export?start_date=2024-01-01T00:00:00Z&end_date=2024-12-31T23:59:59Z"
```

**Query params**:

| Param | Type | Description | Example |
|-------|------|-------------|---------|
| `start_date` | RFC3339 / YYYY-MM-DD | Start date (optional; must be provided together with `end_date`; default = last 30 days) | `?start_date=2024-01-01T00:00:00Z` |
| `end_date` | RFC3339 / YYYY-MM-DD | End date (optional; must be provided together with `start_date`; default = last 30 days) | `?end_date=2024-12-31T23:59:59Z` |
| `limit` | Integer | Export record limit (optional; default 10000; max 100000) | `?limit=10000` |

The exported CSV contains these columns:
`short_code,clicked_at,referrer,user_agent,ip_address,country,city`

### Analytics configuration

These runtime config options control Analytics behavior:

| Config key | Type | Default | Description |
|------------|------|---------|-------------|
| `analytics.enable_detailed_logging` | bool | false | Enable detailed logging (writes to click_logs table; restart required) |
| `analytics.log_retention_days` | int | 30 | Log retention period in days (automatic cleanup is not implemented yet) |
| `analytics.enable_ip_logging` | bool | true | Whether to record IP addresses |
| `analytics.enable_geo_lookup` | bool | false | Whether to enable geo-IP lookup |
