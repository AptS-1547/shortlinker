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
# 最简单的启动方式
docker run -d -p 8080:8080 e1saps/shortlinker
```

### 数据持久化
```bash
# 挂载数据目录
docker run -d \
  -p 8080:8080 \
  -v $(pwd)/data:/data \
  -e STORAGE_TYPE=sqlite \
  -e SQLITE_DB_PATH=/data/links.db \
  e1saps/shortlinker
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
      - ./data:/data
    environment:
      - SERVER_HOST=0.0.0.0
      - SERVER_PORT=8080
      - STORAGE_TYPE=sqlite
      - SQLITE_DB_PATH=/data/links.db
      - DEFAULT_URL=https://example.com
      - RUST_LOG=info
    restart: unless-stopped
```

### 生产环境配置
```yaml
# docker-compose.prod.yml
version: '3.8'

services:
  shortlinker:
    image: e1saps/shortlinker:latest
    ports:
      - "127.0.0.1:8080:8080"  # 仅本地监听
    volumes:
      - ./data:/data
    environment:
      - SERVER_HOST=0.0.0.0
      - STORAGE_TYPE=sqlite
      - SQLITE_DB_PATH=/data/links.db
      - DEFAULT_URL=https://your-domain.com
      - ADMIN_TOKEN=${ADMIN_TOKEN}  # 从环境变量读取
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
FROM rust:1.70 as builder
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
├── links.json          # JSON 文件存储（如果使用）
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
# 仅本地监听
docker run -d -p 127.0.0.1:8080:8080 e1saps/shortlinker

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
# 交互模式运行
docker run -it --rm \
  -p 8080:8080 \
  -v $(pwd)/data:/data \
  -e RUST_LOG=debug \
  e1saps/shortlinker
```

## 生产环境建议

1. **使用具体版本标签**，避免 `latest`
2. **设置资源限制**，防止资源滥用  
3. **配置健康检查**，自动重启异常容器
4. **定期备份数据**，配置监控告警
5. **通过反向代理**暴露服务，不直接暴露容器端口
