# Admin API: Runtime Config and Automation

This page covers runtime configuration endpoints, auth endpoint notes, and a Python automation example.

## Runtime config management

Config endpoints are under `/{ADMIN_ROUTE_PREFIX}/v1/config`. Responses use `{code,message,data}`; sensitive values are masked as `[REDACTED]`.

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
