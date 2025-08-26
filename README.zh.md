# shortlinker

<div align="center">

[![GitHub 最新发布](https://img.shields.io/github/v/release/AptS-1547/shortlinker)](https://github.com/AptS-1547/shortlinker/releases)
[![Rust 构建状态](https://img.shields.io/github/actions/workflow/status/AptS-1547/shortlinker/rust-release.yml?label=rust%20release)](https://github.com/AptS-1547/shortlinker/actions/workflows/rust-release.yml)
[![Docker 构建状态](https://img.shields.io/github/actions/workflow/status/AptS-1547/shortlinker/docker-image.yml?label=docker%20build)](https://github.com/AptS-1547/shortlinker/actions/workflows/docker-image.yml)
[![CodeFactor 评分](https://www.codefactor.io/repository/github/apts-1547/shortlinker/badge)](https://www.codefactor.io/repository/github/apts-1547/shortlinker)
[![MIT 协议](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Docker 拉取数](https://img.shields.io/docker/pulls/e1saps/shortlinker)](https://hub.docker.com/r/e1saps/shortlinker)

**一款极简主义的 URL 缩短服务，支持 HTTP 307 重定向，使用 Rust 构建，易于部署，极速响应。**

[English](README.md) • [中文](README.zh.md)

![管理面板界面](assets/admin-panel-dashboard.png)

</div>

## 🚀 性能基准（v0.1.7-alpha.1）

**测试环境**

- 操作系统：Linux
- CPU：12代 Intel Core i5-12500，单核
- 压测工具：[`wrk`](https://github.com/wg/wrk)

| 类型       | 场景                  | QPS 峰值         | 缓存命中 | 布隆过滤器 | 数据库访问 |
|------------|-----------------------|------------------|-----------|--------------|--------------|
| 命中缓存   | 热门链接（重复访问） | **719,997.22**   | ✅ 是     | ✅ 是         | ❌ 否        |
| 未命中缓存 | 冷门链接（随机访问） | **610,543.39**   | ❌ 否     | ✅ 是         | ✅ 是        |

> 💡 即使在缓存未命中时，系统仍能维持近 60 万 QPS，展示了 SQLite + actix-web + 异步缓存的卓越性能。

---

## ✨ 功能亮点

- 🚀 **高性能**：Rust + actix-web 构建
- 🔧 **运行时动态管理**：添加/删除链接无需重启服务
- 🎲 **智能短码生成**：支持自定义和随机短码
- ⏰ **支持过期时间**：灵活设置链接有效期（v0.1.1+）
- 💾 **多种存储后端**：SQLite、JSON 文件
- 🖥️ **跨平台支持**：Linux、Windows、macOS
- 🛡️ **管理 API**：支持 Bearer Token 的 HTTP API（v0.0.5+）
- 💉 **健康检查 API**：服务存活与就绪检查接口
- 🐳 **Docker 镜像**：适配容器部署，体积小巧
- 🎨 **美观 CLI**：带有颜色高亮的命令行工具
- 🔌 **Unix Socket 支持**

---

## 🚀 快速开始

### 本地运行

```bash
git clone https://github.com/AptS-1547/shortlinker
cd shortlinker
cargo run
````

### Docker 部署

```bash
# TCP 端口模式
docker run -d -p 8080:8080 -v $(pwd)/data:/data e1saps/shortlinker

# Unix Socket 模式
docker run -d -v $(pwd)/data:/data -v $(pwd)/sock:/sock \
  -e UNIX_SOCKET=/sock/shortlinker.sock e1saps/shortlinker
```

---

## 🧪 使用示例

域名绑定后（如 `https://esap.cc`）：

* `https://esap.cc/github` → 自定义短链接
* `https://esap.cc/aB3dF1` → 随机短链接
* `https://esap.cc/` → 默认首页跳转

---

## 🔧 命令行管理示例

```bash
# 启动服务
./shortlinker

# 添加链接
./shortlinker add github https://github.com             # 自定义短码
./shortlinker add https://github.com                    # 随机短码
./shortlinker add github https://new-url.com --force    # 覆盖已有短码

# 设置相对时间（v0.1.1+）
./shortlinker add daily https://example.com --expire 1d
./shortlinker add weekly https://example.com --expire 1w
./shortlinker add complex https://example.com --expire 1d2h30m

# 管理操作
./shortlinker update github https://new-github.com --expire 30d
./shortlinker list
./shortlinker remove github

# 服务控制
./shortlinker start
./shortlinker stop
./shortlinker restart
```

---

## 🔐 管理 API（v0.0.5+）

启用管理功能：

```bash
export ADMIN_TOKEN=你的管理密钥
export ADMIN_ROUTE_PREFIX=/admin  # 可选前缀
```

### API 示例

```bash
# 获取所有链接
curl -H "Authorization: Bearer 你的管理密钥" http://localhost:8080/admin/link

# 创建链接
curl -X POST \
     -H "Authorization: Bearer 你的管理密钥" \
     -H "Content-Type: application/json" \
     -d '{"code":"github","target":"https://github.com","expires_at":"7d"}' \
     http://localhost:8080/admin/link
```

---

## ❤️ 健康检查

```bash
export HEALTH_TOKEN=你的健康密钥

# 总体健康检查
curl -H "Authorization: Bearer $HEALTH_TOKEN" http://localhost:8080/health

# 就绪检查
curl http://localhost:8080/health/ready

# 存活检查
curl http://localhost:8080/health/live
```

---

## 🕒 时间格式支持

### 相对时间（推荐）

```bash
1s, 5m, 2h, 1d, 1w, 1M, 1y
1d2h30m  # 组合时间格式
```

### 绝对时间（RFC3339）

```bash
2024-12-31T23:59:59Z
2024-12-31T23:59:59+08:00
```

---

## ⚙️ 配置方式

**shortlinker 现在支持 TOML 配置文件！**

支持 TOML 配置文件和环境变量两种方式，TOML 配置更清晰易读，推荐使用。

### TOML 配置文件

创建 `config.toml` 文件：

```toml
[server]
host = "0.0.0.0"
port = 8080
# unix_socket = "/tmp/shortlinker.sock"  # 可选：Unix Socket
cpu_count = 4

[storage]
backend = "sqlite"
database_url = "data/links.db"
# db_file_name = "links.json"  # 仅当 backend = "file" 时使用

[cache]
redis_url = "redis://127.0.0.1:6379/"
redis_key_prefix = "shortlinker:"
redis_ttl = 3600

[api]
admin_token = "your_admin_token"
health_token = "your_health_token"

[routes]
admin_prefix = "/admin"
health_prefix = "/health"
frontend_prefix = "/panel"

[features]
enable_admin_panel = false
random_code_length = 8
default_url = "https://example.com"

[logging]
level = "info"
```

配置文件查找顺序：
1. `config.toml`
2. `shortlinker.toml`  
3. `config/config.toml`
4. `/etc/shortlinker/config.toml`

### 环境变量（向后兼容）

仍然支持原有的环境变量配置方式，环境变量会覆盖 TOML 配置：

| 变量                      | 默认值                                          | 说明                 |
| ----------------------- | -------------------------------------------- | ------------------ |
| SERVER\_HOST            | 127.0.0.1                                    | 监听地址               |
| SERVER\_PORT            | 8080                                         | 监听端口               |
| UNIX\_SOCKET            | 空                                            | 使用 Unix Socket 时填写 |
| CPU\_COUNT              | 自动                                           | 工作线程数              |
| STORAGE\_BACKEND        | sqlite                                       | 存储方式（sqlite/file）  |
| DATABASE\_URL           | shortlinks.db                                | 数据库 URL            |
| DB\_FILE\_NAME          | links.json                                   | JSON 文件路径         |
| REDIS\_URL              | redis://127.0.0.1:6379/                     | Redis 连接地址        |
| REDIS\_KEY\_PREFIX      | shortlinker:                                 | Redis 键前缀          |
| REDIS\_TTL              | 3600                                         | Redis TTL(秒)       |
| DEFAULT\_URL            | https://esap.cc/repo                         | 默认跳转 URL           |
| RANDOM\_CODE\_LENGTH    | 6                                            | 随机短码长度             |
| ADMIN\_TOKEN            | 空                                            | 管理 API 密钥          |
| HEALTH\_TOKEN           | 空                                            | 健康检查密钥             |
| ADMIN\_ROUTE\_PREFIX    | /admin                                       | 管理 API 路由前缀       |
| HEALTH\_ROUTE\_PREFIX   | /health                                      | 健康检查路由前缀           |
| ENABLE\_ADMIN\_PANEL    | false                                        | 启用网页管理面板（实验性）      |
| FRONTEND\_ROUTE\_PREFIX | /panel                                       | 面板路由前缀             |
| RUST\_LOG               | info                                         | 日志等级               |

---

## 📦 存储选项

* SQLite（推荐）：稳定、支持高并发
* 文件（JSON）：适合开发测试

---

## 📡 部署示例

### Nginx 反向代理

```nginx
server {
    listen 80;
    server_name esap.cc;
    location / {
        proxy_pass http://127.0.0.1:8080;
    }
}
```

### systemd 服务

```ini
[Unit]
Description=ShortLinker 服务
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

---

## 🔧 开发者指南

```bash
cargo run           # 开发运行
cargo build --release  # 生产构建
cargo test          # 运行测试
cargo fmt && cargo clippy  # 格式化与静态检查
```

---

## 🧩 相关模块

* Web 管理面板：`admin-panel/`
* Cloudflare Worker：无服务器版，位于 `cf-worker/`

---

## 📜 协议

MIT License © AptS:1547

<pre>
        ／＞　 フ
       | 　_　_|    AptS:1547
     ／` ミ＿xノ    — shortlinker assistant bot —
    /　　　　 |
   /　 ヽ　　 ﾉ      Rust / SQLite / Bloom / CLI
   │　　|　|　|
／￣|　　 |　|　|
(￣ヽ＿_ヽ_)__)
＼二)

   「ready to 307 !」
</pre>

> [🔗 Visit Project Docs](https://esap.cc/docs)
> [💬 Powered by AptS:1547](https://github.com/AptS-1547)
