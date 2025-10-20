# TUI Terminal User Interface

Shortlinker provides an interactive Terminal User Interface (TUI) for visually managing short links in the command line.

## Launch TUI

```bash
./shortlinker tui
```

## Interface Overview

The TUI interface consists of several main areas:

```
┌─────────────────────────────────────────────────────────┐
│  Shortlinker - Terminal UI                             │
├─────────────────────────────────────────────────────────┤
│                                                         │
│  Code         Target URL                 Clicks  Status │
│  ────────────────────────────────────────────────────  │
│  github       https://github.com          142    Active │
│  google       https://google.com           89    Active │
│  temp         https://example.com           5    Expired│
│                                                         │
├─────────────────────────────────────────────────────────┤
│  Details Panel                                          │
│  Code: github                                          │
│  Target: https://github.com                            │
│  Created: 2024-12-01 10:30:22                          │
│  Expires: Never                                        │
│  Clicks: 142                                           │
│  Protected: No                                         │
└─────────────────────────────────────────────────────────┘
  q:Quit  r:Refresh  ↑↓:Navigate  Enter:Details
```

## Keyboard Shortcuts

### Main Interface

| Shortcut | Function |
|----------|----------|
| `↑` / `↓` | Move selection up/down |
| `a` | Add new short link |
| `e` | Edit selected short link |
| `d` | Delete selected short link |
| `x` / `i` | Export/Import links |
| `q` | Exit TUI |

### Add/Edit Interface

| Shortcut | Function |
|----------|----------|
| `Tab` | Switch input fields |
| `Enter` | Save link |
| `Esc` | Cancel and return |
| `f` | Toggle force overwrite (add only) |

## Features

### 1. Browse Link List

The main interface displays overview information for all short links:

- **Code**: Short link code (cyan)
- **Target URL**: Redirect destination (blue)
- **Expiration**: Shows expiration time or `(EXPIRED)` marker (yellow/red)
- **Password Protected**: Shows 🔒 icon (magenta)
- **Click Count**: Shows access count (green)

Selected item is highlighted with gray background.

### 2. Add New Link (Press `a`)

After entering add interface, you can configure:

- **Short Code**: Leave empty to auto-generate random code
- **Target URL**: Required, must start with `http://` or `https://`
- **Expire Time**: Optional, supports relative time (e.g., `1d`, `7d`) or RFC3339 format
- **Password**: Optional, password protection (experimental)
- **Force Overwrite**: Check to overwrite existing short codes

**Operations**:
- Press `Tab` to switch between fields
- Press `f` to toggle "Force Overwrite" option
- Press `Enter` to save
- Press `Esc` to cancel

### 3. Edit Existing Link (Press `e`)

Edit interface is similar to add, but short code field is read-only. You can modify:

- Target URL
- Expiration time
- Password (leave empty to keep current password)

**Note**: Creation time and click count are preserved.

### 4. Delete Link (Press `d`)

A confirmation dialog will appear showing link details. Press `y` to confirm deletion, `n` to cancel.

**Warning**: Delete operation cannot be undone!

### 5. Export/Import (Press `x` or `i`)

**Export Function**:
- Exports to `shortlinks_export.json` by default
- Exports all links in JSON format
- Useful for backup or migration

**Import Function**:
- Imports from `shortlinks_import.json` by default
- Supports batch import of links
- Compatible with CLI export format

### 6. Auto Server Notification

After all create/update/delete operations, TUI automatically notifies the server to reload configuration (via SIGUSR1 signal or trigger file).

## Color Scheme

TUI uses different colors to indicate link status:

- 🟢 **Green**: Active links
- 🔴 **Red**: Expired links
- 🔵 **Blue**: Protected links
- ⚪ **Gray**: Selection highlight

## Use Cases

TUI mode is suitable for:

1. **Visual Management**: Intuitive interface for quick CRUD operations
2. **No API Required**: No need to configure Admin API Token or start Web service
3. **Server Management**: Complete short link management in SSH terminals
4. **Batch Operations**: Export/import functions for easy migration and backup
5. **Immediate Feedback**: Operations take effect immediately without waiting for API responses

## Limitations

Current version TUI known limitations:

- ⚠️ Export/import paths cannot be customized (fixed to default filenames)
- ⚠️ No search/filter functionality (planned)
- ⚠️ No batch selection and batch operations (planned)
- ⚠️ Password input shown as masked, cannot see actual content

## Troubleshooting

### Display Issues

```bash
# Ensure terminal supports UTF-8
export LANG=en_US.UTF-8

# Adjust terminal window size (recommend at least 80x24)
```

### Cannot Start TUI

```bash
# Check if storage backend is accessible
./shortlinker list

# View error logs
RUST_LOG=debug ./shortlinker tui
```

### Data Not Refreshing

Press `r` to manually refresh, or exit and re-enter TUI.

## Comparison with Other Tools

| Feature | TUI | CLI | Admin API | Web Panel |
|---------|-----|-----|-----------|-----------|
| View Links | ✅ | ✅ | ✅ | ✅ |
| Create Links | ✅ | ✅ | ✅ | ✅ |
| Edit Links | ✅ | ✅ | ✅ | ✅ |
| Delete Links | ✅ | ✅ | ✅ | ✅ |
| Export/Import | ✅ | ✅ | ❌ | ✅ |
| Visual UI | ✅ | ❌ | ❌ | ✅ |
| Auth Required | ❌ | ❌ | ✅ | ✅ |
| Interactive | ✅ | ❌ | ❌ | ✅ |
| Batch Operations | ⚠️ Import | ✅ | ✅ | ✅ |
| Remote Access | ❌ | ❌ | ✅ | ✅ |

## Next Steps

- 📋 Learn [CLI Commands](/en/cli/commands) for link management operations
- 🛡️ Use [Admin API](/en/api/admin) for programmatic management
- 🎨 Try [Web Admin Panel](/en/admin-panel/) for graphical management
