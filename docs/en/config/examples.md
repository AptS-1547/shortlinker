# Examples and Hot Reload

This page includes environment-specific config examples and hot reload behavior.

## Examples

### Development

```toml
# config.toml (dev)
[server]
host = "127.0.0.1"
port = 8080

[database]
database_url = "dev-links.db"

[logging]
level = "debug"
```

> Runtime config (e.g. `features.enable_admin_panel`, `api.health_token`) lives in the database and should be changed via Admin API or CLI; rotate `api.admin_token` via `./shortlinker reset-password`.

### Production

```toml
[server]
host = "127.0.0.1"
port = 8080
cpu_count = 8

[database]
database_url = "/data/shortlinks.db"
pool_size = 20

[cache]
type = "memory"
default_ttl = 7200

[cache.memory]
max_capacity = 50000

[logging]
level = "info"
format = "json"
file = "/var/log/shortlinker/app.log"
enable_rotation = true
```

### Docker

Mount a startup config file (make sure `server.host` is `0.0.0.0`):

```toml
# /config.toml (inside container)
[server]
host = "0.0.0.0"
port = 8080

[database]
database_url = "sqlite:///data/links.db"
```

Runtime config (stored in the DB) can be set via the built-in CLI; configs marked as “restart required” need a restart to take effect:

```bash
# Enable admin panel (restart required)
/shortlinker config set features.enable_admin_panel true

# Health Bearer token (no restart)
/shortlinker config set api.health_token "your_health_token"
```

## Hot Reload

Shortlinker has two kinds of hot reload / hot apply:

1. **Short-link data sync / hot reload**:
   - CLI link commands (`add/update/remove/import/export/list`) execute through IPC in the server process when IPC is reachable.
   - TUI writes to DB locally, then triggers `ReloadTarget::Data` via IPC to refresh caches.
2. **Runtime config hot apply**:
   - No-restart keys usually apply immediately when updated through Admin API.
   - CLI `config set/reset/import` writes to DB and then automatically attempts `ReloadTarget::Config` via IPC.

### Supported

- ✅ Short-link data (cache rebuild)
- ✅ Runtime configs marked as no-restart (Admin API or CLI+IPC path)
- ✅ Cookie settings (`api.cookie_*`): affect newly issued cookies; re-login to get updated cookies

### Not supported

- ❌ Server host/port
- ❌ Database connection
- ❌ Cache type
- ❌ Route prefixes

### Check and manual reload

```bash
# 1) Check IPC connectivity (CLI/TUI <-> server sync)
./shortlinker status
# If you use a custom socket/pipe path:
./shortlinker --socket /tmp/shortlinker.sock status

# 2) Manually reload runtime config from DB (Admin API; requires cookies)
# Use this when IPC is unreachable (server not running, ipc.enabled=false,
# socket mismatch, etc.) and CLI config commands cannot notify the running server.
curl -sS -X POST \
  -H "Content-Type: application/json" \
  -c cookies.txt \
  -d '{"password":"your_admin_token"}' \
  http://localhost:8080/admin/v1/auth/login

CSRF_TOKEN=$(awk '$6=="csrf_token"{print $7}' cookies.txt | tail -n 1)

curl -sS -X POST -b cookies.txt \
  -H "X-CSRF-Token: ${CSRF_TOKEN}" \
  http://localhost:8080/admin/v1/config/reload
```
