# Shortlinker Admin Panel

A modern web management interface for managing [Shortlinker](../README.md) short link service.

## Features

- 🎨 **Modern Interface** - Responsive design based on Vue 3 + TailwindUI
- 🔐 **Secure Authentication** - Bearer Token authentication with seamless Admin API integration
- 📊 **Complete Management** - Support for CRUD operations on short links
- ⚡ **Real-time Updates** - Automatic data refresh after operations
- 🕐 **Expiration Management** - Visual expiration time setting and display

## Tech Stack

- **Frontend Framework**: Vue 3 + TypeScript
- **UI Components**: TailwindUI + Headless UI
- **State Management**: Pinia
- **Build Tool**: Vite
- **Package Manager**: Yarn

## Development Status

🚧 **Under Development** - This admin panel is currently in the planning and development phase, and will be completed in future versions.

### Planned Features

- [ ] User authentication interface
- [ ] Short link list management
- [ ] Create and edit short links
- [ ] Batch operations
- [ ] Statistics dashboard
- [ ] Internationalization support

## Environment Configuration

Future support for the following environment variables:

```bash
# Shortlinker service address
VITE_API_BASE_URL=http://localhost:8080

# Admin API route prefix
VITE_ADMIN_ROUTE_PREFIX=/admin

# Default admin token (development environment)
VITE_DEFAULT_ADMIN_TOKEN=your_admin_token
```

## API Integration

Admin Panel will be built based on Shortlinker's [Admin API](../src/admin.rs), supporting the following endpoints:

- `GET /admin/link` - Get all short links
- `POST /admin/link` - Create new short link
- `GET /admin/link/{code}` - Get specific short link
- `PUT /admin/link/{code}` - Update short link
- `DELETE /admin/link/{code}` - Delete short link

## Authentication

All API requests require a Bearer Token in the header:

```
Authorization: Bearer {ADMIN_TOKEN}
```

## Development Roadmap

This admin panel will be implemented progressively in future versions. Stay tuned!

## Related Documentation

- 📖 [Shortlinker Main Documentation](../README.md)
- 🔧 [Admin API Source Code](../src/admin.rs)
- ⚙️ [Configuration Guide](../docs/config/index.md)

## License

MIT License - See [LICENSE](../LICENSE) file for details.