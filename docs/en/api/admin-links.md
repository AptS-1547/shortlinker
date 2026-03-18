# Admin API: Links and Batch Operations

This page focuses on link CRUD, batch operations, and CSV import/export. Authentication and common response format are documented in [Admin API Overview](/en/api/admin).

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

> Defaults: `page=1`, `page_size=20`; `page_size` is clamped to `1-100`.
>
> `only_expired` and `only_active` cannot both be `true`; otherwise the API returns `400 Bad Request`.

**Response**:
```json
{
  "code": 0,
  "message": "OK",
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
  - Constraints: non-empty, length ≤ 128, allowed chars `[a-zA-Z0-9_.-/]` (multi-level paths supported)
  - Must not conflict with reserved route prefixes (default `admin` / `health` / `panel`, from `routes.*_prefix`): it cannot equal the prefix, and cannot start with `{prefix}/`
- `target` required
- `expires_at` optional (relative like `"7d"` or RFC3339)
- `force` optional (default `false`); when `code` exists and `force=false`, returns `409 Conflict`
- `password` experimental
  - Admin API treats user input as plaintext and always hashes it with Argon2 (even if input starts with `$argon2...`, it is hashed again)
  - If you need to preserve pre-hashed values, use the CSV import path (import logic keeps `$argon2...` as-is)
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
  - `$argon2...` => still treated as user input and hashed again

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

> All three batch endpoints (`POST/PUT/DELETE /links/batch`) accept at most `5000` items per request. Larger payloads return `400 Bad Request` + `BatchSizeTooLarge`.

### POST /links/batch - Batch create

> The request body is an object with `links`, not a raw array.
>
> `links[].code` follows the same short-code constraints and reserved-prefix rules described above.

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
>
> `payload.target` is required (same rule as single-link update).

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

Supported filters: `search`, `created_after`, `created_before`, `only_expired`, `only_active` (date params must be RFC3339).

Current implementation uses **streaming export** (cursor pagination + `Transfer-Encoding: chunked`), which is suitable for large datasets.

The default filename in `Content-Disposition` is: `shortlinks_export_YYYYMMDD_HHMMSS.csv`.

```bash
curl -sS -b cookies.txt \
  -o shortlinks_export.csv \
  "http://localhost:8080/admin/v1/links/export?only_active=true"
```

### POST /links/import - Import CSV

Multipart form fields:
- `file`: CSV file (max 10MB; oversized uploads return `400` + `FileTooLarge`)
- `mode` (optional): `skip` (default) / `overwrite` / `error` (invalid values fall back to `skip`)

Import behavior details:
- `mode=skip`: existing codes and duplicate codes inside the same CSV are skipped
- `mode=overwrite`: allows overwrite; for duplicate codes inside the same CSV, the last row wins
- `mode=error`: existing codes and duplicate codes inside the same CSV are reported as failed items
- Invalid `created_at` falls back to current time; invalid/empty `expires_at` is treated as no expiration
- `password`: plaintext values are Argon2-hashed; values starting with `$argon2...` are kept as pre-hashed

```bash
curl -sS -X POST \
  -b cookies.txt -c cookies.txt \
  -H "X-CSRF-Token: ${CSRF_TOKEN}" \
  -F "mode=overwrite" \
  -F "file=@./shortlinks_export.csv" \
  http://localhost:8080/admin/v1/links/import
```

**Response**:
```json
{
  "code": 0,
  "message": "OK",
  "data": {
    "total_rows": 10,
    "success_count": 9,
    "skipped_count": 1,
    "failed_count": 0,
    "failed_items": []
  }
}
```

`failed_items` fields:
- `row`: CSV line number (1-based; header is line 1, so first data row is usually 2)
- `code`: failed short code (can be empty for CSV parse failures)
- `error`: human-readable error message
- `error_code`: mapped server error code (optional)

