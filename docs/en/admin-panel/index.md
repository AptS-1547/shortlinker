# Web Admin Panel

Shortlinker includes an optional Vue 3 based administration panel located in the `admin-panel` directory. It communicates with the backend via the Admin API for graphical management.

To enable the panel in Shortlinker, build `admin-panel/dist` and set `ENABLE_FRONTEND_ROUTES=true` along with `ADMIN_TOKEN`. This feature is new and may be unstable.
## Main Features

- 🔑 Token authentication login
- ✨ Create, update and delete short links
- 📊 Realtime statistics and health status display
- 🕒 Expiration time management with reminders

## Local Development

```bash
cd admin-panel
yarn install
yarn dev
```

The development server runs at `http://localhost:5173`. Set `VITE_API_URL` in your environment to point to the backend API.

## Build & Deploy

```bash
# Build static files
yarn build

# The output in dist/ can be served by any static server
```

See [`admin-panel/README.md`](../../admin-panel/README.md) for full details.
