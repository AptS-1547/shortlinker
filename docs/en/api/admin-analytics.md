# Admin API: Analytics

This page covers trends, top links, referrers, geo, and device analytics endpoints.

## Analytics API

Analytics API provides detailed click statistics, including click trends, top links, referrer stats, and geographic distribution.

> You need to enable `analytics.enable_detailed_logging` in runtime config (restart required; restart the server after changing it) to record detailed click logs.
>
> - Default range: last 30 days. To set a custom range, provide **both** `start_date` and `end_date` (if only one is provided, it falls back to the default range; if parsing fails, it falls back to the default start/end values).
> - Date formats: RFC3339 (e.g. `2024-01-01T00:00:00Z`) or `YYYY-MM-DD` (e.g. `2024-01-01`; note: `YYYY-MM-DD` is interpreted as `00:00:00Z` of that day).
> - In the current implementation, GeoIP lookup is not yet wired into the click-write path (`click_logs.country/city` are null by default), so `/analytics/geo` and per-link `geo_distribution` may be empty unless historical data already contains geo fields.
> - Startup `[analytics]` config (`analytics.maxminddb_path` / `analytics.geoip_api_url`) is retained for GeoIP provider selection; the external API provider has built-in caching (LRU 10,000, TTL 15 minutes, negative caching, singleflight, 2-second timeout).
> - Device/browser distribution (`/analytics/devices`) is based on `user_agent_hash` (User-Agent strings are deduplicated into `user_agents` and linked by hash).
> - `click_logs.source` derivation is: `utm_source` first; otherwise `ref:{domain}` (from `Referer`); otherwise `direct`.

## Common Query Parameters (Most Endpoints)

| Param | Type | Description |
|-------|------|-------------|
| `start_date` | RFC3339 / YYYY-MM-DD | Start date (optional; only effective when provided together with `end_date`; default = last 30 days) |
| `end_date` | RFC3339 / YYYY-MM-DD | End date (optional; only effective when provided together with `start_date`; default = last 30 days) |
| `limit` | Integer | Result size (optional; defaults and max values vary by endpoint) |

> Endpoint-specific defaults: `top/referrers=10`, `geo=20`, `devices=10`, `links/{code}/analytics/devices=10`. `export` currently ignores `limit`.

### GET /analytics/trends - Get click trends

```bash
curl -sS -b cookies.txt \
  "http://localhost:8080/admin/v1/analytics/trends?start_date=2024-01-01T00:00:00Z&end_date=2024-12-31T23:59:59Z&group_by=day"
```

**Query params**:

- Supports common params: `start_date`, `end_date`
- Endpoint-specific param: `group_by` (optional; default `day`): `hour` / `day` / `week` / `month`

**Response**:
```json
{
  "code": 0,
  "message": "OK",
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

- Supports common params: `start_date`, `end_date`, `limit`
- `limit` default `10`, max `100`

**Response**:
```json
{
  "code": 0,
  "message": "OK",
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

- Supports common params: `start_date`, `end_date`, `limit`
- `limit` default `10`, max `100`

**Response**:
```json
{
  "code": 0,
  "message": "OK",
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

- Supports common params: `start_date`, `end_date`, `limit`
- `limit` default `20`, max `100`

**Response**:
```json
{
  "code": 0,
  "message": "OK",
  "data": []
}
```

### GET /analytics/devices - Get device analytics

```bash
curl -sS -b cookies.txt \
  "http://localhost:8080/admin/v1/analytics/devices?limit=10"
```

**Query params**:

- Supports common params: `start_date`, `end_date`, `limit`
- `limit` default `10`, max `20`

**Response**:
```json
{
  "code": 0,
  "message": "OK",
  "data": {
    "browsers": [
      {"name": "Chrome", "count": 120, "percentage": 60.0}
    ],
    "operating_systems": [
      {"name": "Mac OS X", "count": 80, "percentage": 40.0}
    ],
    "devices": [
      {"name": "pc", "count": 150, "percentage": 75.0}
    ],
    "bot_percentage": 12.3,
    "total_with_ua": 200
  }
}
```

### GET /links/{code}/analytics - Get single link analytics

```bash
curl -sS -b cookies.txt \
  "http://localhost:8080/admin/v1/links/github/analytics"
```

**Query params**:

- Supports common params: `start_date`, `end_date`

**Response**:
```json
{
  "code": 0,
  "message": "OK",
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
    "geo_distribution": []
  }
}
```

### GET /links/{code}/analytics/devices - Get single link device analytics

```bash
curl -sS -b cookies.txt \
  "http://localhost:8080/admin/v1/links/github/analytics/devices?limit=10"
```

**Query params**:

- Supports common params: `start_date`, `end_date`, `limit`
- `limit` default `10`, max `20`

**Response**:
```json
{
  "code": 0,
  "message": "OK",
  "data": {
    "browsers": [
      {"name": "Chrome", "count": 80, "percentage": 53.3}
    ],
    "operating_systems": [
      {"name": "Windows", "count": 60, "percentage": 40.0}
    ],
    "devices": [
      {"name": "pc", "count": 120, "percentage": 80.0}
    ],
    "bot_percentage": 8.5,
    "total_with_ua": 150
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

- Supports common params: `start_date`, `end_date`, `limit`
- `limit` is currently ignored; export includes all records within the date range (narrow date range to control output size)

The exported CSV contains these columns:
`short_code,clicked_at,referrer,source,ip_address,country,city`

The `Content-Disposition` filename format is: `analytics_YYYYMMDD_YYYYMMDD.csv` (derived from the parsed date range).

Note:
- The `source` column follows the redirect source derivation rule (`utm_source` → `ref:{domain}` → `direct`).
- Export does not include raw `user_agent`; device analytics is built via `user_agent_hash` + `user_agents`.

### Analytics configuration

These runtime config options control Analytics behavior:

| Config key | Type | Default | Description |
|------------|------|---------|-------------|
| `analytics.enable_detailed_logging` | bool | false | Enable detailed logging (writes to click_logs table; restart required) |
| `analytics.enable_auto_rollup` | bool | true | Enable automatic data retention task (restart required; runs every 4 hours by default) |
| `analytics.log_retention_days` | int | 30 | Raw click log retention in days (cleaned by background task; requires `analytics.enable_auto_rollup`) |
| `analytics.hourly_retention_days` | int | 7 | Hourly rollup retention in days (cleans `click_stats_hourly` / `click_stats_global_hourly`; requires `analytics.enable_auto_rollup`) |
| `analytics.daily_retention_days` | int | 365 | Daily rollup retention in days (cleans `click_stats_daily`; requires `analytics.enable_auto_rollup`) |
| `analytics.enable_ip_logging` | bool | true | Whether to record IP addresses |
| `analytics.enable_geo_lookup` | bool | false | Reserved GeoIP switch (currently not consumed in click-write path; `country/city` remain null by default) |
| `analytics.sample_rate` | float | 1.0 | Detailed logging sample rate (0.0-1.0; 1.0 = log all clicks) |
| `analytics.max_log_rows` | int | 0 | Maximum rows in `click_logs` (0 = unlimited) |
| `analytics.max_rows_action` | enum | cleanup | Action when max rows exceeded: `cleanup` (delete oldest) / `stop` (stop detailed logging) |
| `utm.enable_passthrough` | bool | false | Forward UTM params during redirect (`utm_source`/`utm_medium`/`utm_campaign`/`utm_term`/`utm_content`) |

Note: in the current implementation, retention parameters are read when the background task starts; after changing retention days, you may need to restart the server for the cleanup task to pick up new values.
