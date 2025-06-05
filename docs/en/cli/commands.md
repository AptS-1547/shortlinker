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

### start - Start server in background

```bash
./shortlinker start
```

### stop - Stop running server

```bash
./shortlinker stop
```

### restart - Restart server

```bash
./shortlinker restart
```

### help - Show command help

```bash
./shortlinker help
```

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

CLI operations automatically notify the server to reload configuration:

```bash
# Unix/Linux systems - automatically send SIGUSR1 signal
./shortlinker add new https://example.com
# Output: ✓ Added short link: new -> https://example.com
#        ℹ Server reload notification sent

# Windows systems - automatically create trigger file
./shortlinker add new https://example.com
```

## Error Codes

| Error Code | Description |
|------------|-------------|
| 0 | Success |
| 1 | General error |
| 2 | Parameter error |
| 4 | Short code conflict |
| 5 | Short code not found |

## Environment Variables

Main environment variables read by CLI tool:

```bash
RANDOM_CODE_LENGTH=6      # Random short code length
STORAGE_BACKEND=sqlite    # Storage backend type
DB_FILE_NAME=links.db     # Database file path
RUST_LOG=info            # Log level
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
