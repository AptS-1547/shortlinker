# Web Admin Panel

:::warning v0.3.x Version Notice
The current version (v0.3.x) is undergoing significant feature adjustments and refactoring with frequent updates. We recommend:

- 📌 Use stable version tags for production environments
- 🔄 Follow the latest version in development to experience new features
- 📖 Documentation may lag behind code implementation; actual functionality prevails
:::

Shortlinker provides a modern Web administration interface built with React 19 + TypeScript, located in the `admin-panel` directory, offering complete graphical management capabilities through the Admin API.

## Enabling the Panel

To enable the Web admin panel in Shortlinker:

1. **Build frontend assets**:

   ```bash
   cd admin-panel
   bun install
   bun run build
   ```

2. **Configure environment variables**:

   ```bash
   ENABLE_ADMIN_PANEL=true
   ADMIN_TOKEN=your_secure_admin_token
   FRONTEND_ROUTE_PREFIX=/panel  # Optional, defaults to /panel
   ```

3. **Access the interface**:
   After starting Shortlinker, visit `http://your-domain:8080/panel`

:::tip Note
This is an **experimental feature** currently under active development. Please report issues via GitHub Issues.
:::

## Custom Frontend

Shortlinker supports using custom frontend implementations. You can replace the built-in admin panel with your own frontend by placing it in the `./frontend-panel` directory.

### How to Use

1. **Prepare your frontend**:
   - Build your frontend application
   - Place the built files in `./frontend-panel` directory at the project root
   - Ensure `index.html` is in the root of this directory

2. **Template Repository**:
   - Official template: [shortlinker-frontend](https://github.com/AptS-1547/shortlinker-frontend/)
   - Fork and customize according to your needs

3. **Parameter Injection**:
   The following placeholders in HTML files (`index.html`, `manifest.webmanifest`) will be automatically replaced:
   - `%BASE_PATH%` - Frontend route prefix (e.g., `/panel`)
   - `%ADMIN_ROUTE_PREFIX%` - Admin API prefix (e.g., `/admin`)
   - `%HEALTH_ROUTE_PREFIX%` - Health API prefix (e.g., `/health`)
   - `%SHORTLINKER_VERSION%` - Current Shortlinker version

4. **Detection**:
   When Shortlinker starts, it will automatically detect the `./frontend-panel` directory and use it if present. You'll see a log message:

   ```text
   Custom frontend detected at: ./frontend-panel
   ```

:::warning Priority
Custom frontend takes priority over the built-in admin panel. If `./frontend-panel` exists, it will be served instead of the embedded frontend.
:::

## Main Features

### Core Functions

- 🔑 **Login & Session Auth**: Login with `ADMIN_TOKEN`; backend issues JWT cookies via `Set-Cookie` (access/refresh), and the UI uses cookie-based sessions for API calls
- 📋 **Link Management**: Complete CRUD interface
  - Create new short links (custom codes, expiration, password protection)
  - Edit existing links
  - Delete links (with confirmation)
  - Batch selection and batch deletion
  - QR code generation
- 📊 **Data Visualization**:
  - Dashboard showing key metrics
  - Storage backend status monitoring
  - System uptime display
- 🔍 **Advanced Features**:
  - Search by code or URL
  - Filter by status (all/active/expired)
  - Filter by creation date range
  - Multi-column sorting (code, target, clicks, created, expires)
  - Pagination (10/20/50/100 items per page)
  - Copy short links to clipboard
  - Column configuration (show/hide table columns)
- 📥 **Import/Export**:
  - CSV export (supports filter conditions)
  - CSV import (supports skip/overwrite/error conflict modes)
  - Drag and drop upload support

### Interface Features

- 🌓 **Theme Switching**: Light/dark/system auto three modes
- 🌍 **Internationalization**: 5 languages (Chinese, English, Japanese, French, Russian)
- 📱 **Responsive Design**: Desktop and mobile compatible
- ⚡ **Performance Optimized**: React 19 + Vite build
- 📲 **PWA Support**: Installable to desktop, offline access

### Settings Page

- ⚙️ **Preferences**: Theme selection, language switching
- 🔧 **System Configuration**:
  - Runtime configuration management (grouped display)
  - Configuration editing (supports string/number/boolean/json types)
  - Configuration history
  - Reload configuration
- ℹ️ **About**: Version info, tech stack, open source license, project links

## Interface Preview

### Dashboard

- Total link count statistics
- Active/expired link counts
- Click count aggregation
- Storage backend information
- System uptime
- Recently created links list

### Links Management Page

- Table view of all links
- Status badges (active/expired/protected)
- Real-time click count display
- Quick action buttons (edit/delete/copy/QR code)
- Batch selection and operations
- Advanced filter bar
- Pagination navigation
- Column configuration dropdown

### Settings Page

- Preferences tab (theme/language)
- System configuration tab (runtime config management)
- About tab (version/tech stack/links)

### Analytics Page (In Development)

- Click trend charts
- Top links ranking
- Traffic source statistics

## Roadmap

- ✅ Basic CRUD functionality
- ✅ Authentication and authorization
- ✅ Theme switching
- ✅ Internationalization support (5 languages)
- ✅ Batch operations
- ✅ QR code generation
- ✅ Import/export functionality
- ✅ PWA support
- ✅ System configuration management
- 🚧 Click statistics charts
- 📋 Link group management
- 📋 Custom domain support

## Related Links

- 📖 [Admin API Documentation](/en/api/admin)
- 🔧 [Environment Configuration](/en/config/)
- 🚀 [Deployment Guide](/en/deployment/)
- 🛠️ [Development Guide](./development)
- ❓ [Troubleshooting](./troubleshooting)
