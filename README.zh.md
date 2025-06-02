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
- ⏰ **过期时间**：支持灵活的时间格式设置（v0.1.1+）
- 💾 **多后端存储**：支持 SQLite 数据库、JSON 文件存储
- 🔄 **跨平台**：支持 Windows、Linux、macOS
- 🛡️ **Admin API**：HTTP API 管理接口（v0.0.5+）
- 🏥 **健康监控**：内置健康检查端点
- 🐳 **容器化**：优化的 Docker 镜像部署
- 🎨 **美观 CLI**：彩色命令行界面
- 🔌 **Unix 套接字**：支持 Unix 套接字绑定

## 快速开始

### 本地运行

```bash
git clone https://github.com/AptS-1547/shortlinker
cd shortlinker
cargo run
```

### Docker 部署

```bash
# TCP 端口
docker run -d -p 8080:8080 -v $(pwd)/data:/data e1saps/shortlinker

# Unix 套接字
docker run -d -v $(pwd)/data:/data -v $(pwd)/sock:/sock \
  -e UNIX_SOCKET=/sock/shortlinker.sock e1saps/shortlinker
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

# 使用相对时间格式（v0.1.1+）
./shortlinker add daily https://example.com --expire 1d      # 1天后过期
./shortlinker add weekly https://example.com --expire 1w     # 1周后过期
./shortlinker add complex https://example.com --expire 1d2h30m  # 复杂格式

# 管理短链
./shortlinker update github https://new-github.com --expire 30d
./shortlinker list                    # 列出所有
./shortlinker remove github           # 删除指定

# 服务器控制
./shortlinker start                   # 启动服务器
./shortlinker stop                    # 停止服务器
./shortlinker restart                 # 重启服务器
```

## Admin API (v0.0.5+)

通过 HTTP API 管理短链接，使用 Bearer 令牌认证。

### 设置

```bash
export ADMIN_TOKEN=your_secret_token
export ADMIN_ROUTE_PREFIX=/admin  # 可选
```

### 示例

```bash
# 获取所有链接
curl -H "Authorization: Bearer your_secret_token" \
     http://localhost:8080/admin/link

# 使用相对时间创建链接
curl -X POST \
     -H "Authorization: Bearer your_secret_token" \
     -H "Content-Type: application/json" \
     -d '{"code":"github","target":"https://github.com","expires_at":"7d"}' \
     http://localhost:8080/admin/link

# 自动生成随机短码
curl -X POST \
     -H "Authorization: Bearer your_secret_token" \
     -H "Content-Type: application/json" \
     -d '{"target":"https://github.com","expires_at":"30d"}' \
     http://localhost:8080/admin/link

# 更新链接
curl -X PUT \
     -H "Authorization: Bearer your_secret_token" \
     -H "Content-Type: application/json" \
     -d '{"target":"https://new-url.com"}' \
     http://localhost:8080/admin/link/github

# 删除链接
curl -X DELETE \
     -H "Authorization: Bearer your_secret_token" \
     http://localhost:8080/admin/link/github
```

## 健康检查 API

监控服务健康状态和存储状态。

```bash
# 设置
export HEALTH_TOKEN=your_health_token

# 健康检查
curl -H "Authorization: Bearer your_health_token" \
     http://localhost:8080/health

# 就绪检查
curl http://localhost:8080/health/ready

# 活跃性检查
curl http://localhost:8080/health/live
```

## 时间格式支持（v0.1.1+）

### 相对时间格式（推荐）
```bash
1s, 5m, 2h, 1d, 1w, 1M, 1y    # 单个单位
1d2h30m                        # 组合格式
```

### RFC3339 格式
```bash
2024-12-31T23:59:59Z           # UTC 时间
2024-12-31T23:59:59+08:00      # 带时区
```

## 配置选项

通过环境变量或 `.env` 文件配置：

| 环境变量 | 默认值 | 说明 |
|----------|--------|------|
| `SERVER_HOST` | `127.0.0.1` | 监听地址 |
| `SERVER_PORT` | `8080` | 监听端口 |
| `UNIX_SOCKET` | *(空)* | Unix 套接字路径（设置后忽略 HOST/PORT） |
| `STORAGE_BACKEND` | `sqlite` | 存储类型 (sqlite/file) |
| `DB_FILE_NAME` | `links.db` | 数据库文件路径 |
| `DEFAULT_URL` | `https://esap.cc/repo` | 默认跳转地址 |
| `RANDOM_CODE_LENGTH` | `6` | 随机码长度 |
| `ADMIN_TOKEN` | *(空)* | Admin API 令牌 |
| `HEALTH_TOKEN` | *(空)* | 健康检查 API 令牌 |
| `RUST_LOG` | `info` | 日志级别 |

### .env 示例

```bash
# 服务器 - TCP
SERVER_HOST=0.0.0.0
SERVER_PORT=8080

# 服务器 - Unix 套接字
# UNIX_SOCKET=/tmp/shortlinker.sock

# 存储
STORAGE_BACKEND=sqlite
DB_FILE_NAME=data/links.db

# API
ADMIN_TOKEN=your_admin_token
HEALTH_TOKEN=your_health_token

# 功能
DEFAULT_URL=https://example.com
RANDOM_CODE_LENGTH=8
RUST_LOG=info
```

## 存储后端

- **SQLite**（默认，v0.1.0+）：生产就绪，推荐使用
- **文件存储**：基于 JSON 的简单存储，适合开发

```bash
# SQLite（推荐）
STORAGE_BACKEND=sqlite
DB_FILE_NAME=links.db

# 文件存储
STORAGE_BACKEND=file
DB_FILE_NAME=links.json
```

## 部署配置

### 反向代理（Nginx）

```nginx
# TCP 端口
server {
    listen 80;
    server_name esap.cc;
    location / {
        proxy_pass http://127.0.0.1:8080;
        add_header Cache-Control "no-cache, no-store, must-revalidate";
    }
}

# Unix 套接字
server {
    listen 80;
    server_name esap.cc;
    location / {
        proxy_pass http://unix:/tmp/shortlinker.sock;
        add_header Cache-Control "no-cache, no-store, must-revalidate";
    }
}
```

### systemd 服务

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
Environment=SERVER_HOST=127.0.0.1
Environment=SERVER_PORT=8080

[Install]
WantedBy=multi-user.target
```

## 开发

```bash
# 开发编译
cargo run

# 生产编译
cargo build --release

# 运行测试
cargo test

# 代码质量
cargo fmt && cargo clippy
```

## 技术亮点

- **跨平台进程管理**：智能锁文件和信号处理
- **热配置重载**：基于信号的重载（Unix）和文件触发（Windows）
- **容器感知**：对 Docker 环境的特殊处理
- **统一错误处理**：完整的错误类型系统，支持自动转换
- **内存安全**：零成本抽象，保证线程安全
- **高测试覆盖**：全面的单元测试和集成测试

## 许可证

MIT License © AptS:1547