# Docker 部署：运维与安全

本页聚焦镜像特性、数据管理、安全配置、故障排除与生产建议。

## 镜像特性

### 优势
- **极小体积**：基于 scratch 镜像，最终大小仅几 MB
- **安全性**：无操作系统，减少攻击面
- **性能**：单一二进制文件，启动迅速
- **跨平台**：支持 amd64、arm64 架构

### Prometheus metrics（可选）

`/health/metrics`（Prometheus 文本格式）是**编译时** `metrics` feature 提供的能力。

如果你访问 `GET /health/metrics` 返回 `404`，说明当前镜像/二进制未启用该 feature。

**推荐方式：使用预构建的 metrics 镜像**

```bash
# 直接使用带 -metrics 后缀的官方镜像
docker pull e1saps/shortlinker:latest-metrics
```

**手动编译启用：**

```bash
# 直接编译（在默认 features 基础上追加 metrics）
cargo build --release --features metrics

# 或者显式启用（例如 Dockerfile 中常见的 CLI 构建）
cargo build --release --features cli,metrics

# 全功能（包含 metrics）
cargo build --release --features full

# Docker 自构建
docker build --build-arg CARGO_FEATURES="cli,metrics" -t shortlinker:metrics .
```

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
# 推荐从 /deployment/docker-quickstart 复制基础 run 命令
# 这里只展示“安全相关差异项”

# 仅本地监听 - TCP
docker run -d --name shortlinker -p 127.0.0.1:8080:8080 ...

# Unix 套接字
docker run -d --name shortlinker -v $(pwd)/sock:/sock ...

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
