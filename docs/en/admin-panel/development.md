# Development Guide

This document covers local development, build deployment, and contributing to the Web admin panel.

## Local Development

```bash
cd admin-panel

# Install dependencies
bun install

# Start development server
bun dev
```

Development server runs at `http://localhost:5173`.

## Environment Configuration

Create `.env.local` file to configure backend API address:

```bash
# .env.local
VITE_API_URL=http://localhost:8080
```

Available environment variables:

| Variable               | Description         | Default                  |
| ---------------------- | ------------------- | ------------------------ |
| `VITE_API_URL`         | Backend API address | `http://localhost:8080`  |
| `VITE_DEFAULT_LOCALE`  | Default language    | `zh`                     |

## Build & Deploy

```bash
# Build for production
bun run build

# Preview build
bun run preview

# Code linting
bun run lint
```

Build output is in `dist/` directory, which can be:

1. Served through Shortlinker (set `ENABLE_ADMIN_PANEL=true`)
2. Deployed to standalone static server (Nginx, Caddy, etc.)
3. Deployed to CDN (requires CORS configuration)

## Docker Integration

If using Docker, include frontend assets in image build:

```dockerfile
# Multi-stage build example
FROM node:24-alpine AS frontend-builder
RUN npm install -g bun@latest
WORKDIR /app/admin-panel
COPY admin-panel/ ./
RUN bun install --frozen-lockfile
RUN bun run build

FROM rust:1.92-slim AS backend-builder
# ... Rust build steps (using musl static linking) ...

FROM scratch
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/shortlinker /shortlinker
# ... other configurations ...
```

:::tip Note
For the complete Dockerfile, refer to the `Dockerfile` in the project root. The official image uses `scratch` as the base image with musl static linking for minimal deployment.
:::

## Tech Stack

- **Framework**: React 19 + TypeScript
- **Build Tool**: Vite 8
- **Router**: React Router 7
- **State Management**: Zustand
- **UI Components**: Radix UI + Tailwind CSS
- **HTTP Client**: Axios
- **i18n**: react-i18next
- **Form Validation**: React Hook Form + Zod
- **Code Style**: Biome

## Project Structure

```text
admin-panel/
├── src/
│   ├── components/     # UI components
│   │   ├── ui/         # Base components (Button, Dialog, etc.)
│   │   ├── layout/     # Layout components
│   │   ├── links/      # Link management components
│   │   └── settings/   # Settings page components
│   ├── pages/          # Page components
│   ├── hooks/          # Custom Hooks
│   ├── stores/         # Zustand state management
│   ├── services/       # API service layer
│   ├── i18n/           # Internationalization config
│   ├── router/         # Router configuration
│   ├── schemas/        # Zod validation schemas
│   ├── types/          # TypeScript type definitions
│   └── utils/          # Utility functions
├── public/             # Static assets
└── dist/               # Build output
```

## Contributing

PRs welcome to improve the Web admin panel! Before developing:

1. Fork the project and create feature branch
2. Follow existing code style (use Biome)
3. Add necessary type definitions
4. Ensure build passes: `bun run lint && bun run build`
5. Submit PR with description of changes

## Related Links

- 📖 [Feature Overview](./index)
- ❓ [Troubleshooting](./troubleshooting)
