# CLI Command Reference

Detailed command line tool usage instructions and parameter options.

## Basic Commands

### add - Add Short Link

```bash
# Custom short code
./shortlinker add <short_code> <target_url> [options]

# Random short code
./shortlinker add <target_url> [options]
```

**Options**:
- `--force`: Force overwrite existing short code
- `--expire <time>`: Set expiration time
- `--password <password>`: Set password protection (experimental)

**Examples**:
```bash
# Basic usage
./shortlinker add google https://www.google.com

# Random short code
./shortlinker add https://www.example.com

# Set expiration time
./shortlinker add daily https://example.com --expire 1d
./shortlinker add sale https://shop.com --expire 2w3d

# Force overwrite
./shortlinker add google https://www.google.com --force

# Password protected link
./shortlinker add secret https://example.com --password mypass
```

### export - Export Short Links

```bash
./shortlinker export [file_path]
```

**Examples**:
```bash
# Default filename
./shortlinker export

# Specify filename
./shortlinker export backup.json
```

### import - Import Short Links

```bash
./shortlinker import <file_path> [options]
```

**Options**:
- `--force`: Force overwrite existing short codes

**Examples**:
```bash
# Import with default options
./shortlinker import backup.json

# Force overwrite existing codes
./shortlinker import backup.json --force
```

### remove - Delete Short Link

```bash
./shortlinker remove <short_code>
```

### list - List Short Links

```bash
./shortlinker list
```

### help - Show command help

```bash
./shortlinker help
```

### generate-config - Generate Configuration File

```bash
./shortlinker generate-config [output_path]
```

Generate a default configuration file template with all configurable options.

**Examples**:
```bash
./shortlinker generate-config           # Generate config.toml
./shortlinker generate-config myconfig.toml  # Specify filename
```

### reset-password - Reset Admin Password

```bash
./shortlinker reset-password <new_password>
```

Reset the admin API password. The new password will be hashed with Argon2id and stored in the database.

**Requirement**: Password must be at least 8 characters long.

**Examples**:
```bash
./shortlinker reset-password "my_new_secure_password"
```

### config - Runtime config management (DB)

The `config` subcommand manages runtime configuration values stored in the database (the same config system used by the web admin panel).

> Note: `config` writes values into the database. To make a **running** server reload configs from the database, call Admin API `POST /admin/v1/config/reload` or restart the service.  
> Keys marked as “requires restart” (e.g. route prefixes, cookie settings) may not take full effect even after reload; a restart is still recommended.

Common subcommands:

```bash
# List configs (optional --category: auth/cookie/features/routes/cors/tracking)
./shortlinker config list
./shortlinker config list --category routes

# Get a config (use --json for structured output)
./shortlinker config get features.random_code_length
./shortlinker config get api.cookie_same_site --json

# Set/reset
./shortlinker config set features.random_code_length 8
./shortlinker config reset features.random_code_length

# Export/import (JSON)
./shortlinker config export config-backup.json
./shortlinker config import config-backup.json
./shortlinker config import config-backup.json --force   # skip interactive confirmation
```

> Security note: exported config files contain real sensitive values (e.g. `api.admin_token`, `api.jwt_secret`, `api.health_token`). Store them securely.

### tui - Launch Terminal User Interface

```bash
./shortlinker tui
```

**TUI Mode Features**:
- Interactive visual interface
- Real-time view of all short links
- Keyboard navigation and operations
- Display link details (click count, expiration time, etc.)

**Keyboard Shortcuts**:
- `↑/↓` or `j/k`: Move selection up/down
- `Enter` or `v`: View details
- `/`: Search
- `?` (or `h`): Help
- `x`: Export / Import
- `q`: Quit (`Esc` is commonly used for back/cancel/clear search)

> 💡 **Tip**: TUI mode is ideal for quick browsing and link management. For detailed usage, see [TUI User Guide](/en/cli/tui)

**Output Format**:
```bash
Short links list:

  google -> https://www.google.com
  github -> https://github.com
  temp -> https://example.com (expires: 2024-12-31 23:59:59 UTC)

ℹ Total 3 short links
```

### update - Update Short Link

```bash
./shortlinker update <short_code> <new_target_url> [options]
```

**Examples**:
```bash
# Update target URL
./shortlinker update github https://new-github.com

# Update URL and expiration time
./shortlinker update github https://new-github.com --expire 30d
```

## Expiration Time Formats

### Simple Format (Recommended)

```bash
1h    # 1 hour
1d    # 1 day
1w    # 1 week
1M    # 1 month
1y    # 1 year
```

### Combined Format

```bash
1d2h30m     # 1 day 2 hours 30 minutes
2w3d        # 2 weeks 3 days
1h30m15s    # 1 hour 30 minutes 15 seconds
```

### RFC3339 Format (Compatible)

```bash
2024-12-31T23:59:59Z
2024-12-31T23:59:59+08:00
```

> 💡 **Tip**: For more advanced time format options and detailed explanations, check the "Advanced Usage" section in the project documentation

## Common Time Examples

```bash
# Short-term links
./shortlinker add flash https://example.com --expire 1h      # 1 hour
./shortlinker add daily https://example.com --expire 1d     # 1 day

# Medium to long-term links  
./shortlinker add weekly https://example.com --expire 1w    # 1 week
./shortlinker add monthly https://example.com --expire 1M   # 1 month

# Precise time
./shortlinker add meeting https://zoom.us/j/123 --expire 2h30m
./shortlinker add sale https://shop.com --expire 2w3d
```

## Hot Reload Mechanism

After link-management operations (add/update/remove/import), CLI notifies the running server to reload short link data and rebuild in-memory caches:

```bash
# Unix/Linux systems - automatically send SIGUSR1 signal
./shortlinker add new https://example.com
# Output: ✓ Added short link: new -> https://example.com
#        ℹ Server reload notification sent

# Windows systems - automatically create trigger file
./shortlinker add new https://example.com
```

> Note: this reload mechanism is about **link data / caches**, not runtime config. For runtime config changes done outside the server process (e.g. via `./shortlinker config set`), call Admin API `POST /admin/v1/config/reload` or restart the service.

## Exit Codes

| Exit Code | Meaning |
|----------|---------|
| 0 | Success |
| 1 | Failed (validation/storage/command error) |

## Environment Variables

Main environment variables read by CLI tool:

```bash
DATABASE_URL=sqlite://links.db  # Database connection URL
RUST_LOG=info                   # Log level
```

> For complete environment variable configuration, see [Environment Variables Configuration](/en/config/)

## Script Integration

### Batch Operations
```bash
#!/bin/bash
# Batch import links
while IFS=',' read -r code url; do
    ./shortlinker add "$code" "$url"
done < links.csv
```

### Error Checking
```bash
if ./shortlinker add test https://example.com --expire 1d; then
    echo "Added successfully"
else
    echo "Failed to add"
    exit 1
fi
```

## Process Management

### Check Service Status
```bash
# Unix systems
if [ -f shortlinker.pid ]; then
    echo "Server PID: $(cat shortlinker.pid)"
else
    echo "Server not running"
fi
```

### Container Environment
In Docker containers, process management automatically handles container restarts without manual intervention.
