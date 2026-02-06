# CLI Command Reference

Detailed command-line usage and options for day-to-day management.

## Task-Oriented Navigation

- **First-time usage**: `add` → `list` → `update` → `remove`
- **Bulk migration**: `import` / `export`
- **Operations**: `config` / `reset-password`
- **Interactive management**: `tui`

> If you prefer visual management, start with the [TUI guide](/en/cli/tui).

## Global Options

All CLI subcommands support:

- `-s, --socket <path>`: override IPC socket path (Unix) or named pipe path (Windows)

> Priority: CLI `--socket` > `ipc.socket_path` in `config.toml` > platform default.

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

### status - Show Server Status (IPC)

```bash
./shortlinker status
./shortlinker --socket /tmp/custom.sock status
```

When reachable, it shows version, uptime, reload-in-progress status, last data/config reload time, and total link count.
If IPC is unreachable (server not running, `ipc.enabled=false`, socket path mismatch, etc.), it reports "Server is not running".

## Operations Commands

### config - Configuration Management

The `config` subcommand manages Shortlinker configuration.

#### config generate - Generate Configuration File

```bash
./shortlinker config generate [output_path] [options]
```

Generates a **startup config** (`config.toml`) template including `server` / `database` / `cache` / `logging` / `analytics`.
Runtime config (e.g. `features.*`, `api.*`, `routes.*`, `cors.*`) is stored in DB and not part of this file.

> Note: This command does not require a database connection and can be used during initial deployment.

**Options**:
- `--force`: skip confirmation and force overwrite existing file

**Examples**:
```bash
./shortlinker config generate                       # generate config.example.toml
./shortlinker config generate config.toml           # prompts for confirmation if file exists
./shortlinker config generate config.toml --force   # force overwrite
```

#### config list/get/set/reset - Runtime Config Management (DB)

The following subcommands manage runtime config stored in the database (same config system used by the web admin panel).

> Note: `config set/reset/import` automatically **attempt** an IPC `Config` reload after writing to DB (hot-apply for keys marked as "no restart").
> If IPC is unreachable (server not running, `ipc.enabled=false`, socket mismatch, etc.), trigger Admin API `POST /admin/v1/config/reload` manually or restart the service.
> Keys marked as "requires restart" (e.g. `routes.*`, `click.*`, `cors.*`) will not hot-apply even after reload.

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

When the server is running and IPC is reachable, link-management commands execute through IPC in the server process to keep storage/cache state aligned.

If IPC is unreachable, CLI falls back to direct DB operations (good for offline maintenance). If an online server is still running, you should manually refresh data (typically by restarting the service).

> Runtime config changes are a separate path. `config set/reset/import` only attempt `Config` reload; keys marked "requires restart" still require restart.

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
