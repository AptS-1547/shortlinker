# Shortlinker Cloudflare Worker

A serverless short link service built with Cloudflare Workers and KV storage.

## Features

- ⚡ **Serverless** - Runs on Cloudflare's global edge network
- 🗄️ **KV Storage** - Uses Cloudflare KV for persistent data storage
- 🌍 **Global Distribution** - Low latency worldwide
- 🔒 **Secure** - Built-in DDoS protection and security features
- 📈 **Scalable** - Automatic scaling with zero configuration
- 💰 **Cost Effective** - Pay-per-use pricing model

## Tech Stack

- **Runtime**: Cloudflare Workers (Rust + WebAssembly)
- **Language**: Rust + `worker` crate
- **Storage**: Cloudflare KV
- **Build Tool**: `wasm-pack` + `wrangler`

## Development Status

🚧 **Under Development** - This Cloudflare Worker implementation is currently being developed to provide a serverless alternative to the main Shortlinker service.

### Planned Features

- [ ] Short link creation and redirection
- [ ] KV-based data persistence
- [ ] Admin API endpoints
- [ ] Rate limiting and abuse protection
- [ ] Analytics and usage tracking
- [ ] Custom domain support
- [ ] Bulk operations

## Project Structure

```
cf-worker/
├── src/
│   └── lib.rs              # Main worker logic
├── build/                  # Built WebAssembly artifacts
├── wrangler.toml          # Cloudflare Worker configuration
├── Cargo.toml             # Rust dependencies
└── .wrangler/             # Wrangler cache and state
```

## Environment Setup

### Prerequisites

- [Rust](https://rustup.rs/) toolchain
- [wrangler](https://developers.cloudflare.com/workers/wrangler/) CLI
- Cloudflare account with Workers enabled

### Installation

```bash
# Install wrangler
npm install -g wrangler

# Authenticate with Cloudflare
wrangler auth

# Install Rust target for WebAssembly
rustup target add wasm32-unknown-unknown
```

## Configuration

### KV Namespace Setup

Create KV namespaces for the worker:

```bash
# Create production KV namespace
wrangler kv:namespace create "SHORTLINK_STORE"

# Create preview KV namespace for development
wrangler kv:namespace create "SHORTLINK_STORE" --preview
```

### wrangler.toml Configuration

```toml
# wrangler.toml
name = "shortlinker-worker"
main = "build/worker/index.js"
compatibility_date = "2024-01-01"

[build]
command = "cargo build --target wasm32-unknown-unknown --release && wasm-pack build --target no-modules --out-dir build"

[[kv_namespaces]]
binding = "SHORTLINK_STORE"
id = "your_kv_namespace_id"
preview_id = "your_preview_kv_namespace_id"

[vars]
ADMIN_TOKEN = "your_admin_token"
BASE_URL = "https://your-worker.your-subdomain.workers.dev"
```

## Development

### Local Development

```bash
# Navigate to cf-worker directory
cd cf-worker

# Install dependencies
cargo build

# Start local development server
wrangler dev
```

### Build for Production

```bash
# Build WebAssembly
cargo build --target wasm32-unknown-unknown --release

# Generate JavaScript bindings
wasm-pack build --target no-modules --out-dir build

# Deploy to Cloudflare
wrangler deploy
```

## API Endpoints

The worker will implement the following endpoints:

### Public Endpoints

- `GET /{code}` - Redirect to target URL
- `POST /create` - Create new short link (with rate limiting)

### Admin Endpoints (Protected)

- `GET /admin/links` - List all short links
- `POST /admin/links` - Create new short link
- `GET /admin/links/{code}` - Get specific short link
- `PUT /admin/links/{code}` - Update short link
- `DELETE /admin/links/{code}` - Delete short link

## KV Data Structure

Short links will be stored in KV with the following structure:

```json
{
  "code": "abc123",
  "target": "https://example.com",
  "created_at": "2024-01-01T00:00:00Z",
  "expires_at": "2024-12-31T23:59:59Z",
  "hits": 0,
  "last_accessed": "2024-01-01T00:00:00Z"
}
```

### KV Keys

- `link:{code}` - Individual short link data
- `stats:total` - Total number of links
- `stats:hits` - Total number of redirects

## Rust Implementation Preview

```rust
// Future implementation structure
use worker::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct ShortLink {
    code: String,
    target: String,
    created_at: String,
    expires_at: Option<String>,
    hits: u64,
}

#[event(fetch)]
async fn main(req: Request, env: Env, ctx: Context) -> Result<Response> {
    let router = Router::new();
    
    router
        .get_async("/:code", redirect_handler)
        .post_async("/create", create_handler)
        .get_async("/admin/links", admin_list_handler)
        .run(req, env)
        .await
}

async fn redirect_handler(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let kv = ctx.env.kv("SHORTLINK_STORE")?;
    // Implementation for redirect logic
    todo!()
}
```

## Deployment

### Using Wrangler

```bash
# Deploy to production
wrangler deploy

# Deploy with environment variables
wrangler deploy --var ADMIN_TOKEN:your_token
```

### Environment Variables

Configure the following variables in Cloudflare Dashboard or via wrangler:

- `ADMIN_TOKEN` - Authentication token for admin endpoints
- `BASE_URL` - Base URL for the worker
- `RATE_LIMIT` - Rate limiting configuration

## Monitoring and Analytics

The worker will include:

- Request logging and metrics
- KV operation monitoring
- Error tracking and alerting
- Usage analytics dashboard

## Security Features

- Rate limiting per IP
- Admin token authentication
- Input validation and sanitization
- CORS configuration
- Request size limits

## Performance Optimization

- Efficient KV key design
- Caching strategies
- Minimal payload sizes
- Edge-side processing

## Cost Estimation

Cloudflare Workers pricing (as of 2024):

- **Free Tier**: 100,000 requests/day
- **Paid Plan**: $5/month for 10M requests
- **KV Storage**: $0.50 per million reads, $5 per million writes

## Limitations

- KV eventual consistency
- 25ms CPU time limit per request
- 128MB memory limit
- 1MB request/response size limit

## Migration from Main Service

The worker can serve as:

1. **Standalone service** - Complete replacement
2. **Edge cache** - Frontend for the main service
3. **Backup service** - Fallback during maintenance

## Development Roadmap

This implementation will be developed in phases:

1. **Phase 1**: Basic redirect functionality
2. **Phase 2**: KV storage integration
3. **Phase 3**: Admin API implementation
4. **Phase 4**: Advanced features and optimization

## Related Documentation

- 📖 [Shortlinker Main Documentation](../README.md)
- 🔧 [Main Service Source](../src/)
- 🎛️ [Admin Panel](../admin-panel/)
- ☁️ [Cloudflare Workers Documentation](https://developers.cloudflare.com/workers/)

## Contributing

This is part of the Shortlinker project. Please see the main [Contributing Guide](../CONTRIBUTING.md) for guidelines.

## License

MIT License - See [LICENSE](../LICENSE) file for details.
