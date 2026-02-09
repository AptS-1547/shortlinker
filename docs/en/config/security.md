# Security Best Practices

This page focuses on config-related security recommendations, including login rate-limit IP extraction and IPC socket permissions.

### Login Rate-Limit IP Extraction

Shortlinker uses smart proxy detection to balance security and usability when extracting client IPs for login rate limiting.

**Direct deployment** (no reverse proxy):
- No extra config needed; public source IPs do not trust `X-Forwarded-For` by default.

**Reverse proxy deployment** (Nginx/Caddy/Docker):
- **Auto-detect** (recommended): leave `api.trusted_proxies` empty. Connections from private/local addresses automatically trust `X-Forwarded-For` (IPv4: 10.x, 172.16-31.x, 192.168.x, 127.0.0.1; IPv6: `::1`, `fc00::/7`, `fe80::/10`).
- **Explicit config**: if you need strict control, set `api.trusted_proxies` to trusted proxy IPs/CIDRs in Admin config.

**Unix socket mode** (nginx on same machine):
- `X-Forwarded-For` is always used for client IP extraction.
- Ensure nginx sets `proxy_set_header X-Forwarded-For $remote_addr;`.

Optional config examples:

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
> - **Auto-detect mode** fits most setups, but if Shortlinker listens on an internal/private IP without a reverse proxy, prefer explicit `trusted_proxies` to avoid spoofing risk.
> - **Explicit mode** misconfiguration may cause users to share one rate-limit bucket (proxy not matched) or reintroduce bypass risk (over-trusting unsafe proxies).
> - Check startup logs for active mode: `Login rate limiting: Auto-detect mode enabled` or `Login rate limiting: Explicit trusted proxies configured`.

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
