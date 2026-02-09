# Cloudflare Worker Variant (Experimental)

The Cloudflare Worker implementation in `cf-worker/` is currently under development and serves as an exploratory serverless path.

## Current Status

- 🚧 **In development**: not production-ready yet
- 📦 **Code location**: repository root `cf-worker/`
- 📖 **Latest progress**: tracked in `cf-worker/README.md` (EN) and `cf-worker/README.zh.md` (ZH)

## What Exists Today

- Worker project scaffold and baseline build config
- Rust + WebAssembly + Wrangler development workflow
- Local development/run commands

## Not Completed Yet (see README roadmap)

- Full short-link create/query/manage flows
- Full admin API and auth pipeline
- Analytics, rate limiting, and production-hardening features

## Local Tryout (development only)

```bash
cd cf-worker
cargo build
wrangler dev
```

## Deployment Note

- This page no longer treats Worker as a production-ready deployment target.
- Once the README marks it deployable, use `wrangler deploy` for publishing.
