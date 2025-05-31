# CLI Tools

Shortlinker provides powerful command-line tools for managing short links and controlling services.

## Overview

The CLI tools support two modes:
- **Service Mode**: Start HTTP server for redirect services
- **Management Mode**: Manage short links through commands

## Basic Usage

```bash
# Start server (no arguments)
./shortlinker

# Management commands
./shortlinker <command> [options]
```

## Available Commands

### Service Management
- `start` - Start the server
- `stop` - Stop the server  
- `restart` - Restart the server

### Link Management
- `add` - Add a new short link
- `remove` - Delete a short link
- `list` - List all short links

### Help
- `help` - Show help information

## Quick Examples

```bash
# Add short link
./shortlinker add github https://github.com

# Add with expiration
./shortlinker add temp https://example.com --expire 2024-12-31T23:59:59Z

# List all links
./shortlinker list

# Delete link
./shortlinker remove github

# Get help
./shortlinker help
```

## Features

### 🎨 Colorful Output
- Success messages in green
- Error messages in red
- Warning messages in yellow
- Information in blue

### 🔒 Safe Operations
- Confirmation prompts for destructive operations
- Conflict detection for duplicate short codes
- Input validation for URLs and time formats

### 🌐 Cross Platform
- Consistent behavior across Windows, Linux, macOS
- Platform-specific optimizations
- Proper signal handling

### 📝 Rich Feedback
- Detailed success/error messages
- Progress indicators for long operations
- Helpful suggestions for common issues

## Environment Variables

CLI tools read the following environment variables:

```bash
# Random code length
RANDOM_CODE_LENGTH=6

# Storage configuration (v0.1.0+)
STORAGE_BACKEND=sqlite
DB_FILE_NAME=links.db

# Log level
RUST_LOG=info
```

## Exit Codes

| Code | Description |
|------|-------------|
| 0 | Success |
| 1 | General error |
| 2 | Invalid arguments |
| 3 | File operation error |
| 4 | Short code conflict |
| 5 | Short code not found |

## Next Steps

- 📖 [Command Reference](/en/cli/commands) - Detailed command documentation
- ⚙️ [Configuration](/en/config/) - Environment variable settings
- 🚀 [Deployment](/en/deployment/) - Production deployment guide
