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
- 💾 **多后端存储**：支持 SQLite 数据库、JSON 文件存储，Sled 嵌入式数据库即将支持
- 🔄 **跨平台**：支持 Windows、Linux、macOS
- 🔐 **进程管理**：智能进程锁，防止重复启动
- 🐳 **容器化**：优化的 Docker 镜像部署
- 🛡️ **Admin API**：HTTP API 管理接口（v0.0.5+）
- 🧪 **高测试覆盖**：全面的单元测试和集成测试覆盖
- 🔧 **强类型安全**：完备的错误处理和类型系统
- 🎨 **彩色输出**：美观的命令行界面，支持彩色输出

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
./shortlinker update github https://new-github.com    # 更新现有链接
./shortlinker list                    # 列出所有
./shortlinker remove github           # 删除指定

# 服务器控制
./shortlinker start                   # 启动服务器
./shortlinker stop                    # 停止服务器
./shortlinker restart                 # 重启服务器
```

## Admin API (v0.0.5+)

从 v0.0.5 版本开始，支持通过 HTTP API 管理短链接。

### 鉴权设置

```bash
# 设置 Admin Token（必需，为空时禁用 API）
export ADMIN_TOKEN=your_secret_token

# 自定义路由前缀（可选）
export ADMIN_ROUTE_PREFIX=/api/admin
```

### API 端点

#### GET /admin/link
获取所有短链接。

```bash
curl -H "Authorization: Bearer your_secret_token" \
     http://localhost:8080/admin/link
```

#### POST /admin/link
创建新的短链接。

```bash
curl -X POST \
     -H "Authorization: Bearer your_secret_token" \
     -H "Content-Type: application/json" \
     -d '{"code":"github","target":"https://github.com","expires_at":"2024-12-31T23:59:59Z"}' \
     http://localhost:8080/admin/link
```

#### GET /admin/link/{code}
获取指定的短链接。

```bash
curl -H "Authorization: Bearer your_secret_token" \
     http://localhost:8080/admin/link/github
```

#### PUT /admin/link/{code}
更新现有的短链接。

```bash
curl -X PUT \
     -H "Authorization: Bearer your_secret_token" \
     -H "Content-Type: application/json" \
     -d '{"target":"https://new-github.com","expires_at":"2025-01-31T23:59:59Z"}' \
     http://localhost:8080/admin/link/github
```

#### DELETE /admin/link/{code}
删除短链接。

```bash
curl -X DELETE \
     -H "Authorization: Bearer your_secret_token" \
     http://localhost:8080/admin/link/github
```

### 常用操作

```bash
# 获取所有短链接
curl -H "Authorization: Bearer your_secret_token" \
     http://localhost:8080/admin/link

# 创建自动生成短码的链接
curl -X POST \
     -H "Authorization: Bearer your_secret_token" \
     -H "Content-Type: application/json" \
     -d '{"target":"https://github.com"}' \
     http://localhost:8080/admin/link

# 更新短链接
curl -X PUT \
     -H "Authorization: Bearer your_secret_token" \
     -H "Content-Type: application/json" \
     -d '{"target":"https://new-url.com"}' \
     http://localhost:8080/admin/link/github

# 删除短链接
curl -X DELETE \
     -H "Authorization: Bearer your_secret_token" \
     http://localhost:8080/admin/link/github
```

## 存储后端

shortlinker 从 v0.1.0 版本开始支持多种存储后端：

- **SQLite**（默认，v0.1.0+）：生产级性能，推荐用于生产环境
- **文件存储**（v0.1.0 前的默认方式）：简单易用，便于调试和备份
- **Sled**（即将支持）：高并发性能，适合高负载场景

```bash
# SQLite 存储（默认，v0.1.0+）
STORAGE_BACKEND=sqlite
DB_FILE_NAME=links.db

# 文件存储（v0.1.0 前的默认方式）
STORAGE_BACKEND=file
DB_FILE_NAME=links.json

