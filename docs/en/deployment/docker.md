# Docker Deployment

Shortlinker provides optimized Docker images supporting various deployment methods.

## Navigation

- [Quick Start and Compose](/en/deployment/docker-quickstart)
- [Operations and Security](/en/deployment/docker-operations)

## Image Availability

### Standard (Default)

Without Prometheus metrics export, smaller image size.

```bash
# Docker Hub (recommended)
docker pull e1saps/shortlinker

# GitHub Container Registry
docker pull ghcr.io/apts-1547/shortlinker
```

### Metrics Edition

Includes Prometheus metrics export (`/health/metrics` endpoint), suitable for monitored production environments.

```bash
# Docker Hub
docker pull e1saps/shortlinker:latest-metrics

# GitHub Container Registry
docker pull ghcr.io/apts-1547/shortlinker:latest-metrics
```

### Available Tags

| Tag | Description |
|-----|-------------|
| `latest` | Latest build (standard) |
| `latest-metrics` | Latest build (with Prometheus metrics) |
| `stable` / `stable-metrics` | Latest stable release |
| `edge` / `edge-metrics` | Latest pre-release (alpha/beta/rc) |
| `vX.Y.Z` | Specific version (standard, placeholder example) |
| `vX.Y.Z-metrics` | Specific version (with Prometheus metrics, placeholder example) |

> For a pinned deployment, replace `vX.Y.Z` with an actual release tag (for example, `v0.5.0-beta.2`).

### Build from Source

```bash
git clone https://github.com/AptS-1547/shortlinker
cd shortlinker

# Standard
docker build -t shortlinker .

# Metrics edition
docker build --build-arg CARGO_FEATURES="cli,metrics" -t shortlinker:metrics .
```


## Next

- [Quick Start and Compose](/en/deployment/docker-quickstart)
- [Operations and Security](/en/deployment/docker-operations)
