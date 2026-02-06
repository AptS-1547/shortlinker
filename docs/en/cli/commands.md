# CLI Command Reference

Detailed command-line usage and options for day-to-day management.

## Task-Oriented Navigation

- **First-time usage**: `add` → `list` → `update` → `remove`
- **Bulk migration**: `import` / `export`
- **Operations**: `config` / `reset-password` / `generate-config`
- **Interactive management**: `tui`

> If you prefer visual management, start with the [TUI guide](/en/cli/tui).

## Core Commands (Recommended Order)

### add - Add Short Link

```bash
./shortlinker add <short_code> <target_url> [options]
./shortlinker add <target_url> [options]  # random short code
```

> Note: short codes must satisfy constraints (length ≤ 128, allowed chars `[a-zA-Z0-9_.-/]`) and must not conflict with reserved route prefixes (default `admin`/`health`/`panel`, from `routes.*_prefix`).

**Options**:
- `--force`: force overwrite existing short code
- `--expire <time>`: set expiration time
- `--password <password>`: set password protection (experimental)

**Examples**:
```bash
./shortlinker add google https://www.google.com
./shortlinker add https://www.example.com
./shortlinker add daily https://example.com --expire 1d
./shortlinker add google https://www.google.com --force
./shortlinker add secret https://example.com --password mypass
```

### list - List Short Links

```bash
./shortlinker list
```

### update - Update Short Link

```bash
./shortlinker update <short_code> <new_target_url> [options]
```

**Options**:
- `--expire <time>`: set new expiration time
- `--password <password>`: set or update password

**Examples**:
```bash
./shortlinker update github https://new-github.com
./shortlinker update github https://new-github.com --expire 30d
./shortlinker update github https://new-github.com --password secret123
```

### remove - Delete Short Link

```bash
./shortlinker remove <short_code>
```

### import - Import Short Links

```bash
./shortlinker import <file_path> [options]
```

**Options**:
- `--force`: force overwrite existing short codes

**Examples**:
```bash
./shortlinker import backup.csv
./shortlinker import backup.csv --force
```

> CSV is the default format. `.json` is kept only for legacy compatibility (planned removal in v0.5.0).

### export - Export Short Links

```bash
./shortlinker export [file_path]
```

**Examples**:
```bash
./shortlinker export
./shortlinker export backup.csv
```

### help - Show Command Help

```bash
./shortlinker help
```

## Operations Commands

### config - Runtime Config Management (DB)

The `config` subcommand manages runtime config stored in the database (same config system used by the web admin panel).

> Note: `config` writes values into the database. To make a **running** server reload runtime config, call Admin API `POST /admin/v1/config/reload` or restart the service.  
> Keys marked as “requires restart” (e.g. `routes.*`, `click.*`, `cors.*`) will not hot-apply even after reload.

Common subcommands:

```bash
# List configs (optional --category: auth/cookie/features/routes/cors/tracking)
./shortlinker config list
./shortlinker config list --category routes

# Get one config (use --json for structured output)
./shortlinker config get features.random_code_length
./shortlinker config get api.cookie_same_site --json

# Set/reset
./shortlinker config set features.random_code_length 8
./shortlinker config reset features.random_code_length

# Export/import (JSON)
./shortlinker config export config-backup.json
./shortlinker config import config-backup.json
./shortlinker config import config-backup.json --force
```

> Security note: exported config files contain real sensitive values (e.g. `api.admin_token`, `api.jwt_secret`, `api.health_token`). Store them securely.

### reset-password - Reset Admin Password

```bash
./shortlinker reset-password [options]
```

Resets the admin API password. The new password is hashed with Argon2id before being stored.

**Requirement**: password length must be at least 8 characters.

**Examples**:
```bash
# Interactive (recommended)
./shortlinker reset-password

# From stdin (scripting)
echo "my_new_secure_password" | ./shortlinker reset-password --stdin

# From CLI arg (not recommended: visible in shell history)
./shortlinker reset-password --password "my_new_secure_password"
```

### generate-config - Generate Configuration File

```bash
./shortlinker generate-config [output_path]
```

Generates a **startup config** (`config.toml`) template including `server` / `database` / `cache` / `logging` / `analytics`.  
Runtime config (e.g. `features.*`, `api.*`, `routes.*`, `cors.*`) is stored in DB and not part of this file.

**Examples**:
```bash
./shortlinker generate-config                 # generate config.example.toml
./shortlinker generate-config config.toml     # generate/overwrite config.toml
./shortlinker generate-config myconfig.toml   # custom filename
```

## Interactive Interface

### tui - Launch Terminal UI

```bash
./shortlinker tui
```

**TUI features**:
- interactive visual interface
- real-time link list view
- keyboard-based navigation and actions
- link details (clicks, expiration, etc.)

**Keyboard shortcuts**:
- `↑/↓` or `j/k`: move selection
- `Enter` or `v`: view details
- `/`: search
- `?` (or `h`): help
- `x`: export/import
- `q`: quit (`Esc` is commonly used for back/cancel/clear)

> For full details, see the [TUI guide](/en/cli/tui).

## Advanced and Automation

### Expiration Time Formats

```bash
1h      # 1 hour
1d      # 1 day
1w      # 1 week
1M      # 1 month
1y      # 1 year
1d2h30m # combined format
2024-12-31T23:59:59Z  # RFC3339
```

### Import/Export Formats

**CSV (default)**

Export includes header fields:
`code,target,created_at,expires_at,password,click_count`

```csv
code,target,created_at,expires_at,password,click_count
github,https://github.com,2024-12-15T14:30:22Z,,,
```

**JSON (legacy, deprecated)**

> `.json` is for legacy compatibility only (planned removal in v0.5.0).

```json
[
  {
    "code": "github",
    "target": "https://github.com",
    "created_at": "2024-12-15T14:30:22Z",
    "expires_at": null,
    "password": null,
    "click": 0
  }
]
```

### Reload Behavior

Link-data commands (`add` / `update` / `remove` / `import`) notify the running service to refresh in-memory link caches.

> This is different from runtime config reload. For config changes made via `./shortlinker config set`, call Admin API `POST /admin/v1/config/reload` or restart.

### Database Configuration

CLI reads `config.toml` in the current working directory. To use a different DB:

```toml
[database]
database_url = "sqlite://links.db"
```

> See [Configuration Guide](/en/config/).

### Batch Scripts

```bash
# Backup script
./shortlinker export "backup_$(date +%Y%m%d).csv"

# Batch import
while IFS=',' read -r code url; do
    ./shortlinker add "$code" "$url"
done < links.csv
```
