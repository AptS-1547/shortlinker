# Security Best Practices

This page focuses on config-related security recommendations, including login rate-limit IP extraction and IPC socket permissions.

### Login Rate-Limit IP Extraction

Shortlinker trusts `X-Forwarded-For` only on explicitly trusted proxy connections. The resolved client IP is shared by login rate limiting and detailed click analytics.

**Direct deployment** (no reverse proxy):
- No extra config is needed. Shortlinker uses the TCP peer IP and ignores client-supplied `X-Forwarded-For`.

**Reverse proxy deployment** (Nginx/Caddy/Docker):
- Configure `api.trusted_proxies` in Admin config with the proxy IPs or CIDRs that connect directly to Shortlinker.
- `X-Forwarded-For` is accepted only when the direct peer matches that list; otherwise rate limiting and analytics use the proxy peer IP.

**Unix socket mode** (nginx on same machine):
- Shortlinker automatically trusts the local Unix socket proxy transport.
- Ensure nginx sets `proxy_set_header X-Forwarded-For $remote_addr;`.
- Without that header, login still works, but requests fall back to the loopback IP and share one rate-limit bucket.

Optional config examples:

Restart Shortlinker after saving `api.trusted_proxies` so login rate limiting and detailed click analytics use the same proxy rules.

```bash
CSRF_TOKEN=$(awk '$6=="csrf_token"{print $7}' cookies.txt | tail -n 1)

# Local nginx
curl -X PUT -b cookies.txt \
     -H "X-CSRF-Token: ${CSRF_TOKEN}" \
     -H "Content-Type: application/json" \
     -d '{"value": "[\"127.0.0.1\"]"}' \
     http://localhost:8080/admin/v1/config/api.trusted_proxies

# Cloudflare CDN (example ranges)
curl -X PUT -b cookies.txt \
     -H "X-CSRF-Token: ${CSRF_TOKEN}" \
     -H "Content-Type: application/json" \
     -d '{"value": "[\"103.21.244.0/22\", \"103.22.200.0/22\"]"}' \
     http://localhost:8080/admin/v1/config/api.trusted_proxies
```

> Notes:
> - Do not trust an entire VPC merely because the proxy is internal; prefer the direct proxy addresses seen by Shortlinker.
> - Misconfiguration may cause users to share one rate-limit bucket (proxy not matched) or permit spoofing (trust range too broad).
> - Check startup logs for the active mode: `Client IP extraction: direct peer mode enabled` or `Client IP extraction: explicit trusted proxies configured`.

### IPC Socket Permissions

On Unix, IPC socket file permissions are set to `0600` (owner-only) by default for `./shortlinker.sock` (or your `ipc.socket_path` / CLI `--socket` override), preventing other local users from bypassing Admin API protection.

If you need to grant CLI access to specific users:

```bash
# Option 1: ACL
setfacl -m u:username:rw ./shortlinker.sock

# Option 2: group-based access
chgrp shortlinker-users ./shortlinker.sock
chmod 660 ./shortlinker.sock
```
