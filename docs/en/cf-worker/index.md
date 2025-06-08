# Cloudflare Worker Version

A serverless variant built with Cloudflare Workers is included in the `cf-worker` directory. It runs on Cloudflare's edge network.

## Features

- ☁️ Globally distributed and fast
- 🗄️ KV storage for persistence
- 💸 Pay-as-you-go with zero server maintenance

## Deployment Steps

1. Install [Wrangler](https://developers.cloudflare.com/workers/wrangler/)
2. Configure account info and KV namespace in `wrangler.toml`
3. Run `wrangler publish` to deploy

Example:

```bash
cd cf-worker
wrangler publish
```
