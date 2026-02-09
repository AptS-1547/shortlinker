# Health Check API: Monitoring and Troubleshooting

This page focuses on probe strategy, Kubernetes examples, scripted checks, and security notes.

## Monitoring integration notes

If you use **Bearer token** (the value of runtime config `api.health_token`), you can avoid JWT cookie expiration and make automated monitoring easier.

Recommended options:

1. **Recommended: set `api.health_token` and probe `/health/live` or `/health/ready` with `Authorization: Bearer <token>`**
2. **Fallback: probe `/`** (returns `307`, treated as success in Kubernetes) to ensure the process is up
3. **Fallback: login + cookies + `/health`** (for monitors that already have a login step)

### Kubernetes probe example (Bearer token)

```yaml
apiVersion: v1
kind: Pod
spec:
  containers:
  - name: shortlinker
    image: e1saps/shortlinker
    livenessProbe:
      httpGet:
        path: /health/live
        port: 8080
        httpHeaders:
          - name: Authorization
            value: "Bearer your_health_token"
      initialDelaySeconds: 10
      periodSeconds: 10
```

### Kubernetes probe example (simple liveness)

```yaml
apiVersion: v1
kind: Pod
spec:
  containers:
  - name: shortlinker
    image: e1saps/shortlinker
    livenessProbe:
      httpGet:
        path: /
        port: 8080
      initialDelaySeconds: 10
      periodSeconds: 10
```

## Script example (login + health check)

```bash
#!/bin/bash
set -euo pipefail

ADMIN_TOKEN="your_admin_token"
BASE_URL="http://localhost:8080"
COOKIE_JAR="$(mktemp)"

curl -sS -X POST \
  -H "Content-Type: application/json" \
  -c "$COOKIE_JAR" \
  -d "{\"password\":\"${ADMIN_TOKEN}\"}" \
  "${BASE_URL}/admin/v1/auth/login" >/dev/null

curl -sS -b "$COOKIE_JAR" "${BASE_URL}/health"
```

## Security notes

1. Use a strong admin password (`api.admin_token`)
2. Restrict access to health endpoints to trusted networks
3. Use HTTPS in production and configure cookie security correctly
