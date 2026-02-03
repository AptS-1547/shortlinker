# Docker 部署

Shortlinker 提供了优化的 Docker 镜像，支持多种部署方式。

## 镜像获取

```bash
# Docker Hub（推荐）
docker pull e1saps/shortlinker

# GitHub Container Registry  
docker pull ghcr.io/apts-1547/shortlinker

# 自构建镜像
git clone https://github.com/AptS-1547/shortlinker
cd shortlinker && docker build -t shortlinker .
```

## 快速启动

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

## 镜像特性

### 优势
- **极小体积**：基于 scratch 镜像，最终大小仅几 MB
- **安全性**：无操作系统，减少攻击面
- **性能**：单一二进制文件，启动迅速
- **跨平台**：支持 amd64、arm64 架构

### 多阶段构建
```dockerfile
# 构建阶段
FROM rust:1.92-slim as builder
WORKDIR /app
COPY . .
RUN cargo build --release

# 运行阶段
FROM scratch
COPY --from=builder /app/target/release/shortlinker /shortlinker
EXPOSE 8080
CMD ["/shortlinker"]
```

## 数据管理

### 数据目录结构
```
data/
├── links.db            # SQLite 数据库文件
└── backup/            # 备份目录（可选）
    └── links.db.20240101
```

### 备份策略
```bash
# 创建备份脚本
cat > backup.sh << 'EOF'
#!/bin/bash
docker exec shortlinker cp /data/links.db /data/backup/links.db.$(date +%Y%m%d_%H%M%S)
find ./data/backup -name "links.db.*" -mtime +7 -delete
EOF

chmod +x backup.sh

# 定时备份（添加到 crontab）
0 2 * * * /path/to/backup.sh
```

## 安全配置

### 网络安全
```bash
# 仅本地监听 - TCP
docker run -d --name shortlinker \
  -p 127.0.0.1:8080:8080 \
  -v $(pwd)/config.toml:/config.toml:ro \
  -v $(pwd)/data:/data \
  e1saps/shortlinker

# Unix 套接字
docker run -d --name shortlinker \
  -v $(pwd)/config.toml:/config.toml:ro \
  -v $(pwd)/data:/data \
  -v $(pwd)/sock:/sock \
  e1saps/shortlinker

# 自定义网络
docker network create shortlinker-net
docker run -d --network shortlinker-net --name shortlinker e1saps/shortlinker
```

### 资源限制
```yaml
services:
  shortlinker:
    image: e1saps/shortlinker
    deploy:
      resources:
        limits:
          memory: 128M
          cpus: '0.5'
        reservations:
          memory: 64M
          cpus: '0.25'
```

## 故障排除

### 常见问题
```bash
# 检查容器状态
docker ps -a
docker logs shortlinker

# 检查端口占用
netstat -tlnp | grep 8080

# 检查挂载点
docker inspect shortlinker | grep -A 10 Mounts

# 监控资源使用
docker stats shortlinker
```

### 调试模式
```bash
# 交互模式运行（把 `config.toml` 的 `logging.level` 设为 `debug` 以获得更详细日志）
docker run -it --rm \
  -p 8080:8080 \
  -v $(pwd)/config.toml:/config.toml:ro \
  -v $(pwd)/data:/data \
  e1saps/shortlinker
```

## 生产环境建议

1. **使用具体版本标签**，避免 `latest`
2. **设置资源限制**，防止资源滥用  
3. **配置健康检查**，自动重启异常容器
4. **定期备份数据**，配置监控告警
5. **通过反向代理**暴露服务，不直接暴露容器端口
