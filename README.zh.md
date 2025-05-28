# shortlinker

一个极简主义的短链接服务，支持 HTTP 302 跳转，使用 Rust 编写，部署便捷、响应快速。

## ✨ 功能特性

- 🚀 **高性能**：基于 Rust + Actix-web 构建
- 🎯 **动态管理**：支持运行时添加/删除短链，无需重启
- 🎲 **智能短码**：支持自定义短码和随机生成
- ⏰ **过期时间**：支持设置链接过期时间，自动失效
- 💾 **持久化存储**：JSON 文件存储，支持热重载
- 🔄 **跨平台**：支持 Windows、Linux、macOS
- 🔐 **进程管理**：智能进程锁，防止重复启动
- 🐳 **容器化**：优化的 Docker 镜像部署

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

## 配置选项

可以通过环境变量或 `.env` 文件进行配置。程序会自动读取项目根目录下的 `.env` 文件。

| 环境变量 | 默认值 | 说明 |
|----------|--------|------|
| `SERVER_HOST` | `127.0.0.1` | 监听地址 |
| `SERVER_PORT` | `8080` | 监听端口 |
| `LINKS_FILE` | `links.json` | 存储文件路径 |
| `DEFAULT_URL` | `https://esap.cc/repo` | 根路径默认跳转地址 |
| `RANDOM_CODE_LENGTH` | `6` | 随机码长度 |
| `RUST_LOG` | `info` | 日志级别 (`error`, `warn`, `info`, `debug`, `trace`) |

### .env 文件示例

在项目根目录创建 `.env` 文件：

```bash
# 服务器配置
SERVER_HOST=0.0.0.0
SERVER_PORT=8080

# 存储配置
LINKS_FILE=data/links.json

# 默认跳转地址
DEFAULT_URL=https://example.com

# 随机码长度
RANDOM_CODE_LENGTH=8

# 日志级别
RUST_LOG=debug
```

**注意**：环境变量的优先级高于 `.env` 文件中的配置。

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

3. **配置文件损坏**
   ```bash
   # 验证 JSON 格式
   jq . links.json
   ```

## 许可证

MIT License © AptS:1547
