# CLI Command Line Tool

Shortlinker provides an intuitive and easy-to-use command line tool for managing short links.

## Tool Features

- 🎨 **Colored Output** - Clear visual feedback
- 🔄 **Real-time Sync** - Commands take effect immediately  
- ⚡ **Fast Response** - Supports SQLite, file, Sled multiple storage backends
- 🛡️ **Error Handling** - Detailed error messages and suggestions
- 📦 **Import/Export** - JSON format backup and migration support

## Basic Syntax

```bash
./shortlinker <command> [arguments] [options]
```

## Command Overview

| Command | Function | Example |
|---------|----------|---------|
| `help` | Show help | `./shortlinker help` |
| `start` | Start server | `./shortlinker start` |
| `stop` | Stop server | `./shortlinker stop` |
| `restart` | Restart server | `./shortlinker restart` |
| `add` | Add short link | `./shortlinker add github https://github.com` |
| `remove` | Delete short link | `./shortlinker remove github` |
| `update` | Update short link | `./shortlinker update github https://new-url.com` |
| `list` | List all links | `./shortlinker list` |
| `export` | Export data | `./shortlinker export backup.json` |
| `import` | Import data | `./shortlinker import backup.json --force` |

## Quick Examples

### Basic Operations
```bash
# Add short link
./shortlinker add docs https://docs.example.com

# View all links
./shortlinker list

# Delete link
./shortlinker remove docs
```

### Data Management
```bash
# Export data
./shortlinker export backup.json

# Import data
./shortlinker import backup.json --force
```

### Advanced Features
```bash
# Random short code
./shortlinker add https://example.com
# Output: ✓ Added short link: aB3dF1 -> https://example.com

# Set expiration time
./shortlinker add sale https://shop.com/sale --expire 2024-12-25T00:00:00Z

# Force overwrite
./shortlinker add docs https://new-docs.com --force
```

## Output Description

### Success Status
- ✅ Green text indicates successful operation
- 🔵 Blue text shows informational messages

### Error Status  
- ❌ Red text shows error messages
- 💡 Provides solution suggestions

### Example Output
```bash
$ ./shortlinker add github https://github.com
✓ Added short link: github -> https://github.com

$ ./shortlinker add github https://gitlab.com
❌ Error: Short code 'github' already exists, currently points to: https://github.com
💡 To overwrite, use --force parameter
```

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
if ./shortlinker add test https://example.com; then
    echo "Added successfully"
else
    echo "Failed to add"
    exit 1
fi
```

## Next Steps

- 📖 Check [Detailed Command Reference](/en/cli/commands) for all options
- ⚙️ Learn [Configuration Guide](/en/config/) to customize behavior
