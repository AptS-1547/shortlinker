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
  q:Quit  o:System  x:Export/Import  /:Search  ↑↓:Navigate  Enter:Details
```

## Keyboard Shortcuts

### Main Interface

| Shortcut | Function |
|----------|----------|
| `↑` / `↓` | Move selection up/down |
| `j` / `k` | Move down/up (Vim style) |
| `PageUp` / `PageDown` | Fast move within current page (10 rows) |
| `[` / `]` | Previous/next page (database pagination) |
| `Home` / `g` | Jump to top |
| `End` / `G` | Jump to bottom |
| `s` | Cycle sort column (code / URL / clicks / status) |
| `S` | Toggle sort direction (asc / desc) |
| `Space` | Toggle select current link (for batch delete) |
| `Esc` | Clear search state or clear batch selection |
| `/` | Search (fuzzy match on code / target URL) |
| `Enter` / `v` | View details |
| `?` / `h` | Help |
| `a` | Add new short link |
| `e` | Edit selected short link |
| `d` | Delete current link (or open batch delete when selected items exist) |
| `x` | Export/Import menu |
| `o` | System operations (status, config, password reset) |
| `y` | Copy selected short code to clipboard |
| `Y` | Copy selected target URL to clipboard |
| `q` | Quit (with confirmation) |

### Add/Edit Interface

| Shortcut | Function |
|----------|----------|
| `Tab` | Switch input fields |
| `Enter` | Save link |
| `Esc` | Cancel and return |
| `Space` | Toggle force overwrite (add only; focus must be on the short code field) |

## Features

### 1. Browse Link List

The main interface displays overview information for all short links:

- **Code**: Short link code (cyan)
- **Target URL**: Redirect destination (blue)
- **Expiration**: Shows expiration time or `(EXPIRED)` marker (yellow/red)
- **Password Protected**: Shows 🔒 icon (magenta)
- **Click Count**: Shows access count (green)
- **Paged Loading**: Loads 100 links per page by default; use `[` / `]` to switch database pages
- **Virtual Scrolling**: Renders only visible rows for smoother large-list navigation

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
- Press `Space` to toggle "Force Overwrite" (when focus is on the Short Code field)
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

### 5. Batch Selection and Batch Delete (Press `Space` + `d`)

- Press `Space` in the main screen to select/unselect current link
- After selecting multiple links, press `d` to open the batch delete confirmation popup
- In batch delete confirmation, press `y` to delete, `n` or `Esc` to cancel

### 6. Export/Import (Press `x`)

**Export Function**:
- Default filename is a timestamped name like `shortlinks_export_20250115_183000.csv` (editable)
- Exports all links in CSV format (with header)
- Useful for backup or migration

**Import Function**:
- Select a `.csv` file via the built-in file browser
- Choose import mode: **Skip existing** (default) or **Overwrite existing**
- Batch import links
- Compatible with CLI export format

### 7. System Operations (Press `o`)

Access system operations via the System menu:

**Server Status** (`s`):
- View server version, uptime, links count
- Check reload status and last reload timestamps
- Press `r` to refresh, `Esc` to go back
- Requires the server to be running (IPC connection)

**Runtime Configuration** (`c`):
- Browse all runtime configs grouped by category (Authentication, Cookie, Features, Routes, CORS, Tracking, Analytics, Cache)
- Navigate with `j`/`k` or arrow keys
- Press `e` or `Enter` to edit a config value (non-sensitive, editable configs only)
- Press `r` to reset a config to its default value
- Sensitive values are displayed as `[REDACTED]` and cannot be edited in TUI
- Tags indicate `sensitive`, `restart` (requires server restart), and `readonly` configs
- Works with or without the server running (IPC-first with direct DB fallback)

**Reset Admin Password** (`p`):
- Enter new password (min 8 characters) and confirm
- Password is hashed with Argon2id and stored in the database
- Uses direct DB access (works even when server is not running)
- Press `Tab` to switch between password and confirm fields

### 8. Auto Server Notification

After create/update/delete (and import), TUI notifies the server via IPC to run `ReloadTarget::Data`, refreshing short-link data and caches.

> If IPC is unreachable (server not running, `ipc.enabled=false`, socket mismatch, etc.), the notification is skipped and local TUI operations still complete.

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
4. **Batch Operations**: Supports batch import and batch delete
5. **Immediate Feedback**: Operations take effect immediately without waiting for API responses

## Limitations

Current version TUI known limitations:

- ⚠️ Batch operations currently support CSV import and batch delete only; no batch edit or batch create
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

# View error output
./shortlinker tui
```

### Data Not Refreshing

There is no manual refresh shortcut in the current version:

- Lists are refreshed automatically after create/update/delete/import
- If you are in a search state, press `Esc` to clear the filter
- If it still looks stale, exit and re-enter TUI

## Comparison with Other Tools

| Feature | TUI | CLI | Admin API | Web Panel |
|---------|-----|-----|-----------|-----------|
| View Links | ✅ | ✅ | ✅ | ✅ |
| Create Links | ✅ | ✅ | ✅ | ✅ |
| Edit Links | ✅ | ✅ | ✅ | ✅ |
| Delete Links | ✅ | ✅ | ✅ | ✅ |
| Export/Import | ✅ | ✅ | ✅ (CSV) | ✅ |
| Server Status | ✅ | ✅ | ✅ | ✅ |
| Runtime Config | ✅ | ✅ | ✅ | ✅ |
| Reset Password | ✅ | ✅ | ❌ | ❌ |
| Visual UI | ✅ | ❌ | ❌ | ✅ |
| Auth Required | ❌ | ❌ | ✅ | ✅ |
| Interactive | ✅ | ❌ | ❌ | ✅ |
| Batch Operations | ⚠️ Import + Batch delete | ✅ | ✅ | ✅ |
| Remote Access | ❌ | ❌ | ✅ | ✅ |

## Next Steps

- 📋 Learn [CLI Commands](/en/cli/commands) for link management operations
- 🛡️ Use [Admin API](/en/api/admin) for programmatic management
- 🎨 Try [Web Admin Panel](/en/admin-panel/) for graphical management
