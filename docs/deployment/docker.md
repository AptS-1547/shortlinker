# Docker 部署

Shortlinker 提供了优化的 Docker 镜像，支持多种部署方式。

## 文档导航

- [快速开始与 Compose](/deployment/docker-quickstart)
- [运维与安全](/deployment/docker-operations)

## 镜像获取

### 标准版（默认）

不含 Prometheus 指标导出功能，体积更小。

```bash
# Docker Hub（推荐）
docker pull e1saps/shortlinker

# GitHub Container Registry
docker pull ghcr.io/apts-1547/shortlinker
```

### Metrics 版

包含 Prometheus 指标导出功能（`/health/metrics` 端点），适合需要监控的生产环境。

```bash
# Docker Hub
docker pull e1saps/shortlinker:latest-metrics

# GitHub Container Registry
docker pull ghcr.io/apts-1547/shortlinker:latest-metrics
```

### 可用标签

| 标签 | 说明 |
|------|------|
| `latest` | 最新构建（标准版） |
| `latest-metrics` | 最新构建（含 Prometheus 指标） |
| `stable` / `stable-metrics` | 最新正式发布版本 |
| `edge` / `edge-metrics` | 最新预发布版本（alpha/beta/rc） |
| `v0.5.0-alpha.6` | 特定版本（标准版） |
| `v0.5.0-alpha.6-metrics` | 特定版本（含 Prometheus 指标） |

### 自构建镜像

```bash
git clone https://github.com/AptS-1547/shortlinker
cd shortlinker

# 标准版
docker build -t shortlinker .

# Metrics 版
docker build --build-arg CARGO_FEATURES="cli,metrics" -t shortlinker:metrics .
```


## 下一步

- [快速开始与 Compose](/deployment/docker-quickstart)
- [运维与安全](/deployment/docker-operations)
