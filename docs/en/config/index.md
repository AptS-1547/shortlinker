# Configuration Guide

To improve navigation, configuration docs are now split into an overview plus focused topic pages.

## Navigation

- [Startup Parameters](/en/config/startup)
- [Runtime Keys and API Workflow](/en/config/runtime)
- [Examples and Hot Reload](/en/config/examples)
- [Storage Overview](/en/config/storage)
- [Storage Backends](/en/config/storage-backends)
- [Storage Selection and Benchmarks](/en/config/storage-selection)
- [Storage Operations and Monitoring](/en/config/storage-operations)

## Architecture

Shortlinker configuration has two layers:

- **Startup config**: stored in `config.toml`, changes require restart
- **Runtime config**: stored in DB, can be updated at runtime via Admin API/panel

```text
config.toml (read at startup)
       ↓
StaticConfig (startup config, in-memory)
       ↓
Database (short links + runtime config)
       ↓
RuntimeConfig (runtime config cache, in-memory)
       ↓
Business logic (routes/auth/cache/etc)
```

On first startup, runtime defaults are initialized into DB based on built-in definitions; after that, DB values are authoritative.

## Configuration Priority

1. **Database (runtime config)**: `api.*` / `routes.*` / `features.*` / `click.*` / `cors.*` / `analytics.*` / `utm.*`
2. **Environment variables (startup overrides)**: `SL__...`
3. **`config.toml` (startup config; e.g. `[server]` / `[database]` / `[cache]` / `[logging]` / `[analytics]` / `[ipc]`)**
4. **Program defaults**

> Env vars only affect startup config. Runtime config is not auto-migrated from env vars or `config.toml`.

## Next Steps

- 📋 [Storage Overview](/en/config/storage)
- 🚀 [Deployment Guide](/en/deployment/)
- 🛡️ [Admin API](/en/api/admin)
- 🏥 [Health Check API](/en/api/health)