# Sled 存储（即将支持）
# STORAGE_BACKEND=sled
# DB_FILE_NAME=links.sled
```

## 配置选项

通过环境变量或 `.env` 文件配置：

| 环境变量 | 默认值 | 说明 |
|----------|--------|------|
| `SERVER_HOST` | `127.0.0.1` | 监听地址 |
| `SERVER_PORT` | `8080` | 监听端口 |
| `STORAGE_BACKEND` | `sqlite` | 存储后端类型 |
| `DB_FILE_NAME` | `links.db`（SQLite），`links.json`（文件），`links.sled`（Sled） | 数据库文件路径（根据后端而定） |
| `DEFAULT_URL` | `https://esap.cc/repo` | 根路径默认跳转地址 |
| `RANDOM_CODE_LENGTH` | `6` | 随机码长度 |
| `ADMIN_TOKEN` | *(空)* | Admin API 鉴权令牌 |
| `RUST_LOG` | `info` | 日志级别 |

### .env 文件示例

```bash
# 服务器配置
SERVER_HOST=0.0.0.0
SERVER_PORT=8080

# 存储配置
STORAGE_BACKEND=sqlite
DB_FILE_NAME=data/links.db

# 功能配置
DEFAULT_URL=https://example.com
RANDOM_CODE_LENGTH=8
RUST_LOG=info

# Admin API 配置
ADMIN_TOKEN=your_secure_admin_token
```

## 部署配置

### Caddy

```caddy
esap.cc {
    reverse_proxy 127.0.0.1:8080
}
```

### Nginx

```nginx
server {
    listen 80;
    server_name esap.cc;
    location / {
        proxy_pass http://127.0.0.1:8080;
        add_header Cache-Control "no-cache, no-store, must-revalidate";
    }
}
```

### systemd

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
Environment=RUST_LOG=info

[Install]
WantedBy=multi-user.target
```

## 代码质量与测试

shortlinker 项目注重代码质量和可靠性：

### 测试覆盖

- **CLI 模块测试**：命令行解析、参数验证、错误处理
- **存储层测试**：文件存储、SQLite、Sled 等多后端测试
- **服务层测试**：Admin API、认证中间件、HTTP 处理
- **工具模块测试**：随机码生成、颜色输出、工具函数
- **错误处理测试**：完整的错误类型和转换测试
- **系统集成测试**：进程管理、信号处理、并发安全
- **性能测试**：大数据量处理、并发操作、内存使用

### 运行测试

```bash
# 运行所有测试
cargo test

# 运行特定模块测试
cargo test cli_tests
cargo test storages_tests
cargo test services_tests
cargo test utils_tests
cargo test errors_tests

# 显示测试覆盖率
cargo test --verbose

# 并行测试（更快）
cargo test -- --test-threads=4
```

### 代码质量特性

- **类型安全**：严格的 Rust 类型系统，编译时错误检查
- **内存安全**：无 GC 的零成本抽象，防止内存泄漏
- **并发安全**：Arc + Mutex/RwLock 保证线程安全
- **错误处理**：统一的错误类型和传播机制
- **模块化设计**：清晰的模块边界和职责分离
- **文档完备**：详细的代码注释和 API 文档

## 技术特性

- **热重载**：配置文件变更自动检测
- **随机码**：字母数字混合，可配置长度，避免冲突
- **过期检查**：请求时实时检查，自动清理过期链接
- **容器优化**：多阶段构建，scratch 基础镜像
- **内存安全**：Arc + RwLock 保证并发安全
- **彩色终端**：支持 ANSI 颜色码的美观输出
- **智能重试**：网络和存储操作的自动重试机制
- **优雅关闭**：信号处理和资源清理

## 开发

```bash
# 开发编译
cargo run

# 生产编译
cargo build --release

# 运行测试
cargo test

# 代码格式化
cargo fmt

# 代码检查
cargo clippy

# 生成文档
cargo doc --open
```

### 开发指南

1. **添加新功能**：确保编写相应的单元测试
2. **修改存储层**：需要更新所有存储后端的实现
3. **API 变更**：更新 Admin API 测试和文档
4. **错误处理**：使用统一的 `ShortlinkerError` 类型
5. **日志输出**：使用 `log` crate 进行结构化日志

## 许可证

MIT License © AptS:1547
