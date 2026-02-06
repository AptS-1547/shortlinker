# Admin API

The Admin API docs are now split by topic so you can find what you need quickly without scrolling through a single long page.

## Navigation

- [Admin API: Links and Batch Operations](/en/api/admin-links)
- [Admin API: Runtime Config and Automation](/en/api/admin-config)
- [Admin API: Analytics](/en/api/admin-analytics)

## Configuration

Admin API settings are **runtime config (stored in DB)**. See [Configuration Guide](/en/config/).

- `api.admin_token`: admin login password (stored as Argon2 hash; first startup may generate `admin_token.txt`; remove it after saving; recommended reset method: `./shortlinker reset-password`)
- `routes.admin_prefix`: route prefix (default `/admin`, restart required after change)

> API paths always include `/v1`. Default login URL: `http://localhost:8080/admin/v1/auth/login`.

## Authentication (Important)

Admin API supports two authentication modes:

1. **JWT cookies (recommended for browser/admin panel)**
   - Access cookie: `shortlinker_access` (`Path=/`)
   - Refresh cookie: `shortlinker_refresh` (`Path={ADMIN_ROUTE_PREFIX}/v1/auth`)
   - CSRF cookie: `csrf_token` (`Path=/`, not HttpOnly so frontend can read it)
2. **Bearer token (recommended for API clients, no CSRF needed)**
   - `Authorization: Bearer <ACCESS_TOKEN>` (`<ACCESS_TOKEN>` is the same JWT access token used by `shortlinker_access` cookie)

> Cookie names are currently fixed. Expiration/SameSite/Secure/Domain options are configurable via `api.*` in runtime config.

### 1) Login to get cookies

**POST** `/{ADMIN_ROUTE_PREFIX}/v1/auth/login`

Request body:
```json
{ "password": "your_admin_token" }
```

Example (`cookies.txt` as cookie jar):
```bash
curl -sS -X POST \
  -H "Content-Type: application/json" \
  -c cookies.txt \
  -d '{"password":"your_admin_token"}' \
  http://localhost:8080/admin/v1/auth/login
```

### 2) CSRF protection (required for cookie-authenticated writes)

When using **JWT cookies** for write operations (`POST`/`PUT`/`DELETE`), include both:

- Cookie: `csrf_token`
- Header: `X-CSRF-Token: <csrf_token value>`

Exceptions:

- `POST /auth/login`, `POST /auth/refresh`, `POST /auth/logout` do not require CSRF
- `GET/HEAD/OPTIONS` do not require CSRF
- If you use `Authorization: Bearer <ACCESS_TOKEN>` for writes, CSRF is not required

Extract CSRF token example:

```bash
CSRF_TOKEN=$(awk '$6=="csrf_token"{print $7}' cookies.txt | tail -n 1)
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

> If you changed `routes.admin_prefix`, replace `/admin` accordingly.

## Common JSON format

Most endpoints return JSON:

```json
{
  "code": 0,
  "message": "OK",
  "data": { /* payload */ }
}
```

- `code = 0`: success
- `code != 0`: failure (`ErrorCode` enum value; reason in `message`; `data` is often omitted)
- `message`: human-readable text; usually `OK` on success
- HTTP status expresses error class (`401/404/409/500`, etc.)

## Security notes

1. Use a strong admin password (`api.admin_token`) and avoid default/generated values in production
2. Use HTTPS in production and set `api.cookie_secure=true`
3. Expose Admin API only to trusted networks
4. Rotate admin credentials regularly and re-login to refresh cookies
