# Docker 部署：快速开始与 Compose

本页聚焦容器快速开始、数据持久化、运行时配置，以及 Docker Compose 示例。

## 快速开始

### 基本运行
```bash
# 1) 准备最小启动配置（config.toml 只在容器当前工作目录读取；默认是 /config.toml）
cat > config.toml << 'EOF'
[server]
host = "0.0.0.0"
port = 8080

[database]
database_url = "sqlite:///data/shortlinker.db"
EOF

mkdir -p data

# 2) 启动
docker run -d --name shortlinker \
  -p 8080:8080 \
  -v $(pwd)/config.toml:/config.toml:ro \
  -v $(pwd)/data:/data \
  e1saps/shortlinker
```

### 数据持久化
```bash
# TCP（推荐）
docker run -d --name shortlinker \
  -p 8080:8080 \
  -v $(pwd)/config.toml:/config.toml:ro \
  -v $(pwd)/data:/data \
  e1saps/shortlinker

# Unix 套接字（HTTP 走 UDS；host/port 会被忽略）
cat > config.toml << 'EOF'
[server]
unix_socket = "/sock/shortlinker.sock"

[database]
database_url = "sqlite:///data/shortlinker.db"
EOF

docker run -d --name shortlinker \
  -v $(pwd)/config.toml:/config.toml:ro \
  -v $(pwd)/data:/data \
  -v $(pwd)/sock:/sock \
  e1saps/shortlinker
```

### 运行时配置（Admin/Health/Panel）

> 运行时配置存储在数据库中（如 `features.default_url`、`features.enable_admin_panel`、`api.health_token`），可通过容器内 CLI 或 Admin API 修改。详见 [配置指南](/config/)。

```bash
# 获取首次启动生成的管理员密码（容器内通常为 /admin_token.txt）
docker exec shortlinker cat /admin_token.txt

# 设置根路径默认跳转（无需重启）
docker exec shortlinker /shortlinker config set features.default_url https://example.com

# 配置 Health Bearer Token（无需重启）
docker exec shortlinker /shortlinker config set api.health_token your_health_token

# 启用管理面板（需要重启）
docker exec shortlinker /shortlinker config set features.enable_admin_panel true
docker restart shortlinker
```

## Docker Compose

### 基础配置
```yaml
# docker-compose.yml
version: '3.8'

services:
  shortlinker:
    image: e1saps/shortlinker
    container_name: shortlinker
    ports:
      - "8080:8080"
    volumes:
      - ./config.toml:/config.toml:ro
      - ./data:/data
    restart: unless-stopped
```

### 生产环境配置
```yaml
# docker-compose.prod.yml
version: '3.8'

services:
  shortlinker:
    image: e1saps/shortlinker:latest
    # TCP 端口
    ports:
      - "127.0.0.1:8080:8080"
    # Unix 套接字挂载（二选一）
    # volumes:
    #   - ./sock:/sock
    volumes:
      - ./config.toml:/config.toml:ro
      - ./data:/data
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "wget", "--quiet", "--tries=1", "--spider", "http://localhost:8080/"]
      interval: 30s
      timeout: 10s
      retries: 3
```

## 启动和管理

```bash
# 启动服务
docker-compose up -d

# 查看日志
docker-compose logs -f shortlinker

# 停止服务
docker-compose down

# 更新镜像
docker-compose pull && docker-compose up -d
```

