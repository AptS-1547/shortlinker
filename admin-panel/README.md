# Shortlinker Admin Panel

A modern web administration interface for managing the [Shortlinker](../README.md) URL shortening service.

To enable this panel with Shortlinker, build the `dist` directory and set `ENABLE_FRONTEND_ROUTES=true` with an `ADMIN_TOKEN` in your environment variables or `.env` file. This feature is still experimental.
## Features

- 🎨 **Modern Interface** - Responsive design built with Vue 3 + Vite
- 🔐 **Secure Authentication** - Token-based access to Admin API  
- 📊 **Complete Management** - Full CRUD operations for short links
- ⚡ **Real-time Updates** - Auto-refresh data after operations
- 🕐 **Expiration Management** - Visual expiration time setting with local timezone support
- 💚 **Health Monitoring** - Real-time backend service status display
- 🔄 **Client-side Routing** - SPA routing system based on Vue Router

## Tech Stack

- **Frontend Framework**: Vue 3 + TypeScript + Vite
- **UI Components**: TailwindCSS + Vue
- **HTTP Client**: Axios
- **State Management**: Pinia
- **Routing System**: Vue Router
- **Package Manager**: Yarn

## Completed Features

- ✅ User authentication interface
- ✅ Short link list management
- ✅ Create and edit short links
- ✅ Delete link functionality
- ✅ Health status monitoring
- ✅ Local timezone time display
- ✅ Client-side routing system
- ✅ Conflict detection and handling

## Environment Configuration

Supports the following environment variables:

```bash
# Shortlinker service address
VITE_API_BASE_URL=http://localhost:8080

# Admin API route prefix
VITE_ADMIN_ROUTE_PREFIX=/admin

# Health check route prefix
VITE_HEALTH_ROUTE_PREFIX=/health
```

## API Integration

Admin Panel is built on Shortlinker's [Admin API](../src/services/admin.rs), supporting:

- `GET /admin/link` - Get all short links
- `POST /admin/link` - Create new short link
- `GET /admin/link/{code}` - Get specific short link
- `PUT /admin/link/{code}` - Update short link
- `DELETE /admin/link/{code}` - Delete short link
- `GET /health` - Health check

## Authentication

All API requests require Bearer Token in header:

```
Authorization: Bearer {ADMIN_TOKEN}
```

## Development

```bash
# Install dependencies
yarn install

# Development mode
yarn dev

# Build for production
yarn build
```

## Routing System

The admin panel uses Vue Router with the following routes:

- `/admin/login` - User login page
- `/admin/dashboard` - Main dashboard (link management)
- `/admin/links` - Link management page
- `/admin/analytics` - Data analytics page (planned)

### Route Features

- 🔒 **Route Guards** - Automatic authentication status checking
- 📱 **Responsive** - Mobile and desktop support
- ⚡ **Fast Navigation** - Instant routing based on Vue Router
- 🔄 **State Persistence** - Maintain application state during navigation

## Related Documentation

- 📖 [Shortlinker Main Documentation](../README.md)
- 🔧 [Admin API Source Code](../src/services/admin.rs)
- ⚙️ [Configuration Guide](../docs/config/index.md)

## License

MIT License - See [LICENSE](../LICENSE) file for details.
