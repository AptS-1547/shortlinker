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
- `--expire <time>`: Set expiration time (RFC3339 format)

### Examples

```bash
# Basic usage
./shortlinker add google https://www.google.com

# Random short code
./shortlinker add https://www.example.com

# Set expiration time
./shortlinker add temp https://example.com --expire 2024-12-31T23:59:59Z

# Force overwrite
./shortlinker add google https://www.google.com --force

# Complex example
./shortlinker add promo https://shop.com/sale --expire 2024-12-25T00:00:00Z
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

### Output

```bash
# Success
✓ Deleted short link: google

# Not found error
❌ Error: Short link does not exist: nonexistent
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

### Empty List

```bash
Short Link List:

ℹ No short links
```

## Time Format

### RFC3339 Format

Expiration time must use RFC3339 format:

```bash
# Complete format
2024-12-31T23:59:59Z

# With timezone
2024-12-31T23:59:59+08:00

# Other examples
2024-01-01T00:00:00Z        # New Year
2024-06-15T12:00:00Z        # Noon
2024-12-25T00:00:00-05:00   # Christmas (EST)
```

### Common Time Examples

```bash
# Expire in one day
./shortlinker add daily https://example.com --expire 2024-01-02T00:00:00Z

# Expire in one week
./shortlinker add weekly https://example.com --expire 2024-01-08T00:00:00Z

# Expire in one month
./shortlinker add monthly https://example.com --expire 2024-02-01T00:00:00Z

# Expire in one year
./shortlinker add yearly https://example.com --expire 2025-01-01T00:00:00Z
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

### Silent Mode

```bash
# Reduce output information (planned feature)
./shortlinker add google https://www.google.com --quiet

# Output results only
./shortlinker list --format=json
```

### Return Code Checking

```bash
#!/bin/bash
# Check if command succeeded
if ./shortlinker add test https://example.com; then
    echo "Add successful"
else
    echo "Add failed"
    exit 1
fi
```
