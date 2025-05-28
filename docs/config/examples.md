# 配置示例

各种环境下的完整配置示例。

## 开发环境

### 本地开发 (.env)

```bash
# 服务器配置
SERVER_HOST=127.0.0.1
SERVER_PORT=8080

# 功能配置
DEFAULT_URL=https://localhost:3000
RANDOM_CODE_LENGTH=4  # 短一些便于测试

# 存储配置
LINKS_FILE=./dev-links.json

# 日志配置
RUST_LOG=debug  # 开发时启用详细日志
```

### 测试环境

```bash
# 服务器配置
SERVER_HOST=0.0.0.0
SERVER_PORT=3000

# 测试特定配置
DEFAULT_URL=https://test.example.com
RANDOM_CODE_LENGTH=6

# 存储配置
LINKS_FILE=./test-links.json

# 日志配置
RUST_LOG=info
```

## 生产环境

### 基础生产配置 (.env)

```bash
# 服务器配置
SERVER_HOST=127.0.0.1  # 通过反向代理暴露
SERVER_PORT=8080

# 存储配置
LINKS_FILE=data/links.json

# 默认跳转地址
DEFAULT_URL=https://your-company.com

# 随机码长度
RANDOM_CODE_LENGTH=8  # 生产环境建议更长

# 日志级别
RUST_LOG=info
```

### 高可用配置

```bash
# 服务器配置
SERVER_HOST=0.0.0.0
SERVER_PORT=8080

# 存储配置（共享存储）
LINKS_FILE=/shared/data/links.json

# 功能配置
DEFAULT_URL=https://www.example.com
RANDOM_CODE_LENGTH=10  # 更高唯一性

# 日志配置
RUST_LOG=warn  # 减少日志量
```

## Docker 环境

### Docker Compose 环境变量

```yaml
version: '3.8'
services:
  shortlinker:
    image: e1saps/shortlinker
    environment:
      - SERVER_HOST=0.0.0.0
      - SERVER_PORT=8080
      - LINKS_FILE=/data/links.json
      - DEFAULT_URL=https://your-domain.com
      - RANDOM_CODE_LENGTH=8
      - RUST_LOG=info
    volumes:
      - ./data:/data
    ports:
      - "127.0.0.1:8080:8080"
```

### Docker 命令行

```bash
docker run -d \
  --name shortlinker \
  -p 127.0.0.1:8080:8080 \
  -v $(pwd)/data:/data \
  -e SERVER_HOST=0.0.0.0 \
  -e SERVER_PORT=8080 \
  -e LINKS_FILE=/data/links.json \
  -e DEFAULT_URL=https://your-site.com \
  -e RANDOM_CODE_LENGTH=8 \
  -e RUST_LOG=info \
  e1saps/shortlinker
```

## 云服务环境

### AWS EC2

```bash
# EC2 实例配置
SERVER_HOST=0.0.0.0
SERVER_PORT=8080

# 使用 EFS 或 EBS 持久化存储
LINKS_FILE=/mnt/efs/shortlinker/links.json

# 云环境配置
DEFAULT_URL=https://www.your-aws-site.com
RANDOM_CODE_LENGTH=8

# CloudWatch 日志
RUST_LOG=info
```

### Azure Container Instances

```bash
# 容器配置
SERVER_HOST=0.0.0.0
SERVER_PORT=8080

# Azure Files 存储
LINKS_FILE=/mnt/azure-files/links.json

# 功能配置
DEFAULT_URL=https://your-azure-site.azurewebsites.net
RANDOM_CODE_LENGTH=8

# Azure Monitor
RUST_LOG=info
```

### Google Cloud Run

```bash
# Cloud Run 配置
SERVER_HOST=0.0.0.0
SERVER_PORT=8080

# Cloud Storage 或 Cloud Filestore
LINKS_FILE=/mnt/gcs/links.json

# 功能配置
DEFAULT_URL=https://your-gcp-site.appspot.com
RANDOM_CODE_LENGTH=8

# Cloud Logging
RUST_LOG=info
```

## 特殊场景配置

### 高并发场景

```bash
# 服务器配置
SERVER_HOST=0.0.0.0
SERVER_PORT=8080

# 存储优化
LINKS_FILE=/fast-ssd/links.json

# 性能优化
RANDOM_CODE_LENGTH=6  # 平衡性能和唯一性
DEFAULT_URL=https://cdn.example.com

# 日志优化
RUST_LOG=error  # 只记录错误，减少 I/O
```

### 内网部署

```bash
# 内网配置
SERVER_HOST=192.168.1.100
SERVER_PORT=8080

# 内网存储
LINKS_FILE=/shared/nfs/links.json

# 内网地址
DEFAULT_URL=https://intranet.company.com

# 配置
RANDOM_CODE_LENGTH=6
RUST_LOG=info
```

### 微服务环境

```bash
# Kubernetes 配置
SERVER_HOST=0.0.0.0
SERVER_PORT=8080

# PVC 存储
LINKS_FILE=/data/links.json

# 服务网格配置
DEFAULT_URL=https://api.company.com/home

# 配置
RANDOM_CODE_LENGTH=8
RUST_LOG=info
```

## systemd 服务配置

### 服务文件中的环境变量

```ini
[Unit]
Description=Shortlinker Service
After=network.target

[Service]
Type=simple
User=www-data
WorkingDirectory=/opt/shortlinker
ExecStart=/opt/shortlinker/shortlinker

# 生产环境变量
Environment=SERVER_HOST=127.0.0.1
Environment=SERVER_PORT=8080
Environment=LINKS_FILE=/opt/shortlinker/data/links.json
Environment=DEFAULT_URL=https://example.com
Environment=RANDOM_CODE_LENGTH=8
Environment=RUST_LOG=info

# 安全配置
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ReadWritePaths=/opt/shortlinker/data

[Install]
WantedBy=multi-user.target
```

## 配置验证

### 验证脚本

```bash
#!/bin/bash
# validate-config.sh - 配置验证脚本

echo "验证 Shortlinker 配置..."

# 检查必要的环境变量
if [ -z "$LINKS_FILE" ]; then
    echo "警告: LINKS_FILE 未设置，将使用默认值"
fi

# 检查端口是否可用
if netstat -tuln | grep -q ":${SERVER_PORT:-8080} "; then
    echo "错误: 端口 ${SERVER_PORT:-8080} 已被占用"
    exit 1
fi

# 检查存储目录权限
STORAGE_DIR=$(dirname "${LINKS_FILE:-links.json}")
if [ ! -w "$STORAGE_DIR" ]; then
    echo "错误: 存储目录 $STORAGE_DIR 没有写权限"
    exit 1
fi

echo "配置验证通过 ✓"
```

### 配置测试

```bash
# 测试配置是否正确
./shortlinker --version  # 检查程序是否能正常启动
curl -I http://localhost:8080/  # 检查服务是否响应
```
