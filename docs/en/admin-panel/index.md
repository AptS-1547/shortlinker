# Web Admin Panel

:::warning v0.2.x Version Notice
The current version (v0.2.x) is undergoing significant feature adjustments and refactoring with frequent updates. We recommend:
- 📌 Use stable version tags for production environments
- 🔄 Follow the latest version in development to experience new features
- 📖 Documentation may lag behind code implementation; actual functionality prevails
:::

Shortlinker provides a modern Web administration interface built with Vue 3 + TypeScript, located in the `admin-panel` directory, offering complete graphical management capabilities through the Admin API.

## Enabling the Panel

To enable the Web admin panel in Shortlinker:

1. **Build frontend assets**:
   ```bash
   cd admin-panel
   yarn install
   yarn build
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

## Main Features

### Core Functions
- 🔑 **Token Authentication**: Secure Bearer Token-based authentication
- 📋 **Link Management**: Complete CRUD interface
  - Create new short links (custom codes, expiration, password protection)
  - Edit existing links
  - Delete links (with confirmation)
  - Batch operations (planned)
- 📊 **Data Visualization**:
  - Dashboard showing key metrics
  - Click statistics charts
  - Storage backend status monitoring
- 🔍 **Advanced Features**:
  - Filter and search links (active/expired/protected)
  - Pagination
  - Expiration reminders
  - Copy short links to clipboard
  - QR code generation (planned)

### Interface Features
- 🌓 **Theme Switching**: Light and dark theme support
- 🌍 **Internationalization**: Chinese and English interfaces
- 📱 **Responsive Design**: Desktop and mobile compatible
- ⚡ **Performance Optimized**: Vue 3 Composition API + Vite build

## Development Guide

### Local Development

```bash
cd admin-panel

# Install dependencies
yarn install

# Start development server
yarn dev
```

Development server runs at `http://localhost:5173`.

### Environment Configuration

Create `.env.local` file to configure backend API address:

```bash
# .env.local
VITE_API_URL=http://localhost:8080
```

Available environment variables:

| Variable | Description | Default |
|----------|-------------|---------|
| `VITE_API_URL` | Backend API address | `http://localhost:8080` |
| `VITE_DEFAULT_LOCALE` | Default language | `zh` |

### Build & Deploy

```bash
# Build for production
yarn build

# Preview build
yarn preview

# Type checking
yarn type-check

# Code linting
yarn lint
```

Build output is in `dist/` directory, which can be:
1. Served through Shortlinker (set `ENABLE_ADMIN_PANEL=true`)
2. Deployed to standalone static server (Nginx, Caddy, etc.)
3. Deployed to CDN (requires CORS configuration)

### Docker Integration

If using Docker, include frontend assets in image build:

```dockerfile
# Multi-stage build example
FROM node:18 AS frontend-builder
WORKDIR /app/admin-panel
COPY admin-panel/package.json admin-panel/yarn.lock ./
RUN yarn install
COPY admin-panel/ ./
RUN yarn build

FROM rust:1.75 AS backend-builder
# ... Rust build steps ...

FROM debian:bookworm-slim
COPY --from=frontend-builder /app/admin-panel/dist /app/admin-panel/dist
# ... other configurations ...
```

## Tech Stack

- **Framework**: Vue 3 + TypeScript
- **Build Tool**: Vite 5
- **Router**: Vue Router 4
- **State Management**: Pinia
- **UI Styling**: Native CSS + CSS Variables
- **HTTP Client**: Axios
- **Charts**: Chart.js (planned)
- **i18n**: Vue I18n

## Interface Preview

### Dashboard
- Total link count statistics
- Active/expired link counts
- Click count aggregation
- Storage backend information
- Service uptime

### Links Management Page
- Table view of all links
- Status badges (active/expired/protected)
- Real-time click count display
- Quick action buttons (edit/delete/copy)
- Pagination navigation

### Analytics Page (Planned)
- Click trend charts
- Top links ranking
- Traffic source statistics

## Security Recommendations

1. **Strong Password**: Use sufficiently complex `ADMIN_TOKEN`
2. **HTTPS**: Production must enable HTTPS
3. **Path Isolation**: Consider using non-default `FRONTEND_ROUTE_PREFIX`
4. **Network Isolation**: Only expose admin panel in trusted networks
5. **Regular Updates**: Keep dependencies updated for security fixes

## Troubleshooting

### Login Failed

```bash
# Check if ADMIN_TOKEN is correctly configured
echo $ADMIN_TOKEN

# Check API address configuration
cat admin-panel/.env.local

# View browser console errors
```

### Build Failed

```bash
# Clean dependencies and reinstall
rm -rf node_modules yarn.lock
yarn install

# Check Node.js version (requires >= 18)
node --version
```

### Style Issues

```bash
# Clear Vite cache
rm -rf admin-panel/.vite
yarn dev
```

## Roadmap

- ✅ Basic CRUD functionality
- ✅ Authentication and authorization
- ✅ Theme switching
- ✅ Internationalization support
- 🚧 Batch operations
- 🚧 QR code generation
- 🚧 Click statistics charts
- 📋 Export/import functionality
- 📋 Link group management
- 📋 Custom domain support

## Contributing

PRs welcome to improve the Web admin panel! Before developing:

1. Fork the project and create feature branch
2. Follow existing code style (use ESLint + Prettier)
3. Add necessary type definitions
4. Ensure build passes: `yarn type-check && yarn build`
5. Submit PR with description of changes

## Related Links

- 📖 [Admin API Documentation](/en/api/admin)
- 🔧 [Environment Configuration](/en/config/)
- 🚀 [Deployment Guide](/en/deployment/)
