# CLI Command Reference

Detailed command-line tool usage instructions and parameter options.

## add - Add Short Link

Add a new short link with support for custom short codes or random generation.

### Syntax

```bash
# Custom short code
./shortlinker add <short_code> <target_url> [options]

# Random short code
./shortlinker add <target_url> [options]
```

### Parameters

- `<short_code>` (optional): Custom short link code
- `<target_url>` (required): Target URL address

### Options

- `--force`: Force overwrite existing short code
- `--expire <time>`: Set expiration time (multiple formats supported)

### Examples

```bash
# Basic usage
./shortlinker add google https://www.google.com

# Random short code
./shortlinker add https://www.example.com

# Using relative time format (recommended)
./shortlinker add daily https://example.com --expire 1d
./shortlinker add weekly https://example.com --expire 1w
./shortlinker add monthly https://example.com --expire 1M
./shortlinker add yearly https://example.com --expire 1y

# Combined time format
./shortlinker add complex https://example.com --expire 1d2h30m
./shortlinker add sale https://shop.com --expire 2w3d

# Using RFC3339 format (traditional)
./shortlinker add temp https://example.com --expire 2024-12-31T23:59:59Z

# Force overwrite
./shortlinker add google https://www.google.com --force
```

### Output

```bash
# Success
✓ Added short link: google -> https://www.google.com

# Random code success
✓ Added short link: aB3dF1 -> https://www.example.com

# Already exists error
❌ Error: Short code 'google' already exists, currently points to: https://www.google.com
Use --force parameter to overwrite
```

## remove - Delete Short Link

Delete the specified short link.

### Syntax

```bash
./shortlinker remove <short_code>
```

### Parameters

- `<short_code>` (required): Short link code to delete

### Examples

```bash
# Delete short link
./shortlinker remove google

# Delete randomly generated short code
./shortlinker remove aB3dF1
```

## list - List Short Links

Display all created short links.

### Syntax

```bash
./shortlinker list
```

### Output Format

```bash
Short Link List:

  google -> https://www.google.com
  github -> https://github.com
  temp -> https://example.com (expires: 2024-12-31 23:59:59 UTC)
  aB3dF1 -> https://random-example.com

ℹ Total 4 short links
```

## update - Update Short Link

Update the target URL and expiration time of an existing short link.

### Syntax

```bash
./shortlinker update <short_code> <new_target_url> [options]
```

### Options

- `--expire <time>`: Update expiration time (multiple formats supported)

### Examples

```bash
# Update target URL
./shortlinker update github https://new-github.com

# Update URL and expiration (relative time format)
./shortlinker update github https://new-github.com --expire 30d

# Using combined time format
./shortlinker update temp https://example.com --expire 1w2d12h
```

## Time Format

### Relative Time Format (Recommended)

Supports concise relative time format, calculated from current time:

#### Single Time Units
```bash
1s   # Expires in 1 second
5m   # Expires in 5 minutes
2h   # Expires in 2 hours
1d   # Expires in 1 day
1w   # Expires in 1 week
1M   # Expires in 1 month (30 days)
1y   # Expires in 1 year (365 days)
```

#### Combined Time Format
```bash
1d2h30m     # Expires in 1 day, 2 hours, 30 minutes
2w3d        # Expires in 2 weeks, 3 days
1y30d       # Expires in 1 year, 30 days
1h30m15s    # Expires in 1 hour, 30 minutes, 15 seconds
```

#### Supported Time Units
| Unit | Full Forms | Description |
|------|------------|-------------|
| `s` | `sec`, `second`, `seconds` | Seconds |
| `m` | `min`, `minute`, `minutes` | Minutes |
| `h` | `hour`, `hours` | Hours |
| `d` | `day`, `days` | Days |
| `w` | `week`, `weeks` | Weeks |
| `M` | `month`, `months` | Months (30 days) |
| `y` | `year`, `years` | Years (365 days) |

### RFC3339 Format (Compatible)

Still supports traditional RFC3339 format:

```bash
# Complete format
2024-12-31T23:59:59Z

# With timezone
2024-12-31T23:59:59+08:00
```

### Common Time Examples

```bash
# Short-term links
./shortlinker add flash https://example.com --expire 1h      # 1 hour
./shortlinker add daily https://example.com --expire 1d     # 1 day

# Medium-term links  
./shortlinker add weekly https://example.com --expire 1w    # 1 week
./shortlinker add monthly https://example.com --expire 1M   # 1 month

# Long-term links
./shortlinker add yearly https://example.com --expire 1y    # 1 year

# Precise timing
./shortlinker add meeting https://zoom.us/j/123 --expire 2h30m  # 2 hours 30 minutes
./shortlinker add sale https://shop.com --expire 2w3d          # 2 weeks 3 days
```

## Error Codes

| Error Code | Description |
|------------|-------------|
| 0 | Success |
| 1 | General error |
| 2 | Parameter error |
| 3 | File operation error |
| 4 | Short code conflict |
| 5 | Short code not found |

## Environment Variables

CLI tools read the following environment variables:

```bash
# Random short code length
RANDOM_CODE_LENGTH=6

# Storage configuration
STORAGE_BACKEND=sqlite
DB_FILE_NAME=links.db

# Other configuration
RUST_LOG=info
```

## Output Colors

CLI supports colored output, controllable through environment variables:

```bash
# Disable color output
NO_COLOR=1 ./shortlinker list

# Force enable color (even in non-TTY environments)
FORCE_COLOR=1 ./shortlinker list
```

## Script-Friendly Mode

### Return Code Checking

```bash
#!/bin/bash
# Check if command succeeded
if ./shortlinker add test https://example.com --expire 1d; then
    echo "Add successful"
else
    echo "Add failed"
    exit 1
fi
```
