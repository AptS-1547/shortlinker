# Docker 部署

Shortlinker 提供了优化的 Docker 镜像，支持多种部署方式。

## 镜像获取

### 官方镜像

```bash
# Docker Hub
docker pull e1saps/shortlinker

# GitHub Container Registry  
docker pull ghcr.io/apts-1547/shortlinker
```

### 自构建镜像

```bash
# 克隆项目
git clone https://github.com/AptS-1547/shortlinker
cd shortlinker

# 构建镜像
docker build -t shortlinker .
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
  e1saps/shortlinker
```

### 完整配置

```bash
docker run -d \
  --name shortlinker \
  -p 8080:8080 \
  -v $(pwd)/data:/data \
  -e SERVER_HOST=0.0.0.0 \
  -e SERVER_PORT=8080 \
  -e LINKS_FILE=/data/links.json \
  -e DEFAULT_URL=https://example.com \
  -e RANDOM_CODE_LENGTH=8 \
  -e RUST_LOG=info \
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
      - LINKS_FILE=/data/links.json
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
    container_name: shortlinker-prod
    ports:
      - "127.0.0.1:8080:8080"  # 仅本地监听
    volumes:
      - ./data:/data
      - ./logs:/logs
    environment:
      - SERVER_HOST=0.0.0.0
      - SERVER_PORT=8080
      - LINKS_FILE=/data/links.json
      - DEFAULT_URL=https://your-domain.com
      - RANDOM_CODE_LENGTH=8
      - RUST_LOG=info
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/"]
      interval: 30s
      timeout: 10s
      retries: 3
    
  nginx:
    image: nginx:alpine
    container_name: nginx-proxy
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf
      - ./ssl:/etc/nginx/ssl
    depends_on:
      - shortlinker
    restart: unless-stopped
```

## 启动和管理

```bash
# 启动服务
docker-compose up -d

# 查看日志
docker-compose logs -f shortlinker

# 停止服务
docker-compose down

# 重启服务
docker-compose restart shortlinker

# 更新镜像
docker-compose pull
docker-compose up -d
```

## 镜像特性

### 多阶段构建

Dockerfile 使用多阶段构建优化镜像大小：

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

### 镜像优势

- **极小体积**：基于 scratch 镜像，最终大小仅几 MB
- **安全性**：无操作系统，减少攻击面
- **性能**：单一二进制文件，启动迅速
- **跨平台**：支持 amd64、arm64 架构

## 环境变量

容器支持所有标准环境变量：

```bash
# 服务器配置
SERVER_HOST=0.0.0.0
SERVER_PORT=8080

# 存储配置  
LINKS_FILE=/data/links.json

# 功能配置
DEFAULT_URL=https://example.com
RANDOM_CODE_LENGTH=8

# 日志配置
RUST_LOG=info
```

## 数据管理

### 数据目录结构

```
data/
├── links.json          # 链接数据文件
└── backup/            # 备份目录（可选）
    ├── links.json.20240101
    └── links.json.20240102
```

### 备份策略

```bash
# 创建备份
docker exec shortlinker cp /data/links.json /data/backup/links.json.$(date +%Y%m%d)

# 定时备份脚本
#!/bin/bash
docker exec shortlinker cp /data/links.json /data/backup/links.json.$(date +%Y%m%d_%H%M%S)
find ./data/backup -name "links.json.*" -mtime +7 -delete
```

## 故障排除

### 常见问题

1. **容器无法启动**
   ```bash
   # 检查日志
   docker logs shortlinker
   
   # 检查端口占用
   docker ps -a
   netstat -tlnp | grep 8080
   ```

2. **数据丢失**
   ```bash
   # 检查挂载点
   docker inspect shortlinker | grep Mounts -A 10
   
   # 检查权限
   ls -la data/
   ```

3. **性能问题**
   ```bash
   # 监控资源使用
   docker stats shortlinker
   
   # 检查容器健康状态
   docker healthcheck shortlinker
   ```

### 调试模式

```bash
# 以交互模式运行（调试用）
docker run -it --rm \
  -p 8080:8080 \
  -v $(pwd)/data:/data \
  -e RUST_LOG=debug \
  e1saps/shortlinker

# 进入运行中的容器（如果基础镜像支持）
docker exec -it shortlinker /bin/sh
```

## 安全配置

### 网络安全

```bash
# 仅本地监听
docker run -d \
  -p 127.0.0.1:8080:8080 \
  e1saps/shortlinker

# 自定义网络
docker network create shortlinker-net
docker run -d \
  --network shortlinker-net \
  --name shortlinker \
  e1saps/shortlinker
```

### 资源限制

```yaml
# docker-compose.yml
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

## 生产环境建议

1. **使用具体版本标签**，避免 `latest`
2. **设置资源限制**，防止资源滥用  
3. **配置健康检查**，自动重启异常容器
4. **使用非 root 用户**运行容器
5. **定期备份数据**，配置监控告警
6. **通过反向代理**暴露服务，不直接暴露容器端口
