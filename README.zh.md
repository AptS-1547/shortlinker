# shortlinker

<div align="center">

[![GitHub release (latest by date)](https://img.shields.io/github/v/release/AptS-1547/shortlinker)](https://github.com/AptS-1547/shortlinker/releases)
[![Rust Release](https://img.shields.io/github/actions/workflow/status/AptS-1547/shortlinker/rust-release.yml?label=rust%20release)](https://github.com/AptS-1547/shortlinker/actions/workflows/rust-release.yml)
[![Docker Build](https://img.shields.io/github/actions/workflow/status/AptS-1547/shortlinker/docker-image.yml?label=docker%20build)](https://github.com/AptS-1547/shortlinker/actions/workflows/docker-image.yml)
[![CodeFactor](https://www.codefactor.io/repository/github/apts-1547/shortlinker/badge)](https://www.codefactor.io/repository/github/apts-1547/shortlinker)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Docker Pulls](https://img.shields.io/docker/pulls/e1saps/shortlinker)](https://hub.docker.com/r/e1saps/shortlinker)

**一个极简主义的短链接服务，支持 HTTP 302 跳转，使用 Rust 编写，部署便捷、响应快速。**

[English](README.md) • [中文](README.zh.md)

</div>

## ✨ 功能特性

- 🚀 **高性能**：基于 Rust + Actix-web 构建
- 🎯 **动态管理**：支持运行时添加/删除短链，无需重启
- 🎲 **智能短码**：支持自定义短码和随机生成
- ⏰ **过期时间**：支持设置链接过期时间，自动失效
- 💾 **多后端存储**：支持 SQLite 数据库、JSON 文件存储和 Sled 嵌入式数据库
- 🔄 **跨平台**：支持 Windows、Linux、macOS
- 🔐 **进程管理**：智能进程锁，防止重复启动
- 🐳 **容器化**：优化的 Docker 镜像部署
- 🛡️ **Admin API**：HTTP API 管理接口（v0.0.5+）

## 快速开始

### 本地运行

```bash
git clone https://github.com/AptS-1547/shortlinker
cd shortlinker
cargo run
```

### Docker 部署

```bash
# 从 Docker Hub 拉取
docker run -d -p 8080:8080 -v $(pwd)/data:/data e1saps/shortlinker

# 或从 GitHub Container Registry 拉取
docker run -d -p 8080:8080 -v $(pwd)/data:/data ghcr.io/apts-1547/shortlinker

# 自己构建
docker build -t shortlinker .
docker run -d -p 8080:8080 -v $(pwd)/data:/data shortlinker
```

## 使用示例

绑定域名后（如 `esap.cc`），可访问：

- `https://esap.cc/github` → 自定义短链
- `https://esap.cc/aB3dF1` → 随机短链
- `https://esap.cc/` → 默认主页

## 命令行管理

```bash
# 启动服务器
./shortlinker

# 添加短链
./shortlinker add github https://github.com           # 自定义短码
./shortlinker add https://github.com                  # 随机短码
./shortlinker add github https://new-url.com --force  # 强制覆盖
./shortlinker add temp https://example.com --expires "2025-12-31T23:59:59Z"  # 带过期时间

# 管理短链
./shortlinker list                    # 列出所有
./shortlinker remove github           # 删除指定
```

## Admin API (v0.0.5+)

从 v0.0.5 版本开始，支持通过 HTTP API 管理短链接。

### 鉴权设置

```bash
# 设置 Admin Token
export ADMIN_TOKEN=your_secret_token

# 自定义 Admin 路由前缀（可选，默认为 /admin）
export ADMIN_ROUTE_PREFIX=/api/admin
```

### API 接口

#### 获取所有短链接
```bash
curl -H "Authorization: Bearer your_secret_token" \
     http://localhost:8080/admin/link
```

#### 创建短链接
```bash
curl -X POST \
     -H "Authorization: Bearer your_secret_token" \
     -H "Content-Type: application/json" \
     -d '{"code":"github","target":"https://github.com"}' \
     http://localhost:8080/admin/link
```

#### 获取指定短链接
```bash
curl -H "Authorization: Bearer your_secret_token" \
     http://localhost:8080/admin/link/github
```

#### 更新短链接
```bash
curl -X PUT \
     -H "Authorization: Bearer your_secret_token" \
     -H "Content-Type: application/json" \
     -d '{"code":"github","target":"https://github.com/new"}' \
     http://localhost:8080/admin/link/github
```

#### 删除短链接
```bash
curl -X DELETE \
     -H "Authorization: Bearer your_secret_token" \
     http://localhost:8080/admin/link/github
```

## 存储后端

shortlinker 支持多种存储后端，可根据需求选择合适的存储方式。

### SQLite 数据库存储（默认）

使用 SQLite 轻量级关系数据库存储，提供最佳的性能和可靠性。

**优点**：
- 高性能 SQL 查询
- ACID 事务支持
- 成熟稳定，生产环境验证
- 支持并发读取
- 数据完整性保证
- 轻量级，无需额外服务

**缺点**：
- 数据不可直接编辑（需要 SQL 工具）
- 高并发写入有限制

**配置**：
```bash
# 默认使用 SQLite 存储，无需额外配置
STORAGE_TYPE=sqlite        # 可选，默认为 sqlite
SQLITE_DB_PATH=links.db    # 数据库文件路径
```

### 文件存储

使用 JSON 文件存储短链接数据，简单易用，便于备份和迁移。

**优点**：
- 配置简单，无需额外依赖
- 数据可读性好，便于调试
- 支持热重载
- 便于备份和版本控制

**缺点**：
- 高并发写入性能相对较低
- 大量数据时加载较慢
- 无事务支持

**配置**：
```bash
STORAGE_TYPE=file          # 启用文件存储
LINKS_FILE=links.json      # 存储文件路径
```

### Sled 数据库存储

使用 Sled 嵌入式数据库存储，提供高并发性能。

**优点**：
- 高并发读写性能
- 内置事务支持
- 数据压缩，占用空间小
- 崩溃恢复能力强

**缺点**：
- 数据不可直接编辑
- 相对占用更多内存
- 较新的技术，生态不如 SQLite 成熟

**配置**：
```bash
STORAGE_TYPE=sled          # 启用 Sled 存储
SLED_DB_PATH=links.sled    # 数据库文件路径
```

### 选择建议

- **生产环境**：推荐使用 SQLite 存储（默认）
- **高并发场景**：推荐使用 SQLite 或 Sled 存储
- **小规模部署**（< 1,000 链接）：任何存储都可以
- **中大规模部署**（> 10,000 链接）：推荐使用 SQLite 存储
- **需要频繁备份**：推荐使用文件存储
- **开发调试**：推荐使用文件存储

## 配置选项

可以通过环境变量或 `.env` 文件进行配置。程序会自动读取项目根目录下的 `.env` 文件。

| 环境变量 | 默认值 | 说明 |
|----------|--------|------|
| `SERVER_HOST` | `127.0.0.1` | 监听地址 |
| `SERVER_PORT` | `8080` | 监听端口 |
| `STORAGE_TYPE` | `sqlite` | 存储后端类型（`sqlite`、`file` 或 `sled`） |
| `SQLITE_DB_PATH` | `links.db` | SQLite 数据库路径（仅 SQLite 存储） |
| `LINKS_FILE` | `links.json` | 文件存储路径（仅文件存储） |
| `SLED_DB_PATH` | `links.sled` | Sled 数据库路径（仅 Sled 存储） |
| `DEFAULT_URL` | `https://esap.cc/repo` | 根路径默认跳转地址 |
| `RANDOM_CODE_LENGTH` | `6` | 随机码长度 |
| `RUST_LOG` | `info` | 日志级别 (`error`, `warn`, `info`, `debug`, `trace`) |
| `ADMIN_TOKEN` | *(空字符串)* | Admin API 鉴权令牌，为空时禁用 Admin API (v0.0.5+) |
| `ADMIN_ROUTE_PREFIX` | `/admin` | Admin API 路由前缀 (v0.0.5+) |

### .env 文件示例

在项目根目录创建 `.env` 文件：

```bash
# 服务器配置
SERVER_HOST=0.0.0.0
SERVER_PORT=8080

# 存储配置 - 选择其中一种
# SQLite 存储（默认）
STORAGE_TYPE=sqlite
SQLITE_DB_PATH=data/links.db

# 或者使用文件存储
# STORAGE_TYPE=file
# LINKS_FILE=data/links.json

# 或者使用 Sled 存储
# STORAGE_TYPE=sled
# SLED_DB_PATH=data/links.sled

# 默认跳转地址
DEFAULT_URL=https://example.com

# 随机码长度
RANDOM_CODE_LENGTH=8

# 日志级别
RUST_LOG=debug

# Admin API 配置 (v0.0.5+)
ADMIN_TOKEN=your_secure_admin_token
ADMIN_ROUTE_PREFIX=/api/admin
```

## 服务器管理

### 启动和停止

```bash
# 启动服务器
./shortlinker

# 停止服务器
kill $(cat shortlinker.pid)
```

### 进程保护

- **Unix 系统**：使用 PID 文件 (`shortlinker.pid`) 防止重复启动
- **Windows 系统**：使用锁文件 (`.shortlinker.lock`) 防止重复启动
- 程序会自动检测已运行的实例并给出提示

## 数据格式

链接数据存储在 JSON 文件中，格式如下：

```json
{
  "github": {
    "target": "https://github.com",
    "created_at": "2024-01-01T00:00:00Z",
    "expires_at": null
  },
  "temp": {
    "target": "https://example.com",
    "created_at": "2024-01-01T00:00:00Z",
    "expires_at": "2024-12-31T23:59:59Z"
  }
}
```

## 部署配置

### Caddy

```caddy
esap.cc {
    reverse_proxy 127.0.0.1:8080
    
    # 可选：添加缓存控制
    header {
        Cache-Control "no-cache, no-store, must-revalidate"
    }
}
```

### Nginx

```nginx
server {
    listen 80;
    server_name esap.cc;
    
    location / {
        proxy_pass http://127.0.0.1:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        
        # 禁用缓存
        add_header Cache-Control "no-cache, no-store, must-revalidate";
    }
}
```

### 系统服务 (systemd)

```ini
[Unit]
Description=ShortLinker Service
After=network.target

[Service]
Type=simple
User=www-data
WorkingDirectory=/opt/shortlinker
ExecStart=/opt/shortlinker/shortlinker
Restart=always
RestartSec=5

Environment=SERVER_HOST=127.0.0.1
Environment=SERVER_PORT=8080
Environment=RUST_LOG=info

[Install]
WantedBy=multi-user.target
```

## 技术实现

- **热重载**：配置文件变更自动检测
- **随机码**：字母数字混合，可配置长度，避免冲突
- **冲突处理**：智能检测，支持强制覆盖
- **过期检查**：请求时实时检查，自动清理过期链接
- **容器优化**：多阶段构建，scratch 基础镜像
- **内存安全**：Arc + RwLock 保证并发安全

## 开发

```bash
# 开发编译
cargo run

# 生产编译
cargo build --release

# 交叉编译（需要 cross）
cross build --release --target x86_64-unknown-linux-musl

# 运行测试
cargo test

# 检查代码格式
cargo fmt
cargo clippy
```

## 性能优化

- 使用 `Arc<RwLock<HashMap>>` 实现高并发读取
- 302 临时重定向，避免浏览器缓存
- 最小化内存占用和 CPU 使用
- 异步 I/O 处理，支持高并发

## 数据迁移

### 从文件存储迁移到 SQLite

```bash
# 1. 停止服务
./shortlinker stop

# 2. 备份现有数据
cp links.json links.json.backup

# 3. 修改配置
export STORAGE_TYPE=sqlite
export SQLITE_DB_PATH=links.db

# 4. 启动服务（会自动从文件加载数据到 SQLite）
./shortlinker
```

### 从 Sled 迁移到 SQLite

```bash
# 1. 导出数据（通过 Admin API）
curl -H "Authorization: Bearer your_token" \
     http://localhost:8080/admin/link > links_export.json

# 2. 停止服务
./shortlinker stop

# 3. 修改配置
export STORAGE_TYPE=sqlite
export SQLITE_DB_PATH=links.db

# 4. 转换数据格式并启动服务
./shortlinker import links_export.json
```

### 从 SQLite 迁移到文件存储

```bash
# 1. 导出数据（通过 Admin API）
curl -H "Authorization: Bearer your_token" \
     http://localhost:8080/admin/link > links_export.json

# 2. 停止服务
./shortlinker stop

# 3. 修改配置
export STORAGE_TYPE=file
export LINKS_FILE=links.json

# 4. 转换数据格式并启动服务
./shortlinker import links_export.json
```

## 故障排除

### 常见问题

1. **端口被占用**
   ```bash
   # 查看端口占用
   lsof -i :8080
   netstat -tlnp | grep 8080
   ```

2. **权限问题**
   ```bash
   # 确保有写入权限
   chmod 755 /path/to/shortlinker
   chown user:group links.json
   ```

3. **配置文件损坏（文件存储）**
   ```bash
   # 验证 JSON 格式
   jq . links.json
   ```

4. **SQLite 数据库问题**
   ```bash
   # 检查数据库文件权限
   ls -la links.db
   
   # 使用 sqlite3 工具检查数据库
   sqlite3 links.db ".tables"
   sqlite3 links.db "SELECT COUNT(*) FROM links;"
   ```

5. **Sled 数据库锁定**
   ```bash
   # 检查是否有其他进程占用数据库
   ps aux | grep shortlinker
   
   # 如果确认没有其他进程，可以尝试删除锁文件
   rm -rf links.sled/db
   ```

6. **存储后端切换问题**
   ```bash
   # 确保配置正确
   echo $STORAGE_TYPE
   
   # 检查文件权限
   ls -la links.json links.db links.sled/
   ```

## 许可证

MIT License © AptS:1547
