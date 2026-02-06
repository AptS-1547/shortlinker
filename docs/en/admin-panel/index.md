# Web Admin Panel

Shortlinker includes a modern React 19 + TypeScript web admin panel in the `admin-panel` directory, powered by the Admin API.

## 3-Minute Start

1. Enable the panel via [Enabling the Panel](#enabling-the-panel)
2. Sign in with the admin password (`api.admin_token`)
3. Complete daily flow: create link → filter list → export/import

> For API-based automation, jump to [Admin API Documentation](/en/api/admin).

## Enabling the Panel

To enable the web admin panel in Shortlinker:

1. **Build frontend assets**:

   ```bash
   cd admin-panel
   bun install
   bun run build
   ```

2. **Enable settings (runtime config)**:

   ```bash
   # Enable admin panel (runtime config, stored in DB; restart required)
   ./shortlinker config set features.enable_admin_panel true

   # Optional: change frontend route prefix (restart required)
   ./shortlinker config set routes.frontend_prefix /panel
   ```

3. **Access the interface**:
   After starting Shortlinker, visit `http://your-domain:8080/panel`

> Notes:
> - The admin login password is the plaintext value of runtime config `api.admin_token`. On first startup, a random password is generated and written to `admin_token.txt` (if missing). You can rotate it with `./shortlinker reset-password`.
> - Route prefix configs like `routes.frontend_prefix` / `routes.admin_prefix` / `routes.health_prefix` require restart.

## Custom Frontend

Shortlinker supports custom frontend implementations. Place your built frontend in `./frontend-panel` to replace the built-in panel.

### How to Use

1. **Prepare your frontend**:
   - Build your frontend app
   - Put build outputs under `./frontend-panel` at project root
   - Ensure `index.html` is in that directory root

2. **Template repository**:
   - Official template: [shortlinker-frontend](https://github.com/AptS-1547/shortlinker-frontend/)
   - Fork and customize as needed

3. **Parameter injection**:
   Placeholders in HTML files (`index.html`, `manifest.webmanifest`) are auto-replaced:
   - `%BASE_PATH%` - frontend route prefix (e.g. `/panel`)
   - `%ADMIN_ROUTE_PREFIX%` - Admin API prefix (e.g. `/admin`)
   - `%HEALTH_ROUTE_PREFIX%` - Health API prefix (e.g. `/health`)
   - `%SHORTLINKER_VERSION%` - current Shortlinker version

4. **Detection**:
   On startup, Shortlinker auto-detects `./frontend-panel` and serves it when present. Log example:

   ```text
   Custom frontend detected at: ./frontend-panel
   ```

:::warning Priority
Custom frontend takes priority over the built-in panel. If `./frontend-panel` exists, it is served instead.
:::

## Main Features

### Core Capabilities

- 🔑 **Login and session auth**: sign in with `api.admin_token`; backend uses cookie-based session auth
- 📋 **Link management**: create, edit, delete, batch delete, QR code generation
- 🔍 **Search and filtering**: keyword search, status/date filters, sorting, pagination
- 📥 **Import/export**: CSV import/export with conflict strategies and drag-and-drop upload
- ⚙️ **Settings center**: runtime config editing, history, and reload actions

### UI Capabilities

- 🌓 Theme switching (light/dark/system)
- 🌍 Internationalization (Chinese, English, Japanese, French, Russian)
- 📱 Responsive layout (desktop and mobile)
- 📲 PWA install and offline access

## Interface Preview (Quick Tour)

### Dashboard

- Shows total links, active/expired ratio, and click metrics
- Shows storage backend info and system uptime

### Links Management Page

- Table view with status badges and quick actions
- Filtering, sorting, pagination, and column configuration

### Settings Page

- Preferences: theme and language
- System settings: runtime config management and reload

### Analytics Page (In Development)

- Planned views: click trends, top links, and traffic sources

## Roadmap (Brief)

- ✅ Core CRUD, auth, themes, i18n
- ✅ Batch ops, QR code, import/export, PWA
- ✅ System configuration management
- 🚧 Click analytics charts
- 📋 Link grouping and custom domain support

## Related Links

- 📖 [Admin API Documentation](/en/api/admin)
- 🔧 [Configuration Guide](/en/config/)
- 🚀 [Deployment Guide](/en/deployment/)
- 🛠️ [Development Guide](./development)
- ❓ [Troubleshooting](./troubleshooting)
