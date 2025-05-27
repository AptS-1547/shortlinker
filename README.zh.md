# shortlinker

一个极简主义的短链接服务，支持 HTTP 302 跳转，使用 Rust 编写，部署便捷、响应快速。

## ✨ 功能特性

- 🚀 **高性能**：基于 Rust + Actix-web 构建
- 🎯 **动态管理**：支持运行时添加/删除短链，无需重启
- 🎲 **智能短码**：支持自定义短码和随机生成
- 💾 **持久化存储**：JSON 文件存储，支持热重载
- 🔄 **跨平台**：支持 Windows、Linux、macOS
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

# 管理短链
./shortlinker list                    # 列出所有
./shortlinker remove github           # 删除指定
```

## 配置选项

| 环境变量 | 默认值 | 说明 |
|----------|--------|------|
| `SERVER_HOST` | `127.0.0.1` | 监听地址 |
| `SERVER_PORT` | `8080` | 监听端口 |
| `LINKS_FILE` | `links.json` | 存储文件 |
| `RANDOM_CODE_LENGTH` | `6` | 随机码长度 |
| `RUST_LOG` | `info` | 日志级别 |

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
        proxy_set_header Host $host;
    }
}
```

## 技术实现

- **热重载**：Unix 信号（SIGUSR1）/ Windows 文件监听
- **随机码**：字母数字混合，可配置长度
- **冲突处理**：智能检测，支持强制覆盖
- **容器优化**：多阶段构建，scratch 基础镜像

## 开发

```bash
# 开发编译
cargo run

# 生产编译
cargo build --release

# 交叉编译（需要 cross）
cross build --release --target x86_64-unknown-linux-musl
```

## 许可证

MIT License © AptS:1547
