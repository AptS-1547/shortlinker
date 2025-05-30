# 配置示例

各种环境下的完整配置示例。

## 开发环境

```bash
# 服务器配置
SERVER_HOST=127.0.0.1
SERVER_PORT=8080

# 功能配置
DEFAULT_URL=https://localhost:3000
RANDOM_CODE_LENGTH=4  # 短一些便于测试

# 存储配置 - 开发环境推荐文件存储便于调试
STORAGE_TYPE=file
LINKS_FILE=./dev-links.json

# 日志配置
RUST_LOG=debug  # 开发时启用详细日志

# Admin API（开发环境）
ADMIN_TOKEN=dev_token_123
```

## 生产环境

```bash
# 服务器配置
SERVER_HOST=127.0.0.1  # 通过反向代理暴露
SERVER_PORT=8080

# 存储配置 - 生产环境推荐 SQLite
STORAGE_TYPE=sqlite
SQLITE_DB_PATH=/data/links.db

# 功能配置
DEFAULT_URL=https://your-company.com
RANDOM_CODE_LENGTH=8  # 生产环境建议更长

# 安全配置
ADMIN_TOKEN=very_secure_production_token_456
RUST_LOG=info
```

## Docker 环境

### Docker Compose

```yaml
version: '3.8'
services:
  shortlinker:
    image: e1saps/shortlinker
    environment:
      - SERVER_HOST=0.0.0.0
      - SERVER_PORT=8080
      - STORAGE_TYPE=sqlite
      - SQLITE_DB_PATH=/data/links.db
      - DEFAULT_URL=https://your-domain.com
      - RANDOM_CODE_LENGTH=8
      - RUST_LOG=info
    volumes:
      - ./data:/data
    ports:
      - "127.0.0.1:8080:8080"
```

### 环境变量文件

```bash
# .env
SERVER_HOST=0.0.0.0
SERVER_PORT=8080
STORAGE_TYPE=sqlite
SQLITE_DB_PATH=/data/links.db
DEFAULT_URL=https://your-site.com
RANDOM_CODE_LENGTH=8
RUST_LOG=info
```

## 云服务环境

### 通用云配置

```bash
# 服务器配置
SERVER_HOST=0.0.0.0
SERVER_PORT=8080

# 存储配置（使用持久化存储）
STORAGE_TYPE=sqlite
SQLITE_DB_PATH=/mnt/persistent/links.db

# 功能配置
DEFAULT_URL=https://your-cloud-site.com
RANDOM_CODE_LENGTH=8
RUST_LOG=info
```

## 高并发场景

```bash
# 服务器配置
SERVER_HOST=0.0.0.0
SERVER_PORT=8080

# 存储优化 - 使用 SQLite 或 Sled
STORAGE_TYPE=sqlite
SQLITE_DB_PATH=/fast-ssd/links.db

# 性能优化
RANDOM_CODE_LENGTH=6  # 平衡性能和唯一性
DEFAULT_URL=https://cdn.example.com

# 日志优化
RUST_LOG=error  # 只记录错误，减少 I/O
```

## systemd 服务配置

```ini
[Unit]
Description=Shortlinker Service
After=network.target

[Service]
Type=simple
User=www-data
WorkingDirectory=/opt/shortlinker
ExecStart=/opt/shortlinker/shortlinker

# 环境变量
Environment=SERVER_HOST=127.0.0.1
Environment=SERVER_PORT=8080
Environment=STORAGE_TYPE=sqlite
Environment=SQLITE_DB_PATH=/opt/shortlinker/data/links.db
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

### 快速验证脚本

```bash
#!/bin/bash
# validate-config.sh
echo "验证 Shortlinker 配置..."

# 检查端口
if netstat -tuln | grep -q ":${SERVER_PORT:-8080} "; then
    echo "错误: 端口 ${SERVER_PORT:-8080} 已被占用"
    exit 1
fi

# 检查存储目录权限
STORAGE_DIR=$(dirname "${SQLITE_DB_PATH:-links.db}")
if [ ! -w "$STORAGE_DIR" ]; then
    echo "错误: 存储目录 $STORAGE_DIR 没有写权限"
    exit 1
fi

echo "配置验证通过 ✓"
```

### 测试配置

```bash
# 测试启动
./shortlinker --version

# 测试服务响应
curl -I http://localhost:8080/
```
