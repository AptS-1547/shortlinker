# shortlinker

一个极简主义的短链接服务，支持 HTTP 302 跳转，使用 Rust 编写，部署便捷、响应快速，适用于自建短链系统。

## ✨ 项目亮点

- 🚀 **高性能**：基于 Rust + Actix-web 构建，速度与安全性并存
- 🔗 **302 跳转**：临时性重定向，适用于点击追踪、平台导流等场景
- 🎯 **动态管理**：支持运行时添加/删除短链，无需重启服务
- 💾 **持久化存储**：使用 JSON 文件存储，支持配置热重载
- 🐳 **容器化部署**：提供优化的 Docker 镜像，支持多平台
- 🔄 **跨平台兼容**：支持 Windows、Linux、macOS 平台

## 示例使用（绑定域名）

本项目推荐通过自有域名部署，例如绑定 `esap.cc`，用户可访问以下形式的短链：

- `https://esap.cc/github` → 跳转至 GitHub
- `https://esap.cc/blog` → 跳转至个人博客
- `https://esap.cc/` → 跳转至默认主页 (https://www.esaps.net/)

## 快速开始

### 本地运行

```bash
git clone https://github.com/AptS-1547/shortlinker
cd shortlinker
cargo run
```

### Docker 部署

```bash
# 构建镜像
docker build -t shortlinker .

# 运行容器
docker run -d -p 8080:8080 -v $(pwd)/data:/data shortlinker

# 或使用 docker-compose
docker-compose up -d
```

默认监听在 0.0.0.0:8080，你可以通过配置反向代理（如 Caddy/Nginx）绑定域名并启用 HTTPS（推荐）。

## 短链管理

### 添加短链

```bash
# 添加新的短链接
./shortlinker add github https://github.com
./shortlinker add blog https://blog.example.com
```

### 删除短链

```bash
# 删除指定的短链接
./shortlinker remove github
```

### 查看所有短链

```bash
# 列出所有短链接
./shortlinker list
```

## 配置说明

通过环境变量配置服务：

```bash
# 服务器配置
export SERVER_HOST=0.0.0.0          # 监听地址
export SERVER_PORT=8080              # 监听端口
export LINKS_FILE=links.json         # 短链存储文件
export RUST_LOG=info                 # 日志级别
```

或使用 `.env` 文件：

```env
SERVER_HOST=0.0.0.0
SERVER_PORT=8080
LINKS_FILE=links.json
RUST_LOG=info
```

## 部署示例

### Caddy（推荐）

使用 Caddy 自动启用 HTTPS：

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
    return 301 https://$server_name$request_uri;
}

server {
    listen 443 ssl http2;
    server_name esap.cc;
    
    ssl_certificate /path/to/cert.pem;
    ssl_certificate_key /path/to/key.pem;
    
    location / {
        proxy_pass http://127.0.0.1:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

## 项目结构

```
shortlinker/
├── Cargo.toml              # 项目依赖
├── Dockerfile              # Docker 构建文件
├── docker-compose.yml      # Docker Compose 配置
├── nginx.conf              # Nginx 示例配置
├── build.rs                # 构建脚本
└── src/
    └── main.rs             # 主程序文件
```

## 技术特性

- **信号处理**：Unix 系统支持 SIGUSR1 信号热重载
- **文件监听**：Windows 系统使用文件监听机制
- **多阶段构建**：Docker 镜像优化，支持 scratch 基础镜像
- **健康检查**：内置容器健康检查
- **日志记录**：结构化日志输出

## API 说明

当前为单机版本，主要通过命令行管理。计划中的功能：

- RESTful API 接口
- Web 管理界面
- 点击统计功能
- 随机短码生成

## 开发

### 编译

```bash
# 开发模式
cargo run

# 发布模式
cargo build --release

# 交叉编译（需要安装 cross）
cross build --release --target x86_64-unknown-linux-musl
```

### 测试

```bash
cargo test
```

## 许可证

MIT License © AptS:1547
